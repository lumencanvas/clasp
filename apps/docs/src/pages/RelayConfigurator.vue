<script setup>
import { reactive, computed, ref, watch, onMounted } from 'vue'
import ConfigSection from '../components/configurator/ConfigSection.vue'
import CopyBlock from '../components/configurator/CopyBlock.vue'
import FieldRow from '../components/configurator/FieldRow.vue'
import ToggleSwitch from '../components/configurator/ToggleSwitch.vue'
import PresetSelector from '../components/configurator/PresetSelector.vue'
import ScopeBuilder from '../components/configurator/ScopeBuilder.vue'
import WriteRuleBuilder from '../components/configurator/WriteRuleBuilder.vue'
import VisibilityBuilder from '../components/configurator/VisibilityBuilder.vue'
import TransformBuilder from '../components/configurator/TransformBuilder.vue'
import RuleBuilder from '../components/configurator/RuleBuilder.vue'

// ---------- Defaults ----------
const DEFAULTS = {
  wsPort: 7330, host: '0.0.0.0', name: 'CLASP Relay',
  noWebsocket: false, maxSessions: 1000, sessionTimeout: 300,
  authEnabled: false, authPort: 7350, authDb: 'relay-auth.db',
  adminTokenPath: '', tokenTtl: 86400, corsOrigin: '',
  quicEnabled: false, quicPort: 7331, certPath: '', keyPath: '',
  mqttEnabled: false, mqttPort: 1883, mqttNamespace: '/mqtt',
  oscEnabled: false, oscPort: 9000, oscNamespace: '/osc',
  paramTtl: 3600, signalTtl: 3600, noTtl: false,
  persistEnabled: false, persistPath: '', persistInterval: 30,
  journalEnabled: false, journalPath: '', journalMemory: false,
  appConfigEnabled: false,
  rulesEnabled: false, rulesPath: 'rules.json',
  federationEnabled: false, federationHub: '', federationId: '',
  federationNamespaces: [], federationToken: '',
  healthEnabled: false, healthPort: 8080,
  metricsEnabled: false, metricsPort: 9090,
  drainTimeout: 30, rendezvousPort: 7340, rendezvousTtl: 300,
  trustAnchors: [], capMaxDepth: 5,
  registryEnabled: false, registryDbPath: '',

  // Standalone Bridges
  artnetEnabled: false, artnetNamespace: '/artnet', artnetInterface: '0.0.0.0',
  artnetUniverses: '0-15', artnetNormalize: false, artnetMode: 'channel',

  dmxEnabled: false, dmxSerialPort: '', dmxNamespace: '/dmx', dmxBaud: 250000,
  dmxDirection: 'both', dmxChannels: '1-512', dmxRefresh: 40,

  sacnEnabled: false, sacnNamespace: '/sacn', sacnInterface: '0.0.0.0',
  sacnUniverses: '1-10', sacnPriority: 100, sacnMode: 'channel', sacnSourceName: 'clasp-sacn',

  midiEnabled: false, midiDevice: '', midiNamespace: '/midi',

  httpBridgeEnabled: false, httpBridgePort: 8080, httpBridgeBind: '0.0.0.0',
  httpBridgeCors: '*', httpBridgeToken: '',

  wsBridgeEnabled: false, wsBridgePort: 9000, wsBridgeBind: '0.0.0.0',
  wsBridgeToken: ''
}

// ---------- State ----------
const config = reactive({
  ...DEFAULTS,
  appConfig: {
    scopes: [],
    writeRules: [],
    snapshotTransforms: [],
    snapshotVisibility: [],
    rateLimits: { loginMaxAttempts: 5, loginWindowSecs: 60, registerMaxAttempts: 10, registerWindowSecs: 60 }
  },
  rules: []
})

const activeTab = ref('cli')

// ---------- URL Hash Sharing ----------
let hashUpdateTimer = null

function serializeToHash() {
  try {
    const json = JSON.stringify(config)
    window.location.hash = btoa(encodeURIComponent(json))
  } catch { /* ignore */ }
}

function loadFromHash() {
  const hash = window.location.hash.slice(1)
  if (!hash) return
  try {
    const json = decodeURIComponent(atob(hash))
    const parsed = JSON.parse(json)
    Object.assign(config, parsed)
    if (parsed.appConfig) Object.assign(config.appConfig, parsed.appConfig)
    if (parsed.rules) config.rules = parsed.rules
  } catch { /* ignore invalid hash */ }
}

onMounted(() => {
  loadFromHash()
})

watch(config, () => {
  clearTimeout(hashUpdateTimer)
  hashUpdateTimer = setTimeout(serializeToHash, 500)
}, { deep: true })

function copyLink() {
  serializeToHash()
  navigator.clipboard.writeText(window.location.href)
  linkCopied.value = true
  setTimeout(() => { linkCopied.value = false }, 1500)
}

const linkCopied = ref(false)

