<script setup>
import { ref, computed } from 'vue'
import CodeBlock from '../CodeBlock.vue'
import { useScrollAnimation } from '../../composables/useScrollAnimation.js'

const sectionRef = ref(null)
useScrollAnimation(sectionRef)

const activeTab = ref('js')
const activeTransport = ref('ws')

const tabs = [
  { id: 'js', label: 'JavaScript' },
  { id: 'py', label: 'Python' },
  { id: 'rs', label: 'Rust' }
]

const allTransports = [
  { id: 'ws', label: 'WebSocket' },
  { id: 'webrtc', label: 'WebRTC' },
  { id: 'quic', label: 'QUIC' },
  { id: 'tcp', label: 'TCP' },
  { id: 'udp', label: 'UDP' },
  { id: 'serial', label: 'Serial' },
  { id: 'ble', label: 'BLE' }
]

const codeExamples = {
  'js-ws': {
    code: `// Install: npm install @clasp-to/core

import { Clasp } from '@clasp-to/core';

// Connect to a CLASP router
const clasp = new Clasp('ws://localhost:7330');
await clasp.connect();

// Subscribe to addresses (wildcards supported)
const unsubscribe = clasp.on('/lights/*/brightness', (value, address) => {
  console.log(\`\${address} = \${value}\`);
});

// Set a Param (stateful, syncs to all subscribers)
clasp.set('/lights/kitchen/brightness', 0.75);

// Get current value (from cache or server)
const brightness = await clasp.get('/lights/kitchen/brightness');

// Emit an Event (one-shot, no state)
clasp.emit('/cue/fire', { cueId: 'intro', fadeTime: 2.0 });

// Send Stream data (high-rate, fire-and-forget)
clasp.stream('/sensors/accelerometer/x', 0.342);

// Atomic bundle with optional scheduling
clasp.bundle([
  { set: ['/light/1/intensity', 1.0] },
  { set: ['/light/2/intensity', 0.5] },
  { emit: ['/cue/fire', { id: 'intro' }] }
], { at: clasp.time() + 100000 }); // 100ms in the future

// Cleanup
unsubscribe();
clasp.close();`,
    lang: 'javascript'
  },

  'js-webrtc': {
    code: `// Install: npm install @clasp-to/core

import { Clasp } from '@clasp-to/core';

// Connect via WebRTC data channel (P2P with NAT traversal)
const clasp = new Clasp('webrtc://signal.example.com/room/stage1');
await clasp.connect();

// Same CLASP API over peer-to-peer data channels
const unsubscribe = clasp.on('/lights/*/brightness', (value, address) => {
  console.log(\`\${address} = \${value}\`);
});

clasp.set('/lights/kitchen/brightness', 0.75);

clasp.emit('/cue/fire', { cueId: 'intro', fadeTime: 2.0 });

unsubscribe();
clasp.close();`,
    lang: 'javascript'
  },

  'py-ws': {
    code: `# Install: pip install clasp-to

import asyncio
from clasp import ClaspBuilder

async def main():
    # Connect using builder pattern
    client = await (
        ClaspBuilder('ws://localhost:7330')
        .with_name('Python Controller')
        .connect()
    )

    # Subscribe with decorator
    @client.on('/lights/*/brightness')
    def on_brightness(value, address, meta=None):
        print(f"{address} = {value}")

    # Set a Param
    await client.set('/lights/kitchen/brightness', 0.75)

    # Emit an Event
    await client.emit('/cue/fire', {'cueId': 'intro'})

    # Get current value
    brightness = await client.get('/lights/kitchen/brightness')
    print(f"Current: {brightness}")

    # Keep running (processes incoming messages)
    await client.run()

asyncio.run(main())`,
    lang: 'python'
  },

  'rs-ws': {
    code: `// Cargo.toml: clasp-client = "3.0"

use clasp_client::{Clasp, ClaspBuilder};
use clasp_core::Value;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Connect using builder
    let client = ClaspBuilder::new("ws://localhost:7330")
        .name("Rust Controller")
        .connect()
        .await?;

    // Subscribe with pattern matching
    let _unsub = client.subscribe("/lights/**", |value, address| {
        println!("{} = {:?}", address, value);
    }).await?;

    // Set a Param
    client.set("/lights/kitchen/brightness", Value::Float(0.75)).await?;

    // Emit an Event
    client.emit("/cue/fire", Value::Map(
        [("cueId".into(), Value::String("intro".into()))].into()
    )).await?;

    // Scheduled bundle
    let now = client.time();
    client.bundle(vec![
        clasp_core::Message::Set(clasp_core::SetMessage {
            address: "/light/1/intensity".into(),
            value: Value::Float(1.0),
            ..Default::default()
        }),
    ], Some(now + 100_000)).await?;

    // Run until Ctrl-C
    tokio::signal::ctrl_c().await?;
    client.close().await?;
    Ok(())
}`,
    lang: 'rust'
  },

  'rs-webrtc': {
    code: `// Cargo.toml: clasp-client = { version = "3.0", features = ["webrtc"] }

use clasp_client::{ClaspBuilder, WebRtcTransport, WebRtcConfig};
use clasp_core::Value;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Configure WebRTC transport (P2P with NAT traversal)
    let transport = WebRtcTransport::new(
        WebRtcConfig::builder()
            .signaling_url("wss://signal.example.com")
            .room("stage1")
            .build()?,
    );

    let client = ClaspBuilder::with_transport(transport)
        .name("Rust WebRTC Controller")
        .connect()
        .await?;

    let _unsub = client.subscribe("/lights/**", |value, addr| {
        println!("{} = {:?}", addr, value);
    }).await?;

    client.set("/lights/kitchen/brightness", Value::Float(0.75)).await?;

    tokio::signal::ctrl_c().await?;
    client.close().await?;
    Ok(())
}`,
    lang: 'rust'
  },

  'rs-quic': {
    code: `// Cargo.toml: clasp-client = { version = "3.0", features = ["quic"] }

use clasp_client::{ClaspBuilder, QuicTransport, QuicConfig};
use clasp_core::Value;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Configure QUIC transport
    let transport = QuicTransport::new(
        QuicConfig::builder()
            .server_addr("127.0.0.1:7331")
            .server_name("clasp-router")
            .cert_path("certs/ca.pem")
            .build()?,
    );

    // Connect with QUIC
    let client = ClaspBuilder::with_transport(transport)
        .name("Rust QUIC Controller")
        .connect()
        .await?;

    // Same CLASP API — subscribe, set, emit
    let _unsub = client.subscribe("/lights/**", |value, addr| {
        println!("{} = {:?}", addr, value);
    }).await?;

    client.set("/lights/kitchen/brightness", Value::Float(0.75)).await?;

    tokio::signal::ctrl_c().await?;
    client.close().await?;
    Ok(())
}`,
    lang: 'rust'
  },

  'rs-tcp': {
    code: `// Cargo.toml: clasp-client = { version = "3.0", features = ["tcp"] }

use clasp_client::{ClaspBuilder, TcpTransport, TcpConfig};
use clasp_core::Value;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Configure TCP transport
    let transport = TcpTransport::new(
        TcpConfig::builder()
            .server_addr("127.0.0.1:7332")
            .nodelay(true)
            .build()?,
    );

    // Connect with TCP
    let client = ClaspBuilder::with_transport(transport)
        .name("Rust TCP Controller")
        .connect()
        .await?;

    // Same CLASP API — subscribe, set, emit
    let _unsub = client.subscribe("/lights/**", |value, addr| {
        println!("{} = {:?}", addr, value);
    }).await?;

    client.set("/lights/kitchen/brightness", Value::Float(0.75)).await?;

    tokio::signal::ctrl_c().await?;
    client.close().await?;
    Ok(())
}`,
    lang: 'rust'
  },

  'rs-udp': {
    code: `// Cargo.toml: clasp-client = { version = "3.0", features = ["udp"] }

use clasp_client::{ClaspBuilder, UdpTransport, UdpConfig};
use clasp_core::Value;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Configure UDP transport (lightweight, LAN-optimized)
    let transport = UdpTransport::new(
        UdpConfig::builder()
            .server_addr("127.0.0.1:7333")
            .broadcast(false)
            .build()?,
    );

    let client = ClaspBuilder::with_transport(transport)
        .name("Rust UDP Controller")
        .connect()
        .await?;

    // Fire-and-forget — ideal for high-rate sensor streams
    let _unsub = client.subscribe("/sensors/**", |value, addr| {
        println!("{} = {:?}", addr, value);
    }).await?;

    client.stream("/sensors/accelerometer/x", Value::Float(0.342)).await?;

    tokio::signal::ctrl_c().await?;
    client.close().await?;
    Ok(())
}`,
    lang: 'rust'
  },

  'rs-serial': {
    code: `// Cargo.toml: clasp-client = { version = "3.0", features = ["serial"] }

use clasp_client::{ClaspBuilder, SerialTransport, SerialConfig};
use clasp_core::Value;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Configure serial port transport (direct hardware link)
    let transport = SerialTransport::new(
        SerialConfig::builder()
            .port("/dev/ttyUSB0")
            .baud_rate(115200)
            .build()?,
    );

    let client = ClaspBuilder::with_transport(transport)
        .name("Rust Serial Controller")
        .connect()
        .await?;

    // Talk directly to a CLASP-speaking microcontroller
    let _unsub = client.subscribe("/hw/**", |value, addr| {
        println!("{} = {:?}", addr, value);
    }).await?;

    client.set("/hw/led/brightness", Value::Float(1.0)).await?;

    tokio::signal::ctrl_c().await?;
    client.close().await?;
    Ok(())
}`,
    lang: 'rust'
  },

  'rs-ble': {
    code: `// Cargo.toml: clasp-client = { version = "3.0", features = ["ble"] }

use clasp_client::{ClaspBuilder, BleTransport, BleConfig};
use clasp_core::Value;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Configure BLE transport (low-power wireless)
    let transport = BleTransport::new(
        BleConfig::builder()
            .device_name("CLASP-Controller")
            .service_uuid("0000ff00-0000-1000-8000-00805f9b34fb")
            .build()?,
    );

    let client = ClaspBuilder::with_transport(transport)
        .name("Rust BLE Controller")
        .connect()
        .await?;

    // Control IoT devices over Bluetooth
    let _unsub = client.subscribe("/ble/**", |value, addr| {
        println!("{} = {:?}", addr, value);
    }).await?;

    client.set("/ble/light/on", Value::Bool(true)).await?;

    tokio::signal::ctrl_c().await?;
    client.close().await?;
    Ok(())
}`,
    lang: 'rust'
  }
}

