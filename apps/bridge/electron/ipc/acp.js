const { ipcMain } = require('electron')

/**
 * ACP (Access Control Policy) IPC handlers.
 *
 * These handle relationship management for DefraDB ACP. Active only
 * when DefraDB is connected and ACP is enabled. All operations require
 * a secp256k1 identity hex string.
 */
function registerAcpHandlers() {
  ipcMain.handle('acp-add-relationship', async (event, { collection, docId, relation, actor, identity }) => {
    try {
      const url = global.defraUrl
      if (!url) return { success: false, error: 'DefraDB not connected' }

      const resp = await fetch(`${url}/api/v0/acp/relationship`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `bearer ${identity}`,
        },
        body: JSON.stringify({ collection, docID: docId, relation, actor }),
      })

      if (!resp.ok) {
        const text = await resp.text()
        return { success: false, error: `${resp.status}: ${text}` }
      }

      const data = await resp.json()
      return { success: true, existedAlready: data.ExistedAlready || false }
    } catch (e) {
      return { success: false, error: e.message }
    }
  })

  ipcMain.handle('acp-delete-relationship', async (event, { collection, docId, relation, actor, identity }) => {
    try {
      const url = global.defraUrl
      if (!url) return { success: false, error: 'DefraDB not connected' }

      const resp = await fetch(`${url}/api/v0/acp/relationship`, {
        method: 'DELETE',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `bearer ${identity}`,
        },
        body: JSON.stringify({ collection, docID: docId, relation, actor }),
      })

      if (!resp.ok) {
        const text = await resp.text()
        return { success: false, error: `${resp.status}: ${text}` }
      }

      const data = await resp.json()
      return { success: true, recordFound: data.RecordFound || false }
    } catch (e) {
      return { success: false, error: e.message }
    }
  })
}

module.exports = { registerAcpHandlers }