// ---------- Preset Apply ----------
function applyPreset(presetConfig) {
  // Reset to defaults first
  Object.assign(config, { ...DEFAULTS })
  config.appConfig = {
    scopes: [],
    writeRules: [],
    snapshotTransforms: [],
    snapshotVisibility: [],
    rateLimits: { loginMaxAttempts: 5, loginWindowSecs: 60, registerMaxAttempts: 10, registerWindowSecs: 60 }
  }
  config.rules = []
  // Apply preset
  const { appConfig, rules, ...rest } = presetConfig
  Object.assign(config, rest)
  if (appConfig) Object.assign(config.appConfig, appConfig)
  if (rules) config.rules = rules
}

// ---------- Output: CLI Command ----------
const cliOutput = computed(() => {
  const lines = ['clasp-relay']
  const f = (flag, val, def) => {
    if (val !== def && val !== '' && val !== false) lines.push(`  ${flag} ${val}`)
  }
  const b = (flag, val) => { if (val) lines.push(`  ${flag}`) }

  f('--ws-port', config.wsPort, 7330)
  f('--host', config.host, '0.0.0.0')
  f('--name', `"${config.name}"`, `"CLASP Relay"`)
  b('--no-websocket', config.noWebsocket)
  f('--max-sessions', config.maxSessions, 1000)
  f('--session-timeout', config.sessionTimeout, 300)

  if (config.authEnabled) {
    lines.push('  --auth')
    f('--auth-port', config.authPort, 7350)
    f('--auth-db', config.authDb, 'relay-auth.db')
    if (config.adminTokenPath) f('--admin-token-path', config.adminTokenPath, '')
    f('--token-ttl', config.tokenTtl, 86400)
    if (config.corsOrigin) f('--cors-origin', config.corsOrigin, '')
  }

  if (config.quicEnabled) {
    lines.push('  --quic')
    f('--quic-port', config.quicPort, 7331)
    if (config.certPath) f('--cert', config.certPath, '')
    if (config.keyPath) f('--key', config.keyPath, '')
  }

  if (config.mqttEnabled) {
    lines.push('  --mqtt')
    f('--mqtt-port', config.mqttPort, 1883)
    f('--mqtt-namespace', config.mqttNamespace, '/mqtt')
  }

  if (config.oscEnabled) {
    lines.push('  --osc')
    f('--osc-port', config.oscPort, 9000)
    f('--osc-namespace', config.oscNamespace, '/osc')
  }

  f('--param-ttl', config.paramTtl, 3600)
  f('--signal-ttl', config.signalTtl, 3600)
  b('--no-ttl', config.noTtl)

  if (config.persistEnabled) {
    lines.push('  --persist')
    if (config.persistPath) f('--persist-path', config.persistPath, '')
    f('--persist-interval', config.persistInterval, 30)
  }

  if (config.journalEnabled) {
    lines.push('  --journal')
    if (config.journalPath) f('--journal-path', config.journalPath, '')
    b('--journal-memory', config.journalMemory)
  }

  if (config.appConfigEnabled) {
    lines.push('  --app-config app-config.json')
  }

  if (config.rulesEnabled) {
    f('--rules', config.rulesPath, '')
  }

  if (config.federationEnabled) {
    lines.push('  --federation')
    if (config.federationHub) f('--federation-hub', config.federationHub, '')
    if (config.federationId) f('--federation-id', config.federationId, '')
    if (config.federationToken) f('--federation-token', config.federationToken, '')
  }

  if (config.healthEnabled) {
    lines.push('  --health')
    f('--health-port', config.healthPort, 8080)
  }

  if (config.metricsEnabled) {
    lines.push('  --metrics')
    f('--metrics-port', config.metricsPort, 9090)
  }

  f('--drain-timeout', config.drainTimeout, 30)
  f('--rendezvous-port', config.rendezvousPort, 7340)
  f('--rendezvous-ttl', config.rendezvousTtl, 300)

  if (lines.length === 1) return lines[0]
  return lines.join(' \\\n')
})

// ---------- Output: app-config.json ----------
const appConfigOutput = computed(() => {
  if (!config.appConfigEnabled) return '(Enable App Config to generate)'

  const obj = {}

  if (config.appConfig.scopes.length) {
    obj.scopes = config.appConfig.scopes
      .filter(s => s.pattern)
      .map(s => `${s.action}:${s.pattern}`)
  }

  if (config.appConfig.writeRules.length) {
    obj.write_rules = config.appConfig.writeRules
      .filter(r => r.path)
      .map(r => {
        const rule = { path: r.path }
        if (r.mode === 'any') rule.mode = 'any'
        if (r.allowNullWrite) rule.allow_null_write = true
        if (r.preChecks?.length) {
          rule.pre_checks = r.preChecks.filter(c => c.type).map(formatCheck)
        }
        if (r.checks?.length) {
          rule.checks = r.checks.filter(c => c.type).map(formatCheck)
        }
        return rule
      })
  }

  if (config.appConfig.snapshotVisibility.length) {
    obj.snapshot_visibility = config.appConfig.snapshotVisibility.map(v => {
      const rule = {}
      if (v.matchMode === 'path' && v.path) rule.path = v.path
      if (v.matchMode === 'contains' && v.pathContains) rule.path_contains = v.pathContains
      if (v.matchMode === 'catchall') rule.path = '**'
      rule.visible = v.visible
      if (v.visible === 'owner') {
        if (v.ownerSegment) rule.owner_segment = v.ownerSegment
        if (v.publicSub) rule.public_sub = v.publicSub
      }
      if (v.visible === 'require_state_not_null' && v.lookup) {
        rule.lookup = v.lookup
      }
      return rule
    })
  }

  if (config.appConfig.snapshotTransforms.length) {
    obj.snapshot_transforms = config.appConfig.snapshotTransforms
      .filter(t => t.path)
      .map(t => ({
        path: t.path,
        redact_fields: t.redactFields.split(',').map(f => f.trim()).filter(Boolean)
      }))
  }

  const rl = config.appConfig.rateLimits
  if (rl.loginMaxAttempts !== 5 || rl.loginWindowSecs !== 60 ||
      rl.registerMaxAttempts !== 10 || rl.registerWindowSecs !== 60) {
    obj.rate_limits = {
      login_max_attempts: rl.loginMaxAttempts,
      login_window_secs: rl.loginWindowSecs,
      register_max_attempts: rl.registerMaxAttempts,
      register_window_secs: rl.registerWindowSecs
    }
  }

  return JSON.stringify(obj, null, 2)
})

