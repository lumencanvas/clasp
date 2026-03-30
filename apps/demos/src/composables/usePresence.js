import { ref, readonly, onUnmounted } from 'vue'

export function usePresence(client, namespace, userId, userMeta) {
  const peers = ref(new Map())
  const onlineCount = ref(1)

  let timer = null

  function send() {
    const c = client.value
    if (!c) return
    c.set(`${namespace}/pres/${userId}`, JSON.stringify({
      ...userMeta(),
      ts: Date.now(),
    }), { ttl: 35 })
  }

  function clear() {
    const c = client.value
    if (!c) return
    c.set(`${namespace}/pres/${userId}`, null)
  }

  function start() {
    const c = client.value
    if (!c) return

    c.on(`${namespace}/pres/**`, (v, addr) => {
      const uid = addr.slice(`${namespace}/pres/`.length)
      if (uid === userId) return
      if (!v) {
        peers.value.delete(uid)
      } else {
        try { peers.value.set(uid, JSON.parse(v)) } catch {}
      }
      onlineCount.value = peers.value.size + 1
    })

    send()
    timer = setInterval(send, 28000)

    const onVis = () => {
      if (document.hidden) clear()
      else send()
    }
    document.addEventListener('visibilitychange', onVis)

    onUnmounted(() => {
      clearInterval(timer)
      clear()
      document.removeEventListener('visibilitychange', onVis)
    })
  }

  return { peers: readonly(peers), onlineCount: readonly(onlineCount), start, send, clear }
}
