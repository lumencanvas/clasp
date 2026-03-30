<script setup>
import DemoNav from './components/DemoNav.vue'
import AuthModal from './components/AuthModal.vue'
import { useRelay } from './composables/useRelay.js'
import { ref } from 'vue'

const { authToken } = useRelay()
const showAuth = ref(false)

function onAuthRequired() {
  showAuth.value = true
}
</script>

<template>
  <DemoNav @auth="showAuth = true" />
  <router-view @auth-required="onAuthRequired" />
  <AuthModal v-if="showAuth" @close="showAuth = false" />
</template>
