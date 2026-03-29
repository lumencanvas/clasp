<script setup>
import { useRouter, useRoute } from 'vue-router'
import ThemeToggle from './ThemeToggle.vue'
import ClaspLogo from './ClaspLogo.vue'

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
    <div class="navlinks">
      <router-link to="/">Home</router-link>
      <router-link to="/getting-started/first-connection">Tutorials</router-link>
      <router-link to="/reference">Reference</router-link>
      <router-link to="/tools/relay-configurator">Tools</router-link>
      <a href="https://clasp.to" class="site-link">clasp.to</a>
      <a href="https://github.com/lumencanvas/clasp" target="_blank">GitHub</a>
    </div>
    <div class="nav-actions">
      <button class="search-trigger" @click="openSearch">
        <span class="search-trigger-icon">&#x2315;</span>
        <span class="search-trigger-text">Search</span>
        <kbd class="search-trigger-kbd">&#x2318;K</kbd>
      </button>
      <ThemeToggle />
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

.navlinks {
  display: none;
}

.navlinks a {
  text-decoration: none;
  color: #fff;
  opacity: 0.75;
  letter-spacing: 0.08em;
  cursor: pointer;
  transition: opacity 0.15s;
}

.navlinks a:hover {
  opacity: 1;
  text-decoration: none;
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
  .nav-actions {
    order: 2;
  }

  .navlinks {
    display: flex;
    flex-direction: row;
    gap: 0.8rem;
    flex: 1;
    justify-content: flex-end;
    margin-right: 0.5rem;
  }

  .navlinks a {
    font-size: 0.75rem;
    letter-spacing: 0.12em;
  }

  .search-trigger-text,
  .search-trigger-kbd {
    display: inline;
  }
}

@media (min-width: 1024px) {
  .navlinks {
    gap: 1.2rem;
  }

  .navlinks a {
    font-size: 0.8rem;
  }
}
</style>
