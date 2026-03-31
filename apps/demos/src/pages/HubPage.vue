<script setup>
import { useRelay } from '../composables/useRelay.js'

const { authToken } = useRelay()

const demos = [
  {
    title: 'Social Feed',
    path: '/social',
    tag: 'EPHEMERAL',
    desc: 'Real-time posts that expire. Reactions, image sharing, live video streaming. Everything vanishes after TTL.',
    features: ['SET persistence', 'EMIT reactions', 'WebRTC P2P video', 'Presence'],
    external: false,
  },
  {
    title: 'Audio Spaces',
    path: '/spaces',
    tag: 'LIVE AUDIO',
    desc: 'Drop-in audio rooms with host controls. Raise your hand, get promoted to speaker, or just listen.',
    features: ['WebRTC mesh audio', 'Role management', 'Room directory', 'Live chat'],
    external: false,
  },
  {
    title: 'FPS Arena',
    path: '/fps/',
    tag: 'MULTIPLAYER',
    desc: 'Browser-based 3D multiplayer shooter. 20Hz position sync, client-side hit detection, respawn system.',
    features: ['Three.js 3D', 'High-rate SET', 'EMIT hit events', 'Late-join replay'],
    external: true,
  },
  {
    title: 'ghost.webcam',
    path: '/ghost/',
    tag: 'EPHEMERAL VIDEO',
    desc: 'Record short videos with face effects and voice filters. Share one-time-view links. Self-destructing media.',
    features: ['MediaPipe face mesh', 'IndexedDB blobs', 'View counting', 'Auto-deletion'],
    external: true,
  },
]
</script>

<template>
  <div class="hub">
    <header class="hero">
      <div class="container">
        <h1 class="hero-title">CLASP DEMOS</h1>
        <p class="hero-sub">
          Four live applications running on a single CLASP relay.
          Guest access, real-time sync, zero backend code.
        </p>
        <div class="hero-relay">
          <span class="relay-dot"></span>
          <code>wss://demo-relay.clasp.to</code>
        </div>
      </div>
    </header>

    <section class="demos container">
      <component
        v-for="d in demos"
        :key="d.path"
        :is="d.external ? 'a' : 'router-link'"
        :to="d.external ? undefined : d.path"
        :href="d.external ? d.path : undefined"
        class="demo-card fade-in"
      >
        <div class="card-head">
          <span class="card-tag">{{ d.tag }}</span>
          <span class="card-arrow">&rarr;</span>
        </div>
        <h2 class="card-title">{{ d.title }}</h2>
        <p class="card-desc">{{ d.desc }}</p>
        <div class="card-features">
          <span v-for="f in d.features" :key="f" class="feat">{{ f }}</span>
        </div>
      </component>
    </section>

    <footer class="hub-footer container">
      <p>
        Built on <a href="https://clasp.to" target="_blank">CLASP</a>
        -- the real-time protocol for connected devices.
      </p>
      <p class="footer-links">
        <a href="https://docs.clasp.to" target="_blank">Docs</a>
        <a href="https://github.com/lumencanvas/clasp" target="_blank">GitHub</a>
      </p>
    </footer>
  </div>
</template>

<style scoped>
.hub { min-height: 100vh; }
.hero {
  padding: 64px 0 48px;
  border-bottom: 1px solid var(--bdr);
  text-align: center;
}
.hero-title {
  font-family: var(--head);
  font-size: clamp(28px, 5vw, 42px);
  letter-spacing: 0.2em;
  color: var(--br);
  margin-bottom: 12px;
}
.hero-sub {
  max-width: 480px;
  margin: 0 auto 20px;
  color: var(--dim);
  line-height: 1.7;
  font-size: 13px;
}
.hero-relay {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  background: var(--surf);
  border: 1px solid var(--bdr);
  border-radius: var(--r);
  padding: 6px 14px;
}
.relay-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--teal);
}
.hero-relay code {
  font-family: var(--mono);
  font-size: 12px;
  color: var(--teal);
}

.demos {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 14px;
  padding-top: 32px;
  padding-bottom: 32px;
}
.demo-card {
  display: flex;
  flex-direction: column;
  background: var(--surf);
  border: 1px solid var(--bdr);
  border-radius: 6px;
  padding: 20px;
  text-decoration: none;
  color: var(--txt);
  transition: border-color 0.2s, transform 0.2s;
}
.demo-card:hover {
  border-color: var(--teal-m);
  transform: translateY(-2px);
  text-decoration: none;
}
.card-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}
.card-tag {
  font-family: var(--mono);
  font-size: 9px;
  letter-spacing: 0.15em;
  text-transform: uppercase;
  color: var(--teal);
  background: var(--teal-d);
  border: 1px solid var(--teal-m);
  padding: 2px 8px;
  border-radius: 3px;
}
.card-arrow {
  color: var(--dim);
  font-size: 16px;
  transition: color 0.15s, transform 0.15s;
}
.demo-card:hover .card-arrow {
  color: var(--teal);
  transform: translateX(3px);
}
.card-title {
  font-family: var(--head);
  font-size: 16px;
  letter-spacing: 0.08em;
  color: var(--br);
  margin-bottom: 8px;
}
.card-desc {
  font-size: 12px;
  line-height: 1.7;
  color: var(--dim);
  flex: 1;
  margin-bottom: 14px;
}
.card-features {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}
.feat {
  font-family: var(--mono);
  font-size: 9px;
  color: var(--dim);
  background: var(--dim2);
  padding: 3px 7px;
  border-radius: 3px;
  letter-spacing: 0.04em;
}

.hub-footer {
  padding: 32px 16px;
  border-top: 1px solid var(--bdr);
  text-align: center;
  font-size: 12px;
  color: var(--dim);
}
.hub-footer a { color: var(--teal); }
.footer-links {
  margin-top: 8px;
  display: flex;
  justify-content: center;
  gap: 16px;
}

@media (max-width: 480px) {
  .hero { padding: 40px 0 32px; }
  .demos { grid-template-columns: 1fr; }
}
</style>
