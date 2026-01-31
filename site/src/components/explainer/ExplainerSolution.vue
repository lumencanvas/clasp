<script setup>
import { ref, computed } from 'vue'
import { useScrollAnimation } from '../../composables/useScrollAnimation.js'

const sectionRef = ref(null)
useScrollAnimation(sectionRef)

const activeSource = ref(null)

const protocolColors = {
  OSC: '#c084fc',
  MIDI: '#60a5fa',
  'Art-Net': '#fbbf24',
  MQTT: '#4ade80',
  WebSocket: '#22d3ee',
  CLASP: '#f87171'
}

const scenarios = {
  touchosc: {
    source: 'TouchOSC',
    protocol: 'OSC',
    nativeMsg: '/1/fader1  0.8',
    bridge: 'CLASP-OSC Bridge',
    claspMsg: 'SET /mixer/ch1/volume 0.8',
    destinations: [
      { name: 'Ableton', protocol: 'MIDI', bridge: 'CLASP-MIDI Bridge', msg: 'CC 7, Ch 1, Val 102' },
      { name: 'Web UI', protocol: 'WebSocket', bridge: 'CLASP-WS Transport', msg: '{"addr":"/mixer/ch1/volume","v":0.8}' },
      { name: 'Logging', protocol: 'CLASP', bridge: 'Direct', msg: 'SET /mixer/ch1/volume 0.8' }
    ]
  },
  ableton: {
    source: 'Ableton',
    protocol: 'MIDI',
    nativeMsg: 'Note On, Ch 1, #60, Vel 127',
    bridge: 'CLASP-MIDI Bridge',
    claspMsg: 'EVENT /midi/keys/note/60 {vel:1.0}',
    destinations: [
      { name: 'Resolume', protocol: 'OSC', bridge: 'CLASP-OSC Bridge', msg: '/midi/keys/note/60  1.0' },
      { name: 'LED Strip', protocol: 'Art-Net', bridge: 'CLASP-Art-Net Bridge', msg: 'Univ 1, Ch 1-3, RGB flash' },
      { name: 'Web UI', protocol: 'WebSocket', bridge: 'CLASP-WS Transport', msg: '{"addr":"/midi/keys/note/60"}' }
    ]
  },
  sensors: {
    source: 'Sensors',
    protocol: 'MQTT',
    nativeMsg: 'sensors/temp → 22.5',
    bridge: 'CLASP-MQTT Bridge',
    claspMsg: 'STREAM /env/temp 22.5',
    destinations: [
      { name: 'IoT Hub', protocol: 'MQTT', bridge: 'CLASP-MQTT Bridge', msg: 'clasp/env/temp → 22.5' },
      { name: 'Web UI', protocol: 'WebSocket', bridge: 'CLASP-WS Transport', msg: '{"addr":"/env/temp","v":22.5}' },
      { name: 'Logging', protocol: 'CLASP', bridge: 'Direct', msg: 'STREAM /env/temp 22.5' }
    ]
  },
  ledstrip: {
    source: 'LED Strip',
    protocol: 'Art-Net',
    nativeMsg: 'Univ 1, Ch 42, Val 200',
    bridge: 'CLASP-Art-Net Bridge',
    claspMsg: 'SET /artnet/1/42 200',
    destinations: [
      { name: 'Resolume', protocol: 'OSC', bridge: 'CLASP-OSC Bridge', msg: '/artnet/1/42  200' },
      { name: 'Web UI', protocol: 'WebSocket', bridge: 'CLASP-WS Transport', msg: '{"addr":"/artnet/1/42","v":200}' },
      { name: 'Logging', protocol: 'CLASP', bridge: 'Direct', msg: 'SET /artnet/1/42 200' }
    ]
  }
}

const sources = [
  { id: 'touchosc', name: 'TouchOSC', protocol: 'OSC' },
  { id: 'ableton', name: 'Ableton', protocol: 'MIDI' },
  { id: 'ledstrip', name: 'LED Strip', protocol: 'Art-Net' },
  { id: 'sensors', name: 'Sensors', protocol: 'MQTT' }
]

const active = computed(() => activeSource.value ? scenarios[activeSource.value] : null)

function toggle(id) {
  activeSource.value = activeSource.value === id ? null : id
}

