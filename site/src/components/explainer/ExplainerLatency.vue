<script setup>
import { ref } from 'vue'
import { useScrollAnimation } from '../../composables/useScrollAnimation.js'

const sectionRef = ref(null)
useScrollAnimation(sectionRef)

const activeTab = ref('latency')

const tabs = [
  { id: 'latency', label: 'Latency' },
  { id: 'throughput', label: 'Throughput' },
  { id: 'bandwidth', label: 'Bandwidth' },
  { id: 'compare', label: 'Comparison' }
]

const latencyStats = [
  { label: 'SET', p50: '35µs', p99: '125µs', detail: 'Single param write round-trip' },
  { label: 'Single-hop', p50: '48µs', p99: '198µs', detail: 'Client → Router → Client' },
  { label: 'Fanout-10', p50: '0.4ms', p99: '1.2ms', detail: 'One write, 10 subscribers notified' },
  { label: 'Fanout-100', p50: '3.6ms', p99: '7.2ms', detail: 'One write, 100 subscribers notified' },
  { label: 'Wildcard', p50: '58µs', p99: '245µs', detail: 'Pattern-matched subscription delivery' },
  { label: 'Discovery', p50: '1.9ms', p99: '—', detail: 'Device discovery (p95)' }
]

const throughputStats = [
  { label: 'Encode', clasp: '8M msg/s', osc: '4.5M msg/s', mqtt: '11.4M msg/s' },
  { label: 'Decode', clasp: '11M msg/s', osc: '5.7M msg/s', mqtt: '11.4M msg/s' },
  { label: 'Discovery reg.', clasp: '5,593 dev/s', osc: '—', mqtt: '—' },
  { label: 'Concurrent disc.', clasp: '2,290 disc/s', osc: '—', mqtt: '—' }
]

const bandwidthStats = [
  { scenario: '120Hz touchscreen', sent: 122, received: 6, reduction: '95.1%' },
  { scenario: '240Hz pen input', sent: 242, received: 3, reduction: '98.8%' },
  { scenario: 'Fan-out 1→10 subs', sent: 1220, received: 30, reduction: '97.5%' },
  { scenario: 'Multitouch (10 fingers)', sent: 620, received: 121, reduction: '80.5%' }
]

const comparison = {
  headers: ['Feature', 'CLASP', 'OSC', 'MQTT'],
  rows: [
    { feature: 'Message size (SET float)', values: ['31 bytes', '24 bytes', '19 bytes'] },
    { feature: 'Encode speed', values: ['8M msg/s', '4.5M msg/s', '11.4M msg/s'] },
    { feature: 'Decode speed', values: ['11M msg/s', '5.7M msg/s', '11.4M msg/s'] },
    { feature: 'State synchronization', values: ['Yes', 'No', 'No'] },
    { feature: 'Late-joiner support', values: ['Yes', 'No', 'Yes'] },
    { feature: 'Typed signals', values: ['5 types', 'No', 'No'] },
    { feature: 'QoS levels', values: ['3', '0', '3'] },
    { feature: 'Token auth + scopes', values: ['Yes', 'No', 'Yes'] },
    { feature: 'Multi-protocol bridging', values: ['Yes', 'No', 'No'] },
    { feature: 'Clock sync', values: ['Yes', 'Yes', 'No'] },
    { feature: 'Wildcard subscriptions', values: ['Yes', 'No', 'Yes'] },
  ]
}
</script>

