// CLASP address prefixes
export const ADDR = {
  USER_PROFILE: '/chat/user',
  ROOM_REGISTRY: '/chat/registry/rooms',
  ROOM: '/chat/room',
}

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
}

// Room type labels and icons
export const ROOM_TYPE_INFO = {
  text: { label: 'Text', icon: '#' },
  video: { label: 'Video', icon: 'cam' },
  combo: { label: 'Combo', icon: 'layout' },
}

// Default relay URL
export const DEFAULT_RELAY_URL = 'wss://relay.clasp.to'
