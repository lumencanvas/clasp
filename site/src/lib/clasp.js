/**
 * CLASP Client - Local implementation for the playground
 * Matches the current server protocol (v1, subprotocol "clasp")
 */

import { encode as msgpackEncode, decode as msgpackDecode } from '@msgpack/msgpack'

// Protocol constants - match the Rust server
const PROTOCOL_VERSION = 1
const WS_SUBPROTOCOL = 'clasp'
const MAGIC = 0x53 // 'S'
const HEADER_SIZE = 4
const HEADER_SIZE_WITH_TS = 12

// QoS levels
export const QoS = {
  Fire: 0,
  Confirm: 1,
  Commit: 2,
}

function encodeFlags(flags) {
  let byte = 0
  byte |= (flags.qos & 0x03) << 6
  if (flags.hasTimestamp) byte |= 0x20
  if (flags.encrypted) byte |= 0x10
  if (flags.compressed) byte |= 0x08
  return byte
}

function decodeFlags(byte) {
  return {
    qos: (byte >> 6) & 0x03,
    hasTimestamp: (byte & 0x20) !== 0,
    encrypted: (byte & 0x10) !== 0,
    compressed: (byte & 0x08) !== 0,
  }
}

function encodeFrame(message, options = {}) {
  const payload = msgpackEncode(message)
  const hasTimestamp = options.timestamp !== undefined
  const headerSize = hasTimestamp ? HEADER_SIZE_WITH_TS : HEADER_SIZE
  const frame = new Uint8Array(headerSize + payload.length)
  const view = new DataView(frame.buffer)

  frame[0] = MAGIC
  frame[1] = encodeFlags({
    qos: options.qos ?? QoS.Fire,
    hasTimestamp,
    encrypted: false,
    compressed: false,
  })
  view.setUint16(2, payload.length, false) // big-endian

  if (hasTimestamp && options.timestamp !== undefined) {
    const ts = BigInt(options.timestamp)
    view.setBigUint64(4, ts, false)
  }

  frame.set(payload, headerSize)
  return frame
}

function decodeFrame(data) {
  if (data.length < HEADER_SIZE) {
    throw new Error('Frame too small')
  }
  if (data[0] !== MAGIC) {
    throw new Error(`Invalid magic byte: 0x${data[0].toString(16)}`)
  }

  const view = new DataView(data.buffer, data.byteOffset)
  const flags = decodeFlags(data[1])
  const payloadLength = view.getUint16(2, false)
  const headerSize = flags.hasTimestamp ? HEADER_SIZE_WITH_TS : HEADER_SIZE

  if (data.length < headerSize + payloadLength) {
    throw new Error('Frame incomplete')
  }

  let timestamp
  if (flags.hasTimestamp) {
    timestamp = Number(view.getBigUint64(4, false))
  }

  const payload = data.slice(headerSize, headerSize + payloadLength)
  const message = msgpackDecode(payload)
  return { message, flags, timestamp }
}

function matchPattern(pattern, address) {
  const regex = pattern
    .replace(/\*\*/g, '§§')
    .replace(/\*/g, '[^/]+')
    .replace(/§§/g, '.*')
  return new RegExp(`^${regex}$`).test(address)
}

/**
 * CLASP Client
 */
export class Clasp {
  constructor(url, options = {}) {
    this.url = url
    this.options = {
      name: 'CLASP JS Client',
      features: ['param', 'event', 'stream'],
      reconnect: true,
      reconnectInterval: 5000,
      ...options,
    }

    this.ws = null
    this.sessionId = null
    this._connected = false
    this.params = new Map()
    this.subscriptions = new Map()
    this.nextSubId = 1
    this.serverTimeOffset = 0
    this.pendingGets = new Map()

    this.onConnectCallbacks = []
    this.onDisconnectCallbacks = []
    this.onErrorCallbacks = []
  }

  static builder(url) {
    return new ClaspBuilder(url)
  }

  async connect() {
    return new Promise((resolve, reject) => {
      try {
        this.ws = new WebSocket(this.url, WS_SUBPROTOCOL)
        this.ws.binaryType = 'arraybuffer'

        this.ws.onopen = () => {
          this.sendHello()
        }

        this.ws.onmessage = (event) => {
          const data = new Uint8Array(event.data)
          try {
            const { message } = decodeFrame(data)
            this.handleMessage(message)

            if (message.type === 'WELCOME') {
              this._connected = true
              resolve()
              this.onConnectCallbacks.forEach((cb) => cb())
            }
          } catch (e) {
            console.warn('Decode error:', e)
          }
        }

        this.ws.onerror = (event) => {
          const error = new Error('WebSocket error')
          this.onErrorCallbacks.forEach((cb) => cb(error))
          reject(error)
        }

        this.ws.onclose = (event) => {
          this._connected = false
          this.onDisconnectCallbacks.forEach((cb) => cb(event.reason))

          if (this.options.reconnect) {
            setTimeout(() => {
              this.connect().catch(() => {})
            }, this.options.reconnectInterval)
          }
        }
      } catch (e) {
        reject(e)
      }
    })
  }

