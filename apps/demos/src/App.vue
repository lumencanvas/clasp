<script setup>
import DemoNav from './components/DemoNav.vue'
import AuthModal from './components/AuthModal.vue'
import { ref, onMounted } from 'vue'
import { useRelay } from './composables/useRelay.js'

const { authToken } = useRelay()
const showAuth = ref(false)

onMounted(() => {
  if (!authToken.value) showAuth.value = true
})
</script>

<template>
  <DemoNav @auth="showAuth = true" />
  <router-view />
  <AuthModal v-if="showAuth" @close="showAuth = false" />
</template>
