<script setup>
import { ref } from 'vue'
import { useScrollAnimation } from '../../composables/useScrollAnimation.js'

const sectionRef = ref(null)
useScrollAnimation(sectionRef)

const signals = [
  {
    name: 'Param',
    color: '#e63946',
    desc: 'Stateful values that persist and sync. Think: fader position, color, volume.',
    code: [
      'clasp.set(',
      '  "/lights/kitchen/brightness",',
      '  0.75',
      ')'
    ]
  },
  {
    name: 'Event',
    color: '#457b9d',
    desc: 'One-shot fire-and-forget messages. Think: button press, cue trigger.',
    code: [
      'clasp.emit("/cue/fire", {',
      '  cueId: "intro",',
      '  fadeTime: 2.0',
      '})'
    ]
  },
  {
    name: 'Stream',
    color: '#2a9d8f',
    desc: 'High-rate data with no state. Think: accelerometer, audio levels.',
    code: [
      'clasp.stream(',
      '  "/sensors/accel/x",',
      '  0.342',
      ')'
    ]
  },
  {
    name: 'Gesture',
    color: '#f77f00',
    desc: 'Time-series with begin/update/end lifecycle. Think: touch drag, fader sweep.',
    code: [
      'clasp.gesture("/touch/1", {',
      '  phase: "update",',
      '  x: 0.5,',
      '  y: 0.3',
      '})'
    ]
  },
  {
    name: 'Timeline',
    color: '#6b4c9a',
    desc: 'Scheduled bundles with transport controls. Coordinate lights, audio, and video in sync.',
    code: [
      'clasp.bundle([',
      '  { set: ["/lights/front", "#ff0044"] },',
      '  { set: ["/audio/volume", 1.0] },',
      '  { emit: ["/cue/start", "act2"] }',
      '], { at: clasp.time() + 100000 })'
    ]
  }
]
</script>

<template>
  <section class="explainer-section signal-section" id="signals" ref="sectionRef">
    <div class="explainer-inner">
      <h2 class="fade-in">Five Signal Types, One Address Space</h2>
      <p class="subtitle fade-in">
        Every message type creative tools need, unified under a single path-based addressing system.
      </p>
      <div class="signal-grid">
        <div
          v-for="signal in signals"
          :key="signal.name"
          class="signal-card stagger"
          :style="{ borderLeftColor: signal.color }"
        >
          <h3 :style="{ color: signal.color }">{{ signal.name }}</h3>
          <p>{{ signal.desc }}</p>
          <pre class="signal-code"><code>{{ signal.code.join('\n') }}</code></pre>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.signal-section {
  background: #f4f1e8;
  color: #1a1a1a;
}

h2 {
  font-size: clamp(1.8rem, 4vw, 3rem);
  text-align: center;
  margin-bottom: 1rem;
}

.subtitle {
  text-align: center;
  max-width: 600px;
  margin: 0 auto 3rem;
  font-size: 1.1rem;
  line-height: 1.6;
  opacity: 0.8;
}

.signal-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 1.25rem;
}

.signal-card {
  border-left: 4px solid;
  padding: 1.5rem;
  background: rgba(255,255,255,0.6);
  transition: transform 0.2s ease, box-shadow 0.2s ease;
  display: flex;
  flex-direction: column;
}

.signal-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0,0,0,0.08);
}

.signal-card h3 {
  font-size: 1rem;
  letter-spacing: 0.15em;
  margin: 0 0 0.5rem;
}

.signal-card p {
  font-size: 0.9rem;
  line-height: 1.5;
  margin: 0 0 1rem;
  opacity: 0.8;
  flex: 1;
}

.signal-code {
  margin: 0;
  padding: 0.75rem 1rem;
  background: rgba(0,0,0,0.05);
  border-radius: 3px;
  overflow-x: auto;
}

.signal-code code {
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.75rem;
  line-height: 1.6;
  tab-size: 2;
}
</style>
