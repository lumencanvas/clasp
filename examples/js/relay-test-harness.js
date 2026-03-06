/**
 * Example: Test Harness with Ephemeral Relay
 *
 * Shows how to use @clasp-to/relay for integration tests. Each test gets
 * a fresh relay on a random port that is automatically torn down.
 *
 * This pattern is useful for CI pipelines and local development.
 *
 * Prerequisites:
 *   npm install @clasp-to/relay @clasp-to/core
 *
 * Usage:
 *   node examples/js/relay-test-harness.js
 */

const { createRelay } = require('@clasp-to/relay')
const { ClaspBuilder } = require('@clasp-to/core')

let passed = 0
let failed = 0

async function test(name, fn) {
  const relay = await createRelay({
    port: 0,  // random port
    drainTimeout: 1,
  })

  try {
    await fn(relay)
    passed++
    console.log(`  PASS  ${name}`)
  } catch (err) {
    failed++
    console.log(`  FAIL  ${name}: ${err.message}`)
  } finally {
    await relay.stop(3000)
  }
}

async function main() {
  console.log('Running integration tests with ephemeral relays...\n')

  await test('client connects to relay', async (relay) => {
    const client = await new ClaspBuilder(relay.url)
      .name('Test Client')
      .connect()

    if (!client.connected) throw new Error('Not connected')
    client.close()
  })

  await test('set and get round-trip', async (relay) => {
    const client = await new ClaspBuilder(relay.url)
      .name('Test Client')
      .connect()

    client.set('/test/value', 42)
    // Wait for the value to propagate
    await new Promise(resolve => setTimeout(resolve, 200))
    const value = await client.get('/test/value')

    if (value !== 42) throw new Error(`Expected 42, got ${value}`)
    client.close()
  })

  await test('subscription receives updates', async (relay) => {
    const writer = await new ClaspBuilder(relay.url).name('Writer').connect()
    const reader = await new ClaspBuilder(relay.url).name('Reader').connect()

    const received = []
    reader.on('/events/**', (value, address) => {
      received.push({ address, value })
    })

    // Wait for subscription to establish
    await new Promise(resolve => setTimeout(resolve, 200))

    writer.set('/events/temperature', 22.5)
    writer.set('/events/humidity', 65)

    // Wait for delivery
    await new Promise(resolve => setTimeout(resolve, 500))

    if (received.length < 2) {
      throw new Error(`Expected 2 events, got ${received.length}`)
    }

    writer.close()
    reader.close()
  })

  console.log(`\nResults: ${passed} passed, ${failed} failed`)
  process.exit(failed > 0 ? 1 : 0)
}

main().catch(err => {
  console.error(err)
  process.exit(1)
})
