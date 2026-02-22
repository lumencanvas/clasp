<script setup>
import { ref } from 'vue'

const props = defineProps({
  code: { type: String, default: '' },
  lang: { type: String, default: '' }
})

const copied = ref(false)

async function copy() {
  try {
    await navigator.clipboard.writeText(props.code)
    copied.value = true
    setTimeout(() => { copied.value = false }, 1500)
  } catch {
    // fallback
    const ta = document.createElement('textarea')
    ta.value = props.code
    document.body.appendChild(ta)
    ta.select()
    document.execCommand('copy')
    document.body.removeChild(ta)
    copied.value = true
    setTimeout(() => { copied.value = false }, 1500)
  }
}
</script>

<template>
  <div class="copy-block">
    <div class="copy-block-header">
      <span v-if="lang" class="copy-block-lang">{{ lang }}</span>
      <button class="copy-btn" @click="copy">
        {{ copied ? 'Copied' : 'Copy' }}
      </button>
    </div>
    <pre class="copy-block-pre"><code>{{ code }}</code></pre>
  </div>
</template>

<style scoped>
.copy-block {
  position: relative;
  border: 1px solid var(--code-border);
  margin: 0.5rem 0;
}

.copy-block-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.3rem 0.75rem;
  background: var(--code-bg);
  border-bottom: 1px solid var(--code-border);
  font-size: 0.7rem;
}

.copy-block-lang {
  font-family: 'JetBrains Mono', monospace;
  letter-spacing: 0.08em;
  opacity: 0.5;
  text-transform: uppercase;
}

.copy-btn {
  font-family: 'Space Mono', monospace;
  font-size: 0.7rem;
  padding: 0.15rem 0.5rem;
  background: none;
  border: 1px solid var(--border);
  color: var(--ink);
  cursor: pointer;
  border-radius: 2px;
  letter-spacing: 0.06em;
  transition: border-color 0.15s, color 0.15s;
}

.copy-btn:hover {
  border-color: var(--accent);
  color: var(--accent);
}

.copy-block-pre {
  margin: 0;
  padding: 0.75rem 1rem;
  background: var(--code-bg);
  overflow-x: auto;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.8rem;
  line-height: 1.5;
  white-space: pre;
}

.copy-block-pre code {
  font-family: inherit;
}
</style>
