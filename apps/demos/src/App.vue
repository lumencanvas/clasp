<script setup>
import DemoNav from './components/DemoNav.vue'
import AuthModal from './components/AuthModal.vue'
import { ref } from 'vue'

const showAuth = ref(false)
</script>

<template>
  <DemoNav @auth="showAuth = true" />
  <router-view />
  <AuthModal v-if="showAuth" @close="showAuth = false" />
</template>
