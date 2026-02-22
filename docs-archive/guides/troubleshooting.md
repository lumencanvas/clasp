---
title: "CLASP Troubleshooting Guide"
description: "This guide helps diagnose and resolve common issues with CLASP."
section: guides
order: 4
---
# CLASP Troubleshooting Guide

This guide helps diagnose and resolve common issues with CLASP.

## Connection Issues

### Cannot Connect to Server

**Symptoms:**
- "Connection refused" error
- "WebSocket connection failed"
- Client hangs during connect

**Solutions:**

1. **Verify server is running:**
   ```bash
   # Check if server is listening
   lsof -i :7330
   # Or
   netstat -an | grep 7330
   ```

2. **Check URL format:**
   ```javascript
   // Correct
   const clasp = new Clasp('ws://localhost:7330');
   const clasp = new Clasp('wss://example.com:7330');  // TLS

   // Incorrect
   const clasp = new Clasp('localhost:7330');  // Missing protocol
   const clasp = new Clasp('http://localhost:7330');  // Wrong protocol
   ```

3. **Check firewall settings:**
   ```bash
   # macOS
   sudo /usr/libexec/ApplicationFirewall/socketfilterfw --listapps

   # Linux
   sudo ufw status
   ```

4. **Try binding to all interfaces:**
   ```bash
   # Instead of 127.0.0.1:7330, use
   clasp serve --bind 0.0.0.0:7330
   ```

### Connection Drops Frequently

**Symptoms:**
- Intermittent disconnections
- "Connection reset" errors
- Client reconnects repeatedly

**Solutions:**

1. **Check network stability:**
   ```bash
   ping -c 100 server-ip
   ```

2. **Increase WebSocket timeout:**
   - Server-side configuration may need adjustment
   - Check for proxy/load balancer timeouts

3. **Enable keepalive:**
   - CLASP uses ping/pong for connection health
   - Ensure network equipment doesn't drop idle connections

4. **Check for network address translation (NAT) issues:**
   - Some NAT configurations timeout idle connections
   - Use a persistent connection or more frequent messages

### TLS/SSL Connection Fails

**Symptoms:**
- "Certificate verification failed"
- "SSL handshake error"
- Works with `ws://` but not `wss://`

**Solutions:**

1. **Verify certificate:**
   ```bash
   openssl s_client -connect server:7330
   ```

2. **Check certificate chain:**
   - Ensure intermediate certificates are included
   - Verify certificate hasn't expired

3. **For development, skip verification (NOT for production):**
   ```javascript
   // Only for development!
   process.env.NODE_TLS_REJECT_UNAUTHORIZED = '0';
   ```

## Message Issues

### Messages Not Received

**Symptoms:**
- Subscriptions don't fire
- Values don't update
- Events not delivered

**Solutions:**

1. **Verify subscription pattern:**
   ```javascript
   // Exact match
   clasp.on('/path/to/value', callback);

   // Single wildcard - matches one segment
   clasp.on('/path/*/value', callback);  // Matches /path/foo/value

   // Double wildcard - matches multiple segments
   clasp.on('/path/**', callback);  // Matches /path/foo/bar/value
   ```

2. **Check subscription timing:**
   ```javascript
   // Subscribe BEFORE values are published
   await clasp.connect();
   clasp.on('/my/pattern/**', callback);  // Subscribe first
   // Values published after this will be received
   ```

3. **Verify signal types:**
   ```javascript
   // Events are different from params
   clasp.emit('/event', payload);    // Event - ephemeral
   clasp.set('/param', value);       // Param - persisted
   ```

4. **Test with wildcard:**
   ```javascript
   // Subscribe to everything to verify messages are arriving
   clasp.on('/**', (value, address) => {
     console.log('Received:', address, value);
   });
   ```

### Values Not Persisting

**Symptoms:**
- Values reset on reconnect
- `get()` returns undefined
- Values lost after server restart

**Solutions:**

1. **Use `set()` not `stream()` for persistent values:**
   ```javascript
   clasp.set('/my/value', 42);      // Persisted
   clasp.stream('/my/value', 42);   // NOT persisted
   ```

2. **Check if server has persistence enabled:**
   - In-memory storage is lost on restart
   - Configure persistent storage if needed

3. **Verify value is actually being set:**
   ```javascript
   clasp.on('/my/value', (v) => console.log('Value set to:', v));
   clasp.set('/my/value', 42);
   ```

### High Latency

**Symptoms:**
- Delayed responses
- Sluggish UI updates
- Timing-sensitive operations fail

**Solutions:**

1. **Use appropriate QoS:**
   ```javascript
   clasp.stream('/fast/data', value);  // QoS.Fire - fastest, no confirmation
   clasp.set('/important/data', value); // QoS.Confirm - waits for ack
   ```