function pColor(proto) {
  return protocolColors[proto] || '#fff'
}
</script>

<template>
  <section class="explainer-section solution-section" ref="sectionRef">
    <div class="explainer-inner">
      <h2 class="fade-in">One Router to Connect Them All</h2>
      <p class="subtitle fade-in">Tap a device to trace its message through CLASP.</p>

      <div class="graph fade-in">
        <!-- Sources -->
        <div class="source-row">
          <button
            v-for="src in sources"
            :key="src.id"
            class="src-node"
            :class="{ active: activeSource === src.id }"
            :style="{ borderColor: pColor(src.protocol) }"
            @click="toggle(src.id)"
          >
            <span class="src-name">{{ src.name }}</span>
            <span class="src-proto">{{ src.protocol }}</span>
          </button>
        </div>

        <!-- Flow: only when active -->
        <template v-if="active">
          <div class="flow-line" :style="{ background: pColor(active.protocol) }"></div>

          <!-- Native message -->
          <div class="msg-pill native" :style="{ borderLeftColor: pColor(active.protocol) }">
            <span class="msg-label">{{ active.protocol }}</span>
            <code>{{ active.nativeMsg }}</code>
          </div>

          <div class="flow-line short" :style="{ background: pColor(active.protocol) }"></div>

          <!-- Inbound bridge -->
          <div class="bridge-pill">{{ active.bridge }}</div>

          <div class="flow-line short"></div>

          <!-- Router -->
          <div class="router-node">
            <svg class="router-icon" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"><circle cx="12" cy="12" r="3"/><path d="M12 2v4m0 12v4M2 12h4m12 0h4M4.93 4.93l2.83 2.83m8.48 8.48l2.83 2.83M4.93 19.07l2.83-2.83m8.48-8.48l2.83-2.83"/></svg>
            <span>CLASP Router</span>
          </div>

          <!-- CLASP message -->
          <div class="msg-pill clasp">
            <span class="msg-label">CLASP</span>
            <code>{{ active.claspMsg }}</code>
          </div>

          <div class="flow-line short"></div>
          <div class="fanout-label">{{ active.destinations.length }} subscribers</div>

          <!-- Destinations -->
          <div class="dest-row">
            <div
              v-for="dest in active.destinations"
              :key="dest.name"
              class="dest-card"
              :style="{ borderColor: pColor(dest.protocol) }"
            >
              <div class="dest-bridge">{{ dest.bridge }}</div>
              <div class="dest-info">
                <span class="dest-name">{{ dest.name }}</span>
                <span class="dest-proto">{{ dest.protocol }}</span>
              </div>
              <code class="dest-msg" :style="{ borderLeftColor: pColor(dest.protocol) }">{{ dest.msg }}</code>
            </div>
          </div>
        </template>

        <!-- Idle state -->
        <template v-else>
          <div class="flow-line dim"></div>
          <div class="router-node dim">
            <svg class="router-icon" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"><circle cx="12" cy="12" r="3"/><path d="M12 2v4m0 12v4M2 12h4m12 0h4M4.93 4.93l2.83 2.83m8.48 8.48l2.83 2.83M4.93 19.07l2.83-2.83m8.48-8.48l2.83-2.83"/></svg>
            <span>CLASP Router</span>
          </div>
          <div class="flow-line dim"></div>
          <div class="idle-row">
            <div class="idle-node" v-for="n in ['Web UI', 'IoT Hub', 'Resolume', 'Logging']" :key="n">{{ n }}</div>
          </div>
        </template>
      </div>

      <p class="note fade-in">N tools, N bridges &mdash; not N&sup2; translations.</p>
    </div>
  </section>
</template>

