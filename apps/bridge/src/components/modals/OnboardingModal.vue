<script setup lang="ts">
import { ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useOnboarding } from '../../composables/useOnboarding'

const { visible, step, finish: finishOnboarding, skip: skipOnboarding, nextStep, prevStep, selectUseCase, selectedUseCase, applyPreset } = useOnboarding()
const vueRouter = useRouter()

const dialogRef = ref<HTMLDialogElement | null>(null)

watch(visible, (show) => {
  if (show) dialogRef.value?.showModal()
  else dialogRef.value?.close()
})

function skip() {
  skipOnboarding()
}

async function finish() {
  if (selectedUseCase.value) {
    await applyPreset(selectedUseCase.value)
  }
  finishOnboarding()
  vueRouter.push('/flow')
}
</script>

<template>
  <dialog ref="dialogRef" class="modal onboarding-modal" @click.self="skip">
    <div class="modal-content">
      <div class="modal-header">
        <span class="modal-title">WELCOME TO CLASP BRIDGE</span>
        <button class="modal-close" @click="skip">&times;</button>
      </div>
      <div class="onboarding-wizard">
        <!-- Step 1: Welcome -->
        <div class="onboarding-step" :class="{ active: step === 0 }">
          <div class="onboarding-icon">
            <svg width="64" height="64" viewBox="0 0 32 32" fill="none">
              <path d="M 4 16 L 4 6 Q 4 2, 8 2 L 13 2 L 13 5 L 9 5 Q 7 5, 7 8 L 7 16 L 7 24 Q 7 27, 9 27 L 13 27 L 13 30 L 8 30 Q 4 30, 4 26 Z" fill="currentColor"/>
              <path d="M 28 16 L 28 6 Q 28 2, 24 2 L 19 2 L 19 5 L 23 5 Q 25 5, 25 8 L 25 16 L 25 24 Q 25 27, 23 27 L 19 27 L 19 30 L 24 30 Q 28 30, 28 26 Z" fill="currentColor"/>
              <line x1="7" y1="11" x2="25" y2="11" stroke="currentColor" stroke-width="2"/>
              <line x1="7" y1="16" x2="25" y2="16" stroke="currentColor" stroke-width="2"/>
              <line x1="7" y1="21" x2="25" y2="21" stroke="currentColor" stroke-width="2"/>
            </svg>
          </div>
          <h2 class="onboarding-title">CLASP Bridge</h2>
          <p class="onboarding-text">
            Connect and route signals between protocols like OSC, MIDI, Art-Net, DMX, MQTT, and more.
          </p>
          <div class="onboarding-features">
            <span class="onboarding-feature">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg>
              9 Protocols
            </span>
            <span class="onboarding-feature">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg>
              Signal Routing
            </span>
            <span class="onboarding-feature">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg>
              12 Transforms
            </span>
          </div>
        </div>

        <!-- Step 2: Use Case -->
        <div class="onboarding-step" :class="{ active: step === 1 }">
          <h2 class="onboarding-title">What are you building?</h2>
          <p class="onboarding-text">Pick a workflow to get started quickly, or skip to configure manually.</p>
          <div class="onboarding-use-cases">
            <button
              class="use-case-btn"
              :class="{ selected: selectedUseCase === 'vj-setup' }"
              @click="selectUseCase('vj-setup')"
            >
              <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="3" width="20" height="14" rx="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/></svg>
              <span class="use-case-title">VJ Setup</span>
              <span class="use-case-desc">OSC + MIDI to Art-Net</span>
            </button>
            <button
              class="use-case-btn"
              :class="{ selected: selectedUseCase === 'lighting-console' }"
              @click="selectUseCase('lighting-console')"
            >
              <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>
              <span class="use-case-title">Lighting</span>
              <span class="use-case-desc">sACN + Art-Net + DMX</span>
            </button>
            <button
              class="use-case-btn"
              :class="{ selected: selectedUseCase === 'minimal' }"
              @click="selectUseCase('minimal')"
            >
              <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="16"/><line x1="8" y1="12" x2="16" y2="12"/></svg>
              <span class="use-case-title">Manual</span>
              <span class="use-case-desc">I'll configure it myself</span>
            </button>
          </div>
        </div>

        <!-- Step 3: Done -->
        <div class="onboarding-step" :class="{ active: step === 2 }">
          <div class="onboarding-icon success">
            <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg>
          </div>
          <h2 class="onboarding-title">You're all set!</h2>
          <p class="onboarding-text">
            {{ selectedUseCase && selectedUseCase !== 'minimal' ? 'Your preset has been applied. Check the Flow tab to see your setup.' : 'Start by adding a router and connections from the sidebar.' }}
          </p>
          <div class="onboarding-next-steps">
            <h4>NEXT STEPS</h4>
            <ul>
              <li>Add connections from the sidebar</li>
              <li>Create signal routes to transform data</li>
              <li>Use the Monitor tab to see signals in real-time</li>
            </ul>
          </div>
        </div>
      </div>

      <div class="onboarding-nav">
        <div class="onboarding-dots">
          <span v-for="i in 3" :key="i" class="onboarding-dot" :class="{ active: step === i - 1, completed: step > i - 1 }"></span>
        </div>
        <div class="onboarding-buttons">
          <button v-if="step > 0" class="btn btn-secondary" @click="prevStep">BACK</button>
          <button v-if="step < 2" class="btn btn-primary" @click="nextStep">NEXT</button>
          <button v-if="step === 2" class="btn btn-primary" @click="finish">GET STARTED</button>
          <button class="btn btn-secondary" @click="skip">SKIP</button>
        </div>
      </div>
    </div>
  </dialog>
</template>