function formatCheck(c) {
  const out = { type: c.type }
  if (c.type === 'segment_equals_session' && c.segment) out.segment = c.segment
  if (c.type === 'state_field_equals_session') {
    if (c.lookup) out.lookup = c.lookup
    if (c.field) out.field = c.field
    if (c.allow_if_missing) out.allow_if_missing = true
  }
  if (c.type === 'state_not_null' && c.lookup) out.lookup = c.lookup
  if (c.type === 'value_field_equals_session' && c.field) out.field = c.field
  if (c.type === 'either_state_not_null') {
    if (c.lookup_a) out.lookup_a = c.lookup_a
    if (c.lookup_b) out.lookup_b = c.lookup_b
  }
  if (c.type === 'require_value_field' && c.field) out.field = c.field
  if (c.type === 'reject_unless_path_matches') {
    if (c.pattern) out.pattern = c.pattern
    if (c.message) out.message = c.message
  }
  return out
}

// ---------- Output: rules.json ----------
const rulesOutput = computed(() => {
  if (!config.rulesEnabled || !config.rules.length) return '(Enable Rules Engine and add rules to generate)'

  const rules = config.rules
    .filter(r => r.name && r.trigger?.type)
    .map(r => {
      const rule = { id: r.id, name: r.name, trigger: { ...r.trigger } }
      if (r.conditions?.length) rule.conditions = r.conditions.filter(c => c.address)
      if (r.actions?.length) {
        rule.actions = r.actions.map(a => {
          const out = { type: a.type }
          if (a.address) out.address = a.address
          if (a.type === 'set_from_trigger') {
            if (a.transform) out.transform = a.transform
          } else if (a.value !== undefined && a.value !== '') {
            out.value = a.value
          }
          if (a.type === 'delay' && a.delayMs) out.delay_ms = a.delayMs
          return out
        })
      }
      if (r.cooldown) rule.cooldown_ms = r.cooldown
      return rule
    })

  return JSON.stringify({ rules }, null, 2)
})

// ---------- Output: Client Examples ----------
const clientOutput = computed(() => {
  const wsUrl = `ws://${config.host === '0.0.0.0' ? 'localhost' : config.host}:${config.wsPort}`
  const authUrl = config.authEnabled ? `http://${config.host === '0.0.0.0' ? 'localhost' : config.host}:${config.authPort}` : ''

  let js = `import { ClaspBuilder } from '@clasp-to/core'\n\n`
  js += `const client = await new ClaspBuilder('${wsUrl}')\n`
  js += `  .withName('My App')\n`
  if (config.authEnabled) {
    js += `  // After login, pass the token:\n`
    js += `  // .withAuth(token)\n`
  }
  js += `  .connect()\n\n`
  js += `// Set a value\nclient.set('/my/path', { value: 42 })\n\n`
  js += `// Subscribe to changes\nclient.on('/my/**', (value, address) => {\n  console.log(address, value)\n})`

  let rust = `use clasp_client::ClaspClient;\n\n`
  rust += `let client = ClaspClient::builder("${wsUrl}")\n`
  rust += `    .name("My App")\n`
  if (config.authEnabled) {
    rust += `    // .auth(token)\n`
  }
  rust += `    .connect()\n`
  rust += `    .await?;\n\n`
  rust += `client.set("/my/path", json!({ "value": 42 })).await?;`

  return { js, rust }
})

// ---------- Output: Admin Commands ----------
const adminOutput = computed(() => {
  if (!config.authEnabled) return '(Enable Auth to generate admin commands)'

  const base = `http://${config.host === '0.0.0.0' ? 'localhost' : config.host}:${config.authPort}`

  return `# Register a new user
curl -X POST ${base}/register \\
  -H "Content-Type: application/json" \\
  -d '{"username": "alice", "password": "secret123"}'

# Login
curl -X POST ${base}/login \\
  -H "Content-Type: application/json" \\
  -d '{"username": "alice", "password": "secret123"}'

# Guest token
curl -X POST ${base}/guest \\
  -H "Content-Type: application/json" \\
  -d '{"name": "Visitor"}'`
})