<template>
  <section class="explainer-section latency-section" ref="sectionRef">
    <div class="explainer-inner">
      <h2 class="fade-in">Performance</h2>
      <p class="subtitle fade-in">
        Measured with the clasp-e2e test suite. All numbers from real benchmarks, not estimates.
      </p>

      <div class="perf-tabs fade-in">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          class="perf-tab"
          :class="{ active: activeTab === tab.id }"
          @click="activeTab = tab.id"
        >
          {{ tab.label }}
        </button>
      </div>

      <!-- Latency tab -->
      <div v-show="activeTab === 'latency'" class="tab-content">
        <div class="stat-grid">
          <div v-for="stat in latencyStats" :key="stat.label" class="stat-card stagger">
            <div class="stat-value">{{ stat.p50 }}</div>
            <div class="stat-label">{{ stat.label }}</div>
            <div class="stat-detail">{{ stat.detail }}</div>
            <div class="stat-meta">
              <span class="stat-percentile">p50</span>
              <span v-if="stat.p99 !== '—'" class="stat-p99">p99: {{ stat.p99 }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Throughput tab -->
      <div v-show="activeTab === 'throughput'" class="tab-content">
        <div class="throughput-table">
          <div class="table-header">
            <div class="th">Operation</div>
            <div class="th">CLASP</div>
            <div class="th">OSC</div>
            <div class="th">MQTT</div>
          </div>
          <div v-for="row in throughputStats" :key="row.label" class="table-row stagger">
            <div class="td label-cell">{{ row.label }}</div>
            <div class="td mono-cell">{{ row.clasp }}</div>
            <div class="td mono-cell">{{ row.osc }}</div>
            <div class="td mono-cell">{{ row.mqtt }}</div>
          </div>
        </div>
      </div>

      <!-- Bandwidth tab -->
      <div v-show="activeTab === 'bandwidth'" class="tab-content">
        <p class="tab-intro">Gesture coalescing reduces redundant high-frequency input data before it hits the network.</p>
        <div class="throughput-table">
          <div class="table-header">
            <div class="th">Scenario</div>
            <div class="th">Sent</div>
            <div class="th">Received</div>
            <div class="th">Reduction</div>
          </div>
          <div v-for="row in bandwidthStats" :key="row.scenario" class="table-row stagger">
            <div class="td label-cell">{{ row.scenario }}</div>
            <div class="td mono-cell">{{ row.sent }}</div>
            <div class="td mono-cell">{{ row.received }}</div>
            <div class="td mono-cell accent">{{ row.reduction }}</div>
          </div>
        </div>
      </div>

      <!-- Comparison tab -->
      <div v-show="activeTab === 'compare'" class="tab-content">
        <p class="tab-intro">Feature and performance comparison across protocols. Each has different trade-offs.</p>
        <div class="compare-table">
          <div class="table-header four-col">
            <div v-for="h in comparison.headers" :key="h" class="th">{{ h }}</div>
          </div>
          <div v-for="row in comparison.rows" :key="row.feature" class="table-row four-col stagger">
            <div class="td label-cell">{{ row.feature }}</div>
            <div
              v-for="(val, i) in row.values"
              :key="i"
              class="td mono-cell"
              :class="{ 'val-yes': val === 'Yes', 'val-no': val === 'No' }"
            >
              {{ val }}
            </div>
          </div>
        </div>
      </div>

      <p class="caveat fade-in">
        Measured on localhost, M-series Mac, n=10,000 samples. Real-world network latency adds to these numbers.<br>
        <code class="bench-cmd">cargo run --release -p clasp-e2e --bin latency_benchmarks</code>
      </p>
    </div>
  </section>
</template>

<style scoped>
.latency-section {
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
  max-width: 600px;
  margin: 0 auto 2rem;
  font-size: 1.05rem;
  line-height: 1.6;
  opacity: 0.7;
}

/* Tabs */
.perf-tabs {
  display: flex;
  justify-content: center;
  gap: 0.35rem;
  margin-bottom: 2rem;
  flex-wrap: wrap;
}

.perf-tab {
  background: transparent;
  border: 1px solid rgba(255,255,255,0.2);
  color: rgba(255,255,255,0.6);
  padding: 0.5rem 1.2rem;
  font-family: 'Space Mono', monospace;
  font-size: 0.8rem;
  letter-spacing: 0.08em;
  cursor: pointer;
  transition: all 0.12s ease;
}

.perf-tab:hover {
  border-color: rgba(255,255,255,0.5);
  color: #fff;
}

.perf-tab.active {
  background: rgba(255,255,255,0.95);
  border-color: rgba(255,255,255,0.95);
  color: #1a1a1a;
}

.tab-content {
  max-width: 900px;
  margin: 0 auto;
}

.tab-intro {
  text-align: center;
  font-size: 0.9rem;
  opacity: 0.6;
  margin-bottom: 1.5rem;
  line-height: 1.5;
}

/* Latency stat grid */
.stat-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 1rem;
}

.stat-card {
  padding: 1.5rem;
  border: 1px solid rgba(255,255,255,0.12);
  background: rgba(255,255,255,0.03);
  text-align: center;
  transition: border-color 0.15s ease;
  position: relative;
}

.stat-card:hover {
  border-color: var(--accent);
}

.stat-value {
  font-family: 'Archivo Black', sans-serif;
  font-size: 2rem;
  color: var(--accent);
  margin-bottom: 0.4rem;
  line-height: 1;
}

