# Troubleshooting

Solutions to common CLASP issues.

## Connection Issues

### "Connection refused"

**Cause:** Router not running or wrong address.

**Solutions:**
1. Start the router: `clasp server --port 7330`
2. Verify address: `ws://localhost:7330` not `http://`
3. Check port: `lsof -i :7330`

### "WebSocket connection failed"

**Cause:** URL format or network issue.

**Solutions:**
```javascript
// Correct
const client = new Clasp('ws://localhost:7330');
const client = new Clasp('wss://server.com:7330');  // TLS

// Incorrect
const client = new Clasp('http://localhost:7330');
const client = new Clasp('localhost:7330');
```

### Frequent disconnections

**Cause:** Network instability or firewall.

**Solutions:**
1. Check network: `ping server-ip`
2. Enable reconnection:
   ```javascript
   new ClaspBuilder(url).withReconnect(true, 5000).connect();
   ```
3. Check firewall settings

### TLS certificate errors

**Cause:** Invalid or self-signed certificate.

**Solutions:**
1. Verify certificate: `openssl s_client -connect server:7330`
2. For development only:
   ```javascript
   process.env.NODE_TLS_REJECT_UNAUTHORIZED = '0';
   ```

## Message Issues

### Messages not received

**Cause:** Subscription pattern doesn't match.

**Solutions:**
```javascript
// Test with catch-all
client.on('/**', (v, a) => console.log(a, v));

// Check pattern matches
'/lights/*/brightness'  // Matches /lights/front/brightness
'/lights/**'            // Matches /lights/front/rgb/red
```

### Values not updating

**Cause:** Using wrong signal type.

**Solutions:**
```javascript
// Params (stored):
await client.set('/path', value);

// Events (not stored):
await client.emit('/path', value);

// Use set() for state that should persist
```

### High latency

**Cause:** QoS or network issues.

**Solutions:**
```javascript
// Use stream for high-rate data
client.stream('/fast/data', value);

// Rate limit subscriptions
client.on('/fast/**', callback, { maxRate: 30 });
```

## Bridge Issues

### OSC not working

**Cause:** Port conflict or wrong address.

**Solutions:**
```bash
# Check port
lsof -i :9000

# Use different port
clasp osc --port 9001

# Verify OSC addresses map correctly
# OSC: /fader/1 → CLASP: /osc/fader/1
```

### MIDI device not found

**Cause:** Device not connected or permission issue.

**Solutions:**
```bash
# List devices
clasp midi --list

# Linux: Add user to audio group
sudo usermod -a -G audio $USER

# Use exact device name
clasp midi --device "Exact Name Here"
```

### DMX not working

**Cause:** Device permissions or wrong port.

**Solutions:**
```bash
# Linux: Check device
ls -la /dev/ttyUSB*

# Set permissions
sudo chmod 666 /dev/ttyUSB0

# Or add user to dialout group
sudo usermod -a -G dialout $USER
```

## Performance Issues

### High CPU usage

**Cause:** Too many subscriptions or no rate limiting.

**Solutions:**
```javascript
// Rate limit
client.on('/high/rate/**', callback, { maxRate: 30 });

// Use epsilon
client.on('/analog/**', callback, { epsilon: 0.01 });

// Be specific
client.on('/specific/path', callback);  // Not '/**'
```

### Memory growth

**Cause:** Subscription leaks.

**Solutions:**
```javascript
// Always unsubscribe
const unsub = client.on('/path', callback);
// Later:
unsub();

// React pattern
useEffect(() => {
  const unsub = client.on('/path', callback);
  return () => unsub();
}, []);
```

## Debug Logging

### Enable logs

```bash
# Rust (server/CLI)
RUST_LOG=debug clasp server

# Node.js
DEBUG=* node app.js
```

### Log levels

| Level | Use |
|-------|-----|
| error | Only errors |
| warn | Warnings and errors |
| info | Normal operation |
| debug | Detailed debugging |
| trace | Very verbose |

## Getting Help

When reporting issues, include:

1. **Version:** `clasp --version`, `npm list @clasp-to/core`
2. **Platform:** OS, Node/Python version
3. **Steps to reproduce**
4. **Error messages** (full output)
5. **Minimal code example**

File issues at: https://github.com/lumencanvas/clasp/issues
