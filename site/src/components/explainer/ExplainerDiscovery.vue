<script setup>
import { ref } from 'vue'
import { useScrollAnimation } from '../../composables/useScrollAnimation.js'

const sectionRef = ref(null)
useScrollAnimation(sectionRef)

const methods = [
  {
    name: 'mDNS / Bonjour',
    desc: 'Zero-configuration discovery on local networks. Routers advertise _clasp._tcp and clients find them automatically. Works out of the box on macOS, Linux, and Windows.',
    icon: 'M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z'
  },
  {
    name: 'UDP Broadcast',
    desc: 'Fallback for networks where mDNS is blocked. Routers send periodic broadcast packets on port 7331. Simple, reliable, works on any network without special services.',
    icon: 'M1 9l2 2c4.97-4.97 13.03-4.97 18 0l2-2C16.93 2.93 7.08 2.93 1 9zm8 8l3 3 3-3c-1.65-1.66-4.34-1.66-6 0zm-4-4l2 2c2.76-2.76 7.24-2.76 10 0l2-2C15.14 9.14 8.87 9.14 5 13z'
  },
  {
    name: 'Rendezvous Server',
    desc: 'For connecting across networks or through NAT. Clients register with a known server that brokers introductions. Supports WebRTC ICE negotiation for P2P fallback.',
    icon: 'M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4zm0 10.99h7c-.53 4.12-3.28 7.79-7 8.94V12H5V6.3l7-3.11v8.8z'
  }
]
</script>

<template>
  <section class="explainer-section discovery-section" ref="sectionRef">
    <div class="explainer-inner">
      <h2 class="fade-in">Auto-Discovery</h2>
      <p class="subtitle fade-in">
        Plug in a device, it finds the router. No IP addresses to configure, no config files to edit.
      </p>
      <div class="discovery-grid">
        <div v-for="method in methods" :key="method.name" class="discovery-card stagger">
          <svg class="discovery-icon" viewBox="0 0 24 24" width="36" height="36">
            <path :d="method.icon" fill="currentColor"/>
          </svg>
          <h3>{{ method.name }}</h3>
          <p>{{ method.desc }}</p>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.discovery-section {
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
  max-width: 600px;
  margin: 0 auto 3rem;
  font-size: 1.1rem;
  line-height: 1.6;
  opacity: 0.9;
}

.discovery-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
  gap: 1.5rem;
}

.discovery-card {
  padding: 2rem;
  background: rgba(255,255,255,0.1);
  border: 1px solid rgba(255,255,255,0.2);
  transition: transform 0.2s ease, background 0.2s ease;
}

.discovery-card:hover {
  transform: translateY(-3px);
  background: rgba(255,255,255,0.15);
}

.discovery-icon {
  margin-bottom: 1rem;
  opacity: 0.9;
}

.discovery-card h3 {
  font-size: 0.95rem;
  letter-spacing: 0.12em;
  margin-bottom: 0.75rem;
  color: #fff;
}

.discovery-card p {
  font-size: 0.9rem;
  line-height: 1.6;
  opacity: 0.85;
  margin: 0;
}
</style>
