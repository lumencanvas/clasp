<script setup>
import { computed } from 'vue'
import manifest from 'virtual:docs-manifest'

const props = defineProps({
  currentPath: { type: String, default: '' }
})

// Build a flat ordered list following the sidebar structure
const PAGE_ORDER = [
  'index',
  'how-to/installation/cli',
  'how-to/installation/javascript-library',
  'how-to/installation/python-library',
  'how-to/installation/rust-library',
  'tutorials/first-connection',
]

const SECTION_ORDER = [
  'tutorials',
  'how-to',
  'explanation',
  'reference',
  'use-cases',
  'integrations',
  'appendix',
]

const orderedPages = computed(() => {
  const pageSet = new Set(manifest.map(p => p.path))
  const result = []
  const used = new Set()

  // First add the explicit getting-started pages
  for (const path of PAGE_ORDER) {
    if (pageSet.has(path) && !used.has(path)) {
      result.push(path)
      used.add(path)
    }
  }

  // Then add pages by section order
  for (const section of SECTION_ORDER) {
    const sectionPages = manifest
      .filter(p => p.path.startsWith(section + '/') || p.path === section)
      .filter(p => !used.has(p.path))
      .sort((a, b) => a.order - b.order || a.path.localeCompare(b.path))

    for (const p of sectionPages) {
      result.push(p.path)
      used.add(p.path)
    }
  }

  // Any remaining pages
  for (const p of manifest) {
    if (!used.has(p.path)) {
      result.push(p.path)
    }
  }

  return result
})

const pagesByPath = computed(() => {
  const map = {}
  for (const p of manifest) map[p.path] = p
  return map
})

const currentIndex = computed(() => orderedPages.value.indexOf(props.currentPath))

const prevPage = computed(() => {
  if (currentIndex.value <= 0) return null
  const path = orderedPages.value[currentIndex.value - 1]
  return pagesByPath.value[path] || null
})

const nextPage = computed(() => {
  if (currentIndex.value < 0 || currentIndex.value >= orderedPages.value.length - 1) return null
  const path = orderedPages.value[currentIndex.value + 1]
  return pagesByPath.value[path] || null
})
</script>

<template>
  <nav v-if="prevPage || nextPage" class="prev-next-nav">
    <router-link v-if="prevPage" :to="'/' + prevPage.path" class="pn-link pn-prev">
      <span class="pn-label">&larr; Previous</span>
      <span class="pn-title">{{ prevPage.title }}</span>
    </router-link>
    <div v-else></div>

    <router-link v-if="nextPage" :to="'/' + nextPage.path" class="pn-link pn-next">
      <span class="pn-label">Next &rarr;</span>
      <span class="pn-title">{{ nextPage.title }}</span>
    </router-link>
  </nav>
</template>

<style scoped>
.prev-next-nav {
  display: flex;
  justify-content: space-between;
  gap: 1rem;
  margin-top: 3rem;
  padding-top: 1.5rem;
  border-top: 1px solid var(--border);
}

.pn-link {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  padding: 0.75rem 1rem;
  border: 1px solid var(--border);
  text-decoration: none;
  color: var(--ink);
  transition: border-color 0.15s;
  max-width: 45%;
}

.pn-link:hover {
  border-color: var(--accent);
  text-decoration: none;
}

.pn-next {
  text-align: right;
  margin-left: auto;
}

.pn-label {
  font-size: 0.7rem;
  letter-spacing: 0.15em;
  text-transform: uppercase;
  opacity: 0.5;
}

.pn-title {
  font-size: 0.85rem;
  font-weight: 700;
  letter-spacing: 0.04em;
}

@media (max-width: 480px) {
  .prev-next-nav {
    flex-direction: column;
  }

  .pn-link {
    max-width: 100%;
  }

  .pn-next {
    text-align: left;
  }
}
</style>
