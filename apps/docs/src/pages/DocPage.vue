<script setup>
import { computed, ref, watch, onMounted, nextTick } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import docsContent from 'virtual:docs-content'
import DocsSidebar from '../components/DocsSidebar.vue'
import DocsTableOfContents from '../components/DocsTableOfContents.vue'
import Breadcrumbs from '../components/Breadcrumbs.vue'
import PrevNextNav from '../components/PrevNextNav.vue'

const route = useRoute()
const router = useRouter()
const sidebarOpen = ref(false)

const docPath = computed(() => {
  const path = route.params.pathMatch
  if (Array.isArray(path)) return path.join('/')
  return path || 'index'
})

const doc = computed(() => docsContent[docPath.value] || null)

function closeSidebar() {
  sidebarOpen.value = false
}

// Intercept clicks on internal links to use vue-router
function handleContentClick(e) {
  const link = e.target.closest('a')
  if (!link) return
  const href = link.getAttribute('href')
  if (!href) return

  // Handle anchor links
  if (href.startsWith('#')) {
    e.preventDefault()
    const el = document.querySelector(href)
    if (el) el.scrollIntoView({ behavior: 'smooth' })
    return
  }

  if (href.startsWith('http') || href.startsWith('mailto:')) return

  e.preventDefault()
  router.push(href)
}

// Scroll to hash after content loads
watch(doc, async () => {
  if (route.hash) {
    await nextTick()
    const el = document.querySelector(route.hash)
    if (el) el.scrollIntoView({ behavior: 'smooth' })
  }
})

onMounted(() => {
  if (route.hash) {
    nextTick(() => {
      const el = document.querySelector(route.hash)
      if (el) el.scrollIntoView({ behavior: 'smooth' })
    })
  }
})
</script>

<template>
  <div class="docs-layout" :class="{ 'has-toc': doc && doc.headings.length > 0 }">
    <div
      class="sidebar-overlay"
      :class="{ visible: sidebarOpen }"
      @click="closeSidebar"
    ></div>

    <DocsSidebar
      :current-path="docPath"
      :class="{ 'mobile-open': sidebarOpen }"
      @navigate="closeSidebar"
    />

    <div class="docs-content-wrap">
      <template v-if="doc">
        <Breadcrumbs :doc-path="docPath" :title="doc.title" />
        <div class="doc-content" @click="handleContentClick" v-html="doc.html"></div>
        <PrevNextNav :current-path="docPath" />
      </template>

      <div v-else class="doc-not-found">
        <h2>PAGE NOT FOUND</h2>
        <p>No documentation found for <code>/{{ docPath }}</code></p>
        <p style="margin-top: 1rem;">
          <router-link to="/">&larr; Back to docs home</router-link>
        </p>
      </div>
    </div>

    <DocsTableOfContents
      v-if="doc"
      :headings="doc.headings"
    />

    <button
      class="sidebar-toggle"
      @click="sidebarOpen = !sidebarOpen"
      :aria-label="sidebarOpen ? 'Close navigation' : 'Open navigation'"
    >
      {{ sidebarOpen ? '\u2715' : '\u2630' }}
    </button>
  </div>
</template>
