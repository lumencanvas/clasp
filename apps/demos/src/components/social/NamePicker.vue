<script setup>
import { ref } from 'vue'

const emit = defineEmits(['commit', 'skip'])
const name = ref('')

function commit() {
  emit('commit', name.value.trim())
}
</script>

<template>
  <div class="npick">
    <div class="npbox">
      <div class="npttl">RELAY</div>
      <div class="npsub">ephemeral signals on the CLASP protocol<br>pick a name to get started</div>
      <div class="nprow">
        <label>your name</label>
        <input v-model="name" placeholder="enter a display name" maxlength="32" @keydown.enter="commit" autofocus />
      </div>
      <button class="npbtn" @click="commit">JOIN THE RELAY</button>
      <button class="npskip" @click="emit('skip')">skip -- stay anonymous</button>
    </div>
  </div>
</template>
