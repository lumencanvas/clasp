<script setup>
import { ref, computed } from 'vue'
import { EMOJIS, EMOJI_CATEGORIES, CATEGORY_ICONS } from '../lib/emoji-data.js'

const emit = defineEmits(['select'])

const search = ref('')
const activeCategory = ref('Smileys')

// Recent emojis from localStorage
const RECENT_KEY = 'clasp-chat-recent-emojis'
const MAX_RECENT = 24
const recentEmojis = ref(JSON.parse(localStorage.getItem(RECENT_KEY) || '[]'))

const filteredEmojis = computed(() => {
  const q = search.value.toLowerCase().trim()
  if (q) {
    return EMOJIS.filter(e =>
      e.name.includes(q) ||
      e.keywords.some(k => k.includes(q)) ||
      e.emoji === q
    )
  }
  return EMOJIS.filter(e => e.category === activeCategory.value)
})

function selectEmoji(emoji) {
  emit('select', emoji)
  addRecent(emoji)
}

function addRecent(emoji) {
  const filtered = recentEmojis.value.filter(e => e !== emoji)
  filtered.unshift(emoji)
  recentEmojis.value = filtered.slice(0, MAX_RECENT)
  localStorage.setItem(RECENT_KEY, JSON.stringify(recentEmojis.value))
}
</script>

<template>
  <div class="emoji-picker" @click.stop>
    <div class="picker-search">
      <input
        v-model="search"
        type="text"
        placeholder="Search emoji..."
        autofocus
      />
    </div>

    <div class="picker-tabs">
      <button
        v-for="cat in EMOJI_CATEGORIES"
        :key="cat"
        :class="['tab-btn', { active: activeCategory === cat && !search }]"
        :title="cat"
        @click="activeCategory = cat; search = ''"
      >
        {{ CATEGORY_ICONS[cat] }}
      </button>
    </div>

    <div class="picker-grid">
      <!-- Recent section -->
      <template v-if="!search && activeCategory === 'Smileys' && recentEmojis.length">
        <div class="category-label">Recent</div>
        <div class="emoji-grid">
          <button
            v-for="emoji in recentEmojis"
            :key="'recent-' + emoji"
            class="emoji-cell"
            @click="selectEmoji(emoji)"
          >
            {{ emoji }}
          </button>
        </div>
        <div class="category-label">{{ activeCategory }}</div>
      </template>

      <div v-if="search" class="category-label">
        {{ filteredEmojis.length }} result{{ filteredEmojis.length !== 1 ? 's' : '' }}
      </div>

      <div class="emoji-grid">
        <button
          v-for="e in filteredEmojis"
          :key="e.emoji"
          class="emoji-cell"
          :title="e.name"
          @click="selectEmoji(e.emoji)"
        >
          {{ e.emoji }}
        </button>
      </div>

      <div v-if="filteredEmojis.length === 0" class="no-results">
        No emoji found
      </div>
    </div>
  </div>
</template>

<style scoped>
.emoji-picker {
  width: min(320px, calc(100vw - 1rem));
  max-height: 360px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 8px;
  box-shadow: 0 4px 16px rgba(0,0,0,0.3);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.picker-search {
  padding: 0.5rem;
  border-bottom: 1px solid var(--border);
}

.picker-search input {
  width: 100%;
  padding: 0.4rem 0.6rem;
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: 4px;
  font-size: 0.8rem;
}

.picker-search input:focus {
  outline: none;
  border-color: var(--accent);
}

.picker-tabs {
  display: flex;
  border-bottom: 1px solid var(--border);
  padding: 0 0.25rem;
}

.tab-btn {
  flex: 1;
  padding: 0.35rem 0;
  background: transparent;
  border: none;
  border-bottom: 2px solid transparent;
  font-size: 1rem;
  cursor: pointer;
  transition: all 0.1s;
  opacity: 0.5;
}

.tab-btn:hover {
  opacity: 0.8;
  background: var(--bg-tertiary);
}

.tab-btn.active {
  opacity: 1;
  border-bottom-color: var(--accent);
}

.picker-grid {
  flex: 1;
  overflow-y: auto;
  padding: 0.25rem 0.5rem;
}

.category-label {
  font-size: 0.75rem;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--text-muted);
  padding: 0.4rem 0.25rem 0.2rem;
  font-weight: 700;
}

.emoji-grid {
  display: grid;
  grid-template-columns: repeat(8, 1fr);
  gap: 1px;
}

.emoji-cell {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  aspect-ratio: 1;
  background: transparent;
  border: none;
  border-radius: 4px;
  font-size: 1.25rem;
  cursor: pointer;
  transition: background 0.1s;
}

.emoji-cell:hover {
  background: var(--bg-active);
}

.no-results {
  text-align: center;
  padding: 2rem;
  color: var(--text-muted);
  font-size: 0.8rem;
}
</style>
