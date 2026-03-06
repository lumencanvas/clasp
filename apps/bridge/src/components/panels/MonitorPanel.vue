<script setup lang="ts">
import { computed, ref } from 'vue'
import { useMonitor } from '../../composables/useMonitor'
import SignalRow from '../cards/SignalRow.vue'

const { signals, signalRate, clear } = useMonitor()

const filterText = ref('')
const filterProtocol = ref('')

const filteredSignals = computed(() => {
  let list = signals.value
  if (filterProtocol.value) {
    list = list.filter(s => s.protocol === filterProtocol.value)
  }
  if (filterText.value) {
    const q = filterText.value.toLowerCase()
    list = list.filter(s => (s.path || s.address || '').toLowerCase().includes(q))
  }
  return list
})

const hasSignals = computed(() => filteredSignals.value.length > 0)
</script>

<template>
  <div style="display: flex; flex-direction: column; height: 100%;">
    <div class="panel-toolbar">
      <span class="panel-title">SIGNAL MONITOR</span>
      <div class="toolbar-group">
        <input
          v-model="filterText"
          class="input input-sm"
          placeholder="Filter path..."
          style="width: 180px;"
        />
        <select v-model="filterProtocol" class="select select-sm">
          <option value="">All protocols</option>
          <option value="osc">OSC</option>
          <option value="midi">MIDI</option>
          <option value="artnet">Art-Net</option>
          <option value="sacn">sACN</option>
          <option value="dmx">DMX</option>
          <option value="mqtt">MQTT</option>
          <option value="websocket">WebSocket</option>
          <option value="http">HTTP</option>
        </select>
        <span class="stat-value" style="font-size: 11px;">{{ signalRate }}/s</span>
        <button class="btn btn-sm btn-secondary" @click="clear">CLEAR</button>
      </div>
    </div>
    <div class="panel-content panel-monitor-content">
      <div class="signal-monitor">
        <template v-if="hasSignals">
          <SignalRow v-for="(signal, i) in filteredSignals" :key="i" :signal="signal" />
        </template>
        <div v-else class="signal-empty">
          Waiting for signals...
        </div>
      </div>
    </div>
  </div>
</template>
