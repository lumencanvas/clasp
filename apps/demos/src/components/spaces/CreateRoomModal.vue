<script setup>
import { ref } from 'vue'

const props = defineProps({ show: Boolean })
const emit = defineEmits(['close', 'create'])

const ICONS = ['\u25C9','\u2699','\u25C8','\u2318','\u2605','\u266B','\u2736','\u25CE','\u2622','\u269B','\u2764','\u2660']
const name = ref('')
const desc = ref('')
const icon = ref(ICONS[0])

function create() {
  if (!name.value.trim()) return
  emit('create', { name: name.value.trim(), desc: desc.value.trim(), icon: icon.value })
  name.value = ''; desc.value = ''; icon.value = ICONS[0]
}
</script>

<template>
  <div v-if="show" class="modal-overlay" @click.self="emit('close')">
    <div class="modal">
      <div class="modal-title">
        <span>Create a Room</span>
        <button class="modal-close" @click="emit('close')">&times;</button>
      </div>
      <div class="field">
        <label>Room Name</label>
        <input v-model="name" class="input" maxlength="48" placeholder="What's this room about?" @keydown.enter="create" autofocus />
      </div>
      <div class="field">
        <label>Description</label>
        <input v-model="desc" class="input" maxlength="120" placeholder="Optional description" />
      </div>
      <div class="field">
        <label>Icon</label>
        <div class="icon-picker">
          <button v-for="ic in ICONS" :key="ic" class="icon-opt" :class="{ selected: icon === ic }" @click="icon = ic">{{ ic }}</button>
        </div>
      </div>
      <div class="modal-actions">
        <button class="btn btn-ghost" @click="emit('close')">Cancel</button>
        <button class="btn btn-primary" @click="create">Go Live</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed; inset: 0; background: rgba(0,0,0,0.7);
  z-index: 50; display: flex; align-items: flex-end; justify-content: center;
  backdrop-filter: blur(4px);
}
.modal {
  max-width: 480px; width: 100%; background: var(--surface);
  border-radius: 16px 16px 0 0; padding: 20px 16px 24px;
  display: flex; flex-direction: column; gap: 16px;
  animation: slideUp 0.2s ease;
}
@keyframes slideUp { from { transform: translateY(100%); } to { transform: none; } }
.modal-title { display: flex; justify-content: space-between; align-items: center; font-size: 16px; font-weight: 700; color: var(--text); }
.modal-close { background: none; border: none; color: var(--text2); font-size: 20px; cursor: pointer; }
.field { display: flex; flex-direction: column; gap: 6px; }
.field label { font-family: var(--mono); font-size: 11px; font-weight: 700; text-transform: uppercase; color: var(--text2); }
.input { background: var(--surface); border: 1px solid var(--border); border-radius: 8px; padding: 10px 12px; color: var(--text); font-size: 14px; outline: none; transition: border-color 0.15s; }
.input:focus { border-color: var(--accent); }
.icon-picker { display: flex; flex-wrap: wrap; gap: 8px; }
.icon-opt {
  width: 40px; height: 40px; border-radius: 8px;
  background: var(--bg); border: 1px solid var(--border);
  color: var(--text2); font-size: 18px;
  display: flex; align-items: center; justify-content: center;
  cursor: pointer; transition: all 0.15s;
}
.icon-opt.selected { border-color: var(--accent); color: var(--accent); background: var(--accent-dim); }
.modal-actions { display: flex; gap: 8px; }
.btn { flex: 1; padding: 10px 16px; border-radius: 8px; font-size: 14px; font-weight: 600; cursor: pointer; transition: all 0.15s; border: none; }
.btn-primary { background: var(--accent); color: var(--bg); }
.btn-primary:hover { box-shadow: 0 0 16px rgba(0,229,200,0.3); }
.btn-ghost { background: var(--surface); border: 1px solid var(--border); color: var(--text2); }
</style>
