<script setup>
import { ref } from 'vue'

const emit = defineEmits(['apply'])

const PRESETS = {
  minimal: {
    label: 'Minimal (Dev)',
    description: 'Defaults only, nothing extra',
    config: {}
  },
  chat: {
    label: 'Chat App',
    description: 'Auth, app-config with chat scopes + write rules + visibility, persistence',
    config: {
      authEnabled: true,
      authPort: 7350,
      authDb: 'relay-auth.db',
      tokenTtl: 86400,
      persistEnabled: true,
      persistPath: 'relay-data',
      persistInterval: 30,
      appConfigEnabled: true,
      appConfig: {
        scopes: [
          { action: 'read', pattern: '/chat/user/{userId}/**' },
          { action: 'write', pattern: '/chat/user/{userId}/**' },
          { action: 'write', pattern: '/chat/room/*/messages/**' },
          { action: 'subscribe', pattern: '/chat/room/**' },
          { action: 'read', pattern: '/chat/registry/**' },
          { action: 'subscribe', pattern: '/chat/registry/**' }
        ],
        writeRules: [
          {
            path: '/chat/user/{userId}/**',
            mode: 'all',
            allowNullWrite: false,
            preChecks: [],
            checks: [{ type: 'segment_equals_session', segment: 'userId' }]
          },
          {
            path: '/chat/room/{roomId}/messages/{msgId}',
            mode: 'all',
            allowNullWrite: false,
            preChecks: [],
            checks: [
              { type: 'value_field_equals_session', field: 'userId' },
              { type: 'require_value_field', field: 'timestamp' }
            ]
          }
        ],
        snapshotVisibility: [
          { matchMode: 'path', path: '/chat/user/*/private/**', pathContains: '', visible: 'owner', ownerSegment: 'userId', publicSub: '', lookup: '' },
          { matchMode: 'catchall', path: '', pathContains: '', visible: true, ownerSegment: '', publicSub: '', lookup: '' }
        ],
        snapshotTransforms: [],
        rateLimits: { loginMaxAttempts: 5, loginWindowSecs: 60, registerMaxAttempts: 10, registerWindowSecs: 60 }
      }
    }
  },
  iot: {
    label: 'IoT Sensors',
    description: 'MQTT embedded + HTTP bridge, rules engine, persistence, no auth',
    config: {
      mqttEnabled: true,
      mqttPort: 1883,
      mqttNamespace: '/mqtt',
      httpBridgeEnabled: true,
      httpBridgePort: 8080,
      httpBridgeCors: '*',
      persistEnabled: true,
      persistPath: 'sensor-data',
      persistInterval: 10,
      rulesEnabled: true,
      rulesPath: 'rules.json',
      rules: [
        {
          id: 'rule_1',
          name: 'Temperature Alert',
          trigger: { type: 'on_threshold', address: '/sensors/*/temperature', threshold: 40, direction: 'above' },
          conditions: [],
          actions: [
            { type: 'set', address: '/alerts/temperature', value: 'HIGH' },
            { type: 'publish', address: '/notifications/alert', value: 'Temperature exceeded threshold' }
          ],
          cooldown: 30000
        }
      ]
    }
  },
  lighting: {
    label: 'Lighting Control',
    description: 'OSC embedded + Art-Net, DMX, sACN, MIDI bridges, rules, short TTL',
    config: {
      oscEnabled: true,
      oscPort: 9000,
      oscNamespace: '/osc',
      artnetEnabled: true,
      artnetUniverses: '0-3',
      artnetNormalize: true,
      dmxEnabled: true,
      dmxSerialPort: '/dev/ttyUSB0',
      sacnEnabled: true,
      sacnUniverses: '1-4',
      midiEnabled: true,
      paramTtl: 60,
      signalTtl: 30,
      rulesEnabled: true,
      rulesPath: 'rules.json',
      rules: [
        {
          id: 'rule_1',
          name: 'Master Dimmer Scale',
          trigger: { type: 'on_change', address: '/lights/master' },
          conditions: [],
          actions: [
            { type: 'set_from_trigger', address: '/dmx/1/dimmer', value: '', transform: 'value * 255' }
          ],
          cooldown: 0
        }
      ]
    }
  },
  production: {
    label: 'Production',
    description: 'Auth, QUIC+TLS, persistence+journal, health+metrics, HTTP+WS bridges, CORS, drain',
    config: {
      authEnabled: true,
      authPort: 7350,
      authDb: 'relay-auth.db',
      tokenTtl: 3600,
      corsOrigin: 'https://app.example.com',
      quicEnabled: true,
      quicPort: 7331,
      certPath: '/etc/certs/relay.crt',
      keyPath: '/etc/certs/relay.key',
      persistEnabled: true,
      persistPath: '/var/lib/clasp/data',
      persistInterval: 15,
      journalEnabled: true,
      journalPath: '/var/lib/clasp/journal',
      healthEnabled: true,
      healthPort: 8080,
      metricsEnabled: true,
      metricsPort: 9090,
      drainTimeout: 30,
      httpBridgeEnabled: true,
      httpBridgePort: 8081,
      httpBridgeCors: 'https://app.example.com',
      wsBridgeEnabled: true,
      wsBridgePort: 9000
    }
  }
}

