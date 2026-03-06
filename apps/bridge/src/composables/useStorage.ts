import { watch, type Ref } from 'vue'

const STORAGE_PREFIX = 'clasp-bridge'

export function loadFromStorage<T>(key: string, fallback: T): T {
  try {
    const raw = localStorage.getItem(`${STORAGE_PREFIX}-${key}`)
    if (raw) return JSON.parse(raw)
  } catch (e) {
    console.error(`Failed to load ${key} from storage:`, e)
  }
  return fallback
}

export function saveToStorage(key: string, data: any): void {
  try {
    localStorage.setItem(`${STORAGE_PREFIX}-${key}`, JSON.stringify(data))
  } catch (e) {
    console.error(`Failed to save ${key} to storage:`, e)
  }
}

export function removeFromStorage(key: string): void {
  localStorage.removeItem(`${STORAGE_PREFIX}-${key}`)
}

// Auto-persist a ref whenever it changes (debounced)
export function autoPersist<T>(key: string, source: Ref<T>, debounceMs = 300): void {
  let timeout: ReturnType<typeof setTimeout> | null = null
  watch(source, (val) => {
    if (timeout) clearTimeout(timeout)
    timeout = setTimeout(() => saveToStorage(key, val), debounceMs)
  }, { deep: true })
}

export function useStorage() {
  return {
    load: loadFromStorage,
    save: saveToStorage,
    remove: removeFromStorage,
    autoPersist,
  }
}
