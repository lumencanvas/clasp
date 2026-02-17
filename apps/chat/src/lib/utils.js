import { AVATAR_COLORS } from './constants.js'

/**
 * Generate a short alphanumeric ID
 */
export function generateId(length = 8) {
  const chars = 'abcdefghijklmnopqrstuvwxyz0123456789'
  let result = ''
  for (let i = 0; i < length; i++) {
    result += chars[Math.floor(Math.random() * chars.length)]
  }
  return result
}

/**
 * Format timestamp to HH:MM
 */
export function formatTime(timestamp) {
  return new Date(timestamp).toLocaleTimeString([], {
    hour: '2-digit',
    minute: '2-digit',
  })
}

/**
 * Format timestamp to relative time (e.g., "2m ago", "1h ago")
 */
export function formatRelativeTime(timestamp) {
  const diff = Date.now() - timestamp
  const seconds = Math.floor(diff / 1000)
  if (seconds < 60) return 'just now'
  const minutes = Math.floor(seconds / 60)
  if (minutes < 60) return `${minutes}m ago`
  const hours = Math.floor(minutes / 60)
  if (hours < 24) return `${hours}h ago`
  const days = Math.floor(hours / 24)
  return `${days}d ago`
}

/**
 * Get initials from a display name
 */
export function getInitials(name) {
  if (!name) return '?'
  return name
    .split(' ')
    .map(n => n[0])
    .join('')
    .toUpperCase()
    .slice(0, 2)
}

/**
 * Get a deterministic avatar color from a string
 */
export function getAvatarColor(str) {
  if (!str) return AVATAR_COLORS[0]
  let hash = 0
  for (let i = 0; i < str.length; i++) {
    hash = str.charCodeAt(i) + ((hash << 5) - hash)
  }
  return AVATAR_COLORS[Math.abs(hash) % AVATAR_COLORS.length]
}

/**
 * Truncate text to a max length
 */
export function truncate(text, max = 100) {
  if (!text || text.length <= max) return text
  return text.slice(0, max) + '...'
}
