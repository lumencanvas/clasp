<script setup>
import { useRouter, useRoute } from 'vue-router'
import ThemeToggle from './ThemeToggle.vue'

const router = useRouter()
const route = useRoute()

function goHome() {
  if (route.path !== '/') {
    router.push('/')
  } else {
    window.scrollTo({ top: 0, behavior: 'smooth' })
  }
}

function openSearch() {
  // Trigger Cmd+K programmatically
  window.dispatchEvent(new KeyboardEvent('keydown', { key: 'k', metaKey: true }))
}
</script>

<template>
  <nav class="topnav">
    <a class="brand" @click="goHome">CLASP <span class="brand-docs">DOCS</span></a>
    <div class="navlinks">
      <router-link to="/">Home</router-link>
      <router-link to="/getting-started/first-connection">Tutorials</router-link>
      <router-link to="/reference">Reference</router-link>
      <router-link to="/tools/relay-configurator">Tools</router-link>
      <button class="search-trigger" @click="openSearch">
        <span class="search-trigger-icon">&#x2315;</span>
        <span class="search-trigger-text">Search</span>
        <kbd class="search-trigger-kbd">&#x2318;K</kbd>
      </button>
      <a href="https://clasp.to" class="site-link">clasp.to</a>
      <a href="https://github.com/lumencanvas/clasp" target="_blank">GitHub</a>
      <ThemeToggle />
    </div>
  </nav>
</template>

<style scoped>
.brand-docs {
  opacity: 0.5;
  font-size: 0.75rem;
  letter-spacing: 0.22em;
  margin-left: 0.3rem;
}

.site-link {
  opacity: 0.5 !important;
  font-size: 0.75rem !important;
}

.search-trigger {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  background: none;
  border: 1px solid rgba(255,255,255,0.2);
  color: rgba(255,255,255,0.6);
  cursor: pointer;
  font-family: 'Space Mono', monospace;
  font-size: 0.75rem;
  padding: 0.25rem 0.6rem;
  border-radius: 4px;
  transition: border-color 0.15s, color 0.15s;
  letter-spacing: 0.08em;
}

.search-trigger:hover {
  border-color: rgba(255,255,255,0.4);
  color: rgba(255,255,255,0.9);
}

.search-trigger-icon {
  font-size: 0.9rem;
}

.search-trigger-kbd {
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.6rem;
  opacity: 0.5;
  border: 1px solid rgba(255,255,255,0.15);
  padding: 0.1rem 0.3rem;
  border-radius: 2px;
}

@media (max-width: 768px) {
  .search-trigger-text,
  .search-trigger-kbd {
    display: none;
  }
}
</style>
