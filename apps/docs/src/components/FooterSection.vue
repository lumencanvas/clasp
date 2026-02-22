<template>
  <footer class="footer-section">
    <div>CLASP &middot; Creative Low-Latency Application Streaming Protocol &middot; Open Specification</div>
    <div class="attribution">Maintained by <a href="https://lumencanvas.studio">LumenCanvas</a> &middot; 2026</div>
  </footer>
</template>

<style scoped>
.footer-section {
  background: #1a1a1a;
  color: rgba(255,255,255,0.6);
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  text-align: center;
  padding: 3rem 8vw;
}

.attribution {
  font-size: 0.875rem;
  color: rgba(255,255,255,0.4);
}

.attribution a {
  color: var(--accent);
  text-decoration: none;
}

.attribution a:hover {
  text-decoration: underline;
}
</style>
