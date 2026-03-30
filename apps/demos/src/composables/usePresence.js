import { ref, readonly, onUnmounted } from 'vue'

export function usePresence(client, getNamespace, userId, userMeta) {
  const peers = ref(new Map())
  const onlineCount = ref(1)

  let timer = null
  let unsub = null
  let visHandler = null

  function ns() {
    return typeof getNamespace === 'function' ? getNamespace() : getNamespace
  }

  function send() {
    const c = client.value
    if (!c) return
    try {
      c.set(`${ns()}/pres/${userId}`, JSON.stringify({
        ...userMeta(),
        ts: Date.now(),
      }), { ttl: 35 })
    } catch {}
  }

  function clear() {
    const c = client.value
    if (!c) return
    try { c.set(`${ns()}/pres/${userId}`, null) } catch {}
  }

  function start() {
    const c = client.value
    if (!c) return

    const ret = c.on(`${ns()}/pres/**`, (v, addr) => {
      const uid = addr.slice(`${ns()}/pres/`.length)
      if (uid === userId) return
      if (!v) {
        peers.value.delete(uid)
      } else {
        try { peers.value.set(uid, JSON.parse(v)) } catch {}
      }
      onlineCount.value = peers.value.size + 1
    })
    if (typeof ret === 'function') unsub = ret

    send()
    timer = setInterval(send, 28000)

    visHandler = () => {
      if (document.hidden) clear()
      else send()
    }
    document.addEventListener('visibilitychange', visHandler)
  }

  function stop() {
    clearInterval(timer)
    timer = null
    if (typeof unsub === 'function') { unsub(); unsub = null }
    if (visHandler) {
      document.removeEventListener('visibilitychange', visHandler)
      visHandler = null
    }
    clear()
  }

  onUnmounted(stop)

  return { peers: readonly(peers), onlineCount: readonly(onlineCount), start, stop, send, clear }
}
