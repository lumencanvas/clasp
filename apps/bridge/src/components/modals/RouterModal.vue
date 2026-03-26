<script setup lang="ts">
import { ref, computed } from 'vue'
import type { Router } from '../../lib/types'
import { useRouters } from '../../composables/useRouters'
import { useNotifications } from '../../composables/useNotifications'
import CollapsibleSection from '../shared/CollapsibleSection.vue'
import ToggleSwitch from '../shared/ToggleSwitch.vue'

const { add, edit: editRouter } = useRouters()
const { notify } = useNotifications()

const dialogRef = ref<HTMLDialogElement | null>(null)
const isEdit = ref(false)
const editId = ref('')

// Core (always visible)
const name = ref('')
const port = ref(7330)
const host = ref('0.0.0.0')
const maxSessions = ref<number | undefined>()
const sessionTimeout = ref<number | undefined>()

// Auth
const authEnabled = ref(false)
const authPort = ref(7331)
const authDb = ref('')
const adminTokenPath = ref('')
const tokenTtl = ref<number | undefined>()
const corsOrigin = ref('')
const token = ref('')
const tokenFileContent = ref('')

// Transports
const quicEnabled = ref(false)
const quicPort = ref(7332)
const certPath = ref('')
const keyPath = ref('')
const mqttBridgeEnabled = ref(false)
const mqttBridgePort = ref(1883)
const mqttBridgeNamespace = ref('')
const oscBridgeEnabled = ref(false)
const oscBridgePort = ref(9000)
const oscBridgeNamespace = ref('')

// TTL
const paramTtl = ref<number | undefined>()
const signalTtl = ref<number | undefined>()
const noTtl = ref(false)

// Persistence
const persistEnabled = ref(false)
const persistPath = ref('')
const persistInterval = ref(5)
const journalEnabled = ref(false)
const journalPath = ref('')
const journalMemory = ref(false)

// Federation
const federationEnabled = ref(false)
const federationHub = ref('')
const federationId = ref('')
const federationToken = ref('')

// Operations
const healthEnabled = ref(false)
const healthPort = ref(7340)
const metricsEnabled = ref(false)
const metricsPort = ref(9090)
const drainTimeout = ref<number | undefined>()
const rendezvousPort = ref<number | undefined>()
const rendezvousTtl = ref<number | undefined>()

// Rules
const rulesPath = ref('')

// Port conflict detection
const usedPorts = computed(() => {
  const ports: { label: string; port: number }[] = [
    { label: 'WebSocket', port: port.value },
  ]
  if (authEnabled.value) ports.push({ label: 'Auth', port: authPort.value })
  if (quicEnabled.value) ports.push({ label: 'QUIC', port: quicPort.value })
  if (mqttBridgeEnabled.value) ports.push({ label: 'MQTT Bridge', port: mqttBridgePort.value })
  if (oscBridgeEnabled.value) ports.push({ label: 'OSC Bridge', port: oscBridgePort.value })
  if (healthEnabled.value) ports.push({ label: 'Health', port: healthPort.value })
  if (metricsEnabled.value) ports.push({ label: 'Metrics', port: metricsPort.value })
  if (rendezvousPort.value) ports.push({ label: 'Rendezvous', port: rendezvousPort.value })
  return ports
})

const portConflicts = computed(() => {
  const seen = new Map<number, string[]>()
  for (const p of usedPorts.value) {
    const list = seen.get(p.port) || []
    list.push(p.label)
    seen.set(p.port, list)
  }
  return Array.from(seen.entries())
    .filter(([, labels]) => labels.length > 1)
    .map(([p, labels]) => `Port ${p}: ${labels.join(', ')}`)
})

const authConfigured = computed(() => authEnabled.value)
const transportsConfigured = computed(() => quicEnabled.value || mqttBridgeEnabled.value || oscBridgeEnabled.value)
const ttlConfigured = computed(() => noTtl.value || paramTtl.value !== undefined || signalTtl.value !== undefined)
const persistConfigured = computed(() => persistEnabled.value || journalEnabled.value)
const federationConfigured = computed(() => federationEnabled.value)
const operationsConfigured = computed(() => healthEnabled.value || metricsEnabled.value || drainTimeout.value !== undefined || rendezvousPort.value !== undefined || rulesPath.value !== '')

