<script setup lang="ts">
import { useRoute } from 'vue-router'

const route = useRoute()

const tabs = [
  { name: 'flow', label: 'FLOW', path: '/flow' },
  { name: 'links', label: 'LINKS', path: '/links' },
  { name: 'routes', label: 'ROUTES', path: '/routes' },
  { name: 'rules', label: 'RULES', path: '/rules' },
  { name: 'security', label: 'SECURITY', path: '/security' },
  { name: 'monitor', label: 'MONITOR', path: '/monitor' },
  { name: 'test', label: 'TEST', path: '/test' },
  { name: 'logs', label: 'LOGS', path: '/logs' },
  { name: 'defra', label: 'DEFRA', path: '/defra' },
]
</script>

<template>
  <nav class="tabs">
    <router-link
      v-for="tab in tabs"
      :key="tab.name"
      :to="tab.path"
      class="tab"
      :class="{ active: route.name === tab.name }"
    >
      {{ tab.label }}
    </router-link>
  </nav>
</template>
