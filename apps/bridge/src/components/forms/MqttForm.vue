<script setup lang="ts">
import type { Connection } from '../../lib/types'
import CollapsibleSection from '../shared/CollapsibleSection.vue'

const props = defineProps<{ modelValue: Connection }>()
const emit = defineEmits<{ 'update:modelValue': [value: Connection] }>()

function update(field: string, value: any) {
  emit('update:modelValue', { ...props.modelValue, [field]: value })
}

function topicsToText(topics?: string[]): string {
  return (topics ?? []).join('\n')
}

function textToTopics(text: string): string[] {
  return text.split('\n').map(t => t.trim()).filter(Boolean)
}
</script>

<template>
  <div class="protocol-form">
    <div class="form-group">
      <label class="form-label">Host</label>
      <input
        class="input"
        type="text"
        placeholder="localhost"
        :value="modelValue.host ?? 'localhost'"
        @input="update('host', ($event.target as HTMLInputElement).value)"
      />
    </div>
    <div class="form-group">
      <label class="form-label">Port</label>
      <input
        class="input"
        type="number"
        placeholder="1883"
        :value="modelValue.port ?? 1883"
        @input="update('port', Number(($event.target as HTMLInputElement).value))"
      />
    </div>
    <div class="form-group">
      <label class="form-label">Topics (one per line)</label>
      <textarea
        class="input"
        rows="3"
        placeholder="sensor/temperature&#10;actuator/#"
        :value="topicsToText(modelValue.topics)"
        @input="update('topics', textToTopics(($event.target as HTMLTextAreaElement).value))"
      />
    </div>

    <CollapsibleSection title="ADVANCED">
      <div class="form-group">
        <label class="form-label">Namespace</label>
        <input
          class="input"
          type="text"
          placeholder="mqtt"
          :value="modelValue.namespace ?? ''"
          @input="update('namespace', ($event.target as HTMLInputElement).value)"
        />
      </div>
      <div class="form-group">
        <label class="form-label">Client ID</label>
        <input
          class="input"
          type="text"
          placeholder="bridge-mqtt-client"
          :value="modelValue.clientId ?? ''"
          @input="update('clientId', ($event.target as HTMLInputElement).value)"
        />
      </div>
      <div class="form-group">
        <label class="form-label">QoS</label>
        <select
          class="select"
          :value="modelValue.qos ?? 0"
          @change="update('qos', Number(($event.target as HTMLSelectElement).value))"
        >
          <option :value="0">0 - At most once</option>
          <option :value="1">1 - At least once</option>
          <option :value="2">2 - Exactly once</option>
        </select>
      </div>
      <div class="form-group">
        <label class="form-label">Keep Alive (seconds)</label>
        <input
          class="input"
          type="number"
          placeholder="60"
          :value="modelValue.keepAlive ?? 60"
          @input="update('keepAlive', Number(($event.target as HTMLInputElement).value))"
        />
      </div>
      <div class="form-group">
        <label class="form-label">
          <input
            type="checkbox"
            :checked="modelValue.authEnabled ?? false"
            @change="update('authEnabled', ($event.target as HTMLInputElement).checked)"
          />
          Enable Authentication
        </label>
      </div>
      <template v-if="modelValue.authEnabled">
        <div class="form-group">
          <label class="form-label">Username</label>
          <input
            class="input"
            type="text"
            placeholder="Username"
            :value="modelValue.username ?? ''"
            @input="update('username', ($event.target as HTMLInputElement).value)"
          />
        </div>
        <div class="form-group">
          <label class="form-label">Password</label>
          <input
            class="input"
            type="password"
            placeholder="Password"
            :value="modelValue.password ?? ''"
            @input="update('password', ($event.target as HTMLInputElement).value)"
          />
        </div>
      </template>
    </CollapsibleSection>
  </div>
</template>
