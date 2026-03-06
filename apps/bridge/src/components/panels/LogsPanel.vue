<script setup lang="ts">
import { ref, nextTick, watch } from 'vue'
import { useLogs } from '../../composables/useLogs'
import LogEntry from '../cards/LogEntry.vue'

const {
  filteredLogs,
  logCounts,
  systemLogs,
  clear,
  exportLogs,
  setLevelFilter,
  setSearchQuery,
  searchQuery,
  levelFilter,
} = useLogs()

const logViewerRef = ref<HTMLDivElement | null>(null)

function onFilterText(e: Event) {
  setSearchQuery((e.target as HTMLInputElement).value)
}

function onFilterLevel(e: Event) {
  setLevelFilter((e.target as HTMLSelectElement).value)
}

watch(filteredLogs, () => {
  nextTick(() => {
    const el = logViewerRef.value
    if (el) el.scrollTop = el.scrollHeight
  })
})
</script>

<template>
  <div style="display: flex; flex-direction: column; height: 100%;">
    <div class="panel-toolbar">
      <span class="panel-title">LOGS</span>
      <div class="toolbar-group">
        <input
          :value="searchQuery"
          @input="onFilterText"
          class="input input-sm"
          placeholder="Search logs..."
          style="width: 180px;"
        />
        <select :value="levelFilter" @change="onFilterLevel" class="select select-sm">
          <option value="all">All levels</option>
          <option value="error">Error</option>
          <option value="warning">Warning</option>
          <option value="info">Info</option>
          <option value="debug">Debug</option>
        </select>
        <button class="btn btn-sm btn-secondary" @click="exportLogs">EXPORT</button>
        <button class="btn btn-sm btn-secondary" @click="clear">CLEAR</button>
      </div>
    </div>
    <div class="log-stats">
      <span class="log-stat log-stat-error">Errors: {{ logCounts.error || 0 }}</span>
      <span class="log-stat log-stat-warning">Warnings: {{ logCounts.warning || 0 }}</span>
      <span class="log-stat log-stat-info">Info: {{ logCounts.info || 0 }}</span>
      <span v-if="searchQuery" class="log-stat log-stat-search">Matches: {{ filteredLogs.length }}</span>
    </div>
    <div class="panel-content panel-logs-content" style="flex: 1;">
      <div ref="logViewerRef" class="log-viewer">
        <template v-if="filteredLogs.length > 0">
          <LogEntry v-for="(entry, i) in filteredLogs" :key="i" :entry="entry" :highlight="searchQuery" />
        </template>
        <div v-else class="log-empty">
          {{ systemLogs.length === 0 ? 'No logs yet' : 'No matching logs' }}
        </div>
      </div>
    </div>
  </div>
</template>