  get connected() {
    return this._connected
  }

  get session() {
    return this.sessionId
  }

  time() {
    return Date.now() * 1000 + this.serverTimeOffset
  }

  subscribe(pattern, callback, options) {
    const id = this.nextSubId++
    this.subscriptions.set(id, { pattern, callback })

    const msg = {
      type: 'SUBSCRIBE',
      id,
      pattern,
      options: options ? { maxRate: options.maxRate, epsilon: options.epsilon } : undefined,
    }
    this.send(msg)

    return () => {
      this.subscriptions.delete(id)
      this.send({ type: 'UNSUBSCRIBE', id })
    }
  }

  on(pattern, callback, options) {
    return this.subscribe(pattern, callback, options)
  }

  set(address, value) {
    const msg = {
      type: 'SET',
      address,
      value,
    }
    this.send(msg, QoS.Confirm)
  }

  async get(address) {
    if (this.params.has(address)) {
      return this.params.get(address)
    }

    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        this.pendingGets.delete(address)
        reject(new Error('Timeout'))
      }, 5000)

      this.pendingGets.set(address, (value) => {
        clearTimeout(timeout)
        resolve(value)
      })

      this.send({ type: 'GET', address })
    })
  }

  emit(address, payload) {
    const msg = {
      type: 'PUBLISH',
      address,
      signal: 'event',
      payload: payload ?? null,
      timestamp: this.time(),
    }
    this.send(msg, QoS.Confirm)
  }

  stream(address, value) {
    const msg = {
      type: 'PUBLISH',
      address,
      signal: 'stream',
      value,
      timestamp: this.time(),
    }
    this.send(msg, QoS.Fire)
  }

  bundle(messages, options) {
    const formatted = messages.map((m) => {
      if (m.set) {
        return { type: 'SET', address: m.set[0], value: m.set[1] }
      }
      if (m.emit) {
        return { type: 'PUBLISH', address: m.emit[0], signal: 'event', payload: m.emit[1] }
      }
      throw new Error('Invalid bundle message')
    })
    this.send({ type: 'BUNDLE', timestamp: options?.at, messages: formatted }, QoS.Commit)
  }

  cached(address) {
    return this.params.get(address)
  }

  onConnect(callback) {
    this.onConnectCallbacks.push(callback)
  }

  onDisconnect(callback) {
    this.onDisconnectCallbacks.push(callback)
  }

  onError(callback) {
    this.onErrorCallbacks.push(callback)
  }

  close() {
    this.options.reconnect = false
    this.ws?.close()
    this.ws = null
    this._connected = false
  }

  // Private methods
  sendHello() {
    const hello = {
      type: 'HELLO',
      version: PROTOCOL_VERSION,
      name: this.options.name,
      features: this.options.features,
      token: this.options.token,
    }
    this.send(hello)
  }

  send(message, qos = QoS.Fire) {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      const frame = encodeFrame(message, { qos })
      this.ws.send(frame)
    }
  }

  handleMessage(message) {
    switch (message.type) {
      case 'WELCOME': {
        this.sessionId = message.session
        this.serverTimeOffset = message.time - Date.now() * 1000
        break
      }
      case 'SET': {
        this.params.set(message.address, message.value)
        this.notifySubscribers(message.address, message.value)
        break
      }
      case 'SNAPSHOT': {
        for (const param of message.params) {
          this.params.set(param.address, param.value)
          const resolver = this.pendingGets.get(param.address)
          if (resolver) {
            resolver(param.value)
            this.pendingGets.delete(param.address)
          }
          this.notifySubscribers(param.address, param.value, param)
        }
        break
      }
      case 'PUBLISH': {
        const value = message.value ?? message.payload ?? null
        this.notifySubscribers(message.address, value)
        break
      }
      case 'PING':
        this.send({ type: 'PONG' })
        break
      case 'ERROR':
        console.error('CLASP error:', message)
        break
    }
  }

  notifySubscribers(address, value, meta) {
    for (const [, sub] of this.subscriptions) {
      if (matchPattern(sub.pattern, address)) {
        sub.callback(value, address, meta)
      }
    }
  }
}

/**
 * Builder for Clasp client
 */
export class ClaspBuilder {
  constructor(url) {
    this.url = url
    this.options = {}
  }

  name(name) {
    this.options.name = name
    return this
  }

  features(features) {
    this.options.features = features
    return this
  }

  token(token) {
    this.options.token = token
    return this
  }

  reconnect(enabled) {
    this.options.reconnect = enabled
    return this
  }

  reconnectInterval(ms) {
    this.options.reconnectInterval = ms
    return this
  }

  async connect() {
    const client = new Clasp(this.url, this.options)
    await client.connect()
    return client
  }
}