// ---------- Output: Standalone Bridges ----------
const bridgesOutput = computed(() => {
  const target = `ws://${config.host === '0.0.0.0' ? 'localhost' : config.host}:${config.wsPort}`
  const cmds = []

  if (config.artnetEnabled) {
    const parts = [`clasp bridge artnet --target ${target}`]
    if (config.artnetNamespace !== '/artnet') parts.push(`--namespace ${config.artnetNamespace}`)
    if (config.artnetInterface !== '0.0.0.0') parts.push(`--interface ${config.artnetInterface}`)
    if (config.artnetUniverses !== '0-15') parts.push(`--universes ${config.artnetUniverses}`)
    if (config.artnetNormalize) parts.push('--normalize')
    if (config.artnetMode !== 'channel') parts.push(`--mode ${config.artnetMode}`)
    cmds.push('# Art-Net bridge (DMX over Ethernet)\n' + parts.join(' \\\n  '))
  }

  if (config.dmxEnabled) {
    const port = config.dmxSerialPort || '/dev/ttyUSB0'
    const parts = [`clasp bridge dmx --port ${port} --target ${target}`]
    if (config.dmxNamespace !== '/dmx') parts.push(`--namespace ${config.dmxNamespace}`)
    if (config.dmxBaud !== 250000) parts.push(`--baud ${config.dmxBaud}`)
    if (config.dmxDirection !== 'both') parts.push(`--direction ${config.dmxDirection}`)
    if (config.dmxChannels !== '1-512') parts.push(`--channels ${config.dmxChannels}`)
    if (config.dmxRefresh !== 40) parts.push(`--refresh ${config.dmxRefresh}`)
    cmds.push('# DMX bridge (USB serial)\n' + parts.join(' \\\n  '))
  }

  if (config.sacnEnabled) {
    const parts = [`clasp bridge sacn --target ${target}`]
    if (config.sacnNamespace !== '/sacn') parts.push(`--namespace ${config.sacnNamespace}`)
    if (config.sacnInterface !== '0.0.0.0') parts.push(`--interface ${config.sacnInterface}`)
    if (config.sacnUniverses !== '1-10') parts.push(`--universes ${config.sacnUniverses}`)
    if (config.sacnPriority !== 100) parts.push(`--priority ${config.sacnPriority}`)
    if (config.sacnMode !== 'channel') parts.push(`--mode ${config.sacnMode}`)
    if (config.sacnSourceName !== 'clasp-sacn') parts.push(`--source-name "${config.sacnSourceName}"`)
    cmds.push('# sACN bridge (E1.31 multicast)\n' + parts.join(' \\\n  '))
  }

  if (config.midiEnabled) {
    const parts = [`clasp bridge midi --target ${target}`]
    if (config.midiDevice) parts.push(`--device "${config.midiDevice}"`)
    if (config.midiNamespace !== '/midi') parts.push(`--namespace ${config.midiNamespace}`)
    cmds.push('# MIDI bridge\n' + parts.join(' \\\n  '))
  }

  if (config.httpBridgeEnabled) {
    const parts = [`clasp bridge http --port ${config.httpBridgePort} --target ${target}`]
    if (config.httpBridgeBind !== '0.0.0.0') parts.push(`--bind ${config.httpBridgeBind}`)
    if (config.httpBridgeCors !== '*') parts.push(`--cors "${config.httpBridgeCors}"`)
    if (config.httpBridgeToken) parts.push(`--token "${config.httpBridgeToken}"`)
    cmds.push('# HTTP REST bridge\n' + parts.join(' \\\n  '))
  }

  if (config.wsBridgeEnabled) {
    const parts = [`clasp bridge websocket --port ${config.wsBridgePort} --target ${target}`]
    if (config.wsBridgeBind !== '0.0.0.0') parts.push(`--bind ${config.wsBridgeBind}`)
    if (config.wsBridgeToken) parts.push(`--token "${config.wsBridgeToken}"`)
    cmds.push('# WebSocket JSON bridge\n' + parts.join(' \\\n  '))
  }

  if (!cmds.length) return '(Enable standalone bridges to generate commands)'
  return cmds.join('\n\n')
})

// ---------- Port Conflict Detection ----------
const portConflicts = computed(() => {
  const ports = {}
  const add = (port, label) => {
    if (!port) return
    const key = String(port)
    if (!ports[key]) ports[key] = []
    ports[key].push(label)
  }
  add(config.wsPort, 'WebSocket')
  if (config.authEnabled) add(config.authPort, 'Auth')
  if (config.quicEnabled) add(config.quicPort, 'QUIC')
  if (config.mqttEnabled) add(config.mqttPort, 'MQTT (embedded)')
  if (config.oscEnabled) add(config.oscPort, 'OSC (embedded)')
  if (config.healthEnabled) add(config.healthPort, 'Health')
  if (config.metricsEnabled) add(config.metricsPort, 'Metrics')
  add(config.rendezvousPort, 'Rendezvous')
  if (config.httpBridgeEnabled) add(config.httpBridgePort, 'HTTP Bridge')
  if (config.wsBridgeEnabled) add(config.wsBridgePort, 'WS Bridge')

  return Object.entries(ports)
    .filter(([, labels]) => labels.length > 1)
    .map(([port, labels]) => `Port ${port}: ${labels.join(', ')}`)
})
</script>

