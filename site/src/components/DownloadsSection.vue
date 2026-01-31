<script setup>
import ScreenshotCarousel from './ScreenshotCarousel.vue'

const downloads = [
  {
    platform: 'macOS',
    desc: 'Apple Silicon & Intel',
    links: [
      { label: 'Apple Silicon (.dmg)', url: 'https://github.com/lumencanvas/clasp/releases/latest/download/CLASP.Bridge-arm64.dmg' },
      { label: 'Intel (.dmg)', url: 'https://github.com/lumencanvas/clasp/releases/latest/download/CLASP.Bridge-x64.dmg' }
    ]
  },
  {
    platform: 'Windows',
    desc: 'Windows 10/11',
    links: [
      { label: 'Installer (.exe)', url: 'https://github.com/lumencanvas/clasp/releases/latest/download/CLASP.Bridge-Setup.exe' },
      { label: 'Portable (.exe)', url: 'https://github.com/lumencanvas/clasp/releases/latest/download/CLASP.Bridge-portable.exe' }
    ]
  },
  {
    platform: 'Linux',
    desc: 'AppImage & Deb',
    links: [
      { label: 'AppImage', url: 'https://github.com/lumencanvas/clasp/releases/latest/download/CLASP.Bridge.AppImage' },
      { label: 'Debian (.deb)', url: 'https://github.com/lumencanvas/clasp/releases/latest/download/clasp-bridge.deb' }
    ]
  }
]

const features = [
  'Multi-protocol servers (OSC, MQTT, WebSocket, HTTP, Art-Net, DMX)',
  'Visual signal routing with transforms',
  'Learn mode for quick MIDI/OSC mapping',
  'Real-time signal monitor',
  'Automatic state persistence'
]
</script>

<template>
  <section class="explainer-section downloads-section" id="downloads">
    <div class="explainer-inner">
      <h2>Desktop App</h2>
      <p class="section-desc">
        The CLASP Bridge desktop app provides a visual interface for protocol bridging,
        signal mapping, and monitoring.
      </p>

      <ScreenshotCarousel />

      <div class="features">
        <div class="feature" v-for="feature in features" :key="feature">
          <svg class="check-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
            <polyline points="20 6 9 17 4 12"/>
          </svg>
          <span>{{ feature }}</span>
        </div>
      </div>

      <div class="macos-note">
        <b>macOS users:</b> If you see "damaged and can't be opened", run this in Terminal:
        <code>xattr -cr /Applications/CLASP\ Bridge.app</code>
      </div>

      <div class="download-grid">
        <div class="download-card" v-for="dl in downloads" :key="dl.platform">
          <div class="platform-icon">
            <svg v-if="dl.platform === 'macOS'" width="48" height="48" viewBox="0 0 24 24" fill="currentColor">
              <path d="M18.71 19.5c-.83 1.24-1.71 2.45-3.05 2.47-1.34.03-1.77-.79-3.29-.79-1.53 0-2 .77-3.27.82-1.31.05-2.3-1.32-3.14-2.53C4.25 17 2.94 12.45 4.7 9.39c.87-1.52 2.43-2.48 4.12-2.51 1.28-.02 2.5.87 3.29.87.78 0 2.26-1.07 3.81-.91.65.03 2.47.26 3.64 1.98-.09.06-2.17 1.28-2.15 3.81.03 3.02 2.65 4.03 2.68 4.04-.03.07-.42 1.44-1.38 2.83M13 3.5c.73-.83 1.94-1.46 2.94-1.5.13 1.17-.34 2.35-1.04 3.19-.69.85-1.83 1.51-2.95 1.42-.15-1.15.41-2.35 1.05-3.11z"/>
            </svg>
            <svg v-else-if="dl.platform === 'Windows'" width="48" height="48" viewBox="0 0 24 24" fill="currentColor">
              <path d="M3 12V6.75l6-1.32v6.48L3 12zm17-9v8.75l-10 .15V5.21L20 3zM3 13l6 .09v6.81l-6-1.15V13zm17 .25V22l-10-1.91V13.1l10 .15z"/>
            </svg>
            <svg v-else width="48" height="48" viewBox="0 0 24 24" fill="currentColor">
              <path d="M12.504 0c-.155 0-.315.008-.48.021-4.226.333-3.105 4.807-3.17 6.298-.076 1.092-.3 1.953-1.05 3.02-.885 1.051-2.127 2.75-2.716 4.521-.278.832-.41 1.684-.287 2.489a.424.424 0 00-.11.135c-.26.268-.45.6-.663.839-.199.199-.485.267-.797.4-.313.136-.658.269-.864.68-.09.189-.136.394-.132.602 0 .199.027.4.055.536.058.399.116.728.04.97-.249.68-.28 1.145-.106 1.484.174.334.535.47.94.601.81.2 1.91.135 2.774.6.926.466 1.866.67 2.616.47.526-.116.97-.464 1.208-.946.587-.003 1.23-.269 2.26-.334.699-.058 1.574.267 2.577.2.025.134.063.198.114.333l.003.003c.391.778 1.113 1.132 1.884 1.071.771-.06 1.592-.536 2.257-1.306.631-.765 1.683-1.084 2.378-1.503.348-.199.629-.469.649-.853.023-.4-.2-.811-.714-1.376v-.097l-.003-.003c-.17-.2-.25-.535-.338-.926-.085-.401-.182-.786-.492-1.046h-.003c-.059-.054-.123-.067-.188-.135a.357.357 0 00-.19-.064c.431-1.278.264-2.55-.173-3.694-.533-1.41-1.465-2.638-2.175-3.483-.796-1.005-1.576-1.957-1.56-3.368.026-2.152.236-6.133-3.544-6.139z"/>
            </svg>
          </div>
          <h3>{{ dl.platform }}</h3>
          <p class="platform-desc">{{ dl.desc }}</p>
          <div class="download-links">
            <a v-for="link in dl.links" :key="link.label" :href="link.url" class="download-link">
              <svg class="download-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M7 10l5 5 5-5M12 15V3"/>
              </svg>
              {{ link.label }}
            </a>
          </div>
        </div>
      </div>

      <div class="cli-section">
        <svg class="terminal-icon" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/>
        </svg>
        <h3>Command Line</h3>
        <p>Install the CLI tool via Cargo:</p>
        <code class="cli-install">cargo install clasp-cli</code>
      </div>
    </div>
  </section>
