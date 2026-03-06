<script setup lang="ts">
import { ref, onMounted } from 'vue'
import type { Connection } from '../../lib/types'
import { useDevices } from '../../composables/useDevices'
import CollapsibleSection from '../shared/CollapsibleSection.vue'

const props = defineProps<{ modelValue: Connection }>()
const emit = defineEmits<{ 'update:modelValue': [value: Connection] }>()

const { listMidiPorts } = useDevices()
const midiInputs = ref<Array<{ id: string; name: string }>>([])
const midiOutputs = ref<Array<{ id: string; name: string }>>([])

function update(field: string, value: any) {
  emit('update:modelValue', { ...props.modelValue, [field]: value })
}

onMounted(async () => {
  try {
    const ports = await listMidiPorts()
    if (ports) {
      midiInputs.value = ports.inputs
      midiOutputs.value = ports.outputs
    }
  } catch {
    // MIDI ports unavailable
  }
})
</script>

<template>
  <div class="protocol-form">
    <div class="form-group">
      <label class="form-label">Input Port</label>
      <select
        class="select"
        :value="modelValue.inputPort ?? ''"
        @change="update('inputPort', ($event.target as HTMLSelectElement).value)"
      >
        <option value="">-- Select Input --</option>
        <option v-for="port in midiInputs" :key="port.id" :value="port.id">
          {{ port.name }}
        </option>
      </select>
    </div>
    <div class="form-group">
      <label class="form-label">Output Port</label>
      <select
        class="select"
        :value="modelValue.outputPort ?? ''"
        @change="update('outputPort', ($event.target as HTMLSelectElement).value)"
      >
        <option value="">-- Select Output --</option>
        <option v-for="port in midiOutputs" :key="port.id" :value="port.id">
          {{ port.name }}
        </option>
      </select>
    </div>

    <CollapsibleSection title="ADVANCED">
      <div class="form-group">
        <label class="form-label">Namespace</label>
        <input
          class="input"
          type="text"
          placeholder="midi"
          :value="modelValue.namespace ?? ''"
          @input="update('namespace', ($event.target as HTMLInputElement).value)"
        />
      </div>
      <div class="form-group">
        <label class="form-label">Device Name</label>
        <input
          class="input"
          type="text"
          placeholder="Auto-detect"
          :value="modelValue.deviceName ?? ''"
          @input="update('deviceName', ($event.target as HTMLInputElement).value)"
        />
      </div>
    </CollapsibleSection>
  </div>
</template>
