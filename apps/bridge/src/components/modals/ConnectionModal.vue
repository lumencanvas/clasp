<script setup lang="ts">
import { ref, computed, watch, markRaw, type Component } from 'vue'
import type { Connection, Protocol } from '../../lib/types'
import { useConnections } from '../../composables/useConnections'
import { useNotifications } from '../../composables/useNotifications'
import { allProtocols, protocolNames, protocolHints } from '../../lib/constants'
import RouterSelector from '../forms/RouterSelector.vue'
import OscForm from '../forms/OscForm.vue'
import MidiForm from '../forms/MidiForm.vue'
import MqttForm from '../forms/MqttForm.vue'
import WebSocketForm from '../forms/WebSocketForm.vue'
import SocketIoForm from '../forms/SocketIoForm.vue'
import HttpForm from '../forms/HttpForm.vue'
import ArtNetForm from '../forms/ArtNetForm.vue'
import SacnForm from '../forms/SacnForm.vue'
import DmxForm from '../forms/DmxForm.vue'

const formRegistry: Record<Protocol, Component> = {
  osc: markRaw(OscForm),
  midi: markRaw(MidiForm),
  mqtt: markRaw(MqttForm),
  websocket: markRaw(WebSocketForm),
  socketio: markRaw(SocketIoForm),
  http: markRaw(HttpForm),
  artnet: markRaw(ArtNetForm),
  sacn: markRaw(SacnForm),
  dmx: markRaw(DmxForm),
}

const { add, edit: editConn } = useConnections()
const { notify } = useNotifications()

const dialogRef = ref<HTMLDialogElement | null>(null)
const isEdit = ref(false)
const editId = ref('')

const name = ref('')
const protocol = ref<Protocol>('osc')
const routerId = ref('')
const config = ref<Record<string, any>>({})

const currentForm = computed(() => formRegistry[protocol.value])
const currentHint = computed(() => protocolHints[protocol.value] || '')

function open(connection?: Connection) {
  if (connection) {
    isEdit.value = true
    editId.value = connection.id
    name.value = connection.name
    protocol.value = (connection.protocol || connection.type || 'osc') as Protocol
    routerId.value = connection.routerId || ''
    config.value = { ...connection }
  } else {
    isEdit.value = false
    editId.value = ''
    name.value = ''
    protocol.value = 'osc'
    routerId.value = ''
    config.value = {}
  }
  dialogRef.value?.showModal()
}

function close() {
  dialogRef.value?.close()
}

async function save() {
  try {
    const connConfig = {
      ...config.value,
      name: name.value || `${protocolNames[protocol.value] || protocol.value} Connection`,
      protocol: protocol.value,
      type: protocol.value,
      routerId: routerId.value || undefined,
    }
    if (isEdit.value) {
      editConn(editId.value)
      await add({ id: editId.value, ...connConfig })
      notify('Connection updated', 'success')
    } else {
      await add(connConfig)
      notify('Connection added', 'success')
    }
    close()
  } catch (e: any) {
    notify(`Failed: ${e.message || e}`, 'error')
  }
}

defineExpose({ open, close })
</script>

<template>
  <dialog ref="dialogRef" class="modal modal-lg" @click.self="close">
    <div class="modal-content">
      <div class="modal-header">
        <span class="modal-title">{{ isEdit ? 'EDIT CONNECTION' : 'NEW CONNECTION' }}</span>
        <button class="modal-close" @click="close">&times;</button>
      </div>
      <form @submit.prevent="save">
        <div class="form-group">
          <label class="form-label">Name</label>
          <input v-model="name" class="input" placeholder="Connection name" />
        </div>
        <div class="form-row">
          <div class="form-group">
            <label class="form-label">Protocol</label>
            <select v-model="protocol" class="select" :disabled="isEdit">
              <option v-for="p in allProtocols" :key="p" :value="p">{{ protocolNames[p] || p }}</option>
            </select>
          </div>
          <div class="form-group">
            <label class="form-label">Router</label>
            <RouterSelector v-model="routerId" />
          </div>
        </div>
        <div v-if="currentHint" class="form-hint" style="margin-bottom: var(--space-md);">
          {{ currentHint }}
        </div>
        <component
          :is="currentForm"
          v-model="config"
        />
        <div class="modal-actions">
          <button type="button" class="btn btn-secondary" @click="close">CANCEL</button>
          <button type="submit" class="btn btn-primary">{{ isEdit ? 'SAVE' : 'CREATE' }}</button>
        </div>
      </form>
    </div>
  </dialog>
</template>
