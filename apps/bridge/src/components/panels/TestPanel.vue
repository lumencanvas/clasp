<script setup lang="ts">
import { ref, computed } from 'vue'
import { useConnections } from '../../composables/useConnections'
import { useRouters } from '../../composables/useRouters'
import { useDiagnostics } from '../../composables/useDiagnostics'
import { useElectron } from '../../composables/useElectron'
import { useNotifications } from '../../composables/useNotifications'
import { defaultAddresses } from '../../lib/constants'
import HealthCard from '../cards/HealthCard.vue'

const { connections } = useConnections()
const { routers } = useRouters()
const { diagnosticsResult, runDiagnostics: runDiag } = useDiagnostics()
const diagRunning = ref(false)
const { invoke } = useElectron()
const { notify } = useNotifications()

const testProtocol = ref('osc')
const testPath = ref('/test')
const testValue = ref('0.5')
const testTtl = ref('')
const testTtlAbsolute = ref(false)
const testResult = ref('')
const testRunning = ref(false)

const allServers = computed(() => [
  ...routers.value.map(r => ({ ...r, kind: 'router' as const })),
  ...connections.value.map(c => ({ ...c, kind: 'connection' as const })),
])

async function runDiagnostics() {
  diagRunning.value = true
  try {
    await runDiag()
  } finally {
    diagRunning.value = false
  }
}

async function sendTestSignal() {
  testRunning.value = true
  testResult.value = ''
  try {
    // Find a matching connection for the target address, or use default
    const proto = testProtocol.value
    const conn = connections.value.find(c => c.protocol === proto || c.type === proto)
    const address = conn?.address || defaultAddresses[proto as keyof typeof defaultAddresses] || ''

    const payload: Record<string, any> = {
      protocol: proto,
      address,
      signalAddress: testPath.value,
      value: parseFloat(testValue.value) || testValue.value,
    }
    if (testTtl.value !== '') {
      const ttlSeconds = parseFloat(testTtl.value)
      if (!isNaN(ttlSeconds) && ttlSeconds >= 0) {
        payload.ttl = ttlSeconds
        payload.absolute = testTtlAbsolute.value
      }
    }
    await invoke('sendTestSignal', payload)
    testResult.value = 'Signal sent'
    notify('Test signal sent', 'success')
  } catch (e: any) {
    testResult.value = `Error: ${e.message || e}`
    notify(`Test failed: ${e.message || e}`, 'error')
  } finally {
    testRunning.value = false
  }
}
</script>

<template>
  <div style="display: flex; flex-direction: column; height: 100%;">
    <div class="panel-toolbar">
      <span class="panel-title">TEST & DIAGNOSTICS</span>
      <div class="toolbar-group">
        <button class="btn btn-sm btn-secondary" :disabled="diagRunning" @click="runDiagnostics">
          {{ diagRunning ? 'RUNNING...' : 'RUN DIAGNOSTICS' }}
        </button>
      </div>
    </div>
    <div class="panel-content" style="overflow-y: auto;">
      <div class="test-panel-grid">
        <div class="test-section">
          <div class="test-section-title">SEND TEST SIGNAL</div>
          <div class="test-generator">
            <div class="form-group">
              <label class="form-label">Protocol</label>
              <select v-model="testProtocol" class="select select-sm">
                <option value="osc">OSC</option>
                <option value="midi">MIDI</option>
                <option value="artnet">Art-Net</option>
                <option value="mqtt">MQTT</option>
                <option value="websocket">WebSocket</option>
              </select>
            </div>
            <div class="form-group">
              <label class="form-label">Path</label>
              <input v-model="testPath" class="input input-sm" placeholder="/test" />
            </div>
            <div class="form-group">
              <label class="form-label">Value</label>
              <input v-model="testValue" class="input input-sm" placeholder="0.5" />
            </div>
            <div class="form-group">
              <label class="form-label">TTL (seconds)</label>
              <input v-model="testTtl" type="number" min="0" step="any" class="input input-sm" placeholder="Optional" />
            </div>
            <div v-if="testTtl" class="form-group">
              <label>
                <input v-model="testTtlAbsolute" type="checkbox" />
                Absolute expiry
              </label>
              <span class="form-hint">{{ testTtlAbsolute ? 'Expires at fixed time' : 'Sliding window (resets on update)' }}</span>
            </div>
            <button class="btn btn-primary btn-sm" :disabled="testRunning" @click="sendTestSignal">
              {{ testRunning ? 'SENDING...' : 'SEND' }}
            </button>
            <div v-if="testResult" class="test-result-area">{{ testResult }}</div>
          </div>
        </div>

        <div class="test-section">
          <div class="test-section-title">SERVER HEALTH</div>
          <div class="server-health-grid">
            <HealthCard v-for="server in allServers" :key="server.id" :server="server" />
            <div v-if="allServers.length === 0" class="empty-state-small">
              No active servers
            </div>
          </div>
        </div>

        <div v-if="diagnosticsResult" class="test-section test-section-full">
          <div class="test-section-title">DIAGNOSTICS RESULTS</div>
          <div class="diagnostics-output">
            <div v-for="(section, key) in diagnosticsResult" :key="key" class="diagnostics-section">
              <div class="diagnostics-section-title">{{ key }}</div>
              <div v-for="(value, label) in section" :key="label" class="diagnostics-row">
                <span class="diagnostics-label">{{ label }}</span>
                <span class="diagnostics-value">{{ value }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
