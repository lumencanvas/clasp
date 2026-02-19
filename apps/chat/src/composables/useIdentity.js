import { ref } from 'vue'
import { AVATAR_COLORS, ADDR } from '../lib/constants.js'
import { useClasp } from './useClasp.js'

// Persisted identity
function loadOrCreate(key, factory) {
  const stored = localStorage.getItem(key)
  if (stored) return stored
  const val = factory()
  localStorage.setItem(key, val)
  return val
}

const userId = ref(loadOrCreate('clasp-chat-userId', () => crypto.randomUUID()))
const displayName = ref(localStorage.getItem('clasp-chat-displayName') || '')
const avatarColor = ref(
  localStorage.getItem('clasp-chat-avatarColor') || AVATAR_COLORS[Math.floor(Math.random() * AVATAR_COLORS.length)]
)
const status = ref(localStorage.getItem('clasp-chat-status') || 'online')

function setDisplayName(name) {
  displayName.value = name
  localStorage.setItem('clasp-chat-displayName', name)
}

function setAvatarColor(color) {
  avatarColor.value = color
  localStorage.setItem('clasp-chat-avatarColor', color)
}

function setUserId(newId) {
  userId.value = newId
  localStorage.setItem('clasp-chat-userId', newId)
}

function setStatus(newStatus) {
  status.value = newStatus
  localStorage.setItem('clasp-chat-status', newStatus)
  announceProfile()
}

function announceProfile() {
  const { set, connected } = useClasp()
  if (!connected.value) return
  set(`${ADDR.USER_PROFILE}/${userId.value}/profile`, {
    name: displayName.value,
    avatarColor: avatarColor.value,
    userId: userId.value,
    status: status.value,
  })
}

export function useIdentity() {
  return {
    userId,
    displayName,
    avatarColor,
    status,
    setUserId,
    setDisplayName,
    setAvatarColor,
    setStatus,
    announceProfile,
  }
}
