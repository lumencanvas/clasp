import { ref, computed, watch } from 'vue'

/**
 * Video layout mode composable
 * Manages grid/spotlight/sidebar layouts and peer pinning
 */
export function useVideoLayout(isScreenSharing, speakingPeerIds) {
  const layout = ref('grid')
  const pinnedPeerId = ref(null)

  let layoutBeforeScreenShare = null

  const activeSpeakerId = computed(() => {
    if (!speakingPeerIds?.value) return null
    const ids = speakingPeerIds.value
    if (ids.size === 0) return null
    // Prefer non-local speaker
    for (const id of ids) {
      if (id !== '__local__') return id
    }
    // Fall back to local if only local is speaking
    return ids.values().next().value
  })

  const spotlightPeer = computed(() => {
    // Pinned peer always takes priority
    if (pinnedPeerId.value) return pinnedPeerId.value
    // Auto-speaker in spotlight/sidebar layouts
    if ((layout.value === 'spotlight' || layout.value === 'sidebar') && activeSpeakerId.value) {
      return activeSpeakerId.value
    }
    return null
  })

  function setLayout(mode) {
    if (['grid', 'spotlight', 'sidebar'].includes(mode)) {
      layout.value = mode
    }
  }

  function pinPeer(id) {
    // Toggle: if already pinned, unpin
    if (pinnedPeerId.value === id) {
      pinnedPeerId.value = null
    } else {
      pinnedPeerId.value = id
    }
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
    activeSpeakerId,
    spotlightPeer,
    setLayout,
    pinPeer,
    unpinPeer,
  }
}
