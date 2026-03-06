// Protocol types
export type Protocol = 'osc' | 'midi' | 'artnet' | 'sacn' | 'dmx' | 'mqtt' | 'websocket' | 'socketio' | 'http'
export type RouterProtocol = 'clasp'
export type AnyProtocol = Protocol | RouterProtocol

export type ConnectionStatus = 'connected' | 'running' | 'starting' | 'reconnecting' | 'error' | 'disconnected' | 'available' | 'stopped'

export interface Router {
  id: string
  type: 'clasp'
  protocol: 'clasp'
  name: string
  address: string
  status: ConnectionStatus
  error?: string
  announce?: boolean
  isRemote?: boolean
  remoteAddress?: string
  discoveredFrom?: string

  // Core
  maxSessions?: number
  sessionTimeout?: number

  // Auth (CPSK)
  authEnabled?: boolean
  token?: string
  tokenFileContent?: string
  authPort?: number
  authDb?: string
  adminTokenPath?: string
  tokenTtl?: number
  corsOrigin?: string

  // Transports
  quicEnabled?: boolean
  quicPort?: number
  certPath?: string
  keyPath?: string
  mqttBridgeEnabled?: boolean
  mqttBridgePort?: number
  mqttBridgeNamespace?: string
  oscBridgeEnabled?: boolean
  oscBridgePort?: number
  oscBridgeNamespace?: string

  // TTL
  paramTtl?: number
  signalTtl?: number
  noTtl?: boolean

  // Persistence
  persistEnabled?: boolean
  persistPath?: string
  persistInterval?: number
  journalEnabled?: boolean
  journalPath?: string
  journalMemory?: boolean

  // Federation
  federationEnabled?: boolean
  federationHub?: string
  federationId?: string
  federationToken?: string

  // Operations
  healthEnabled?: boolean
  healthPort?: number
  metricsEnabled?: boolean
  metricsPort?: number
  drainTimeout?: number
  rendezvousPort?: number
  rendezvousTtl?: number

  // Rules
  rulesPath?: string
}

export interface Connection {
  id: string
  type: Protocol
  protocol: Protocol
  name: string
  address: string
  status: ConnectionStatus
  error?: string
  routerId?: string
  connectedRouterId?: string
  routerConnected?: boolean
  routerError?: string
  // OSC
  bind?: string
  port?: number
  // MQTT
  host?: string
  topics?: string[]
  clientId?: string
  qos?: number
  keepAlive?: number
  namespace?: string
  authEnabled?: boolean
  username?: string
  password?: string
  // WebSocket
  mode?: string
  format?: string
  pingInterval?: number
  // HTTP
  basePath?: string
  cors?: boolean
  // Art-Net / sACN
  subnet?: number
  universe?: number
  // sACN specific
  universes?: number[]
  sourceName?: string
  priority?: number
  multicast?: boolean
  bindAddress?: string
  unicastDestinations?: string[]
  // DMX
  serialPort?: string
  // MIDI
  inputPort?: string
  outputPort?: string
  // Socket.IO (uses mode, address, namespace from above)
  autoReconnect?: boolean
  // HTTP extended
  authToken?: string
  timeout?: number
  pollInterval?: number
  // Art-Net / sACN extended
  normalize?: boolean
  artnetMode?: 'channel' | 'universe'
  syncAddress?: string
  // DMX extended
  baudRate?: number
  direction?: 'input' | 'output' | 'both'
  channels?: number
  refreshRate?: number
  // MIDI extended
  deviceName?: string
  // WebSocket extended
  customHeaders?: string
  // Security
  token?: string
}

export interface DirectLink {
  id: string
  source: AnyProtocol
  sourceAddr: string
  target: AnyProtocol
  targetAddr: string
  active: boolean
}

export interface SignalEndpoint {
  protocol: AnyProtocol
  address?: string
  midiType?: string
  midiChannel?: number | null
  midiNumber?: number | null
  dmxUniverse?: number | null
  dmxChannel?: number | null
  valueType?: string
  jsonPath?: string
  jsonTemplate?: string
}

export type TransformType = 'direct' | 'scale' | 'invert' | 'clamp' | 'round' | 'threshold' | 'gate' | 'trigger' | 'toggle' | 'expression' | 'javascript' | 'deadzone' | 'smooth' | 'quantize' | 'curve' | 'modulo' | 'negate' | 'power'

