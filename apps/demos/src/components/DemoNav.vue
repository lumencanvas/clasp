<script setup>
import { useRelay } from '../composables/useRelay.js'

const { userName, connected } = useRelay()
</script>

<template>
  <nav class="nav">
    <div class="nav-inner">
      <router-link to="/" class="nav-brand">
        <span class="nav-logo">C</span>
        <span class="nav-title">DEMOS</span>
      </router-link>

      <div class="nav-links">
        <router-link to="/social">Social</router-link>
        <router-link to="/spaces">Spaces</router-link>
        <a href="/fps/">FPS</a>
        <a href="/ghost/">Ghost</a>
      </div>

      <div class="nav-right">
        <span class="conn-dot" :class="{ on: connected }"></span>
        <span class="nav-user">{{ connected ? (userName || 'guest') : 'offline' }}</span>
      </div>
    </div>
  </nav>
</template>

<style scoped>
.nav {
  position: sticky;
  top: 0;
  z-index: 200;
  background: rgba(10,10,10,0.94);
  backdrop-filter: blur(20px);
  border-bottom: 1px solid var(--bdr);
}
.nav-inner {
  max-width: 1000px;
  margin: 0 auto;
  padding: 0 16px;
  height: 50px;
  display: flex;
  align-items: center;
  gap: 16px;
}
.nav-brand {
  display: flex;
  align-items: center;
  gap: 8px;
  text-decoration: none;
  flex-shrink: 0;
}
.nav-logo {
  width: 28px;
  height: 28px;
  background: var(--teal);
  color: var(--bg);
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: var(--head);
  font-size: 16px;
  border-radius: 3px;
}
.nav-title {
  font-family: var(--head);
  font-size: 13px;
  color: var(--br);
  letter-spacing: 0.2em;
}
.nav-links {
  display: flex;
  gap: 4px;
  flex: 1;
}
.nav-links a {
  font-size: 11px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--dim);
  padding: 6px 10px;
  border-radius: var(--r);
  text-decoration: none;
  transition: color 0.15s, background 0.15s;
}
.nav-links a:hover,
.nav-links a.router-link-active {
  color: var(--br);
  background: var(--bdr);
}
.nav-right {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}
.conn-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--dim);
}
.conn-dot.on {
  background: var(--grn);
}
.nav-user {
  font-family: var(--mono);
  font-size: 10px;
  letter-spacing: 0.06em;
  color: var(--dim);
}

@media (max-width: 520px) {
  .nav-links { gap: 0; }
  .nav-links a { padding: 6px 6px; font-size: 10px; }
  .nav-title { display: none; }
}
</style>
