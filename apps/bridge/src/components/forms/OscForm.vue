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
      <label class="form-label">Bind Address</label>
      <input
        class="input"
        type="text"
        placeholder="0.0.0.0"
        :value="modelValue.bind ?? '0.0.0.0'"
        @input="update('bind', ($event.target as HTMLInputElement).value)"
      />
    </div>
    <div class="form-group">
      <label class="form-label">Port</label>
      <input
        class="input"
        type="number"
        placeholder="9000"
        :value="modelValue.port ?? 9000"
        @input="update('port', Number(($event.target as HTMLInputElement).value))"
      />
    </div>

    <CollapsibleSection title="ADVANCED">
      <div class="form-group">
        <label class="form-label">Namespace</label>
        <input
          class="input"
          type="text"
          placeholder="/"
          :value="modelValue.namespace ?? '/'"
          @input="update('namespace', ($event.target as HTMLInputElement).value)"
        />
      </div>
    </CollapsibleSection>
  </div>
</template>
