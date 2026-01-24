/**
 * CLASP Late Joiner Synchronization Example
 *
 * Demonstrates how late-joining clients receive current state:
 * - Automatic snapshot on subscribe
 * - Selective sync based on subscription patterns
 * - Revision tracking for conflict detection
 *
 * This is a key differentiator from OSC, which has no state sync.
 *
 * Usage:
 *   node late-joiner.js
 */

import { Clasp } from '@clasp-to/core';

const CLASP_URL = process.env.CLASP_URL || 'ws://localhost:7330';

async function main() {
  console.log('=== CLASP Late Joiner Synchronization Example ===\n');

  // =====================
  // Setup: Initialize state with first client
  // =====================
  console.log('--- Setup: Initializing State ---');

  const initializer = new Clasp(CLASP_URL);
  await initializer.connect({ name: 'State Initializer' });

  // Set up initial state
  console.log('Setting up initial state...\n');

  await initializer.set('/lights/living-room/brightness', 0.8);
  await initializer.set('/lights/living-room/color', { r: 255, g: 240, b: 220 });
  await initializer.set('/lights/kitchen/brightness', 1.0);
  await initializer.set('/lights/bedroom/brightness', 0.3);

  await initializer.set('/audio/master/volume', 0.65);
  await initializer.set('/audio/master/mute', false);
  await initializer.set('/audio/channel/1/volume', 0.8);
  await initializer.set('/audio/channel/2/volume', 0.7);

  await initializer.set('/scene/active', 'evening');
  await initializer.set('/scene/last-change', Date.now());

  console.log('Initial state created with 10 params');
  console.log('Disconnecting initializer...\n');
  await initializer.disconnect();

  // Wait a moment to simulate real-world scenario
  await new Promise(r => setTimeout(r, 500));

  // =====================
  // 1. Late Joiner - Full Wildcard
  // =====================
  console.log('--- 1. Late Joiner with Full Wildcard ---');

  const lateJoiner1 = new Clasp(CLASP_URL);
  await lateJoiner1.connect({ name: 'Late Joiner 1' });

  console.log('New client connected. Subscribing to /**...\n');

  let snapshotCount = 0;

  // When subscribing, we'll receive a SNAPSHOT with all matching state
  lateJoiner1.on('/**', (value, address, meta) => {
    snapshotCount++;
    console.log(`  [SNAPSHOT] ${address} = ${JSON.stringify(value)} (rev: ${meta.revision})`);
  });

  // Wait for snapshot to complete
  await new Promise(r => setTimeout(r, 300));
  console.log(`\nReceived ${snapshotCount} params in snapshot`);

  await lateJoiner1.disconnect();

  // =====================
  // 2. Late Joiner - Selective Subscription
  // =====================
  console.log('\n--- 2. Late Joiner with Selective Subscription ---');

  const lateJoiner2 = new Clasp(CLASP_URL);
  await lateJoiner2.connect({ name: 'Late Joiner 2 (Lights Only)' });

  console.log('Subscribing to /lights/** only...\n');

  let lightsCount = 0;
  lateJoiner2.on('/lights/**', (value, address) => {
    lightsCount++;
    console.log(`  [SNAPSHOT] ${address} = ${JSON.stringify(value)}`);
  });

  await new Promise(r => setTimeout(r, 200));
  console.log(`\nReceived ${lightsCount} light params (audio/scene excluded)`);

  await lateJoiner2.disconnect();

  // =====================
  // 3. Late Joiner - Multiple Subscriptions
  // =====================
  console.log('\n--- 3. Late Joiner with Multiple Subscriptions ---');

  const lateJoiner3 = new Clasp(CLASP_URL);
  await lateJoiner3.connect({ name: 'Late Joiner 3 (Multi)' });

  console.log('Subscribing to /lights/*/brightness and /audio/master/*...\n');

  const received = new Set();

  lateJoiner3.on('/lights/*/brightness', (value, address) => {
    received.add(address);
    console.log(`  [LIGHTS] ${address} = ${value}`);
  });

  lateJoiner3.on('/audio/master/*', (value, address) => {
    received.add(address);
    console.log(`  [AUDIO] ${address} = ${JSON.stringify(value)}`);
  });

  await new Promise(r => setTimeout(r, 200));
  console.log(`\nReceived ${received.size} params matching patterns`);

  await lateJoiner3.disconnect();

  // =====================
  // 4. Snapshot with Revisions
  // =====================
  console.log('\n--- 4. Tracking Revisions ---');

  const revisionTracker = new Clasp(CLASP_URL);
  await revisionTracker.connect({ name: 'Revision Tracker' });

  console.log('Tracking revisions for conflict detection...\n');

  const revisions = new Map();

  revisionTracker.on('/lights/**', (value, address, meta) => {
    revisions.set(address, {
      value,
      revision: meta.revision,
      timestamp: meta.timestamp
    });
  });

  await new Promise(r => setTimeout(r, 200));

  console.log('Current state with revisions:');
  revisions.forEach((data, address) => {
    console.log(`  ${address}: rev=${data.revision}`);
  });

  // Demonstrate revision-based update
  console.log('\nAttempting revision-based update...');

  const livingRoom = revisions.get('/lights/living-room/brightness');
  if (livingRoom) {
    try {
      // This will only succeed if no one else modified it
      await revisionTracker.set('/lights/living-room/brightness', 0.5, {
        ifRevision: livingRoom.revision
      });
      console.log('Update succeeded - we had the latest revision');
    } catch (err) {
      if (err.code === 'REVISION_MISMATCH') {
        console.log('Update failed - someone else modified it first');
      } else {
        throw err;
      }
    }
  }

  await revisionTracker.disconnect();

  // =====================
  // 5. Instant Sync Demo
  // =====================
  console.log('\n--- 5. Instant Sync Demo (OSC vs CLASP) ---');

  // Simulate what happens with OSC (no state)
  console.log('\nWith OSC (stateless):');
  console.log('  - Client connects');
  console.log('  - Client waits for someone to send updates...');
  console.log('  - Minutes pass with no data');
  console.log('  - Finally someone moves a fader');
  console.log('  - Client sees ONE value');

  console.log('\nWith CLASP (stateful):');
  console.log('  - Client connects');
  console.log('  - Subscribes to patterns');
  console.log('  - IMMEDIATELY receives ALL current values');
  console.log('  - Ready to work in milliseconds');

  // Demonstrate this
  const demonstrator = new Clasp(CLASP_URL);
  await demonstrator.connect({ name: 'Instant Sync Demo' });

  const startTime = Date.now();
  let syncComplete = false;
  let receivedCount = 0;

  demonstrator.on('/**', (value, address) => {
    receivedCount++;
    if (!syncComplete && receivedCount >= 5) {
      syncComplete = true;
      const elapsed = Date.now() - startTime;
      console.log(`\n  CLASP: Received ${receivedCount} params in ${elapsed}ms after connect!`);
    }
  });

  await new Promise(r => setTimeout(r, 300));

  await demonstrator.disconnect();

  console.log('\n=== Late joiner demo complete ===');
  console.log('\nKey takeaway: Unlike OSC, CLASP clients get current state immediately.');
  console.log('This is essential for:');
  console.log('  - Show control (know current cue/scene)');
  console.log('  - Lighting (see current levels)');
  console.log('  - Audio mixing (see current fader positions)');
  console.log('  - Any application where state matters');
}

main().catch(console.error);