<style scoped>
.solution-section { background: #1a1a1a; color: #fff; }

h2 { font-size: clamp(1.6rem, 4vw, 2.8rem); text-align: center; margin-bottom: 0.5rem; color: #fff; }
.subtitle { text-align: center; margin: 0 auto 2rem; font-size: 0.95rem; opacity: 0.6; }

.graph {
  display: flex;
  flex-direction: column;
  align-items: center;
  max-width: 680px;
  margin: 0 auto;
}

/* Source row */
.source-row {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 0.5rem;
  width: 100%;
}

.src-node {
  background: transparent;
  border: 1.5px solid;
  color: #fff;
  font-family: 'Space Mono', monospace;
  cursor: pointer;
  padding: 0.5rem 1rem;
  transition: all 0.12s ease;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.15rem;
  flex: 1;
  min-width: 90px;
  max-width: 160px;
}

.src-node:hover { background: rgba(255,255,255,0.05); }
.src-node.active { background: rgba(255,255,255,0.1); border-width: 2px; }
.src-name { font-size: 0.75rem; font-weight: 700; letter-spacing: 0.04em; }
.src-proto { font-size: 0.55rem; text-transform: uppercase; letter-spacing: 0.1em; opacity: 0.5; }

/* Flow lines */
.flow-line { width: 1.5px; height: 20px; background: rgba(255,255,255,0.25); }
.flow-line.short { height: 12px; }
.flow-line.dim { height: 16px; background: rgba(255,255,255,0.08); }

/* Message pills */
.msg-pill {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.35rem 0.75rem;
  background: rgba(255,255,255,0.04);
  border-left: 3px solid;
  max-width: 100%;
}

.msg-label {
  font-size: 0.5rem;
  text-transform: uppercase;
  letter-spacing: 0.12em;
  font-weight: 700;
  white-space: nowrap;
  opacity: 0.45;
}

.msg-pill code {
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.7rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  opacity: 0.85;
}

.msg-pill.clasp {
  border-left-color: var(--accent, #e63946);
}

/* Bridge pills */
.bridge-pill {
  font-family: 'Space Mono', monospace;
  font-size: 0.6rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  padding: 0.25rem 0.7rem;
  background: rgba(255,255,255,0.04);
  border: 1px solid rgba(255,255,255,0.15);
  border-radius: 2px;
  opacity: 0.6;
}

/* Router */
.router-node {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1.25rem;
  background: rgba(230, 57, 70, 0.12);
  border: 2px solid var(--accent, #e63946);
}

.router-node.dim { border-color: rgba(255,255,255,0.1); background: rgba(255,255,255,0.03); opacity: 0.4; }

.router-icon { opacity: 0.7; flex-shrink: 0; }

.router-node span {
  font-family: 'Archivo Black', sans-serif;
  text-transform: uppercase;
  font-size: 0.7rem;
  letter-spacing: 0.12em;
}

/* Fanout */
.fanout-label {
  font-size: 0.55rem;
  text-transform: uppercase;
  letter-spacing: 0.12em;
  opacity: 0.35;
  padding: 0.15rem 0;
}

/* Destinations */
.dest-row {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  width: 100%;
  justify-content: center;
}

.dest-card {
  flex: 1;
  min-width: 150px;
  max-width: 220px;
  background: rgba(255,255,255,0.03);
  border: 1.5px solid;
  padding: 0.6rem;
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
  animation: fadeUp 0.25s ease;
}

.dest-bridge {
  font-size: 0.5rem;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  opacity: 0.35;
}

.dest-info { display: flex; align-items: baseline; gap: 0.4rem; }
.dest-name { font-size: 0.75rem; font-weight: 700; }
.dest-proto { font-size: 0.5rem; text-transform: uppercase; letter-spacing: 0.1em; opacity: 0.4; }

.dest-msg {
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.6rem;
  background: rgba(0,0,0,0.3);
  padding: 0.25rem 0.4rem;
  border-left: 2px solid;
  line-height: 1.3;
  word-break: break-word;
  opacity: 0.75;
}

/* Idle */
.idle-row {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 0.4rem;
}

.idle-node {
  border: 1px solid rgba(255,255,255,0.1);
  padding: 0.4rem 0.8rem;
  font-size: 0.7rem;
  opacity: 0.3;
  letter-spacing: 0.04em;
}

.note { text-align: center; opacity: 0.4; font-size: 0.85rem; margin-top: 1.5rem; }

/* Mobile */
@media (max-width: 480px) {
  .src-node { padding: 0.35rem 0.5rem; min-width: 75px; }
  .src-name { font-size: 0.65rem; }
  .msg-pill code { font-size: 0.6rem; }
  .dest-card { min-width: 130px; max-width: none; }
  .dest-msg { font-size: 0.55rem; }
}
</style>