<template>
  <div class="relay-configurator">
    <div class="rc-form">
      <div class="rc-header">
        <h1>RELAY CONFIGURATOR</h1>
        <p class="rc-subtitle">Build your clasp-relay CLI command, app-config, and rules visually.</p>
        <div class="rc-toolbar">
          <PresetSelector @apply="applyPreset" />
          <button class="rc-link-btn" @click="copyLink">
            {{ linkCopied ? 'Link copied' : 'Copy Link' }}
          </button>
        </div>
      </div>

      <div v-if="portConflicts.length" class="rc-warnings">
        <div v-for="w in portConflicts" :key="w" class="rc-warning">Port conflict: {{ w }}</div>
      </div>

      <!-- CORE -->
      <ConfigSection title="Core" default-open>
        <FieldRow label="WS Port" type="number" v-model="config.wsPort" hint="default: 7330" :min="1" :max="65535" />
        <FieldRow label="Host" v-model="config.host" hint="default: 0.0.0.0" />
        <FieldRow label="Name" v-model="config.name" hint="default: CLASP Relay" />
        <FieldRow label="No WebSocket" hint="disable WS transport">
          <ToggleSwitch v-model="config.noWebsocket" />
        </FieldRow>
        <FieldRow label="Max Sessions" type="number" v-model="config.maxSessions" hint="default: 1000" />
        <FieldRow label="Session Timeout" type="number" v-model="config.sessionTimeout" hint="seconds, default: 300" />
      </ConfigSection>

      <!-- AUTH -->
      <ConfigSection title="Auth" :badge="config.authEnabled ? 'ON' : ''">
        <FieldRow label="Enable Auth">
          <ToggleSwitch v-model="config.authEnabled" />
        </FieldRow>
        <template v-if="config.authEnabled">
          <FieldRow label="Auth Port" type="number" v-model="config.authPort" hint="default: 7350" />
          <FieldRow label="Auth DB" v-model="config.authDb" hint="default: relay-auth.db" />
          <FieldRow label="Admin Token Path" v-model="config.adminTokenPath" placeholder="/path/to/admin.token" />
          <FieldRow label="Token TTL" type="number" v-model="config.tokenTtl" hint="seconds, default: 86400" />
          <FieldRow label="CORS Origin" v-model="config.corsOrigin" placeholder="https://app.example.com" />
        </template>
      </ConfigSection>

      <!-- TRANSPORTS & EMBEDDED BRIDGES -->
      <ConfigSection title="Transports & Embedded Bridges" :badge="[config.quicEnabled && 'QUIC', config.mqttEnabled && 'MQTT', config.oscEnabled && 'OSC'].filter(Boolean).join('+') || ''">
        <div class="rc-subsection">
          <FieldRow label="QUIC / TLS">
            <ToggleSwitch v-model="config.quicEnabled" />
          </FieldRow>
          <template v-if="config.quicEnabled">
            <FieldRow label="QUIC Port" type="number" v-model="config.quicPort" hint="default: 7331" />
            <FieldRow label="Cert Path" v-model="config.certPath" placeholder="/path/to/cert.pem" />
            <FieldRow label="Key Path" v-model="config.keyPath" placeholder="/path/to/key.pem" />
          </template>
        </div>
        <div class="rc-subsection">
          <FieldRow label="MQTT Bridge">
            <ToggleSwitch v-model="config.mqttEnabled" />
          </FieldRow>
          <template v-if="config.mqttEnabled">
            <FieldRow label="MQTT Port" type="number" v-model="config.mqttPort" hint="default: 1883" />
            <FieldRow label="MQTT Namespace" v-model="config.mqttNamespace" hint="default: /mqtt" />
          </template>
        </div>
        <div class="rc-subsection">
          <FieldRow label="OSC Bridge">
            <ToggleSwitch v-model="config.oscEnabled" />
          </FieldRow>
          <template v-if="config.oscEnabled">
            <FieldRow label="OSC Port" type="number" v-model="config.oscPort" hint="default: 9000" />
            <FieldRow label="OSC Namespace" v-model="config.oscNamespace" hint="default: /osc" />
          </template>
        </div>
      </ConfigSection>

      <!-- STANDALONE BRIDGES -->
      <ConfigSection title="Standalone Bridges" :badge="[config.artnetEnabled && 'ArtNet', config.dmxEnabled && 'DMX', config.sacnEnabled && 'sACN', config.midiEnabled && 'MIDI', config.httpBridgeEnabled && 'HTTP', config.wsBridgeEnabled && 'WS'].filter(Boolean).join('+') || ''">
        <p class="rc-section-note">Separate processes that connect to the relay as clients. Each runs as <code>clasp bridge &lt;protocol&gt;</code>.</p>

        <div class="rc-subsection">
          <FieldRow label="Art-Net">
            <ToggleSwitch v-model="config.artnetEnabled" />
          </FieldRow>
          <template v-if="config.artnetEnabled">
            <FieldRow label="Namespace" v-model="config.artnetNamespace" hint="default: /artnet" />
            <FieldRow label="Interface" v-model="config.artnetInterface" hint="default: 0.0.0.0" />
            <FieldRow label="Universes" v-model="config.artnetUniverses" hint="default: 0-15" />
            <FieldRow label="Normalize" hint="map 0-255 to 0.0-1.0">
              <ToggleSwitch v-model="config.artnetNormalize" />
            </FieldRow>
            <FieldRow label="Mode">
              <select class="rc-select" v-model="config.artnetMode">
                <option value="channel">channel</option>
                <option value="universe">universe</option>
              </select>
            </FieldRow>
          </template>
        </div>

        <div class="rc-subsection">
          <FieldRow label="DMX (USB)">
            <ToggleSwitch v-model="config.dmxEnabled" />
          </FieldRow>
          <template v-if="config.dmxEnabled">
            <FieldRow label="Serial Port" v-model="config.dmxSerialPort" placeholder="/dev/ttyUSB0" />
            <FieldRow label="Namespace" v-model="config.dmxNamespace" hint="default: /dmx" />
            <FieldRow label="Baud Rate" type="number" v-model="config.dmxBaud" hint="default: 250000" />
            <FieldRow label="Direction">
              <select class="rc-select" v-model="config.dmxDirection">
                <option value="both">both</option>
                <option value="input">input</option>
                <option value="output">output</option>
              </select>
            </FieldRow>
            <FieldRow label="Channels" v-model="config.dmxChannels" hint="default: 1-512" />
            <FieldRow label="Refresh (Hz)" type="number" v-model="config.dmxRefresh" hint="default: 40" />
          </template>
        </div>

        <div class="rc-subsection">
          <FieldRow label="sACN (E1.31)">
            <ToggleSwitch v-model="config.sacnEnabled" />
          </FieldRow>
          <template v-if="config.sacnEnabled">
            <FieldRow label="Namespace" v-model="config.sacnNamespace" hint="default: /sacn" />
            <FieldRow label="Interface" v-model="config.sacnInterface" hint="default: 0.0.0.0" />
            <FieldRow label="Universes" v-model="config.sacnUniverses" hint="default: 1-10" />
            <FieldRow label="Priority" type="number" v-model="config.sacnPriority" hint="default: 100" :min="0" :max="200" />
            <FieldRow label="Mode">
              <select class="rc-select" v-model="config.sacnMode">
                <option value="channel">channel</option>
                <option value="universe">universe</option>
              </select>
            </FieldRow>
            <FieldRow label="Source Name" v-model="config.sacnSourceName" hint="default: clasp-sacn" />
          </template>
        </div>

        <div class="rc-subsection">
          <FieldRow label="MIDI">
            <ToggleSwitch v-model="config.midiEnabled" />
          </FieldRow>
          <template v-if="config.midiEnabled">
            <FieldRow label="Device" v-model="config.midiDevice" placeholder="Launchpad Pro (optional)" />
            <FieldRow label="Namespace" v-model="config.midiNamespace" hint="default: /midi" />
          </template>
        </div>

        <div class="rc-subsection">
          <FieldRow label="HTTP REST">
            <ToggleSwitch v-model="config.httpBridgeEnabled" />
          </FieldRow>
          <template v-if="config.httpBridgeEnabled">
            <FieldRow label="Port" type="number" v-model="config.httpBridgePort" hint="default: 8080" />
            <FieldRow label="Bind" v-model="config.httpBridgeBind" hint="default: 0.0.0.0" />
            <FieldRow label="CORS" v-model="config.httpBridgeCors" hint="default: *" />
            <FieldRow label="Auth Token" v-model="config.httpBridgeToken" placeholder="optional Bearer token" />
          </template>
        </div>

        <div class="rc-subsection">
          <FieldRow label="WebSocket JSON">
            <ToggleSwitch v-model="config.wsBridgeEnabled" />
          </FieldRow>
          <template v-if="config.wsBridgeEnabled">
            <FieldRow label="Port" type="number" v-model="config.wsBridgePort" hint="default: 9000" />
            <FieldRow label="Bind" v-model="config.wsBridgeBind" hint="default: 0.0.0.0" />
            <FieldRow label="Auth Token" v-model="config.wsBridgeToken" placeholder="optional token" />
          </template>
        </div>
      </ConfigSection>

      <!-- TTL -->
      <ConfigSection title="TTL">
        <FieldRow label="Param TTL" type="number" v-model="config.paramTtl" hint="seconds, default: 3600" />
        <FieldRow label="Signal TTL" type="number" v-model="config.signalTtl" hint="seconds, default: 3600" />
        <FieldRow label="No TTL" hint="disable all TTL expiry">
          <ToggleSwitch v-model="config.noTtl" />
        </FieldRow>
      </ConfigSection>

      <!-- PERSISTENCE -->
      <ConfigSection title="Persistence" :badge="config.persistEnabled ? 'ON' : ''">
        <FieldRow label="Enable Persistence">
          <ToggleSwitch v-model="config.persistEnabled" />
        </FieldRow>
        <template v-if="config.persistEnabled">
          <FieldRow label="Persist Path" v-model="config.persistPath" placeholder="./relay-data" />
          <FieldRow label="Persist Interval" type="number" v-model="config.persistInterval" hint="seconds, default: 30" />
        </template>
        <FieldRow label="Enable Journal">
          <ToggleSwitch v-model="config.journalEnabled" />
        </FieldRow>
        <template v-if="config.journalEnabled">
          <FieldRow label="Journal Path" v-model="config.journalPath" placeholder="./journal" />
          <FieldRow label="In-Memory Journal" hint="store journal in memory only">
            <ToggleSwitch v-model="config.journalMemory" />
          </FieldRow>
        </template>
      </ConfigSection>

      <!-- APP CONFIG -->
      <ConfigSection title="App Config" :badge="config.appConfigEnabled ? 'ON' : ''">
        <FieldRow label="Enable App Config">
          <ToggleSwitch v-model="config.appConfigEnabled" />
        </FieldRow>
        <template v-if="config.appConfigEnabled">
          <div class="rc-subsection">
            <div class="rc-subsection-label">SCOPES</div>
            <ScopeBuilder v-model="config.appConfig.scopes" />
          </div>

          <div class="rc-subsection">
            <div class="rc-subsection-label">WRITE RULES</div>
            <WriteRuleBuilder v-model="config.appConfig.writeRules" />
          </div>

          <div class="rc-subsection">
            <div class="rc-subsection-label">SNAPSHOT VISIBILITY</div>
            <VisibilityBuilder v-model="config.appConfig.snapshotVisibility" />
          </div>

          <div class="rc-subsection">
            <div class="rc-subsection-label">SNAPSHOT TRANSFORMS</div>
            <TransformBuilder v-model="config.appConfig.snapshotTransforms" />
          </div>

          <div class="rc-subsection">
            <div class="rc-subsection-label">RATE LIMITS</div>
            <FieldRow label="Login Max Attempts" type="number" v-model="config.appConfig.rateLimits.loginMaxAttempts" hint="default: 5" />
            <FieldRow label="Login Window (s)" type="number" v-model="config.appConfig.rateLimits.loginWindowSecs" hint="default: 60" />
            <FieldRow label="Register Max" type="number" v-model="config.appConfig.rateLimits.registerMaxAttempts" hint="default: 10" />
            <FieldRow label="Register Window (s)" type="number" v-model="config.appConfig.rateLimits.registerWindowSecs" hint="default: 60" />
          </div>
        </template>
      </ConfigSection>

      <!-- RULES ENGINE -->
      <ConfigSection title="Rules Engine" :badge="config.rulesEnabled ? 'ON' : ''">
        <FieldRow label="Enable Rules">
          <ToggleSwitch v-model="config.rulesEnabled" />
        </FieldRow>
        <template v-if="config.rulesEnabled">
          <FieldRow label="Rules File Path" v-model="config.rulesPath" hint="default: rules.json" />
          <RuleBuilder v-model="config.rules" />
        </template>
      </ConfigSection>

      <!-- FEDERATION -->
      <ConfigSection title="Federation" :badge="config.federationEnabled ? 'ON' : ''">
        <FieldRow label="Enable Federation">
          <ToggleSwitch v-model="config.federationEnabled" />
        </FieldRow>
        <template v-if="config.federationEnabled">
          <FieldRow label="Hub URL" v-model="config.federationHub" placeholder="ws://hub.example.com:7330" />
          <FieldRow label="Federation ID" v-model="config.federationId" placeholder="site-01" />
          <FieldRow label="Federation Token" v-model="config.federationToken" placeholder="shared-secret" />
        </template>
      </ConfigSection>

      <!-- OPERATIONS -->
      <ConfigSection title="Operations">
        <FieldRow label="Health Check">
          <ToggleSwitch v-model="config.healthEnabled" />
        </FieldRow>
        <template v-if="config.healthEnabled">
          <FieldRow label="Health Port" type="number" v-model="config.healthPort" hint="default: 8080" />
        </template>
        <FieldRow label="Metrics">
          <ToggleSwitch v-model="config.metricsEnabled" />
        </FieldRow>
        <template v-if="config.metricsEnabled">
          <FieldRow label="Metrics Port" type="number" v-model="config.metricsPort" hint="default: 9090" />
        </template>
        <FieldRow label="Drain Timeout" type="number" v-model="config.drainTimeout" hint="seconds, default: 30" />
        <FieldRow label="Rendezvous Port" type="number" v-model="config.rendezvousPort" hint="default: 7340" />
        <FieldRow label="Rendezvous TTL" type="number" v-model="config.rendezvousTtl" hint="seconds, default: 300" />
      </ConfigSection>
    </div>

    <!-- OUTPUT PANEL -->
    <div class="rc-output">
      <div class="rc-output-sticky">
        <div class="rc-tabs">
          <button
            v-for="tab in [
              { key: 'cli', label: 'CLI' },
              { key: 'bridges', label: 'Bridges' },
              { key: 'json', label: 'JSON' },
              { key: 'rules', label: 'Rules' },
              { key: 'client', label: 'Client' },
              { key: 'admin', label: 'Admin' }
            ]"
            :key="tab.key"
            class="rc-tab"
            :class="{ active: activeTab === tab.key }"
            @click="activeTab = tab.key"
          >
            {{ tab.label }}
          </button>
        </div>

        <div class="rc-tab-content">
          <CopyBlock v-if="activeTab === 'cli'" :code="cliOutput" lang="bash" />

          <CopyBlock v-if="activeTab === 'bridges'" :code="bridgesOutput" lang="bash" />

          <CopyBlock v-if="activeTab === 'json'" :code="appConfigOutput" lang="json" />

          <CopyBlock v-if="activeTab === 'rules'" :code="rulesOutput" lang="json" />

          <template v-if="activeTab === 'client'">
            <CopyBlock :code="clientOutput.js" lang="javascript" />
            <CopyBlock :code="clientOutput.rust" lang="rust" />
          </template>

          <CopyBlock v-if="activeTab === 'admin'" :code="adminOutput" lang="bash" />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.relay-configurator {
  display: grid;
  grid-template-columns: 1fr;
  min-height: calc(100vh - var(--nav-height));
  max-width: 1600px;
  margin: 0 auto;
}

