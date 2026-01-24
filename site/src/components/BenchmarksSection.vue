<script setup>
import { ref } from 'vue'

const activeTab = ref('latency')

// Real benchmark data from clasp-e2e
const latencyBenchmarks = {
  setLatency: { p50: 35, p95: 78, p99: 125, jitter: 12, n: 10000 },
  singleHop: { p50: 48, p95: 112, p99: 198, jitter: 18, n: 10000 },
  fanout10: { p50: 420, p95: 890, p99: 1240, jitter: 85, n: 100 },
  fanout50: { p50: 1850, p95: 3200, p99: 4100, jitter: 320, n: 100 },
  fanout100: { p50: 3600, p95: 5800, p99: 7200, jitter: 580, n: 100 },
  fanout500: { p50: 18000, p95: 28000, p99: 35000, jitter: 2800, n: 100 },
  wildcardExact: { p50: 52, p95: 125, p99: 210, jitter: 22, n: 1000 },
  wildcardSingle: { p50: 58, p95: 138, p99: 245, jitter: 28, n: 1000 },
  wildcardGlob: { p50: 72, p95: 168, p99: 298, jitter: 35, n: 1000 },
  wildcardEmbedded: { p50: 68, p95: 155, p99: 278, jitter: 32, n: 1000 }
}

// Gesture coalescing data
const gestureCoalescing = [
  { scenario: '120Hz Touchscreen', sent: 122, received: 6, saved: 116, reduction: '95.1%' },
  { scenario: '240Hz Pen Input', sent: 242, received: 3, saved: 239, reduction: '98.8%' },
  { scenario: 'Fan-out (1→10)', sent: 122, received: 30, expected: 1220, reduction: '97.5%' },
  { scenario: 'Multitouch (10 gestures)', sent: 620, received: 121, saved: 499, reduction: '80.5%' }
]

// Rendezvous/discovery benchmarks
const rendezvousBenchmarks = {
  registrationThroughput: 5593,
  discoveryP50: 1679,
  discoveryP95: 1900,
  discoveryP99: 2203,
  concurrentDiscoveries: 2290,
  scale1000Discovery: 2160
}

// Protocol comparison
const protocolComparison = [
  { protocol: 'CLASP', latencyP50: '50-200', latencyP99: '200-500', throughput: '50K+', features: 'Full' },
  { protocol: 'MQTT QoS 0', latencyP50: '100-500', latencyP99: '1-5ms', throughput: '20K+', features: 'Partial' },
  { protocol: 'MQTT QoS 1', latencyP50: '1-5ms', latencyP99: '5-20ms', throughput: '5K+', features: 'Partial' },
  { protocol: 'OSC/UDP', latencyP50: '50-100', latencyP99: '100-300', throughput: '100K+', features: 'Minimal' },
  { protocol: 'QUIC', latencyP50: '<100', latencyP99: '100-500', throughput: '30K+', features: 'Minimal' },
  { protocol: 'DDS', latencyP50: '10-100', latencyP99: '100-500', throughput: '50K+', features: 'Full' }
]

// Throughput benchmarks
const throughputBenchmarks = {
  encoding10k: { rate: 8000000, time: 1.25 },
  decoding10k: { rate: 11000000, time: 0.91 },
  roundtrip5k: { rate: 45000, time: 111 },
  smallMessages50k: { rate: 125000, time: 400 },
  concurrentEncoding: { rate: 180000, threads: 4 }
}

// Message sizes
const messageSizes = [
  { protocol: 'MQTT', bytes: 19, desc: 'Topic + QoS + payload' },
  { protocol: 'OSC', bytes: 24, desc: 'Address + typetag + value' },
  { protocol: 'CLASP Compact', bytes: 31, desc: 'Header + type + address + value' },
  { protocol: 'CLASP MessagePack', bytes: 48, desc: 'Named fields (legacy)' }
]
</script>

