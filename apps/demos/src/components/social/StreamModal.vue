<script setup>
import { ref, watch, nextTick, onMounted } from 'vue'

const props = defineProps({
  show: Boolean,
  meta: Object,
  status: String,
  viewerCount: Number,
  chatMessages: { type: Array, default: () => [] },
})
const emit = defineEmits(['close', 'end', 'video-ready', 'send-chat'])

const videoEl = ref(null)
const chatText = ref('')
const chatListEl = ref(null)

onMounted(() => {
  if (videoEl.value) emit('video-ready', videoEl.value)
})

// Auto-scroll chat
watch(() => props.chatMessages.length, () => {
  nextTick(() => { if (chatListEl.value) chatListEl.value.scrollTop = chatListEl.value.scrollHeight })
})

function sendChat() {
  const text = chatText.value.trim().slice(0, 200)
  if (!text) return
  emit('send-chat', text)
  chatText.value = ''
}

defineExpose({ videoEl })
</script>

<template>
  <div v-show="show" class="modal">
    <div class="mbk" @click="emit('close')"></div>
    <div class="mbox">
      <div class="mhd">
        <div class="mttl">
          <div class="mttl-name">{{ meta.name }}</div>
          <div class="msub">{{ meta.sub }}</div>
        </div>
        <div class="mbdge"><div class="mbdot"></div>LIVE</div>
        <button class="mclose" @click="emit('close')">&times;</button>
      </div>
      <div class="mvw">
        <video ref="videoEl" autoplay playsinline></video>
        <div class="shud">
          <div class="slive"><div class="sbdot"></div>LIVE</div>
          <div class="spill">{{ status }}</div>
        </div>
      </div>
      <div class="mchat">
        <div ref="chatListEl" class="mclst">
          <div v-if="!chatMessages.length" class="mc-empty">no messages yet</div>
          <div v-for="m in chatMessages" :key="m.id" class="mcmsg">
            <span class="mcn">{{ m.name }}</span>
            <span class="mct">{{ m.text }}</span>
          </div>
        </div>
        <div class="mci">
          <textarea v-model="chatText" placeholder="say something..." maxlength="200" rows="1" @keydown.enter.exact.prevent="sendChat"></textarea>
          <button class="mcsend" @click="sendChat">SEND</button>
        </div>
      </div>
      <div class="mft">
        <div class="msi">
          <div class="mstrm">{{ meta.name }}</div>
          <div v-if="meta.isSelf" class="mvcl">{{ viewerCount }} {{ viewerCount === 1 ? 'viewer' : 'viewers' }}</div>
        </div>
        <button v-if="meta.isSelf" class="endbtn" @click="emit('end')">END STREAM</button>
      </div>
    </div>
  </div>
</template>
