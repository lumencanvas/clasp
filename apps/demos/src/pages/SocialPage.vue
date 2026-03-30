<script setup>
import { ref, reactive, computed, onMounted, onUnmounted } from 'vue'
import { useRelay } from '../composables/useRelay.js'
import { usePresence } from '../composables/usePresence.js'

const emit = defineEmits(['auth-required'])
const { client, connected, sessionId, userName, authToken, connect } = useRelay()

// --- Identity ---
function loadMe() {
  const saved = localStorage.getItem('clasp_demo_social_me')
  if (saved) try { return JSON.parse(saved) } catch {}
  const id = crypto.randomUUID().replace(/-/g, '').slice(0, 16)
  return { id, name: userName.value || 'Anon', handle: '@' + id.slice(0, 8) }
}
const me = reactive(loadMe())
function saveMe() { localStorage.setItem('clasp_demo_social_me', JSON.stringify(me)) }

// --- Channel ---
const channel = ref('main')
const NS = computed(() => `/social/v1/${channel.value}`)

// --- State ---
const posts = reactive(new Map())
const sortedPosts = computed(() => {
  return [...posts.values()].sort((a, b) => b.created - a.created)
})

// --- Presence ---
const presence = usePresence(
  client, () => NS.value, me.id,
  () => ({ id: me.id, name: me.name, handle: me.handle })
)

// --- Composer ---
const composerText = ref('')
const selectedTtl = ref(1800) // 30m default
const compressedImage = ref(null)
const imageInput = ref(null)

const ttlOptions = [
  { label: '5m', value: 300 },
  { label: '30m', value: 1800 },
  { label: '1h', value: 3600 },
  { label: '6h', value: 21600 },
  { label: '24h', value: 86400 },
  { label: 'never', value: 0 },
]

// --- Rate limiting ---
const postTimestamps = []
const RATE_MAX = 8
const RATE_WINDOW = 10000
const isFlooded = ref(false)

function canPost() {
  const now = Date.now()
  while (postTimestamps.length && now - postTimestamps[0] > RATE_WINDOW) postTimestamps.shift()
  return postTimestamps.length < RATE_MAX
}

// --- Image compression ---
const MAX_DIM = 720
const MAX_BYTES = 62000

function compressImage(file) {
  return new Promise((resolve, reject) => {
    const rd = new FileReader()
    rd.onerror = reject
    rd.onload = (e) => {
      const img = new Image()
      img.onerror = reject
      img.onload = () => {
        let w = img.width, h = img.height
        const ratio = Math.min(MAX_DIM / w, MAX_DIM / h, 1)
        w = Math.round(w * ratio)
        h = Math.round(h * ratio)
        const cv = document.createElement('canvas')
        cv.width = w
        cv.height = h
        cv.getContext('2d').drawImage(img, 0, 0, w, h)
        function tryQ(q) {
          const d = cv.toDataURL('image/jpeg', q)
          if (d.length * 0.75 > MAX_BYTES && q > 0.2) return tryQ(q - 0.15)
          return d
        }
        resolve(tryQ(0.6))
      }
      img.src = e.target.result
    }
    rd.readAsDataURL(file)
  })
}

async function handleImage(e) {
  const file = e.target.files?.[0]
  if (!file) return
  try {
    compressedImage.value = await compressImage(file)
  } catch {
    compressedImage.value = null
  }
}

function clearImage() {
  compressedImage.value = null
  if (imageInput.value) imageInput.value.value = ''
}

// --- Post management ---
function addPost(p) {
  if (posts.has(p.id)) return
  p.reactions = p.reactions || {}
  p.myReactions = p.myReactions || {}
  posts.set(p.id, p)
}

function removePost(id) {
  posts.delete(id)
}

