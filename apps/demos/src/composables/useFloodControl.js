import { ref } from 'vue'

const RATE_MAX = 8
const RATE_WINDOW = 10000
const DRAIN_MS = 1900

export function useFloodControl() {
  const queue = ref([])
  const active = ref(false)
  const timestamps = []
  let drainTimer = null

  function canPost() {
    const now = Date.now()
    while (timestamps.length && now - timestamps[0] > RATE_WINDOW) timestamps.shift()
    return timestamps.length < RATE_MAX
  }

  function submit(post, onSend) {
    if (canPost()) {
      timestamps.push(Date.now())
      onSend(post)
      if (!queue.value.length) active.value = false
    } else {
      queue.value.push(post)
      active.value = true
      startDrain(onSend)
    }
  }

  function startDrain(onSend) {
    if (drainTimer) return
    drainTimer = setInterval(() => {
      if (!queue.value.length) {
        clearInterval(drainTimer)
        drainTimer = null
        active.value = false
        return
      }
      const p = queue.value.shift()
      timestamps.push(Date.now())
      onSend(p)
    }, DRAIN_MS)
  }

  function stop() {
    if (drainTimer) { clearInterval(drainTimer); drainTimer = null }
  }

  return { queue, active, canPost, submit, stop, RATE_MAX }
}
