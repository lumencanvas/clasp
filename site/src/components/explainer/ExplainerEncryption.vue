<script setup>
import { ref } from 'vue'
import { useScrollAnimation } from '../../composables/useScrollAnimation.js'

const sectionRef = ref(null)
useScrollAnimation(sectionRef)

const features = [
  {
    title: 'ECDH Key Exchange',
    desc: 'Clients negotiate a shared secret using ephemeral P-256 keys. The router is never part of the exchange and never holds key material.',
    icon: 'M18 8h-1V6c0-2.76-2.24-5-5-5S7 3.24 7 6v2H6c-1.1 0-2 .9-2 2v10c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V10c0-1.1-.9-2-2-2zM12 17c-1.1 0-2-.9-2-2s.9-2 2-2 2 .9 2 2-.9 2-2 2zm3.1-9H8.9V6c0-1.71 1.39-3.1 3.1-3.1s3.1 1.39 3.1 3.1v2z'
  },
  {
    title: 'AES-256-GCM',
    desc: 'All payload data is encrypted client-side before it leaves the device. The router relays ciphertext. Auto-rotation every 60 seconds minimum.',
    icon: 'M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4zm0 10.99h7c-.53 4.12-3.28 7.79-7 8.94V12H5V6.3l7-3.11v8.8z'
  },
  {
    title: 'Capability Tokens',
    desc: 'Ed25519 tokens with UCAN-style delegation chains. Scope access to specific address patterns. Delegate narrower permissions without contacting the issuer.',
    icon: 'M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z'
  }
]
</script>

<template>
  <section class="explainer-section encryption-section" id="encryption" ref="sectionRef">
    <div class="explainer-inner">
      <h2 class="fade-in">End-to-End Encryption</h2>
      <p class="subtitle fade-in">
        The router never sees your data. Clients encrypt locally. The server relays ciphertext and nothing else.
      </p>

      <div class="enc-flow fade-in">
        <div class="enc-node">
          <div class="enc-label">ALICE</div>
          <div class="enc-sub">encrypts</div>
        </div>
        <div class="enc-arrow">
          <div class="enc-arrow-line"></div>
          <div class="enc-arrow-label">ciphertext</div>
        </div>
        <div class="enc-node router">
          <div class="enc-label">ROUTER</div>
          <div class="enc-sub">blind relay</div>
        </div>
        <div class="enc-arrow">
          <div class="enc-arrow-line"></div>
          <div class="enc-arrow-label">ciphertext</div>
        </div>
        <div class="enc-node">
          <div class="enc-label">BOB</div>
          <div class="enc-sub">decrypts</div>
        </div>
      </div>

      <div class="feature-grid">
        <div v-for="feature in features" :key="feature.title" class="feature-card stagger">
          <svg class="feature-icon" viewBox="0 0 24 24" width="32" height="32">
            <path :d="feature.icon" fill="currentColor"/>
          </svg>
          <h3>{{ feature.title }}</h3>
          <p>{{ feature.desc }}</p>
        </div>
      </div>

      <p class="note fade-in">
        TOFU (trust on first use) key verification. ECDSA P-256 signatures for message authenticity. Replay protection via timestamps.
      </p>
    </div>
  </section>
</template>

<style scoped>
.encryption-section {
  background: #1a1a1a;
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
  max-width: 560px;
  margin: 0 auto 2.5rem;
  font-size: 1.05rem;
  line-height: 1.6;
  opacity: 0.6;
}

/* E2E flow diagram */
.enc-flow {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0;
  margin: 0 auto 3rem;
  max-width: 600px;
  flex-wrap: wrap;
}

.enc-node {
  border: 1.5px solid rgba(255,255,255,0.2);
  padding: 0.8rem 1.5rem;
  text-align: center;
  background: rgba(255,255,255,0.03);
  min-width: 100px;
}

.enc-node.router {
  border-color: var(--accent, #e63946);
  background: rgba(230, 57, 70, 0.08);
}

.enc-label {
  font-family: 'Archivo Black', sans-serif;
  font-size: 0.7rem;
  letter-spacing: 0.12em;
  text-transform: uppercase;
}

.enc-sub {
  font-size: 0.6rem;
  opacity: 0.4;
  margin-top: 0.2rem;
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

.enc-arrow {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 0 0.3rem;
}

.enc-arrow-line {
  width: 40px;
  height: 1.5px;
  background: rgba(255,255,255,0.15);
}

.enc-arrow-label {
  font-size: 0.5rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  opacity: 0.3;
  margin-top: 0.2rem;
}

/* Feature cards */
.feature-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 1.5rem;
  margin-bottom: 2rem;
}

.feature-card {
  padding: 1.5rem;
  background: rgba(255,255,255,0.04);
  border: 1px solid rgba(255,255,255,0.08);
  transition: transform 0.2s ease, background 0.2s ease;
}

.feature-card:hover {
  transform: translateY(-2px);
  background: rgba(255,255,255,0.06);
}

.feature-icon {
  margin-bottom: 0.75rem;
  opacity: 0.7;
  color: #8b5cf6;
}

.feature-card h3 {
  font-size: 0.85rem;
  letter-spacing: 0.1em;
  margin-bottom: 0.5rem;
  color: #fff;
}

.feature-card p {
  font-size: 0.85rem;
  line-height: 1.55;
  opacity: 0.6;
  margin: 0;
}

.note {
  text-align: center;
  font-size: 0.8rem;
  opacity: 0.3;
  max-width: 560px;
  margin: 0 auto;
}

@media (max-width: 480px) {
  .enc-flow { gap: 0.5rem; }
  .enc-node { padding: 0.5rem 0.8rem; min-width: 70px; }
  .enc-arrow-line { width: 20px; }
}
</style>
