<script setup lang="ts">
import { protocolNames } from '../../lib/constants'

const props = defineProps<{
  protocol: string
  remote?: boolean
}>()
</script>

<template>
  <span class="device-protocol-badge" :class="remote ? 'remote' : protocol">
    {{ remote ? 'REMOTE' : (protocolNames[protocol as keyof typeof protocolNames] || protocol.toUpperCase()) }}
  </span>
</template>
