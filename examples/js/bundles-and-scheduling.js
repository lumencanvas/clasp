/**
 * CLASP Bundles & Scheduling Example
 *
 * Demonstrates atomic bundles and scheduled execution:
 * - Atomic bundles: Multiple messages executed together
 * - Scheduled bundles: Execute at a specific future time
 * - Cancellable scheduled bundles
 *
 * Usage:
 *   node bundles-and-scheduling.js
 */

import { Clasp } from '@clasp-to/core';

const CLASP_URL = process.env.CLASP_URL || 'ws://localhost:7330';

async function main() {
  console.log('=== CLASP Bundles & Scheduling Example ===\n');

  const client = new Clasp(CLASP_URL);
  await client.connect({ name: 'Bundle Demo' });
  console.log('Connected to CLASP server');

  // Subscribe to see all changes
  client.on('/**', (value, address) => {
    const time = new Date().toISOString().slice(11, 23);
    console.log(`[${time}] ${address} = ${JSON.stringify(value)}`);
  });

  // Wait for subscription to be active
  await new Promise(r => setTimeout(r, 100));

  // =====================
  // 1. Atomic Bundle
  // =====================
  console.log('\n--- 1. Atomic Bundle ---');
  console.log('Setting multiple values atomically...');

  // All these changes happen together - subscribers see them as one update
  await client.bundle([
    { type: 'set', address: '/scene/active', value: 'sunset' },
    { type: 'set', address: '/lights/1/brightness', value: 0.8 },
    { type: 'set', address: '/lights/1/color', value: { r: 255, g: 180, b: 100 } },
    { type: 'set', address: '/lights/2/brightness', value: 0.6 },
    { type: 'set', address: '/lights/2/color', value: { r: 255, g: 150, b: 80 } },
    { type: 'emit', address: '/scene/activated', payload: { name: 'sunset', time: Date.now() } }
  ]);
  console.log('Atomic bundle sent!');

  await new Promise(r => setTimeout(r, 500));

  // =====================
  // 2. Scheduled Bundle
  // =====================
  console.log('\n--- 2. Scheduled Bundle ---');

  // Get current server time (synchronized via NTP-style protocol)
  const serverTime = client.time();
  console.log(`Server time: ${serverTime}`);

  // Schedule a bundle to execute 2 seconds from now
  const executeAt = serverTime + 2_000_000; // 2 seconds in microseconds
  console.log(`Scheduling bundle for: ${new Date(executeAt / 1000).toISOString()}`);

  await client.bundle([
    { type: 'set', address: '/scheduled/counter', value: 1 },
    { type: 'emit', address: '/scheduled/triggered', payload: { scheduled: true } }
  ], { at: executeAt });

  console.log('Scheduled bundle sent! Waiting for execution...');

  // Wait for the scheduled time
  await new Promise(r => setTimeout(r, 2500));

  // =====================
  // 3. Chained Scheduled Bundles (Animation)
  // =====================
  console.log('\n--- 3. Chained Scheduled Bundles (Animation) ---');
  console.log('Creating a 5-step fade animation...');

  const animationStart = client.time() + 500_000; // Start in 0.5s
  const stepDuration = 200_000; // 200ms per step

  // Schedule 5 brightness steps
  for (let i = 0; i <= 5; i++) {
    const brightness = i / 5;
    const executeTime = animationStart + (i * stepDuration);

    await client.bundle([
      { type: 'set', address: '/animation/brightness', value: brightness },
      { type: 'set', address: '/animation/step', value: i }
    ], { at: executeTime });
  }

  console.log('Animation scheduled! Watching...');
  await new Promise(r => setTimeout(r, 2000));

  // =====================
  // 4. Cancellable Scheduled Bundle
  // =====================
  console.log('\n--- 4. Cancellable Scheduled Bundle ---');

  const cancelTime = client.time() + 5_000_000; // 5 seconds from now
  console.log('Scheduling a bundle for 5 seconds from now...');

  const bundleId = await client.bundle([
    { type: 'set', address: '/cancel/test', value: 'should not see this' }
  ], { at: cancelTime, id: 'cancel-demo' });

  console.log(`Bundle scheduled with ID: ${bundleId}`);
  console.log('Waiting 1 second then cancelling...');

  await new Promise(r => setTimeout(r, 1000));

  // Cancel the scheduled bundle
  await client.cancelBundle(bundleId);
  console.log('Bundle cancelled!');

  console.log('Waiting to verify it was cancelled...');
  await new Promise(r => setTimeout(r, 5000));
  console.log('If no message appeared, cancellation worked!');

  // =====================
  // 5. Mixed Bundle with Events and Params
  // =====================
  console.log('\n--- 5. Mixed Bundle with Events and Params ---');

  await client.bundle([
    // Params (stateful)
    { type: 'set', address: '/cue/current', value: 'intro' },
    { type: 'set', address: '/cue/progress', value: 0 },

    // Events (one-shot triggers)
    { type: 'emit', address: '/cue/started', payload: { name: 'intro' } },
    { type: 'emit', address: '/log/entry', payload: { msg: 'Cue started', level: 'info' } }
  ]);
  console.log('Mixed bundle sent!');

  await new Promise(r => setTimeout(r, 500));

  // =====================
  // 6. Conditional Bundle (with revision check)
  // =====================
  console.log('\n--- 6. Conditional Bundle (Optimistic Locking) ---');

  // Get current value and revision
  const current = await client.get('/counter');
  const revision = current?.revision || 0;
  console.log(`Current counter revision: ${revision}`);

  // Only apply if revision matches (optimistic locking)
  try {
    await client.bundle([
      { type: 'set', address: '/counter', value: (current?.value || 0) + 1 }
    ], { ifRevision: revision });
    console.log('Conditional bundle applied!');
  } catch (err) {
    if (err.code === 'REVISION_MISMATCH') {
      console.log('Revision mismatch - someone else modified the value');
    } else {
      throw err;
    }
  }

  await new Promise(r => setTimeout(r, 500));

  await client.disconnect();
  console.log('\n=== Bundle demos complete ===');
}

main().catch(console.error);
