<script setup>
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { useRouter } from 'vue-router'
import MiniSearch from 'minisearch'
import manifest from 'virtual:docs-manifest'
import docsContent from 'virtual:docs-content'

const router = useRouter()
const isOpen = ref(false)
const query = ref('')
const selectedIndex = ref(0)
const inputRef = ref(null)

let miniSearch = null

const SECTION_LABELS = {
  'tutorials': 'Tutorials',
  'how-to': 'Guides',
  'explanation': 'Concepts',
  'reference': 'Reference',
  'use-cases': 'Use Cases',
  'integrations': 'Integrations',
  'appendix': 'Appendix',
  'api': 'API',
  'guides': 'Guides',
  'getting-started': 'Getting Started',
  'security': 'Security',
  'protocols': 'Protocols',
}

function stripHtml(html) {
  return html
    .replace(/<pre[\s\S]*?<\/pre>/g, ' ')
    .replace(/<[^>]+>/g, ' ')
    .replace(/&[a-z]+;/g, ' ')
    .replace(/\s+/g, ' ')
    .trim()
}

function buildIndex() {
  if (miniSearch) return

  miniSearch = new MiniSearch({
    fields: ['title', 'headingText', 'body'],
    storeFields: ['title', 'path', 'section', 'description'],
    searchOptions: {
      boost: { title: 10, headingText: 5, body: 1 },
      fuzzy: 0.2,
      prefix: true,
    }
  })

  const docs = manifest.map(page => {
    const content = docsContent[page.path]
    const body = content ? stripHtml(content.html).slice(0, 5000) : ''
    const headingText = page.headings.map(h => h.text).join(' ')

    return {
      id: page.path,
      title: page.title,
      path: page.path,
      section: page.section,
      description: page.description,
      headingText,
      body,
    }
  })

  miniSearch.addAll(docs)
}

const results = computed(() => {
  if (!query.value.trim() || !miniSearch) return []
  return miniSearch.search(query.value.trim()).slice(0, 20)
})

watch(query, () => {
  selectedIndex.value = 0
})

function open() {
  buildIndex()
  isOpen.value = true
  query.value = ''
  selectedIndex.value = 0
  nextTick(() => inputRef.value?.focus())
}

function close() {
  isOpen.value = false
  query.value = ''
}

function navigate(path) {
  close()
  router.push('/' + path)
}

function handleKeydown(e) {
  if (e.key === 'ArrowDown') {
    e.preventDefault()
    if (selectedIndex.value < results.value.length - 1) {
      selectedIndex.value++
    }
  } else if (e.key === 'ArrowUp') {
    e.preventDefault()
    if (selectedIndex.value > 0) {
      selectedIndex.value--
    }
  } else if (e.key === 'Enter') {
    e.preventDefault()
    const result = results.value[selectedIndex.value]
    if (result) navigate(result.path)
  } else if (e.key === 'Escape') {
    close()
  }
}

function handleGlobalKeydown(e) {
  if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
    e.preventDefault()
    if (isOpen.value) {
      close()
    } else {
      open()
    }
  }
  if (e.key === 'Escape' && isOpen.value) {
    close()
  }
}

function sectionLabel(section) {
  return SECTION_LABELS[section] || section
}

function getSnippet(result) {
  if (result.description) return result.description
  const content = docsContent[result.path]
  if (!content) return ''
  const text = stripHtml(content.html)
  return text.slice(0, 120) + (text.length > 120 ? '...' : '')
}

onMounted(() => {
  window.addEventListener('keydown', handleGlobalKeydown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleGlobalKeydown)
})

defineExpose({ open })
</script>

<template>
  <Teleport to="body">
    <div v-if="isOpen" class="search-backdrop" @click.self="close">
      <div class="search-modal">
        <div class="search-input-wrap">
          <span class="search-icon">&#x2315;</span>
          <input
            ref="inputRef"
            v-model="query"
            type="text"
            class="search-input"
            placeholder="Search documentation..."
            @keydown="handleKeydown"
          />
          <kbd class="search-kbd">ESC</kbd>
        </div>

        <div v-if="query.trim() && results.length === 0" class="search-empty">
          No results for "{{ query }}"
        </div>

        <div v-if="results.length > 0" class="search-results">
          <a
            v-for="(result, i) in results"
            :key="result.id"
            class="search-result"
            :class="{ selected: i === selectedIndex }"
            @click.prevent="navigate(result.path)"
            @mouseenter="selectedIndex = i"
            :href="'/' + result.path"
          >
            <div class="result-header">
              <span class="result-title">{{ result.title }}</span>
              <span class="result-section">{{ sectionLabel(result.section) }}</span>
            </div>
            <div class="result-snippet">{{ getSnippet(result) }}</div>
          </a>
        </div>

        <div v-if="!query.trim()" class="search-hint">
          Type to search across all documentation
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.search-backdrop {
  position: fixed;
  inset: 0;
  z-index: 100;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding-top: 15vh;
}

.search-modal {
  width: 90vw;
  max-width: 600px;
  background: var(--paper);
  border: 1px solid var(--border);
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.25);
  max-height: 70vh;
  display: flex;
  flex-direction: column;
}

.search-input-wrap {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem 1rem;
  border-bottom: 1px solid var(--border);
}

.search-icon {
  font-size: 1.2rem;
  opacity: 0.4;
}

.search-input {
  flex: 1;
  border: none;
  outline: none;
  background: transparent;
  font-family: 'Space Mono', monospace;
  font-size: 0.9rem;
  color: var(--ink);
}

.search-input::placeholder {
  color: var(--ink);
  opacity: 0.35;
}

.search-kbd {
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.65rem;
  padding: 0.15rem 0.4rem;
  border: 1px solid var(--border);
  opacity: 0.5;
  letter-spacing: 0.05em;
}

.search-results {
  overflow-y: auto;
  flex: 1;
}

.search-result {
  display: block;
  padding: 0.75rem 1rem;
  border-bottom: 1px solid var(--border);
  text-decoration: none;
  color: var(--ink);
  cursor: pointer;
  transition: background 0.1s;
}

.search-result:hover,
.search-result.selected {
  background: var(--hover-bg);
  text-decoration: none;
}

.result-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  margin-bottom: 0.25rem;
}

.result-title {
  font-weight: 700;
  font-size: 0.85rem;
  letter-spacing: 0.04em;
}

.result-section {
  font-size: 0.65rem;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  opacity: 0.4;
  white-space: nowrap;
}

.result-snippet {
  font-size: 0.78rem;
  opacity: 0.6;
  line-height: 1.4;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.search-empty {
  padding: 2rem 1rem;
  text-align: center;
  opacity: 0.5;
  font-size: 0.85rem;
}

.search-hint {
  padding: 1.5rem 1rem;
  text-align: center;
  opacity: 0.35;
  font-size: 0.8rem;
  letter-spacing: 0.04em;
}

@media (max-width: 480px) {
  .search-backdrop {
    padding-top: 5vh;
  }

  .search-modal {
    max-height: 80vh;
  }
}
</style>
