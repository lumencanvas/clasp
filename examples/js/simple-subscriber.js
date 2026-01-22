/**
 * CLASP Simple Subscriber Example
 *
 * Demonstrates how to subscribe to values and events from a CLASP server.
 *
 * Usage:
 *   npm install @clasp-to/core
 *   node simple-subscriber.js
 */

const { Clasp } = require('@clasp-to/core');

const SERVER_URL = process.env.CLASP_URL || 'ws://localhost:7330';

async function main() {
  console.log('CLASP Simple Subscriber');
  console.log('=======================\n');

  // Create and connect client
  const clasp = Clasp.builder(SERVER_URL)
    .withName('example-subscriber')
    .build();

  try {
    await clasp.connect();
    console.log(`Connected to ${SERVER_URL}`);
    console.log(`Session: ${clasp.session}\n`);

    // Example 1: Subscribe to a specific address
    console.log('Subscribing to addresses...\n');

    clasp.on('/example/fader/1', (value, address) => {
      console.log(`[FADER] ${address} = ${value}`);
    });

    // Example 2: Subscribe with wildcard (single segment)
    // Matches: /example/button/1, /example/button/2, etc.
    clasp.on('/example/button/*', (value, address) => {
      console.log(`[BUTTON] ${address} = ${value}`);
    });

    // Example 3: Subscribe with double wildcard (multiple segments)
    // Matches: /example/nested/a/b/c, /example/nested/x, etc.
    clasp.on('/example/nested/**', (value, address) => {
      console.log(`[NESTED] ${address} = ${JSON.stringify(value)}`);
    });

    // Example 4: Subscribe to events
    clasp.on('/example/cue/*', (value, address) => {
      console.log(`[CUE] ${address} triggered:`, value);
    });

    // Example 5: Subscribe with rate limiting
    // Only receive updates at most 10 times per second
    clasp.on('/example/stream/*', (value, address) => {
      console.log(`[STREAM] ${address} = ${typeof value === 'number' ? value.toFixed(3) : value}`);
    }, { maxRate: 10 });

    // Example 6: Subscribe with epsilon (change threshold)
    // Only receive updates when value changes by more than 0.05
    clasp.on('/example/filtered/*', (value, address) => {
      console.log(`[FILTERED] ${address} = ${value}`);
    }, { epsilon: 0.05 });

    // Example 7: Get current value (async)
    console.log('Getting current values...');
    try {
      const fader1 = await clasp.get('/example/fader/1');
      console.log(`  /example/fader/1 = ${fader1}`);
    } catch (e) {
      console.log('  /example/fader/1 = (not set)');
    }

    // Example 8: Check cached value (sync, may be undefined)
    const cached = clasp.cached('/example/fader/2');
    console.log(`  /example/fader/2 (cached) = ${cached ?? '(not cached)'}`);

    // Example 9: Unsubscribe after some time
    const tempUnsub = clasp.on('/example/temporary', (value, address) => {
      console.log(`[TEMP] ${address} = ${value}`);
    });

    setTimeout(() => {
      console.log('\nUnsubscribing from /example/temporary');
      tempUnsub();
    }, 10000);

    // Event handlers
    clasp.onDisconnect((reason) => {
      console.log(`\nDisconnected: ${reason || 'unknown reason'}`);
    });

    clasp.onError((error) => {
      console.error(`\nError: ${error.message}`);
    });

    console.log('\nListening for messages (press Ctrl+C to stop)...\n');

    // Handle graceful shutdown
    process.on('SIGINT', () => {
      console.log('\nShutting down...');
      clasp.close();
      process.exit(0);
    });

  } catch (error) {
    console.error('Connection error:', error.message);
    process.exit(1);
  }
}

main();