.rc-form {
  padding: 1rem 1rem 3rem;
  overflow-y: auto;
}

.rc-header {
  margin-bottom: 1.5rem;
}

.rc-header h1 {
  font-size: 1rem;
  letter-spacing: 0.2em;
  margin: 0 0 0.3rem;
}

.rc-subtitle {
  font-size: 0.82rem;
  opacity: 0.6;
  margin: 0 0 1rem;
}

.rc-toolbar {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  flex-wrap: wrap;
}

.rc-link-btn {
  font-family: 'Space Mono', monospace;
  font-size: 0.72rem;
  padding: 0.25rem 0.6rem;
  background: none;
  border: 1px solid var(--border);
  color: var(--ink);
  cursor: pointer;
  border-radius: 3px;
  letter-spacing: 0.04em;
  transition: border-color 0.15s;
}

.rc-link-btn:hover {
  border-color: var(--accent);
  color: var(--accent);
}

.rc-warnings {
  margin-bottom: 1rem;
}

.rc-warning {
  padding: 0.4rem 0.75rem;
  background: rgba(247, 127, 0, 0.12);
  border-left: 3px solid var(--accent4);
  font-size: 0.78rem;
  margin-bottom: 0.3rem;
}

.rc-subsection {
  margin-top: 0.75rem;
  padding-top: 0.75rem;
  border-top: 1px solid var(--border);
}

