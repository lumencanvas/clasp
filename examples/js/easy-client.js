/**
 * Easy Client SDK -- comprehensive example
 *
 * Shows how @clasp-to/sdk wraps @clasp-to/core with a simpler API.
 * Compare this to simple-publisher.js / simple-subscriber.js to see
 * how much boilerplate the SDK eliminates.
 *
 * Usage:
 *   npm install @clasp-to/sdk
 *   node easy-client.js
 */

import clasp from '@clasp-to/sdk'

const CLASP_URL = process.env.CLASP_URL || 'ws://localhost:7330'

// --- Connect ----------------------------------------------------------------

const c = await clasp(CLASP_URL, { name: 'SDK Example' })
console.log('Connected to', CLASP_URL)

// --- Pub/Sub ----------------------------------------------------------------

// Set persistent state
await c.set('/demo/brightness', 0.8)
await c.set('/demo/color', { r: 255, g: 100, b: 0 })

// Read it back
const brightness = await c.get('/demo/brightness')
console.log('Brightness:', brightness)

// Subscribe to changes with rate limiting
c.on('/demo/**', (val, addr) => {
  console.log('Change:', addr, val)
}, { maxRate: 10 })

// Fire-and-forget event
await c.emit('/demo/alert', { message: 'SDK example started' })

// High-rate stream
for (let i = 0; i < 5; i++) {
  c.stream('/demo/position', { x: Math.random(), y: Math.random() })
}

// Atomic bundle
c.bundle([
  { set: ['/demo/a', 1] },
  { set: ['/demo/b', 2] },
  { emit: ['/demo/batch-done'] },
])

// --- Rules ------------------------------------------------------------------

c.rule('demo-threshold', {
  when: '/demo/brightness',
  above: 0.9,
  then: { emit: ['/demo/alert', { msg: 'Too bright!' }] },
  cooldown: '10s',
})

c.rule('demo-heartbeat', {
  every: '30s',
  then: { emit: ['/demo/heartbeat', { ts: Date.now() }] },
})

// --- Bridges ----------------------------------------------------------------

const osc = c.bridge('osc', { port: 9000 })
console.log('OSC bridge command:', osc.command)

const mqtt = c.bridge('mqtt', {
  broker: 'mqtt://localhost:1883',
  topics: ['sensors/#'],
})
console.log('MQTT bridge command:', mqtt.command)

// --- Cleanup ----------------------------------------------------------------

// Let subscriptions fire for a moment, then disconnect
setTimeout(() => {
  c.close()
  console.log('Disconnected')
}, 2000)
