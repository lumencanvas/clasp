import { ref, readonly, computed } from 'vue'
import type { SignalRoute, Signal, SignalEndpoint, TransformConfig } from '../lib/types'
import { loadFromStorage, saveToStorage } from './useStorage'
import { applyTransform } from '../lib/transforms'
import { matchPattern } from '../lib/pattern-cache'

const routes = ref<SignalRoute[]>([])
const editingRoute = ref<string | null>(null)

function saveRoutes() {
  saveToStorage('mappings', routes.value)
}

function load() {
  routes.value = loadFromStorage<SignalRoute[]>('mappings', [])
}

function add(route: SignalRoute) {
  if (editingRoute.value) {
    const idx = routes.value.findIndex(r => r.id === editingRoute.value)
    if (idx >= 0) routes.value[idx] = route
  } else {
    routes.value.push(route)
  }
  editingRoute.value = null
  saveRoutes()
}

function remove(id: string) {
  routes.value = routes.value.filter(r => r.id !== id)
  saveRoutes()
}

function edit(id: string) {
  editingRoute.value = id
}

function cancelEdit() {
  editingRoute.value = null
}

function toggle(id: string) {
  const route = routes.value.find(r => r.id === id)
  if (route) {
    route.enabled = !route.enabled
    saveRoutes()
  }
}

// Signal matching & routing
function matchesSource(signal: Signal, source: SignalEndpoint): boolean {
  switch (source.protocol) {
    case 'clasp':
    case 'osc':
      if (!signal.address) return false
      if (source.address) return matchPattern(source.address, signal.address)
      return true
    case 'midi':
      if (source.midiChannel && signal.channel !== source.midiChannel) return false
      if (source.midiNumber != null && signal.note !== source.midiNumber && signal.cc !== source.midiNumber) return false
      return true
    case 'dmx':
    case 'artnet':
      if (source.dmxUniverse != null && signal.universe !== source.dmxUniverse) return false
      if (source.dmxChannel != null && signal.channel !== source.dmxChannel) return false
      return true
    default:
      return false
  }
}

function extractValue(signal: Signal, source: SignalEndpoint): number {
  let value: any
  if (typeof signal.value === 'number') value = signal.value
  else if (signal.velocity !== undefined) value = signal.velocity / 127
  else if (signal.value !== undefined) value = signal.value
  else value = 0

  if (source.jsonPath && typeof value === 'object') {
    try {
      const parts = source.jsonPath.replace(/^\$\.?/, '').split(/\.|\[|\]/).filter(Boolean)
      let result = value
      for (const part of parts) {
        if (result == null) return 0
        result = result[part]
      }
      value = result
    } catch { /* ignore */ }
  }

  return typeof value === 'number' ? value : 0
}

function processSignal(signal: Signal): void {
  const api = (window as any).clasp
  if (!api?.sendSignal) return

  for (const route of routes.value) {
    if (!route.enabled) continue
    if (!matchesSource(signal, route.source)) continue

    const value = extractValue(signal, route.source)
    const transformed = applyTransform(value, route.transform)
    if (!Number.isFinite(transformed)) continue
    const targetAddress = route.target.address || signal.address || '/*'

    api.sendSignal(signal.bridgeId || '', targetAddress, transformed)
  }
}

const routeCount = computed(() => routes.value.length)

export function useRoutes() {
  return {
    routes: readonly(routes),
    editingRoute: readonly(editingRoute),
    routeCount,
    load,
    add,
    remove,
    edit,
    cancelEdit,
    toggle,
    processSignal,
    saveRoutes,
  }
}