2. **Rate limit high-frequency data:**
   ```javascript
   clasp.on('/fast/source', callback, { maxRate: 30 });
   ```

3. **Use bundles for atomic operations:**
   ```javascript
   // Single network round-trip instead of multiple
   clasp.bundle([
     { set: ['/a', 1] },
     { set: ['/b', 2] },
     { set: ['/c', 3] }
   ]);
   ```

4. **Check network path:**
   ```bash
   traceroute server-ip
   mtr server-ip
   ```

## Bridge Issues

### OSC Bridge Not Working

**Symptoms:**
- OSC messages not received
- OSC messages not sent
- Connection appears ok but no data flows

**Solutions:**

1. **Verify OSC port binding:**
   ```bash
   # Check if port is in use
   lsof -i :8000

   # Try different port
   clasp bridge create --source osc:0.0.0.0:8001
   ```

2. **Test with OSC monitor:**
   - Use a tool like Protocol to verify OSC messages
   - Check source is sending to correct port

3. **Check address format:**
   ```javascript
   // OSC addresses become /osc/... in CLASP
   // OSC: /fader/1 -> CLASP: /osc/fader/1
   clasp.on('/osc/fader/1', callback);
   ```

### MIDI Bridge Not Working

**Symptoms:**
- MIDI device not detected
- MIDI messages not received
- MIDI output not working

**Solutions:**

1. **List available MIDI devices:**
   ```bash
   # Check MIDI devices are visible to system
   # macOS: Audio MIDI Setup
   # Linux: aconnect -l
   ```

2. **Verify device name:**
   ```bash
   clasp midi list
   # Use exact name from list
   clasp bridge create --source midi:"Device Name"
   ```

3. **Check MIDI routing:**
   - Ensure device is not claimed by another application
   - Some devices need driver installation

### DMX/Art-Net Bridge Not Working

**Symptoms:**
- Lights not responding
- Art-Net packets not received
- DMX output erratic

**Solutions:**

1. **Verify network configuration:**
   ```bash
   # Art-Net uses UDP broadcast
   # Check interface IP is in 2.x.x.x or 10.x.x.x range
   ifconfig
   ```

2. **Check universe numbers:**
   ```javascript
   // Universes are 0-indexed in most implementations
   clasp.set('/dmx/0/1', 255);  // Universe 0, Channel 1
   ```

3. **Verify hardware:**
   - Check DMX interface is powered
   - Verify termination on DMX chain
   - Test with DMX test software

## Performance Issues

### High CPU Usage

**Symptoms:**
- Server using excessive CPU
- System becomes unresponsive
- Fan noise increases

**Solutions:**

1. **Rate limit subscriptions:**
   ```javascript
   // Don't receive more than needed
   clasp.on('/high/rate/data', callback, { maxRate: 30 });
   ```

2. **Use epsilon for analog values:**
   ```javascript
   // Only receive when value changes significantly
   clasp.on('/fader/*', callback, { epsilon: 0.01 });
   ```

3. **Reduce subscription scope:**
   ```javascript
   // Instead of
   clasp.on('/**', callback);

   // Be specific
   clasp.on('/my/namespace/**', callback);
   ```

### Memory Growth

**Symptoms:**
- Memory usage increases over time
- Eventually crashes with OOM
- Performance degrades

**Solutions:**

1. **Clean up subscriptions:**
   ```javascript
   const unsub = clasp.on('/pattern', callback);
   // Later...
   unsub();  // Remove subscription when done
   ```

2. **Avoid storing all messages:**
   ```javascript
   // Don't do this
   const history = [];
   clasp.on('/**', (v, a) => history.push({ address: a, value: v }));

   // Do this instead
   const recentHistory = [];
   const MAX_HISTORY = 1000;
   clasp.on('/**', (v, a) => {
     recentHistory.push({ address: a, value: v });
     if (recentHistory.length > MAX_HISTORY) recentHistory.shift();
   });
   ```

3. **Check for circular references:**
   - Avoid storing WebSocket objects in closures
   - Use WeakMap for caches if appropriate

## Getting Help

### Collecting Debug Information

When reporting issues, include:

1. **Version information:**
   ```bash
   clasp --version
   npm list @clasp-to/core
   ```

2. **Enable debug logging:**
   ```bash
   # Server
   RUST_LOG=debug clasp serve

   # Node.js client
   DEBUG=* node app.js
   ```

3. **Network capture (if appropriate):**
   ```bash
   # Capture WebSocket traffic
   tcpdump -i any port 7330 -w clasp.pcap
   ```

### Reporting Bugs

File issues at: https://github.com/lumencanvas/clasp/issues

Include:
- CLASP version
- Operating system
- Steps to reproduce
- Expected vs actual behavior
- Relevant logs
- Minimal reproduction code
