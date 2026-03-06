import { useElectron } from './useElectron'
import { useRouters } from './useRouters'
import { useConnections } from './useConnections'
import { useBridges } from './useBridges'
import { useRoutes } from './useRoutes'
import { useNotifications } from './useNotifications'
import { exportConfig, importConfig, downloadConfig, loadConfigFromFile, mergeConfig } from '../lib/config-io.js'

export function useConfig() {
  const { invoke } = useElectron()
  const { notify } = useNotifications()

  async function exportToFile() {
    const { routers } = useRouters()
    const { connections } = useConnections()
    const { bridges } = useBridges()
    const { routes } = useRoutes()

    const state = {
      servers: connections.value,
      bridges: bridges.value,
      mappings: routes.value,
      routers: routers.value,
    }

    try {
      const api = (window as any).clasp
      if (api) {
        const result = await api.showSaveDialog({
          title: 'Export CLASP Configuration',
          defaultPath: 'clasp-config.json',
        })
        if (!result.canceled && result.filePath) {
          const config = exportConfig(state)
          await api.writeFile(result.filePath, JSON.stringify(config, null, 2))
          notify('Configuration exported successfully!', 'success')
        }
      } else {
        downloadConfig(state)
        notify('Configuration downloaded!', 'success')
      }
    } catch (e: any) {
      notify(`Export failed: ${e.message}`, 'error')
    }
  }

  async function importFromFile() {
    try {
      const api = (window as any).clasp
      if (api) {
        const result = await api.showOpenDialog({ title: 'Import CLASP Configuration' })
        if (!result.canceled && result.filePaths?.length > 0) {
          const fileResult = await api.readFile(result.filePaths[0])
          if (fileResult.success) {
            const config = JSON.parse(fileResult.content)
            const validated = importConfig(config)
            await applyImportedConfig(validated)
            notify('Configuration imported successfully!', 'success')
          } else {
            notify(`Failed to read file: ${fileResult.error}`, 'error')
          }
        }
      } else {
        // Browser fallback
        const input = document.createElement('input')
        input.type = 'file'
        input.accept = '.json'
        input.onchange = async (e: any) => {
          const file = e.target.files[0]
          if (file) {
            try {
              const validated = await loadConfigFromFile(file)
              await applyImportedConfig(validated)
              notify('Configuration imported successfully!', 'success')
            } catch (err: any) {
              notify(`Import failed: ${err.message}`, 'error')
            }
          }
        }
        input.click()
      }
    } catch (e: any) {
      notify(`Import failed: ${e.message}`, 'error')
    }
  }

  async function applyImportedConfig(config: any) {
    const { connections, remove: removeConn, add: addConn } = useConnections()
    const { bridges, remove: removeBridge, add: addBridge } = useBridges()
    const { routes: existingRoutes, remove: removeRoute, add: addRoute } = useRoutes()

    // Stop and remove existing connections
    for (const conn of [...connections.value]) {
      await removeConn(conn.id)
    }

    // Remove existing bridges
    for (const bridge of [...bridges.value]) {
      await removeBridge(bridge.id)
    }

    // Remove existing routes
    for (const route of [...existingRoutes.value]) {
      removeRoute(route.id)
    }

    // Add imported connections (servers)
    for (const server of config.servers) {
      await addConn(server)
    }

    // Add imported bridges
    for (const bridge of config.bridges) {
      await addBridge(bridge)
    }

    // Add imported mappings (routes)
    for (const mapping of config.mappings) {
      addRoute(mapping)
    }
  }

  return {
    exportToFile,
    importFromFile,
  }
}
