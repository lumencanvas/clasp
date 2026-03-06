import { ref, readonly } from 'vue'
import type { AppConfig, Scope, WriteRule, WriteCheck, VisibilityRule, SnapshotTransform, RateLimits } from '../lib/types'
import { loadFromStorage, saveToStorage } from './useStorage'
import { useElectron } from './useElectron'
import { useNotifications } from './useNotifications'

const defaultRateLimits: RateLimits = {
  loginMaxAttempts: 5,
  loginWindow: 300,
  registerMaxAttempts: 3,
  registerWindow: 3600,
}

const appConfig = ref<AppConfig>(loadFromStorage<AppConfig>('appConfig', {
  scopes: [],
  writeRules: [],
  visibility: [],
  snapshotTransforms: [],
  rateLimits: { ...defaultRateLimits },
}))

function save() {
  saveToStorage('appConfig', appConfig.value)
}

// Scopes
function addScope(scope: Omit<Scope, 'id'>) {
  appConfig.value.scopes.push({ ...scope, id: Date.now().toString() })
  save()
}

function removeScope(id: string) {
  appConfig.value.scopes = appConfig.value.scopes.filter(s => s.id !== id)
  save()
}

function updateScope(id: string, patch: Partial<Scope>) {
  const idx = appConfig.value.scopes.findIndex(s => s.id === id)
  if (idx !== -1) {
    appConfig.value.scopes[idx] = { ...appConfig.value.scopes[idx], ...patch }
    save()
  }
}

// Scope templates
const scopeTemplates = [
  { name: 'User Own Data', action: 'write', pattern: '/users/{user_id}/**' },
  { name: 'Room Messages', action: 'write', pattern: '/rooms/*/messages' },
  { name: 'Global Read', action: 'read', pattern: '/**' },
  { name: 'Admin', action: '*', pattern: '/**' },
]

function applyScopeTemplate(template: typeof scopeTemplates[number]) {
  addScope({ name: template.name, action: template.action, pattern: template.pattern })
}

// Write rules
function addWriteRule(rule: Omit<WriteRule, 'id'>) {
  appConfig.value.writeRules.push({ ...rule, id: Date.now().toString() })
  save()
}

function removeWriteRule(id: string) {
  appConfig.value.writeRules = appConfig.value.writeRules.filter(r => r.id !== id)
  save()
}

function updateWriteRule(id: string, patch: Partial<WriteRule>) {
  const idx = appConfig.value.writeRules.findIndex(r => r.id === id)
  if (idx !== -1) {
    appConfig.value.writeRules[idx] = { ...appConfig.value.writeRules[idx], ...patch }
    save()
  }
}

function addCheckToWriteRule(ruleId: string, phase: 'preChecks' | 'checks', check: WriteCheck) {
  const rule = appConfig.value.writeRules.find(r => r.id === ruleId)
  if (!rule) return
  rule[phase] = [...rule[phase], check]
  save()
}

function removeCheckFromWriteRule(ruleId: string, phase: 'preChecks' | 'checks', idx: number) {
  const rule = appConfig.value.writeRules.find(r => r.id === ruleId)
  if (!rule) return
  rule[phase] = rule[phase].filter((_, i) => i !== idx)
  save()
}

// Visibility
function addVisibilityRule(rule: Omit<VisibilityRule, 'id'>) {
  appConfig.value.visibility.push({ ...rule, id: Date.now().toString() })
  save()
}

function removeVisibilityRule(id: string) {
  appConfig.value.visibility = appConfig.value.visibility.filter(v => v.id !== id)
  save()
}

function updateVisibilityRule(id: string, patch: Partial<VisibilityRule>) {
  const idx = appConfig.value.visibility.findIndex(v => v.id === id)
  if (idx !== -1) {
    appConfig.value.visibility[idx] = { ...appConfig.value.visibility[idx], ...patch }
    save()
  }
}

// Snapshot transforms
function addSnapshotTransform(t: Omit<SnapshotTransform, 'id'>) {
  appConfig.value.snapshotTransforms.push({ ...t, id: Date.now().toString() })
  save()
}

function removeSnapshotTransform(id: string) {
  appConfig.value.snapshotTransforms = appConfig.value.snapshotTransforms.filter(t => t.id !== id)
  save()
}

function updateSnapshotTransform(id: string, patch: Partial<SnapshotTransform>) {
  const idx = appConfig.value.snapshotTransforms.findIndex(t => t.id === id)
  if (idx !== -1) {
    appConfig.value.snapshotTransforms[idx] = { ...appConfig.value.snapshotTransforms[idx], ...patch }
    save()
  }
}

// Rate limits
function updateRateLimits(limits: Partial<RateLimits>) {
  appConfig.value.rateLimits = { ...appConfig.value.rateLimits, ...limits }
  save()
}

// Export app-config.json
async function exportAppConfig(): Promise<string | null> {
  const { invoke } = useElectron()
  const { notify } = useNotifications()

  const output = {
    scopes: appConfig.value.scopes.map(({ id, ...rest }) => rest),
    write_rules: appConfig.value.writeRules.map(({ id, ...rest }) => ({
      path: rest.path,
      pre_checks: rest.preChecks,
      checks: rest.checks,
    })),
    snapshot: {
      visibility: appConfig.value.visibility.map(({ id, ...rest }) => rest),
      transforms: appConfig.value.snapshotTransforms.map(({ id, ...rest }) => ({
        path: rest.path,
        redact_fields: rest.redactFields,
      })),
    },
    rate_limits: {
      login: {
        max_attempts: appConfig.value.rateLimits.loginMaxAttempts,
        window: appConfig.value.rateLimits.loginWindow,
      },
      register: {
        max_attempts: appConfig.value.rateLimits.registerMaxAttempts,
        window: appConfig.value.rateLimits.registerWindow,
      },
    },
  }

  try {
    const result = await invoke<{ canceled: boolean; filePath?: string }>('showSaveDialog', {
      title: 'Save App Config',
      defaultPath: 'app-config.json',
      filters: [{ name: 'JSON', extensions: ['json'] }],
    })

    if (result?.canceled || !result?.filePath) return null

    await invoke('writeFile', result.filePath, JSON.stringify(output, null, 2))
    notify(`App config exported to ${result.filePath}`, 'success')
    return result.filePath
  } catch (e: any) {
    notify(`Failed to export: ${e.message}`, 'error')
    return null
  }
}

export function useAppConfig() {
  return {
    appConfig: readonly(appConfig),
    scopeTemplates,
    addScope,
    removeScope,
    updateScope,
    applyScopeTemplate,
    addWriteRule,
    removeWriteRule,
    updateWriteRule,
    addCheckToWriteRule,
    removeCheckFromWriteRule,
    addVisibilityRule,
    removeVisibilityRule,
    updateVisibilityRule,
    addSnapshotTransform,
    removeSnapshotTransform,
    updateSnapshotTransform,
    updateRateLimits,
    exportAppConfig,
  }
}
