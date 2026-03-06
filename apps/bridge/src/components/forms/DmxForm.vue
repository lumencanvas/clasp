<script setup lang="ts">
import { ref, onMounted } from 'vue'
import type { Connection } from '../../lib/types'
import { useDevices } from '../../composables/useDevices'
import CollapsibleSection from '../shared/CollapsibleSection.vue'

const props = defineProps<{ modelValue: Connection }>()
const emit = defineEmits<{ 'update:modelValue': [value: Connection] }>()

const { listSerialPorts } = useDevices()
const serialPorts = ref<Array<{ path: string; name: string }>>([])

function update(field: string, value: any) {
  emit('update:modelValue', { ...props.modelValue, [field]: value })
}

onMounted(async () => {
  try {
    const ports = await listSerialPorts()
    if (ports) {
      serialPorts.value = ports
    }
  } catch {
    // Serial ports unavailable
  }
})
</script>

<template>
  <div class="protocol-form">
    <div class="form-group">
      <label class="form-label">Serial Port</label>
      <select
        class="select"
        :value="modelValue.serialPort ?? ''"
        @change="update('serialPort', ($event.target as HTMLSelectElement).value)"
      >
        <option value="">-- Select Port --</option>
        <option v-for="port in serialPorts" :key="port.path" :value="port.path">
          {{ port.name }} ({{ port.path }})
        </option>
      </select>
    </div>

    <CollapsibleSection title="ADVANCED">
      <div class="form-group">
        <label class="form-label">Namespace</label>
        <input
          class="input"
          type="text"
          placeholder="dmx"
          :value="modelValue.namespace ?? ''"
          @input="update('namespace', ($event.target as HTMLInputElement).value)"
        />
      </div>
      <div class="form-group">
        <label class="form-label">Direction</label>
        <select
          class="select"
          :value="modelValue.direction ?? 'output'"
          @change="update('direction', ($event.target as HTMLSelectElement).value)"
        >
          <option value="input">Input</option>
          <option value="output">Output</option>
          <option value="both">Both</option>
        </select>
      </div>
      <div class="form-group">
        <label class="form-label">Channels</label>
        <input
          class="input"
          type="number"
          min="1"
          max="512"
          placeholder="512"
          :value="modelValue.channels ?? 512"
          @input="update('channels', Number(($event.target as HTMLInputElement).value))"
        />
      </div>
      <div class="form-group">
        <label class="form-label">Baud Rate</label>
        <input
          class="input"
          type="number"
          placeholder="250000"
          :value="modelValue.baudRate ?? 250000"
          @input="update('baudRate', Number(($event.target as HTMLInputElement).value))"
        />
      </div>
      <div class="form-group">
        <label class="form-label">Refresh Rate (Hz)</label>
        <input
          class="input"
          type="number"
          min="1"
          max="44"
          placeholder="40"
          :value="modelValue.refreshRate ?? 40"
          @input="update('refreshRate', Number(($event.target as HTMLInputElement).value))"
        />
      </div>
    </CollapsibleSection>
  </div>
</template>
