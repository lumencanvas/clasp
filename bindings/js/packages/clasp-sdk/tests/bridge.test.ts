import { describe, it, expect } from 'vitest'
import { BridgeCommand } from '../src/bridge'

describe('BridgeCommand', () => {
  describe('CLI command generation', () => {
    it('OSC with default port (9000)', () => {
      const bridge = new BridgeCommand('osc', 'ws://localhost:7330')
      expect(bridge.command).toBe('clasp bridge osc --router ws://localhost:7330 --port 9000')
    })

    it('OSC with custom port', () => {
      const bridge = new BridgeCommand('osc', 'ws://localhost:7330', { port: 8000 })
      expect(bridge.command).toBe('clasp bridge osc --router ws://localhost:7330 --port 8000')
    })

    it('OSC with namespace', () => {
      const bridge = new BridgeCommand('osc', 'ws://localhost:7330', { namespace: '/stage' })
      expect(bridge.command).toContain('--namespace /stage')
    })

    it('MIDI (no port in command)', () => {
      const bridge = new BridgeCommand('midi', 'ws://localhost:7330')
      expect(bridge.command).toBe('clasp bridge midi --router ws://localhost:7330')
      expect(bridge.command).not.toContain('--port')
    })

    it('MQTT with broker and topics', () => {
      const bridge = new BridgeCommand('mqtt', 'ws://localhost:7330', {
        broker: 'mqtt://localhost:1883',
        topics: ['sensors/#', 'actuators/#'],
      })
      expect(bridge.command).toContain('--broker mqtt://localhost:1883')
      expect(bridge.command).toContain('--topic sensors/#')
      expect(bridge.command).toContain('--topic actuators/#')
    })

    it('MQTT without broker (just port)', () => {
      const bridge = new BridgeCommand('mqtt', 'ws://localhost:7330')
      expect(bridge.command).toContain('--port 1883')
      expect(bridge.command).not.toContain('--broker')
    })

    it('ArtNet with default port (6454)', () => {
      const bridge = new BridgeCommand('artnet', 'ws://localhost:7330')
      expect(bridge.command).toContain('--port 6454')
    })

    it('ArtNet with universe and subnet', () => {
      const bridge = new BridgeCommand('artnet', 'ws://localhost:7330', {
        universe: 3,
        subnet: 1,
      })
      expect(bridge.command).toContain('--universe 3')
      expect(bridge.command).toContain('--subnet 1')
    })

    it('sACN with default port (5568)', () => {
      const bridge = new BridgeCommand('sacn', 'ws://localhost:7330')
      expect(bridge.command).toContain('--port 5568')
    })

    it('DMX (no port, serial)', () => {
      const bridge = new BridgeCommand('dmx', 'ws://localhost:7330', { serial: '/dev/ttyUSB0' })
      expect(bridge.command).not.toContain('--port')
      expect(bridge.command).toContain('--serial /dev/ttyUSB0')
    })

    it('all protocols include --router URL', () => {
      const protocols = ['osc', 'midi', 'mqtt', 'artnet', 'sacn', 'dmx'] as const
      for (const proto of protocols) {
        const bridge = new BridgeCommand(proto, 'ws://myhost:7330')
        expect(bridge.command).toContain('--router ws://myhost:7330')
      }
    })

    it('includes token when provided', () => {
      const bridge = new BridgeCommand('osc', 'ws://localhost:7330', { token: 'cpsk_abc' })
      expect(bridge.command).toContain('--token cpsk_abc')
    })

    it('includes reconnect when provided', () => {
      const bridge = new BridgeCommand('osc', 'ws://localhost:7330', { reconnect: true })
      expect(bridge.command).toContain('--reconnect true')
    })
  })

  describe('Docker Compose generation', () => {
    it('OSC uses UDP port mapping', () => {
      const bridge = new BridgeCommand('osc', 'ws://localhost:7330', { port: 9000 })
      const compose = bridge.toDockerCompose()
      expect(compose).toContain('9000:9000/udp')
    })

    it('MQTT uses TCP port mapping', () => {
      const bridge = new BridgeCommand('mqtt', 'ws://localhost:7330', { port: 1883 })
      const compose = bridge.toDockerCompose()
      expect(compose).toContain('1883:1883/tcp')
    })

    it('ArtNet uses UDP port mapping', () => {
      const bridge = new BridgeCommand('artnet', 'ws://localhost:7330')
      const compose = bridge.toDockerCompose()
      expect(compose).toContain('6454:6454/udp')
    })

    it('sACN uses UDP port mapping', () => {
      const bridge = new BridgeCommand('sacn', 'ws://localhost:7330')
      const compose = bridge.toDockerCompose()
      expect(compose).toContain('5568:5568/udp')
    })

    it('MIDI has no ports section', () => {
      const bridge = new BridgeCommand('midi', 'ws://localhost:7330')
      const compose = bridge.toDockerCompose()
      expect(compose).not.toContain('ports:')
      expect(compose).toContain('bridge-midi:')
    })

    it('DMX has no ports section', () => {
      const bridge = new BridgeCommand('dmx', 'ws://localhost:7330')
      const compose = bridge.toDockerCompose()
      expect(compose).not.toContain('ports:')
    })

    it('includes environment variables', () => {
      const bridge = new BridgeCommand('osc', 'ws://localhost:7330')
      const compose = bridge.toDockerCompose()
      expect(compose).toContain('CLASP_ROUTER: ws://localhost:7330')
    })

    it('includes correct service name', () => {
      const bridge = new BridgeCommand('mqtt', 'ws://localhost:7330')
      const compose = bridge.toDockerCompose()
      expect(compose).toContain('bridge-mqtt:')
    })

    it('includes token in environment', () => {
      const bridge = new BridgeCommand('osc', 'ws://localhost:7330', { token: 'cpsk_test' })
      const compose = bridge.toDockerCompose()
      expect(compose).toContain('CLASP_TOKEN: cpsk_test')
    })
  })

  describe('Environment variable generation', () => {
    it('includes all common vars', () => {
      const bridge = new BridgeCommand('osc', 'ws://localhost:7330', { namespace: '/stage' })
      const env = bridge.toEnv()
      expect(env).toContain('CLASP_ROUTER=ws://localhost:7330')
      expect(env).toContain('CLASP_BRIDGE_PROTOCOL=osc')
      expect(env).toContain('CLASP_BRIDGE_NAMESPACE=/stage')
    })

    it('includes port when applicable', () => {
      const bridge = new BridgeCommand('osc', 'ws://localhost:7330')
      const env = bridge.toEnv()
      expect(env).toContain('CLASP_BRIDGE_PORT=9000')
    })

    it('omits port for MIDI', () => {
      const bridge = new BridgeCommand('midi', 'ws://localhost:7330')
      const env = bridge.toEnv()
      expect(env).not.toContain('CLASP_BRIDGE_PORT')
    })

    it('omits port for DMX', () => {
      const bridge = new BridgeCommand('dmx', 'ws://localhost:7330')
      const env = bridge.toEnv()
      expect(env).not.toContain('CLASP_BRIDGE_PORT')
    })

    it('includes MQTT_BROKER when set', () => {
      const bridge = new BridgeCommand('mqtt', 'ws://localhost:7330', {
        broker: 'mqtt://broker:1883',
      })
      const env = bridge.toEnv()
      expect(env).toContain('MQTT_BROKER=mqtt://broker:1883')
    })

    it('includes MQTT_TOPICS comma-separated', () => {
      const bridge = new BridgeCommand('mqtt', 'ws://localhost:7330', {
        topics: ['a/#', 'b/#'],
      })
      const env = bridge.toEnv()
      expect(env).toContain('MQTT_TOPICS=a/#,b/#')
    })

    it('includes token when set', () => {
      const bridge = new BridgeCommand('osc', 'ws://localhost:7330', { token: 'cpsk_x' })
      const env = bridge.toEnv()
      expect(env).toContain('CLASP_TOKEN=cpsk_x')
    })

    it('includes reconnect when set', () => {
      const bridge = new BridgeCommand('osc', 'ws://localhost:7330', { reconnect: true })
      const env = bridge.toEnv()
      expect(env).toContain('CLASP_RECONNECT=true')
    })

    it('includes serial for DMX', () => {
      const bridge = new BridgeCommand('dmx', 'ws://localhost:7330', { serial: '/dev/ttyUSB0' })
      const env = bridge.toEnv()
      expect(env).toContain('CLASP_BRIDGE_SERIAL=/dev/ttyUSB0')
    })

    it('includes universe/subnet for ArtNet', () => {
      const bridge = new BridgeCommand('artnet', 'ws://localhost:7330', { universe: 3, subnet: 1 })
      const env = bridge.toEnv()
      expect(env).toContain('CLASP_BRIDGE_UNIVERSE=3')
      expect(env).toContain('CLASP_BRIDGE_SUBNET=1')
    })
  })

  describe('Docker Compose includes protocol-specific options', () => {
    it('includes reconnect in Docker env', () => {
      const bridge = new BridgeCommand('osc', 'ws://localhost:7330', { reconnect: true })
      const compose = bridge.toDockerCompose()
      expect(compose).toContain('CLASP_RECONNECT')
    })

    it('includes serial for DMX in Docker env', () => {
      const bridge = new BridgeCommand('dmx', 'ws://localhost:7330', { serial: '/dev/ttyUSB0' })
      const compose = bridge.toDockerCompose()
      expect(compose).toContain('CLASP_BRIDGE_SERIAL: /dev/ttyUSB0')
    })

    it('includes universe/subnet for ArtNet in Docker env', () => {
      const bridge = new BridgeCommand('artnet', 'ws://localhost:7330', { universe: 3, subnet: 1 })
      const compose = bridge.toDockerCompose()
      expect(compose).toContain('CLASP_BRIDGE_UNIVERSE')
      expect(compose).toContain('CLASP_BRIDGE_SUBNET')
    })
  })
})