async function submitPost() {
  const text = composerText.value.trim()
  if (!text && !compressedImage.value) return
  if (!canPost()) {
    isFlooded.value = true
    setTimeout(() => { isFlooded.value = false }, 3000)
    return
  }

  const c = client.value
  if (!c) return

  const p = {
    id: crypto.randomUUID().replace(/-/g, '').slice(0, 16),
    author: me.name,
    handle: me.handle,
    authorId: me.id,
    text,
    image: compressedImage.value || null,
    created: Date.now(),
    ttl: selectedTtl.value,
    reactions: {},
    myReactions: {},
  }

  postTimestamps.push(Date.now())
  addPost(p)
  composerText.value = ''
  clearImage()

  const payload = { ...p }
  delete payload.myReactions
  c.set(`${NS.value}/post/${p.id}`, JSON.stringify(payload), {
    ttl: p.ttl || undefined,
    absolute: true,
  })
}

function deletePost(id) {
  removePost(id)
  const c = client.value
  if (c) c.set(`${NS.value}/post/${id}`, null)
}

function react(postId, key) {
  const p = posts.get(postId)
  if (!p) return
  if (p.myReactions[key]) return // already reacted
  p.reactions[key] = (p.reactions[key] || 0) + 1
  p.myReactions[key] = true
  const c = client.value
  if (c) c.emit(`${NS.value}/react/${postId}`, JSON.stringify({
    postId, reaction: key, userId: me.id,
  }))
}

function applyReact(postId, key) {
  const p = posts.get(postId)
  if (!p) return
  p.reactions[key] = (p.reactions[key] || 0) + 1
}

// --- Time formatting ---
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

// --- Age ticker ---
const ageTick = ref(0)
let ageTimer = null

// --- CLASP setup ---
const unsubs = []

function setupSubscriptions() {
  const c = client.value
  if (!c) return

  const u1 = c.on(`${NS.value}/post/**`, (v, addr) => {
    if (!v) { removePost(addr.split('/').pop()); return }
    try {
      const p = JSON.parse(v)
      if (p.author) addPost(p)
    } catch {}
  })
  if (typeof u1 === 'function') unsubs.push(u1)

  const u2 = c.on(`${NS.value}/react/**`, (v) => {
    if (!v) return
    try {
      const d = JSON.parse(v)
      if (d && d.userId !== me.id && posts.has(d.postId)) {
        applyReact(d.postId, d.reaction)
      }
    } catch {}
  })
  if (typeof u2 === 'function') unsubs.push(u2)
}

onMounted(async () => {
  if (!authToken.value) {
    emit('auth-required')
    return
  }

  me.name = userName.value || me.name
  saveMe()

  try {
    await connect()
    setupSubscriptions()
    presence.start()
  } catch (e) {
    console.error('[Social]', e)
  }

  ageTimer = setInterval(() => { ageTick.value++ }, 5000)
})

onUnmounted(() => {
  clearInterval(ageTimer)
  unsubs.forEach(u => { try { u() } catch {} })
  unsubs.length = 0
})

const reactions = [
  { key: 'zap', label: 'zap' },
  { key: 'heart', label: 'heart' },
  { key: 'fire', label: 'fire' },
]
</script>

