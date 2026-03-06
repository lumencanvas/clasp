import type { BridgeProtocol, BridgeOptions } from './types'

/** Default ports for bridge protocols. */
const DEFAULT_PORTS: Record<BridgeProtocol, number> = {
  osc: 9000,
  midi: 0, // no port
  mqtt: 1883,
  artnet: 6454,
  sacn: 5568,
  dmx: 0, // serial, no port
}

/**
 * Bridge command generator. Does not spawn processes --
 * provides CLI commands and Docker Compose config for running bridges.
 */
export class BridgeCommand {
  readonly protocol: BridgeProtocol
  readonly routerUrl: string
  readonly options: BridgeOptions

  constructor(protocol: BridgeProtocol, routerUrl: string, options: BridgeOptions = {}) {
    this.protocol = protocol
    this.routerUrl = routerUrl
    this.options = options
  }

  /** The CLI command to run this bridge. */
  get command(): string {
    const parts = ['clasp', 'bridge', this.protocol]
    parts.push('--router', this.routerUrl)

    const port = this.options.port ?? DEFAULT_PORTS[this.protocol]
    if (port > 0) {
      parts.push('--port', String(port))
    }

    if (this.options.namespace) {
      parts.push('--namespace', this.options.namespace)
    }

    if (this.options.token) {
      parts.push('--token', this.options.token)
    }

    if (this.options.reconnect !== undefined) {
      parts.push('--reconnect', String(this.options.reconnect))
    }

    if (this.protocol === 'mqtt' && this.options.broker) {
      parts.push('--broker', this.options.broker)
    }

    if (this.protocol === 'mqtt' && this.options.topics?.length) {
      for (const topic of this.options.topics) {
        parts.push('--topic', topic)
      }
    }

    if (this.protocol === 'dmx' && this.options.serial) {
      parts.push('--serial', this.options.serial)
    }

    if (this.protocol === 'artnet') {
      if (this.options.universe !== undefined) {
        parts.push('--universe', String(this.options.universe))
      }
      if (this.options.subnet !== undefined) {
        parts.push('--subnet', String(this.options.subnet))
      }
    }

    return parts.join(' ')
  }

  /** Generate a Docker Compose service entry. */
  toDockerCompose(): string {
    const port = this.options.port ?? DEFAULT_PORTS[this.protocol]
    const namespace = this.options.namespace ?? `/${this.protocol}`
    const serviceName = `bridge-${this.protocol}`

    const lines = [
      `  ${serviceName}:`,
      `    image: clasp-relay`,
      `    command: ${this.command}`,
    ]

    if (port > 0) {
      lines.push(`    ports:`)
      const portProto = this.protocol === 'osc' || this.protocol === 'artnet' || this.protocol === 'sacn'
        ? 'udp'
        : 'tcp'
      lines.push(`      - "${port}:${port}/${portProto}"`)
    }

    lines.push(`    environment:`)
    lines.push(`      CLASP_ROUTER: ${this.routerUrl}`)
    lines.push(`      CLASP_BRIDGE_NAMESPACE: ${namespace}`)

    if (this.options.token) {
      lines.push(`      CLASP_TOKEN: ${this.options.token}`)
    }

    if (this.options.reconnect !== undefined) {
      lines.push(`      CLASP_RECONNECT: "${this.options.reconnect}"`)
    }

    if (this.protocol === 'mqtt' && this.options.broker) {
      lines.push(`      MQTT_BROKER: ${this.options.broker}`)
    }

    if (this.protocol === 'dmx' && this.options.serial) {
      lines.push(`      CLASP_BRIDGE_SERIAL: ${this.options.serial}`)
    }

    if (this.protocol === 'artnet') {
      if (this.options.universe !== undefined) {
        lines.push(`      CLASP_BRIDGE_UNIVERSE: "${this.options.universe}"`)
      }
      if (this.options.subnet !== undefined) {
        lines.push(`      CLASP_BRIDGE_SUBNET: "${this.options.subnet}"`)
      }
    }

    return lines.join('\n')
  }

  /** Generate environment variable format. */
  toEnv(): string {
    const port = this.options.port ?? DEFAULT_PORTS[this.protocol]
    const namespace = this.options.namespace ?? `/${this.protocol}`

    const lines = [
      `CLASP_ROUTER=${this.routerUrl}`,
      `CLASP_BRIDGE_PROTOCOL=${this.protocol}`,
      `CLASP_BRIDGE_NAMESPACE=${namespace}`,
    ]

    if (port > 0) {
      lines.push(`CLASP_BRIDGE_PORT=${port}`)
    }

    if (this.options.token) {
      lines.push(`CLASP_TOKEN=${this.options.token}`)
    }

    if (this.options.reconnect !== undefined) {
      lines.push(`CLASP_RECONNECT=${this.options.reconnect}`)
    }

    if (this.protocol === 'mqtt' && this.options.broker) {
      lines.push(`MQTT_BROKER=${this.options.broker}`)
    }

    if (this.options.topics?.length) {
      lines.push(`MQTT_TOPICS=${this.options.topics.join(',')}`)
    }

    if (this.protocol === 'dmx' && this.options.serial) {
      lines.push(`CLASP_BRIDGE_SERIAL=${this.options.serial}`)
    }

    if (this.protocol === 'artnet') {
      if (this.options.universe !== undefined) {
        lines.push(`CLASP_BRIDGE_UNIVERSE=${this.options.universe}`)
      }
      if (this.options.subnet !== undefined) {
        lines.push(`CLASP_BRIDGE_SUBNET=${this.options.subnet}`)
      }
    }

    return lines.join('\n')
  }
}
