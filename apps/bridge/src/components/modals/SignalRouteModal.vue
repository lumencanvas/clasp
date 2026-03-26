<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import type { SignalRoute, SignalEndpoint, TransformConfig, TransformType, AnyProtocol, Signal } from '../../lib/types'
import { useRoutes } from '../../composables/useRoutes'
import { useLearnMode } from '../../composables/useLearnMode'
import { useNotifications } from '../../composables/useNotifications'
import { allProtocols, protocolNames, transformTypes, defaultAddresses } from '../../lib/constants'

const { add, edit: editRoute } = useRoutes()
const learn = useLearnMode()
const { notify } = useNotifications()

const dialogRef = ref<HTMLDialogElement | null>(null)
const isEdit = ref(false)
const editId = ref('')

const protocols: AnyProtocol[] = [...allProtocols, 'clasp']

// Source fields
const srcProtocol = ref<AnyProtocol>('osc')
const srcAddress = ref('')
const srcMidiType = ref('')
const srcMidiChannel = ref<number | null>(null)
const srcMidiNumber = ref<number | null>(null)
const srcDmxUniverse = ref<number | null>(null)
const srcDmxChannel = ref<number | null>(null)
const srcValueType = ref('')
const srcJsonPath = ref('')

// Transform fields
const transformType = ref<TransformType>('direct')
const scaleInMin = ref(0)
const scaleInMax = ref(1)
const scaleOutMin = ref(0)
const scaleOutMax = ref(1)
const clampMin = ref(0)
const clampMax = ref(1)
const threshold = ref(0.5)
const expression = ref('')
const javascriptCode = ref('')
const deadzoneMin = ref(0.4)
const deadzoneMax = ref(0.6)
const smoothFactor = ref(0.3)
const quantizeSteps = ref(8)
const curveType = ref<string>('linear')
const moduloDivisor = ref(1)
const powerExponent = ref(2)

// Target fields
const tgtProtocol = ref<AnyProtocol>('clasp')
const tgtAddress = ref('')
const tgtMidiType = ref('')
const tgtMidiChannel = ref<number | null>(null)
const tgtMidiNumber = ref<number | null>(null)
const tgtDmxUniverse = ref<number | null>(null)
const tgtDmxChannel = ref<number | null>(null)
const tgtValueType = ref('')
const tgtJsonPath = ref('')
const tgtJsonTemplate = ref('')

const isMidiSource = computed(() => srcProtocol.value === 'midi')
const isDmxSource = computed(() => ['artnet', 'sacn', 'dmx'].includes(srcProtocol.value))
const isMidiTarget = computed(() => tgtProtocol.value === 'midi')
const isDmxTarget = computed(() => ['artnet', 'sacn', 'dmx'].includes(tgtProtocol.value))

const transformPreview = computed(() => {
  switch (transformType.value) {
    case 'direct': return 'out = in'
    case 'scale': return `out = ${scaleOutMin.value} + (in - ${scaleInMin.value}) / (${scaleInMax.value} - ${scaleInMin.value}) * (${scaleOutMax.value} - ${scaleOutMin.value})`
    case 'invert': return 'out = 1 - in'
    case 'clamp': return `out = clamp(in, ${clampMin.value}, ${clampMax.value})`
    case 'round': return 'out = round(in)'
    case 'threshold': return `out = in >= ${threshold.value} ? 1 : 0`
    case 'gate': return 'out = in > 0 ? 1 : 0'
    case 'trigger': return 'out = 1 (on any input)'
    case 'toggle': return 'out = toggle on > 0.5'
    case 'expression': return `out = ${expression.value || '...'}`
    case 'javascript': return 'out = fn(in)'
    case 'deadzone': return `out = in in [${deadzoneMin.value}, ${deadzoneMax.value}] ? 0 : in`
    case 'smooth': return `out = EMA(in, factor=${smoothFactor.value})`
    case 'quantize': return `out = round(in * ${quantizeSteps.value}) / ${quantizeSteps.value}`
    case 'curve': return `out = ${curveType.value}(in)`
    case 'modulo': return `out = in % ${moduloDivisor.value}`
    case 'negate': return 'out = -in'
    case 'power': return `out = in ^ ${powerExponent.value}`
    default: return ''
  }
})

