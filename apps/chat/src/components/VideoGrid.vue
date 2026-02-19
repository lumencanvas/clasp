<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'
import VideoTile from './VideoTile.vue'

const props = defineProps({
  localStream: { type: MediaStream, default: null },
  localName: { type: String, default: 'You' },
  audioEnabled: { type: Boolean, default: true },
  videoEnabled: { type: Boolean, default: true },
  isScreenShare: { type: Boolean, default: false },
  avatarColor: { type: String, default: null },
  peers: { type: Array, default: () => [] },
  layout: { type: String, default: 'grid' },
  spotlightPeer: { type: String, default: null },
})

const allTiles = computed(() => {
  const local = {
    id: '__local__',
    stream: props.localStream,
    name: props.localName,
    audioEnabled: props.audioEnabled,
    videoEnabled: props.videoEnabled,
    isLocal: true,
    isScreenShare: props.isScreenShare,
    avatarColor: props.avatarColor,
  }
  const remote = props.peers.map(p => ({
    id: p.id,
    stream: p.stream,
    name: p.name,
    audioEnabled: p.audioEnabled,
    videoEnabled: p.videoEnabled,
    isLocal: false,
    isScreenShare: false,
    avatarColor: p.avatarColor || null,
  }))
  return [local, ...remote]
})

const spotlightTile = computed(() => {
  if (!props.spotlightPeer) return allTiles.value[0]
  return allTiles.value.find(t => t.id === props.spotlightPeer) || allTiles.value[0]
})

const stripTiles = computed(() => {
  const spotlight = spotlightTile.value
  return allTiles.value.filter(t => t.id !== spotlight.id)
})

const totalTiles = computed(() => allTiles.value.length)

const gridClass = computed(() => {
  const n = totalTiles.value
  if (n <= 1) return 'grid-1'
  if (n <= 4) return 'grid-4'
  return 'grid-9'
})

// IntersectionObserver: only attach streams for visible tiles
const gridRef = ref(null)
const visibleTileIds = ref(new Set())
let observer = null

function isTileVisible(id) {
  // For small counts, always visible
  if (totalTiles.value <= 9) return true
  return visibleTileIds.value.has(id)
}

onMounted(() => {
  if (totalTiles.value > 9 && gridRef.value) {
    observer = new IntersectionObserver((entries) => {
      for (const entry of entries) {
        const id = entry.target.dataset.tileId
        if (entry.isIntersecting) {
          visibleTileIds.value.add(id)
        } else {
          visibleTileIds.value.delete(id)
        }
      }
      visibleTileIds.value = new Set(visibleTileIds.value)
    }, { root: gridRef.value, threshold: 0.1 })
  }
})

onUnmounted(() => {
  observer?.disconnect()
})
</script>

<template>
  <!-- Grid layout -->
  <div v-if="layout === 'grid'" ref="gridRef" :class="['video-grid', gridClass]">
    <div v-for="tile in allTiles" :key="tile.id" :data-tile-id="tile.id">
      <VideoTile
        :stream="isTileVisible(tile.id) ? tile.stream : null"
        :name="tile.name"
        :audio-enabled="tile.audioEnabled"
        :video-enabled="tile.videoEnabled"
        :is-local="tile.isLocal"
        :is-screen-share="tile.isScreenShare"
        :avatar-color="tile.avatarColor"
        :muted="tile.isLocal"
      />
    </div>
  </div>

  <!-- Spotlight layout: one large + horizontal strip below -->
  <div v-else-if="layout === 'spotlight'" class="video-spotlight">
    <div class="spotlight-main">
      <VideoTile
        :stream="spotlightTile.stream"
        :name="spotlightTile.name"
        :audio-enabled="spotlightTile.audioEnabled"
        :video-enabled="spotlightTile.videoEnabled"
        :is-local="spotlightTile.isLocal"
        :is-screen-share="spotlightTile.isScreenShare"
        :avatar-color="spotlightTile.avatarColor"
        :muted="spotlightTile.isLocal"
      />
    </div>
    <div v-if="stripTiles.length" class="spotlight-strip">
      <VideoTile
        v-for="tile in stripTiles"
        :key="tile.id"
        :stream="tile.stream"
        :name="tile.name"
        :audio-enabled="tile.audioEnabled"
        :video-enabled="tile.videoEnabled"
        :is-local="tile.isLocal"
        :is-screen-share="tile.isScreenShare"
        :avatar-color="tile.avatarColor"
        :muted="tile.isLocal"
      />
    </div>
  </div>

  <!-- Sidebar layout: one large left + vertical strip right -->
  <div v-else class="video-sidebar">
    <div class="sidebar-main">
      <VideoTile
        :stream="spotlightTile.stream"
        :name="spotlightTile.name"
        :audio-enabled="spotlightTile.audioEnabled"
        :video-enabled="spotlightTile.videoEnabled"
        :is-local="spotlightTile.isLocal"
        :is-screen-share="spotlightTile.isScreenShare"
        :avatar-color="spotlightTile.avatarColor"
        :muted="spotlightTile.isLocal"
      />
    </div>
    <div v-if="stripTiles.length" class="sidebar-strip">
      <VideoTile
        v-for="tile in stripTiles"
        :key="tile.id"
        :stream="tile.stream"
        :name="tile.name"
        :audio-enabled="tile.audioEnabled"
        :video-enabled="tile.videoEnabled"
        :is-local="tile.isLocal"
        :is-screen-share="tile.isScreenShare"
        :avatar-color="tile.avatarColor"
        :muted="tile.isLocal"
      />
    </div>
  </div>
</template>

<style scoped>
/* Grid layout */
.video-grid {
  flex: 1;
  display: grid;
  gap: 0.5rem;
  padding: 0.5rem;
  min-height: 0;
  overflow: hidden;
  align-content: center;
}

.video-grid > div {
  min-height: 0;
  overflow: hidden;
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
  .grid-4 {
    grid-template-columns: repeat(2, 1fr);
  }

  .grid-9 {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (max-width: 320px) {
  .grid-4, .grid-9 {
    grid-template-columns: 1fr;
  }
}

/* Spotlight layout */
.video-spotlight {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  padding: 0.5rem;
  min-height: 0;
}

.spotlight-main {
  flex: 1;
  min-height: 0;
}

.spotlight-main :deep(.video-tile) {
  aspect-ratio: unset;
  height: 100%;
}

.spotlight-strip {
  display: flex;
  gap: 0.5rem;
  flex-shrink: 0;
  overflow-x: auto;
  height: 120px;
}

.spotlight-strip :deep(.video-tile) {
  aspect-ratio: 16/9;
  height: 100%;
  width: auto;
  min-width: 160px;
  flex-shrink: 0;
}

/* Sidebar layout */
.video-sidebar {
  flex: 1;
  display: flex;
  gap: 0.5rem;
  padding: 0.5rem;
  min-height: 0;
}

.sidebar-main {
  flex: 7;
  min-width: 0;
}

.sidebar-main :deep(.video-tile) {
  aspect-ratio: unset;
  height: 100%;
}

.sidebar-strip {
  flex: 3;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  overflow-y: auto;
  min-width: 0;
}

.sidebar-strip :deep(.video-tile) {
  flex-shrink: 0;
}

@media (max-width: 480px) {
  .video-sidebar {
    flex-direction: column;
  }

  .sidebar-main {
    flex: 1;
  }

  .sidebar-strip {
    flex-direction: row;
    overflow-x: auto;
    overflow-y: hidden;
    height: 100px;
  }

  .sidebar-strip :deep(.video-tile) {
    min-width: 140px;
    height: 100%;
  }
}
</style>
