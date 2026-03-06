<script setup lang="ts">
import type { SignalEndpoint, AnyProtocol } from '../../lib/types'
import { allProtocols, protocolNames } from '../../lib/constants'

const props = defineProps<{
  modelValue: SignalEndpoint
  label?: string
}>()
const emit = defineEmits<{ 'update:modelValue': [value: SignalEndpoint] }>()

function update(field: string, value: any) {
  emit('update:modelValue', { ...props.modelValue, [field]: value })
}

function updateProtocol(protocol: string) {
  emit('update:modelValue', { protocol: protocol as AnyProtocol })
}

const midiTypes = [
  { value: 'noteon', label: 'Note On' },
  { value: 'noteoff', label: 'Note Off' },
  { value: 'cc', label: 'Control Change' },
  { value: 'program', label: 'Program Change' },
  { value: 'pitchbend', label: 'Pitch Bend' },
]
</script>

<template>
  <div class="endpoint-picker">
    <span v-if="label" class="form-label">{{ label }}</span>

    <div class="form-group">
      <label class="form-label">Protocol</label>
      <select
        class="select"
        :value="modelValue.protocol"
        @change="updateProtocol(($event.target as HTMLSelectElement).value)"
      >
        <option value="clasp">{{ protocolNames.clasp }}</option>
        <option
          v-for="p in allProtocols"
          :key="p"
          :value="p"
        >
          {{ protocolNames[p] }}
        </option>
      </select>
    </div>

    <!-- Address field (most protocols) -->
    <div class="form-group">
      <label class="form-label">Address / Path</label>
      <input
        class="input"
        type="text"
        placeholder="/address/path"
        :value="modelValue.address ?? ''"
        @input="update('address', ($event.target as HTMLInputElement).value)"
      />
    </div>

    <!-- MIDI-specific fields -->
    <template v-if="modelValue.protocol === 'midi'">
      <div class="form-group">
        <label class="form-label">MIDI Type</label>
        <select
          class="select"
          :value="modelValue.midiType ?? 'cc'"
          @change="update('midiType', ($event.target as HTMLSelectElement).value)"
        >
          <option
            v-for="mt in midiTypes"
            :key="mt.value"
            :value="mt.value"
          >
            {{ mt.label }}
          </option>
        </select>
      </div>
      <div class="form-group">
        <label class="form-label">MIDI Channel (1-16)</label>
        <input
          class="input"
          type="number"
          min="1"
          max="16"
          placeholder="1"
          :value="modelValue.midiChannel ?? 1"
          @input="update('midiChannel', Number(($event.target as HTMLInputElement).value) || null)"
        />
      </div>
      <div class="form-group">
        <label class="form-label">MIDI Number (0-127)</label>
        <input
          class="input"
          type="number"
          min="0"
          max="127"
          placeholder="0"
          :value="modelValue.midiNumber ?? 0"
          @input="update('midiNumber', Number(($event.target as HTMLInputElement).value) ?? null)"
        />
      </div>
    </template>

    <!-- DMX / Art-Net specific fields -->
    <template v-if="modelValue.protocol === 'dmx' || modelValue.protocol === 'artnet'">
      <div class="form-group">
        <label class="form-label">Universe</label>
        <input
          class="input"
          type="number"
          min="0"
          placeholder="0"
          :value="modelValue.dmxUniverse ?? 0"
          @input="update('dmxUniverse', Number(($event.target as HTMLInputElement).value) ?? null)"
        />
      </div>
      <div class="form-group">
        <label class="form-label">Channel (1-512)</label>
        <input
          class="input"
          type="number"
          min="1"
          max="512"
          placeholder="1"
          :value="modelValue.dmxChannel ?? 1"
          @input="update('dmxChannel', Number(($event.target as HTMLInputElement).value) ?? null)"
        />
      </div>
    </template>
  </div>
</template>