// Filter transports to only those available for the active language
const visibleTransports = computed(() => {
  return allTransports.filter(t => `${activeTab.value}-${t.id}` in codeExamples)
})

const activeExample = computed(() => {
  return codeExamples[`${activeTab.value}-${activeTransport.value}`]
})

function selectTransport(id) {
  activeTransport.value = id
}

function selectLang(id) {
  activeTab.value = id
  // If current transport isn't available for this lang, fall back to ws
  if (!(`${id}-${activeTransport.value}` in codeExamples)) {
    activeTransport.value = 'ws'
  }
}
</script>

<template>
  <section class="explainer-section crosslang-section" ref="sectionRef">
    <div class="explainer-inner">
      <h2 class="fade-in">Write in Your Language</h2>
      <p class="subtitle fade-in">
        First-class SDKs for JavaScript, Python, and Rust. Same API shape, native idioms.
      </p>

      <div class="lang-tabs fade-in">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          class="lang-tab"
          :class="{ active: activeTab === tab.id }"
          @click="selectLang(tab.id)"
        >
          {{ tab.label }}
        </button>
      </div>

      <div class="transport-tabs fade-in">
        <button
          v-for="t in visibleTransports"
          :key="t.id"
          class="transport-tab"
          :class="{ active: activeTransport === t.id }"
          @click="selectTransport(t.id)"
        >
          {{ t.label }}
        </button>
      </div>

      <div class="code-wrapper fade-in">
        <CodeBlock
          :code="activeExample.code"
          :language="activeExample.lang"
        />
      </div>

      <div class="install-notes fade-in">
        <span>npm install @clasp-to/core</span>
        <span>pip install clasp-to</span>
        <span>cargo add clasp-client</span>
      </div>
    </div>
  </section>
