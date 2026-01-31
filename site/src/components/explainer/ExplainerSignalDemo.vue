<script setup>
import { ref, computed } from 'vue'
import { useScrollAnimation } from '../../composables/useScrollAnimation.js'

const sectionRef = ref(null)
useScrollAnimation(sectionRef)

const activeTab = ref('param')

const signalData = {
  param: {
    label: 'Param',
    desc: 'Stateful. The router remembers the last value. Late joiners get current state automatically.',
    json: `{
  "type": "SET",
  "address": "/lights/kitchen/brightness",
  "value": 0.75,
  "revision": 42
}`,
    use: 'Faders, color pickers, toggles, any persistent control value.'
  },
  event: {
    label: 'Event',
    desc: 'Fire-and-forget. No state stored. If nobody is listening, the message is lost.',
    json: `{
  "type": "EVENT",
  "address": "/cue/fire",
  "value": { "cueId": "intro", "fadeTime": 2.0 }
}`,
    use: 'Button presses, cue triggers, one-shot actions.'
  },
  stream: {
    label: 'Stream',
    desc: 'High-rate data sent as fast as possible. No acknowledgment, no state. Optimized for throughput.',
    json: `{
  "type": "STREAM",
  "address": "/sensors/accelerometer/x",
  "value": 0.342
}`,
    use: 'Sensor data, audio levels, motion tracking at 60Hz+.'
  },
  gesture: {
    label: 'Gesture',
    desc: 'Time-series with lifecycle: begin, update, end. The router coalesces rapid updates (95-98% bandwidth savings).',
    json: `{
  "type": "GESTURE",
  "address": "/touch/1",
  "value": { "phase": "update", "x": 0.5, "y": 0.3 },
  "gestureId": "abc123"
}`,
    use: 'Touch drags, fader sweeps, pen strokes.'
  },
  timeline: {
    label: 'Timeline',
    desc: 'Scheduled sequences with transport controls (play, pause, seek). Clock-synced across all clients. Bundle multiple cues to fire at precise times.',
    json: `{
  "type": "BUNDLE",
  "at": 1706745600000000,
  "messages": [
    { "type": "SET", "address": "/lights/front/color", "value": "#ff0044" },
    { "type": "SET", "address": "/lights/back/intensity", "value": 1.0 },
    { "type": "EVENT", "address": "/audio/cue/start", "value": "act2-intro" },
    { "type": "SET", "address": "/video/layer1/opacity", "value": 0.0 }
  ]
}`,
    use: 'Show cue lists, synchronized lighting + audio + video, timed animation sequences.'
  }
}

const current = computed(() => signalData[activeTab.value])
</script>

<template>
  <section class="explainer-section demo-section" ref="sectionRef">
    <div class="explainer-inner">
      <h2 class="fade-in">See the Signals</h2>

      <div class="demo-tabs fade-in">
        <button
          v-for="(data, key) in signalData"
          :key="key"
          class="demo-tab"
          :class="{ active: activeTab === key }"
          @click="activeTab = key"
        >
          {{ data.label }}
        </button>
      </div>

      <div class="demo-content fade-in">
        <div class="demo-explain">
          <h3>{{ current.label }}</h3>
          <p>{{ current.desc }}</p>
          <p class="use-case"><strong>Use cases:</strong> {{ current.use }}</p>
        </div>
        <div class="demo-code">
          <pre><code>{{ current.json }}</code></pre>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.demo-section {
  background: #1a1a1a;
  color: #fff;
}

h2 {
  font-size: clamp(1.8rem, 4vw, 3rem);
  text-align: center;
  margin-bottom: 2rem;
  color: #fff;
}

.demo-tabs {
  display: flex;
  justify-content: center;
  gap: 0.5rem;
  margin-bottom: 2rem;
  flex-wrap: wrap;
}

.demo-tab {
  background: transparent;
  border: 1px solid rgba(255,255,255,0.3);
  color: rgba(255,255,255,0.7);
  padding: 0.6rem 1.2rem;
  font-family: 'Space Mono', monospace;
  font-size: 0.85rem;
  letter-spacing: 0.1em;
  cursor: pointer;
  transition: all 0.2s ease;
}

.demo-tab:hover {
  border-color: rgba(255,255,255,0.6);
  color: #fff;
}

.demo-tab.active {
  background: var(--accent);
  border-color: var(--accent);
  color: #fff;
}

.demo-content {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 2rem;
  max-width: 900px;
  margin: 0 auto;
}

.demo-explain h3 {
  font-size: 1.2rem;
  letter-spacing: 0.15em;
  margin-bottom: 0.75rem;
  color: var(--accent);
}

.demo-explain p {
  line-height: 1.7;
  opacity: 0.85;
  margin-bottom: 1rem;
}

.use-case {
  font-size: 0.9rem;
  opacity: 0.7 !important;
}

.use-case strong {
  color: var(--accent);
}

.demo-code {
  min-width: 0;
}

.demo-code pre {
  background: rgba(255,255,255,0.05);
  border: 1px solid rgba(255,255,255,0.1);
  padding: 1.25rem;
  border-radius: 0;
  overflow-x: auto;
  margin: 0;
}

.demo-code code {
  color: rgba(255,255,255,0.9);
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.8rem;
  line-height: 1.6;
  word-break: break-all;
  white-space: pre-wrap;
}

@media (max-width: 768px) {
  .demo-content {
    grid-template-columns: 1fr;
  }

  .demo-code code {
    font-size: 0.7rem;
  }

  .demo-explain p {
    font-size: 0.9rem;
  }
}
</style>