function buildSource(): SignalEndpoint {
  const ep: SignalEndpoint = { protocol: srcProtocol.value }
  if (srcAddress.value) ep.address = srcAddress.value
  if (isMidiSource.value) {
    if (srcMidiType.value) ep.midiType = srcMidiType.value
    if (srcMidiChannel.value != null) ep.midiChannel = srcMidiChannel.value
    if (srcMidiNumber.value != null) ep.midiNumber = srcMidiNumber.value
  }
  if (isDmxSource.value) {
    if (srcDmxUniverse.value != null) ep.dmxUniverse = srcDmxUniverse.value
    if (srcDmxChannel.value != null) ep.dmxChannel = srcDmxChannel.value
  }
  if (srcValueType.value) ep.valueType = srcValueType.value
  if (srcJsonPath.value) ep.jsonPath = srcJsonPath.value
  return ep
}

function buildTarget(): SignalEndpoint {
  const ep: SignalEndpoint = { protocol: tgtProtocol.value }
  if (tgtAddress.value) ep.address = tgtAddress.value
  if (isMidiTarget.value) {
    if (tgtMidiType.value) ep.midiType = tgtMidiType.value
    if (tgtMidiChannel.value != null) ep.midiChannel = tgtMidiChannel.value
    if (tgtMidiNumber.value != null) ep.midiNumber = tgtMidiNumber.value
  }
  if (isDmxTarget.value) {
    if (tgtDmxUniverse.value != null) ep.dmxUniverse = tgtDmxUniverse.value
    if (tgtDmxChannel.value != null) ep.dmxChannel = tgtDmxChannel.value
  }
  if (tgtValueType.value) ep.valueType = tgtValueType.value
  if (tgtJsonPath.value) ep.jsonPath = tgtJsonPath.value
  if (tgtJsonTemplate.value) ep.jsonTemplate = tgtJsonTemplate.value
  return ep
}

function buildTransform(): TransformConfig {
  const cfg: TransformConfig = { type: transformType.value }
  switch (transformType.value) {
    case 'scale':
      cfg.scaleInMin = scaleInMin.value
      cfg.scaleInMax = scaleInMax.value
      cfg.scaleOutMin = scaleOutMin.value
      cfg.scaleOutMax = scaleOutMax.value
      break
    case 'clamp':
      cfg.clampMin = clampMin.value
      cfg.clampMax = clampMax.value
      break
    case 'threshold':
      cfg.threshold = threshold.value
      break
    case 'expression':
      cfg.expression = expression.value
      break
    case 'javascript':
      cfg.javascriptCode = javascriptCode.value
      break
    case 'deadzone':
      cfg.deadzoneMin = deadzoneMin.value
      cfg.deadzoneMax = deadzoneMax.value
      break
    case 'smooth':
      cfg.smoothFactor = smoothFactor.value
      break
    case 'quantize':
      cfg.quantizeSteps = quantizeSteps.value
      break
    case 'curve':
      cfg.curveType = curveType.value as any
      break
    case 'modulo':
      cfg.moduloDivisor = moduloDivisor.value
      break
    case 'power':
      cfg.powerExponent = powerExponent.value
      break
  }
  return cfg
}

function loadEndpoint(ep: SignalEndpoint, side: 'source' | 'target') {
  if (side === 'source') {
    srcProtocol.value = ep.protocol
    srcAddress.value = ep.address || ''
    srcMidiType.value = ep.midiType || ''
    srcMidiChannel.value = ep.midiChannel ?? null
    srcMidiNumber.value = ep.midiNumber ?? null
    srcDmxUniverse.value = ep.dmxUniverse ?? null
    srcDmxChannel.value = ep.dmxChannel ?? null
    srcValueType.value = ep.valueType || ''
    srcJsonPath.value = ep.jsonPath || ''
  } else {
    tgtProtocol.value = ep.protocol
    tgtAddress.value = ep.address || ''
    tgtMidiType.value = ep.midiType || ''
    tgtMidiChannel.value = ep.midiChannel ?? null
    tgtMidiNumber.value = ep.midiNumber ?? null
    tgtDmxUniverse.value = ep.dmxUniverse ?? null
    tgtDmxChannel.value = ep.dmxChannel ?? null
    tgtValueType.value = ep.valueType || ''
    tgtJsonPath.value = ep.jsonPath || ''
    tgtJsonTemplate.value = ep.jsonTemplate || ''
  }
}

