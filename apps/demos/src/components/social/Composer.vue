<script setup>
import { ref, computed } from 'vue'

const props = defineProps({ handle: String, isLive: Boolean })
const emit = defineEmits(['transmit', 'go-live'])

const text = ref('')
const selectedTtl = ref(1800)
const image = ref(null)
const imageInput = ref(null)
const showUrlRow = ref(false)
const urlInput = ref('')
const maxChars = 500
const charCount = computed(() => maxChars - text.value.length)
const charClass = computed(() => {
  const n = text.value.length
  if (n > maxChars) return 'ovr'
  if (n > maxChars * 0.85) return 'wrn'
  return ''
})

const ttlOptions = [
  { label: '5m', value: 300 },
  { label: '30m', value: 1800 },
  { label: '1h', value: 3600 },
  { label: '24h', value: 86400 },
  { label: 'never', value: 0 },
]

function compressImage(file) {
  return new Promise((resolve, reject) => {
    const rd = new FileReader()
    rd.onerror = reject
    rd.onload = (e) => {
      const img = new Image()
      img.onerror = reject
      img.onload = () => {
        let w = img.width, h = img.height
        const ratio = Math.min(720 / w, 720 / h, 1)
        w = Math.round(w * ratio); h = Math.round(h * ratio)
        const cv = document.createElement('canvas')
        cv.width = w; cv.height = h
        cv.getContext('2d').drawImage(img, 0, 0, w, h)
        function tryQ(q) {
          const d = cv.toDataURL('image/jpeg', q)
          if (d.length * 0.75 > 62000 && q > 0.2) return tryQ(q - 0.15)
          return d
        }
        resolve(tryQ(0.6))
      }
      img.src = e.target.result
    }
    rd.readAsDataURL(file)
  })
}

async function handleFile(e) {
  const f = e.target.files?.[0]
  if (!f) return
  try { image.value = await compressImage(f) } catch { /* toast handled by parent if needed */ }
  if (imageInput.value) imageInput.value.value = ''
}

function handleUrlKey(e) {
  if (e.key === 'Enter') {
    const u = urlInput.value.trim()
    if (u) { image.value = u; showUrlRow.value = false }
  }
}

function handleUrlPaste() {
  const u = urlInput.value.trim()
  if (u.match(/^https?:\/\//)) image.value = u
}

function clearImage() { image.value = null; urlInput.value = '' }

function transmit() {
  const t = text.value.trim().slice(0, maxChars)
  if (!t && !image.value) return
  emit('transmit', { text: t, image: image.value, ttl: selectedTtl.value })
  text.value = ''; clearImage(); showUrlRow.value = false
}

defineExpose({ transmit: (opts) => {
  emit('transmit', { text: opts.text || '', image: null, ttl: selectedTtl.value, stype: opts.stype })
}})
</script>

<template>
  <div class="composer">
    <div class="ct">
      <div class="cav">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" width="15" height="15"><circle cx="12" cy="8" r="4"/><path d="M4 20c0-4 3.6-7 8-7s8 3 8 7"/></svg>
      </div>
      <div class="cr">
        <div class="chandl">{{ handle }}</div>
        <textarea v-model="text" placeholder="transmit a signal..." rows="2" @keydown.meta.enter.prevent="transmit" @keydown.ctrl.enter.prevent="transmit"></textarea>
      </div>
    </div>

    <div v-if="image" class="img-prev">
      <img :src="image" @error="clearImage" />
      <button class="img-prev-rm" @click="clearImage">&times;</button>
    </div>

    <div v-if="showUrlRow" class="url-row">
      <input v-model="urlInput" placeholder="paste image URL and press enter" @keydown="handleUrlKey" @input="handleUrlPaste" />
    </div>

    <div class="ttl-row">
      <span class="ttl-lbl">ttl</span>
      <button v-for="o in ttlOptions" :key="o.value" class="ttb" :class="{ on: selectedTtl === o.value }" @click="selectedTtl = o.value">{{ o.label }}</button>
    </div>

    <div class="ca">
      <label class="ctool">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="13" height="13"><path d="M23 19a2 2 0 01-2 2H3a2 2 0 01-2-2V8a2 2 0 012-2h4l2-3h6l2 3h4a2 2 0 012 2z"/><circle cx="12" cy="13" r="4"/></svg>
        photo
        <input ref="imageInput" type="file" accept="image/*" style="display:none" @change="handleFile" />
      </label>
      <button class="ctool" @click="showUrlRow = !showUrlRow">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="13" height="13"><path d="M10 13a5 5 0 007.54.54l3-3a5 5 0 00-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 00-7.54-.54l-3 3a5 5 0 007.07 7.07l1.71-1.71"/></svg>
        url
      </button>
      <button class="lbtn" :class="{ on: isLive }" @click="emit('go-live')">
        <svg width="10" height="10" viewBox="0 0 24 24" fill="currentColor"><circle cx="12" cy="12" r="9"/></svg>
        <span>{{ isLive ? 'you are live' : 'go live' }}</span>
      </button>
      <span class="cc" :class="charClass">{{ charCount }}</span>
      <button class="txbtn" @click="transmit">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="13" height="13"><line x1="22" y1="2" x2="11" y2="13"/><polygon points="22 2 15 22 11 13 2 9 22 2"/></svg>
        TRANSMIT
      </button>
    </div>
  </div>
</template>