<template>
  <div class="social">
    <div class="container">
      <!-- Header -->
      <div class="page-head">
        <div class="page-head-left">
          <h1 class="page-title">SOCIAL FEED</h1>
          <span class="page-tag">#{{ channel }}</span>
        </div>
        <span class="online-ct">{{ presence.onlineCount.value }} online</span>
      </div>

      <!-- Rate limit warning -->
      <div v-if="isFlooded" class="flood-banner fade-in">
        Slow down -- rate limit reached
      </div>

      <!-- Composer -->
      <div class="composer" v-if="authToken">
        <div class="comp-top">
          <div class="comp-avatar">{{ me.name[0]?.toUpperCase() }}</div>
          <div class="comp-input">
            <div class="comp-handle">{{ me.handle }}</div>
            <textarea
              v-model="composerText"
              placeholder="What's happening on the relay?"
              rows="2"
              @keydown.meta.enter="submitPost"
              @keydown.ctrl.enter="submitPost"
            ></textarea>
          </div>
        </div>

        <!-- Image preview -->
        <div v-if="compressedImage" class="comp-img-preview">
          <img :src="compressedImage" alt="attached" />
          <button class="comp-img-rm" @click="clearImage">&times;</button>
        </div>

        <div class="comp-bar">
          <div class="comp-actions">
            <label class="comp-btn" title="Attach image">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><circle cx="8.5" cy="8.5" r="1.5"/>
                <polyline points="21 15 16 10 5 21"/>
              </svg>
              <input
                ref="imageInput"
                type="file"
                accept="image/*"
                style="display:none"
                @change="handleImage"
              />
            </label>
          </div>
          <div class="comp-right">
            <select v-model.number="selectedTtl" class="ttl-select">
              <option v-for="o in ttlOptions" :key="o.value" :value="o.value">
                {{ o.label }}
              </option>
            </select>
            <button class="post-btn" @click="submitPost" :disabled="!composerText.trim() && !compressedImage">
              POST
            </button>
          </div>
        </div>
      </div>

      <!-- Posts -->
      <div class="feed">
        <div
          v-for="p in sortedPosts"
          :key="p.id"
          class="post fade-in"
        >
          <div class="post-head">
            <div class="post-avatar">{{ p.author[0]?.toUpperCase() }}</div>
            <div class="post-meta">
              <span class="post-author">{{ p.author }}</span>
              <span class="post-handle">{{ p.handle }}</span>
            </div>
            <span class="post-age" :data-tick="ageTick">{{ fmtAge(p.created) }}</span>
            <button
              v-if="p.authorId === me.id"
              class="post-del"
              @click="deletePost(p.id)"
              title="Delete"
            >&times;</button>
          </div>

          <p v-if="p.text" class="post-text">{{ p.text }}</p>
          <img v-if="p.image" :src="p.image" class="post-img" alt="" loading="lazy" />

          <!-- TTL bar -->
          <div v-if="p.ttl" class="ttl-bar" :data-tick="ageTick">
            <div class="ttl-fill" :style="{ width: expiryPct(p) + '%' }"></div>
          </div>

          <!-- Reactions -->
          <div class="post-reactions">
            <button
              v-for="r in reactions"
              :key="r.key"
              class="react-btn"
              :class="{ mine: p.myReactions?.[r.key] }"
              @click="react(p.id, r.key)"
            >
              {{ r.label }}
              <span v-if="p.reactions?.[r.key]" class="react-ct">{{ p.reactions[r.key] }}</span>
            </button>
          </div>
        </div>

        <div v-if="!sortedPosts.length" class="empty-feed">
          <p>No posts yet. Be the first to post something.</p>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.social { min-height: 100vh; padding-bottom: 80px; }

.page-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 0 16px;
  border-bottom: 1px solid var(--bdr);
  margin-bottom: 16px;
}
.page-head-left { display: flex; align-items: center; gap: 10px; }
.page-title {
  font-family: var(--head);
  font-size: 16px;
  letter-spacing: 0.15em;
  color: var(--br);
}
.page-tag {
  font-family: var(--mono);
  font-size: 10px;
  color: var(--teal);
  background: var(--teal-d);
  border: 1px solid var(--teal-m);
  padding: 2px 8px;
  border-radius: 3px;
}
.online-ct {
  font-family: var(--mono);
  font-size: 11px;
  color: var(--dim);
}

/* Flood */
.flood-banner {
  background: rgba(212,168,64,0.06);
  border: 1px solid rgba(212,168,64,0.2);
  border-radius: var(--r);
  padding: 10px 14px;
  margin-bottom: 12px;
  font-size: 12px;
  color: var(--ylw);
  text-align: center;
}