function loadTransform(cfg: TransformConfig) {
  transformType.value = cfg.type || 'direct'
  scaleInMin.value = cfg.scaleInMin ?? 0
  scaleInMax.value = cfg.scaleInMax ?? 1
  scaleOutMin.value = cfg.scaleOutMin ?? 0
  scaleOutMax.value = cfg.scaleOutMax ?? 1
  clampMin.value = cfg.clampMin ?? 0
  clampMax.value = cfg.clampMax ?? 1
  threshold.value = cfg.threshold ?? 0.5
  expression.value = cfg.expression || ''
  javascriptCode.value = cfg.javascriptCode || ''
  deadzoneMin.value = cfg.deadzoneMin ?? 0.4
  deadzoneMax.value = cfg.deadzoneMax ?? 0.6
  smoothFactor.value = cfg.smoothFactor ?? 0.3
  quantizeSteps.value = cfg.quantizeSteps ?? 8
  curveType.value = cfg.curveType ?? 'linear'
  moduloDivisor.value = cfg.moduloDivisor ?? 1
  powerExponent.value = cfg.powerExponent ?? 2
}

function resetForm() {
  srcProtocol.value = 'osc'
  srcAddress.value = ''
  srcMidiType.value = ''
  srcMidiChannel.value = null
  srcMidiNumber.value = null
  srcDmxUniverse.value = null
  srcDmxChannel.value = null
  srcValueType.value = ''
  srcJsonPath.value = ''

  transformType.value = 'direct'
  scaleInMin.value = 0
  scaleInMax.value = 1
  scaleOutMin.value = 0
  scaleOutMax.value = 1
  clampMin.value = 0
  clampMax.value = 1
  threshold.value = 0.5
  expression.value = ''
  javascriptCode.value = ''
  deadzoneMin.value = 0.4
  deadzoneMax.value = 0.6
  smoothFactor.value = 0.3
  quantizeSteps.value = 8
  curveType.value = 'linear'
  moduloDivisor.value = 1
  powerExponent.value = 2

  tgtProtocol.value = 'clasp'
  tgtAddress.value = ''
  tgtMidiType.value = ''
  tgtMidiChannel.value = null
  tgtMidiNumber.value = null
  tgtDmxUniverse.value = null
  tgtDmxChannel.value = null
  tgtValueType.value = ''
  tgtJsonPath.value = ''
  tgtJsonTemplate.value = ''
}

function open(route?: SignalRoute) {
  if (route) {
    isEdit.value = true
    editId.value = route.id
    loadEndpoint(route.source, 'source')
    loadEndpoint(route.target, 'target')
    loadTransform(route.transform)
  } else {
    isEdit.value = false
    editId.value = ''
    resetForm()
  }
  dialogRef.value?.showModal()
}

function close() {
  learn.stop()
  dialogRef.value?.close()
}

function save() {
  try {
    const route: SignalRoute = {
      id: isEdit.value ? editId.value : Date.now().toString(),
      enabled: true,
      source: buildSource(),
      target: buildTarget(),
      transform: buildTransform(),
    }

    if (isEdit.value) {
      editRoute(editId.value)
    }
    add(route)
    notify(isEdit.value ? 'Signal route updated' : 'Signal route created', 'success')
    close()
  } catch (e: any) {
    notify(`Failed: ${e.message || e}`, 'error')
  }
}

function startLearn() {
  learn.start('source')
  if (window.clasp?.startLearnMode) {
    window.clasp.startLearnMode('source')
  }
}

// Watch for learned signal and auto-fill source fields
watch(() => learn.learnedSignal.value, (signal: Signal | null) => {
  if (!signal) return
  const protocol = learn.detectProtocol(signal) as AnyProtocol
  srcProtocol.value = protocol
  if (signal.address) srcAddress.value = signal.address
  if (signal.channel !== undefined) srcMidiChannel.value = signal.channel
  if (signal.note !== undefined) srcMidiNumber.value = signal.note
  if (signal.cc !== undefined) srcMidiNumber.value = signal.cc
  if (signal.universe !== undefined) srcDmxUniverse.value = signal.universe
})

defineExpose({ open, close })
</script>

