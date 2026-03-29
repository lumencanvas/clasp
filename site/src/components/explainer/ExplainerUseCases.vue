<script setup>
import { ref } from 'vue'
import { useScrollAnimation } from '../../composables/useScrollAnimation.js'

const sectionRef = ref(null)
useScrollAnimation(sectionRef)

const cases = [
  {
    title: 'Live Performance',
    desc: 'Lighting rigs, VJ software, MIDI controllers, projection mapping. The domain CLASP was built for.',
    protocols: ['OSC', 'MIDI', 'DMX', 'Art-Net']
  },
  {
    title: 'Smart Buildings',
    desc: 'HVAC, access control, lighting, occupancy. One router per building. Rules replace BMS programming. Federation across campus.',
    protocols: ['MQTT', 'HTTP', 'Serial', 'WebSocket']
  },
  {
    title: 'Industrial / SCADA',
    desc: 'Factory floor sensors, PLCs, actuators. Rules engine replaces ladder logic. Journal provides audit trails. 3KB client on sensor nodes.',
    protocols: ['MQTT', 'Serial', 'HTTP', 'WebSocket']
  },
  {
    title: 'IoT Sensor Networks',
    desc: 'BLE wearables, MQTT gateways, HTTP endpoints. Late-joiner sync means dashboards start with current state, not blank screens.',
    protocols: ['MQTT', 'BLE', 'HTTP', 'Serial']
  },
  {
    title: 'Digital Twins',
    desc: 'Bidirectional sync between physical sensors and virtual models. Rules detect divergence. DefraDB persistence for model retraining.',
    protocols: ['MQTT', 'HTTP', 'WebSocket']
  },
  {
    title: 'Collaborative Apps',
    desc: 'CLASP Chat proves the pattern: real-time collaboration on a generic relay. Rooms, presence, E2E encryption. Zero chat-specific server code.',
    protocols: ['WebSocket', 'WebRTC']
  },
  {
    title: 'Robotics',
    desc: 'Position as Streams. Task assignment as Params with Lock. Emergency stop as Events. Embedded client runs on the robot.',
    protocols: ['Serial', 'BLE', 'UDP', 'MQTT']
  },
  {
    title: 'Edge Computing',
    desc: 'Inference results as Streams. Model health as Params. Retrain triggers as Events. Federation connects edge sites to central ops.',
    protocols: ['HTTP', 'MQTT', 'WebSocket']
  }
]
</script>

<template>
  <section class="explainer-section usecases-section" id="usecases" ref="sectionRef">
    <div class="explainer-inner">
      <h2 class="fade-in">Where CLASP Fits</h2>
      <p class="subtitle fade-in">
        The same architecture works wherever heterogeneous devices need real-time state coordination with security, persistence, and automation.
      </p>
      <div class="case-grid">
        <div v-for="c in cases" :key="c.title" class="case-card stagger">
          <h3>{{ c.title }}</h3>
          <p>{{ c.desc }}</p>
          <div class="case-protocols">
            <span v-for="p in c.protocols" :key="p" class="proto-tag">{{ p }}</span>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.usecases-section {
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
  font-size: 1.05rem;
  line-height: 1.6;
  opacity: 0.7;
}

.case-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 1rem;
}

.case-card {
  border: 2px solid #1a1a1a;
  padding: 1.25rem;
  background: #fefcf6;
  transition: transform 0.15s ease, box-shadow 0.15s ease;
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

.case-card:hover {
  transform: translate(-2px, -2px);
  box-shadow: 4px 4px 0 #1a1a1a;
}

.case-card h3 {
  font-size: 0.85rem;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  margin: 0;
}

.case-card p {
  font-size: 0.85rem;
  line-height: 1.5;
  opacity: 0.7;
  margin: 0;
  flex: 1;
}

.case-protocols {
  display: flex;
  flex-wrap: wrap;
  gap: 0.3rem;
  margin-top: 0.3rem;
}

.proto-tag {
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.6rem;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  padding: 0.15rem 0.45rem;
  border: 1px solid rgba(0,0,0,0.15);
  opacity: 0.5;
}

@media (max-width: 480px) {
  .case-grid { grid-template-columns: 1fr; }
}
</style>
