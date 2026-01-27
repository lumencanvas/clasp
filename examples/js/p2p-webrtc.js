/**
 * CLASP P2P WebRTC Example
 *
 * Demonstrates peer-to-peer communication using WebRTC DataChannels.
 * Two peers discover each other via a rendezvous server and establish
 * a direct connection for ultra-low-latency communication.
 *
 * Usage:
 *   # Terminal 1 - Start as initiator
 *   PEER_ID=peer-a node p2p-webrtc.js
 *
 *   # Terminal 2 - Start as responder
 *   PEER_ID=peer-b CONNECT_TO=peer-a node p2p-webrtc.js
 */

import { Clasp, P2PManager } from '@clasp-to/core';

const RENDEZVOUS_URL = process.env.RENDEZVOUS_URL || 'https://relay.clasp.to';
const PEER_ID = process.env.PEER_ID || `peer-${Date.now()}`;
const CONNECT_TO = process.env.CONNECT_TO;

async function main() {
  console.log(`\n=== CLASP P2P WebRTC Example ===`);
  console.log(`Peer ID: ${PEER_ID}`);

  // Create P2P manager with STUN/TURN configuration
  const p2p = new P2PManager({
    peerId: PEER_ID,
    rendezvousUrl: RENDEZVOUS_URL,
    iceServers: [
      { urls: 'stun:stun.l.google.com:19302' },
      { urls: 'stun:stun1.l.google.com:19302' }
    ],
    // Use both reliable (ordered) and unreliable (fast) channels
    useUnreliableChannel: true
  });

  // Handle incoming peer connections
  p2p.on('connection', (peer) => {
    console.log(`[P2P] Peer connected: ${peer.id}`);

    // Subscribe to messages from this peer
    peer.on('/chat/*', (value, address) => {
      console.log(`[${peer.id}] ${address}: ${value}`);
    });

    peer.on('/sensor/accel', (value) => {
      console.log(`[${peer.id}] Accelerometer: x=${value.x.toFixed(2)}, y=${value.y.toFixed(2)}, z=${value.z.toFixed(2)}`);
    });
  });

  p2p.on('disconnection', (peerId) => {
    console.log(`[P2P] Peer disconnected: ${peerId}`);
  });

  p2p.on('error', (err) => {
    console.error(`[P2P] Error: ${err.message}`);
  });

  // Register with rendezvous server
  console.log(`\nRegistering with rendezvous server...`);
  await p2p.register({
    tags: ['demo', 'webrtc'],
    metadata: {
      name: `Demo Peer ${PEER_ID}`,
      capabilities: ['chat', 'sensors']
    }
  });
  console.log(`Registered successfully!`);

  // If CONNECT_TO is specified, initiate connection
  if (CONNECT_TO) {
    console.log(`\nConnecting to peer: ${CONNECT_TO}...`);

    try {
      const peer = await p2p.connect(CONNECT_TO);
      console.log(`Connected to ${peer.id}!`);

      // Send a greeting over the reliable channel
      peer.set('/chat/greeting', `Hello from ${PEER_ID}!`);

      // Stream sensor data over the unreliable (fast) channel
      let i = 0;
      const sensorInterval = setInterval(() => {
        const t = i * 0.1;
        peer.stream('/sensor/accel', {
          x: Math.sin(t),
          y: Math.cos(t),
          z: Math.sin(t * 0.5)
        });
        i++;
      }, 100); // 10 Hz

      // Clean up on disconnect
      peer.on('close', () => {
        clearInterval(sensorInterval);
      });

    } catch (err) {
      console.error(`Failed to connect: ${err.message}`);
    }
  } else {
    console.log(`\nWaiting for incoming connections...`);
    console.log(`Run another instance with: CONNECT_TO=${PEER_ID} node p2p-webrtc.js`);
  }

  // Discover other peers
  console.log(`\nDiscovering peers...`);
  const peers = await p2p.discover({ tags: ['demo'] });
  console.log(`Found ${peers.length} peer(s):`);
  peers.forEach(p => {
    console.log(`  - ${p.id}: ${p.metadata?.name || 'Unknown'}`);
  });

  // Keep running
  console.log(`\nPress Ctrl+C to exit`);
  await new Promise(() => {});
}

main().catch(console.error);
