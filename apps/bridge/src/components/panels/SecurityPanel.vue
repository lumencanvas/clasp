<script setup lang="ts">
import { ref } from 'vue'
import type { VisibilityMode, WriteCheck } from '../../lib/types'
import { useAppConfig } from '../../composables/useAppConfig'
import CollapsibleSection from '../shared/CollapsibleSection.vue'

const {
  appConfig,
  scopeTemplates,
  addScope,
  removeScope,
  updateScope,
  applyScopeTemplate,
  addWriteRule,
  removeWriteRule,
  addCheckToWriteRule,
  removeCheckFromWriteRule,
  addVisibilityRule,
  removeVisibilityRule,
  updateVisibilityRule,
  addSnapshotTransform,
  removeSnapshotTransform,
  updateRateLimits,
  exportAppConfig,
} = useAppConfig()

// New scope form
const newScopeName = ref('')
const newScopeAction = ref('read')
const newScopePattern = ref('')

function submitScope() {
  if (!newScopePattern.value) return
  addScope({
    name: newScopeName.value || newScopeAction.value,
    action: newScopeAction.value,
    pattern: newScopePattern.value,
  })
  newScopeName.value = ''
  newScopeAction.value = 'read'
  newScopePattern.value = ''
}

// New write rule form
const newWrPath = ref('')

function submitWriteRule() {
  if (!newWrPath.value) return
  addWriteRule({ path: newWrPath.value, preChecks: [], checks: [] })
  newWrPath.value = ''
}

// New visibility rule form
const newVisType = ref<'path' | 'contains' | 'catchall'>('path')
const newVisPattern = ref('')
const newVisVisible = ref<'true' | 'false' | 'owner' | 'require_state_not_null'>('true')

function parseVisibility(val: string): VisibilityMode {
  if (val === 'true') return true
  if (val === 'false') return false
  return val as 'owner' | 'require_state_not_null'
}

function submitVisibility() {
  if (!newVisPattern.value && newVisType.value !== 'catchall') return
  addVisibilityRule({
    type: newVisType.value,
    pattern: newVisType.value === 'catchall' ? '*' : newVisPattern.value,
    visible: parseVisibility(newVisVisible.value),
  })
  newVisPattern.value = ''
}

// New snapshot transform
const newStPath = ref('')
const newStFields = ref('')

function submitSnapshotTransform() {
  if (!newStPath.value) return
  addSnapshotTransform({
    path: newStPath.value,
    redactFields: newStFields.value.split(',').map(f => f.trim()).filter(Boolean),
  })
  newStPath.value = ''
  newStFields.value = ''
}

const checkTypes = [
  { value: 'require_auth', label: 'Require Auth' },
  { value: 'require_scope', label: 'Require Scope' },
  { value: 'require_state_not_null', label: 'Require State Not Null' },
  { value: 'rate_limit', label: 'Rate Limit' },
  { value: 'max_size', label: 'Max Size' },
  { value: 'schema_validate', label: 'Schema Validate' },
  { value: 'owner_only', label: 'Owner Only' },
]

function addCheckToRule(ruleId: string, phase: 'preChecks' | 'checks', type: string) {
  addCheckToWriteRule(ruleId, phase, { type: type as WriteCheck['type'] })
}

function removeCheckFromRule(ruleId: string, phase: 'preChecks' | 'checks', idx: number) {
  removeCheckFromWriteRule(ruleId, phase, idx)
}
</script>

