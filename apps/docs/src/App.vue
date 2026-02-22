<script setup>
import NavBar from './components/NavBar.vue'
import FooterSection from './components/FooterSection.vue'
import SearchModal from './components/SearchModal.vue'
</script>

<template>
  <NavBar />
  <SearchModal />
  <router-view />
  <FooterSection />
</template>
