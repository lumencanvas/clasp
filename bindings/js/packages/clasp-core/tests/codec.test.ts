import { describe, it, expect } from 'vitest';
import { encodeFrame, decodeFrame, MAGIC_BYTE, QoS, encode, decode } from '../src/codec';
import type { SetMessage } from '../src/types';

describe('Frame Codec', () => {
  describe('encodeFrame', () => {
    it('should encode a basic frame', () => {
      const payload = new Uint8Array([1, 2, 3, 4]);
      const frame = encodeFrame(payload);

      expect(frame[0]).toBe(MAGIC_BYTE);
      expect(frame.length).toBeGreaterThan(4);
    });

    it('should encode frame with QoS', () => {
      const payload = new Uint8Array([1, 2, 3]);

      const frameFire = encodeFrame(payload, { qos: QoS.Fire });
      const frameConfirm = encodeFrame(payload, { qos: QoS.Confirm });
      const frameCommit = encodeFrame(payload, { qos: QoS.Commit });

      // QoS is in bits 6-7 of flags byte
      expect(frameFire[1] & 0xC0).toBe(0x00);
      expect(frameConfirm[1] & 0xC0).toBe(0x40);
      expect(frameCommit[1] & 0xC0).toBe(0x80);
    });

    it('should encode frame with timestamp', () => {
      const payload = new Uint8Array([1, 2, 3]);
      const timestamp = BigInt(1234567890);
      const frame = encodeFrame(payload, { timestamp });

      // Timestamp flag should be set
      expect(frame[1] & 0x20).toBe(0x20);
      // Frame should be longer due to timestamp
      expect(frame.length).toBeGreaterThan(4 + 3);
    });
  });

  describe('decodeFrame', () => {
    it('should decode a basic frame', () => {
      const payload = new Uint8Array([10, 20, 30, 40, 50]);
      const encoded = encodeFrame(payload);
      const decoded = decodeFrame(encoded);

      expect(decoded.payload).toEqual(payload);
      expect(decoded.qos).toBe(QoS.Fire);
    });

    it('should decode frame with QoS', () => {
      const payload = new Uint8Array([1, 2, 3]);

      for (const qos of [QoS.Fire, QoS.Confirm, QoS.Commit]) {
        const encoded = encodeFrame(payload, { qos });
        const decoded = decodeFrame(encoded);
        expect(decoded.qos).toBe(qos);
      }
    });

    it('should decode frame with timestamp', () => {
      const payload = new Uint8Array([1, 2, 3]);
      const timestamp = BigInt(9876543210);
      const encoded = encodeFrame(payload, { timestamp });
      const decoded = decodeFrame(encoded);

      expect(decoded.timestamp).toBe(timestamp);
      expect(decoded.payload).toEqual(payload);
    });

    it('should throw on invalid magic byte', () => {
      const invalid = new Uint8Array([0x00, 0x00, 0x00, 0x04, 1, 2, 3, 4]);
      expect(() => decodeFrame(invalid)).toThrow();
    });

    it('should throw on truncated frame', () => {
      const truncated = new Uint8Array([MAGIC_BYTE, 0x00]);
      expect(() => decodeFrame(truncated)).toThrow();
    });
  });

  describe('roundtrip', () => {
    it('should roundtrip various payload sizes', () => {
      const sizes = [0, 1, 10, 100, 1000, 10000];

      for (const size of sizes) {
        const payload = new Uint8Array(size);
        for (let i = 0; i < size; i++) {
          payload[i] = i % 256;
        }

        const encoded = encodeFrame(payload);
        const decoded = decodeFrame(encoded);

        expect(decoded.payload).toEqual(payload);
      }
    });

    it('should roundtrip with all options', () => {
      const payload = new Uint8Array([1, 2, 3, 4, 5]);
      const options = {
        qos: QoS.Commit,
        timestamp: BigInt(1234567890123456),
        sequence: 42,
      };

      const encoded = encodeFrame(payload, options);
      const decoded = decodeFrame(encoded);

      expect(decoded.payload).toEqual(payload);
      expect(decoded.qos).toBe(QoS.Commit);
      expect(decoded.timestamp).toBe(options.timestamp);
    });
  });
});

