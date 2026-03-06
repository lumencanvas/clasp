<script setup lang="ts">
import type { Signal } from '../../lib/types'

const props = defineProps<{ signal: Signal }>()

function protocolClass(protocol: string): string {
  return `protocol-${protocol || 'unknown'}`
}

function formatValue(val: any): string {
  if (val === undefined || val === null) return '—'
  if (typeof val === 'number') return val.toFixed(2)
  if (typeof val === 'string') return val.length > 20 ? val.slice(0, 20) + '...' : val
  return String(val)
}

function barWidth(val: any): string {
  const n = typeof val === 'number' ? val : parseFloat(val)
  if (isNaN(n)) return '0%'
  return `${Math.min(Math.max(n * 100, 0), 100)}%`
}
</script>

<template>
  <div class="signal-item" :class="{ 'signal-forwarded': signal.forwarded }">
    <span class="signal-direction">{{ signal.forwarded ? '->' : '<-' }}</span>
    <span class="signal-protocol-badge" :class="protocolClass(signal.protocol)">
      {{ (signal.protocol || '?').toUpperCase() }}
    </span>
    <span class="signal-address">{{ signal.path || signal.address || '—' }}</span>
    <span class="signal-value">{{ formatValue(signal.value) }}</span>
    <div class="signal-bar">
      <div class="signal-bar-fill" :style="{ width: barWidth(signal.value) }"></div>
    </div>
  </div>
</template>
