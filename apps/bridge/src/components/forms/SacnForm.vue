<script setup lang="ts">
import type { Connection } from '../../lib/types'
import CollapsibleSection from '../shared/CollapsibleSection.vue'

const props = defineProps<{ modelValue: Connection }>()
const emit = defineEmits<{ 'update:modelValue': [value: Connection] }>()

function update(field: string, value: any) {
  emit('update:modelValue', { ...props.modelValue, [field]: value })
}

function universesToText(universes?: number[]): string {
  return (universes ?? []).join(', ')
}

function textToUniverses(text: string): number[] {
  return text
    .split(',')
    .map(s => parseInt(s.trim(), 10))
    .filter(n => !isNaN(n))
}

function destinationsToText(destinations?: string[]): string {
  return (destinations ?? []).join('\n')
}

function textToDestinations(text: string): string[] {
  return text.split('\n').map(s => s.trim()).filter(Boolean)
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
        placeholder="5568"
        :value="modelValue.port ?? 5568"
        @input="update('port', Number(($event.target as HTMLInputElement).value))"
      />
    </div>
    <div class="form-group">
      <label class="form-label">Universes (comma-separated)</label>
      <textarea
        class="input"
        rows="2"
        placeholder="1, 2, 3"
        :value="universesToText(modelValue.universes)"
        @input="update('universes', textToUniverses(($event.target as HTMLTextAreaElement).value))"
      />
    </div>

    <CollapsibleSection title="ADVANCED">
      <div class="form-group">
        <label class="form-label">Source Name</label>
        <input
          class="input"
          type="text"
          placeholder="CLASP Bridge"
          :value="modelValue.sourceName ?? ''"
          @input="update('sourceName', ($event.target as HTMLInputElement).value)"
        />
      </div>
      <div class="form-group">
        <label class="form-label">Priority (0-200)</label>
        <input
          class="input"
          type="number"
          min="0"
          max="200"
          placeholder="100"
          :value="modelValue.priority ?? 100"
          @input="update('priority', Number(($event.target as HTMLInputElement).value))"
        />
      </div>
      <div class="form-group">
        <label class="form-label">
          <input
            type="checkbox"
            :checked="modelValue.multicast ?? true"
            @change="update('multicast', ($event.target as HTMLInputElement).checked)"
          />
          Multicast
        </label>
      </div>
      <div class="form-group">
        <label class="form-label">Bind Address</label>
        <input
          class="input"
          type="text"
          placeholder="0.0.0.0"
          :value="modelValue.bindAddress ?? ''"
          @input="update('bindAddress', ($event.target as HTMLInputElement).value)"
        />
      </div>
      <div class="form-group">
        <label class="form-label">Unicast Destinations (one per line)</label>
        <textarea
          class="input"
          rows="3"
          placeholder="192.168.1.10&#10;192.168.1.11"
          :value="destinationsToText(modelValue.unicastDestinations)"
          @input="update('unicastDestinations', textToDestinations(($event.target as HTMLTextAreaElement).value))"
        />
      </div>
      <div class="form-group">
        <label class="form-label">Namespace</label>
        <input
          class="input"
          type="text"
          placeholder="sacn"
          :value="modelValue.namespace ?? ''"
          @input="update('namespace', ($event.target as HTMLInputElement).value)"
        />
      </div>
      <div class="form-group">
        <label class="form-label">Sync Address</label>
        <input
          class="input"
          type="text"
          placeholder="239.255.0.1"
          :value="modelValue.syncAddress ?? ''"
          @input="update('syncAddress', ($event.target as HTMLInputElement).value)"
        />
      </div>
    </CollapsibleSection>
  </div>
</template>
