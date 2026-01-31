<script setup>
import { ref } from 'vue'
import CodeBlock from '../CodeBlock.vue'
import { useScrollAnimation } from '../../composables/useScrollAnimation.js'

const sectionRef = ref(null)
useScrollAnimation(sectionRef)

const activeTab = ref('js')

const tabs = [
  { id: 'js', label: 'JavaScript' },
  { id: 'py', label: 'Python' },
  { id: 'rs', label: 'Rust' }
]

const jsCode = `// Install: npm install @clasp-to/core

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
clasp.close();`

const pyCode = `# Install: pip install clasp-to

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

asyncio.run(main())`

const rsCode = `// Cargo.toml: clasp-client = "3.0"

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
}`

const codeExamples = {
  js: { code: jsCode, lang: 'javascript' },
  py: { code: pyCode, lang: 'python' },
  rs: { code: rsCode, lang: 'rust' }
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
          @click="activeTab = tab.id"
        >
          {{ tab.label }}
        </button>
      </div>

      <div class="code-wrapper fade-in">
        <div
          v-for="tab in tabs"
          :key="tab.id"
          v-show="activeTab === tab.id"
        >
          <CodeBlock
            :code="codeExamples[tab.id].code"
            :language="codeExamples[tab.id].lang"
          />
        </div>
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
  margin-bottom: 1.5rem;
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
