<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoutes } from '../../composables/useRoutes'
import SignalRouteCard from '../cards/SignalRouteCard.vue'
import SignalRouteModal from '../modals/SignalRouteModal.vue'
import EmptyState from '../shared/EmptyState.vue'

const { routes } = useRoutes()
const modalRef = ref<InstanceType<typeof SignalRouteModal> | null>(null)

const hasRoutes = computed(() => routes.value.length > 0)

function openAdd() {
  modalRef.value?.open()
}

function openEdit(route: any) {
  modalRef.value?.open(route)
}
</script>

<template>
  <div style="display: flex; flex-direction: column; height: 100%;">
    <div class="panel-toolbar">
      <span class="panel-title">SIGNAL ROUTES</span>
      <div class="toolbar-group">
        <button class="btn btn-primary btn-sm" @click="openAdd">+ NEW ROUTE</button>
      </div>
    </div>
    <div class="panel-content">
      <div v-if="hasRoutes" class="mapping-list">
        <SignalRouteCard v-for="route in routes" :key="route.id" :route="route" @edit="openEdit" />
      </div>
      <EmptyState v-else message="No signal routes" hint="Signal routes transform and forward signals between connections">
        <template #icon>
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" opacity="0.4">
            <polyline points="16 3 21 3 21 8"/><line x1="4" y1="20" x2="21" y2="3"/><polyline points="21 16 21 21 16 21"/><line x1="15" y1="15" x2="21" y2="21"/>
          </svg>
        </template>
      </EmptyState>
    </div>
    <SignalRouteModal ref="modalRef" />
  </div>
</template>
