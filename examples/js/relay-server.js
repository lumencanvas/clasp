/**
 * Example: Programmatic Relay Server with @clasp-to/relay
 *
 * Demonstrates how to start, configure, and manage a CLASP relay server
 * from Node.js. No need to manually run the relay binary.
 *
 * Prerequisites:
 *   npm install @clasp-to/relay @clasp-to/core
 *   # Ensure clasp-relay binary is on PATH or set CLASP_RELAY_BIN
 *
 * Usage:
 *   node examples/js/relay-server.js
 */

const { createRelay } = require('@clasp-to/relay')
const { ClaspBuilder } = require('@clasp-to/core')

async function main() {
  console.log('Starting CLASP relay server...')

  // Start a relay with auth enabled
  const relay = await createRelay(r => r
    .port(7330)
    .authPort(7350)
    .name('Example Relay')
    .drainTimeout(5)
    .verbose()
  )

  console.log(`Relay running at ${relay.url} (PID: ${relay.pid})`)
  console.log(`Auth API at ${relay.authUrl}`)

  // Forward relay logs to console
  relay.on('log', line => {
    console.log(`  [relay] ${line}`)
  })

  // Connect a client to the relay
  const client = await new ClaspBuilder(relay.url)
    .name('Example Client')
    .connect()

  console.log('Client connected')

  // Publish some data
  client.set('/demo/greeting', 'Hello from programmatic relay!')
  client.set('/demo/timestamp', Date.now())

  // Subscribe and log
  client.on('/demo/**', (value, address) => {
    console.log(`Received: ${address} = ${JSON.stringify(value)}`)
  })

  // Wait a moment, then clean up
  await new Promise(resolve => setTimeout(resolve, 2000))

  console.log('\nShutting down...')
  client.close()
  await relay.stop()
  console.log('Done.')
}

main().catch(err => {
  console.error(err)
  process.exit(1)
})