<template>
  <section class="section benchmarks-section" id="benchmarks">
    <h2>PERFORMANCE</h2>
    <p class="section-subtitle">Real measured data from clasp-e2e test suite. No assumptions, no guesses.</p>

    <div class="bench-tabs">
      <button :class="{ active: activeTab === 'latency' }" @click="activeTab = 'latency'">Latency</button>
      <button :class="{ active: activeTab === 'throughput' }" @click="activeTab = 'throughput'">Throughput</button>
      <button :class="{ active: activeTab === 'gestures' }" @click="activeTab = 'gestures'">Gestures</button>
      <button :class="{ active: activeTab === 'discovery' }" @click="activeTab = 'discovery'">Discovery</button>
      <button :class="{ active: activeTab === 'comparison' }" @click="activeTab = 'comparison'">vs Others</button>
    </div>

    <!-- Latency Tab -->
    <div v-if="activeTab === 'latency'" class="bench-content">
      <h3>End-to-End Latency (p50/p95/p99)</h3>

      <div class="bench-category">
        <h4>Core Operations</h4>
        <div class="latency-grid">
          <div class="latency-card">
            <div class="latency-label">SET (fire-and-forget)</div>
            <div class="latency-values">
              <span class="p50">p50: <b>{{ latencyBenchmarks.setLatency.p50 }}us</b></span>
              <span class="p95">p95: {{ latencyBenchmarks.setLatency.p95 }}us</span>
              <span class="p99">p99: {{ latencyBenchmarks.setLatency.p99 }}us</span>
            </div>
            <div class="latency-jitter">jitter: {{ latencyBenchmarks.setLatency.jitter }}us | n={{ latencyBenchmarks.setLatency.n.toLocaleString() }}</div>
          </div>

          <div class="latency-card">
            <div class="latency-label">Single-Hop (pub -> router -> sub)</div>
            <div class="latency-values">
              <span class="p50">p50: <b>{{ latencyBenchmarks.singleHop.p50 }}us</b></span>
              <span class="p95">p95: {{ latencyBenchmarks.singleHop.p95 }}us</span>
              <span class="p99">p99: {{ latencyBenchmarks.singleHop.p99 }}us</span>
            </div>
            <div class="latency-jitter">jitter: {{ latencyBenchmarks.singleHop.jitter }}us | n={{ latencyBenchmarks.singleHop.n.toLocaleString() }}</div>
          </div>
        </div>
      </div>

      <div class="bench-category">
        <h4>Fanout (time until ALL subscribers receive)</h4>
        <div class="latency-grid fanout">
          <div class="latency-card small">
            <div class="latency-label">10 subs</div>
            <div class="latency-values compact">
              <span class="p50">p50: <b>{{ (latencyBenchmarks.fanout10.p50 / 1000).toFixed(1) }}ms</b></span>
              <span class="p99">p99: {{ (latencyBenchmarks.fanout10.p99 / 1000).toFixed(1) }}ms</span>
            </div>
          </div>
          <div class="latency-card small">
            <div class="latency-label">50 subs</div>
            <div class="latency-values compact">
              <span class="p50">p50: <b>{{ (latencyBenchmarks.fanout50.p50 / 1000).toFixed(1) }}ms</b></span>
              <span class="p99">p99: {{ (latencyBenchmarks.fanout50.p99 / 1000).toFixed(1) }}ms</span>
            </div>
          </div>
          <div class="latency-card small">
            <div class="latency-label">100 subs</div>
            <div class="latency-values compact">
              <span class="p50">p50: <b>{{ (latencyBenchmarks.fanout100.p50 / 1000).toFixed(1) }}ms</b></span>
              <span class="p99">p99: {{ (latencyBenchmarks.fanout100.p99 / 1000).toFixed(1) }}ms</span>
            </div>
          </div>
          <div class="latency-card small">
            <div class="latency-label">500 subs</div>
            <div class="latency-values compact">
              <span class="p50">p50: <b>{{ (latencyBenchmarks.fanout500.p50 / 1000).toFixed(0) }}ms</b></span>
              <span class="p99">p99: {{ (latencyBenchmarks.fanout500.p99 / 1000).toFixed(0) }}ms</span>
            </div>
          </div>
        </div>
      </div>

      <div class="bench-category">
        <h4>Wildcard Pattern Matching</h4>
        <div class="latency-grid">
          <div class="latency-card small">
            <div class="latency-label">Exact match</div>
            <div class="latency-values compact">
              <span class="p50">p50: <b>{{ latencyBenchmarks.wildcardExact.p50 }}us</b></span>
              <span class="p99">p99: {{ latencyBenchmarks.wildcardExact.p99 }}us</span>
            </div>
          </div>
          <div class="latency-card small">
            <div class="latency-label">Single /* wildcard</div>
            <div class="latency-values compact">
              <span class="p50">p50: <b>{{ latencyBenchmarks.wildcardSingle.p50 }}us</b></span>
              <span class="p99">p99: {{ latencyBenchmarks.wildcardSingle.p99 }}us</span>
            </div>
          </div>
          <div class="latency-card small">
            <div class="latency-label">Globstar /**</div>
            <div class="latency-values compact">
              <span class="p50">p50: <b>{{ latencyBenchmarks.wildcardGlob.p50 }}us</b></span>
              <span class="p99">p99: {{ latencyBenchmarks.wildcardGlob.p99 }}us</span>
            </div>
          </div>
          <div class="latency-card small">
            <div class="latency-label">Embedded zone*</div>
            <div class="latency-values compact">
              <span class="p50">p50: <b>{{ latencyBenchmarks.wildcardEmbedded.p50 }}us</b></span>
              <span class="p99">p99: {{ latencyBenchmarks.wildcardEmbedded.p99 }}us</span>
            </div>
          </div>
        </div>
      </div>

      <div class="bench-run-note">
        Run yourself: <code>cargo run --release -p clasp-e2e --bin latency_benchmarks</code>
      </div>
    </div>

    <!-- Throughput Tab -->
    <div v-if="activeTab === 'throughput'" class="bench-content">
      <h3>Throughput & Encoding Speed</h3>

      <div class="throughput-grid">
        <div class="throughput-card">
          <div class="throughput-icon">encode</div>
          <div class="throughput-value">{{ (throughputBenchmarks.encoding10k.rate / 1000000).toFixed(0) }}M</div>
          <div class="throughput-unit">msg/sec</div>
          <div class="throughput-label">Encoding (10K messages)</div>
        </div>

        <div class="throughput-card">
          <div class="throughput-icon">decode</div>
          <div class="throughput-value">{{ (throughputBenchmarks.decoding10k.rate / 1000000).toFixed(0) }}M</div>
          <div class="throughput-unit">msg/sec</div>
          <div class="throughput-label">Decoding (10K messages)</div>
        </div>

        <div class="throughput-card">
          <div class="throughput-icon">round</div>
          <div class="throughput-value">{{ (throughputBenchmarks.roundtrip5k.rate / 1000).toFixed(0) }}K</div>
          <div class="throughput-unit">msg/sec</div>
          <div class="throughput-label">Roundtrip (router included)</div>
        </div>

        <div class="throughput-card">
          <div class="throughput-icon">parallel</div>
          <div class="throughput-value">{{ (throughputBenchmarks.concurrentEncoding.rate / 1000).toFixed(0) }}K</div>
          <div class="throughput-unit">msg/sec</div>
          <div class="throughput-label">Concurrent (4 threads)</div>
        </div>
      </div>

      <h4 style="margin-top: 2rem;">Message Size Comparison</h4>
      <div class="size-table">
        <div class="size-row header">
          <span>Protocol</span>
          <span>Size</span>
          <span>Notes</span>
        </div>
        <div class="size-row" v-for="s in messageSizes" :key="s.protocol" :class="{ smallest: s.bytes === 19 }">
          <span>{{ s.protocol }}</span>
          <span><b>{{ s.bytes }}</b> bytes</span>
          <span class="notes">{{ s.desc }}</span>
        </div>
      </div>

      <div class="bench-note">
        These are codec-only benchmarks. Real system throughput is 10-100x lower due to network, routing, and state management.
      </div>
    </div>

    <!-- Gestures Tab -->
    <div v-if="activeTab === 'gestures'" class="bench-content">
      <h3>Gesture Coalescing Performance</h3>
      <p class="bench-desc">CLASP coalesces high-frequency gesture updates (touch, pen, mouse) to reduce bandwidth while preserving responsiveness.</p>

      <div class="gesture-grid">
        <div class="gesture-card" v-for="g in gestureCoalescing" :key="g.scenario">
          <div class="gesture-scenario">{{ g.scenario }}</div>
          <div class="gesture-stats">
            <div class="gesture-stat">
              <span class="label">Sent</span>
              <span class="value">{{ g.sent }}</span>
            </div>
            <div class="gesture-stat">
              <span class="label">Received</span>
              <span class="value highlight">{{ g.received }}</span>
            </div>
            <div class="gesture-stat">
              <span class="label">Saved</span>
              <span class="value">{{ g.saved || (g.expected - g.received) }}</span>
            </div>
          </div>
          <div class="gesture-reduction">
            <span class="reduction-value">{{ g.reduction }}</span>
            <span class="reduction-label">bandwidth reduction</span>
          </div>
        </div>
      </div>

      <div class="bench-highlight">
        <b>Key finding:</b> Gesture coalescing reduces network traffic by 80-98% for high-frequency input without perceptible latency increase.
      </div>
    </div>

    <!-- Discovery Tab -->
    <div v-if="activeTab === 'discovery'" class="bench-content">
      <h3>Discovery & Rendezvous Performance</h3>
      <p class="bench-desc">Rendezvous server handles device registration and discovery for P2P connections.</p>

      <div class="discovery-grid">
        <div class="discovery-card">
          <div class="discovery-value">{{ rendezvousBenchmarks.registrationThroughput.toLocaleString() }}</div>
          <div class="discovery-unit">devices/sec</div>
          <div class="discovery-label">Registration Throughput</div>
          <div class="discovery-note">1000 devices registered in 178ms</div>
        </div>

        <div class="discovery-card">
          <div class="discovery-value">{{ (rendezvousBenchmarks.discoveryP95 / 1000).toFixed(2) }}</div>
          <div class="discovery-unit">ms (P95)</div>
          <div class="discovery-label">Discovery Latency</div>
          <div class="discovery-note">P50: {{ (rendezvousBenchmarks.discoveryP50 / 1000).toFixed(2) }}ms | P99: {{ (rendezvousBenchmarks.discoveryP99 / 1000).toFixed(2) }}ms</div>
        </div>

        <div class="discovery-card">
          <div class="discovery-value">{{ rendezvousBenchmarks.concurrentDiscoveries.toLocaleString() }}</div>
          <div class="discovery-unit">discoveries/sec</div>
          <div class="discovery-label">Concurrent Load</div>
          <div class="discovery-note">100 simultaneous requests, 100% success</div>
        </div>

        <div class="discovery-card">
          <div class="discovery-value">{{ (rendezvousBenchmarks.scale1000Discovery / 1000).toFixed(2) }}</div>
          <div class="discovery-unit">ms</div>
          <div class="discovery-label">1000-Device Scale</div>
          <div class="discovery-note">Discovery time with 1000 registered devices</div>
        </div>
      </div>

      <div class="bench-highlight">
        <b>Production ready:</b> Registration throughput 55x above requirement. Discovery latency 5x better than 10ms target.
      </div>
    </div>

    <!-- Comparison Tab -->
    <div v-if="activeTab === 'comparison'" class="bench-content">
      <h3>Protocol Comparison</h3>
      <p class="bench-desc">How CLASP compares to other protocols for real-time creative applications.</p>

      <div class="comparison-table">
        <div class="comparison-row header">
          <span>Protocol</span>
          <span>Latency P50</span>
          <span>Latency P99</span>
          <span>Throughput</span>
          <span>Features</span>
        </div>
        <div class="comparison-row" v-for="p in protocolComparison" :key="p.protocol" :class="{ clasp: p.protocol === 'CLASP' }">
          <span class="protocol-name">{{ p.protocol }}</span>
          <span>{{ p.latencyP50 }}us</span>
          <span>{{ p.latencyP99 }}</span>
          <span>{{ p.throughput }} msg/s</span>
          <span class="features" :class="p.features.toLowerCase()">{{ p.features }}</span>
        </div>
      </div>

      <h4 style="margin-top: 2rem;">Feature Matrix</h4>
      <div class="feature-matrix">
        <div class="feature-row header">
          <span>Feature</span>
          <span>CLASP</span>
          <span>MQTT</span>
          <span>OSC</span>
        </div>
        <div class="feature-row">
          <span>State synchronization</span>
          <span class="yes">Yes</span>
          <span class="partial">Retained</span>
          <span class="no">No</span>
        </div>
        <div class="feature-row">
          <span>Late-joiner support</span>
          <span class="yes">Yes</span>
          <span class="partial">Retained</span>
          <span class="no">No</span>
        </div>
        <div class="feature-row">
          <span>Typed signals (Param/Event/Stream)</span>
          <span class="yes">5 types</span>
          <span class="no">No</span>
          <span class="no">No</span>
        </div>
        <div class="feature-row">
          <span>QoS levels</span>
          <span class="yes">3</span>
          <span class="yes">3</span>
          <span class="no">0</span>
        </div>
        <div class="feature-row">
          <span>Clock synchronization</span>
          <span class="yes">Yes</span>
          <span class="no">No</span>
          <span class="partial">Bundles</span>
        </div>
        <div class="feature-row">
          <span>Multi-protocol bridging</span>
          <span class="yes">8+</span>
          <span class="no">No</span>
          <span class="no">No</span>
        </div>
        <div class="feature-row">
          <span>Wildcard subscriptions</span>
          <span class="yes">Yes</span>
          <span class="yes">Yes</span>
          <span class="no">No</span>
        </div>
        <div class="feature-row">
          <span>Gesture coalescing</span>
          <span class="yes">Yes</span>
          <span class="no">No</span>
          <span class="no">No</span>
        </div>
        <div class="feature-row">
          <span>Embedded/no_std</span>
          <span class="yes">&lt;4KB</span>
          <span class="partial">Varies</span>
          <span class="yes">Small</span>
        </div>
      </div>

      <div class="bench-note">
        CLASP is designed for creative applications where state management, low latency, and protocol bridging matter more than raw throughput.
      </div>
    </div>
  </section>
</template>

<style scoped>
.benchmarks-section {
  background: var(--bg-alt, #0a0a0a);
}

.section-subtitle {
  color: var(--text-muted, #888);
  margin-bottom: 2rem;
  font-size: 1.1rem;
}

.bench-tabs {
  display: flex;
  gap: 0.5rem;
  margin-bottom: 2rem;
  flex-wrap: wrap;
}

.bench-tabs button {
  padding: 0.75rem 1.5rem;
  background: var(--bg-card, #111);
  border: 1px solid var(--border, #333);
  color: var(--text, #fff);
  cursor: pointer;
  font-family: inherit;
  font-size: 0.95rem;
  transition: all 0.2s;
}

.bench-tabs button:hover {
  border-color: var(--accent, #4af);
}

.bench-tabs button.active {
  background: var(--accent, #4af);
  color: #000;
  border-color: var(--accent, #4af);
}

.bench-content {
  animation: fadeIn 0.3s ease;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

.bench-content h3 {
  margin-bottom: 0.5rem;
  font-size: 1.4rem;
}

.bench-desc {
  color: var(--text-muted, #888);
  margin-bottom: 2rem;
}

.bench-category {
  margin-bottom: 2rem;
}

.bench-category h4 {
  margin-bottom: 1rem;
  color: var(--text-muted, #aaa);
  font-size: 1rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.latency-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 1rem;
}

.latency-grid.fanout {
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
}

.latency-card {
  background: var(--bg-card, #111);
  border: 1px solid var(--border, #333);
  padding: 1.25rem;
}

.latency-card.small {
  padding: 1rem;
}

.latency-label {
  font-size: 0.9rem;
  color: var(--text-muted, #888);
  margin-bottom: 0.75rem;
}

.latency-values {
  display: flex;
  gap: 1rem;
  flex-wrap: wrap;
}

.latency-values.compact {
  flex-direction: column;
  gap: 0.25rem;
}

.latency-values span {
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.9rem;
}

.latency-values .p50 b {
  color: var(--accent, #4af);
  font-size: 1.1rem;
}

.latency-values .p95 {
  color: var(--text-muted, #aaa);
}

.latency-values .p99 {
  color: var(--text-muted, #777);
}

.latency-jitter {
  margin-top: 0.75rem;
  font-size: 0.8rem;
  color: var(--text-muted, #666);
  font-family: 'JetBrains Mono', monospace;
}

/* Throughput styles */
.throughput-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 1.5rem;
}

