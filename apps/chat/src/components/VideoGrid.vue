<script setup>
import { computed } from 'vue'
import VideoTile from './VideoTile.vue'

const props = defineProps({
  localStream: { type: MediaStream, default: null },
  localName: { type: String, default: 'You' },
  audioEnabled: { type: Boolean, default: true },
  videoEnabled: { type: Boolean, default: true },
  peers: { type: Array, default: () => [] },
})

const totalTiles = computed(() => 1 + props.peers.length) // local + peers

const gridClass = computed(() => {
  const n = totalTiles.value
  if (n <= 1) return 'grid-1'
  if (n <= 4) return 'grid-4'
  return 'grid-9'
})
</script>

<template>
  <div :class="['video-grid', gridClass]">
    <!-- Local tile -->
    <VideoTile
      :stream="localStream"
      :name="localName"
      :audio-enabled="audioEnabled"
      :video-enabled="videoEnabled"
      :is-local="true"
      :muted="true"
    />

    <!-- Remote tiles -->
    <VideoTile
      v-for="peer in peers"
      :key="peer.id"
      :stream="peer.stream"
      :name="peer.name"
      :audio-enabled="peer.audioEnabled"
      :video-enabled="peer.videoEnabled"
    />
  </div>
</template>

<style scoped>
.video-grid {
  flex: 1;
  display: grid;
  gap: 0.5rem;
  padding: 0.5rem;
  min-height: 0;
  align-content: center;
}

.grid-1 {
  grid-template-columns: 1fr;
  max-width: 800px;
  margin: 0 auto;
  width: 100%;
}

.grid-4 {
  grid-template-columns: repeat(2, 1fr);
}

.grid-9 {
  grid-template-columns: repeat(3, 1fr);
}

@media (max-width: 480px) {
  .grid-4, .grid-9 {
    grid-template-columns: 1fr;
  }
}
</style>
