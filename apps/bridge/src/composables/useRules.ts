import { ref, readonly, computed } from 'vue'
import type { Rule, RuleTrigger, RuleAction, RuleCondition } from '../lib/types'
import { loadFromStorage, saveToStorage } from './useStorage'
import { useElectron } from './useElectron'
import { useNotifications } from './useNotifications'

const rules = ref<Rule[]>(loadFromStorage<Rule[]>('rules', []))

function saveRules() {
  saveToStorage('rules', rules.value)
}

function add(rule: Omit<Rule, 'id'>) {
  const newRule: Rule = { ...rule, id: Date.now().toString() }
  rules.value.push(newRule)
  saveRules()
  return newRule
}

function update(id: string, patch: Partial<Rule>) {
  const idx = rules.value.findIndex(r => r.id === id)
  if (idx !== -1) {
    rules.value[idx] = { ...rules.value[idx], ...patch }
    saveRules()
  }
}

function remove(id: string) {
  rules.value = rules.value.filter(r => r.id !== id)
  saveRules()
}

function toggle(id: string) {
  const rule = rules.value.find(r => r.id === id)
  if (rule) {
    rule.enabled = !rule.enabled
    saveRules()
  }
}

function duplicate(id: string) {
  const rule = rules.value.find(r => r.id === id)
  if (rule) {
    const copy: Rule = {
      ...JSON.parse(JSON.stringify(rule)),
      id: Date.now().toString(),
      name: `${rule.name} (copy)`,
    }
    rules.value.push(copy)
    saveRules()
  }
}

function createEmptyTrigger(): RuleTrigger {
  return { type: 'on_change', address: '' }
}

function createEmptyCondition(): RuleCondition {
  return { address: '', operator: 'eq', value: 0 }
}

function createEmptyAction(): RuleAction {
  return { type: 'set', address: '', value: 0 }
}

function createEmptyRule(): Omit<Rule, 'id'> {
  return {
    name: 'New Rule',
    enabled: true,
    trigger: createEmptyTrigger(),
    conditions: [],
    actions: [createEmptyAction()],
    cooldown: 0,
  }
}

// Export rules to JSON file for use with --rules flag
async function exportRulesFile(routerId?: string): Promise<string | null> {
  const { invoke } = useElectron()
  const { notify } = useNotifications()

  const enabledRules = rules.value.filter(r => r.enabled)
  if (enabledRules.length === 0) {
    notify('No enabled rules to export', 'warning')
    return null
  }

  const rulesJson = JSON.stringify(
    enabledRules.map(({ id, enabled, ...rest }) => rest),
    null,
    2
  )

  try {
    const result = await invoke<{ canceled: boolean; filePath?: string }>('showSaveDialog', {
      title: 'Save Rules File',
      defaultPath: 'rules.json',
      filters: [{ name: 'JSON', extensions: ['json'] }],
    })

    if (result?.canceled || !result?.filePath) return null

    await invoke('writeFile', result.filePath, rulesJson)
    notify(`Rules exported to ${result.filePath}`, 'success')
    return result.filePath
  } catch (e: any) {
    notify(`Failed to export rules: ${e.message}`, 'error')
    return null
  }
}

const enabledCount = computed(() => rules.value.filter(r => r.enabled).length)

export function useRules() {
  return {
    rules: readonly(rules),
    enabledCount,
    add,
    update,
    remove,
    toggle,
    duplicate,
    createEmptyRule,
    createEmptyTrigger,
    createEmptyCondition,
    createEmptyAction,
    exportRulesFile,
    saveRules,
  }
}
