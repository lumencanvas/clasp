/**
 * CLASP Signal Types Example
 *
 * Demonstrates all five signal types in CLASP:
 * - Param: Stateful values that persist and sync
 * - Event: One-shot triggers that don't persist
 * - Stream: High-frequency data (lossy, fast)
 * - Gesture: Phased input (start/move/end)
 * - Timeline: Automation lanes with keyframes
 *
 * Usage:
 *   node signal-types.js
 */

import { Clasp } from '@clasp-to/core';

const CLASP_URL = process.env.CLASP_URL || 'ws://localhost:7330';

async function main() {
  console.log('=== CLASP Signal Types Example ===\n');

  const client = new Clasp(CLASP_URL);
  await client.connect({ name: 'Signal Types Demo' });
  console.log('Connected to CLASP server');

  // =====================
  // 1. PARAM - Stateful Values
  // =====================
  console.log('\n--- 1. PARAM (Stateful Values) ---');
  console.log('Params persist on the server and sync to late joiners.\n');

  // Subscribe to params
  client.on('/mixer/**', (value, address, meta) => {
    console.log(`[PARAM] ${address} = ${JSON.stringify(value)} (rev: ${meta.revision})`);
  });

  await new Promise(r => setTimeout(r, 100));

  // Set some param values
  await client.set('/mixer/master/volume', 0.8);
  await client.set('/mixer/master/mute', false);
  await client.set('/mixer/channel/1/volume', 0.65);
  await client.set('/mixer/channel/1/pan', 0.0);
  await client.set('/mixer/channel/1/eq', { low: 0, mid: 2, high: -1 });

  // Get current value (sync - uses cached value)
  const masterVol = client.value('/mixer/master/volume');
  console.log(`\nCached master volume: ${masterVol}`);

  // Get current value (async - fetches from server)
  const channel1 = await client.get('/mixer/channel/1/volume');
  console.log(`Fetched channel 1 volume: ${channel1.value} (rev: ${channel1.revision})`);

  await new Promise(r => setTimeout(r, 300));

  // =====================
  // 2. EVENT - One-Shot Triggers
  // =====================
  console.log('\n--- 2. EVENT (One-Shot Triggers) ---');
  console.log('Events fire once and are not stored. Miss it and it\'s gone.\n');

  // Subscribe to events
  client.onEvent('/cue/**', (payload, address) => {
    console.log(`[EVENT] ${address}:`, payload);
  });

  client.onEvent('/button/**', (payload, address) => {
    console.log(`[EVENT] ${address} pressed`);
  });

  await new Promise(r => setTimeout(r, 100));

  // Emit events
  await client.emit('/cue/go', { cueId: 'intro', fadeTime: 2.0 });
  await client.emit('/cue/stop', { immediate: false });
  await client.emit('/button/play', null);
  await client.emit('/button/record', { armed: true });

  // Events with metadata
  await client.emit('/notification/alert', {
    title: 'Warning',
    message: 'CPU usage high',
    level: 'warning'
  });

  await new Promise(r => setTimeout(r, 300));

  // =====================
  // 3. STREAM - High-Frequency Data
  // =====================
  console.log('\n--- 3. STREAM (High-Frequency Data) ---');
  console.log('Streams are lossy but fast. Use for sensors, meters, etc.\n');

  let streamCount = 0;

  // Subscribe to streams with rate limiting
  client.onStream('/sensor/**', (value, address) => {
    streamCount++;
    if (streamCount % 10 === 0) {
      console.log(`[STREAM] ${address} = ${JSON.stringify(value)} (received ${streamCount} total)`);
    }
  }, { maxRate: 30 }); // Limit to 30 updates/sec

  await new Promise(r => setTimeout(r, 100));

  // Stream sensor data at 60Hz for 1 second
  console.log('Streaming 60 values...');
  for (let i = 0; i < 60; i++) {
    const t = i / 60;
    await client.stream('/sensor/accelerometer', {
      x: Math.sin(t * Math.PI * 2),
      y: Math.cos(t * Math.PI * 2),
      z: 0.98 + Math.random() * 0.02
    });

    await client.stream('/sensor/temperature', 22.5 + Math.random() * 0.5);

    await new Promise(r => setTimeout(r, 16));
  }

  console.log(`Sent 120 stream values, received ${streamCount} (rate limited)`);

  await new Promise(r => setTimeout(r, 300));

  // =====================
  // 4. GESTURE - Phased Input
  // =====================
  console.log('\n--- 4. GESTURE (Phased Input) ---');
  console.log('Gestures have start/move/end phases. Used for touch, pen, drag.\n');

  // Subscribe to gestures
  client.onGesture('/input/**', (gesture) => {
    const { id, phase, x, y } = gesture;
    console.log(`[GESTURE] ${phase.toUpperCase()} id=${id.slice(-6)} at (${x?.toFixed(0)}, ${y?.toFixed(0)})`);
  });

  await new Promise(r => setTimeout(r, 100));

  // Simulate a drag gesture
  const gestureId = `drag-${Date.now()}`;

  await client.gesture('/input/mouse', { id: gestureId, phase: 'start', x: 100, y: 100 });

  for (let i = 1; i <= 5; i++) {
    await client.gesture('/input/mouse', {
      id: gestureId,
      phase: 'move',
      x: 100 + i * 20,
      y: 100 + i * 10
    });
    await new Promise(r => setTimeout(r, 50));
  }

  await client.gesture('/input/mouse', { id: gestureId, phase: 'end', x: 200, y: 150 });

  await new Promise(r => setTimeout(r, 300));

  // =====================
  // 5. TIMELINE - Automation
  // =====================
  console.log('\n--- 5. TIMELINE (Automation Lanes) ---');
  console.log('Timelines store keyframes for automated playback.\n');

  // Subscribe to timeline updates
  client.onTimeline('/automation/**', (timeline, address) => {
    console.log(`[TIMELINE] ${address}: ${timeline.keyframes.length} keyframes, duration=${timeline.duration}ms`);
  });

  await new Promise(r => setTimeout(r, 100));

  // Create a timeline with keyframes
  await client.timeline('/automation/light/brightness', {
    duration: 5000, // 5 seconds
    loop: true,
    keyframes: [
      { time: 0, value: 0.0, easing: 'linear' },
      { time: 1000, value: 1.0, easing: 'ease-out' },
      { time: 3000, value: 1.0, easing: 'linear' },
      { time: 4000, value: 0.3, easing: 'ease-in-out' },
      { time: 5000, value: 0.0, easing: 'linear' }
    ]
  });

  // Create a color timeline
  await client.timeline('/automation/light/color', {
    duration: 5000,
    loop: true,
    keyframes: [
      { time: 0, value: { r: 255, g: 0, b: 0 }, easing: 'linear' },
      { time: 2500, value: { r: 0, g: 255, b: 0 }, easing: 'linear' },
      { time: 5000, value: { r: 255, g: 0, b: 0 }, easing: 'linear' }
    ]
  });

  // Control timeline playback
  await client.emit('/automation/light/play', { startTime: client.time() });

  await new Promise(r => setTimeout(r, 500));

  // Query timeline state
  const brightnessTL = await client.getTimeline('/automation/light/brightness');
  console.log('\nBrightness timeline:', brightnessTL);

  await new Promise(r => setTimeout(r, 300));

  // =====================
  // Summary
  // =====================
  console.log('\n=== Signal Type Summary ===');
  console.log('| Type     | QoS     | Persists | Use Case                    |');
  console.log('|----------|---------|----------|-----------------------------|');
  console.log('| Param    | Confirm | Yes      | Faders, settings, state     |');
  console.log('| Event    | Confirm | No       | Button press, cue trigger   |');
  console.log('| Stream   | Fire    | No       | Sensors, meters (30-60Hz)   |');
  console.log('| Gesture  | Fire    | No       | Touch, pen, mouse drag      |');
  console.log('| Timeline | Commit  | Yes      | Animation, automation       |');

  await client.disconnect();
  console.log('\n=== Signal types demo complete ===');
}

main().catch(console.error);
