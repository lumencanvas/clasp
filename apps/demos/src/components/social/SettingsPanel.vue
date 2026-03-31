<script setup>
import { ref, watch } from 'vue'

const props = defineProps({ show: Boolean, name: String, handle: String })
const emit = defineEmits(['close', 'save'])

const localName = ref('')
const localHandle = ref('')

watch(() => props.show, (v) => {
  if (v) { localName.value = props.name; localHandle.value = props.handle.replace(/^@/, '') }
})

function save() {
  emit('save', { name: localName.value.trim(), handle: localHandle.value.trim() })
}
</script>

<template>
  <div v-if="show" class="spnl" @click.self="emit('close')">
    <div class="sbk" @click="emit('close')"></div>
    <div class="sbox">
      <div class="shd"><h3>IDENTITY</h3><button class="ib" @click="emit('close')">&times;</button></div>
      <div class="srow"><label>display name</label><input v-model="localName" placeholder="your name" maxlength="32" /></div>
      <div class="srow"><label>handle</label><input v-model="localHandle" placeholder="yourhandle" maxlength="24" /></div>
      <div class="snote">
        <strong>channels:</strong> append <code>#ch=roomname</code> to the URL.<br><br>
        <strong>relay:</strong> demo-relay.clasp.to -- CLASP router with auth. posts use SET (persisted until TTL). reactions + live signaling use emit (ephemeral). live video is WebRTC P2P.<br><br>
        <strong>images:</strong> compressed to ~60KB client-side, sent as base64 in the CLASP payload. expire with the post.
      </div>
      <button class="ssave" @click="save">SAVE</button>
    </div>
  </div>
</template>
