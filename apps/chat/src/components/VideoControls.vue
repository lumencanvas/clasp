<script setup>
const props = defineProps({
  audioEnabled: { type: Boolean, default: true },
  videoEnabled: { type: Boolean, default: true },
  isScreenSharing: { type: Boolean, default: false },
  layout: { type: String, default: 'grid' },
})

const emit = defineEmits(['toggle-audio', 'toggle-video', 'share-screen', 'leave', 'set-layout'])
</script>

<template>
  <div class="video-controls">
    <button
      :class="['control-btn', { off: !audioEnabled }]"
      @click="emit('toggle-audio')"
      :title="audioEnabled ? 'Mute' : 'Unmute'"
      :aria-label="audioEnabled ? 'Mute microphone' : 'Unmute microphone'"
    >
      <svg v-if="audioEnabled" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z"/>
        <path d="M19 10v2a7 7 0 0 1-14 0v-2"/>
        <line x1="12" y1="19" x2="12" y2="23"/>
        <line x1="8" y1="23" x2="16" y2="23"/>
      </svg>
      <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="1" y1="1" x2="23" y2="23"/>
        <path d="M9 9v3a3 3 0 0 0 5.12 2.12M15 9.34V4a3 3 0 0 0-5.94-.6"/>
        <path d="M17 16.95A7 7 0 0 1 5 12v-2m14 0v2c0 .76-.13 1.49-.35 2.17"/>
        <line x1="12" y1="19" x2="12" y2="23"/>
        <line x1="8" y1="23" x2="16" y2="23"/>
      </svg>
    </button>

    <button
      :class="['control-btn', { off: !videoEnabled }]"
      @click="emit('toggle-video')"
      :title="videoEnabled ? 'Camera off' : 'Camera on'"
      :aria-label="videoEnabled ? 'Turn camera off' : 'Turn camera on'"
    >
      <svg v-if="videoEnabled" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polygon points="23 7 16 12 23 17 23 7"/>
        <rect x="1" y="5" width="15" height="14" rx="2" ry="2"/>
      </svg>
      <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M16 16v1a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V7a2 2 0 0 1 2-2h2m5.66 0H14a2 2 0 0 1 2 2v3.34l1 1L23 7v10"/>
        <line x1="1" y1="1" x2="23" y2="23"/>
      </svg>
    </button>

    <button
      :class="['control-btn', { sharing: isScreenSharing }]"
      @click="emit('share-screen')"
      title="Share screen"
      aria-label="Share screen"
    >
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="2" y="3" width="20" height="14" rx="2" ry="2"/>
        <line x1="8" y1="21" x2="16" y2="21"/>
        <line x1="12" y1="17" x2="12" y2="21"/>
      </svg>
    </button>

    <!-- Layout switcher -->
    <div class="layout-group">
      <button
        :class="['layout-btn', { active: layout === 'grid' }]"
        @click="emit('set-layout', 'grid')"
        title="Grid layout"
        aria-label="Grid layout"
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="3" y="3" width="7" height="7"/>
          <rect x="14" y="3" width="7" height="7"/>
          <rect x="3" y="14" width="7" height="7"/>
          <rect x="14" y="14" width="7" height="7"/>
        </svg>
      </button>
      <button
        :class="['layout-btn', { active: layout === 'spotlight' }]"
        @click="emit('set-layout', 'spotlight')"
        title="Spotlight layout"
        aria-label="Spotlight layout"
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="3" y="3" width="18" height="12"/>
          <rect x="3" y="18" width="5" height="3"/>
          <rect x="10" y="18" width="5" height="3"/>
          <rect x="17" y="18" width="4" height="3"/>
        </svg>
      </button>
      <button
        :class="['layout-btn', { active: layout === 'sidebar' }]"
        @click="emit('set-layout', 'sidebar')"
        title="Sidebar layout"
        aria-label="Sidebar layout"
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="3" y="3" width="12" height="18"/>
          <rect x="18" y="3" width="3" height="5"/>
          <rect x="18" y="10" width="3" height="5"/>
          <rect x="18" y="17" width="3" height="4"/>
        </svg>
      </button>
    </div>

    <button
      class="control-btn leave-btn"
      @click="emit('leave')"
      title="Leave video"
      aria-label="Leave video"
    >
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M10.68 13.31a16 16 0 0 0 3.41 2.6l1.27-1.27a2 2 0 0 1 2.11-.45 12.84 12.84 0 0 0 2.81.7 2 2 0 0 1 1.72 2v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.42 19.42 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72 12.84 12.84 0 0 0 .7 2.81 2 2 0 0 1-.45 2.11L8.09 9.91"/>
        <line x1="23" y1="1" x2="1" y2="23"/>
      </svg>
    </button>
  </div>
</template>

<style scoped>
.video-controls {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-wrap: wrap;
  gap: 0.5rem;
  padding: 0.75rem;
  background: var(--bg-secondary);
  border-top: 1px solid var(--border);
  flex-shrink: 0;
}

.control-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 44px;
  height: 44px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: 50%;
  color: var(--text-primary);
  transition: all 0.15s;
}

.control-btn svg {
  width: 18px;
  height: 18px;
}

.control-btn:hover {
  background: var(--bg-active);
}

.control-btn:active {
  transform: scale(0.96);
  opacity: 0.8;
}

.control-btn.off {
  background: rgba(230,57,70,0.15);
  border-color: var(--danger);
  color: var(--danger);
}

.control-btn.sharing {
  background: rgba(var(--accent-rgb),0.15);
  border-color: var(--accent);
  color: var(--accent);
}

.leave-btn {
  background: var(--danger);
  border-color: var(--danger);
  color: white;
}

.leave-btn:hover {
  opacity: 0.9;
  background: var(--danger);
}

.layout-group {
  display: flex;
  gap: 2px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 2px;
}

.layout-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  background: transparent;
  border: none;
  border-radius: 4px;
  color: var(--text-muted);
  transition: all 0.15s;
}

@media (hover: hover) and (pointer: fine) {
  .layout-btn {
    width: 32px;
    height: 32px;
  }
}

.layout-btn svg {
  width: 16px;
  height: 16px;
}

.layout-btn:hover {
  color: var(--text-primary);
  background: var(--bg-active);
}

.layout-btn.active {
  color: var(--accent);
  background: var(--bg-active);
}
</style>
