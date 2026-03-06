import { ref, readonly, computed } from 'vue'
import type { LogEntry } from '../lib/types'

const systemLogs = ref<LogEntry[]>([])
const serverLogs = ref(new Map<string, LogEntry[]>())
const levelFilter = ref('all')
const serverFilter = ref('all')
const searchQuery = ref('')

function addServerLog(serverId: string, log: LogEntry) {
  if (!serverLogs.value.has(serverId)) {
    serverLogs.value.set(serverId, [])
  }
  const logs = serverLogs.value.get(serverId)!
  logs.push(log)
  if (logs.length > 500) logs.shift()
}

function addSystemLog(log: LogEntry) {
  systemLogs.value.push(log)
  if (systemLogs.value.length > 500) systemLogs.value.shift()
}

function clear() {
  systemLogs.value = []
  serverLogs.value.clear()
}

const allLogs = computed(() => {
  let logs: (LogEntry & { source?: string })[] = [...systemLogs.value]
  serverLogs.value.forEach((entries, serverId) => {
    entries.forEach(log => logs.push({ ...log, source: serverId }))
  })
  logs.sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime())
  return logs
})

const filteredLogs = computed(() => {
  let logs = allLogs.value
  if (levelFilter.value !== 'all') {
    const levels: Record<string, string[]> = {
      error: ['error'],
      warning: ['error', 'warning'],
      info: ['error', 'warning', 'info'],
      debug: ['error', 'warning', 'info', 'debug'],
    }
    logs = logs.filter(l => levels[levelFilter.value]?.includes(l.level))
  }
  if (serverFilter.value !== 'all') {
    logs = logs.filter(l => l.source === serverFilter.value)
  }
  if (searchQuery.value) {
    const q = searchQuery.value.toLowerCase()
    logs = logs.filter(l =>
      l.message?.toLowerCase().includes(q) ||
      l.source?.toLowerCase().includes(q) ||
      l.level?.toLowerCase().includes(q)
    )
  }
  return logs.slice(0, 500)
})

const logCounts = computed(() => ({
  error: allLogs.value.filter(l => l.level === 'error').length,
  warning: allLogs.value.filter(l => l.level === 'warning').length,
  info: allLogs.value.filter(l => l.level === 'info').length,
  debug: allLogs.value.filter(l => l.level === 'debug').length,
}))

function setLevelFilter(level: string) { levelFilter.value = level }
function setServerFilter(server: string) { serverFilter.value = server }
function setSearchQuery(query: string) { searchQuery.value = query }

function exportLogs(): string {
  return allLogs.value.map(log =>
    `[${new Date(log.timestamp).toISOString()}] [${log.level.toUpperCase()}] [${log.source || 'System'}] ${log.message}`
  ).join('\n')
}

export function useLogs() {
  return {
    systemLogs: readonly(systemLogs),
    serverLogs: readonly(serverLogs),
    filteredLogs,
    logCounts,
    levelFilter: readonly(levelFilter),
    serverFilter: readonly(serverFilter),
    searchQuery: readonly(searchQuery),
    addServerLog,
    addSystemLog,
    clear,
    setLevelFilter,
    setServerFilter,
    setSearchQuery,
    exportLogs,
  }
}
