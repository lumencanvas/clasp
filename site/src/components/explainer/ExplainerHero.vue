<script setup>
import ClaspLogo from '../ClaspLogo.vue'
</script>

<template>
  <section class="explainer-section hero-section" id="hero">
    <div class="lockup">
      <div class="logo">
        <ClaspLogo color="#fff" />
      </div>
      <h1>CLASP</h1>
      <p class="tagline">Creative Low-Latency Application Streaming Protocol</p>
      <p class="meta">
        One protocol to connect every creative tool.<br>
        Real-time. Sub-millisecond. Open.
      </p>
      <div class="scroll-hint" aria-hidden="true">
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="6 9 12 15 18 9"/>
        </svg>
      </div>
    </div>
  </section>
</template>

<style scoped>
.hero-section {
  background: #1a1a1a;
  color: #fff;
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
}

.lockup {
  animation: fadeUp 1s ease;
}

.lockup h1 {
  color: #fff;
}

.lockup .tagline {
  color: rgba(255,255,255,0.8);
}

.lockup .meta {
  color: rgba(255,255,255,0.6);
}

.scroll-hint {
  position: absolute;
  bottom: 2rem;
  left: 50%;
  animation: bounce 2s ease infinite;
  color: rgba(255,255,255,0.4);
}
</style>
