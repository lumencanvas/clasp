import { ref } from 'vue'

const RTC_CFG = {
  iceServers: [
    { urls: 'stun:stun.l.google.com:19302' },
    { urls: 'stun:stun1.l.google.com:19302' },
  ],
}
const SPEAKING_THRESHOLD = 0.02

export function useAudioMesh(getClient, getPrefix, getMyId, getCurrentRoom) {
  const isMuted = ref(true)
  const myVolume = ref(0)
  const speakingState = ref({}) // { [uid]: { speaking, volume } }

  let localStream = null
  let audioContext = null
  let analyser = null
  let volumeRAF = null
  const peers = {}

  // --- Microphone ---
  async function startMic() {
    try {
      localStream = await navigator.mediaDevices.getUserMedia({
        audio: { echoCancellation: true, noiseSuppression: true, autoGainControl: true },
      })
      audioContext = new (window.AudioContext || window.webkitAudioContext)()
      const source = audioContext.createMediaStreamSource(localStream)
      analyser = audioContext.createAnalyser()
      analyser.fftSize = 256
      analyser.smoothingTimeConstant = 0.8
      source.connect(analyser)

      const data = new Uint8Array(analyser.frequencyBinCount)
      function tick() {
        analyser.getByteFrequencyData(data)
        const avg = data.reduce((a, b) => a + b, 0) / data.length / 255
        myVolume.value = avg
        const speaking = avg > SPEAKING_THRESHOLD
        const room = getCurrentRoom()
        const c = getClient()
        if (c && room) {
          c.stream(`${getPrefix()}/rooms/${room.id}/speaking/${getMyId()}`, { speaking, volume: avg })
        }
        speakingState.value = {
          ...speakingState.value,
          [getMyId()]: { speaking, volume: avg },
        }
        volumeRAF = requestAnimationFrame(tick)
      }
      volumeRAF = requestAnimationFrame(tick)

      // Add tracks to existing peers
      Object.values(peers).forEach(p => {
        if (!p.pc) return
        const senders = p.pc.getSenders()
        localStream.getTracks().forEach(track => {
          const existing = senders.find(s => s.track?.kind === track.kind)
          if (existing) existing.replaceTrack(track)
          else p.pc.addTrack(track, localStream)
        })
      })

      isMuted.value = false
      return true
    } catch (e) {
      console.warn('[mic]', e)
      return false
    }
  }

  function stopMic() {
    if (volumeRAF) { cancelAnimationFrame(volumeRAF); volumeRAF = null }
    if (localStream) { localStream.getTracks().forEach(t => t.stop()); localStream = null }
    const c = getClient(); const room = getCurrentRoom()
    if (c && room) {
      c.stream(`${getPrefix()}/rooms/${room.id}/speaking/${getMyId()}`, { speaking: false, volume: 0 })
    }
    speakingState.value = { ...speakingState.value, [getMyId()]: { speaking: false, volume: 0 } }
    myVolume.value = 0
    isMuted.value = true
    if (audioContext) { audioContext.close().catch(() => {}); audioContext = null }
    analyser = null
  }

  // --- Peer connections ---
  async function createPC(pid, init) {
    if (peers[pid]) return
    const pc = new RTCPeerConnection(RTC_CFG)
    const el = document.createElement('audio')
    el.autoplay = true; el.playsInline = true
    document.body.appendChild(el)
    peers[pid] = { pc, audioEl: el }

    if (localStream) localStream.getTracks().forEach(t => pc.addTrack(t, localStream))

    pc.ontrack = (ev) => {
      el.srcObject = ev.streams[0]
      analyzeRemote(pid, ev.streams[0])
    }

    pc.onicecandidate = (ev) => {
      if (!ev.candidate) return
      const c = getClient(); const room = getCurrentRoom()
      if (c && room) {
        c.set(`${getPrefix()}/rooms/${room.id}/signal/${pid}/${getMyId()}`, {
          type: 'ice', candidate: ev.candidate.toJSON(), ts: Date.now(),
        })
      }
    }

    if (init) {
      if (!localStream) pc.addTransceiver('audio', { direction: 'recvonly' })
      const offer = await pc.createOffer()
      await pc.setLocalDescription(offer)
      const c = getClient(); const room = getCurrentRoom()
      if (c && room) {
        c.set(`${getPrefix()}/rooms/${room.id}/signal/${pid}/${getMyId()}`, {
          type: 'offer', sdp: offer.sdp, ts: Date.now(),
        })
      }
    }
  }

  function analyzeRemote(pid, stream) {
    const ctx = new (window.AudioContext || window.webkitAudioContext)()
    const src = ctx.createMediaStreamSource(stream)
    const an = ctx.createAnalyser()
    an.fftSize = 256; an.smoothingTimeConstant = 0.7
    src.connect(an)
    const data = new Uint8Array(an.frequencyBinCount)

    function tick() {
      if (!peers[pid]) { ctx.close().catch(() => {}); return }
      an.getByteFrequencyData(data)
      const avg = data.reduce((a, b) => a + b, 0) / data.length / 255
      speakingState.value = {
        ...speakingState.value,
        [pid]: { speaking: avg > SPEAKING_THRESHOLD, volume: avg },
      }
      requestAnimationFrame(tick)
    }
    requestAnimationFrame(tick)
  }

  async function handleSignal(from, d) {
    const c = getClient(); const room = getCurrentRoom()
    if (d.type === 'offer') {
      if (!peers[from]) await createPC(from, false)
      const pc = peers[from].pc
      await pc.setRemoteDescription({ type: 'offer', sdp: d.sdp })
      if (localStream && !pc.getSenders().length) {
        localStream.getTracks().forEach(t => pc.addTrack(t, localStream))
      }
      const ans = await pc.createAnswer()
      await pc.setLocalDescription(ans)
      if (c && room) {
        c.set(`${getPrefix()}/rooms/${room.id}/signal/${from}/${getMyId()}`, {
          type: 'answer', sdp: ans.sdp, ts: Date.now(),
        })
      }
    } else if (d.type === 'answer') {
      if (peers[from]?.pc.signalingState === 'have-local-offer') {
        await peers[from].pc.setRemoteDescription({ type: 'answer', sdp: d.sdp })
      }
    } else if (d.type === 'ice' && peers[from] && d.candidate) {
      try { await peers[from].pc.addIceCandidate(d.candidate) } catch {}
    }
  }

  function destroyPeer(pid) {
    if (!peers[pid]) return
    peers[pid].pc.close()
    peers[pid].audioEl.remove()
    delete peers[pid]
    const s = { ...speakingState.value }
    delete s[pid]
    speakingState.value = s
  }

  function destroyAll() {
    Object.keys(peers).forEach(destroyPeer)
  }

  function cleanup() {
    stopMic()
    destroyAll()
  }

  return {
    isMuted, myVolume, speakingState, peers,
    startMic, stopMic, createPC, handleSignal, destroyPeer, destroyAll, cleanup,
  }
}