export interface TransformConfig {
  type: TransformType
  scaleInMin?: number
  scaleInMax?: number
  scaleOutMin?: number
  scaleOutMax?: number
  clampMin?: number
  clampMax?: number
  threshold?: number
  expression?: string
  javascriptCode?: string
  // Deadzone
  deadzoneMin?: number
  deadzoneMax?: number
  // Smooth
  smoothFactor?: number
  // Quantize
  quantizeSteps?: number
  // Curve
  curveType?: 'linear' | 'ease-in' | 'ease-out' | 'ease-in-out' | 'exponential' | 'logarithmic'
  // Modulo
  moduloDivisor?: number
  // Power
  powerExponent?: number
}

// Rules engine
export type RuleTriggerType = 'on_change' | 'on_threshold' | 'on_event' | 'on_interval'
export type RuleActionType = 'set' | 'publish' | 'set_from_trigger' | 'delay'
export type RuleOperator = 'eq' | 'ne' | 'gt' | 'gte' | 'lt' | 'lte'

export interface RuleTrigger {
  type: RuleTriggerType
  address?: string
  threshold?: number
  direction?: 'rising' | 'falling' | 'both'
  event?: string
  seconds?: number
}

export interface RuleCondition {
  address: string
  operator: RuleOperator
  value: number | string
}

export interface RuleAction {
  type: RuleActionType
  address: string
  value?: number | string
  transform?: string
  delayMs?: number
}

export interface Rule {
  id: string
  name: string
  enabled: boolean
  trigger: RuleTrigger
  conditions: RuleCondition[]
  actions: RuleAction[]
  cooldown?: number
}

// App config / security
export interface Scope {
  id: string
  name: string
  action: string
  pattern: string
}

export type CheckType = 'require_auth' | 'require_scope' | 'require_state_not_null' | 'rate_limit' | 'max_size' | 'schema_validate' | 'owner_only'

export interface WriteCheck {
  type: CheckType
  scope?: string
  limit?: number
  window?: number
  maxBytes?: number
  schema?: string
}

export interface WriteRule {
  id: string
  path: string
  preChecks: WriteCheck[]
  checks: WriteCheck[]
}

export type VisibilityMode = true | false | 'owner' | 'require_state_not_null'

export interface VisibilityRule {
  id: string
  type: 'path' | 'contains' | 'catchall'
  pattern: string
  visible: VisibilityMode
}

export interface SnapshotTransform {
  id: string
  path: string
  redactFields: string[]
}

export interface RateLimits {
  loginMaxAttempts: number
  loginWindow: number
  registerMaxAttempts: number
  registerWindow: number
}

export interface AppConfig {
  scopes: Scope[]
  writeRules: WriteRule[]
  visibility: VisibilityRule[]
  snapshotTransforms: SnapshotTransform[]
  rateLimits: RateLimits
}

export interface SignalRoute {
  id: string
  enabled: boolean
  source: SignalEndpoint
  target: SignalEndpoint
  transform: TransformConfig
}

export interface Signal {
  address?: string
  topic?: string
  value?: any
  protocol?: AnyProtocol
  timestamp?: number
  serverName?: string
  serverPort?: number
  serverAddress?: string
  bridgeId?: string
  forwarded?: boolean
  originalProtocol?: string
  channel?: number
  note?: number
  cc?: number
  velocity?: number
  universe?: number
}

export interface Device {
  id: string
  name: string
  protocol?: AnyProtocol
  status?: string
  address?: string
  host?: string
  port?: number
}

export interface Token {
  id: string
  name: string
  token: string
  scopes: string[]
  created: string
}

export interface SignalHistory {
  values: number[]
  updateCount: number
  lastUpdate: number
}

export interface ServerStats {
  id: string
  messagesIn?: number
  messagesOut?: number
  errors?: number
  uptime?: number
}

export interface LogEntry {
  timestamp: string
  level: 'error' | 'warning' | 'info' | 'debug'
  message: string
  source?: string
}

export interface DiagnosticsResult {
  bridgeService: {
    running: boolean
    pid?: number
  }
  system: {
    platform: string
    nodeVersion: string
    electronVersion: string
    uptime: number
    memoryUsage: {
      heapUsed: number
      heapTotal: number
    }
  }
  servers: Array<{
    name: string
    type: string
    status: string
    messagesIn: number
    messagesOut: number
    errors: number
  }>
}

