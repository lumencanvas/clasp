<script setup>
import { ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import ThemeToggle from './ThemeToggle.vue'
import ClaspLogo from './ClaspLogo.vue'

const router = useRouter()
const route = useRoute()
const menuOpen = ref(false)

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
    <a class="brand" @click="goHome">
      <ClaspLogo :size="24" />
      <span class="brand-text">CLASP</span>
      <span class="brand-docs">DOCS</span>
    </a>
    <div class="nav-actions">
      <button class="search-trigger" @click="openSearch">
        <span class="search-trigger-icon">&#x2315;</span>
        <span class="search-trigger-text">Search</span>
        <kbd class="search-trigger-kbd">&#x2318;K</kbd>
      </button>
      <ThemeToggle />
      <button class="nav-toggle" @click="menuOpen = !menuOpen" aria-label="Toggle navigation">
        {{ menuOpen ? '\u2715' : '\u2630' }}
      </button>
    </div>
    <div class="navlinks" :class="{ open: menuOpen }">
      <router-link to="/" @click="menuOpen = false">Home</router-link>
      <router-link to="/getting-started/first-connection" @click="menuOpen = false">Tutorials</router-link>
      <router-link to="/reference" @click="menuOpen = false">Reference</router-link>
      <router-link to="/tools/relay-configurator" @click="menuOpen = false">Tools</router-link>
      <a href="https://clasp.to" class="site-link" @click="menuOpen = false">clasp.to</a>
      <a href="https://github.com/lumencanvas/clasp" target="_blank" @click="menuOpen = false">GitHub</a>
    </div>
  </nav>
</template>

<style scoped>
.brand {
  display: flex;
  align-items: center;
  gap: 0.45rem;
}

.brand-text {
  letter-spacing: 0.28em;
}

.brand-docs {
  opacity: 0.5;
  font-size: 0.75rem;
  letter-spacing: 0.22em;
}

.nav-actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.nav-toggle {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 44px;
  height: 44px;
  background: none;
  border: none;
  color: #fff;
  font-size: 1.2rem;
  cursor: pointer;
  padding: 0;
}

.navlinks {
  display: none;
  position: absolute;
  top: var(--nav-height);
  left: 0;
  right: 0;
  background: rgba(26,26,26,0.96);
  backdrop-filter: blur(8px);
  flex-direction: column;
  padding: 0.5rem 0;
  border-bottom: 1px solid rgba(255,255,255,0.08);
  z-index: 49;
}

.navlinks.open {
  display: flex;
}

.navlinks a {
  padding: 0.75rem 1.5rem;
  min-height: 44px;
  display: flex;
  align-items: center;
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

.search-trigger-text,
.search-trigger-kbd {
  display: none;
}

@media (min-width: 768px) {
  .nav-toggle {
    display: none;
  }

  .nav-actions {
    order: 2;
  }

  .navlinks {
    display: flex;
    position: static;
    flex-direction: row;
    background: none;
    backdrop-filter: none;
    padding: 0;
    border-bottom: none;
    flex: 1;
    justify-content: flex-end;
    margin-right: 0.5rem;
  }

  .navlinks a {
    padding: 0;
    min-height: 0;
    display: inline;
  }

  .search-trigger-text,
  .search-trigger-kbd {
    display: inline;
  }
}
</style>
