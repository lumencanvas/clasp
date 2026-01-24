/**
 * CLASP Discovery Example
 *
 * Demonstrates automatic service discovery:
 * - mDNS (Bonjour/Avahi) for LAN discovery
 * - UDP broadcast fallback
 * - Manual connection for browsers/WAN
 *
 * Usage:
 *   node discovery.js
 */

import { Clasp, Discovery } from '@clasp-to/core';

async function main() {
  console.log('=== CLASP Discovery Example ===\n');

  // =====================
  // 1. Automatic Discovery (mDNS)
  // =====================
  console.log('--- 1. mDNS Discovery ---');
  console.log('Searching for CLASP servers on the local network...\n');

  const discovery = new Discovery();

  // Listen for discovered servers
  discovery.on('found', (server) => {
    console.log(`[FOUND] ${server.name}`);
    console.log(`        URL: ${server.url}`);
    console.log(`        Version: ${server.version}`);
    console.log(`        Features: ${server.features.join(', ')}`);
    console.log();
  });

  discovery.on('lost', (server) => {
    console.log(`[LOST] ${server.name} (${server.url})`);
  });

  // Start mDNS discovery
  // Service type: _clasp._tcp.local
  await discovery.start({
    mdns: true,
    udpBroadcast: true,
    timeout: 5000 // Search for 5 seconds
  });

  console.log('Waiting for discovery (5 seconds)...\n');
  await new Promise(r => setTimeout(r, 5000));

  // Get all discovered servers
  const servers = discovery.servers;
  console.log(`\nDiscovered ${servers.length} server(s)`);

  // =====================
  // 2. Connect to First Available
  // =====================
  if (servers.length > 0) {
    console.log('\n--- 2. Connecting to First Available ---');

    const server = servers[0];
    console.log(`Connecting to: ${server.name} at ${server.url}`);

    const client = new Clasp(server.url);
    await client.connect({ name: 'Discovery Client' });
    console.log('Connected successfully!');

    // Do something with the connection
    client.on('/**', (value, address) => {
      console.log(`[${server.name}] ${address} = ${JSON.stringify(value)}`);
    });

    await new Promise(r => setTimeout(r, 2000));
    await client.disconnect();
  }

  // =====================
  // 3. Discovery with Filtering
  // =====================
  console.log('\n--- 3. Filtered Discovery ---');
  console.log('Searching for servers with specific features...\n');

  const filteredServers = await Discovery.find({
    features: ['param', 'event'], // Must support params and events
    timeout: 3000
  });

  console.log(`Found ${filteredServers.length} server(s) with required features`);
  filteredServers.forEach(s => {
    console.log(`  - ${s.name}: ${s.url}`);
  });

  // =====================
  // 4. UDP Broadcast Fallback
  // =====================
  console.log('\n--- 4. UDP Broadcast Fallback ---');
  console.log('Using UDP broadcast when mDNS is unavailable...\n');

  const udpDiscovery = new Discovery();

  await udpDiscovery.start({
    mdns: false, // Disable mDNS
    udpBroadcast: true,
    udpPort: 7331, // CLASP discovery port
    timeout: 3000
  });

  await new Promise(r => setTimeout(r, 3000));

  const udpServers = udpDiscovery.servers;
  console.log(`UDP broadcast found ${udpServers.length} server(s)`);

  // =====================
  // 5. Manual Connection (for browsers)
  // =====================
  console.log('\n--- 5. Manual Connection ---');
  console.log('Browsers cannot use mDNS/UDP. Use manual URL or QR code.\n');

  // Example: Connect with known URL
  const manualUrl = process.env.CLASP_URL || 'ws://localhost:7330';
  console.log(`Manual URL: ${manualUrl}`);

  try {
    const manual = new Clasp(manualUrl);
    await manual.connect({ name: 'Manual Client' });
    console.log('Manual connection successful!');

    // Get server info
    const info = manual.serverInfo;
    console.log(`Server: ${info.name} v${info.version}`);
    console.log(`Features: ${info.features.join(', ')}`);

    await manual.disconnect();
  } catch (err) {
    console.log(`Could not connect to ${manualUrl}: ${err.message}`);
  }

  // =====================
  // 6. Discovery Events
  // =====================
  console.log('\n--- 6. Continuous Discovery ---');
  console.log('Monitoring for servers joining/leaving (10 seconds)...\n');

  const continuous = new Discovery();

  continuous.on('found', (server) => {
    console.log(`[+] Server joined: ${server.name} at ${server.url}`);
  });

  continuous.on('lost', (server) => {
    console.log(`[-] Server left: ${server.name}`);
  });

  continuous.on('error', (err) => {
    console.log(`[!] Discovery error: ${err.message}`);
  });

  // Start continuous discovery (no timeout)
  await continuous.start({
    mdns: true,
    udpBroadcast: true,
    continuous: true
  });

  await new Promise(r => setTimeout(r, 10000));

  // Clean up
  await continuous.stop();
  await discovery.stop();
  await udpDiscovery.stop();

  console.log('\n=== Discovery demo complete ===');
}

main().catch(console.error);
