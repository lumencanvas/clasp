const MSG = {
  HELLO: 'HELLO',
  WELCOME: 'WELCOME',
  SUBSCRIBE: 'SUBSCRIBE',
  UNSUBSCRIBE: 'UNSUBSCRIBE',
  SET: 'SET',
  PUBLISH: 'PUBLISH',
  SNAPSHOT: 'SNAPSHOT',
  PING: 'PING',
  PONG: 'PONG',
  ACK: 'ACK',
  ERROR: 'ERROR',
};

function encodeClaspFrame(message) {
  try {
    const { encodeMessage } = require('@clasp-to/core');
    const encoded = encodeMessage(message);
    return Buffer.from(encoded);
  } catch (e) {
    const { encode } = require('@msgpack/msgpack');
    const payload = Buffer.from(encode(message));
    const frame = Buffer.alloc(4 + payload.length);
    frame[0] = 0x53;
    frame[1] = 0x00;
    frame.writeUInt16BE(payload.length, 2);
    payload.copy(frame, 4);
    return frame;
  }
}

function decodeClaspFrame(buffer) {
  try {
    const { decodeMessage } = require('@clasp-to/core');
    const uint8Array = new Uint8Array(buffer);
    const result = decodeMessage(uint8Array);
    return result.message;
  } catch (e) {
    const { decode } = require('@msgpack/msgpack');
    if (buffer[0] !== 0x53) {
      throw new Error(`Invalid magic byte: expected 0x53, got 0x${buffer[0].toString(16)}`);
    }
    const flags = buffer[1];
    const hasTimestamp = (flags & 0x20) !== 0;
    const payloadLength = buffer.readUInt16BE(2);
    const payloadOffset = hasTimestamp ? 12 : 4;
    const payload = buffer.slice(payloadOffset, payloadOffset + payloadLength);
    return decode(payload);
  }
}

module.exports = { MSG, encodeClaspFrame, decodeClaspFrame };
