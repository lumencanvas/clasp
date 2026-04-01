import { ref, reactive } from 'vue'

const ICE = [
  { urls: 'stun:stun.l.google.com:19302' },
  { urls: 'stun:stun1.l.google.com:19302' },
]

export function useLiveStream(getClient, getNS, getMe) {
  const streams = ref(new Map())
  const isLive = ref(false)
  const viewerCount = ref(0)
  const showModal = ref(false)
  const modalMeta = reactive({ name: '', sub: '', isSelf: false })
  const streamStatus = ref('connecting')
  const chatMessages = ref([])
  let videoEl = null

  let myStream = null
  const viewerPCs = new Map()
  const pendVIce = new Map()
  let viewerPC = null
  let watchId = null
  const pendSIce = []
  let liveTimer = null

  // --- subscriptions (called once after CLASP connected) ---
  function subscribe(toast) {
    const c = getClient()
    if (!c) return []
    const ns = getNS()
    const me = getMe()
    const unsubs = []

    const u1 = c.on(`${ns}/live/**`, (v, addr) => {
      const uid = addr.slice(`${ns}/live/`.length)
      if (uid === me.id) return
      if (!v) {
        streams.value.delete(uid); streams.value = new Map(streams.value)
        if (viewerPC && watchId === uid) { viewerPC.close(); viewerPC = null; watchId = null; showModal.value = false }
      } else {
        try {
          const e = JSON.parse(v)
          if (!streams.value.has(e.userId)) { streams.value.set(e.userId, e); streams.value = new Map(streams.value); toast(e.name + ' went live') }
        } catch {}
      }
    })
    if (typeof u1 === 'function') unsubs.push(u1)

    const u2 = c.on(`${ns}/watch/${me.id}`, (v) => {
      if (!v || !isLive.value) return
      try { handleWatchReq(JSON.parse(v).viewerId) } catch {}
    })
    if (typeof u2 === 'function') unsubs.push(u2)

    const u3 = c.on(`${ns}/offer/${me.id}`, (v) => {
      if (!v) return
      try { const d = JSON.parse(v); handleOffer(d.streamerId, d.sdp) } catch (e) { console.error('[offer]', e) }
    })
    if (typeof u3 === 'function') unsubs.push(u3)

    const u4 = c.on(`${ns}/answer/${me.id}`, (v) => {
      if (!v) return
      try { const d = JSON.parse(v); handleAnswer(d.viewerId, d.sdp) } catch {}
    })
    if (typeof u4 === 'function') unsubs.push(u4)

    const u5 = c.on(`${ns}/ice/${me.id}`, (v) => {
      if (!v) return
      try { const d = JSON.parse(v); handleIce(d.fromId, d.candidate) } catch {}
    })
    if (typeof u5 === 'function') unsubs.push(u5)

    // Live chat: SET-based messages per stream
    const u6 = c.on(`${ns}/livechat/**`, (v, addr) => {
      const prefix = `${ns}/livechat/`
      const rest = addr.slice(prefix.length)
      const sep = rest.indexOf('/')
      if (sep < 1) return
      const streamerId = rest.slice(0, sep)
      if (!v) return // deletion
      try {
        const m = JSON.parse(v)
        // Only show if this chat belongs to the currently open stream
        const activeStreamer = isLive.value ? me.id : watchId
        if (streamerId === activeStreamer) {
          if (!chatMessages.value.find(x => x.id === m.id)) {
            chatMessages.value = [...chatMessages.value, m].slice(-80)
          }
        }
      } catch {}
    })
    if (typeof u6 === 'function') unsubs.push(u6)

    return unsubs
  }

  // --- WebRTC: Broadcaster ---
  async function handleWatchReq(viewerId) {
    if (!isLive.value || !myStream) return
    if (viewerPCs.has(viewerId)) { viewerPCs.get(viewerId).close(); viewerPCs.delete(viewerId) }
    const pc = new RTCPeerConnection({ iceServers: ICE })
    viewerPCs.set(viewerId, pc)
    myStream.getTracks().forEach(t => pc.addTrack(t, myStream))
    const c = getClient(); const ns = getNS(); const me = getMe()
    pc.onicecandidate = (e) => {
      if (!e.candidate || !c) return
      c.emit(`${ns}/ice/${viewerId}`, JSON.stringify({ fromId: me.id, candidate: e.candidate.toJSON() }))
    }
    pc.onconnectionstatechange = () => {
      if (['failed', 'closed', 'disconnected'].includes(pc.connectionState)) viewerPCs.delete(viewerId)
      let n = 0; viewerPCs.forEach(p => { if (p.connectionState === 'connected') n++ }); viewerCount.value = n
    }
    try {
      const offer = await pc.createOffer()
      await pc.setLocalDescription(offer)
      if (c) c.emit(`${ns}/offer/${viewerId}`, JSON.stringify({ streamerId: me.id, sdp: pc.localDescription.toJSON() }))
      const buf = pendVIce.get(viewerId) || []; pendVIce.delete(viewerId)
      for (const ice of buf) pc.addIceCandidate(new RTCIceCandidate(ice)).catch(() => {})
    } catch (e) { console.error('[offer]', e) }
  }

  async function handleAnswer(vid, sdp) {
    const pc = viewerPCs.get(vid)
    if (!pc) return
    try {
      await pc.setRemoteDescription(new RTCSessionDescription(sdp))
      const buf = pendVIce.get(vid) || []; pendVIce.delete(vid)
      for (const ice of buf) pc.addIceCandidate(new RTCIceCandidate(ice)).catch(() => {})
    } catch (e) { console.error('[answer]', e) }
  }

  // --- WebRTC: Viewer ---
  async function handleOffer(streamerId, sdp) {
    if (viewerPC) { viewerPC.close(); viewerPC = null }
    watchId = streamerId
    const pc = new RTCPeerConnection({ iceServers: ICE })
    viewerPC = pc
    pc.ontrack = (ev) => {
      if (videoEl) { videoEl.srcObject = ev.streams[0]; videoEl.muted = false; videoEl.play().catch(() => {}) }
      streamStatus.value = 'live'
    }
    const c = getClient(); const ns = getNS(); const me = getMe()
    pc.onicecandidate = (e) => {
      if (!e.candidate || !c) return
      c.emit(`${ns}/ice/${streamerId}`, JSON.stringify({ fromId: me.id, candidate: e.candidate.toJSON() }))
    }
    pc.onconnectionstatechange = () => {
      if (pc.connectionState === 'connected') streamStatus.value = 'live'
      else if (pc.connectionState === 'failed') streamStatus.value = 'lost'
    }
    try {
      await pc.setRemoteDescription(new RTCSessionDescription(sdp))
      const ans = await pc.createAnswer()
      await pc.setLocalDescription(ans)
      if (c) c.emit(`${ns}/answer/${streamerId}`, JSON.stringify({ viewerId: me.id, sdp: pc.localDescription.toJSON() }))
      for (const ice of pendSIce) pc.addIceCandidate(new RTCIceCandidate(ice)).catch(() => {})
      pendSIce.length = 0
    } catch (e) { console.error('[viewer]', e) }
  }

  function handleIce(fromId, candidate) {
    if (viewerPCs.has(fromId)) {
      const pc = viewerPCs.get(fromId)
      if (pc.remoteDescription) pc.addIceCandidate(new RTCIceCandidate(candidate)).catch(() => {})
      else { const buf = pendVIce.get(fromId) || []; buf.push(candidate); pendVIce.set(fromId, buf) }
    } else if (viewerPC && fromId === watchId) {
      if (viewerPC.remoteDescription) viewerPC.addIceCandidate(new RTCIceCandidate(candidate)).catch(() => {})
      else pendSIce.push(candidate)
    }
  }

  // --- Go live ---
  async function goLive(toast) {
    if (isLive.value) { openSelf(); return }
    try {
      const stream = await navigator.mediaDevices.getUserMedia({
        video: { facingMode: 'user', width: { ideal: 1280 }, height: { ideal: 720 } }, audio: true,
      })
      myStream = stream
      isLive.value = true
      const c = getClient(); const ns = getNS(); const me = getMe()
      if (c) c.set(`${ns}/live/${me.id}`, JSON.stringify({ userId: me.id, name: me.name, handle: me.handle }), { ttl: 35 })
      // Republish live entry every 25s (TTL is 35s)
      clearInterval(liveTimer)
      liveTimer = setInterval(republishLive, 25000)
      openSelf()
      return true // caller should add "started live stream" post
    } catch (e) {
      const msgs = { NotAllowedError: 'camera permission denied', NotFoundError: 'no camera found', NotReadableError: 'camera in use', OverconstrainedError: 'resolution unsupported' }
      toast(msgs[e.name] || 'camera error', 'err')
      return false
    }
  }

  function stopLive(toast) {
    if (!isLive.value) return
    clearInterval(liveTimer); liveTimer = null
    if (myStream) { myStream.getTracks().forEach(t => t.stop()); myStream = null }
    viewerPCs.forEach(p => p.close()); viewerPCs.clear(); pendVIce.clear()
    isLive.value = false
    const c = getClient(); const ns = getNS(); const me = getMe()
    if (c) {
      c.set(`${ns}/live/${me.id}`, null)
      // Clear live chat from relay
      chatMessages.value.forEach(m => { c.set(`${ns}/livechat/${me.id}/${m.id}`, null) })
    }
    chatMessages.value = []
    showModal.value = false
    toast('stream ended')
  }

  function addChatMsg(streamerId, msg) {
    if (!chatMessages.value.find(x => x.id === msg.id)) {
      chatMessages.value = [...chatMessages.value, msg].slice(-80)
    }
  }

  function openSelf() {
    if (!isLive.value || !myStream) return
    const me = getMe()
    modalMeta.name = me.name; modalMeta.sub = 'your broadcast'; modalMeta.isSelf = true
    streamStatus.value = 'broadcasting'
    chatMessages.value = []
    if (videoEl) { videoEl.srcObject = myStream; videoEl.muted = true; videoEl.play().catch(() => {}) }
    showModal.value = true
  }

  function openViewer(entry) {
    if (viewerPC) { viewerPC.close(); viewerPC = null }
    watchId = entry.userId
    modalMeta.name = entry.name; modalMeta.sub = 'live stream'; modalMeta.isSelf = false
    streamStatus.value = 'connecting'
    chatMessages.value = []
    if (videoEl) { videoEl.srcObject = null; videoEl.muted = false }
    showModal.value = true
    const c = getClient(); const ns = getNS(); const me = getMe()
    if (c) c.emit(`${ns}/watch/${entry.userId}`, JSON.stringify({ viewerId: me.id }))
  }

  function closeModal() {
    showModal.value = false
    // Only clean up video/viewer when NOT broadcasting (matches HTML behavior)
    if (!isLive.value) {
      if (viewerPC) { viewerPC.close(); viewerPC = null; watchId = null }
      if (videoEl) videoEl.srcObject = null
    }
  }

  function clearLive() {
    const c = getClient(); const ns = getNS(); const me = getMe()
    if (c && isLive.value) c.set(`${ns}/live/${me.id}`, null)
  }

  function republishLive() {
    const c = getClient(); const ns = getNS(); const me = getMe()
    if (c && isLive.value) c.set(`${ns}/live/${me.id}`, JSON.stringify({ userId: me.id, name: me.name, handle: me.handle }), { ttl: 35 })
  }

  function setVideoEl(el) {
    videoEl = el
  }

  function cleanup() {
    clearInterval(liveTimer); liveTimer = null
    if (isLive.value) {
      if (myStream) { myStream.getTracks().forEach(t => t.stop()); myStream = null }
      viewerPCs.forEach(p => p.close()); viewerPCs.clear()
      isLive.value = false
    }
    if (viewerPC) { viewerPC.close(); viewerPC = null }
  }

  return {
    streams, isLive, viewerCount, showModal, modalMeta, streamStatus, chatMessages,
    get watchId() { return watchId },
    subscribe, goLive, stopLive, openSelf, openViewer, closeModal, clearLive, republishLive, cleanup, setVideoEl, addChatMsg,
  }
}