.rc-subsection:first-child {
  margin-top: 0;
  padding-top: 0;
  border-top: none;
}

.rc-subsection-label {
  font-family: 'Archivo Black', sans-serif;
  font-size: 0.65rem;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  opacity: 0.45;
  margin-bottom: 0.4rem;
}

.rc-section-note {
  font-size: 0.75rem;
  opacity: 0.5;
  margin: 0 0 0.75rem;
  line-height: 1.4;
}

.rc-section-note code {
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.72rem;
  background: var(--code-bg);
  padding: 0.1em 0.3em;
  border-radius: 2px;
}

.rc-select {
  padding: 0.4rem 0.6rem;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.82rem;
  background: var(--code-bg);
  border: 1px solid var(--border);
  border-radius: 3px;
  color: var(--ink);
  cursor: pointer;
}

.rc-select:focus {
  outline: none;
  border-color: var(--accent);
}

/* Output panel */
.rc-output {
  border-left: none;
  border-top: 1px solid var(--border);
  background: var(--sidebar-bg);
}

.rc-output-sticky {
  position: static;
  height: auto;
  max-height: 50vh;
  overflow-y: auto;
  padding: 1rem;
}

.rc-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 0;
  border-bottom: 1px solid var(--border);
  margin-bottom: 1rem;
}

.rc-tab {
  font-family: 'Archivo Black', sans-serif;
  font-size: 0.68rem;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  padding: 0.5rem 0.75rem;
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  color: var(--ink);
  opacity: 0.5;
  cursor: pointer;
  transition: opacity 0.15s, border-color 0.15s;
}

.rc-tab:hover {
  opacity: 0.8;
}

.rc-tab.active {
  opacity: 1;
  border-bottom-color: var(--accent);
}

.rc-tab-content {
  min-height: 200px;
}

/* Responsive */
@media (min-width: 1024px) {
  .relay-configurator {
    grid-template-columns: 55% 45%;
  }

  .rc-form {
    padding: 2rem 2rem 4rem;
  }

  .rc-header h1 {
    font-size: 1.4rem;
  }

  .rc-output {
    border-left: 1px solid var(--border);
    border-top: none;
  }

  .rc-output-sticky {
    position: sticky;
    top: var(--nav-height);
    height: calc(100vh - var(--nav-height));
    max-height: none;
    padding: 1.5rem;
  }

  .rc-tabs {
    flex-wrap: nowrap;
  }
}
</style>
