import { ref, watch, onUnmounted } from 'vue'

/**
 * Lightweight audio level detection using Web Audio API AnalyserNode.
 * Returns reactive isSpeaking and audioLevel refs.
 */
export function useAudioLevel(stream) {
  const isSpeaking = ref(false)
  const audioLevel = ref(0)

  let audioCtx = null
  let analyser = null
  let source = null
  let rafId = null
  let lastFrameTime = 0
  let speakingTimeout = null

  const THRESHOLD = 15
  const DEBOUNCE_MS = 300
  const FRAME_INTERVAL = 1000 / 15 // ~15fps

  function start(mediaStream) {
    cleanup()
    if (!mediaStream) return

    const audioTrack = mediaStream.getAudioTracks()[0]
    if (!audioTrack) return

    try {
      audioCtx = new AudioContext()
      analyser = audioCtx.createAnalyser()
      analyser.fftSize = 256
      source = audioCtx.createMediaStreamSource(mediaStream)
      source.connect(analyser)

      const dataArray = new Uint8Array(analyser.frequencyBinCount)

      function poll(timestamp) {
        if (timestamp - lastFrameTime < FRAME_INTERVAL) {
          rafId = requestAnimationFrame(poll)
          return
        }
        lastFrameTime = timestamp

        analyser.getByteFrequencyData(dataArray)

        let sum = 0
        for (let i = 0; i < dataArray.length; i++) {
          sum += dataArray[i]
        }
        const avg = sum / dataArray.length
        audioLevel.value = avg

        if (avg > THRESHOLD) {
          if (!isSpeaking.value) isSpeaking.value = true
          if (speakingTimeout) {
            clearTimeout(speakingTimeout)
            speakingTimeout = null
          }
        } else if (isSpeaking.value && !speakingTimeout) {
          speakingTimeout = setTimeout(() => {
            isSpeaking.value = false
            speakingTimeout = null
          }, DEBOUNCE_MS)
        }

        rafId = requestAnimationFrame(poll)
      }

      rafId = requestAnimationFrame(poll)
    } catch {
      // AudioContext not supported or stream issue
    }
  }

  function cleanup() {
    if (rafId) {
      cancelAnimationFrame(rafId)
      rafId = null
    }
    if (speakingTimeout) {
      clearTimeout(speakingTimeout)
      speakingTimeout = null
    }
    if (source) {
      try { source.disconnect() } catch {}
      source = null
    }
    if (audioCtx && audioCtx.state !== 'closed') {
      try { audioCtx.close() } catch {}
      audioCtx = null
    }
    analyser = null
    isSpeaking.value = false
    audioLevel.value = 0
  }

  // Watch for stream changes
  if (stream && typeof stream === 'object' && 'value' in stream) {
    watch(stream, (newStream) => {
      if (newStream) start(newStream)
      else cleanup()
    }, { immediate: true })
  }

  // Only register onUnmounted if inside a component setup context
  try { onUnmounted(cleanup) } catch {}

  return { isSpeaking, audioLevel, start, cleanup }
}
