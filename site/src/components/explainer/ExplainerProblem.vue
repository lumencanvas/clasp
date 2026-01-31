<script setup>
import { ref, onMounted } from 'vue'
import { useScrollAnimation } from '../../composables/useScrollAnimation.js'

const sectionRef = ref(null)
useScrollAnimation(sectionRef)

const protocols = [
  { name: 'MIDI', year: 1983 },
  { name: 'DMX', year: 1986 },
  { name: 'Art-Net', year: 1998 },
  { name: 'MQTT', year: 1999 },
  { name: 'OSC', year: 2002 },
  { name: 'sACN', year: 2009 },
  { name: 'Socket.IO', year: 2010 },
  { name: 'WebSocket', year: 2011 }
]

function randomRotation() {
  return (Math.random() - 0.5) * 8
}
</script>

<template>
  <section class="explainer-section problem-section" id="problem" ref="sectionRef">
    <div class="explainer-inner">
      <h2 class="fade-in">Creative Tools Speak Different Languages</h2>
      <p class="subtitle fade-in">
        Every protocol was designed for a single domain. Connecting them means writing glue code for every pair.
      </p>
      <div class="protocol-grid">
        <div
          v-for="proto in protocols"
          :key="proto.name"
          class="protocol-box stagger"
          @mouseenter="$event.target.style.transform = `rotate(${randomRotation()}deg) scale(1.05)`"
          @mouseleave="$event.target.style.transform = ''"
        >
          <span class="proto-name">{{ proto.name }}</span>
          <span class="proto-year">{{ proto.year }}</span>
        </div>
      </div>
      <p class="problem-punchline fade-in">
        That's <strong>{{ protocols.length * (protocols.length - 1) / 2 }}</strong> possible translation pairs.
      </p>
    </div>
  </section>
</template>

<style scoped>
.problem-section {
  background: #fefcf6;
  color: #1a1a1a;
}

h2 {
  font-size: clamp(1.8rem, 4vw, 3rem);
  margin-bottom: 1rem;
  text-align: center;
}

.subtitle {
  text-align: center;
  max-width: 600px;
  margin: 0 auto 3rem;
  font-size: 1.1rem;
  line-height: 1.6;
  opacity: 0.8;
}

.protocol-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
  gap: 1rem;
  max-width: 700px;
  margin: 0 auto 2rem;
}

.protocol-box {
  border: 2px solid #1a1a1a;
  padding: 1.2rem 1rem;
  text-align: center;
  transition: transform 0.08s ease, box-shadow 0.08s ease;
  cursor: default;
  background: #fff;
}

.protocol-box:hover {
  box-shadow: 4px 4px 0 #1a1a1a;
}

.proto-name {
  display: block;
  font-family: 'Archivo Black', sans-serif;
  text-transform: uppercase;
  font-size: 1rem;
  letter-spacing: 0.1em;
}

.proto-year {
  display: block;
  font-size: 0.75rem;
  opacity: 0.5;
  margin-top: 0.3rem;
  font-family: 'JetBrains Mono', monospace;
}

.problem-punchline {
  text-align: center;
  font-size: 1.25rem;
  margin-top: 2rem;
}

.problem-punchline strong {
  color: var(--accent);
  font-size: 1.5rem;
}
</style>
