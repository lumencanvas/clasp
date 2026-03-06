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
        placeholder="6454"
        :value="modelValue.port ?? 6454"
        @input="update('port', Number(($event.target as HTMLInputElement).value))"
      />
    </div>
    <div class="form-group">
      <label class="form-label">Subnet</label>
      <input
        class="input"
        type="number"
        min="0"
        max="15"
        placeholder="0"
        :value="modelValue.subnet ?? 0"
        @input="update('subnet', Number(($event.target as HTMLInputElement).value))"
      />
    </div>
    <div class="form-group">
      <label class="form-label">Universe</label>
      <input
        class="input"
        type="number"
        min="0"
        max="15"
        placeholder="0"
        :value="modelValue.universe ?? 0"
        @input="update('universe', Number(($event.target as HTMLInputElement).value))"
      />
    </div>

    <CollapsibleSection title="ADVANCED">
      <div class="form-group">
        <label class="form-label">Namespace</label>
        <input
          class="input"
          type="text"
          placeholder="artnet"
          :value="modelValue.namespace ?? ''"
          @input="update('namespace', ($event.target as HTMLInputElement).value)"
        />
      </div>
      <div class="form-group">
        <label class="form-label">
          <input
            type="checkbox"
            :checked="modelValue.normalize ?? false"
            @change="update('normalize', ($event.target as HTMLInputElement).checked)"
          />
          Normalize (0-255 to 0.0-1.0)
        </label>
      </div>
      <div class="form-group">
        <label class="form-label">Mode</label>
        <select
          class="select"
          :value="modelValue.artnetMode ?? 'channel'"
          @change="update('artnetMode', ($event.target as HTMLSelectElement).value)"
        >
          <option value="channel">Per Channel</option>
          <option value="universe">Full Universe</option>
        </select>
      </div>
    </CollapsibleSection>
  </div>
</template>
