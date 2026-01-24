/**
 * CLASP Locks Example
 *
 * Demonstrates exclusive control using locks:
 * - Acquire/release locks on addresses
 * - Lock timeouts and expiration
 * - Lock contention handling
 * - Hierarchical locks
 *
 * Usage:
 *   node locks.js
 */

import { Clasp } from '@clasp-to/core';

const CLASP_URL = process.env.CLASP_URL || 'ws://localhost:7330';

async function main() {
  console.log('=== CLASP Locks Example ===\n');

  // Create two clients to demonstrate lock contention
  const client1 = new Clasp(CLASP_URL);
  const client2 = new Clasp(CLASP_URL);

  await client1.connect({ name: 'Client 1 (Operator)' });
  await client2.connect({ name: 'Client 2 (Assistant)' });
  console.log('Both clients connected\n');

  // =====================
  // 1. Basic Lock/Unlock
  // =====================
  console.log('--- 1. Basic Lock/Unlock ---');

  // Client 1 acquires a lock
  const lock1 = await client1.lock('/lights/stage/spotlight');
  console.log(`Client 1 acquired lock: ${lock1.id}`);
  console.log(`  Address: ${lock1.address}`);
  console.log(`  Expires: ${new Date(lock1.expiresAt).toISOString()}`);

  // Client 1 can modify the value
  await client1.set('/lights/stage/spotlight', 0.8);
  console.log('Client 1 set spotlight to 0.8');

  // Client 2 tries to modify (should fail)
  try {
    await client2.set('/lights/stage/spotlight', 0.5);
    console.log('Client 2 set spotlight - UNEXPECTED!');
  } catch (err) {
    console.log(`Client 2 blocked: ${err.code}`);
  }

  // Client 1 releases the lock
  await client1.unlock(lock1.id);
  console.log('Client 1 released lock');

  // Now client 2 can modify
  await client2.set('/lights/stage/spotlight', 0.5);
  console.log('Client 2 set spotlight to 0.5 - OK');

  await new Promise(r => setTimeout(r, 300));

  // =====================
  // 2. Lock with Timeout
  // =====================
  console.log('\n--- 2. Lock with Timeout ---');

  // Acquire lock with 2-second timeout
  const shortLock = await client1.lock('/mixer/master', { timeout: 2000 });
  console.log('Client 1 acquired lock with 2s timeout');

  // Client 2 waits for lock (will succeed after timeout)
  console.log('Client 2 waiting for lock...');

  const waitStart = Date.now();
  const waitedLock = await client2.lock('/mixer/master', { wait: true, waitTimeout: 5000 });
  const waitTime = Date.now() - waitStart;

  console.log(`Client 2 acquired lock after ${waitTime}ms wait`);
  await client2.unlock(waitedLock.id);

  await new Promise(r => setTimeout(r, 300));

  // =====================
  // 3. Try Lock (Non-Blocking)
  // =====================
  console.log('\n--- 3. Try Lock (Non-Blocking) ---');

  // Client 1 holds a lock
  const holdLock = await client1.lock('/audio/effects/reverb');
  console.log('Client 1 holds lock on /audio/effects/reverb');

  // Client 2 tries to get lock without waiting
  const tryResult = await client2.tryLock('/audio/effects/reverb');

  if (tryResult) {
    console.log('Client 2 got lock - UNEXPECTED!');
    await client2.unlock(tryResult.id);
  } else {
    console.log('Client 2 tryLock returned null (lock busy) - OK');
  }

  // Try on a different address (should succeed)
  const freeLock = await client2.tryLock('/audio/effects/delay');
  if (freeLock) {
    console.log('Client 2 got lock on /audio/effects/delay - OK');
    await client2.unlock(freeLock.id);
  }

  await client1.unlock(holdLock.id);

  await new Promise(r => setTimeout(r, 300));

  // =====================
  // 4. Hierarchical Locks
  // =====================
  console.log('\n--- 4. Hierarchical Locks ---');

  // Lock a parent address - children are also locked
  const parentLock = await client1.lock('/dmx/universe/1', { recursive: true });
  console.log('Client 1 locked /dmx/universe/1 (recursive)');

  // Attempts to lock children should fail
  try {
    await client2.lock('/dmx/universe/1/channel/1');
    console.log('Client 2 locked child - UNEXPECTED!');
  } catch (err) {
    console.log(`Client 2 blocked from child: ${err.code} - OK`);
  }

  // Writing to children is also blocked
  try {
    await client2.set('/dmx/universe/1/channel/50', 128);
    console.log('Client 2 wrote to child - UNEXPECTED!');
  } catch (err) {
    console.log(`Client 2 blocked from writing child: ${err.code} - OK`);
  }

  await client1.unlock(parentLock.id);
  console.log('Parent lock released');

  await new Promise(r => setTimeout(r, 300));

  // =====================
  // 5. Lock Info
  // =====================
  console.log('\n--- 5. Lock Information ---');

  const infoLock = await client1.lock('/video/output/1', { timeout: 30000 });

  // Query lock info
  const info = await client2.lockInfo('/video/output/1');

  if (info) {
    console.log('Lock info:');
    console.log(`  Held by: ${info.holder}`);
    console.log(`  Session: ${info.session}`);
    console.log(`  Acquired: ${new Date(info.acquiredAt).toISOString()}`);
    console.log(`  Expires: ${new Date(info.expiresAt).toISOString()}`);
    console.log(`  Remaining: ${((info.expiresAt - Date.now()) / 1000).toFixed(1)}s`);
  }

  await client1.unlock(infoLock.id);

  await new Promise(r => setTimeout(r, 300));

  // =====================
  // 6. Lock with Auto-Renewal
  // =====================
  console.log('\n--- 6. Lock with Auto-Renewal ---');

  // Acquire lock with short timeout but auto-renewal
  const renewLock = await client1.lock('/critical/resource', {
    timeout: 1000, // 1 second
    autoRenew: true // Automatically extend while held
  });
  console.log('Client 1 acquired lock with 1s timeout + auto-renew');

  // Do some work over 3 seconds (lock auto-renews)
  for (let i = 0; i < 6; i++) {
    await new Promise(r => setTimeout(r, 500));
    console.log(`  Working... (${(i + 1) * 0.5}s elapsed, lock still held)`);
  }

  // Lock should still be valid
  const stillValid = await client1.lockInfo('/critical/resource');
  console.log(`Lock still valid: ${stillValid?.holder === 'Client 1 (Operator)'}`);

  await client1.unlock(renewLock.id);
  console.log('Lock released');

  await new Promise(r => setTimeout(r, 300));

  // =====================
  // 7. Force Unlock (Admin)
  // =====================
  console.log('\n--- 7. Force Unlock (Admin) ---');

  // Client 1 holds a lock
  const stuckLock = await client1.lock('/emergency/control');
  console.log('Client 1 holds lock on /emergency/control');

  // In emergencies, admin can force-unlock
  // This requires admin:/** scope in production
  try {
    await client2.forceUnlock('/emergency/control');
    console.log('Client 2 force-unlocked (admin action)');

    // Now client 2 can acquire
    const freedLock = await client2.lock('/emergency/control');
    console.log('Client 2 acquired the freed lock');
    await client2.unlock(freedLock.id);
  } catch (err) {
    console.log(`Force unlock failed (may need admin scope): ${err.message}`);
    await client1.unlock(stuckLock.id);
  }

  await client1.disconnect();
  await client2.disconnect();
  console.log('\n=== Locks demo complete ===');
}

main().catch(console.error);
