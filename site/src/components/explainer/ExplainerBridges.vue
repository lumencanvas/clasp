<script setup>
import { ref } from 'vue'
import { useScrollAnimation } from '../../composables/useScrollAnimation.js'

const sectionRef = ref(null)
useScrollAnimation(sectionRef)

const bridges = [
  {
    from: 'MIDI',
    to: 'CLASP',
    native: 'CC 7, Channel 1, Value 100',
    clasp: '/midi/launchpad/cc/1/7  →  SET 0.787',
    desc: 'CC, notes, pitchbend, program changes. 14-bit CC support. Values normalized 0.0-1.0.'
  },
  {
    from: 'OSC',
    to: 'CLASP',
    native: '/1/fader1  f:0.8',
    clasp: '/1/fader1  →  SET 0.8',
    desc: 'OSC paths preserved as-is. Bundle support. Type tags maintained in translation.'
  },
  {
    from: 'DMX',
    to: 'CLASP',
    native: 'Universe 1, Channel 42, Value 200',
    clasp: '/dmx/1/42  →  SET 200',
    desc: 'ENTTEC Pro and Open DMX USB. Art-Net universe mapping via /artnet/{universe}/{channel}.'
  },
  {
    from: 'MQTT',
    to: 'CLASP',
    native: 'Topic: sensors/temp  Payload: 22.5',
    clasp: '/mqtt/sensors/temp  →  SET 22.5',
    desc: 'v3.1.1 and v5. Topics map directly to CLASP addresses. QoS levels preserved.'
  }
]
</script>

<template>
  <section class="explainer-section bridges-section" ref="sectionRef">
    <div class="explainer-inner">
      <h2 class="fade-in">Protocol Bridges</h2>
      <p class="subtitle fade-in">
        Bidirectional translation. Every protocol keeps working as-is. CLASP bridges handle the conversion.
      </p>

      <div class="bridge-list">
        <div v-for="bridge in bridges" :key="bridge.from" class="bridge-row stagger">
          <div class="bridge-labels">
            <span class="bridge-from">{{ bridge.from }}</span>
            <span class="bridge-arrow">&harr;</span>
            <span class="bridge-to">{{ bridge.to }}</span>
          </div>
          <div class="bridge-translation">
            <div class="translation-side native">
              <span class="side-label">Native</span>
              <code>{{ bridge.native }}</code>
            </div>
            <div class="translation-arrow">&darr;</div>
            <div class="translation-side clasp">
              <span class="side-label">CLASP</span>
              <code>{{ bridge.clasp }}</code>
            </div>
          </div>
          <div class="bridge-desc">{{ bridge.desc }}</div>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.bridges-section {
  background: #fefcf6;
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

.bridge-list {
  max-width: 700px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
}

.bridge-row {
  border: 2px solid #1a1a1a;
  padding: 1.5rem;
  transition: box-shadow 0.2s ease;
}

.bridge-row:hover {
  box-shadow: 4px 4px 0 #1a1a1a;
}

.bridge-labels {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  margin-bottom: 1rem;
}

.bridge-from, .bridge-to {
  font-family: 'Archivo Black', sans-serif;
  text-transform: uppercase;
  font-size: 1rem;
  letter-spacing: 0.12em;
}

.bridge-from {
  color: var(--accent);
}

.bridge-arrow {
  font-size: 1.2rem;
  opacity: 0.5;
}

.bridge-translation {
  margin-bottom: 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.translation-side {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.side-label {
  font-family: 'Archivo Black', sans-serif;
  text-transform: uppercase;
  font-size: 0.65rem;
  letter-spacing: 0.1em;
  opacity: 0.45;
  min-width: 48px;
}

.translation-side code {
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.8rem;
  padding: 0.4rem 0.8rem;
  display: inline-block;
  flex: 1;
  word-break: break-all;
  min-width: 0;
}

.native code {
  background: rgba(0,0,0,0.06);
  color: rgba(0,0,0,0.6);
}

.clasp code {
  background: rgba(230, 57, 70, 0.08);
  color: #1a1a1a;
  font-weight: 500;
}

.translation-arrow {
  text-align: center;
  opacity: 0.3;
  font-size: 0.9rem;
  padding-left: 48px;
}

.bridge-desc {
  font-size: 0.85rem;
  opacity: 0.6;
  line-height: 1.5;
}

@media (max-width: 768px) {
  .translation-side {
    flex-direction: column;
    align-items: flex-start;
    gap: 0.25rem;
  }

  .translation-side code {
    font-size: 0.7rem;
    width: 100%;
  }

  .translation-arrow {
    padding-left: 0;
  }

  .bridge-row {
    padding: 1rem;
  }
}
</style>