</template>

<style scoped>
.crosslang-section {
  background: #f77f00;
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
  max-width: 550px;
  margin: 0 auto 2rem;
  font-size: 1.1rem;
  line-height: 1.6;
  opacity: 0.9;
}

.lang-tabs {
  display: flex;
  justify-content: center;
  gap: 0.5rem;
  margin-bottom: 0.75rem;
}

.lang-tab {
  background: transparent;
  border: 2px solid rgba(255,255,255,0.4);
  color: rgba(255,255,255,0.8);
  padding: 0.6rem 1.5rem;
  font-family: 'Space Mono', monospace;
  font-size: 0.85rem;
  letter-spacing: 0.1em;
  cursor: pointer;
  transition: all 0.2s ease;
}

.lang-tab:hover {
  border-color: #fff;
  color: #fff;
}

.lang-tab.active {
  background: rgba(255,255,255,0.95);
  border-color: rgba(255,255,255,0.95);
  color: #f77f00;
}

.transport-tabs {
  display: flex;
  justify-content: center;
  gap: 0.4rem;
  margin-bottom: 1rem;
  flex-wrap: wrap;
}

.transport-tab {
  background: transparent;
  border: 1.5px solid rgba(255,255,255,0.3);
  color: rgba(255,255,255,0.7);
  padding: 0.4rem 1rem;
  font-family: 'Space Mono', monospace;
  font-size: 0.7rem;
  letter-spacing: 0.1em;
  cursor: pointer;
  transition: all 0.2s ease;
}

.transport-tab:hover {
  border-color: #fff;
  color: #fff;
}

.transport-tab.active {
  background: rgba(255,255,255,0.95);
  border-color: rgba(255,255,255,0.95);
  color: #f77f00;
}

.code-wrapper {
  max-width: 750px;
  margin: 0 auto;
  background: #1a1a1a;
  padding: 1.5rem;
  border: 2px solid rgba(255,255,255,0.2);
  overflow-x: auto;
  min-width: 0;
}

.code-wrapper :deep(.hljs) {
  color: rgba(255,255,255,0.9) !important;
}

.code-wrapper :deep(pre) {
  margin: 0;
  white-space: pre-wrap;
  word-break: break-word;
}

.code-wrapper :deep(code) {
  font-size: 0.8rem;
}

.install-notes {
  display: flex;
  justify-content: center;
  gap: 2rem;
  margin-top: 2rem;
  flex-wrap: wrap;
}

.install-notes span {
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.75rem;
  opacity: 0.7;
  padding: 0.3rem 0.6rem;
  background: rgba(255,255,255,0.15);
}

@media (max-width: 768px) {
  .code-wrapper {
    padding: 1rem;
  }

  .code-wrapper :deep(code) {
    font-size: 0.7rem;
  }

  .install-notes {
    gap: 0.75rem;
  }

  .install-notes span {
    font-size: 0.65rem;
  }
}
</style>