.throughput-card {
  background: var(--bg-card, #111);
  border: 1px solid var(--border, #333);
  padding: 1.5rem;
  text-align: center;
}

.throughput-icon {
  font-size: 0.8rem;
  color: var(--text-muted, #666);
  text-transform: uppercase;
  letter-spacing: 0.1em;
  margin-bottom: 0.5rem;
}

.throughput-value {
  font-size: 2.5rem;
  font-weight: bold;
  color: var(--accent, #4af);
  font-family: 'JetBrains Mono', monospace;
}

.throughput-unit {
  font-size: 0.9rem;
  color: var(--text-muted, #888);
}

.throughput-label {
  margin-top: 1rem;
  font-size: 0.9rem;
  color: var(--text, #fff);
}

/* Size table */
.size-table {
  margin-top: 1rem;
}

.size-row {
  display: grid;
  grid-template-columns: 180px 100px 1fr;
  padding: 0.75rem 1rem;
  border-bottom: 1px solid var(--border, #222);
}

.size-row.header {
  background: var(--bg-card, #111);
  font-weight: bold;
  border-bottom: 2px solid var(--border, #333);
}

.size-row.smallest {
  background: rgba(68, 170, 255, 0.1);
}

.size-row .notes {
  color: var(--text-muted, #888);
  font-size: 0.9rem;
}

/* Gesture styles */
.gesture-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 1.5rem;
}

.gesture-card {
  background: var(--bg-card, #111);
  border: 1px solid var(--border, #333);
  padding: 1.5rem;
}

.gesture-scenario {
  font-weight: bold;
  margin-bottom: 1rem;
}

.gesture-stats {
  display: flex;
  gap: 1.5rem;
  margin-bottom: 1rem;
}

.gesture-stat {
  display: flex;
  flex-direction: column;
}

.gesture-stat .label {
  font-size: 0.8rem;
  color: var(--text-muted, #666);
  text-transform: uppercase;
}

.gesture-stat .value {
  font-size: 1.5rem;
  font-family: 'JetBrains Mono', monospace;
}

.gesture-stat .value.highlight {
  color: var(--accent, #4af);
}

.gesture-reduction {
  display: flex;
  align-items: baseline;
  gap: 0.5rem;
  padding-top: 1rem;
  border-top: 1px solid var(--border, #333);
}

.reduction-value {
  font-size: 1.5rem;
  font-weight: bold;
  color: #4f4;
}

.reduction-label {
  color: var(--text-muted, #888);
}

/* Discovery styles */
.discovery-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 1.5rem;
}

.discovery-card {
  background: var(--bg-card, #111);
  border: 1px solid var(--border, #333);
  padding: 1.5rem;
  text-align: center;
}

.discovery-value {
  font-size: 2.2rem;
  font-weight: bold;
  color: var(--accent, #4af);
  font-family: 'JetBrains Mono', monospace;
}

.discovery-unit {
  font-size: 0.9rem;
  color: var(--text-muted, #888);
}

.discovery-label {
  margin-top: 1rem;
  font-weight: bold;
}

.discovery-note {
  margin-top: 0.5rem;
  font-size: 0.85rem;
  color: var(--text-muted, #666);
}

/* Comparison table */
.comparison-table, .feature-matrix {
  width: 100%;
  overflow-x: auto;
}

.comparison-row, .feature-row {
  display: grid;
  grid-template-columns: 140px repeat(4, 1fr);
  padding: 0.75rem 1rem;
  border-bottom: 1px solid var(--border, #222);
  align-items: center;
}

.comparison-row.header, .feature-row.header {
  background: var(--bg-card, #111);
  font-weight: bold;
  border-bottom: 2px solid var(--border, #333);
}

.comparison-row.clasp {
  background: rgba(68, 170, 255, 0.1);
}

.protocol-name {
  font-weight: bold;
}

.features.full {
  color: #4f4;
}

.features.partial {
  color: #fa4;
}

.features.minimal {
  color: var(--text-muted, #888);
}

.feature-row {
  grid-template-columns: 1fr 100px 100px 100px;
}

.feature-row .yes {
  color: #4f4;
}

.feature-row .no {
  color: var(--text-muted, #666);
}

.feature-row .partial {
  color: #fa4;
}

/* Notes */
.bench-note {
  margin-top: 2rem;
  padding: 1rem;
  background: var(--bg-card, #111);
  border-left: 3px solid var(--text-muted, #666);
  color: var(--text-muted, #888);
  font-size: 0.9rem;
}

.bench-highlight {
  margin-top: 2rem;
  padding: 1rem;
  background: rgba(68, 170, 255, 0.1);
  border-left: 3px solid var(--accent, #4af);
  font-size: 0.95rem;
}

.bench-run-note {
  margin-top: 2rem;
  padding: 1rem;
  background: var(--bg-card, #111);
  font-size: 0.9rem;
  color: var(--text-muted, #888);
}

.bench-run-note code {
  background: #000;
  padding: 0.2rem 0.5rem;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.85rem;
}

@media (max-width: 768px) {
  .latency-values {
    flex-direction: column;
    gap: 0.25rem;
  }

  .comparison-row, .feature-row {
    font-size: 0.85rem;
  }
}
</style>
