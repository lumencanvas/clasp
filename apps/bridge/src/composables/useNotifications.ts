import { ref, readonly } from 'vue'
import type { Notification, NotificationType } from '../lib/types'

const notifications = ref<Notification[]>([])
let idCounter = 0

function notify(message: string, type: NotificationType = 'info', duration = 5000) {
  const id = String(++idCounter)
  const notification: Notification = {
    id,
    message,
    type,
    timestamp: Date.now(),
  }
  notifications.value.push(notification)

  if (duration > 0) {
    setTimeout(() => dismiss(id), duration)
  }
}

function dismiss(id: string) {
  notifications.value = notifications.value.filter(n => n.id !== id)
}

function clear() {
  notifications.value = []
}

export function useNotifications() {
  return {
    notifications: readonly(notifications),
    notify,
    dismiss,
    clear,
  }
}
