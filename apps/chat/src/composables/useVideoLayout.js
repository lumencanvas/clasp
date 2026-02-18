import { ref, computed, watch } from 'vue'

/**
 * Video layout mode composable
 * Manages grid/spotlight/sidebar layouts and peer pinning
 */
export function useVideoLayout(isScreenSharing) {
  const layout = ref('grid')
  const pinnedPeerId = ref(null)

  let layoutBeforeScreenShare = null

  const spotlightPeer = computed(() => pinnedPeerId.value || null)

  function setLayout(mode) {
    if (['grid', 'spotlight', 'sidebar'].includes(mode)) {
      layout.value = mode
    }
  }

  function pinPeer(id) {
    pinnedPeerId.value = id
  }

  function unpinPeer() {
    pinnedPeerId.value = null
  }

  // Auto-switch to spotlight when screen share starts
  if (isScreenSharing) {
    watch(isScreenSharing, (sharing) => {
      if (sharing) {
        layoutBeforeScreenShare = layout.value
        layout.value = 'spotlight'
        pinnedPeerId.value = '__local__'
      } else {
        if (layoutBeforeScreenShare) {
          layout.value = layoutBeforeScreenShare
          layoutBeforeScreenShare = null
        }
        if (pinnedPeerId.value === '__local__') {
          pinnedPeerId.value = null
        }
      }
    })
  }

  return {
    layout,
    pinnedPeerId,
    spotlightPeer,
    setLayout,
    pinPeer,
    unpinPeer,
  }
}
