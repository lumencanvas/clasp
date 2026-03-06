<script setup lang="ts">
import { ref } from 'vue'
import type { DirectLink, AnyProtocol } from '../../lib/types'
import { useBridges } from '../../composables/useBridges'
import { useNotifications } from '../../composables/useNotifications'
import { allProtocols, protocolNames, defaultAddresses } from '../../lib/constants'

const { add, remove, saveBridges, bridges } = useBridges()
const { notify } = useNotifications()

const dialogRef = ref<HTMLDialogElement | null>(null)
const isEdit = ref(false)
const editId = ref('')

const sourceProtocol = ref<AnyProtocol>('osc')
const sourceAddr = ref('')
const targetProtocol = ref<AnyProtocol>('clasp')
const targetAddr = ref('')

const protocols: AnyProtocol[] = [...allProtocols, 'clasp']

function open(link?: DirectLink) {
  if (link) {
    isEdit.value = true
    editId.value = link.id
    sourceProtocol.value = link.source
    sourceAddr.value = link.sourceAddr
    targetProtocol.value = link.target
    targetAddr.value = link.targetAddr
  } else {
    isEdit.value = false
    editId.value = ''
    sourceProtocol.value = 'osc'
    sourceAddr.value = defaultAddresses['osc'] || ''
    targetProtocol.value = 'clasp'
    targetAddr.value = defaultAddresses['clasp'] || ''
  }
  dialogRef.value?.showModal()
}

function close() {
  dialogRef.value?.close()
}

async function save() {
  if (!sourceAddr.value || !targetAddr.value) {
    notify('Source and target addresses are required', 'error')
    return
  }

  try {
    if (isEdit.value) {
      // Remove old bridge and re-add with updated config
      await remove(editId.value)
      await add({
        source: sourceProtocol.value,
        sourceAddr: sourceAddr.value,
        target: targetProtocol.value,
        targetAddr: targetAddr.value,
      })
      notify('Direct link updated', 'success')
    } else {
      await add({
        source: sourceProtocol.value,
        sourceAddr: sourceAddr.value,
        target: targetProtocol.value,
        targetAddr: targetAddr.value,
      })
      notify('Direct link created', 'success')
    }
    close()
  } catch (e: any) {
    notify(`Failed: ${e.message || e}`, 'error')
  }
}

function onSourceProtocolChange() {
  sourceAddr.value = defaultAddresses[sourceProtocol.value] || ''
}

function onTargetProtocolChange() {
  targetAddr.value = defaultAddresses[targetProtocol.value] || ''
}

defineExpose({ open, close })
</script>

<template>
  <dialog ref="dialogRef" class="modal" @click.self="close">
    <div class="modal-content">
      <div class="modal-header">
        <span class="modal-title">{{ isEdit ? 'EDIT DIRECT LINK' : 'NEW DIRECT LINK' }}</span>
        <button class="modal-close" @click="close">&times;</button>
      </div>
      <form @submit.prevent="save">
        <div class="form-group">
          <label class="form-label">Source Protocol</label>
          <select v-model="sourceProtocol" class="select" @change="onSourceProtocolChange">
            <option v-for="p in protocols" :key="p" :value="p">{{ protocolNames[p] || p }}</option>
          </select>
        </div>
        <div class="form-group">
          <label class="form-label">Source Address</label>
          <input v-model="sourceAddr" class="input" placeholder="e.g. 0.0.0.0:9000" />
        </div>

        <div class="form-divider-arrow">
          <span class="arrow-icon">&#x2193;</span>
        </div>

        <div class="form-group">
          <label class="form-label">Target Protocol</label>
          <select v-model="targetProtocol" class="select" @change="onTargetProtocolChange">
            <option v-for="p in protocols" :key="p" :value="p">{{ protocolNames[p] || p }}</option>
          </select>
        </div>
        <div class="form-group">
          <label class="form-label">Target Address</label>
          <input v-model="targetAddr" class="input" placeholder="e.g. localhost:7330" />
        </div>

        <div class="modal-actions">
          <button type="button" class="btn btn-secondary" @click="close">CANCEL</button>
          <button type="submit" class="btn btn-primary">{{ isEdit ? 'SAVE' : 'CREATE' }}</button>
        </div>
      </form>
    </div>
  </dialog>
</template>
