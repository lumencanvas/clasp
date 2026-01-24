/**
 * CLASP Gestures Example
 *
 * Demonstrates gesture handling for touch, pen, and mouse input:
 * - Gesture lifecycle (start -> move -> end)
 * - Multi-touch support
 * - Gesture coalescing (bandwidth reduction)
 * - Gesture metadata (pressure, tilt, etc.)
 *
 * Usage:
 *   node gestures.js
 */

import { Clasp } from '@clasp-to/core';

const CLASP_URL = process.env.CLASP_URL || 'ws://localhost:7330';

async function main() {
  console.log('=== CLASP Gestures Example ===\n');

  // Create publisher (simulates touch input device)
  const publisher = new Clasp(CLASP_URL);
  await publisher.connect({ name: 'Touch Input' });
  console.log('Publisher connected');

  // Create subscriber (receives gestures)
  const subscriber = new Clasp(CLASP_URL);
  await subscriber.connect({ name: 'Gesture Viewer' });
  console.log('Subscriber connected');

  // Track active gestures
  const activeGestures = new Map();

  // Subscribe to gesture updates
  subscriber.onGesture('/touch/**', (gesture) => {
    const { id, phase, x, y, pressure, timestamp } = gesture;

    switch (phase) {
      case 'start':
        activeGestures.set(id, { startX: x, startY: y, moves: 0 });
        console.log(`[GESTURE START] id=${id} at (${x.toFixed(1)}, ${y.toFixed(1)})`);
        break;

      case 'move':
        const g = activeGestures.get(id);
        if (g) {
          g.moves++;
          g.lastX = x;
          g.lastY = y;
          // Only log every 10th move to reduce spam
          if (g.moves % 10 === 0) {
            console.log(`[GESTURE MOVE] id=${id} at (${x.toFixed(1)}, ${y.toFixed(1)}) pressure=${(pressure || 1).toFixed(2)} moves=${g.moves}`);
          }
        }
        break;

      case 'end':
        const ended = activeGestures.get(id);
        if (ended) {
          const dx = (ended.lastX || x) - ended.startX;
          const dy = (ended.lastY || y) - ended.startY;
          const distance = Math.sqrt(dx * dx + dy * dy);
          console.log(`[GESTURE END] id=${id} total_moves=${ended.moves} distance=${distance.toFixed(1)}px`);
          activeGestures.delete(id);
        }
        break;

      case 'cancel':
        console.log(`[GESTURE CANCEL] id=${id}`);
        activeGestures.delete(id);
        break;
    }
  });

  await new Promise(r => setTimeout(r, 100));

  // =====================
  // 1. Simple Touch Gesture
  // =====================
  console.log('\n--- 1. Simple Touch Gesture ---');

  const gestureId1 = `touch-${Date.now()}`;

  // Start
  await publisher.gesture('/touch/finger/1', {
    id: gestureId1,
    phase: 'start',
    x: 100,
    y: 100
  });

  // Simulate 60Hz movement for 1 second
  for (let i = 0; i < 60; i++) {
    await publisher.gesture('/touch/finger/1', {
      id: gestureId1,
      phase: 'move',
      x: 100 + i * 3,
      y: 100 + Math.sin(i * 0.2) * 50
    });
    await new Promise(r => setTimeout(r, 16)); // ~60fps
  }

  // End
  await publisher.gesture('/touch/finger/1', {
    id: gestureId1,
    phase: 'end',
    x: 280,
    y: 100
  });

  await new Promise(r => setTimeout(r, 200));

  // =====================
  // 2. Pressure-Sensitive Pen
  // =====================
  console.log('\n--- 2. Pressure-Sensitive Pen ---');

  const penId = `pen-${Date.now()}`;

  await publisher.gesture('/touch/pen', {
    id: penId,
    phase: 'start',
    x: 50,
    y: 50,
    pressure: 0.1,
    tiltX: 0,
    tiltY: 45
  });

  // Draw a stroke with varying pressure
  for (let i = 0; i < 30; i++) {
    const t = i / 30;
    await publisher.gesture('/touch/pen', {
      id: penId,
      phase: 'move',
      x: 50 + i * 10,
      y: 50 + Math.sin(t * Math.PI) * 30,
      pressure: 0.1 + Math.sin(t * Math.PI) * 0.7, // 0.1 -> 0.8 -> 0.1
      tiltX: t * 20 - 10,
      tiltY: 45
    });
    await new Promise(r => setTimeout(r, 33)); // ~30fps
  }

  await publisher.gesture('/touch/pen', {
    id: penId,
    phase: 'end',
    x: 350,
    y: 50,
    pressure: 0
  });

  await new Promise(r => setTimeout(r, 200));

  // =====================
  // 3. Multi-Touch (Pinch Zoom)
  // =====================
  console.log('\n--- 3. Multi-Touch (Pinch Zoom) ---');

  const finger1 = `finger-1-${Date.now()}`;
  const finger2 = `finger-2-${Date.now()}`;

  // Start both fingers
  await publisher.gesture('/touch/finger/1', {
    id: finger1,
    phase: 'start',
    x: 200,
    y: 200
  });

  await publisher.gesture('/touch/finger/2', {
    id: finger2,
    phase: 'start',
    x: 220,
    y: 200
  });

  // Pinch out (spread fingers apart)
  for (let i = 0; i < 20; i++) {
    const spread = i * 5;

    await publisher.gesture('/touch/finger/1', {
      id: finger1,
      phase: 'move',
      x: 200 - spread,
      y: 200
    });

    await publisher.gesture('/touch/finger/2', {
      id: finger2,
      phase: 'move',
      x: 220 + spread,
      y: 200
    });

    await new Promise(r => setTimeout(r, 50));
  }

  // End both fingers
  await publisher.gesture('/touch/finger/1', { id: finger1, phase: 'end', x: 100, y: 200 });
  await publisher.gesture('/touch/finger/2', { id: finger2, phase: 'end', x: 320, y: 200 });

  await new Promise(r => setTimeout(r, 200));

  // =====================
  // 4. High-Frequency Input (Demonstrates Coalescing)
  // =====================
  console.log('\n--- 4. High-Frequency Input (240Hz Pen) ---');
  console.log('Sending 240 gesture moves (with coalescing, expect ~24 received)...');

  const hfId = `hf-${Date.now()}`;
  let sentCount = 0;

  await publisher.gesture('/touch/highfreq', { id: hfId, phase: 'start', x: 0, y: 0 });

  // Simulate 240Hz pen input for 1 second
  const startTime = Date.now();
  for (let i = 0; i < 240; i++) {
    await publisher.gesture('/touch/highfreq', {
      id: hfId,
      phase: 'move',
      x: i,
      y: Math.sin(i * 0.1) * 50
    });
    sentCount++;

    // Simulate ~240Hz
    const elapsed = Date.now() - startTime;
    const expected = (i + 1) * (1000 / 240);
    if (expected > elapsed) {
      await new Promise(r => setTimeout(r, expected - elapsed));
    }
  }

  await publisher.gesture('/touch/highfreq', { id: hfId, phase: 'end', x: 240, y: 0 });

  console.log(`Sent ${sentCount} moves`);
  await new Promise(r => setTimeout(r, 500));

  // Check coalescing stats
  const hfGesture = activeGestures.get(hfId);
  if (hfGesture) {
    const reduction = ((sentCount - hfGesture.moves) / sentCount * 100).toFixed(1);
    console.log(`Received ${hfGesture.moves} moves (${reduction}% bandwidth reduction)`);
  }

  // =====================
  // 5. Gesture with Metadata
  // =====================
  console.log('\n--- 5. Gesture with Custom Metadata ---');

  const metaId = `meta-${Date.now()}`;

  await publisher.gesture('/touch/custom', {
    id: metaId,
    phase: 'start',
    x: 100,
    y: 100,
    // Custom metadata
    metadata: {
      device: 'wacom-intuos',
      tool: 'brush',
      size: 24,
      color: '#ff5500'
    }
  });

  await publisher.gesture('/touch/custom', {
    id: metaId,
    phase: 'move',
    x: 150,
    y: 120,
    metadata: {
      device: 'wacom-intuos',
      tool: 'brush',
      size: 24,
      color: '#ff5500'
    }
  });

  await publisher.gesture('/touch/custom', {
    id: metaId,
    phase: 'end',
    x: 200,
    y: 140
  });

  await new Promise(r => setTimeout(r, 300));

  await publisher.disconnect();
  await subscriber.disconnect();
  console.log('\n=== Gesture demos complete ===');
}

main().catch(console.error);
