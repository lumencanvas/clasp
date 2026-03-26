import { ref, readonly } from 'vue'

const defraUrl = ref('')
const defraHealthy = ref(false)
const defraEnabled = ref(false)
const checking = ref(false)

export function useDefra() {
  async function checkHealth() {
    if (!defraUrl.value) {
      defraHealthy.value = false
      return
    }
    checking.value = true
    try {
      const api = (window as any).clasp
      if (api?.defraHealthCheck) {
        const result = await api.defraHealthCheck(defraUrl.value)
        defraHealthy.value = result.healthy
      }
    } catch {
      defraHealthy.value = false
    } finally {
      checking.value = false
    }
  }

  async function exportConfig() {
    if (!defraUrl.value) return null
    try {
      const api = (window as any).clasp
      if (api?.defraConfigExport) {
        return await api.defraConfigExport(defraUrl.value)
      }
    } catch {
      return null
    }
  }

  async function importConfig(config: any) {
    if (!defraUrl.value) return false
    try {
      const api = (window as any).clasp
      if (api?.defraConfigImport) {
        const result = await api.defraConfigImport(defraUrl.value, config)
        return result.success
      }
    } catch {
      return false
    }
  }

  function setUrl(url: string) {
    defraUrl.value = url
    if (url) {
      checkHealth()
    } else {
      defraHealthy.value = false
    }
  }

  function setEnabled(enabled: boolean) {
    defraEnabled.value = enabled
  }

  return {
    url: readonly(defraUrl),
    healthy: readonly(defraHealthy),
    enabled: readonly(defraEnabled),
    checking: readonly(checking),
    setUrl,
    setEnabled,
    checkHealth,
    exportConfig,
    importConfig,
  }
}
