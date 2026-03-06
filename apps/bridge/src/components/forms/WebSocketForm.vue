<script setup lang="ts">
import type { Connection } from '../../lib/types'
import CollapsibleSection from '../shared/CollapsibleSection.vue'

const props = defineProps<{ modelValue: Connection }>()
const emit = defineEmits<{ 'update:modelValue': [value: Connection] }>()

function update(field: string, value: any) {
  emit('update:modelValue', { ...props.modelValue, [field]: value })
}
</script>

<template>
  <div class="protocol-form">
    <div class="form-group">
      <label class="form-label">Host</label>
      <input
        class="input"
        type="text"
        placeholder="0.0.0.0"
        :value="modelValue.host ?? '0.0.0.0'"
        @input="update('host', ($event.target as HTMLInputElement).value)"
      />
    </div>
    <div class="form-group">
      <label class="form-label">Port</label>
      <input
        class="input"
        type="number"
        placeholder="8080"
        :value="modelValue.port ?? 8080"
        @input="update('port', Number(($event.target as HTMLInputElement).value))"
      />
    </div>
    <div class="form-group">
      <label class="form-label">Mode</label>
      <select
        class="select"
        :value="modelValue.mode ?? 'server'"
        @change="update('mode', ($event.target as HTMLSelectElement).value)"
      >
        <option value="server">Server</option>
        <option value="client">Client</option>
      </select>
    </div>

    <CollapsibleSection title="ADVANCED">
      <div class="form-group">
        <label class="form-label">Namespace</label>
        <input
          class="input"
          type="text"
          placeholder="ws"
          :value="modelValue.namespace ?? ''"
          @input="update('namespace', ($event.target as HTMLInputElement).value)"
        />
      </div>
      <div class="form-group">
        <label class="form-label">Format</label>
        <select
          class="select"
          :value="modelValue.format ?? 'json'"
          @change="update('format', ($event.target as HTMLSelectElement).value)"
        >
          <option value="json">JSON</option>
          <option value="msgpack">MessagePack</option>
        </select>
      </div>
      <div class="form-group">
        <label class="form-label">Ping Interval (ms)</label>
        <input
          class="input"
          type="number"
          placeholder="30000"
          :value="modelValue.pingInterval ?? 30000"
          @input="update('pingInterval', Number(($event.target as HTMLInputElement).value))"
        />
      </div>
      <div class="form-group">
        <label class="form-label">
          <input
            type="checkbox"
            :checked="modelValue.autoReconnect ?? true"
            @change="update('autoReconnect', ($event.target as HTMLInputElement).checked)"
          />
          Auto-Reconnect
        </label>
      </div>
      <div class="form-group">
        <label class="form-label">Custom Headers (JSON)</label>
        <input
          class="input"
          type="text"
          placeholder='{"Authorization": "Bearer ..."}'
          :value="modelValue.customHeaders ?? ''"
          @input="update('customHeaders', ($event.target as HTMLInputElement).value)"
        />
      </div>
    </CollapsibleSection>
  </div>
</template>
