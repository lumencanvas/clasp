<script setup>
import { ref, onMounted } from 'vue'

const props = defineProps({
  show: Boolean,
  meta: Object,
  status: String,
  viewerCount: Number,
})
const emit = defineEmits(['close', 'end', 'video-ready'])

const videoEl = ref(null)

// Emit video-ready once on mount -- the element always exists (v-show, not v-if)
onMounted(() => {
  if (videoEl.value) emit('video-ready', videoEl.value)
})

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
