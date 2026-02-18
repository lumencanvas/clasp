// CLASP address prefixes
export const ADDR = {
  USER_PROFILE: '/chat/user',
  ROOM_REGISTRY: '/chat/registry/rooms',
  ROOM: '/chat/room',
  REQUESTS: '/chat/requests',
  CRYPTO: '/chat/room', // crypto paths: /chat/room/{rid}/crypto/...
}

// Auth API URL (set via env or default for local dev)
export const AUTH_API_URL = import.meta.env.VITE_AUTH_API_URL || 'https://relay.clasp.chat'

// TTL values (milliseconds)
export const TTL = {
  PRESENCE_HEARTBEAT: 10_000,
  PRESENCE_STALE: 25_000,
  TYPING_TIMEOUT: 2_000,
  TYPING_EXPIRE: 3_000,
  DISCONNECT_GRACE: 5_000,
}

// Avatar color palette
export const AVATAR_COLORS = [
  '#e63946', // red
  '#457b9d', // blue
  '#2a9d8f', // teal
  '#f77f00', // orange
  '#9b5de5', // purple
  '#00bbf9', // cyan
  '#f15bb5', // pink
  '#fee440', // yellow
  '#8ac926', // lime
  '#ff595e', // coral
]

// Room types
export const ROOM_TYPES = {
  TEXT: 'text',
  VIDEO: 'video',
  COMBO: 'combo',
  DM: 'dm',
}

// Room type labels and icons
export const ROOM_TYPE_INFO = {
  text: { label: 'Text', icon: '#' },
  video: { label: 'Video', icon: 'cam' },
  combo: { label: 'Combo', icon: 'layout' },
  dm: { label: 'DM', icon: 'dm' },
}

// User status options
export const USER_STATUSES = [
  { value: 'online', label: 'Online', color: '#2a9d8f' },
  { value: 'away', label: 'Away', color: '#f77f00' },
  { value: 'dnd', label: 'Do Not Disturb', color: '#e63946' },
  { value: 'invisible', label: 'Invisible', color: '#6b7280' },
]

// Default relay URL
export const DEFAULT_RELAY_URL = import.meta.env.VITE_RELAY_URL || 'wss://relay.clasp.chat'
