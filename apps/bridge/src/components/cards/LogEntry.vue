<script setup lang="ts">
import type { LogEntry as LogEntryType } from '../../lib/types'

const props = defineProps<{
  entry: LogEntryType
  highlight?: string
}>()

function formatTime(ts: number | string): string {
  const d = new Date(ts)
  return d.toLocaleTimeString('en-US', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' })
}

function highlightText(text: string): string {
  if (!props.highlight || !text) return text
  const escaped = props.highlight.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  return text.replace(new RegExp(`(${escaped})`, 'gi'), '<mark>$1</mark>')
}
</script>

<template>
  <div class="log-entry" :class="[`log-entry-${entry.level || 'info'}`]">
    <span class="log-timestamp">{{ formatTime(entry.timestamp) }}</span>
    <span class="log-level" :class="`log-level-${entry.level || 'info'}`">
      {{ (entry.level || 'info').toUpperCase() }}
    </span>
    <span v-if="entry.source" class="log-source">{{ entry.source }}</span>
    <span class="log-message" v-html="highlightText(entry.message)"></span>
  </div>
</template>