describe('SET with TTL', () => {
  it('should roundtrip SET with sliding TTL', () => {
    const msg: SetMessage = {
      type: 'SET',
      address: '/test/ttl',
      value: 1.0,
      ttl: 60,
    };

    const encoded = encode(msg);
    const decoded = decode(encoded) as SetMessage;

    expect(decoded.type).toBe('SET');
    expect(decoded.address).toBe('/test/ttl');
    expect(decoded.ttl).toBe(60);
    expect(decoded.absolute).toBeUndefined();
  });

  it('should roundtrip SET with absolute TTL', () => {
    const msg: SetMessage = {
      type: 'SET',
      address: '/test/ttl',
      value: 42,
      ttl: 300,
      absolute: true,
    };

    const encoded = encode(msg);
    const decoded = decode(encoded) as SetMessage;

    expect(decoded.type).toBe('SET');
    expect(decoded.ttl).toBe(300);
    expect(decoded.absolute).toBe(true);
  });

  it('should roundtrip SET with never-expire TTL', () => {
    const msg: SetMessage = {
      type: 'SET',
      address: '/test/ttl',
      value: true,
      ttl: 0,
    };

    const encoded = encode(msg);
    const decoded = decode(encoded) as SetMessage;

    expect(decoded.type).toBe('SET');
    expect(decoded.ttl).toBe(0);
  });

  it('should encode ttl=0 with absolute as never-expire (not Absolute(0))', () => {
    const msg: SetMessage = {
      type: 'SET',
      address: '/test/ttl-never-abs',
      value: 'persist',
      ttl: 0,
      absolute: true,
    };

    const encoded = encode(msg);
    const decoded = decode(encoded) as SetMessage;

    expect(decoded.type).toBe('SET');
    // ttl=0 must encode as raw 0 (Ttl::Never), NOT 0x80000000 (Ttl::Absolute(0))
    expect(decoded.ttl).toBe(0);
    // absolute flag is irrelevant for never-expire
    expect(decoded.absolute).toBeUndefined();
  });

  it('should roundtrip SET without TTL (backward compat)', () => {
    const msg: SetMessage = {
      type: 'SET',
      address: '/test/no-ttl',
      value: 'hello',
    };

    const encoded = encode(msg);
    const decoded = decode(encoded) as SetMessage;

    expect(decoded.type).toBe('SET');
    expect(decoded.address).toBe('/test/no-ttl');
    expect(decoded.ttl).toBeUndefined();
  });

  it('should roundtrip SET with TTL and revision', () => {
    const msg: SetMessage = {
      type: 'SET',
      address: '/test/rev-ttl',
      value: 3.14,
      revision: 42,
      ttl: 3600,
    };

    const encoded = encode(msg);
    const decoded = decode(encoded) as SetMessage;

    expect(decoded.revision).toBe(42);
    expect(decoded.ttl).toBe(3600);
  });

  it('should roundtrip SET with large TTL values', () => {
    // 24 hours
    const msg24h: SetMessage = { type: 'SET', address: '/ttl/24h', value: 1, ttl: 86400, absolute: true };
    const dec24h = decode(encode(msg24h)) as SetMessage;
    expect(dec24h.ttl).toBe(86400);
    expect(dec24h.absolute).toBe(true);

    // 7 days
    const msg7d: SetMessage = { type: 'SET', address: '/ttl/7d', value: 1, ttl: 604800 };
    const dec7d = decode(encode(msg7d)) as SetMessage;
    expect(dec7d.ttl).toBe(604800);
    expect(dec7d.absolute).toBeUndefined();
  });

  it('should preserve TTL=1 (minimum non-zero) correctly', () => {
    const sliding: SetMessage = { type: 'SET', address: '/ttl/1s', value: 1, ttl: 1 };
    const decSliding = decode(encode(sliding)) as SetMessage;
    expect(decSliding.ttl).toBe(1);
    expect(decSliding.absolute).toBeUndefined();

    const absolute: SetMessage = { type: 'SET', address: '/ttl/1s-abs', value: 1, ttl: 1, absolute: true };
    const decAbsolute = decode(encode(absolute)) as SetMessage;
    expect(decAbsolute.ttl).toBe(1);
    expect(decAbsolute.absolute).toBe(true);
  });

  it('should encode ttl=0 identically regardless of absolute flag', () => {
    // Both should produce identical binary: ttl=0 always means Never
    const withAbs: SetMessage = { type: 'SET', address: '/t', value: 0, ttl: 0, absolute: true };
    const withoutAbs: SetMessage = { type: 'SET', address: '/t', value: 0, ttl: 0 };

    const encWithAbs = encode(withAbs);
    const encWithoutAbs = encode(withoutAbs);

    // Binary output must be identical -- absolute flag is ignored for ttl=0
    expect(encWithAbs).toEqual(encWithoutAbs);

    // Both decode to ttl=0, no absolute
    const decWithAbs = decode(encWithAbs) as SetMessage;
    const decWithoutAbs = decode(encWithoutAbs) as SetMessage;
    expect(decWithAbs.ttl).toBe(0);
    expect(decWithoutAbs.ttl).toBe(0);
    expect(decWithAbs.absolute).toBeUndefined();
    expect(decWithoutAbs.absolute).toBeUndefined();
  });

  it('should roundtrip all TTL modes used by social demo', () => {
    // Social demo TTL options: 5m, 30m, 1h, 24h, never
    const cases = [
      { ttl: 300, absolute: true, label: '5m' },
      { ttl: 1800, absolute: true, label: '30m' },
      { ttl: 3600, absolute: true, label: '1h' },
      { ttl: 86400, absolute: true, label: '24h' },
      { ttl: 0, absolute: true, label: 'never' },
    ];

    for (const c of cases) {
      const msg: SetMessage = { type: 'SET', address: `/post/${c.label}`, value: 'test', ttl: c.ttl, absolute: c.absolute };
      const decoded = decode(encode(msg)) as SetMessage;
      if (c.ttl === 0) {
        expect(decoded.ttl).toBe(0);
        expect(decoded.absolute).toBeUndefined(); // never-expire has no absolute concept
      } else {
        expect(decoded.ttl).toBe(c.ttl);
        expect(decoded.absolute).toBe(true);
      }
    }
  });

  it('should roundtrip presence/live TTL (sliding, 35s)', () => {
    // Presence and live stream entries use sliding TTL
    const msg: SetMessage = { type: 'SET', address: '/live/user123', value: '{"name":"test"}', ttl: 35 };
    const decoded = decode(encode(msg)) as SetMessage;
    expect(decoded.ttl).toBe(35);
    expect(decoded.absolute).toBeUndefined(); // sliding
  });
});