const showConfirm = ref(false)
const pendingPreset = ref(null)

function selectPreset(key) {
  pendingPreset.value = key
  showConfirm.value = true
}

function confirm() {
  if (pendingPreset.value) {
    emit('apply', PRESETS[pendingPreset.value].config)
  }
  showConfirm.value = false
  pendingPreset.value = null
}

function cancel() {
  showConfirm.value = false
  pendingPreset.value = null
}
</script>

<template>
  <div class="preset-selector">
    <span class="preset-label">Presets:</span>
    <button
      v-for="(preset, key) in PRESETS"
      :key="key"
      class="preset-btn"
      :title="preset.description"
      @click="selectPreset(key)"
    >
      {{ preset.label }}
    </button>

    <div v-if="showConfirm" class="preset-confirm-overlay" @click.self="cancel">
      <div class="preset-confirm">
        <p>Apply <strong>{{ PRESETS[pendingPreset]?.label }}</strong> preset?</p>
        <p class="preset-confirm-desc">{{ PRESETS[pendingPreset]?.description }}</p>
        <p class="preset-confirm-warn">This will overwrite your current configuration.</p>
        <div class="preset-confirm-actions">
          <button class="preset-confirm-btn preset-confirm-apply" @click="confirm">Apply</button>
          <button class="preset-confirm-btn" @click="cancel">Cancel</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.preset-selector {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  flex-wrap: wrap;
}

.preset-label {
  font-size: 0.72rem;
  letter-spacing: 0.1em;
  opacity: 0.5;
  font-weight: 700;
  text-transform: uppercase;
}

.preset-btn {
  font-family: 'Space Mono', monospace;
  font-size: 0.72rem;
  padding: 0.25rem 0.6rem;
  background: var(--code-bg);
  border: 1px solid var(--border);
  color: var(--ink);
  cursor: pointer;
  border-radius: 3px;
  letter-spacing: 0.04em;
  transition: border-color 0.15s, color 0.15s;
}

.preset-btn:hover {
  border-color: var(--accent);
  color: var(--accent);
}

.preset-confirm-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.preset-confirm {
  background: var(--paper);
  border: 1px solid var(--border);
  padding: 1.5rem;
  max-width: 400px;
  width: 90%;
}

.preset-confirm p {
  margin: 0 0 0.5rem;
  font-size: 0.85rem;
}

.preset-confirm-desc {
  opacity: 0.7;
  font-size: 0.8rem !important;
}

.preset-confirm-warn {
  color: var(--accent4);
  font-size: 0.75rem !important;
}

.preset-confirm-actions {
  display: flex;
  gap: 0.5rem;
  margin-top: 1rem;
}

.preset-confirm-btn {
  font-family: 'Space Mono', monospace;
  font-size: 0.78rem;
  padding: 0.35rem 0.75rem;
  background: none;
  border: 1px solid var(--border);
  color: var(--ink);
  cursor: pointer;
  border-radius: 3px;
  letter-spacing: 0.04em;
  transition: border-color 0.15s;
}

.preset-confirm-btn:hover {
  border-color: var(--accent);
}

.preset-confirm-apply {
  background: var(--accent);
  color: #fff;
  border-color: var(--accent);
}

.preset-confirm-apply:hover {
  opacity: 0.9;
}
</style>
