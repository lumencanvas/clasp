# Video Streaming Handoff - 2026-01-26

## Summary

Implementing P2P Video Tab for CLASP Vue Playground with two transport modes:
1. **P2P Mode**: WebRTC with CLASP signaling
2. **Relay Mode**: Chunked video over CLASP protocol using WebCodecs

## Current Status: DEBUGGING

Video streaming is partially working but has issues on both modes.

### Files Created/Modified

| File | Purpose |
|------|---------|
| `site/src/lib/videoChunker.js` | Frame chunking, ChunkAssembler, JitterBuffer |
| `site/src/composables/useVideoStream.js` | Relay mode - WebCodecs encoder/decoder |
| `site/src/composables/useVideoCall.js` | P2P mode - WebRTC signaling |
| `site/src/components/playground/VideoTab.vue` | Main UI component |
| `bindings/js/packages/clasp-core/src/codec.ts` | Fixed dynamic buffer allocation |

### Issues Fixed This Session

1. **Codec RangeError** - Fixed by adding `estimateMessageSize()` for dynamic buffer allocation instead of fixed 4096 bytes

2. **RTCSessionDescription serialization** - WebRTC objects don't serialize through CLASP. Fixed by converting to plain objects:
   ```javascript
   sdp: { type: desc.type, sdp: desc.sdp }
   ```

3. **RTCIceCandidate serialization** - Same issue, fixed:
   ```javascript
   candidate: { candidate: c.candidate, sdpMid: c.sdpMid, sdpMLineIndex: c.sdpMLineIndex }
   ```

4. **JitterBuffer playback restart bug** - When buffer emptied, `playing` stayed true so new frames couldn't restart playback. Fixed by setting `playing = false` when buffer empties.

5. **JitterBuffer threshold** - Reduced from 3 frames to 1 frame to start playback sooner.

### Outstanding Issues

#### Issue 1: Relay Mode - Only receiving first chunk
- Broadcaster sends frames continuously (logs confirm)
- Receiver subscribes and gets ONE chunk (seq: 0, key frame)
- Then no more chunks received
- Subscription appears active but callback stops firing

**Possible causes:**
- CLASP router dropping stream messages (QoS Fire = unreliable)
- Subscription pattern mismatch after first message
- WebSocket connection issue

#### Issue 2: Local video showing black
- Both broadcaster and viewer show black video cells
- Even local preview (direct camera feed) is black
- Logs show stream is set on video element

**Possible causes:**
- CSS issue (unlikely - checked styling)
- Video element autoplay policy
- Stream not actually having video data

### Debug Logging Added

Extensive logging added to trace the flow:

```
[VideoTab] Local stream changed: ...
[VideoTab] Setting localVideoRef.srcObject
[VideoStream] Subscribing to: /video/relay/.../stream/...
[VideoStream] Subscribe callback #N for ...
[VideoStream] Received chunk: { seq, chunkIndex, totalChunks, frameType, dataType, dataLength }
[ChunkAssembler] addChunk called: ...
[ChunkAssembler] Frame complete, emitting: ...
[VideoStream] Assembler emitting frame: ...
[VideoStream] Jitter emitting frame: ...
[VideoStream] Decoded frame: ...
[VideoTab] Render loop: { peerId, hasDisplayCanvas, hasSourceCanvas, sourceSize }
[VideoStream] Subscription health check for X - chunks received: Y (every 3s)
```

### Next Steps

1. **Test with new logging** - Run both tabs and check console for where flow breaks
2. **Verify keyframes** - Ensure encoder is actually outputting keyframes (first frame should be key)
3. **Check CLASP router** - May need to verify stream messages are being forwarded
4. **Test local video separately** - Ensure camera permission granted and stream is valid

### Architecture Reference

```
Sender Flow:
  Camera -> MediaStream -> VideoEncoder -> chunkFrame() -> claspStream()
                                              |
                                      [16KB chunks with seq, type, data]

Receiver Flow:
  subscribe() -> decodeChunkFromTransport() -> ChunkAssembler -> JitterBuffer
                                                                      |
                                                              VideoDecoder -> Canvas
                                                                                |
                                                              Render Loop -> Display Canvas
```

### Key Code Locations

- Encoder config: `useVideoStream.js:580-612` (H.264 Baseline, annexb format)
- Chunk sending: `useVideoStream.js:668-707` (handleEncodedFrame)
- Subscription: `useVideoStream.js:400-445` (subscribeToStream)
- Decoder setup: `useVideoStream.js:313-361`
- Canvas render: `VideoTab.vue:95-128`

### Testing Checklist

- [ ] Local video preview works on broadcaster
- [ ] Chunks are being received continuously (not just first one)
- [ ] ChunkAssembler emits complete frames
- [ ] JitterBuffer forwards frames to decoder
- [ ] VideoDecoder successfully decodes frames
- [ ] Canvas render loop copies decoded frames to display
- [ ] P2P mode WebRTC signaling works

### Environment

- Vite dev server at localhost:5173
- CLASP router at localhost:7330
- Codec fix is in node_modules and vite cache (verified)
