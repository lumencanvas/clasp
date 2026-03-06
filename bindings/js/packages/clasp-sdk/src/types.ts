import type {
  Value,
  SubscriptionCallback,
  Unsubscribe,
  SignalDefinition,
  ErrorMessage,
  ParamValue,
  ConnectOptions,
  SubscribeOptions,
  TimelineKeyframe,
} from '@clasp-to/core'
import { QoS } from '@clasp-to/core'

/** Options for the clasp() entry point. */
export interface ClaspOptions {
  /** Device name shown to the router */
  name?: string
  /** Authentication token (cpsk_, cap_, or ent_ prefix) */
  token?: string
  /** Enable E2E encryption for all set/emit operations */
  encrypted?: boolean
  /** Auto-reconnect on disconnect (default: true) */
  reconnect?: boolean
  /** Auth server base URL override (default: inferred from router URL) */
  authUrl?: string
}

/** Options for device registration. */
export interface RegisterOptions {
  name: string
  scopes?: string[]
  username?: string
  password?: string
}

/** Options for guest access. */
export interface GuestOptions {
  scopes?: string[]
  name?: string
}

/** Options for login. */
export interface LoginOptions {
  username: string
  password: string
}

/** Options for provisioning a child device. */
export interface ProvisionOptions {
  name: string
  scopes: string[]
  type?: 'cpsk' | 'capability'
  expires?: string
  delegatable?: boolean
}

/** Credential bundle returned by provision(). */
export interface Credentials {
  token: string
  url: string
  name: string
  scopes: string[]
  expires?: string
  toJSON(): string
  toEnv(): string
}

/** Options for creating a child device. */
export interface ChildDeviceOptions {
  name: string
  scopes: string[]
}

/** Options for creating/joining an encrypted room. */
export interface RoomOptions {
  /** Password to gate room access */
  password?: string
  /** Auto-rotate encryption keys on this interval (e.g. '1h', '5m') */
  rotateKeys?: string
  /** Callback when a peer's key changes (TOFU violation) */
  onKeyChange?: (peerId: string, oldFingerprint: string, newFingerprint: string) => boolean | Promise<boolean>
  /** Callback on key rotation events */
  onRotation?: () => void
  /** Max age (ms) for stale announcements. Default: 300000 (5 min) */
  maxAnnouncementAge?: number
}

/** Programmatic rule definition. */
export interface RuleDefinition {
  /** Human-readable name (defaults to id) */
  name?: string
  /** Whether the rule is active (default: true) */
  enabled?: boolean
  /** Address pattern that triggers on value change (on_change) or threshold check */
  when?: string
  /** Numeric threshold: fire when value rises above this */
  above?: number
  /** Numeric threshold: fire when value drops below this */
  below?: number
  /** Address pattern that triggers on event publish (on_event) */
  onEvent?: string
  /** Triggers on session join events */
  onSessionJoin?: string
  /** Triggers on session leave events */
  onSessionLeave?: string
  /** Conditions that must be true for the rule to fire.
   *  Simple form: { '/path': value } checks equality.
   *  Operator form: { '/path': { gt: 10 } } uses comparison operators. */
  if?: Record<string, Value | { eq?: Value; ne?: Value; gt?: Value; gte?: Value; lt?: Value; lte?: Value }>
  /** Duration string for periodic trigger ('30s', '5m', '1h') */
  every?: string
  /** Actions to execute when the rule fires */
  then: RuleAction | RuleAction[]
  /** Minimum time between firings ('60s', '5m') */
  cooldown?: string
}

/** A single rule action. */
export interface RuleAction {
  /** Set a param: [address, value] */
  set?: [string, Value]
  /** Emit an event: [address, payload] */
  emit?: [string, Value]
  /** Copy trigger value to address: [address] */
  setFrom?: [string]
  /** Transform to apply with setFrom */
  transform?: RuleTransform
  /** Delay in milliseconds before next action */
  delay?: number
}

