<script setup>
import { useToast } from '../../composables/useToast.js'
const { toasts } = useToast()
</script>

<template>
  <div class="toasts">
    <div v-for="t in toasts" :key="t.id" class="toast" :class="[t.type, { out: t.out }]">{{ t.msg }}</div>
  </div>
</template>
