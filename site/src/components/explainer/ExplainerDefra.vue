<script setup>
import { ref } from 'vue'
import { useScrollAnimation } from '../../composables/useScrollAnimation.js'

const sectionRef = ref(null)
useScrollAnimation(sectionRef)

const crates = [
  { name: 'state-defra', desc: 'Write-through cache with async DefraDB flush' },
  { name: 'journal-defra', desc: 'Append-only event journal with P2P sync' },
  { name: 'defra-bridge', desc: 'Bidirectional DefraDB/CLASP signals' },
  { name: 'defra-transport', desc: 'DefraDB P2P over CLASP transports' },
  { name: 'registry-defra', desc: 'P2P device identity store' },
  { name: 'config-defra', desc: 'P2P config sync with version history' }
]
</script>

<template>
  <section class="explainer-section defra-section" id="defra" ref="sectionRef">
    <div class="explainer-inner">
      <h2 class="fade-in">Persistent Distributed State</h2>
      <p class="subtitle fade-in">
        CLASP moves data at microsecond speed but forgets everything on restart.
        DefraDB remembers everything and syncs between devices automatically.
        Six crates plug them together.
      </p>

      <div class="arch-diagram fade-in">
        <div class="arch-layer">
          <div class="arch-box hot">
            <div class="arch-title">Memory Cache</div>
            <div class="arch-stat">&lt;100us</div>
            <div class="arch-desc">DashMap hot path. All reads and writes hit memory first. Zero latency penalty.</div>
          </div>
          <div class="arch-arrow">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M12 5v14M19 12l-7 7-7-7"/></svg>
            <span>async flush</span>
          </div>
          <div class="arch-box cold">
            <div class="arch-title">DefraDB</div>
            <div class="arch-stat">Merkle CRDTs</div>
            <div class="arch-desc">Persistent, content-addressed, peer-to-peer. Survives restarts. Syncs between nodes.</div>
          </div>
        </div>
      </div>

      <div class="crate-grid">
        <div v-for="crate in crates" :key="crate.name" class="crate-card stagger">
          <span class="crate-name">{{ crate.name }}</span>
          <span class="crate-desc">{{ crate.desc }}</span>
        </div>
      </div>

      <div class="defra-topo fade-in">
        <code>Router A  -->  DefraDB Node 1  &lt;--P2P--&gt;  DefraDB Node 2  &lt;--  Router B</code>
      </div>
    </div>
  </section>
</template>

<style scoped>
.defra-section {
  background: #fefcf6;
  color: #1a1a1a;
}

h2 {
  font-size: clamp(1.8rem, 4vw, 3rem);
  text-align: center;
  margin-bottom: 1rem;
}

.subtitle {
  text-align: center;
  max-width: 580px;
  margin: 0 auto 2.5rem;
  font-size: 1.05rem;
  line-height: 1.6;
  opacity: 0.7;
}

/* Architecture diagram */
.arch-diagram {
  max-width: 360px;
  margin: 0 auto 2.5rem;
}

.arch-layer {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0;
}

.arch-box {
  width: 100%;
  border: 2px solid #1a1a1a;
  padding: 1.25rem;
  text-align: center;
  background: #fff;
}

.arch-box.hot {
  box-shadow: 3px 3px 0 #1a1a1a;
}

.arch-box.cold {
  border-color: #8b5cf6;
  box-shadow: 3px 3px 0 #8b5cf6;
}

.arch-title {
  font-family: 'Archivo Black', sans-serif;
  font-size: 0.85rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  margin-bottom: 0.25rem;
}

.arch-box.cold .arch-title {
  color: #8b5cf6;
}

.arch-stat {
  font-family: 'JetBrains Mono', monospace;
  font-size: 1.1rem;
  font-weight: 700;
  color: var(--accent, #e63946);
  margin-bottom: 0.25rem;
}

.arch-box.cold .arch-stat {
  color: #8b5cf6;
}

.arch-desc {
  font-size: 0.8rem;
  opacity: 0.6;
  line-height: 1.5;
}

.arch-arrow {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 0.4rem 0;
  opacity: 0.35;
}

.arch-arrow span {
  font-size: 0.6rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  margin-top: 0.15rem;
}

/* Crate grid */
.crate-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 0.5rem;
  margin-bottom: 2rem;
}

.crate-card {
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
  padding: 0.75rem 1rem;
  border: 1px solid rgba(0,0,0,0.08);
  background: rgba(0,0,0,0.02);
  transition: border-color 0.15s;
}

.crate-card:hover {
  border-color: #8b5cf6;
}

.crate-name {
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.75rem;
  font-weight: 700;
  color: #8b5cf6;
}

.crate-desc {
  font-size: 0.78rem;
  opacity: 0.6;
  line-height: 1.4;
}

/* Topology diagram */
.defra-topo {
  text-align: center;
}

.defra-topo code {
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.75rem;
  opacity: 0.4;
  letter-spacing: 0.02em;
}

@media (max-width: 480px) {
  .crate-grid { grid-template-columns: 1fr; }
}
</style>
