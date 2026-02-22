<script setup>
import { ref, onMounted, watch } from 'vue'

const isDark = ref(false)

onMounted(() => {
  const stored = localStorage.getItem('clasp-docs-theme')
  if (stored) {
    isDark.value = stored === 'dark'
  } else {
    isDark.value = window.matchMedia('(prefers-color-scheme: dark)').matches
  }
  applyTheme()
})

watch(isDark, () => {
  applyTheme()
  localStorage.setItem('clasp-docs-theme', isDark.value ? 'dark' : 'light')
})

function applyTheme() {
  document.documentElement.setAttribute('data-theme', isDark.value ? 'dark' : 'light')
}

function toggle() {
  isDark.value = !isDark.value
}
</script>

<template>
  <button
    class="theme-toggle"
    @click="toggle"
    :aria-label="isDark ? 'Switch to light mode' : 'Switch to dark mode'"
    :title="isDark ? 'Light mode' : 'Dark mode'"
  >
    {{ isDark ? '\u2600' : '\u263E' }}
  </button>
</template>

<style scoped>
.theme-toggle {
  background: none;
  border: 1px solid rgba(255,255,255,0.2);
  color: #fff;
  cursor: pointer;
  font-size: 1rem;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: border-color 0.15s;
  padding: 0;
  line-height: 1;
}

.theme-toggle:hover {
  border-color: rgba(255,255,255,0.5);
}
</style>
