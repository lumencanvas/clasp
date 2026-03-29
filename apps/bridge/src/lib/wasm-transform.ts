/**
 * Browser-side LensVM WASM transform host.
 *
 * Loads lens WASM modules via the browser WebAssembly API and executes
 * transforms using the LensVM protocol (alloc, next, set_param, transform).
 *
 * This mirrors the Rust-side clasp-lens host but runs in the renderer
 * process for UI preview and client-side transforms.
 */

// Transport buffer type IDs
const TYPE_ERROR = 0xFF // -1 as u8
const TYPE_NIL = 0
const TYPE_JSON = 1
const TYPE_EOS = 127

/** Encode a JSON value into a LensVM transport buffer. */
function encodeJson(value: unknown): Uint8Array {
  const payload = new TextEncoder().encode(JSON.stringify(value))
  const buf = new Uint8Array(1 + 4 + payload.length)
  buf[0] = TYPE_JSON
  const view = new DataView(buf.buffer)
  view.setUint32(1, payload.length, true)
  buf.set(payload, 5)
  return buf
}

/** Encode an end-of-stream marker. */
function encodeEos(): Uint8Array {
  return new Uint8Array([TYPE_EOS])
}

/** Decode a transport buffer from WASM memory at a given pointer. */
function decodeFromMemory(mem: WebAssembly.Memory, ptr: number): { type: 'json'; value: unknown } | { type: 'eos' } | { type: 'nil' } | { type: 'error'; message: string } {
  const bytes = new Uint8Array(mem.buffer)
  const typeId = bytes[ptr]

  if (typeId === TYPE_NIL) return { type: 'nil' }
  if (typeId === TYPE_EOS) return { type: 'eos' }

  const view = new DataView(mem.buffer)
  const len = view.getUint32(ptr + 1, true)
  const payload = bytes.slice(ptr + 5, ptr + 5 + len)

  if (typeId === TYPE_JSON) {
    const json = new TextDecoder().decode(payload)
    return { type: 'json', value: JSON.parse(json) }
  }

  if (typeId === TYPE_ERROR) {
    const msg = new TextDecoder().decode(payload)
    return { type: 'error', message: msg }
  }

  return { type: 'error', message: `unknown type ID: ${typeId}` }
}

/** A compiled WASM transform module ready for execution. */
export class WasmTransformHost {
  private module: WebAssembly.Module
  private params: unknown | null = null

  private constructor(module: WebAssembly.Module) {
    this.module = module
  }

  /** Compile a WASM module from raw bytes. */
  static async load(bytes: ArrayBuffer | Uint8Array): Promise<WasmTransformHost> {
    const module = await WebAssembly.compile(bytes)
    return new WasmTransformHost(module)
  }

  /** Set parameters passed to the lens via set_param(). */
  setParams(params: unknown): void {
    this.params = params
  }

  /** Run the forward transform on a value. */
  async transform(input: unknown): Promise<unknown> {
    return this.runLens('transform', input)
  }

  /** Run the inverse transform on a value. */
  async inverse(input: unknown): Promise<unknown> {
    return this.runLens('inverse', input)
  }

  /** Check if the module exports an inverse function. */
  hasInverse(): boolean {
    const exports = WebAssembly.Module.exports(this.module)
    return exports.some(e => e.name === 'inverse')
  }

  private async runLens(funcName: string, input: unknown): Promise<unknown> {
    const inputQueue: Uint8Array[] = [encodeJson(input), encodeEos()]
    let inputPos = 0

    const importObject: WebAssembly.Imports = {
      lens: {
        next: () => {
          const data = inputPos < inputQueue.length
            ? inputQueue[inputPos]
            : encodeEos()
          inputPos++

          const instance = currentInstance!
          const memory = instance.exports.memory as WebAssembly.Memory
          const alloc = instance.exports.alloc as (size: number) => number

          const ptr = alloc(data.length)
          const memBytes = new Uint8Array(memory.buffer)
          memBytes.set(data, ptr)
          return ptr
        }
      }
    }

    const instance = await WebAssembly.instantiate(this.module, importObject)
    let currentInstance: WebAssembly.Instance | null = instance

    const memory = instance.exports.memory as WebAssembly.Memory
    const alloc = instance.exports.alloc as (size: number) => number

    // Set params if configured
    if (this.params != null && instance.exports.set_param) {
      const paramData = encodeJson(this.params)
      const paramPtr = alloc(paramData.length)
      new Uint8Array(memory.buffer).set(paramData, paramPtr)
      const setParam = instance.exports.set_param as (ptr: number) => number
      setParam(paramPtr)
    }

    // Call transform/inverse
    const fn = instance.exports[funcName] as () => number
    if (!fn) throw new Error(`WASM module does not export '${funcName}'`)
    const resultPtr = fn()

    if (resultPtr === 0) throw new Error('transform returned null pointer')

    const result = decodeFromMemory(memory, resultPtr)
    currentInstance = null

    switch (result.type) {
      case 'json': return result.value
      case 'nil': return null
      case 'eos': throw new Error('lens returned end of stream (no output)')
      case 'error': throw new Error(`lens error: ${result.message}`)
    }
  }
}

/**
 * Cache of compiled WASM modules keyed by content hash.
 * Avoids re-compilation when the same module is used across routes.
 */
export class WasmTransformPool {
  private cache = new Map<string, WasmTransformHost>()

  /** Get or compile a WASM module. Key should be a stable identifier (hash or module ID). */
  async get(key: string, bytes: ArrayBuffer | Uint8Array): Promise<WasmTransformHost> {
    let host = this.cache.get(key)
    if (!host) {
      host = await WasmTransformHost.load(bytes)
      this.cache.set(key, host)
    }
    return host
  }

  /** Remove a cached module. */
  evict(key: string): void {
    this.cache.delete(key)
  }

  /** Clear all cached modules. */
  clear(): void {
    this.cache.clear()
  }
}
