<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { TransformConfig } from '../../lib/types'
import { transformTypes, curveTypes } from '../../lib/constants'
import { previewTransform } from '../../lib/transforms'

const props = defineProps<{ modelValue: TransformConfig }>()
const emit = defineEmits<{ 'update:modelValue': [value: TransformConfig] }>()

const testValue = ref(0.5)

const preview = computed(() => previewTransform(props.modelValue, testValue.value))

function update(field: string, value: any) {
  emit('update:modelValue', { ...props.modelValue, [field]: value })
}

function updateType(type: string) {
  emit('update:modelValue', { type: type as TransformConfig['type'] })
}
</script>

<template>
  <div class="transform-editor">
    <div class="form-group">
      <label class="form-label">Transform Type</label>
      <select
        class="select"
        :value="modelValue.type"
        @change="updateType(($event.target as HTMLSelectElement).value)"
      >
        <option
          v-for="t in transformTypes"
          :key="t.value"
          :value="t.value"
        >
          {{ t.label }}
        </option>
      </select>
    </div>

    <!-- Scale params -->
    <template v-if="modelValue.type === 'scale'">
      <div class="form-group">
        <label class="form-label">Input Min</label>
        <input
          class="input"
          type="number"
          step="any"
          :value="modelValue.scaleInMin ?? 0"
          @input="update('scaleInMin', Number(($event.target as HTMLInputElement).value))"
        />
      </div>
      <div class="form-group">
        <label class="form-label">Input Max</label>
        <input
          class="input"
          type="number"
          step="any"
          :value="modelValue.scaleInMax ?? 1"
          @input="update('scaleInMax', Number(($event.target as HTMLInputElement).value))"
        />
      </div>
      <div class="form-group">
        <label class="form-label">Output Min</label>
        <input
          class="input"
          type="number"
          step="any"
          :value="modelValue.scaleOutMin ?? 0"
          @input="update('scaleOutMin', Number(($event.target as HTMLInputElement).value))"
        />
      </div>
      <div class="form-group">
        <label class="form-label">Output Max</label>
        <input
          class="input"
          type="number"
          step="any"
          :value="modelValue.scaleOutMax ?? 127"
          @input="update('scaleOutMax', Number(($event.target as HTMLInputElement).value))"
        />
      </div>
    </template>

    <!-- Clamp params -->
    <template v-if="modelValue.type === 'clamp'">
      <div class="form-group">
        <label class="form-label">Min</label>
        <input
          class="input"
          type="number"
          step="any"
          :value="modelValue.clampMin ?? 0"
          @input="update('clampMin', Number(($event.target as HTMLInputElement).value))"
        />
      </div>
      <div class="form-group">
        <label class="form-label">Max</label>
        <input
          class="input"
          type="number"
          step="any"
          :value="modelValue.clampMax ?? 1"
          @input="update('clampMax', Number(($event.target as HTMLInputElement).value))"
        />
      </div>
    </template>

    <!-- Threshold param -->
    <template v-if="modelValue.type === 'threshold'">
      <div class="form-group">
        <label class="form-label">Threshold</label>
        <input
          class="input"
          type="number"
          step="any"
          :value="modelValue.threshold ?? 0.5"
          @input="update('threshold', Number(($event.target as HTMLInputElement).value))"
        />
      </div>
    </template>

    <!-- Expression param -->
    <template v-if="modelValue.type === 'expression'">
      <div class="form-group">
        <label class="form-label">Expression</label>
        <input
          class="input"
          type="text"
          placeholder="value * 2 + 1"
          :value="modelValue.expression ?? 'value'"
          @input="update('expression', ($event.target as HTMLInputElement).value)"
        />
      </div>
    </template>

    <!-- Deadzone params -->
    <template v-if="modelValue.type === 'deadzone'">
      <div class="form-group">
        <label class="form-label">Min (below = 0)</label>
        <input
          class="input"
          type="number"
          step="any"
          :value="modelValue.deadzoneMin ?? 0.4"
          @input="update('deadzoneMin', Number(($event.target as HTMLInputElement).value))"
        />
      </div>
      <div class="form-group">
        <label class="form-label">Max (above = 0)</label>
        <input
          class="input"
          type="number"
          step="any"
          :value="modelValue.deadzoneMax ?? 0.6"
          @input="update('deadzoneMax', Number(($event.target as HTMLInputElement).value))"
        />
      </div>
    </template>

    <!-- Smooth params -->
    <template v-if="modelValue.type === 'smooth'">
      <div class="form-group">
        <label class="form-label">Factor (0-1)</label>
        <input
          class="input"
          type="number"
          step="0.05"
          min="0"
          max="1"
          :value="modelValue.smoothFactor ?? 0.3"
          @input="update('smoothFactor', Number(($event.target as HTMLInputElement).value))"
        />
      </div>
    </template>

    <!-- Quantize params -->
    <template v-if="modelValue.type === 'quantize'">
      <div class="form-group">
        <label class="form-label">Steps</label>
        <input
          class="input"
          type="number"
          min="2"
          :value="modelValue.quantizeSteps ?? 8"
          @input="update('quantizeSteps', Number(($event.target as HTMLInputElement).value))"
        />
      </div>
    </template>

    <!-- Curve params -->
    <template v-if="modelValue.type === 'curve'">
      <div class="form-group">
        <label class="form-label">Curve Type</label>
        <select
          class="select"
          :value="modelValue.curveType ?? 'linear'"
          @change="update('curveType', ($event.target as HTMLSelectElement).value)"
        >
          <option v-for="c in curveTypes" :key="c.value" :value="c.value">{{ c.label }}</option>
        </select>
      </div>
    </template>

    <!-- Modulo params -->
    <template v-if="modelValue.type === 'modulo'">
      <div class="form-group">
        <label class="form-label">Divisor</label>
        <input
          class="input"
          type="number"
          step="any"
          :value="modelValue.moduloDivisor ?? 1"
          @input="update('moduloDivisor', Number(($event.target as HTMLInputElement).value))"
        />
      </div>
    </template>

    <!-- Power params -->
    <template v-if="modelValue.type === 'power'">
      <div class="form-group">
        <label class="form-label">Exponent</label>
        <input
          class="input"
          type="number"
          step="any"
          :value="modelValue.powerExponent ?? 2"
          @input="update('powerExponent', Number(($event.target as HTMLInputElement).value))"
        />
      </div>
    </template>

    <!-- JavaScript param -->
    <template v-if="modelValue.type === 'javascript'">
      <div class="form-group">
        <label class="form-label">JavaScript Code</label>
        <textarea
          class="input code-editor"
          rows="6"
          placeholder="function transform(input) {&#10;  return input;&#10;}"
          :value="modelValue.javascriptCode ?? 'function transform(input) {\n  return input;\n}'"
          @input="update('javascriptCode', ($event.target as HTMLTextAreaElement).value)"
        />
      </div>
    </template>

    <!-- Transform preview -->
    <div class="transform-preview">
      <div class="form-group">
        <label class="form-label">Test Input</label>
        <input
          class="input"
          type="number"
          step="any"
          v-model.number="testValue"
        />
      </div>
      <div class="preview-result">
        <span class="form-label">Output:</span>
        <span class="preview-value">{{ preview.output }}</span>
        <span v-if="preview.error" class="preview-error">{{ preview.error }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.transform-preview {
  margin-top: var(--space-sm);
  padding: var(--space-sm);
  background: var(--stone-200);
  border: 1px solid var(--stone-300);
}

.preview-result {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  margin-top: var(--space-xs);
  font-family: var(--font-mono);
  font-size: 12px;
}

.preview-value {
  color: var(--color-accent);
  font-weight: 600;
}

.preview-error {
  color: var(--color-error);
  font-size: 11px;
}

.code-editor {
  font-family: var(--font-mono);
  font-size: 12px;
  tab-size: 2;
  resize: vertical;
}
</style>
