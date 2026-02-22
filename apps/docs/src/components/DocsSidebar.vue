<script setup>
import { computed, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import manifest from 'virtual:docs-manifest'

const props = defineProps({
  currentPath: { type: String, default: '' }
})

const emit = defineEmits(['navigate'])
const router = useRouter()

// Section ordering and labels matching the planned IA
const SECTION_CONFIG = [
  {
    key: 'getting-started',
    label: 'GETTING STARTED',
    overrideItems: [
      { path: 'index', title: 'Overview' },
      { path: 'getting-started', title: 'Installation' },
      { path: 'getting-started/first-connection', title: 'First Connection' },
    ]
  },
  {
    key: 'sdk',
    label: 'CLIENT SDKS',
    basePath: 'sdk'
  },
  {
    key: 'core',
    label: 'CORE CONCEPTS',
    basePath: 'core'
  },
  {
    key: 'protocols',
    label: 'PROTOCOL BRIDGES',
    basePath: 'protocols'
  },
  {
    key: 'deployment',
    label: 'DEPLOYMENT',
    basePath: 'deployment'
  },
  {
    key: 'auth',
    label: 'AUTH & SECURITY',
    basePath: 'auth'
  },
  {
    key: 'server',
    label: 'SERVER FEATURES',
    basePath: 'server'
  },
  {
    key: 'reference',
    label: 'REFERENCE',
    basePath: 'reference'
  },
  {
    key: 'concepts',
    label: 'DEEP DIVES',
    basePath: 'concepts'
  },
  {
    key: 'tools',
    label: 'TOOLS',
    overrideItems: [
      { path: 'tools/relay-configurator', title: 'Relay Configurator' },
    ]
  },
]

// Build a lookup of all pages by path
const pagesByPath = computed(() => {
  const map = {}
  for (const page of manifest) {
    map[page.path] = page
  }
  return map
})

// Build the sidebar sections
const sections = computed(() => {
  const usedPaths = new Set()
  const result = []

  for (const config of SECTION_CONFIG) {
    const section = { label: config.label, key: config.key, items: [], groups: null }

    if (config.overrideItems) {
      // Manual item list
      for (const item of config.overrideItems) {
        const page = pagesByPath.value[item.path]
        if (page) {
          section.items.push({ path: page.path, title: item.title || page.title })
          usedPaths.add(page.path)
        } else if (item.title) {
          // Allow non-manifest items (e.g. tool pages with their own routes)
          section.items.push({ path: item.path, title: item.title })
        }
      }
    } else if (config.basePath) {
      // Auto-populate from manifest
      const excludeSet = new Set(config.exclude || [])
      const pages = manifest
        .filter(p => {
          if (!p.path.startsWith(config.basePath + '/') && p.path !== config.basePath) return false
          if (excludeSet.has(p.path)) return false
          // Check exclude prefixes
          for (const ex of excludeSet) {
            if (p.path.startsWith(ex + '/')) return false
          }
          return true
        })
        .sort((a, b) => a.order - b.order || a.title.localeCompare(b.title))

      if (config.subgroups) {
        section.groups = []
        const ungrouped = []

        for (const [subPath, groupLabel] of Object.entries(config.subgroups)) {
          const fullPrefix = config.basePath + '/' + subPath
          const groupPages = pages.filter(p =>
            p.path.startsWith(fullPrefix + '/') || p.path === fullPrefix
          )
          if (groupPages.length > 0) {
            section.groups.push({
              label: groupLabel,
              items: groupPages.map(p => ({ path: p.path, title: p.title }))
            })
            groupPages.forEach(p => usedPaths.add(p.path))
          }
        }

        // Add ungrouped pages
        for (const p of pages) {
          if (!usedPaths.has(p.path)) {
            ungrouped.push({ path: p.path, title: p.title })
            usedPaths.add(p.path)
          }
        }
        if (ungrouped.length > 0) {
          section.items = ungrouped
        }
      } else {
        // Section-level README goes first
        const readme = pages.find(p => p.path === config.basePath)
        const rest = pages.filter(p => p.path !== config.basePath)
        if (readme) {
          section.items.push({ path: readme.path, title: 'Overview' })
          usedPaths.add(readme.path)
        }
        for (const p of rest) {
          section.items.push({ path: p.path, title: p.title })
          usedPaths.add(p.path)
        }
      }
    }

    if (section.items.length > 0 || (section.groups && section.groups.length > 0)) {
      result.push(section)
    }
  }

  // Catch-all for pages not in any defined section
  const uncategorized = manifest
    .filter(p => !usedPaths.has(p.path) && p.path !== 'index')
    .sort((a, b) => a.title.localeCompare(b.title))

  if (uncategorized.length > 0) {
    result.push({
      label: 'OTHER',
      key: 'other',
      items: uncategorized.map(p => ({ path: p.path, title: p.title })),
      groups: null
    })
  }

  return result
})

// Track which sections are expanded
const openSections = ref(new Set())

// Auto-open the section containing the current page
watch(() => props.currentPath, (path) => {
  for (const section of sections.value) {
    const hasPage = section.items.some(i => i.path === path) ||
      (section.groups && section.groups.some(g => g.items.some(i => i.path === path)))
    if (hasPage) {
      openSections.value.add(section.key)
    }
  }
}, { immediate: true })

function toggleSection(key) {
  if (openSections.value.has(key)) {
    openSections.value.delete(key)
  } else {
    openSections.value.add(key)
  }
}

function navigate(path) {
  router.push('/' + path)
  emit('navigate')
}
</script>

<template>
  <aside class="docs-sidebar">
    <div v-for="section in sections" :key="section.key" class="sidebar-section" :class="{ open: openSections.has(section.key) }">
      <div class="sidebar-section-label" @click="toggleSection(section.key)">
        {{ section.label }}
      </div>

      <div class="sidebar-section-items">
        <!-- Direct items (no subgroups) -->
        <a
          v-for="item in section.items"
          :key="item.path"
          class="sidebar-link"
          :class="{ active: currentPath === item.path }"
          @click.prevent="navigate(item.path)"
          :href="'/' + item.path"
        >
          {{ item.title }}
        </a>

        <!-- Subgroups -->
        <template v-if="section.groups">
          <div v-for="group in section.groups" :key="group.label">
            <div class="sidebar-group-label">{{ group.label }}</div>
            <a
              v-for="item in group.items"
              :key="item.path"
              class="sidebar-link sidebar-sublink"
              :class="{ active: currentPath === item.path }"
              @click.prevent="navigate(item.path)"
              :href="'/' + item.path"
            >
              {{ item.title }}
            </a>
          </div>
        </template>
      </div>
    </div>
  </aside>
</template>
