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
        placeholder="3000"
        :value="modelValue.port ?? 3000"
        @input="update('port', Number(($event.target as HTMLInputElement).value))"
      />
    </div>
    <div class="form-group">
      <label class="form-label">Base Path</label>
      <input
        class="input"
        type="text"
        placeholder="/api"
        :value="modelValue.basePath ?? '/api'"
        @input="update('basePath', ($event.target as HTMLInputElement).value)"
      />
    </div>

    <CollapsibleSection title="ADVANCED">
      <div class="form-group">
        <label class="form-label">Namespace</label>
        <input
          class="input"
          type="text"
          placeholder="http"
          :value="modelValue.namespace ?? ''"
          @input="update('namespace', ($event.target as HTMLInputElement).value)"
        />
      </div>
      <div class="form-group">
        <label class="form-label">
          <input
            type="checkbox"
            :checked="modelValue.cors ?? true"
            @change="update('cors', ($event.target as HTMLInputElement).checked)"
          />
          Enable CORS
        </label>
      </div>
      <div class="form-group">
        <label class="form-label">Auth Token</label>
        <input
          class="input"
          type="password"
          placeholder="Bearer token"
          :value="modelValue.authToken ?? ''"
          @input="update('authToken', ($event.target as HTMLInputElement).value)"
        />
      </div>
      <div class="form-group">
        <label class="form-label">Timeout (ms)</label>
        <input
          class="input"
          type="number"
          placeholder="30000"
          :value="modelValue.timeout ?? 30000"
          @input="update('timeout', Number(($event.target as HTMLInputElement).value))"
        />
      </div>
      <div class="form-group">
        <label class="form-label">Poll Interval (ms)</label>
        <input
          class="input"
          type="number"
          placeholder="1000"
          :value="modelValue.pollInterval ?? 1000"
          @input="update('pollInterval', Number(($event.target as HTMLInputElement).value))"
        />
      </div>
    </CollapsibleSection>
  </div>
</template>
