import type { Protocol, AnyProtocol, TransformType } from './types'

export const protocolNames: Record<AnyProtocol, string> = {
  osc: 'OSC',
  midi: 'MIDI',
  artnet: 'Art-Net',
  sacn: 'sACN',
  dmx: 'DMX',
  clasp: 'CLASP',
  mqtt: 'MQTT',
  websocket: 'WS',
  socketio: 'SIO',
  http: 'HTTP',
}

export const defaultAddresses: Partial<Record<AnyProtocol, string>> = {
  osc: '0.0.0.0:9000',
  midi: 'default',
  artnet: '0.0.0.0:6454',
  dmx: '/dev/ttyUSB0',
  clasp: 'localhost:7330',
  mqtt: 'localhost:1883',
  websocket: '0.0.0.0:8080',
  socketio: '0.0.0.0:3001',
  http: '0.0.0.0:3000',
}

export const protocolHints: Record<Protocol, string> = {
  osc: 'OSC connection - receive OSC messages from controllers and translate to CLASP',
  midi: 'MIDI connection - connect to MIDI devices and translate to/from CLASP signals',
  mqtt: 'MQTT connection - connect to an MQTT broker with full auth and QoS support',
  websocket: 'WebSocket connection - accept JSON or MsgPack messages from web apps',
  socketio: 'Socket.IO connection - real-time bidirectional event-based communication',
  http: 'HTTP connection - expose signals as HTTP endpoints for webhooks and integrations',
  artnet: 'Art-Net connection - receive DMX512 data over Ethernet from lighting consoles',
  sacn: 'sACN/E1.31 connection - industry-standard streaming ACN for professional lighting',
  dmx: 'DMX connection - connect directly to DMX fixtures via USB adapter',
}

// Protocol badge CSS class mapping (matches global.css .device-protocol-badge.X)
export const protocolBadgeClass: Record<AnyProtocol, string> = {
  osc: 'osc',
  midi: 'midi',
  artnet: 'artnet',
  sacn: 'sacn',
  dmx: 'dmx',
  clasp: 'clasp',
  mqtt: 'mqtt',
  websocket: 'websocket',
  socketio: 'socketio',
  http: 'http',
}

// Protocols that are "source" types in flow diagram (left side)
export const sourceProtocols: Protocol[] = ['osc', 'midi', 'mqtt', 'websocket', 'http', 'socketio']
// Protocols that are "target" types in flow diagram (right side)
export const targetProtocols: Protocol[] = ['artnet', 'dmx', 'sacn']

// All protocol types available for connections
export const allProtocols: Protocol[] = ['osc', 'midi', 'mqtt', 'websocket', 'socketio', 'http', 'artnet', 'sacn', 'dmx']

export const transformTypes: { value: TransformType; label: string }[] = [
  { value: 'direct', label: 'Direct (pass-through)' },
  { value: 'scale', label: 'Scale (range mapping)' },
  { value: 'invert', label: 'Invert (1 - value)' },
  { value: 'clamp', label: 'Clamp (min/max)' },
  { value: 'round', label: 'Round (nearest int)' },
  { value: 'threshold', label: 'Threshold (on/off)' },
  { value: 'gate', label: 'Gate (> 0 = 1)' },
  { value: 'trigger', label: 'Trigger (any = 1)' },
  { value: 'toggle', label: 'Toggle (> 0.5 = 1)' },
  { value: 'deadzone', label: 'Deadzone (center null)' },
  { value: 'smooth', label: 'Smooth (EMA filter)' },
  { value: 'quantize', label: 'Quantize (snap steps)' },
  { value: 'curve', label: 'Curve (easing)' },
  { value: 'modulo', label: 'Modulo (wrap)' },
  { value: 'negate', label: 'Negate (-value)' },
  { value: 'power', label: 'Power (x^n)' },
  { value: 'expression', label: 'Expression (math)' },
  { value: 'javascript', label: 'JavaScript (custom)' },
  { value: 'wasm', label: 'WASM (LensVM module)' },
]

export const curveTypes = [
  { value: 'linear', label: 'Linear' },
  { value: 'ease-in', label: 'Ease In' },
  { value: 'ease-out', label: 'Ease Out' },
  { value: 'ease-in-out', label: 'Ease In/Out' },
  { value: 'exponential', label: 'Exponential' },
  { value: 'logarithmic', label: 'Logarithmic' },
]

export const ruleTriggerTypes = [
  { value: 'on_change', label: 'On Change' },
  { value: 'on_threshold', label: 'On Threshold' },
  { value: 'on_event', label: 'On Event' },
  { value: 'on_interval', label: 'On Interval' },
]

export const ruleActionTypes = [
  { value: 'set', label: 'Set Value' },
  { value: 'publish', label: 'Publish' },
  { value: 'set_from_trigger', label: 'Set From Trigger' },
  { value: 'delay', label: 'Delayed Set' },
]

export const ruleOperators = [
  { value: 'eq', label: '=' },
  { value: 'ne', label: '!=' },
  { value: 'gt', label: '>' },
  { value: 'gte', label: '>=' },
  { value: 'lt', label: '<' },
  { value: 'lte', label: '<=' },
]
