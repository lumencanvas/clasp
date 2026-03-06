<script setup lang="ts">
import type { Rule } from '../../lib/types'

defineProps<{ rule: Rule }>()
defineEmits<{
  edit: [rule: Rule]
  toggle: [id: string]
  duplicate: [id: string]
  delete: [id: string]
}>()

function triggerLabel(rule: Rule): string {
  const t = rule.trigger
  switch (t.type) {
    case 'on_change': return `Change: ${t.address || '(any)'}`
    case 'on_threshold': return `Threshold: ${t.address || '?'} ${t.direction || 'both'} ${t.threshold ?? 0.5}`
    case 'on_event': return `Event: ${t.event || '(any)'}`
    case 'on_interval': return `Every ${t.seconds ?? 1}s`
    default: return t.type
  }
}

function actionsSummary(rule: Rule): string {
  if (rule.actions.length === 0) return 'No actions'
  if (rule.actions.length === 1) {
    const a = rule.actions[0]
    return `${a.type}: ${a.address}`
  }
  return `${rule.actions.length} actions`
}
</script>

<template>
  <div class="rule-card" :class="{ disabled: !rule.enabled }">
    <div class="rule-header">
      <button class="rule-toggle" :aria-label="rule.enabled ? 'Disable rule' : 'Enable rule'" :class="{ on: rule.enabled }" @click="$emit('toggle', rule.id)">
        {{ rule.enabled ? 'ON' : 'OFF' }}
      </button>
      <div class="rule-info" @click="$emit('edit', rule)">
        <span class="rule-name">{{ rule.name }}</span>
        <span class="rule-trigger">{{ triggerLabel(rule) }}</span>
      </div>
      <div class="rule-actions">
        <button class="btn-icon" title="Edit" @click="$emit('edit', rule)">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7" />
            <path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z" />
          </svg>
        </button>
        <button class="btn-icon" title="Duplicate" @click="$emit('duplicate', rule.id)">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
            <path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1" />
          </svg>
        </button>
        <button class="btn-icon btn-icon--danger" title="Delete" @click="$emit('delete', rule.id)">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="3 6 5 6 21 6" />
            <path d="M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2" />
          </svg>
        </button>
      </div>
    </div>
    <div class="rule-meta">
      <span class="rule-badge">{{ actionsSummary(rule) }}</span>
      <span v-if="rule.conditions.length" class="rule-badge">{{ rule.conditions.length }} condition{{ rule.conditions.length > 1 ? 's' : '' }}</span>
      <span v-if="rule.cooldown" class="rule-badge">{{ rule.cooldown }}ms cooldown</span>
    </div>
  </div>
</template>

<style scoped>
.rule-card {
  background: var(--color-surface);
  border: var(--border-width) solid var(--color-border-dark);
  box-shadow: var(--shadow-sm);
  padding: var(--space-sm) var(--space-md);
  transition: opacity 0.15s;
}

.rule-card.disabled {
  opacity: 0.5;
}

.rule-header {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.rule-toggle {
  font-family: var(--font-mono);
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 1px;
  padding: 2px 6px;
  border: var(--border-width) solid var(--stone-900);
  background: var(--stone-300);
  color: var(--stone-600);
  cursor: pointer;
}

.rule-toggle.on {
  background: var(--color-accent);
  color: white;
}

.rule-info {
  flex: 1;
  min-width: 0;
  cursor: pointer;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.rule-name {
  font-family: var(--font-mono);
  font-size: 12px;
  font-weight: 600;
}

.rule-trigger {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--color-text-muted);
}

.rule-actions {
  display: flex;
  gap: 2px;
  flex-shrink: 0;
}

.btn-icon {
  background: none;
  border: none;
  padding: 4px;
  cursor: pointer;
  color: var(--color-text-muted);
  display: flex;
}

.btn-icon:hover { color: var(--color-text); }
.btn-icon--danger:hover { color: var(--color-error); }

.rule-meta {
  display: flex;
  gap: var(--space-xs);
  margin-top: var(--space-xs);
  flex-wrap: wrap;
}

.rule-badge {
  font-family: var(--font-mono);
  font-size: 9px;
  font-weight: 600;
  letter-spacing: 0.5px;
  padding: 1px 6px;
  background: var(--stone-200);
  border: 1px solid var(--stone-300);
  color: var(--color-text-muted);
}
</style>
