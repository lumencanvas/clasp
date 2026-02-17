import { ref, readonly } from 'vue'

const unreadCounts = ref(new Map()) // roomId -> count
const toasts = ref([])
let toastId = 0

function incrementUnread(roomId) {
  const current = unreadCounts.value.get(roomId) || 0
  unreadCounts.value.set(roomId, current + 1)
  // Trigger reactivity
  unreadCounts.value = new Map(unreadCounts.value)
}

function markRead(roomId) {
  unreadCounts.value.delete(roomId)
  unreadCounts.value = new Map(unreadCounts.value)
}

function getUnread(roomId) {
  return unreadCounts.value.get(roomId) || 0
}

function addToast(message, type = 'info', duration = 3000) {
  const id = ++toastId
  toasts.value.push({ id, message, type })
  setTimeout(() => removeToast(id), duration)
}

function removeToast(id) {
  toasts.value = toasts.value.filter(t => t.id !== id)
}

let permissionGranted = false

async function requestPermission() {
  if (!('Notification' in window)) return false
  if (Notification.permission === 'granted') {
    permissionGranted = true
    return true
  }
  if (Notification.permission === 'denied') return false
  const result = await Notification.requestPermission()
  permissionGranted = result === 'granted'
  return permissionGranted
}

function notifyMessage(roomId, sender, text) {
  if (!permissionGranted || document.hasFocus()) return
  const n = new Notification(`${sender} in #${roomId}`, {
    body: text.slice(0, 100),
    icon: '/favicon.svg',
    tag: `clasp-chat-${roomId}`,
  })
  setTimeout(() => n.close(), 4000)
}

export function useNotifications() {
  return {
    unreadCounts: readonly(unreadCounts),
    toasts,
    incrementUnread,
    markRead,
    getUnread,
    addToast,
    removeToast,
    requestPermission,
    notifyMessage,
  }
}
