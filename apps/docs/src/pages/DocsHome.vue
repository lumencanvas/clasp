<script setup>
import { computed } from 'vue'
import manifest from 'virtual:docs-manifest'

const sdkCount = computed(() => manifest.filter(p => p.section === 'sdk').length)
const bridgeCount = computed(() => manifest.filter(p => p.section === 'protocols').length)
const refCount = computed(() => manifest.filter(p => p.section === 'reference').length)
</script>

<template>
  <div class="docs-home">
    <div class="docs-hero">
      <h1>CLASP DOCS</h1>
      <p class="tagline">
        Real-time signal router and protocol bridge.
        One API for state, events, and streams across JS, Python, Rust
        &mdash; with auth, persistence, federation, and 8 protocol bridges.
      </p>
    </div>

    <div class="home-quickstart">
      <h2>QUICK START</h2>
      <pre><code>npm install @clasp-to/core</code></pre>
      <pre><code><span style="opacity:0.5">// connect and send your first signal</span>
import { ClaspBuilder } from '@clasp-to/core'

const client = await new ClaspBuilder('ws://localhost:7330')
  .withName('My App')
  .connect()

client.set('/lights/brightness', 0.8)
client.on('/lights/*', (value, address) =&gt; console.log(address, value))</code></pre>
      <p style="margin-top: 1rem; opacity: 0.7; font-size: 0.85rem;">
        <router-link to="/getting-started/first-connection">Follow the full tutorial &rarr;</router-link>
      </p>
    </div>

    <div class="home-sections">
      <router-link to="/getting-started/first-connection" class="home-card">
        <h3>FIRST CONNECTION</h3>
        <p>Send your first signal through CLASP in 5 minutes. Publisher, subscriber, router.</p>
      </router-link>

      <router-link to="/sdk/javascript" class="home-card">
        <h3>CLIENT SDKS</h3>
        <p>{{ sdkCount }} SDK guides: JavaScript, Python, Rust, and embedded (ESP32/RP2040).</p>
      </router-link>

      <router-link to="/protocols" class="home-card">
        <h3>PROTOCOL BRIDGES</h3>
        <p>{{ bridgeCount }} bridges: OSC, MIDI, MQTT, Art-Net, DMX, sACN, HTTP, WebSocket.</p>
      </router-link>

      <router-link to="/deployment/relay" class="home-card">
        <h3>DEPLOYMENT</h3>
        <p>Deploy the relay server with Docker. Auth, persistence, federation, rules engine.</p>
      </router-link>

      <router-link to="/auth" class="home-card">
        <h3>AUTH &amp; SECURITY</h3>
        <p>3 token types: CPSK login, Ed25519 capability delegation, entity registry.</p>
      </router-link>

      <router-link to="/reference" class="home-card">
        <h3>REFERENCE</h3>
        <p>{{ refCount }} pages: CLI flags, wire protocol, config schemas, full SDK APIs.</p>
      </router-link>
    </div>

    <div class="home-quickstart">
      <h2>WHAT CLASP PROVIDES</h2>
      <div class="home-sections" style="margin-bottom: 0;">
        <div>
          <p><strong>Protocol bridges</strong> &mdash; OSC, MIDI, MQTT, DMX, Art-Net, sACN, HTTP, WebSocket</p>
          <p><strong>Semantic signals</strong> &mdash; Param, Event, Stream, Gesture, Timeline</p>
          <p><strong>State management</strong> &mdash; Revisions, conflict resolution, late-joiner sync</p>
          <p><strong>3 SDKs + embedded</strong> &mdash; JavaScript, Python, Rust, ESP32/RP2040</p>
        </div>
        <div>
          <p><strong>Auth</strong> &mdash; CPSK tokens, Ed25519 capability delegation, entity registry</p>
          <p><strong>Server features</strong> &mdash; Rules engine, app config, journal persistence</p>
          <p><strong>Federation</strong> &mdash; Hub-leaf multi-site state sync</p>
          <p><strong>Discovery</strong> &mdash; mDNS, UDP broadcast, rendezvous server</p>
        </div>
      </div>
    </div>

    <div class="home-quickstart">
      <h2>CHOOSE YOUR PATH</h2>
      <div class="home-sections" style="margin-bottom: 0;">
        <div>
          <p><strong>Web/desktop developer</strong><br/>
            <router-link to="/sdk/javascript">SDK guide</router-link> &rarr;
            <router-link to="/core/state">State</router-link> &rarr;
            <router-link to="/auth">Auth</router-link>
          </p>
          <p><strong>Creative/live performance</strong><br/>
            <router-link to="/getting-started/first-connection">First Connection</router-link> &rarr;
            <router-link to="/protocols">Bridges</router-link> &rarr;
            <router-link to="/deployment">Deploy</router-link>
          </p>
        </div>
        <div>
          <p><strong>IoT/embedded</strong><br/>
            <router-link to="/sdk/embedded">Embedded SDK</router-link> &rarr;
            <router-link to="/server/discovery">Discovery</router-link> &rarr;
            <router-link to="/server/federation">Federation</router-link>
          </p>
          <p><strong>DevOps</strong><br/>
            <router-link to="/deployment/relay">Relay Server</router-link> &rarr;
            <router-link to="/deployment/docker">Docker</router-link> &rarr;
            <router-link to="/deployment/production-checklist">Checklist</router-link>
          </p>
        </div>
      </div>
    </div>
  </div>
</template>
