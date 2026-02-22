<script setup>
import { computed } from 'vue'

const props = defineProps({
  docPath: { type: String, default: '' },
  title: { type: String, default: '' }
})

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
  'protocols': 'Protocols',
  'getting-started': 'Getting Started',
  'security': 'Security',
}

const SUBSECTION_LABELS = {
  'connections': 'Connections',
  'state': 'State',
  'timing': 'Timing',
  'discovery': 'Discovery',
  'security': 'Security',
  'advanced': 'Advanced',
  'installation': 'Installation',
  'protocol': 'Protocol',
  'cli': 'CLI',
  'bridges': 'Bridges',
  'transports': 'Transports',
  'configuration': 'Configuration',
  'rust': 'Rust',
  'javascript': 'JavaScript',
  'python': 'Python',
  'common': 'Common',
  'migration': 'Migration',
}

const crumbs = computed(() => {
  if (!props.docPath || props.docPath === 'index') return []

  const parts = props.docPath.split('/')
  const result = []

  for (let i = 0; i < parts.length; i++) {
    const segment = parts[i]
    const path = '/' + parts.slice(0, i + 1).join('/')
    const isLast = i === parts.length - 1

    if (isLast) {
      result.push({ label: props.title || formatSegment(segment), path: null })
    } else {
      const label = (i === 0 ? SECTION_LABELS[segment] : SUBSECTION_LABELS[segment]) || formatSegment(segment)
      result.push({ label, path })
    }
  }

  return result
})

function formatSegment(seg) {
  return seg.replace(/-/g, ' ').replace(/\b\w/g, c => c.toUpperCase())
}
</script>

<template>
  <nav v-if="crumbs.length > 0" class="breadcrumbs">
    <router-link to="/" class="crumb">Docs</router-link>
    <template v-for="(crumb, i) in crumbs" :key="i">
      <span class="crumb-sep">/</span>
      <router-link v-if="crumb.path" :to="crumb.path" class="crumb">{{ crumb.label }}</router-link>
      <span v-else class="crumb current">{{ crumb.label }}</span>
    </template>
  </nav>
</template>

<style scoped>
.breadcrumbs {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 0.25rem;
  margin-bottom: 1.5rem;
  font-size: 0.75rem;
  letter-spacing: 0.06em;
}

.crumb {
  color: var(--ink);
  text-decoration: none;
  opacity: 0.5;
  transition: opacity 0.15s;
}

.crumb:hover {
  opacity: 0.8;
  text-decoration: none;
}

.crumb.current {
  opacity: 0.8;
  font-weight: 700;
}

.crumb-sep {
  opacity: 0.3;
  font-size: 0.7rem;
}
</style>