<template>
  <div class="panel">
    <div class="panel-toolbar">
      <span class="panel-title">APP CONFIG / SECURITY</span>
      <button class="btn btn-primary btn--sm" title="Export security config as app-config.json" @click="exportAppConfig">EXPORT JSON</button>
    </div>

    <div class="panel-body">
      <!-- Scopes -->
      <CollapsibleSection title="SCOPES" :default-open="true">
        <div class="section-hint">Define access control patterns for read/write operations. <a href="https://docs.clasp.to/relay/auth#scopes" target="_blank" class="docs-link">Docs</a></div>
        <div class="template-row">
          <button
            v-for="tmpl in scopeTemplates"
            :key="tmpl.name"
            class="btn btn-secondary btn--xs"
            @click="applyScopeTemplate(tmpl)"
          >
            + {{ tmpl.name }}
          </button>
        </div>

        <div v-for="scope in appConfig.scopes" :key="scope.id" class="list-item">
          <div class="list-item-body">
            <input
              :value="scope.name"
              class="input input--inline"
              @change="updateScope(scope.id, { name: ($event.target as HTMLInputElement).value })"
            />
            <select
              :value="scope.action"
              class="select select--sm"
              @change="updateScope(scope.id, { action: ($event.target as HTMLSelectElement).value })"
            >
              <option value="read">read</option>
              <option value="write">write</option>
              <option value="*">*</option>
            </select>
            <input
              :value="scope.pattern"
              class="input input--inline"
              placeholder="/path/**"
              @change="updateScope(scope.id, { pattern: ($event.target as HTMLInputElement).value })"
            />
          </div>
          <button class="btn-icon btn-icon--danger" @click="removeScope(scope.id)">&times;</button>
        </div>

        <div class="add-row">
          <input v-model="newScopeName" class="input input--sm" placeholder="Name" />
          <select v-model="newScopeAction" class="select select--sm">
            <option value="read">read</option>
            <option value="write">write</option>
            <option value="*">*</option>
          </select>
          <input v-model="newScopePattern" class="input input--sm" placeholder="/path/**" />
          <button class="btn btn-secondary btn--sm" @click="submitScope">ADD</button>
        </div>
      </CollapsibleSection>

      <!-- Write Rules -->
      <CollapsibleSection title="WRITE RULES">
        <div class="section-hint">Validation checks applied before and after write operations.</div>
        <div v-for="rule in appConfig.writeRules" :key="rule.id" class="write-rule-block">
          <div class="write-rule-header">
            <code class="path-display">{{ rule.path }}</code>
            <button class="btn-icon btn-icon--danger" @click="removeWriteRule(rule.id)">&times;</button>
          </div>
          <div class="checks-section">
            <span class="checks-label">Pre-checks:</span>
            <span v-for="(check, i) in rule.preChecks" :key="i" class="check-badge">
              {{ check.type }}
              <button class="badge-x" @click="removeCheckFromRule(rule.id, 'preChecks', i)">&times;</button>
            </span>
            <select class="select select--xs" @change="addCheckToRule(rule.id, 'preChecks', ($event.target as HTMLSelectElement).value); ($event.target as HTMLSelectElement).value = ''">
              <option value="">+ add</option>
              <option v-for="ct in checkTypes" :key="ct.value" :value="ct.value">{{ ct.label }}</option>
            </select>
          </div>
          <div class="checks-section">
            <span class="checks-label">Checks:</span>
            <span v-for="(check, i) in rule.checks" :key="i" class="check-badge">
              {{ check.type }}
              <button class="badge-x" @click="removeCheckFromRule(rule.id, 'checks', i)">&times;</button>
            </span>
            <select class="select select--xs" @change="addCheckToRule(rule.id, 'checks', ($event.target as HTMLSelectElement).value); ($event.target as HTMLSelectElement).value = ''">
              <option value="">+ add</option>
              <option v-for="ct in checkTypes" :key="ct.value" :value="ct.value">{{ ct.label }}</option>
            </select>
          </div>
        </div>
        <div class="add-row">
          <input v-model="newWrPath" class="input input--sm" placeholder="/path/pattern" />
          <button class="btn btn-secondary btn--sm" @click="submitWriteRule">ADD RULE</button>
        </div>
      </CollapsibleSection>

      <!-- Visibility -->
      <CollapsibleSection title="SNAPSHOT VISIBILITY">
        <div class="section-hint">Control which address tree nodes are visible to clients.</div>
        <div v-for="vis in appConfig.visibility" :key="vis.id" class="list-item">
          <div class="list-item-body">
            <select
              :value="vis.type"
              class="select select--sm"
              @change="updateVisibilityRule(vis.id, { type: ($event.target as HTMLSelectElement).value as 'path' | 'contains' | 'catchall' })"
            >
              <option value="path">Path</option>
              <option value="contains">Contains</option>
              <option value="catchall">Catchall</option>
            </select>
            <input
              v-if="vis.type !== 'catchall'"
              :value="vis.pattern"
              class="input input--inline"
              @change="updateVisibilityRule(vis.id, { pattern: ($event.target as HTMLInputElement).value })"
            />
            <select
              :value="String(vis.visible)"
              class="select select--sm"
              @change="updateVisibilityRule(vis.id, { visible: parseVisibility(($event.target as HTMLSelectElement).value) })"
            >
              <option value="true">Visible</option>
              <option value="false">Hidden</option>
              <option value="owner">Owner Only</option>
              <option value="require_state_not_null">If State Not Null</option>
            </select>
          </div>
          <button class="btn-icon btn-icon--danger" @click="removeVisibilityRule(vis.id)">&times;</button>
        </div>
        <div class="add-row">
          <select v-model="newVisType" class="select select--sm">
            <option value="path">Path</option>
            <option value="contains">Contains</option>
            <option value="catchall">Catchall</option>
          </select>
          <input v-if="newVisType !== 'catchall'" v-model="newVisPattern" class="input input--sm" placeholder="pattern" />
          <select v-model="newVisVisible" class="select select--sm">
            <option value="true">Visible</option>
            <option value="false">Hidden</option>
            <option value="owner">Owner Only</option>
            <option value="require_state_not_null">If State Not Null</option>
          </select>
          <button class="btn btn-secondary btn--sm" @click="submitVisibility">ADD</button>
        </div>
      </CollapsibleSection>

      <!-- Snapshot Transforms -->
      <CollapsibleSection title="SNAPSHOT TRANSFORMS">
        <div class="section-hint">Redact fields from snapshot responses for specific paths.</div>
        <div v-for="st in appConfig.snapshotTransforms" :key="st.id" class="list-item">
          <div class="list-item-body">
            <code class="path-display">{{ st.path }}</code>
            <span class="field-tags">{{ st.redactFields.join(', ') }}</span>
          </div>
          <button class="btn-icon btn-icon--danger" @click="removeSnapshotTransform(st.id)">&times;</button>
        </div>
        <div class="add-row">
          <input v-model="newStPath" class="input input--sm" placeholder="/path" />
          <input v-model="newStFields" class="input input--sm" placeholder="field1, field2" />
          <button class="btn btn-secondary btn--sm" @click="submitSnapshotTransform">ADD</button>
        </div>
      </CollapsibleSection>

      <!-- Rate Limits -->
      <CollapsibleSection title="RATE LIMITS">
        <div class="section-hint">Throttle authentication and registration attempts.</div>
        <div class="form-row">
          <div class="form-group">
            <label class="form-label">Login Max Attempts</label>
            <input
              :value="appConfig.rateLimits.loginMaxAttempts"
              class="input"
              type="number"
              @input="updateRateLimits({ loginMaxAttempts: Number(($event.target as HTMLInputElement).value) })"
            />
          </div>
          <div class="form-group">
            <label class="form-label">Login Window (s)</label>
            <input
              :value="appConfig.rateLimits.loginWindow"
              class="input"
              type="number"
              @input="updateRateLimits({ loginWindow: Number(($event.target as HTMLInputElement).value) })"
            />
          </div>
        </div>
        <div class="form-row">
          <div class="form-group">
            <label class="form-label">Register Max Attempts</label>
            <input
              :value="appConfig.rateLimits.registerMaxAttempts"
              class="input"
              type="number"
              @input="updateRateLimits({ registerMaxAttempts: Number(($event.target as HTMLInputElement).value) })"
            />
          </div>
          <div class="form-group">
            <label class="form-label">Register Window (s)</label>
            <input
              :value="appConfig.rateLimits.registerWindow"
              class="input"
              type="number"
              @input="updateRateLimits({ registerWindow: Number(($event.target as HTMLInputElement).value) })"
            />
          </div>
        </div>
      </CollapsibleSection>
    </div>
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

