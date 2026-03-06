<script setup lang="ts">
import type { SignalRoute } from '../../lib/types'
import { useRoutes } from '../../composables/useRoutes'
import { formatTransformBadge } from '../../lib/transforms'
import ProtocolBadge from '../shared/ProtocolBadge.vue'

const props = defineProps<{ route: SignalRoute }>()
const emit = defineEmits<{ edit: [route: SignalRoute] }>()
const { remove } = useRoutes()

const transformLabel = formatTransformBadge(props.route.transform)
</script>

<template>
  <div class="mapping-item">
    <div class="mapping-source">
      <div class="mapping-protocol">
        <ProtocolBadge :protocol="route.source?.protocol || 'any'" />
      </div>
      <div class="mapping-address">{{ route.source?.address || '*' }}</div>
    </div>
    <div class="mapping-transform-badge">{{ transformLabel }}</div>
    <div class="mapping-target">
      <div class="mapping-protocol">
        <ProtocolBadge :protocol="route.target?.protocol || 'any'" />
      </div>
      <div class="mapping-address">{{ route.target?.address || '*' }}</div>
    </div>
    <div class="bridge-actions">
      <button class="btn btn-sm btn-secondary" @click="emit('edit', route)" title="Edit route">
        <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
      </button>
      <button class="btn btn-sm btn-delete" @click="remove(route.id)" title="Delete route">
        <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
      </button>
    </div>
  </div>
</template>
