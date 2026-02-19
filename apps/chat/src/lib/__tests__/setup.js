import { webcrypto } from 'node:crypto'

// Polyfill Web Crypto API for jsdom environment
if (!globalThis.crypto?.subtle) {
  globalThis.crypto = webcrypto
}
