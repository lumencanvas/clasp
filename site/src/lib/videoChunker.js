/**
 * Video Chunker Utility
 * Handles splitting encoded video frames into chunks for CLASP transport
 * and reassembling them on the receiving end.
 */

/**
 * Create a sequence generator for a specific stream
 * Avoids global mutable state issues with multiple instances
 */
export function createSequenceGenerator() {
  let seq = 0
  return () => seq++
}

// Default sequence generator (for backward compatibility)
const defaultSeqGen = createSequenceGenerator()

/**
 * Split an encoded video frame into chunks suitable for CLASP transport
 * @param {Uint8Array} frameData - The encoded frame data
 * @param {string} frameType - 'key' or 'delta'
 * @param {number} timestamp - Frame timestamp in microseconds
 * @param {number} maxChunkSize - Maximum chunk size in bytes (default 30KB)
 * @param {Function} seqGen - Optional sequence generator function
 * @returns {Array} Array of chunk objects
 */
export function chunkFrame(frameData, frameType, timestamp, maxChunkSize = 30000, seqGen = defaultSeqGen) {
  if (!(frameData instanceof Uint8Array)) {
    throw new TypeError('frameData must be a Uint8Array')
  }

  const chunks = []
  const totalChunks = Math.ceil(frameData.byteLength / maxChunkSize)
  const frameSeq = seqGen()

  for (let i = 0; i < totalChunks; i++) {
    const start = i * maxChunkSize
    const end = Math.min(start + maxChunkSize, frameData.byteLength)
    const chunkData = frameData.slice(start, end)

    chunks.push({
      seq: frameSeq,
      chunkIndex: i,
      totalChunks,
      frameType,
      timestamp,
      data: chunkData,
    })
  }

  return chunks
}

/**
 * Prepare chunk for transport - keeps Uint8Array as-is for binary transport
 * CLASP codec natively supports Uint8Array (VAL.BYTES), no base64 needed
 * @param {Object} chunk - Chunk object with Uint8Array data
 * @returns {Object} Chunk ready for transport
 */
export function encodeChunkForTransport(chunk) {
  // Return as-is - CLASP codec handles Uint8Array natively
  return chunk
}

/**
 * Decode chunk from transport
 * @param {Object} chunk - Chunk object from transport
 * @returns {Object} Chunk with Uint8Array data
 */
export function decodeChunkFromTransport(chunk) {
  if (!chunk) {
    throw new TypeError('chunk is null or undefined')
  }

  // If data is already Uint8Array, return as-is
  if (chunk.data instanceof Uint8Array) {
    return chunk
  }

  // If data is ArrayBuffer, convert to Uint8Array
  if (chunk.data instanceof ArrayBuffer) {
    return {
      ...chunk,
      data: new Uint8Array(chunk.data),
    }
  }

  // Handle typed array views (e.g., from some decoders)
  if (chunk.data && chunk.data.buffer instanceof ArrayBuffer) {
    return {
      ...chunk,
      data: new Uint8Array(chunk.data.buffer, chunk.data.byteOffset, chunk.data.byteLength),
    }
  }

  // If data is base64 string (legacy), decode it
  if (typeof chunk.data === 'string') {
    try {
      return {
        ...chunk,
        data: base64ToArrayBuffer(chunk.data),
      }
    } catch (e) {
      throw new Error(`Failed to decode chunk: ${e.message}`)
    }
  }

  throw new TypeError(`chunk.data must be Uint8Array, ArrayBuffer, or base64 string. Got: ${chunk.data?.constructor?.name || typeof chunk.data}`)
}

/**
 * Convert base64 string to Uint8Array (for legacy support)
 */
function base64ToArrayBuffer(base64) {
  const binary = atob(base64)
  const bytes = new Uint8Array(binary.length)
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i)
  }
  return bytes
}

/**
 * ChunkAssembler - Buffers incoming chunks and emits complete frames
 */
export class ChunkAssembler {
  constructor(options = {}) {
    this.frameBuffers = new Map() // seq -> { chunks: Map, totalChunks, frameType, timestamp }
    this.onFrame = options.onFrame || (() => {})
    this.onError = options.onError || (() => {})
    this.maxBufferedFrames = options.maxBufferedFrames || 30
    this.lastEmittedSeq = -1
  }

  /**
   * Add a chunk to the assembler
   * @param {Object} chunk - Chunk object (already decoded from transport)
   */
  addChunk(chunk) {
    if (!chunk || typeof chunk.seq !== 'number') {
      this.onError(new Error('Invalid chunk: missing seq'))
      return
    }

    const { seq, chunkIndex, totalChunks, frameType, timestamp, data } = chunk

    if (!(data instanceof Uint8Array)) {
      this.onError(new Error('Invalid chunk: data must be Uint8Array'))
      return
    }

    // Create frame buffer if needed
    if (!this.frameBuffers.has(seq)) {
      // Clean up old buffers if we have too many
      if (this.frameBuffers.size >= this.maxBufferedFrames) {
        this.pruneOldBuffers()
      }

      this.frameBuffers.set(seq, {
        chunks: new Map(),
        totalChunks,
        frameType,
        timestamp,
        receivedAt: Date.now(),
        totalSize: 0,
      })
    }

    const frameBuffer = this.frameBuffers.get(seq)

    // Don't add duplicate chunks
    if (frameBuffer.chunks.has(chunkIndex)) {
      return
    }

    frameBuffer.chunks.set(chunkIndex, data)
    frameBuffer.totalSize += data.byteLength

    // Store codec description from first chunk of keyframe
    if (chunk.description && chunkIndex === 0) {
      frameBuffer.description = chunk.description
    }

    // Check if frame is complete
    if (frameBuffer.chunks.size === frameBuffer.totalChunks) {
      this.emitFrame(seq)
    }
  }