.btn--sm {
  font-size: 10px;
  padding: 3px 8px;
}

.btn--xs {
  font-size: 9px;
  padding: 2px 6px;
}

.panel-body {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-md);
}

.template-row {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-xs);
  margin-bottom: var(--space-sm);
}

.list-item {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  padding: var(--space-xs) 0;
  border-bottom: 1px dashed var(--stone-200);
}

.list-item-body {
  flex: 1;
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  min-width: 0;
  flex-wrap: wrap;
}

.input--inline {
  font-size: 11px;
  padding: 2px 6px;
  flex: 1;
  min-width: 80px;
}

.input--sm {
  font-size: 11px;
  padding: 4px 6px;
  min-width: 0;
}

.select--sm {
  font-size: 10px;
  padding: 3px 6px;
  min-width: 60px;
}

.select--xs {
  font-size: 9px;
  padding: 2px 4px;
  min-width: 50px;
}

.add-row {
  display: flex;
  gap: var(--space-xs);
  align-items: center;
  margin-top: var(--space-sm);
}

.btn-icon {
  background: none;
  border: none;
  padding: 2px 4px;
  cursor: pointer;
  color: var(--color-text-muted);
  font-size: 16px;
  line-height: 1;
  flex-shrink: 0;
}

.btn-icon:hover { color: var(--color-text); }
.btn-icon--danger:hover { color: var(--color-error); }

.write-rule-block {
  background: var(--color-surface);
  border: 1px solid var(--stone-300);
  padding: var(--space-sm);
  margin-bottom: var(--space-sm);
}

.write-rule-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--space-xs);
}

.path-display {
  font-family: var(--font-mono);
  font-size: 11px;
  font-weight: 600;
  color: var(--color-accent);
}

.checks-section {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  flex-wrap: wrap;
  margin-top: var(--space-xs);
}

.checks-label {
  font-family: var(--font-mono);
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 0.5px;
  color: var(--color-text-muted);
  min-width: 65px;
}

.check-badge {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  font-family: var(--font-mono);
  font-size: 9px;
  padding: 1px 6px;
  background: var(--stone-200);
  border: 1px solid var(--stone-300);
}

.badge-x {
  background: none;
  border: none;
  padding: 0;
  cursor: pointer;
  color: var(--color-text-muted);
  font-size: 12px;
  line-height: 1;
}

.badge-x:hover { color: var(--color-error); }

.section-hint {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--color-text-muted);
  margin-bottom: var(--space-sm);
  line-height: 1.4;
}

.docs-link {
  color: var(--color-accent);
  text-decoration: none;
  font-weight: 600;
}

.docs-link:hover {
  text-decoration: underline;
}

.field-tags {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--color-text-muted);
}
</style>
