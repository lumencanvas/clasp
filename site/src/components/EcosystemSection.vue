<script setup>
import { ref } from 'vue'
import { useScrollAnimation } from '../composables/useScrollAnimation.js'

const sectionRef = ref(null)
useScrollAnimation(sectionRef)

const apps = [
  {
    name: 'Latch',
    logo: '/logos/latch-wordmark-dark.svg',
    url: 'https://latch.design',
    desc: 'Node-based visual programming for audio, video, 3D, and AI. Uses CLASP for real-time parameter control and inter-node communication.'
  },
  {
    name: 'LumenCanvas',
    logo: '/logos/lumencanvas-logo-wordmark-dark.svg',
    url: 'https://lumencanvas.studio',
    desc: 'Browser-based projection mapping studio with drag-and-warp controls, live shader coding, and wireless casting to Chromecast/AirPlay. Uses CLASP for real-time parameter control.'
  },
  {
    name: 'Slab',
    logo: '/logos/slab-logo-wordmark.svg',
    logoHeight: '56px',
    url: 'https://slab.clasp.to',
    desc: 'Vue 3 dashboard builder that connects to any data source (REST, WebSocket, MQTT, CLASP, SSE, GraphQL) and visualizes it with 18+ interactive widgets. Drag-and-drop layout, bidirectional controls, and auto-discovered data paths.'
  },
  {
    name: 'Video Streaming',
    logo: null,
    url: '#',
    desc: 'The CLASP Playground demonstrates WebCodecs H.264 encoding with CLASP relay transport and P2P WebRTC video calling with CLASP signaling.',
    isDemo: true,
    icon: 'video'
  }
]
</script>

<template>
  <section class="explainer-section ecosystem-section" id="ecosystem" ref="sectionRef">
    <div class="explainer-inner">
      <h2 class="fade-in">Built With CLASP</h2>
      <p class="subtitle fade-in">
        Real projects using CLASP in production.
      </p>

      <div class="app-grid">
        <div v-for="app in apps" :key="app.name" class="app-card stagger">
          <img
            v-if="app.logo"
            :src="app.logo"
            :alt="app.name"
            class="app-logo"
            :style="app.logoHeight ? { height: app.logoHeight } : {}"
          />
          <div v-else class="app-title-row">
            <svg v-if="app.icon === 'video'" class="app-icon" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="5" width="14" height="14" rx="2"/><path d="M16 10l5-3v10l-5-3z"/></svg>
            <h3>{{ app.name }}</h3>
          </div>
          <p>{{ app.desc }}</p>
          <a
            v-if="!app.isDemo"
            :href="app.url"
            target="_blank"
            class="app-link"
          >
            {{ app.url.replace('https://', '') }} &rarr;
          </a>
          <router-link
            v-else
            to="/playground"
            class="app-link"
          >
            Try the Playground &rarr;
          </router-link>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.ecosystem-section {
  background: #2a9d8f;
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
  max-width: 500px;
  margin: 0 auto 3rem;
  font-size: 1.1rem;
  line-height: 1.6;
  opacity: 0.9;
}

.app-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
  gap: 1.25rem;
}

.app-card {
  padding: 1.75rem;
  background: rgba(255,255,255,0.1);
  border: 1px solid rgba(255,255,255,0.2);
  transition: transform 0.2s ease, background 0.2s ease;
  display: flex;
  flex-direction: column;
}

.app-card:hover {
  transform: translateY(-3px);
  background: rgba(255,255,255,0.15);
}

.app-logo {
  height: 48px;
  width: auto;
  object-fit: contain;
  align-self: flex-start;
  margin-bottom: 0.75rem;
}

.app-title-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.75rem;
}

.app-icon {
  flex-shrink: 0;
  opacity: 0.8;
}

.app-card h3 {
  font-size: 1.1rem;
  letter-spacing: 0.12em;
  margin: 0;
  color: #fff;
}

.app-card p {
  font-size: 0.88rem;
  line-height: 1.6;
  opacity: 0.85;
  margin: 0 0 1rem;
  flex: 1;
}

.app-link {
  color: #fff;
  text-decoration: none;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.8rem;
  opacity: 0.7;
  transition: opacity 0.2s ease;
}

.app-link:hover {
  opacity: 1;
}

@media (max-width: 768px) {
  .app-grid {
    grid-template-columns: 1fr;
  }
}
</style>