function resetForm() {
  name.value = ''
  port.value = 7330
  host.value = '0.0.0.0'
  maxSessions.value = undefined
  sessionTimeout.value = undefined
  authEnabled.value = false
  authPort.value = 7331
  authDb.value = ''
  adminTokenPath.value = ''
  tokenTtl.value = undefined
  corsOrigin.value = ''
  token.value = ''
  tokenFileContent.value = ''
  quicEnabled.value = false
  quicPort.value = 7332
  certPath.value = ''
  keyPath.value = ''
  mqttBridgeEnabled.value = false
  mqttBridgePort.value = 1883
  mqttBridgeNamespace.value = ''
  oscBridgeEnabled.value = false
  oscBridgePort.value = 9000
  oscBridgeNamespace.value = ''
  paramTtl.value = undefined
  signalTtl.value = undefined
  noTtl.value = false
  persistEnabled.value = false
  persistPath.value = ''
  persistInterval.value = 5
  journalEnabled.value = false
  journalPath.value = ''
  journalMemory.value = false
  federationEnabled.value = false
  federationHub.value = ''
  federationId.value = ''
  federationToken.value = ''
  healthEnabled.value = false
  healthPort.value = 7340
  metricsEnabled.value = false
  metricsPort.value = 9090
  drainTimeout.value = undefined
  rendezvousPort.value = undefined
  rendezvousTtl.value = undefined
  rulesPath.value = ''
}

