<script setup>
import { ref, watch, onMounted, onUnmounted, nextTick } from 'vue'

const props = defineProps({
  headings: { type: Array, default: () => [] }
})

const activeId = ref('')
let observer = null

// Only show h2 and h3 in the TOC
const tocHeadings = ref([])

watch(() => props.headings, (h) => {
  tocHeadings.value = h.filter(h => h.level === 2 || h.level === 3)
}, { immediate: true })

function setupObserver() {
  if (observer) observer.disconnect()
  if (tocHeadings.value.length === 0) return

  const ids = tocHeadings.value.map(h => h.id)
  const elements = ids.map(id => document.getElementById(id)).filter(Boolean)
  if (elements.length === 0) return

  observer = new IntersectionObserver(
    (entries) => {
      // Find the topmost visible heading
      const visible = entries
        .filter(e => e.isIntersecting)
        .sort((a, b) => a.boundingClientRect.top - b.boundingClientRect.top)

      if (visible.length > 0) {
        activeId.value = visible[0].target.id
      }
    },
    {
      rootMargin: '-80px 0px -60% 0px',
      threshold: 0
    }
  )

  elements.forEach(el => observer.observe(el))
}

watch(tocHeadings, async () => {
  await nextTick()
  setupObserver()
})

onMounted(() => {
  nextTick(() => setupObserver())
})

onUnmounted(() => {
  if (observer) observer.disconnect()
})

function scrollTo(id) {
  const el = document.getElementById(id)
  if (el) {
    el.scrollIntoView({ behavior: 'smooth', block: 'start' })
    activeId.value = id
  }
}
</script>

<template>
  <nav v-if="tocHeadings.length > 0" class="docs-toc">
    <div class="toc-title">ON THIS PAGE</div>
    <a
      v-for="heading in tocHeadings"
      :key="heading.id"
      class="toc-link"
      :class="{
        active: activeId === heading.id,
        'toc-h3': heading.level === 3
      }"
      @click.prevent="scrollTo(heading.id)"
      :href="'#' + heading.id"
    >
      {{ heading.text }}
    </a>
  </nav>
</template>

<style scoped>
.docs-toc {
  position: sticky;
  top: calc(var(--nav-height) + 1.5rem);
  align-self: start;
  padding: 0 1rem;
  font-size: 0.78rem;
  max-height: calc(100vh - var(--nav-height) - 3rem);
  overflow-y: auto;
}

.docs-toc::-webkit-scrollbar {
  width: 3px;
}

.docs-toc::-webkit-scrollbar-thumb {
  background: var(--border);
  border-radius: 2px;
}

.toc-title {
  letter-spacing: 0.22em;
  font-size: 0.65rem;
  font-weight: 700;
  font-family: 'Archivo Black', sans-serif;
  text-transform: uppercase;
  opacity: 0.4;
  margin-bottom: 0.75rem;
}

.toc-link {
  display: block;
  padding: 0.25rem 0 0.25rem 0.75rem;
  color: var(--ink);
  text-decoration: none;
  opacity: 0.5;
  border-left: 2px solid transparent;
  transition: all 0.15s;
  line-height: 1.4;
}

.toc-link:hover {
  opacity: 0.8;
  text-decoration: none;
}

.toc-link.active {
  opacity: 1;
  border-left-color: var(--accent);
  font-weight: 700;
}

.toc-h3 {
  padding-left: 1.5rem;
  font-size: 0.74rem;
}

@media (max-width: 1100px) {
  .docs-toc {
    display: none;
  }
}
</style>
