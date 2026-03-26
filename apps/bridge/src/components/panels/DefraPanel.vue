<template>
  <div class="panel defra-panel">
    <h2>DefraDB Integration</h2>
    <p class="panel-description">
      Connect to a DefraDB instance for P2P configuration sync and persistent state.
    </p>

    <div class="form-group">
      <label for="defra-enabled">Enable DefraDB</label>
      <input
        id="defra-enabled"
        type="checkbox"
        :checked="enabled"
        @change="onToggle"
      />
    </div>

    <div v-if="enabled" class="form-group">
      <label for="defra-url">DefraDB URL</label>
      <input
        id="defra-url"
        type="text"
        :value="url"
        placeholder="http://localhost:9181"
        @input="onUrlChange"
      />
      <div class="status-indicator">
        <span v-if="checking" class="status checking">Checking...</span>
        <span v-else-if="healthy" class="status healthy">Connected</span>
        <span v-else-if="url" class="status unhealthy">Not reachable</span>
      </div>
    </div>

    <div v-if="enabled && healthy" class="actions">
      <button @click="onExport" class="btn">Export Config to DefraDB</button>
      <button @click="onImport" class="btn">Import Config from DefraDB</button>
    </div>
  </div>
</template>

<script setup>
import { useDefra } from '../../composables/useDefra'
import { useNotifications } from '../../composables/useNotifications'

const { url, healthy, enabled, checking, setUrl, setEnabled, exportConfig, importConfig } = useDefra()
const { notify } = useNotifications()

function onToggle(e) {
  setEnabled(e.target.checked)
}

function onUrlChange(e) {
  setUrl(e.target.value)
}

async function onExport() {
  const result = await exportConfig()
  if (result?.success) {
    notify('Configuration exported to DefraDB', 'success')
  } else {
    notify('Failed to export to DefraDB', 'error')
  }
}

async function onImport() {
  const result = await importConfig({})
  if (result) {
    notify('Configuration imported from DefraDB', 'success')
  } else {
    notify('Failed to import from DefraDB', 'error')
  }
}
</script>