/** Rule transform for set_from_trigger actions. */
export type RuleTransform =
  | { type: 'identity' }
  | { type: 'scale'; factor: number; offset: number }
  | { type: 'clamp'; min: number; max: number }
  | { type: 'threshold'; value: number; above?: Value; below?: Value }
  | { type: 'invert' }
  | { type: 'map'; table: Record<string, Value> }
  | { type: 'round'; precision?: number }
  | { type: 'abs' }

/** Bridge protocol types. */
export type BridgeProtocol = 'osc' | 'midi' | 'mqtt' | 'artnet' | 'sacn' | 'dmx'

/** Options for bridge creation. */
export interface BridgeOptions {
  port?: number
  namespace?: string
  broker?: string
  topics?: string[]
  /** Auth token for router connection */
  token?: string
  /** Enable auto-reconnect to router */
  reconnect?: boolean
  /** Serial device path (for DMX) */
  serial?: string
  /** ArtNet universe number */
  universe?: number
  /** ArtNet subnet number */
  subnet?: number
}

/** Discovered router info. */
export interface DiscoveredRouter {
  name: string
  url: string
}

/** Discovery watch event. */
export interface DiscoveryEvent {
  type: 'found' | 'lost' | 'error'
  name: string
  url?: string
  error?: Error
}

// --- Relay configuration types ---

/** App config for declarative scopes, write rules, visibility. */
export interface AppConfig {
  scopes?: string[]
  write_rules?: WriteRule[]
  snapshot_transforms?: SnapshotTransform[]
  snapshot_visibility?: SnapshotVisibility[]
  rate_limits?: RateLimits
}

/** Write validation rule. */
export interface WriteRule {
  path: string
  mode?: 'all' | 'any'
  allow_null_write?: boolean
  checks?: WriteCheck[]
  pre_checks?: WriteCheck[]
}

/** Individual write check. */
export interface WriteCheck {
  type: 'state_field_equals_session' | 'state_not_null' | 'value_field_equals_session'
    | 'segment_equals_session' | 'either_state_not_null' | 'require_value_field'
    | 'reject_unless_path_matches'
  field?: string
  lookup?: string
  segment?: string
  pattern?: string
  path_a?: string
  path_b?: string
  allow_if_missing?: boolean
}

/** Snapshot field redaction. */
export interface SnapshotTransform {
  path: string
  redact_fields: string[]
}

/** Snapshot visibility rule. */
export interface SnapshotVisibility {
  path: string
  visible: boolean | 'owner' | 'require_state_not_null'
  owner_segment?: string
  public_sub?: string
  lookup?: string
}

/** Rate limit config. */
export interface RateLimits {
  login_max_attempts?: number
  login_window_secs?: number
  register_max_attempts?: number
  register_window_secs?: number
}

/** Relay builder configuration. */
export interface RelayConfig {
  port?: number
  host?: string
  name?: string
  authPort?: number
  corsOrigin?: string | string[]
  adminTokenPath?: string
  tokenTtl?: number
  maxSessions?: number
  sessionTimeout?: number
  paramTtl?: number
  signalTtl?: number
  noTtl?: boolean
  verbose?: boolean
  logLevel?: 'error' | 'warn' | 'info' | 'debug' | 'trace'
  persist?: { path: string; interval?: number }
  journal?: { path: string; batchSize?: number; flushMs?: number } | 'memory'
  cert?: string
  key?: string
  appConfig?: AppConfig | string
  rules?: Record<string, unknown> | string
  trustAnchors?: string[]
  capMaxDepth?: number
  registryDb?: string
  mqtt?: { port: number; namespace?: string }
  osc?: { port: number; namespace?: string }
  quic?: { port: number; cert: string; key: string }
  federation?: { hub: string; id: string; namespaces: string[]; token?: string }
  rendezvous?: { port?: number; ttl?: number }
  drainTimeout?: number
  healthPort?: number
}

/** Re-export core types for convenience. */
export type { Value, SubscriptionCallback, Unsubscribe, SignalDefinition, ErrorMessage, ParamValue, ConnectOptions, SubscribeOptions, TimelineKeyframe }
export { QoS }
