<script setup>
import { ref } from 'vue'
import { useScrollAnimation } from '../../composables/useScrollAnimation.js'

const sectionRef = ref(null)
useScrollAnimation(sectionRef)

const steps = [
  {
    num: '01',
    title: 'Start a Router',
    desc: 'Run the CLASP router on any machine. It listens on port 7330 and handles all message routing, state storage, and clock synchronization.'
  },
  {
    num: '02',
    title: 'Connect Bridges',
    desc: 'Bridges translate between protocols. MIDI controller? OSC app? DMX lights? Each bridge speaks CLASP on one side and the native protocol on the other.'
  },
  {
    num: '03',
    title: 'Messages Translate',
    desc: 'A MIDI CC becomes /midi/ch1/cc74. An OSC message maps to /osc/fader1. Every protocol gets a clean address in CLASP\'s unified namespace.'
  },
  {
    num: '04',
    title: 'Use Wildcards',
    desc: 'Subscribe to /lights/*/brightness to hear all lights. Use /sensors/** for everything under sensors. Pattern matching makes routing flexible without configuration.'
  }
]
</script>

<template>
  <section class="explainer-section howitworks-section" id="howitworks" ref="sectionRef">
    <div class="explainer-inner">
      <h2 class="fade-in">How It Works</h2>
      <div class="steps">
        <div v-for="step in steps" :key="step.num" class="step stagger">
          <div class="step-num">{{ step.num }}</div>
          <div class="step-body">
            <h3>{{ step.title }}</h3>
            <p>{{ step.desc }}</p>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.howitworks-section {
  background: #fefcf6;
  color: #1a1a1a;
}

h2 {
  font-size: clamp(1.8rem, 4vw, 3rem);
  text-align: center;
  margin-bottom: 3rem;
}

.steps {
  max-width: 700px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 2rem;
}

.step {
  display: flex;
  gap: 1.5rem;
  align-items: flex-start;
}

.step-num {
  font-family: 'Archivo Black', sans-serif;
  font-size: 2.5rem;
  color: var(--accent);
  line-height: 1;
  flex-shrink: 0;
  width: 60px;
}

.step-body h3 {
  font-size: 1.1rem;
  letter-spacing: 0.12em;
  margin: 0 0 0.5rem;
}

.step-body p {
  line-height: 1.7;
  opacity: 0.8;
  margin: 0;
}

@media (max-width: 768px) {
  .step {
    flex-direction: column;
    gap: 0.5rem;
  }
}
</style>
