<script setup>
import { ref } from 'vue'
import { useScrollAnimation } from '../composables/useScrollAnimation.js'

const sectionRef = ref(null)
useScrollAnimation(sectionRef)

const demos = [
  {
    title: 'Social Feed',
    tag: 'Ephemeral',
    desc: 'Real-time posts with TTL expiry, reactions, image sharing, and live video streaming. Everything vanishes after its TTL.',
    href: 'https://demos.clasp.to/social',
    features: ['SET persistence', 'EMIT reactions', 'WebRTC P2P', 'Presence'],
  },
  {
    title: 'Audio Spaces',
    tag: 'Live Audio',
    desc: 'Drop-in audio rooms with host controls, hand-raising, speaker promotion, and live chat. Full WebRTC mesh.',
    href: 'https://demos.clasp.to/spaces',
    features: ['WebRTC mesh', 'Role management', 'Room discovery', 'STREAM volume'],
  },
  {
    title: 'FPS Arena',
    tag: 'Multiplayer',
    desc: 'Browser-based 3D multiplayer shooter with 20Hz position sync, client-side hit detection, and respawn.',
    href: 'https://demos.clasp.to/fps/',
    features: ['Three.js', 'High-rate sync', 'Late-join replay', 'Web Audio'],
  },
  {
    title: 'ghost.webcam',
    tag: 'Ephemeral Video',
    desc: 'Record short videos with face effects and voice filters. Share one-time-view links that self-destruct after viewing.',
    href: 'https://demos.clasp.to/ghost/',
    features: ['MediaPipe', 'View counting', 'Blob transport', 'Auto-deletion'],
  },
]
</script>

<template>
  <section class="explainer-section demos-section" id="demos" ref="sectionRef">
    <div class="explainer-inner">
      <h2 class="fade-in">Live Demos</h2>
      <p class="subtitle fade-in">
        Four applications running on a single CLASP relay. No backend code.
        Guest access, real-time sync, zero server logic.
      </p>

      <div class="demo-grid">
        <a
          v-for="d in demos"
          :key="d.title"
          :href="d.href"
          target="_blank"
          class="demo-card stagger"
        >
          <div class="demo-head">
            <span class="demo-tag">{{ d.tag }}</span>
            <span class="demo-arrow">&rarr;</span>
          </div>
          <h3>{{ d.title }}</h3>
          <p>{{ d.desc }}</p>
          <div class="demo-features">
            <span v-for="f in d.features" :key="f">{{ f }}</span>
          </div>
        </a>
      </div>

      <p class="demo-relay fade-in">
        All demos connect to <code>wss://demo-relay.clasp.to</code>
      </p>
    </div>
  </section>
</template>

<style scoped>
.demos-section {
  background: var(--ink, #1a1a1a);
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
  max-width: 520px;
  margin: 0 auto 3rem;
  font-size: 1rem;
  line-height: 1.7;
  opacity: 0.75;
}

.demo-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 1rem;
}

.demo-card {
  padding: 1.5rem;
  background: rgba(255,255,255,0.04);
  border: 1px solid rgba(255,255,255,0.1);
  transition: transform 0.2s ease, border-color 0.2s ease;
  display: flex;
  flex-direction: column;
  text-decoration: none;
  color: #fff;
}

.demo-card:hover {
  transform: translateY(-3px);
  border-color: rgba(42,157,143,0.5);
}

.demo-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 0.75rem;
}

.demo-tag {
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.65rem;
  letter-spacing: 0.15em;
  text-transform: uppercase;
  color: #2a9d8f;
  background: rgba(42,157,143,0.12);
  border: 1px solid rgba(42,157,143,0.25);
  padding: 2px 8px;
}

.demo-arrow {
  color: rgba(255,255,255,0.3);
  font-size: 1rem;
  transition: color 0.15s, transform 0.15s;
}
.demo-card:hover .demo-arrow {
  color: #2a9d8f;
  transform: translateX(3px);
}

.demo-card h3 {
  font-size: 1rem;
  letter-spacing: 0.12em;
  margin-bottom: 0.5rem;
}

.demo-card p {
  font-size: 0.82rem;
  line-height: 1.65;
  opacity: 0.7;
  flex: 1;
  margin-bottom: 1rem;
}

.demo-features {
  display: flex;
  flex-wrap: wrap;
  gap: 0.3rem;
}
.demo-features span {
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.6rem;
  color: rgba(255,255,255,0.4);
  background: rgba(255,255,255,0.04);
  padding: 2px 6px;
  letter-spacing: 0.04em;
}

.demo-relay {
  text-align: center;
  margin-top: 2rem;
  font-size: 0.85rem;
  opacity: 0.5;
}
.demo-relay code {
  font-family: 'JetBrains Mono', monospace;
  color: #2a9d8f;
}

@media (max-width: 768px) {
  .demo-grid {
    grid-template-columns: 1fr;
  }
}
</style>