/* Composer */
.composer {
  background: var(--surf);
  border: 1px solid var(--bdr);
  border-radius: 6px;
  margin-bottom: 16px;
  transition: border-color 0.2s;
}
.composer:focus-within { border-color: var(--teal-m); }
.comp-top { display: flex; gap: 12px; padding: 14px 14px 0; }
.comp-avatar {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: var(--teal-d);
  border: 1px solid var(--teal-m);
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: var(--head);
  font-size: 13px;
  color: var(--teal);
  flex-shrink: 0;
}
.comp-input { flex: 1; min-width: 0; }
.comp-handle {
  font-family: var(--mono);
  font-size: 10px;
  color: var(--teal);
  margin-bottom: 4px;
}
.comp-input textarea {
  width: 100%;
  background: transparent;
  border: none;
  outline: none;
  color: var(--br);
  font-size: 13px;
  line-height: 1.6;
  resize: none;
  min-height: 44px;
  max-height: 200px;
}
.comp-img-preview {
  position: relative;
  margin: 8px 14px;
}
.comp-img-preview img {
  max-width: 100%;
  max-height: 200px;
  border-radius: var(--r);
  border: 1px solid var(--bdr);
}
.comp-img-rm {
  position: absolute;
  top: 4px;
  right: 4px;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  background: rgba(0,0,0,0.7);
  color: var(--br);
  font-size: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
}
.comp-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 14px;
  border-top: 1px solid var(--bdr);
}
.comp-actions { display: flex; gap: 4px; }
.comp-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: var(--r);
  color: var(--dim);
  cursor: pointer;
  transition: color 0.15s;
}
.comp-btn:hover { color: var(--br); }
.comp-right { display: flex; align-items: center; gap: 8px; }
.ttl-select {
  font-family: var(--mono);
  font-size: 10px;
  background: var(--dim2);
  color: var(--dim);
  border: 1px solid var(--bdr);
  border-radius: var(--r);
  padding: 4px 8px;
  cursor: pointer;
}
.post-btn {
  font-family: var(--head);
  font-size: 11px;
  letter-spacing: 0.12em;
  background: var(--teal);
  color: var(--bg);
  padding: 6px 16px;
  border-radius: var(--r);
  transition: opacity 0.15s;
}
.post-btn:hover { opacity: 0.9; }
.post-btn:disabled { opacity: 0.4; cursor: not-allowed; }

/* Posts */
.feed { display: flex; flex-direction: column; gap: 10px; }
.post {
  background: var(--surf);
  border: 1px solid var(--bdr);
  border-radius: 6px;
  padding: 14px;
}
.post-head {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}
.post-avatar {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background: var(--dim2);
  border: 1px solid var(--bdr);
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: var(--head);
  font-size: 11px;
  color: var(--dim);
  flex-shrink: 0;
}
.post-meta { flex: 1; min-width: 0; }
.post-author {
  font-size: 13px;
  font-weight: 700;
  color: var(--br);
  margin-right: 6px;
}
.post-handle {
  font-family: var(--mono);
  font-size: 10px;
  color: var(--dim);
}
.post-age {
  font-family: var(--mono);
  font-size: 10px;
  color: var(--dim);
  flex-shrink: 0;
}
.post-del {
  font-size: 16px;
  color: var(--dim);
  line-height: 1;
  flex-shrink: 0;
}
.post-del:hover { color: var(--red); }
.post-text {
  font-size: 13px;
  line-height: 1.7;
  color: var(--br);
  white-space: pre-wrap;
  word-break: break-word;
  margin-bottom: 8px;
}
.post-img {
  max-width: 100%;
  border-radius: var(--r);
  margin-bottom: 8px;
  border: 1px solid var(--bdr);
}

/* TTL bar */
.ttl-bar {
  height: 2px;
  background: var(--dim2);
  border-radius: 1px;
  overflow: hidden;
  margin-bottom: 8px;
}
.ttl-fill {
  height: 100%;
  background: var(--teal);
  border-radius: 1px;
  transition: width 5s linear;
}

/* Reactions */
.post-reactions { display: flex; gap: 6px; }
.react-btn {
  font-family: var(--mono);
  font-size: 10px;
  color: var(--dim);
  background: var(--dim2);
  border: 1px solid transparent;
  padding: 3px 8px;
  border-radius: 3px;
  display: flex;
  align-items: center;
  gap: 4px;
  transition: color 0.15s, border-color 0.15s;
}
.react-btn:hover { color: var(--br); border-color: var(--bdr2); }
.react-btn.mine { color: var(--teal); border-color: var(--teal-m); }
.react-ct { font-weight: 700; }

.empty-feed {
  text-align: center;
  padding: 48px 16px;
  color: var(--dim);
  font-size: 13px;
}
</style>