<template>
  <dialog ref="dialogRef" class="modal modal-lg" @click.self="close">
    <div class="modal-content">
      <div class="modal-header">
        <span class="modal-title">{{ isEdit ? 'EDIT SIGNAL ROUTE' : 'NEW SIGNAL ROUTE' }}</span>
        <button class="modal-close" @click="close">&times;</button>
      </div>
      <form @submit.prevent="save">
        <div class="mapping-editor-v2">
          <!-- Source Panel -->
          <div class="mapping-panel">
            <div class="mapping-panel-header">
              <span>SOURCE</span>
              <button
                type="button"
                class="btn btn-secondary btn-sm"
                :class="{ active: learn.active.value }"
                @click="startLearn"
              >
                {{ learn.active.value ? 'LISTENING...' : 'LEARN' }}
              </button>
            </div>
            <div class="mapping-panel-body">
              <div class="form-group">
                <label class="form-label">Protocol</label>
                <select v-model="srcProtocol" class="select">
                  <option v-for="p in protocols" :key="p" :value="p">{{ protocolNames[p] || p }}</option>
                </select>
              </div>
              <div class="form-group">
                <label class="form-label">Address</label>
                <input v-model="srcAddress" class="input" placeholder="/osc/fader1" />
              </div>
              <template v-if="isMidiSource">
                <div class="form-group">
                  <label class="form-label">MIDI Type</label>
                  <select v-model="srcMidiType" class="select">
                    <option value="">Any</option>
                    <option value="noteon">Note On</option>
                    <option value="noteoff">Note Off</option>
                    <option value="cc">CC</option>
                    <option value="pitchbend">Pitch Bend</option>
                  </select>
                </div>
                <div class="form-row">
                  <div class="form-group">
                    <label class="form-label">Channel</label>
                    <input v-model.number="srcMidiChannel" class="input" type="number" min="1" max="16" placeholder="Any" />
                  </div>
                  <div class="form-group">
                    <label class="form-label">Number</label>
                    <input v-model.number="srcMidiNumber" class="input" type="number" min="0" max="127" placeholder="Any" />
                  </div>
                </div>
              </template>
              <template v-if="isDmxSource">
                <div class="form-row">
                  <div class="form-group">
                    <label class="form-label">Universe</label>
                    <input v-model.number="srcDmxUniverse" class="input" type="number" min="0" placeholder="Any" />
                  </div>
                  <div class="form-group">
                    <label class="form-label">Channel</label>
                    <input v-model.number="srcDmxChannel" class="input" type="number" min="1" max="512" placeholder="Any" />
                  </div>
                </div>
              </template>
              <div class="form-group">
                <label class="form-label">JSON Path</label>
                <input v-model="srcJsonPath" class="input" placeholder="$.value (optional)" />
              </div>
            </div>
          </div>

          <!-- Transform Panel -->
          <div class="mapping-panel">
            <div class="mapping-panel-header">
              <span>TRANSFORM</span>
            </div>
            <div class="mapping-panel-body">
              <div class="form-group">
                <label class="form-label">Type</label>
                <select v-model="transformType" class="select">
                  <option v-for="t in transformTypes" :key="t.value" :value="t.value">{{ t.label }}</option>
                </select>
              </div>

              <template v-if="transformType === 'scale'">
                <div class="form-row">
                  <div class="form-group">
                    <label class="form-label">In Min</label>
                    <input v-model.number="scaleInMin" class="input" type="number" step="any" />
                  </div>
                  <div class="form-group">
                    <label class="form-label">In Max</label>
                    <input v-model.number="scaleInMax" class="input" type="number" step="any" />
                  </div>
                </div>
                <div class="form-row">
                  <div class="form-group">
                    <label class="form-label">Out Min</label>
                    <input v-model.number="scaleOutMin" class="input" type="number" step="any" />
                  </div>
                  <div class="form-group">
                    <label class="form-label">Out Max</label>
                    <input v-model.number="scaleOutMax" class="input" type="number" step="any" />
                  </div>
                </div>
              </template>

              <template v-if="transformType === 'clamp'">
                <div class="form-row">
                  <div class="form-group">
                    <label class="form-label">Min</label>
                    <input v-model.number="clampMin" class="input" type="number" step="any" />
                  </div>
                  <div class="form-group">
                    <label class="form-label">Max</label>
                    <input v-model.number="clampMax" class="input" type="number" step="any" />
                  </div>
                </div>
              </template>

              <template v-if="transformType === 'threshold'">
                <div class="form-group">
                  <label class="form-label">Threshold</label>
                  <input v-model.number="threshold" class="input" type="number" step="any" />
                </div>
              </template>

              <template v-if="transformType === 'expression'">
                <div class="form-group">
                  <label class="form-label">Expression</label>
                  <input v-model="expression" class="input" placeholder="x * 2 + 1" />
                </div>
              </template>

              <template v-if="transformType === 'javascript'">
                <div class="form-group">
                  <label class="form-label">JavaScript Code</label>
                  <textarea v-model="javascriptCode" class="input" rows="4" placeholder="return value * 2;" />
                </div>
              </template>

              <template v-if="transformType === 'deadzone'">
                <div class="form-row">
                  <div class="form-group">
                    <label class="form-label">Min</label>
                    <input v-model.number="deadzoneMin" class="input" type="number" step="any" />
                  </div>
                  <div class="form-group">
                    <label class="form-label">Max</label>
                    <input v-model.number="deadzoneMax" class="input" type="number" step="any" />
                  </div>
                </div>
              </template>

              <template v-if="transformType === 'smooth'">
                <div class="form-group">
                  <label class="form-label">Smoothing Factor (0-1)</label>
                  <input v-model.number="smoothFactor" class="input" type="number" step="0.01" min="0" max="1" />
                </div>
              </template>

              <template v-if="transformType === 'quantize'">
                <div class="form-group">
                  <label class="form-label">Steps</label>
                  <input v-model.number="quantizeSteps" class="input" type="number" min="2" />
                </div>
              </template>

              <template v-if="transformType === 'curve'">
                <div class="form-group">
                  <label class="form-label">Curve Type</label>
                  <select v-model="curveType" class="select">
                    <option value="linear">Linear</option>
                    <option value="ease-in">Ease In</option>
                    <option value="ease-out">Ease Out</option>
                    <option value="ease-in-out">Ease In-Out</option>
                    <option value="exponential">Exponential</option>
                    <option value="logarithmic">Logarithmic</option>
                  </select>
                </div>
              </template>

              <template v-if="transformType === 'modulo'">
                <div class="form-group">
                  <label class="form-label">Divisor</label>
                  <input v-model.number="moduloDivisor" class="input" type="number" step="any" />
                </div>
              </template>

              <template v-if="transformType === 'power'">
                <div class="form-group">
                  <label class="form-label">Exponent</label>
                  <input v-model.number="powerExponent" class="input" type="number" step="any" />
                </div>
              </template>

              <div class="transform-preview">
                <span class="form-label">Preview:</span>
                <code>{{ transformPreview }}</code>
              </div>
            </div>
          </div>

          <!-- Target Panel -->
          <div class="mapping-panel">
            <div class="mapping-panel-header">
              <span>TARGET</span>
            </div>
            <div class="mapping-panel-body">
              <div class="form-group">
                <label class="form-label">Protocol</label>
                <select v-model="tgtProtocol" class="select">
                  <option v-for="p in protocols" :key="p" :value="p">{{ protocolNames[p] || p }}</option>
                </select>
              </div>
              <div class="form-group">
                <label class="form-label">Address</label>
                <input v-model="tgtAddress" class="input" placeholder="/dmx/channel/1" />
              </div>
              <template v-if="isMidiTarget">
                <div class="form-group">
                  <label class="form-label">MIDI Type</label>
                  <select v-model="tgtMidiType" class="select">
                    <option value="">Same as source</option>
                    <option value="noteon">Note On</option>
                    <option value="noteoff">Note Off</option>
                    <option value="cc">CC</option>
                    <option value="pitchbend">Pitch Bend</option>
                  </select>
                </div>
                <div class="form-row">
                  <div class="form-group">
                    <label class="form-label">Channel</label>
                    <input v-model.number="tgtMidiChannel" class="input" type="number" min="1" max="16" placeholder="Same" />
                  </div>
                  <div class="form-group">
                    <label class="form-label">Number</label>
                    <input v-model.number="tgtMidiNumber" class="input" type="number" min="0" max="127" placeholder="Same" />
                  </div>
                </div>
              </template>
              <template v-if="isDmxTarget">
                <div class="form-row">
                  <div class="form-group">
                    <label class="form-label">Universe</label>
                    <input v-model.number="tgtDmxUniverse" class="input" type="number" min="0" placeholder="0" />
                  </div>
                  <div class="form-group">
                    <label class="form-label">Channel</label>
                    <input v-model.number="tgtDmxChannel" class="input" type="number" min="1" max="512" placeholder="1" />
                  </div>
                </div>
              </template>
              <div class="form-group">
                <label class="form-label">JSON Template</label>
                <input v-model="tgtJsonTemplate" class="input" placeholder='{"value": $} (optional)' />
              </div>
            </div>
          </div>
        </div>

        <div class="modal-actions">
          <button type="button" class="btn btn-secondary" @click="close">CANCEL</button>
          <button type="submit" class="btn btn-primary">{{ isEdit ? 'SAVE' : 'CREATE' }}</button>
        </div>
      </form>
    </div>
  </dialog>
</template>
