import type { ClaspAPI } from '../lib/types'

function getApi(): ClaspAPI | null {
  return (window as any).clasp ?? null
}

export function useElectron() {
  const api = getApi()
  const available = !!api

  // Wrap every API call with null check
  async function invoke<T>(method: keyof ClaspAPI, ...args: any[]): Promise<T | null> {
    const a = getApi()
    if (!a || typeof a[method] !== 'function') return null
    return (a[method] as any)(...args)
  }

  // Event listeners that return cleanup functions
  function on(event: string, callback: (...args: any[]) => void): () => void {
    const a = getApi()
    if (!a) return () => {}
    const methodName = `on${event.charAt(0).toUpperCase()}${event.slice(1)}` as keyof ClaspAPI
    if (typeof a[methodName] === 'function') {
      return (a[methodName] as any)(callback)
    }
    return () => {}
  }

  return {
    available,
    api,
    invoke,
    on,
  }
}
