<script setup lang="ts">
import type { Device } from '../../lib/types'
import { protocolNames } from '../../lib/constants'
import { useRouters } from '../../composables/useRouters'
import StatusDot from '../shared/StatusDot.vue'
import ProtocolBadge from '../shared/ProtocolBadge.vue'

const props = defineProps<{ device: Device }>()
const { addRemote } = useRouters()

const address = props.device.address || props.device.host || ''
const port = props.device.port || 7330
const displayAddress = address ? (address.includes(':') ? address : `${address}:${port}`) : ''

function handleClick() {
  addRemote(props.device)
}
</script>

<template>
  <div class="device-item device-item-clickable" @click="handleClick" :title="`Click to add as connection${displayAddress ? ` (${displayAddress})` : ''}`">
    <StatusDot :status="device.status || 'available'" />
    <ProtocolBadge :protocol="device.protocol || 'clasp'" />
    <span class="device-name">{{ device.name }}</span>
    <svg class="device-connect-icon" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
    </svg>
  </div>
</template>
