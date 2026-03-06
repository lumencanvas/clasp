<script setup lang="ts">
import { computed, ref } from 'vue'
import { useBridges } from '../../composables/useBridges'
import DirectLinkCard from '../cards/DirectLinkCard.vue'
import DirectLinkModal from '../modals/DirectLinkModal.vue'
import EmptyState from '../shared/EmptyState.vue'

const { bridges } = useBridges()
const modalRef = ref<InstanceType<typeof DirectLinkModal> | null>(null)

const hasBridges = computed(() => bridges.value.length > 0)

function openAdd() {
  modalRef.value?.open()
}

function openEdit(bridge: any) {
  modalRef.value?.open(bridge)
}
</script>

<template>
  <div style="display: flex; flex-direction: column; height: 100%;">
    <div class="panel-toolbar">
      <span class="panel-title">DIRECT LINKS</span>
      <div class="toolbar-group">
        <button class="btn btn-primary btn-sm" @click="openAdd">+ NEW LINK</button>
      </div>
    </div>
    <div class="panel-content">
      <div v-if="hasBridges" class="bridge-grid">
        <DirectLinkCard v-for="bridge in bridges" :key="bridge.id" :bridge="bridge" @edit="openEdit" />
      </div>
      <EmptyState v-else message="No direct links" hint="Direct links forward signals between protocols without routing through CLASP">
        <template #icon>
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" opacity="0.4">
            <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/><polyline points="15 3 21 3 21 9"/><line x1="10" y1="14" x2="21" y2="3"/>
          </svg>
        </template>
      </EmptyState>
    </div>
    <DirectLinkModal ref="modalRef" />
  </div>
</template>
