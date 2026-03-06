<script setup lang="ts">
import { ref } from 'vue'

const props = defineProps<{
  title?: string
  message: string
  confirmLabel?: string
  cancelLabel?: string
}>()

const emit = defineEmits<{
  confirm: []
  cancel: []
}>()

const dialogRef = ref<HTMLDialogElement | null>(null)

function open() {
  dialogRef.value?.showModal()
}

function close() {
  dialogRef.value?.close()
}

function onConfirm() {
  emit('confirm')
  close()
}

function onCancel() {
  emit('cancel')
  close()
}

defineExpose({ open, close })
</script>

<template>
  <dialog ref="dialogRef" class="modal modal-sm" @click.self="onCancel">
    <div class="modal-content">
      <div class="modal-header">
        <span class="modal-title">{{ title || 'CONFIRM' }}</span>
        <button class="modal-close" @click="onCancel">&times;</button>
      </div>
      <div style="padding: var(--space-lg);">
        <p style="font-size: 13px;">{{ message }}</p>
      </div>
      <div class="modal-actions">
        <button class="btn btn-secondary" @click="onCancel">{{ cancelLabel || 'CANCEL' }}</button>
        <button class="btn btn-primary" @click="onConfirm">{{ confirmLabel || 'CONFIRM' }}</button>
      </div>
    </div>
  </dialog>
</template>
