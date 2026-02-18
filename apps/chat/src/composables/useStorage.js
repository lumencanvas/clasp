import { saveMessage, loadMessages, exportAll, importAll } from '../lib/storage.js'

/**
 * Reactive storage composable for message persistence.
 * Loads cached messages on join, saves incoming messages in background.
 */
export function useStorage() {
  /**
   * Load cached messages for a room. Returns array of message objects.
   */
  async function loadCachedMessages(roomId) {
    try {
      return await loadMessages(roomId, 100)
    } catch {
      return []
    }
  }

  /**
   * Persist a message to IndexedDB. Fire-and-forget.
   */
  function persistMessage(roomId, msg) {
    if (!msg.msgId) return
    saveMessage(roomId, msg).catch(() => {})
  }

  /**
   * Export all stored data as a downloadable JSON file.
   */
  async function exportData() {
    const data = await exportAll()
    const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `clasp-chat-backup-${Date.now()}.json`
    a.click()
    URL.revokeObjectURL(url)
  }

  /**
   * Import data from a JSON file.
   */
  async function importData(file) {
    const text = await file.text()
    const json = JSON.parse(text)
    await importAll(json)
  }

  return {
    loadCachedMessages,
    persistMessage,
    exportData,
    importData,
  }
}