// Notification types
export type NotificationType = 'success' | 'error' | 'warning' | 'info'

export interface Notification {
  id: string
  message: string
  type: NotificationType
  timestamp: number
}

// Config import/export
export interface BridgeConfig {
  version: number
  exportedAt: string
  name: string
  routers: Partial<Router>[]
  servers: Partial<Connection>[]
  bridges: Partial<DirectLink>[]
  mappings: Partial<SignalRoute>[]
}

// Preset types
export interface Preset {
  id: string
  name: string
  description: string
  icon: string
  category: string
  tags: string[]
  servers: Partial<Connection & Router>[]
  bridges: Partial<DirectLink>[]
  mappings: Partial<SignalRoute>[]
}

// Electron IPC API shape (from preload)
export interface ClaspAPI {
  getDevices: () => Promise<Device[]>
  scanNetwork: () => Promise<Device[]>
  addServer: (address: string) => Promise<any>
  startServer: (config: any) => Promise<{ id: string }>
  stopServer: (id: string) => Promise<void>
  getBridges: () => Promise<DirectLink[]>
  createBridge: (config: any) => Promise<DirectLink>
  deleteBridge: (id: string) => Promise<void>
  getServerLogs: (id: string) => Promise<LogEntry[]>
  testConnection: (address: string) => Promise<{ success: boolean; error?: string }>
  listSerialPorts: () => Promise<Array<{ path: string; name: string }>>
  listMidiPorts: () => Promise<{ inputs: Array<{ id: string; name: string }>; outputs: Array<{ id: string; name: string }> }>
  listNetworkInterfaces: () => Promise<Array<{ address: string; label: string }>>
  testSerialPort: (portPath: string) => Promise<{ success: boolean; error?: string }>
  testPortAvailable: (host: string, port: number) => Promise<{ success: boolean; error?: string }>
  getServerStats: (id: string) => Promise<ServerStats>
  getAllServerStats: () => Promise<ServerStats[]>
  healthCheck: (id: string) => Promise<{ healthy: boolean }>
  runDiagnostics: () => Promise<DiagnosticsResult>
  getBridgeStatus: () => Promise<{ ready: boolean }>
  sendTestSignal: (config: any) => Promise<{ success: boolean; error?: string }>
  sendTestSignalBatch: (signals: any[]) => Promise<any>
  sendSignal: (bridgeId: string | any, address?: string, value?: any) => Promise<void>
  startLearnMode: (target: string) => Promise<void>
  stopLearnMode: () => Promise<void>
  getAppVersion: () => Promise<string>
  isFirstRun: () => Promise<boolean>
  setFirstRunComplete: () => Promise<void>
  showSaveDialog: (options: any) => Promise<{ canceled: boolean; filePath?: string }>
  showOpenDialog: (options: any) => Promise<{ canceled: boolean; filePaths?: string[] }>
  writeFile: (path: string, content: string) => Promise<void>
  readFile: (path: string) => Promise<{ success: boolean; content?: string; error?: string }>
  // Events (return cleanup functions)
  onDeviceFound: (callback: (device: Device) => void) => () => void
  onDeviceUpdated: (callback: (device: Device) => void) => () => void
  onDeviceLost: (callback: (deviceId: string) => void) => () => void
  onSignal: (callback: (signal: Signal) => void) => () => void
  onScanStarted: (callback: () => void) => () => void
  onScanComplete: (callback: () => void) => () => void
  onServerStatus: (callback: (status: { id: string; status: string; error?: string }) => void) => () => void
  onServerLog: (callback: (data: { serverId: string; log: LogEntry }) => void) => () => void
  onBridgeEvent: (callback: (data: { bridgeId: string; event: string; data?: string }) => void) => () => void
  onLearnedSignal: (callback: (signal: Signal) => void) => () => void
  onServerStatsUpdate: (callback: (stats: ServerStats[]) => void) => () => void
  onBridgeReady: (callback: (ready: boolean) => void) => () => void
  onBridgeRouterStatus: (callback: (status: { bridgeId: string; connected: boolean; error?: string; routerId?: string }) => void) => () => void
}

declare global {
  interface Window {
    clasp?: ClaspAPI
  }
}
