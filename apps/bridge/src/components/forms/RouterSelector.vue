<script setup lang="ts">
import { useRouters } from '../../composables/useRouters'

defineProps<{ modelValue: string }>()
const emit = defineEmits<{ 'update:modelValue': [value: string] }>()

const { availableRouters } = useRouters()
</script>

<template>
  <div class="form-group">
    <label class="form-label">Router</label>
    <select
      class="select"
      :value="modelValue"
      @change="emit('update:modelValue', ($event.target as HTMLSelectElement).value)"
    >
      <option value="">-- No Router --</option>
      <option
        v-for="router in availableRouters"
        :key="router.id"
        :value="router.id"
      >
        {{ router.name }} ({{ router.status }})
      </option>
    </select>
  </div>
</template>