function loadFromRouter(router: Router) {
  name.value = router.name
  const addr = router.address || ''
  const addrParts = addr.replace(/^wss?:\/\//, '').split(':')
  port.value = parseInt(addrParts[1], 10) || 7330
  host.value = addrParts[0] || '0.0.0.0'
  maxSessions.value = router.maxSessions
  sessionTimeout.value = router.sessionTimeout
  authEnabled.value = router.authEnabled ?? false
  authPort.value = router.authPort ?? 7331
  authDb.value = router.authDb ?? ''
  adminTokenPath.value = router.adminTokenPath ?? ''
  tokenTtl.value = router.tokenTtl
  corsOrigin.value = router.corsOrigin ?? ''
  token.value = router.token ?? ''
  tokenFileContent.value = router.tokenFileContent ?? ''
  quicEnabled.value = router.quicEnabled ?? false
  quicPort.value = router.quicPort ?? 7332
  certPath.value = router.certPath ?? ''
  keyPath.value = router.keyPath ?? ''
  mqttBridgeEnabled.value = router.mqttBridgeEnabled ?? false
  mqttBridgePort.value = router.mqttBridgePort ?? 1883
  mqttBridgeNamespace.value = router.mqttBridgeNamespace ?? ''
  oscBridgeEnabled.value = router.oscBridgeEnabled ?? false
  oscBridgePort.value = router.oscBridgePort ?? 9000
  oscBridgeNamespace.value = router.oscBridgeNamespace ?? ''
  paramTtl.value = router.paramTtl
  signalTtl.value = router.signalTtl
  noTtl.value = router.noTtl ?? false
  persistEnabled.value = router.persistEnabled ?? false
  persistPath.value = router.persistPath ?? ''
  persistInterval.value = router.persistInterval ?? 5
  journalEnabled.value = router.journalEnabled ?? false
  journalPath.value = router.journalPath ?? ''
  journalMemory.value = router.journalMemory ?? false
  federationEnabled.value = router.federationEnabled ?? false
  federationHub.value = router.federationHub ?? ''
  federationId.value = router.federationId ?? ''
  federationToken.value = router.federationToken ?? ''
  healthEnabled.value = router.healthEnabled ?? false
  healthPort.value = router.healthPort ?? 7340
  metricsEnabled.value = router.metricsEnabled ?? false
  metricsPort.value = router.metricsPort ?? 9090
  drainTimeout.value = router.drainTimeout
  rendezvousPort.value = router.rendezvousPort
  rendezvousTtl.value = router.rendezvousTtl
  rulesPath.value = router.rulesPath ?? ''
}

function open(router?: Router) {
  if (router) {
    isEdit.value = true
    editId.value = router.id
    loadFromRouter(router)
  } else {
    isEdit.value = false
    editId.value = ''
    resetForm()
  }
  dialogRef.value?.showModal()
}

function close() {
  dialogRef.value?.close()
}

async function save() {
  if (portConflicts.value.length > 0) {
    notify('Fix port conflicts before saving', 'warning')
    return
  }

  try {
    const config: Partial<Router> = {
      name: name.value || 'CLASP Router',
      address: `${host.value}:${port.value}`,
      maxSessions: maxSessions.value,
      sessionTimeout: sessionTimeout.value,
      authEnabled: authEnabled.value,
      token: token.value || undefined,
      tokenFileContent: tokenFileContent.value || undefined,
      authPort: authEnabled.value ? authPort.value : undefined,
      authDb: authDb.value || undefined,
      adminTokenPath: adminTokenPath.value || undefined,
      tokenTtl: tokenTtl.value,
      corsOrigin: corsOrigin.value || undefined,
      quicEnabled: quicEnabled.value,
      quicPort: quicEnabled.value ? quicPort.value : undefined,
      certPath: certPath.value || undefined,
      keyPath: keyPath.value || undefined,
      mqttBridgeEnabled: mqttBridgeEnabled.value,
      mqttBridgePort: mqttBridgeEnabled.value ? mqttBridgePort.value : undefined,
      mqttBridgeNamespace: mqttBridgeNamespace.value || undefined,
      oscBridgeEnabled: oscBridgeEnabled.value,
      oscBridgePort: oscBridgeEnabled.value ? oscBridgePort.value : undefined,
      oscBridgeNamespace: oscBridgeNamespace.value || undefined,
      paramTtl: paramTtl.value,
      signalTtl: signalTtl.value,
      noTtl: noTtl.value,
      persistEnabled: persistEnabled.value,
      persistPath: persistPath.value || undefined,
      persistInterval: persistEnabled.value ? persistInterval.value : undefined,
      journalEnabled: journalEnabled.value,
      journalPath: journalPath.value || undefined,
      journalMemory: journalMemory.value,
      federationEnabled: federationEnabled.value,
      federationHub: federationHub.value || undefined,
      federationId: federationId.value || undefined,
      federationToken: federationToken.value || undefined,
      healthEnabled: healthEnabled.value,
      healthPort: healthEnabled.value ? healthPort.value : undefined,
      metricsEnabled: metricsEnabled.value,
      metricsPort: metricsEnabled.value ? metricsPort.value : undefined,
      drainTimeout: drainTimeout.value,
      rendezvousPort: rendezvousPort.value,
      rendezvousTtl: rendezvousTtl.value,
      rulesPath: rulesPath.value || undefined,
    }

    if (isEdit.value) {
      editRouter(editId.value)
      await add({ id: editId.value, ...config })
      notify('Router updated', 'success')
    } else {
      await add(config)
      notify('Router added', 'success')
    }
    close()
  } catch (e: any) {
    notify(`Failed: ${e.message || e}`, 'error')
  }
}

defineExpose({ open, close })
</script>

<template>
  <dialog ref="dialogRef" class="modal" @click.self="close">
    <div class="modal-content modal-content--wide">
      <div class="modal-header">
        <span class="modal-title">{{ isEdit ? 'EDIT ROUTER' : 'NEW ROUTER' }}</span>
        <button class="modal-close" @click="close">&times;</button>
      </div>
      <form class="router-form" @submit.prevent="save">
        <!-- Port conflict warnings -->
        <div v-if="portConflicts.length" class="port-warnings">
          <div v-for="conflict in portConflicts" :key="conflict" class="port-warning">
            {{ conflict }}
          </div>
        </div>

        <!-- Core (always visible) -->
        <div class="form-group">
          <label class="form-label">Name</label>
          <input v-model="name" class="input" placeholder="CLASP Router" />
        </div>
        <div class="form-row">
          <div class="form-group">
            <label class="form-label">Host</label>
            <input v-model="host" class="input" placeholder="0.0.0.0" />
          </div>
          <div class="form-group">
            <label class="form-label">Port</label>
            <input v-model.number="port" class="input" type="number" />
          </div>
        </div>
        <div class="form-row">
          <div class="form-group">
            <label class="form-label">Max Sessions</label>
            <input v-model.number="maxSessions" class="input" type="number" placeholder="unlimited" />
          </div>
          <div class="form-group">
            <label class="form-label">Session Timeout (s)</label>
            <input v-model.number="sessionTimeout" class="input" type="number" placeholder="300" />
          </div>
        </div>

        <!-- Auth (CPSK) -->
        <CollapsibleSection :title="authConfigured ? 'AUTHENTICATION \u2022' : 'AUTHENTICATION'">
          <div class="form-group">
            <ToggleSwitch v-model="authEnabled" label="Enable authentication" />
          </div>
          <div class="section-hint">Pre-shared key authentication. <a href="https://docs.clasp.to/relay/auth" target="_blank" class="docs-link">Docs</a></div>
          <template v-if="authEnabled">
            <div class="form-group">
              <label class="form-label">Token</label>
              <input v-model="token" class="input" type="password" placeholder="Pre-shared key" />
            </div>
            <div class="form-group">
              <label class="form-label">Token File (one per line)</label>
              <textarea v-model="tokenFileContent" class="input" rows="3" placeholder="One token per line" />
            </div>
            <div class="form-row">
              <div class="form-group">
                <label class="form-label">Auth Port</label>
                <input v-model.number="authPort" class="input" type="number" />
              </div>
              <div class="form-group">
                <label class="form-label">Token TTL (s)</label>
                <input v-model.number="tokenTtl" class="input" type="number" placeholder="3600" />
              </div>
            </div>
            <div class="form-group">
              <label class="form-label">Auth DB Path</label>
              <input v-model="authDb" class="input" placeholder="./auth.db" />
            </div>
            <div class="form-group">
              <label class="form-label">Admin Token Path</label>
              <input v-model="adminTokenPath" class="input" placeholder="/path/to/admin.token" />
            </div>
            <div class="form-group">
              <label class="form-label">CORS Origin</label>
              <input v-model="corsOrigin" class="input" placeholder="*" />
            </div>
          </template>
        </CollapsibleSection>

        <!-- Transports -->
        <CollapsibleSection :title="transportsConfigured ? 'TRANSPORTS \u2022' : 'TRANSPORTS'">
          <div class="section-hint">Additional transport protocols alongside WebSocket.</div>
          <div class="transport-block">
            <div class="form-group">
              <ToggleSwitch v-model="quicEnabled" label="QUIC transport" />
            </div>
            <template v-if="quicEnabled">
              <div class="form-row">
                <div class="form-group">
                  <label class="form-label">QUIC Port</label>
                  <input v-model.number="quicPort" class="input" type="number" />
                </div>
              </div>
              <div class="form-group">
                <label class="form-label">TLS Certificate</label>
                <input v-model="certPath" class="input" placeholder="/path/to/cert.pem" />
              </div>
              <div class="form-group">
                <label class="form-label">TLS Key</label>
                <input v-model="keyPath" class="input" placeholder="/path/to/key.pem" />
              </div>
            </template>
          </div>

          <div class="transport-block">
            <div class="form-group">
              <ToggleSwitch v-model="mqttBridgeEnabled" label="MQTT bridge" />
            </div>
            <template v-if="mqttBridgeEnabled">
              <div class="form-row">
                <div class="form-group">
                  <label class="form-label">MQTT Port</label>
                  <input v-model.number="mqttBridgePort" class="input" type="number" />
                </div>
                <div class="form-group">
                  <label class="form-label">Namespace</label>
                  <input v-model="mqttBridgeNamespace" class="input" placeholder="clasp" />
                </div>
              </div>
            </template>
          </div>

          <div class="transport-block">
            <div class="form-group">
              <ToggleSwitch v-model="oscBridgeEnabled" label="OSC bridge" />
            </div>
            <template v-if="oscBridgeEnabled">
              <div class="form-row">
                <div class="form-group">
                  <label class="form-label">OSC Port</label>
                  <input v-model.number="oscBridgePort" class="input" type="number" />
                </div>
                <div class="form-group">
                  <label class="form-label">Namespace</label>
                  <input v-model="oscBridgeNamespace" class="input" placeholder="clasp" />
                </div>
              </div>
            </template>
          </div>
        </CollapsibleSection>

        <!-- TTL -->
        <CollapsibleSection :title="ttlConfigured ? 'TTL \u2022' : 'TTL'">
          <div class="section-hint">Time-to-live for cached parameter and signal values. <a href="https://docs.clasp.to/relay/config" target="_blank" class="docs-link">Docs</a></div>
          <div class="form-group">
            <ToggleSwitch v-model="noTtl" label="Disable all TTL" />
          </div>
          <template v-if="!noTtl">
            <div class="form-row">
              <div class="form-group">
                <label class="form-label">Param TTL (s)</label>
                <input v-model.number="paramTtl" class="input" type="number" placeholder="60" />
              </div>
              <div class="form-group">
                <label class="form-label">Signal TTL (s)</label>
                <input v-model.number="signalTtl" class="input" type="number" placeholder="30" />
              </div>
            </div>
          </template>
        </CollapsibleSection>

        <!-- Persistence -->
        <CollapsibleSection :title="persistConfigured ? 'PERSISTENCE \u2022' : 'PERSISTENCE'">
          <div class="section-hint">Persist state to disk across restarts. <a href="https://docs.clasp.to/relay/persistence" target="_blank" class="docs-link">Docs</a></div>
          <div class="form-group">
            <ToggleSwitch v-model="persistEnabled" label="Enable persistence" />
          </div>
          <template v-if="persistEnabled">
            <div class="form-group">
              <label class="form-label">Persist Path</label>
              <input v-model="persistPath" class="input" placeholder="./clasp-data" />
            </div>
            <div class="form-group">
              <label class="form-label">Persist Interval (s)</label>
              <input v-model.number="persistInterval" class="input" type="number" />
            </div>
          </template>
          <div class="form-group" style="margin-top: var(--space-sm)">
            <ToggleSwitch v-model="journalEnabled" label="Enable journal" />
          </div>
          <template v-if="journalEnabled">
            <div class="form-group">
              <label class="form-label">Journal Path</label>
              <input v-model="journalPath" class="input" placeholder="./clasp-journal" />
            </div>
            <div class="form-group">
              <ToggleSwitch v-model="journalMemory" label="Journal in-memory only" />
            </div>
          </template>
        </CollapsibleSection>

        <!-- Federation -->
        <CollapsibleSection :title="federationConfigured ? 'FEDERATION \u2022' : 'FEDERATION'">
          <div class="section-hint">Connect multiple CLASP routers into a mesh. <a href="https://docs.clasp.to/relay/federation" target="_blank" class="docs-link">Docs</a></div>
          <div class="form-group">
            <ToggleSwitch v-model="federationEnabled" label="Enable federation" />
          </div>
          <template v-if="federationEnabled">
            <div class="form-group">
              <label class="form-label">Hub Address</label>
              <input v-model="federationHub" class="input" placeholder="hub.example.com:7330" />
            </div>
            <div class="form-row">
              <div class="form-group">
                <label class="form-label">Node ID</label>
                <input v-model="federationId" class="input" placeholder="node-1" />
              </div>
              <div class="form-group">
                <label class="form-label">Token</label>
                <input v-model="federationToken" class="input" type="password" placeholder="Federation token" />
              </div>
            </div>
          </template>
        </CollapsibleSection>

        <!-- Operations -->
        <CollapsibleSection :title="operationsConfigured ? 'OPERATIONS \u2022' : 'OPERATIONS'">
          <div class="section-hint">Health checks, metrics, and operational settings.</div>
          <div class="form-row">
            <div class="form-group">
              <ToggleSwitch v-model="healthEnabled" label="Health endpoint" />
            </div>
            <div v-if="healthEnabled" class="form-group">
              <label class="form-label">Health Port</label>
              <input v-model.number="healthPort" class="input" type="number" />
            </div>
          </div>
          <div class="form-row">
            <div class="form-group">
              <ToggleSwitch v-model="metricsEnabled" label="Metrics endpoint" />
            </div>
            <div v-if="metricsEnabled" class="form-group">
              <label class="form-label">Metrics Port</label>
              <input v-model.number="metricsPort" class="input" type="number" />
            </div>
          </div>
          <div class="form-group">
            <label class="form-label">Drain Timeout (s)</label>
            <input v-model.number="drainTimeout" class="input" type="number" placeholder="30" />
          </div>
          <div class="form-row">
            <div class="form-group">
              <label class="form-label">Rendezvous Port</label>
              <input v-model.number="rendezvousPort" class="input" type="number" placeholder="disabled" />
            </div>
            <div class="form-group">
              <label class="form-label">Rendezvous TTL (s)</label>
              <input v-model.number="rendezvousTtl" class="input" type="number" placeholder="300" />
            </div>
          </div>
          <div class="form-group">
            <label class="form-label">Rules File Path</label>
            <input v-model="rulesPath" class="input" placeholder="rules.json" />
          </div>
        </CollapsibleSection>

        <div class="modal-actions">
          <button type="button" class="btn btn-secondary" @click="close">CANCEL</button>
          <button type="submit" class="btn btn-primary">{{ isEdit ? 'SAVE' : 'CREATE' }}</button>
        </div>
      </form>
    </div>
  </dialog>
</template>

<style scoped>
.modal-content--wide {
  max-width: 560px;
}

.router-form {
  max-height: 70vh;
  overflow-y: auto;
  padding-right: var(--space-xs);
}

.port-warnings {
  margin-bottom: var(--space-md);
}

.port-warning {
  padding: var(--space-xs) var(--space-sm);
  background: var(--color-warning);
  color: var(--stone-900);
  font-family: var(--font-mono);
  font-size: 11px;
  font-weight: 600;
  border: var(--border-width) solid var(--stone-900);
  margin-bottom: var(--space-xs);
}

.transport-block {
  padding-bottom: var(--space-sm);
}

.transport-block + .transport-block {
  border-top: 1px dashed var(--stone-300);
  padding-top: var(--space-sm);
}

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
</style>
