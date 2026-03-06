<script setup lang="ts">
import { ref } from 'vue'
import { useConfigExport } from '../../composables/useConfigExport'

const dialogRef = ref<HTMLDialogElement | null>(null)
const activeTab = ref<'cli' | 'docker' | 'js' | 'rust'>('cli')
const copied = ref(false)

const { cliOutput, dockerOutput, clientJsExample, clientRustExample } = useConfigExport()

const tabs = [
  { id: 'cli' as const, label: 'CLI' },
  { id: 'docker' as const, label: 'DOCKER' },
  { id: 'js' as const, label: 'JS CLIENT' },
  { id: 'rust' as const, label: 'RUST CLIENT' },
]

function currentOutput(): string {
  switch (activeTab.value) {
    case 'cli': return cliOutput.value
    case 'docker': return dockerOutput.value
    case 'js': return clientJsExample.value
    case 'rust': return clientRustExample.value
  }
}

async function copyToClipboard() {
  try {
    await navigator.clipboard.writeText(currentOutput())
    copied.value = true
    setTimeout(() => { copied.value = false }, 1500)
  } catch { /* fallback: noop */ }
}

function open() {
  dialogRef.value?.showModal()
}

function close() {
  dialogRef.value?.close()
}

defineExpose({ open, close })
</script>

<template>
  <dialog ref="dialogRef" class="modal" @click.self="close">
    <div class="modal-content modal-content--wide">
      <div class="modal-header">
        <span class="modal-title">EXPORT CONFIG</span>
        <button class="modal-close" @click="close">&times;</button>
      </div>

      <div class="export-tabs">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          class="export-tab"
          :class="{ active: activeTab === tab.id }"
          @click="activeTab = tab.id"
        >
          {{ tab.label }}
        </button>
      </div>

      <div class="export-hint">Generated from your current router and connection configuration. <a href="https://docs.clasp.to/relay/cli" target="_blank" class="docs-link">CLI Reference</a></div>
      <div class="export-output">
        <pre class="code-block">{{ currentOutput() }}</pre>
      </div>

      <div class="modal-actions">
        <button class="btn btn-secondary" @click="copyToClipboard" title="Copy to clipboard">{{ copied ? 'COPIED' : 'COPY' }}</button>
        <button class="btn btn-primary" title="Close export dialog" @click="close">DONE</button>
      </div>
    </div>
  </dialog>
</template>

<style scoped>
.modal-content--wide {
  max-width: 640px;
}

.export-tabs {
  display: flex;
  gap: 0;
  border-bottom: var(--border-width) solid var(--color-border-dark);
  margin-bottom: var(--space-md);
}

.export-tab {
  font-family: var(--font-mono);
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 1px;
  padding: var(--space-xs) var(--space-md);
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  color: var(--color-text-muted);
  cursor: pointer;
}

.export-tab:hover {
  color: var(--color-text);
}

.export-tab.active {
  color: var(--color-accent);
  border-bottom-color: var(--color-accent);
}

.export-output {
  max-height: 50vh;
  overflow-y: auto;
  margin-bottom: var(--space-md);
}

.code-block {
  font-family: var(--font-mono);
  font-size: 11px;
  line-height: 1.5;
  background: var(--stone-200);
  border: 1px solid var(--stone-300);
  padding: var(--space-md);
  white-space: pre-wrap;
  word-break: break-all;
  margin: 0;
}

.export-hint {
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
</style>
