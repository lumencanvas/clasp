<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  status: string
  title?: string
}>()

function getStatusClass(status: string): string {
  switch (status) {
    case 'connected':
    case 'running':
      return 'connected'
    case 'starting':
    case 'reconnecting':
      return 'connecting'
    case 'error':
    case 'disconnected':
      return 'error'
    case 'available':
      return 'available'
    default:
      return 'available'
  }
}

const statusClass = computed(() => getStatusClass(props.status))
</script>

<template>
  <span class="status-dot" :class="statusClass" :title="title"></span>
</template>
