<script setup>
import { ref } from 'vue'

const props = defineProps({ post: Object, isOwn: Boolean, ageTick: Number })
const emit = defineEmits(['react', 'delete'])
const imgBroken = ref(false)

function fmtAge(ts) {
  const s = Math.floor((Date.now() - ts) / 1000)
  if (s < 60) return s + 's'
  if (s < 3600) return Math.floor(s / 60) + 'm'
  if (s < 86400) return Math.floor(s / 3600) + 'h'
  return Math.floor(s / 86400) + 'd'
}

function expiryPct(p) {
  if (!p.ttl) return 100
  return Math.max(0, Math.min(100, (1 - (Date.now() - p.created) / (p.ttl * 1000)) * 100))
}
</script>

<template>
  <article class="post" :class="{ exp: post.ttl && expiryPct(post) < 15 }">
    <div class="ph">
      <div class="pav">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" width="14" height="14"><circle cx="12" cy="8" r="4"/><path d="M4 20c0-4 3.6-7 8-7s8 3 8 7"/></svg>
      </div>
      <div class="pm">
        <div class="pn">{{ post.author?.name || 'anon' }}</div>
        <div class="phan">{{ post.author?.handle || '@anon' }}</div>
      </div>
      <div class="pr">
        <span class="stag" :class="(post.stype || 'SET').toLowerCase()">{{ post.stype || 'SET' }}</span>
        <span class="pt" :data-tick="ageTick">{{ fmtAge(post.created) }}</span>
      </div>
    </div>
    <div v-if="post.text" class="pb">{{ post.text }}</div>
    <div v-if="post.image && !imgBroken" class="pimg"><img :src="post.image" loading="lazy" @error="imgBroken = true" /></div>
    <div class="pf">
      <button v-for="rk in ['zap', 'rep', 'heart']" :key="rk" class="ab rb" :class="{ act: post.myReactions?.[rk] }" @click="emit('react', post.id, rk)">
        <svg v-if="rk === 'zap'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="12" height="12"><polyline points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/></svg>
        <svg v-else-if="rk === 'rep'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="12" height="12"><polyline points="17 1 21 5 17 9"/><path d="M3 11V9a4 4 0 014-4h14"/><polyline points="7 23 3 19 7 15"/><path d="M21 13v2a4 4 0 01-4 4H3"/></svg>
        <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="12" height="12"><path d="M20.84 4.61a5.5 5.5 0 00-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 00-7.78 7.78L12 21.23l8.84-8.84a5.5 5.5 0 000-7.78z"/></svg>
        <span class="rc">{{ post.reactions?.[rk] || 0 }}</span>
      </button>
      <button v-if="isOwn" class="ab db" @click="emit('delete', post.id)">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="12" height="12"><polyline points="3 6 5 6 21 6"/><path d="M19 6l-1 14H6L5 6"/><path d="M10 11v6M14 11v6"/><path d="M9 6V4h6v2"/></svg>
      </button>
    </div>
    <div class="pex"><div class="pexf" :style="{ width: expiryPct(post) + '%', background: (post.ttl && expiryPct(post) < 15) ? 'var(--red)' : undefined }"></div></div>
  </article>
</template>
