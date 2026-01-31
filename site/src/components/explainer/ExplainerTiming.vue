<script setup>
import { ref } from 'vue'
import { useScrollAnimation } from '../../composables/useScrollAnimation.js'

const sectionRef = ref(null)
useScrollAnimation(sectionRef)
</script>

<template>
  <section class="explainer-section timing-section" ref="sectionRef">
    <div class="explainer-inner">
      <h2 class="fade-in">Clock Synchronization</h2>
      <p class="subtitle fade-in">
        NTP-style clock sync lets every client agree on "now" within microseconds.
        Schedule bundles to execute at a precise future time across all devices.
      </p>

      <div class="clock-diagram fade-in">
        <div class="clock-row">
          <div class="clock-node client">Client</div>
          <div class="clock-arrow right">
            <span class="arrow-label">T1: Send request</span>
            <div class="arrow-line"></div>
          </div>
          <div class="clock-node server">Server</div>
        </div>

        <div class="clock-row">
          <div class="clock-node client transparent"></div>
          <div class="clock-arrow left">
            <div class="arrow-line"></div>
            <span class="arrow-label">T2: Receive, T3: Reply</span>
          </div>
          <div class="clock-node server transparent"></div>
        </div>

        <div class="clock-row">
          <div class="clock-node client">Client</div>
          <div class="clock-formula">
            <span class="arrow-label">T4: Receive reply</span>
          </div>
          <div class="clock-node server transparent"></div>
        </div>
      </div>

      <div class="formula-box fade-in">
        <div class="formula">offset = ((T2 - T1) + (T3 - T4)) / 2</div>
        <div class="formula-note">Measured multiple times, averaged with outlier rejection</div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.timing-section {
  background: #f4f1e8;
  color: #1a1a1a;
}

h2 {
  font-size: clamp(1.8rem, 4vw, 3rem);
  text-align: center;
  margin-bottom: 1rem;
}

.subtitle {
  text-align: center;
  max-width: 650px;
  margin: 0 auto 3rem;
  font-size: 1.1rem;
  line-height: 1.6;
  opacity: 0.8;
}

.clock-diagram {
  max-width: 500px;
  margin: 0 auto 2rem;
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.clock-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
}

.clock-node {
  font-family: 'Archivo Black', sans-serif;
  text-transform: uppercase;
  font-size: 0.85rem;
  letter-spacing: 0.12em;
  padding: 0.6rem 1.2rem;
  border: 2px solid #1a1a1a;
  min-width: 90px;
  text-align: center;
}

.clock-node.transparent {
  border-color: transparent;
  color: transparent;
}

.clock-node.server {
  background: var(--accent);
  color: #fff;
  border-color: var(--accent);
}

.clock-arrow {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.3rem;
}

.arrow-line {
  width: 100%;
  height: 2px;
  background: #1a1a1a;
  position: relative;
}

.arrow-line::after {
  content: '';
  position: absolute;
  right: 0;
  top: -4px;
  border: 5px solid transparent;
  border-left: 8px solid #1a1a1a;
}

.clock-arrow.left .arrow-line::after {
  right: auto;
  left: 0;
  border-left: none;
  border-right: 8px solid #1a1a1a;
}

.arrow-label {
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.75rem;
  opacity: 0.7;
}

.clock-formula {
  flex: 1;
  text-align: center;
}

.formula-box {
  max-width: 500px;
  margin: 0 auto;
  padding: 1.5rem;
  background: #fff;
  border: 2px solid #1a1a1a;
  text-align: center;
}

.formula {
  font-family: 'JetBrains Mono', monospace;
  font-size: 1rem;
  font-weight: 700;
  margin-bottom: 0.5rem;
}

.formula-note {
  font-size: 0.8rem;
  opacity: 0.6;
}

@media (max-width: 768px) {
  .clock-diagram {
    max-width: 100%;
  }

  .clock-node {
    font-size: 0.7rem;
    padding: 0.4rem 0.6rem;
    min-width: 60px;
  }

  .arrow-label {
    font-size: 0.6rem;
  }

  .formula {
    font-size: 0.75rem;
    word-break: break-all;
  }

  .formula-box {
    padding: 1rem;
  }
}
</style>
