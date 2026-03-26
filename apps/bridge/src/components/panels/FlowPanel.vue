<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted, watch } from 'vue'
import { useRouters } from '../../composables/useRouters'
import { useConnections } from '../../composables/useConnections'
import { useBridges } from '../../composables/useBridges'
import { useMonitor } from '../../composables/useMonitor'
import EmptyState from '../shared/EmptyState.vue'

const { routers } = useRouters()
const { connections } = useConnections()
const { bridges } = useBridges()
const { signalRate } = useMonitor()

const canvasRef = ref<HTMLCanvasElement | null>(null)
const containerRef = ref<HTMLDivElement | null>(null)

interface FlowNode {
  id: string
  type: 'router' | 'connection' | 'bridge'
  name: string
  protocol?: string
  status: string
  x: number
  y: number
}

const nodes = computed<FlowNode[]>(() => {
  const result: FlowNode[] = []
  const centerX = 400
  const routerY = 200

  routers.value.forEach((r, i) => {
    result.push({
      id: `router-${r.id}`,
      type: 'router',
      name: r.name,
      protocol: 'clasp',
      status: r.status || 'stopped',
      x: centerX + (i - (routers.value.length - 1) / 2) * 180,
      y: routerY,
    })
  })

  connections.value.forEach((c, i) => {
    const angle = (i / Math.max(connections.value.length, 1)) * Math.PI * 2 - Math.PI / 2
    result.push({
      id: `conn-${c.id}`,
      type: 'connection',
      name: c.name,
      protocol: c.protocol || c.type,
      status: c.status || 'stopped',
      x: centerX + Math.cos(angle) * 280,
      y: routerY + Math.sin(angle) * 200 + 40,
    })
  })

  return result
})

const hasContent = computed(() => routers.value.length > 0 || connections.value.length > 0)

function drawConnections() {
  const canvas = canvasRef.value
  if (!canvas) return
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  const container = containerRef.value
  if (container) {
    canvas.width = container.clientWidth
    canvas.height = container.clientHeight
  }

  ctx.clearRect(0, 0, canvas.width, canvas.height)
  ctx.strokeStyle = getComputedStyle(canvas).getPropertyValue('--stone-400').trim() || '#a8a29e'
  ctx.lineWidth = 1.5
  ctx.setLineDash([4, 4])

  const routerNodes = nodes.value.filter(n => n.type === 'router')
  const connNodes = nodes.value.filter(n => n.type === 'connection')

  for (const conn of connNodes) {
    for (const router of routerNodes) {
      ctx.beginPath()
      ctx.moveTo(router.x + 70, router.y + 30)
      ctx.lineTo(conn.x + 70, conn.y + 20)
      ctx.stroke()
    }
  }
}

let resizeObserver: ResizeObserver | null = null

onMounted(() => {
  drawConnections()
  if (containerRef.value) {
    resizeObserver = new ResizeObserver(() => drawConnections())
    resizeObserver.observe(containerRef.value)
  }
})

onUnmounted(() => {
  resizeObserver?.disconnect()
})

watch(nodes, () => drawConnections(), { deep: true })

function statusClass(status: string): string {
  switch (status) {
    case 'connected':
    case 'running': return 'active'
    default: return ''
  }
}
</script>

<template>
  <div class="panel-content panel-flow-content" style="height: 100%;">
    <div v-if="hasContent" ref="containerRef" class="flow-diagram">
      <canvas ref="canvasRef" id="flow-canvas"></canvas>
      <div class="flow-nodes">
        <div
          v-for="node in nodes"
          :key="node.id"
          class="flow-node"
          :class="{ 'flow-node-hub': node.type === 'router' }"
          :style="{ left: node.x + 'px', top: node.y + 'px' }"
        >
          <div class="flow-node-status" :class="statusClass(node.status)"></div>
          <div class="flow-node-title">{{ node.name }}</div>
          <div class="flow-node-detail">{{ node.protocol?.toUpperCase() || '' }}</div>
        </div>
      </div>
    </div>
    <EmptyState v-else message="No routers or connections" hint="Add a router or connection from the sidebar to see the flow diagram">
      <template #icon>
        <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" opacity="0.4">
          <circle cx="12" cy="12" r="3"/><path d="M12 2v4m0 12v4M2 12h4m12 0h4"/>
        </svg>
      </template>
    </EmptyState>
  </div>
</template>
