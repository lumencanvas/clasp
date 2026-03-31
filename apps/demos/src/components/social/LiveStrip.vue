<script setup>
defineProps({ streams: Map, isLive: Boolean, myName: String })
const emit = defineEmits(['open-self', 'open-viewer'])
</script>

<template>
  <div class="ls">
    <div class="ls-hd">
      <div class="lbdge"><div class="lbdot"></div>LIVE</div>
      <span class="live-ct">{{ streams.size + (isLive ? 1 : 0) }} live</span>
    </div>
    <div class="ls-sc">
      <button v-if="isLive" class="lp own" @click="emit('open-self')">
        <div class="lp-dot"></div>{{ myName }}
      </button>
      <button v-for="[uid, entry] in streams" :key="uid" class="lp" @click="emit('open-viewer', entry)">
        <div class="lp-dot"></div>{{ entry.name }}
      </button>
      <div v-if="!streams.size && !isLive" class="ls-em">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" width="13" height="13"><line x1="1" y1="1" x2="23" y2="23"/><path d="M16.72 11.06A10.94 10.94 0 0119 12.55"/><path d="M5 12.55a11 11 0 0110.11-2.97"/></svg>
        no active streams
      </div>
    </div>
  </div>
</template>