.stat-label {
  font-family: 'Archivo Black', sans-serif;
  text-transform: uppercase;
  letter-spacing: 0.12em;
  font-size: 0.8rem;
  margin-bottom: 0.35rem;
}

.stat-detail {
  font-size: 0.75rem;
  opacity: 0.5;
  line-height: 1.3;
}

.stat-meta {
  display: flex;
  justify-content: space-between;
  margin-top: 0.6rem;
  padding-top: 0.5rem;
  border-top: 1px solid rgba(255,255,255,0.06);
}

.stat-percentile {
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.6rem;
  opacity: 0.35;
  text-transform: uppercase;
  letter-spacing: 0.1em;
}

.stat-p99 {
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.6rem;
  opacity: 0.35;
}

/* Tables (shared) */
.throughput-table,
.compare-table {
  width: 100%;
  border: 1px solid rgba(255,255,255,0.12);
}

.table-header {
  display: grid;
  grid-template-columns: 1.4fr 1fr 1fr 1fr;
  background: rgba(255,255,255,0.06);
  border-bottom: 1px solid rgba(255,255,255,0.12);
}

.table-header.four-col {
  grid-template-columns: 1.8fr 1fr 1fr 1fr;
}

.th {
  font-family: 'Archivo Black', sans-serif;
  text-transform: uppercase;
  font-size: 0.65rem;
  letter-spacing: 0.12em;
  padding: 0.7rem 0.8rem;
  opacity: 0.6;
}

.table-row {
  display: grid;
  grid-template-columns: 1.4fr 1fr 1fr 1fr;
  border-bottom: 1px solid rgba(255,255,255,0.06);
  transition: background 0.1s ease;
}

.table-row.four-col {
  grid-template-columns: 1.8fr 1fr 1fr 1fr;
}

.table-row:last-child {
  border-bottom: none;
}

.table-row:hover {
  background: rgba(255,255,255,0.03);
}

.td {
  padding: 0.6rem 0.8rem;
  font-size: 0.8rem;
  display: flex;
  align-items: center;
}

.label-cell {
  font-weight: 600;
  font-size: 0.78rem;
}

.mono-cell {
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.75rem;
  opacity: 0.8;
}

.mono-cell.accent {
  color: var(--accent);
  opacity: 1;
  font-weight: 700;
}

.val-yes {
  color: #22c55e;
  opacity: 1;
}

.val-no {
  opacity: 0.3;
}

/* Caveat */
.caveat {
  text-align: center;
  font-size: 0.8rem;
  opacity: 0.4;
  margin-top: 2rem;
  font-style: italic;
  line-height: 1.6;
}

.bench-cmd {
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.7rem;
  background: rgba(255,255,255,0.06);
  padding: 0.15rem 0.5rem;
  margin-top: 0.3rem;
  display: inline-block;
  opacity: 0.7;
}

/* Mobile */
@media (max-width: 768px) {
  .stat-grid {
    grid-template-columns: repeat(2, 1fr);
    gap: 0.75rem;
  }

  .stat-value {
    font-size: 1.5rem;
  }

  .stat-card {
    padding: 1rem;
  }

  .table-header,
  .table-row {
    grid-template-columns: 1.2fr 1fr 1fr 1fr;
  }

  .table-header.four-col,
  .table-row.four-col {
    grid-template-columns: 1.4fr 1fr 1fr 1fr;
  }

  .th {
    font-size: 0.55rem;
    padding: 0.5rem 0.4rem;
  }

  .td {
    font-size: 0.7rem;
    padding: 0.5rem 0.4rem;
  }

  .mono-cell {
    font-size: 0.65rem;
  }

  .label-cell {
    font-size: 0.68rem;
  }

  .perf-tab {
    padding: 0.4rem 0.8rem;
    font-size: 0.7rem;
  }
}

@media (max-width: 480px) {
  .stat-grid {
    grid-template-columns: 1fr 1fr;
  }

  .table-header,
  .table-row,
  .table-header.four-col,
  .table-row.four-col {
    grid-template-columns: 1.2fr 0.8fr 0.8fr 0.8fr;
  }

  .th {
    font-size: 0.5rem;
    padding: 0.4rem 0.3rem;
    letter-spacing: 0.05em;
  }

  .td {
    font-size: 0.6rem;
    padding: 0.4rem 0.3rem;
  }

  .mono-cell {
    font-size: 0.55rem;
  }

  .label-cell {
    font-size: 0.58rem;
  }

  .bench-cmd {
    font-size: 0.55rem;
    word-break: break-all;
  }
}
</style>
