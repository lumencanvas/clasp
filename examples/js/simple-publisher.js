/**
 * CLASP Simple Publisher Example
 *
 * Demonstrates how to publish values and emit events to a CLASP server.
 *
 * Usage:
 *   npm install @clasp-to/core
 *   node simple-publisher.js
 */

const { Clasp } = require('@clasp-to/core');

const SERVER_URL = process.env.CLASP_URL || 'ws://localhost:7330';

async function main() {
  console.log('CLASP Simple Publisher');
  console.log('======================\n');

  // Create and connect client
  const clasp = Clasp.builder(SERVER_URL)
    .withName('example-publisher')
    .build();

  try {
    await clasp.connect();
    console.log(`Connected to ${SERVER_URL}`);
    console.log(`Session: ${clasp.session}\n`);

    // Example 1: Set parameter values
    console.log('Setting parameter values...');
    clasp.set('/example/fader/1', 0.75);
    clasp.set('/example/fader/2', 0.5);
    clasp.set('/example/button/1', true);
    clasp.set('/example/text', 'Hello CLASP!');
    console.log('  /example/fader/1 = 0.75');
    console.log('  /example/fader/2 = 0.5');
    console.log('  /example/button/1 = true');
    console.log('  /example/text = "Hello CLASP!"\n');

    // Example 2: Emit events (fire-and-forget)
    console.log('Emitting events...');
    clasp.emit('/example/cue/fire', { name: 'intro', duration: 5000 });
    clasp.emit('/example/notification', { type: 'info', message: 'Publisher started' });
    console.log('  /example/cue/fire -> { name: "intro", duration: 5000 }');
    console.log('  /example/notification -> { type: "info", message: "Publisher started" }\n');

    // Example 3: Stream high-rate data
    console.log('Streaming values (press Ctrl+C to stop)...');
    let value = 0;
    const streamInterval = setInterval(() => {
      // Sine wave from 0 to 1
      value = (Math.sin(Date.now() / 1000) + 1) / 2;
      clasp.stream('/example/stream/sine', value);

      // Random noise
      clasp.stream('/example/stream/noise', Math.random());

      process.stdout.write(`\r  sine: ${value.toFixed(3)}`);
    }, 33); // ~30 fps

    // Example 4: Atomic bundle
    setTimeout(() => {
      console.log('\n\nSending atomic bundle...');
      clasp.bundle([
        { set: ['/example/bundle/a', 1.0] },
        { set: ['/example/bundle/b', 2.0] },
        { emit: ['/example/bundle/done', { timestamp: Date.now() }] },
      ]);
      console.log('  Bundle sent with 2 sets and 1 emit');
    }, 2000);

    // Example 5: Scheduled bundle (execute at specific time)
    setTimeout(() => {
      const futureTime = clasp.time() + 1000000; // 1 second from now
      console.log('\nSending scheduled bundle (executes in 1 second)...');
      clasp.bundle(
        [
          { set: ['/example/scheduled/value', 42] },
          { emit: ['/example/scheduled/trigger', {}] },
        ],
        { at: futureTime }
      );
      console.log('  Scheduled bundle queued');
    }, 4000);

    // Handle graceful shutdown
    process.on('SIGINT', () => {
      console.log('\n\nShutting down...');
      clearInterval(streamInterval);
      clasp.close();
      process.exit(0);
    });

  } catch (error) {
    console.error('Error:', error.message);
    process.exit(1);
  }
}

main();
