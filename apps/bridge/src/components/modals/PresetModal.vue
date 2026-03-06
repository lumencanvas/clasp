<script setup lang="ts">
import { ref, computed } from 'vue'
import { presets, searchPresets, categories } from '../../presets/index.js'
import type { Preset } from '../../lib/types'

const emit = defineEmits<{
  select: [preset: Preset]
}>()

const dialogRef = ref<HTMLDialogElement | null>(null)
const searchQuery = ref('')
const activeCategory = ref<string | null>(null)

const categoryList = computed(() => {
  return Object.entries(categories).map(([key, meta]) => ({
    key,
    ...meta,
  }))
})

const filteredPresets = computed(() => {
  let results = searchQuery.value
    ? searchPresets(searchQuery.value)
    : [...presets]

  if (activeCategory.value) {
    results = results.filter((p: Preset) => p.category === activeCategory.value)
  }

  return results
})

function toggleCategory(key: string) {
  activeCategory.value = activeCategory.value === key ? null : key
}

function selectPreset(preset: Preset) {
  emit('select', preset)
  close()
}

function open() {
  searchQuery.value = ''
  activeCategory.value = null
  dialogRef.value?.showModal()
}

function close() {
  dialogRef.value?.close()
}

defineExpose({ open, close })
</script>

<template>
  <dialog ref="dialogRef" class="modal modal-lg" @click.self="close">
    <div class="modal-content preset-picker-content">
      <div class="modal-header">
        <span class="modal-title">PRESETS</span>
        <button class="modal-close" @click="close">&times;</button>
      </div>

      <div class="form-group">
        <input
          v-model="searchQuery"
          class="input"
          placeholder="Search presets..."
          type="search"
        />
      </div>

      <div class="preset-categories">
        <button
          v-for="cat in categoryList"
          :key="cat.key"
          class="btn btn-secondary btn-sm"
          :class="{ active: activeCategory === cat.key }"
          :style="activeCategory === cat.key ? { borderColor: cat.color, color: cat.color } : {}"
          @click="toggleCategory(cat.key)"
        >
          {{ cat.name }}
        </button>
      </div>

      <div class="preset-grid">
        <div
          v-for="preset in filteredPresets"
          :key="preset.id"
          class="preset-card"
          @click="selectPreset(preset as Preset)"
        >
          <div class="preset-card-title">{{ preset.name }}</div>
          <div class="preset-card-desc">{{ preset.description }}</div>
          <div class="preset-card-tags">
            <span v-for="tag in preset.tags" :key="tag" class="preset-tag">{{ tag }}</span>
          </div>
        </div>
      </div>

      <div v-if="filteredPresets.length === 0" class="empty-state">
        No presets match your search.
      </div>

      <div class="modal-actions">
        <button type="button" class="btn btn-secondary" @click="close">CLOSE</button>
      </div>
    </div>
  </dialog>
</template>
