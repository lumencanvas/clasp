import { ref, readonly } from 'vue'
import type { Signal } from '../lib/types'

const active = ref(false)
const target = ref<'source' | 'target' | null>(null)
const learnedSignal = ref<Signal | null>(null)

function start(side: 'source' | 'target') {
  active.value = true
  target.value = side
  learnedSignal.value = null
}

function stop() {
  active.value = false
  target.value = null
}

function handleSignal(signal: Signal): boolean {
  if (!active.value) return false
  learnedSignal.value = signal
  stop()
  return true
}

function detectProtocol(signal: Signal): string {
  if (signal.address?.startsWith('/mqtt/')) return 'mqtt'
  if (signal.address?.startsWith('/osc/')) return 'osc'
  if (signal.address?.startsWith('/')) return 'clasp'
  if (signal.topic !== undefined) return 'mqtt'
  if (signal.channel !== undefined && (signal.note !== undefined || signal.cc !== undefined)) return 'midi'
  if (signal.universe !== undefined) return 'artnet'
  return 'clasp'
}

export function useLearnMode() {
  return {
    active: readonly(active),
    target: readonly(target),
    learnedSignal: readonly(learnedSignal),
    start,
    stop,
    handleSignal,
    detectProtocol,
  }
}
