<script setup lang="ts">
import { ref } from 'vue'
import type { Rule } from '../../lib/types'
import { useRules } from '../../composables/useRules'
import { useRouters } from '../../composables/useRouters'
import RuleCard from '../forms/RuleCard.vue'
import RuleBuilder from '../forms/RuleBuilder.vue'
import EmptyState from '../shared/EmptyState.vue'

const { rules, add, update, remove, toggle, duplicate, exportRulesFile, createEmptyRule } = useRules()
const { availableRouters } = useRouters()
const builderRef = ref<InstanceType<typeof RuleBuilder> | null>(null)
const selectedRouterId = ref('')

function openBuilder(rule?: Rule) {
  builderRef.value?.open(rule)
}

function handleSave(rule: Rule) {
  const existing = rules.value.find(r => r.id === rule.id)
  if (existing) {
    update(rule.id, rule)
  } else {
    add(rule)
  }
}

async function applyToRouter() {
  const path = await exportRulesFile(selectedRouterId.value || undefined)
  if (path && selectedRouterId.value) {
    // The path is returned for the user to assign to a router's rulesPath
    // or it's already been set via the export flow
  }
}
</script>

<template>
  <div class="panel">
    <div class="panel-toolbar">
      <span class="panel-title">RULES ENGINE</span>
      <div class="toolbar-actions">
        <select v-model="selectedRouterId" class="select select--sm">
          <option value="">Select router...</option>
          <option v-for="r in availableRouters" :key="r.id" :value="r.id">{{ r.name }}</option>
        </select>
        <button class="btn btn-secondary btn--sm" :disabled="rules.length === 0" title="Export rules as JSON file" @click="applyToRouter">EXPORT</button>
        <button class="btn btn-primary btn--sm" title="Create a new automation rule" @click="openBuilder()">+ RULE</button>
      </div>
    </div>

    <div class="panel-body">
      <EmptyState v-if="rules.length === 0" message="No rules configured." hint="Rules automate signal behavior with triggers, conditions, and actions." />
      <div v-else class="rule-list">
        <RuleCard
          v-for="rule in rules"
          :key="rule.id"
          :rule="rule"
          @edit="openBuilder"
          @toggle="toggle"
          @duplicate="duplicate"
          @delete="remove"
        />
      </div>
    </div>

    <RuleBuilder ref="builderRef" @save="handleSave" />
  </div>
</template>

<style scoped>
.panel {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.panel-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-sm) var(--space-md);
  border-bottom: var(--border-width) solid var(--color-border-dark);
  background: var(--stone-200);
  flex-shrink: 0;
}

.panel-title {
  font-family: var(--font-mono);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 1px;
}

.toolbar-actions {
  display: flex;
  gap: var(--space-xs);
  align-items: center;
}

.select--sm {
  font-size: 10px;
  padding: 3px 6px;
}

.btn--sm {
  font-size: 10px;
  padding: 3px 8px;
}

.panel-body {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-md);
}

.rule-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
}
</style>