  /**
   * Emit a complete frame
   */
  emitFrame(seq) {
    const frameBuffer = this.frameBuffers.get(seq)
    if (!frameBuffer) return

    // Use pre-calculated total size
    const frameData = new Uint8Array(frameBuffer.totalSize)
    let offset = 0

    for (let i = 0; i < frameBuffer.totalChunks; i++) {
      const chunk = frameBuffer.chunks.get(i)
      if (!chunk) {
        this.onError(new Error(`Missing chunk ${i} for frame ${seq}`))
        this.frameBuffers.delete(seq)
        return
      }
      frameData.set(chunk, offset)
      offset += chunk.byteLength
    }

    // Emit the complete frame
    const frame = {
      seq,
      frameType: frameBuffer.frameType,
      timestamp: frameBuffer.timestamp,
      data: frameData,
    }
    if (frameBuffer.description) {
      frame.description = frameBuffer.description
    }
    this.onFrame(frame)

    this.lastEmittedSeq = seq
    this.frameBuffers.delete(seq)
  }

  /**
   * Remove old incomplete frame buffers
   */
  pruneOldBuffers() {
    const now = Date.now()
    const maxAge = 2000 // 2 seconds

    for (const [seq, buffer] of this.frameBuffers) {
      if (now - buffer.receivedAt > maxAge) {
        this.frameBuffers.delete(seq)
      }
    }

    // If still too many, remove oldest by sequence number
    if (this.frameBuffers.size >= this.maxBufferedFrames) {
      const seqs = Array.from(this.frameBuffers.keys()).sort((a, b) => a - b)
      const toRemove = seqs.slice(0, this.frameBuffers.size - this.maxBufferedFrames + 1)
      toRemove.forEach(seq => this.frameBuffers.delete(seq))
    }
  }

  /**
   * Clear all buffered data
   */
  clear() {
    this.frameBuffers.clear()
    this.lastEmittedSeq = -1
  }

  /**
   * Get stats about buffered frames
   */
  getStats() {
    return {
      bufferedFrames: this.frameBuffers.size,
      lastEmittedSeq: this.lastEmittedSeq,
    }
  }
}

/**
 * JitterBuffer - Smooths out frame delivery for playback
 * Uses index-based circular buffer for efficiency
 */
export class JitterBuffer {
  constructor(options = {}) {
    this.buffer = []
    this.head = 0 // Read position
    this.tail = 0 // Write position
    this.size = 0
    this.targetLatency = options.targetLatency || 100 // ms
    this.maxBufferSize = options.maxBufferSize || 60 // frames
    this.onFrame = options.onFrame || (() => {})
    this.playing = false
    this.playbackTimer = null
    this.lastFrameTime = 0
    this.seenSeqs = new Set() // Track seen sequence numbers
  }

  /**
   * Add a frame to the jitter buffer
   */
  addFrame(frame) {
    // Reject duplicates
    if (this.seenSeqs.has(frame.seq)) {
      return
    }
    this.seenSeqs.add(frame.seq)

    // Limit seen seqs memory
    if (this.seenSeqs.size > this.maxBufferSize * 2) {
      const seqsArray = Array.from(this.seenSeqs).sort((a, b) => a - b)
      const toDelete = seqsArray.slice(0, this.maxBufferSize)
      toDelete.forEach(s => this.seenSeqs.delete(s))
    }

    // Insert in sorted order by sequence number
    const frameWithMeta = {
      ...frame,
      receivedAt: Date.now(),
    }

    // Binary search for insert position
    let left = 0
    let right = this.buffer.length
    while (left < right) {
      const mid = (left + right) >>> 1
      if (this.buffer[mid].seq < frame.seq) {
        left = mid + 1
      } else {
        right = mid
      }
    }

    this.buffer.splice(left, 0, frameWithMeta)

    // Trim buffer if too large (remove oldest)
    while (this.buffer.length > this.maxBufferSize) {
      this.buffer.shift()
    }

    // Start playback if not already playing and we have at least 1 frame
    if (!this.playing && this.buffer.length >= 1) {
      this.startPlayback()
    }
  }

  /**
   * Start frame playback
   */
  startPlayback() {
    if (this.playing) return
    this.playing = true
    this.scheduleNextFrame()
  }

  /**
   * Stop frame playback
   */
  stopPlayback() {
    this.playing = false
    if (this.playbackTimer) {
      clearTimeout(this.playbackTimer)
      this.playbackTimer = null
    }
  }

  /**
   * Schedule the next frame for playback
   */
  scheduleNextFrame() {
    if (!this.playing) {
      return
    }

    if (this.buffer.length === 0) {
      // Buffer empty, stop playback so addFrame can restart it
      this.playing = false
      return
    }

    const frame = this.buffer.shift()

    // Calculate delay based on frame timestamp
    let delay = 0
    if (this.lastFrameTime > 0 && frame.timestamp > 0) {
      // Use timestamp difference (timestamps are in microseconds)
      const expectedDelay = (frame.timestamp - this.lastFrameTime) / 1000
      delay = Math.max(0, Math.min(expectedDelay, 100)) // Cap at 100ms
    } else {
      delay = 33 // Default to ~30fps
    }

    this.playbackTimer = setTimeout(() => {
      this.onFrame(frame)
      this.lastFrameTime = frame.timestamp
      this.scheduleNextFrame()
    }, delay)
  }

  /**
   * Clear the buffer
   */
  clear() {
    this.stopPlayback()
    this.buffer = []
    this.seenSeqs.clear()
    this.lastFrameTime = 0
  }

  /**
   * Get buffer stats
   */
  getStats() {
    return {
      bufferedFrames: this.buffer.length,
      playing: this.playing,
    }
  }
}
