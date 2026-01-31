<script setup>
import { ref } from 'vue'
import { useScrollAnimation } from '../../composables/useScrollAnimation.js'

const sectionRef = ref(null)
useScrollAnimation(sectionRef)

const copied = ref(false)

function copyInstall() {
  navigator.clipboard.writeText('npm install @clasp-to/core').then(() => {
    copied.value = true
    setTimeout(() => { copied.value = false }, 2000)
  })
}
</script>

<template>
  <section class="explainer-section cta-section" ref="sectionRef">
    <div class="explainer-inner cta-inner">
      <h2 class="fade-in">Get Started</h2>

      <div class="install-block fade-in" @click="copyInstall">
        <code>npm install @clasp-to/core</code>
        <span class="copy-hint">{{ copied ? 'Copied!' : 'Click to copy' }}</span>
      </div>

      <div class="cta-buttons fade-in">
        <a href="https://github.com/lumencanvas/clasp" target="_blank" class="cta-btn primary">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
          </svg>
          GitHub
        </a>
        <a href="#spec" class="cta-btn secondary">
          Read the Spec
        </a>
      </div>

      <p class="cta-note fade-in">
        Also available: <code>pip install clasp-to</code> &middot; <code>cargo add clasp-client</code> &middot; Desktop app for macOS, Windows, Linux
      </p>
    </div>
  </section>
</template>

<style scoped>
.cta-section {
  background: #1a1a1a;
  color: #fff;
}

.cta-inner {
  text-align: center;
}

h2 {
  font-size: clamp(2rem, 5vw, 3.5rem);
  margin-bottom: 2rem;
  color: #fff;
}

.install-block {
  display: inline-block;
  padding: 1.2rem 2rem;
  background: rgba(255,255,255,0.05);
  border: 2px solid rgba(255,255,255,0.2);
  cursor: pointer;
  transition: all 0.2s ease;
  margin-bottom: 2rem;
}

.install-block:hover {
  border-color: var(--accent);
  background: rgba(230, 57, 70, 0.1);
}

.install-block code {
  font-family: 'JetBrains Mono', monospace;
  font-size: 1.1rem;
  color: var(--accent);
}

.copy-hint {
  display: block;
  font-size: 0.7rem;
  opacity: 0.5;
  margin-top: 0.5rem;
  letter-spacing: 0.1em;
}

.cta-buttons {
  display: flex;
  justify-content: center;
  gap: 1rem;
  margin-bottom: 2rem;
  flex-wrap: wrap;
}

.cta-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.9rem 2rem;
  text-decoration: none;
  font-family: 'Space Mono', monospace;
  font-size: 0.9rem;
  letter-spacing: 0.12em;
  transition: all 0.15s ease;
  text-transform: uppercase;
}

.cta-btn.primary {
  background: #fff;
  color: #1a1a1a;
  border: 2px solid #fff;
}

.cta-btn.primary:hover {
  background: transparent;
  color: #fff;
}

.cta-btn.secondary {
  background: transparent;
  color: #fff;
  border: 2px solid rgba(255,255,255,0.4);
}

.cta-btn.secondary:hover {
  border-color: #fff;
}

.cta-note {
  opacity: 0.5;
  font-size: 0.85rem;
}

.cta-note code {
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.8rem;
  color: rgba(255,255,255,0.8);
}
</style>
