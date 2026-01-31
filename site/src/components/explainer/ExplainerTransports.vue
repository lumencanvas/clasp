<script setup>
import { ref } from 'vue'
import { useScrollAnimation } from '../../composables/useScrollAnimation.js'

const sectionRef = ref(null)
useScrollAnimation(sectionRef)

const transports = [
  { name: 'WebSocket', desc: 'Default transport. Works everywhere, through firewalls, in browsers.', status: 'Primary' },
  { name: 'WebRTC', desc: 'Peer-to-peer with ICE/NAT traversal. Auto-fallback to relay.', status: 'P2P' },
  { name: 'QUIC', desc: 'Multiplexed streams, 0-RTT reconnect, built-in encryption.', status: 'Fast' },
  { name: 'UDP', desc: 'Raw datagrams for lowest latency. Best for trusted local networks.', status: 'Fastest' },
  { name: 'TCP', desc: 'Reliable ordered delivery. Good for control messages, state sync.', status: 'Reliable' },
  { name: 'Serial', desc: 'RS-232/USB serial for embedded devices and legacy hardware.', status: 'Embedded' },
  { name: 'BLE', desc: 'Bluetooth Low Energy for battery-powered sensors and controllers.', status: 'Wireless' }
]
</script>

<template>
  <section class="explainer-section transports-section" ref="sectionRef">
    <div class="explainer-inner">
      <h2 class="fade-in">Seven Transports</h2>
      <p class="subtitle fade-in">
        CLASP doesn't care how bytes travel. Pick the transport that fits your constraints.
      </p>

      <div class="transport-grid">
        <div v-for="t in transports" :key="t.name" class="transport-card stagger">
          <div class="transport-status">{{ t.status }}</div>
          <h3>{{ t.name }}</h3>
          <p>{{ t.desc }}</p>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.transports-section {
  background: #457b9d;
  color: #fff;
}

h2 {
  font-size: clamp(1.8rem, 4vw, 3rem);
  text-align: center;
  margin-bottom: 1rem;
  color: #fff;
}

.subtitle {
  text-align: center;
  max-width: 550px;
  margin: 0 auto 3rem;
  font-size: 1.1rem;
  line-height: 1.6;
  opacity: 0.9;
}

.transport-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 1rem;
}

.transport-card {
  padding: 1.5rem;
  background: rgba(255,255,255,0.08);
  border: 1px solid rgba(255,255,255,0.15);
  transition: transform 0.2s ease, background 0.2s ease;
}

.transport-card:hover {
  transform: translateY(-2px);
  background: rgba(255,255,255,0.14);
}

.transport-status {
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.65rem;
  text-transform: uppercase;
  letter-spacing: 0.15em;
  opacity: 0.6;
  margin-bottom: 0.5rem;
}

.transport-card h3 {
  font-size: 0.95rem;
  letter-spacing: 0.12em;
  margin: 0 0 0.5rem;
  color: #fff;
}

.transport-card p {
  font-size: 0.85rem;
  line-height: 1.5;
  opacity: 0.8;
  margin: 0;
}
</style>