</template>

<style scoped>
.downloads-section {
  background: #f4f1e8;
  color: #1a1a1a;
}

h2 {
  font-size: clamp(1.8rem, 4vw, 3rem);
  text-align: center;
  margin-bottom: 1rem;
}

.section-desc {
  max-width: 600px;
  margin: 0 auto 1.5rem;
  text-align: center;
  opacity: 0.7;
}

.features {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 0.75rem;
  margin-bottom: 2rem;
}

.feature {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
  background: rgba(255,255,255,0.6);
  border: 1px solid rgba(0,0,0,0.1);
  font-size: 0.875rem;
}

.check-icon {
  color: var(--accent);
  flex-shrink: 0;
}

.macos-note {
  max-width: 600px;
  margin: 0 auto 1.5rem;
  padding: 0.75rem 1rem;
  background: rgba(230, 57, 70, 0.08);
  border: 1px solid rgba(230, 57, 70, 0.25);
  font-size: 0.85rem;
  text-align: center;
}

.macos-note code {
  display: block;
  margin-top: 0.5rem;
  padding: 0.5rem;
  background: rgba(255,255,255,0.7);
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.8rem;
  user-select: all;
}

.download-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
  gap: 1rem;
  max-width: 900px;
  margin: 0 auto 2rem;
}

.download-card {
  padding: 2rem;
  background: rgba(255,255,255,0.6);
  border: 2px solid rgba(0,0,0,0.1);
  text-align: center;
}

.platform-icon {
  margin-bottom: 1rem;
  opacity: 0.6;
}

.download-card h3 {
  font-size: 1.25rem;
  margin-bottom: 0.5rem;
}

.platform-desc {
  opacity: 0.6;
  font-size: 0.875rem;
  margin-bottom: 1.5rem;
}

.download-links {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.download-link {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 0.75rem 1rem;
  background: var(--accent);
  color: white;
  text-decoration: none;
  font-weight: 600;
  font-size: 0.875rem;
  transition: opacity 0.2s;
}

.download-link:hover {
  opacity: 0.85;
}

.download-icon {
  flex-shrink: 0;
}

.cli-section {
  max-width: 500px;
  margin: 0 auto;
  padding: 2rem;
  background: rgba(255,255,255,0.6);
  border: 2px solid rgba(0,0,0,0.1);
  text-align: center;
}

.terminal-icon {
  color: var(--accent);
  margin-bottom: 1rem;
}

.cli-section h3 {
  margin-bottom: 0.5rem;
}

.cli-section p {
  opacity: 0.6;
  margin-bottom: 1rem;
}

.cli-install {
  display: block;
  padding: 1rem;
  background: #1a1a1a;
  color: #fff;
  border: 1px solid rgba(0,0,0,0.1);
  font-family: 'JetBrains Mono', monospace;
  font-size: 1rem;
  user-select: all;
}
</style>
