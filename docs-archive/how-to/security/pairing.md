---
title: "Pairing"
description: "Zero-configuration secure pairing between clients and routers."
section: how-to
order: 3
---
# Pairing

Zero-configuration secure pairing between clients and routers.

## Overview

Pairing provides a user-friendly way to establish secure connections without pre-sharing tokens. It uses a short PIN code for initial authentication, then exchanges long-term credentials.

## How Pairing Works

```
1. Client requests pairing
2. Router displays PIN code (e.g., "847291")
3. User enters PIN in client
4. Router verifies PIN
5. Router issues capability token to client
6. Client stores token for future connections
```

## Router Configuration

### Enable Pairing

```yaml
# clasp.yaml
server:
  port: 7330
  security:
    pairing:
      enabled: true
      pin_length: 6
      pin_timeout: 300  # 5 minutes
      default_permissions:
        read: ["/**"]
        write: ["/**"]
```

### CLI

```bash
clasp server --pairing --pairing-timeout 300
```

## Initiating Pairing

### Client Request

```javascript
const { ClaspBuilder } = require('@clasp-to/core');

// Start pairing process
const pairing = await Clasp.pair('ws://192.168.1.100:7330', {
  name: 'My Phone'
});

console.log('Please enter this PIN on the router:');
// User sees PIN displayed on router

// Wait for user to enter PIN on router side
const client = await pairing.waitForApproval();

// Client is now connected with issued token
// Token is automatically stored for future use
```

### Python

```python
from clasp import ClaspBuilder

# Start pairing
pairing = await Clasp.pair('ws://192.168.1.100:7330', name='My Device')

# Wait for approval
client = await pairing.wait_for_approval()
```

## Router-Side PIN Display

### Desktop App

The CLASP desktop app displays the PIN in the UI when a pairing request arrives.

### CLI

```bash
# Router shows pairing requests
clasp server --pairing

# Output when pairing requested:
# Pairing request from "My Phone"
# PIN: 847291
# Expires in 5:00
```

### Programmatic

```javascript
// Custom pairing UI
router.on('pairing_request', (request) => {
  console.log(`Device "${request.name}" wants to pair`);
  console.log(`PIN: ${request.pin}`);

  // Display PIN to user via your UI
  displayPinToUser(request.pin, request.expiresIn);
});
```

## Approving Pairing

### Automatic (PIN-based)

User enters PIN displayed by router into client:

```javascript
// Client-side: user enters PIN
await pairing.submitPin('847291');
```

### Manual Approval

Router operator manually approves:

```javascript
// Router-side
router.on('pairing_request', async (request) => {
  const approved = await askUserToApprove(request);
  if (approved) {
    await request.approve({
      read: ['/sensors/**'],
      write: ['/control/**']
    });
  } else {
    await request.reject('Access denied');
  }
});
```

## Stored Credentials

After pairing, tokens are stored locally:

### Default Locations

- **macOS**: `~/Library/Application Support/clasp/tokens.json`
- **Linux**: `~/.config/clasp/tokens.json`
- **Windows**: `%APPDATA%\clasp\tokens.json`

### Token File Format

```json
{
  "routers": {
    "192.168.1.100:7330": {
      "token": "eyJhbGciOi...",
      "paired_at": "2024-01-15T10:30:00Z",
      "name": "Studio Router"
    }
  }
}
```

### Auto-Reconnect

```javascript
// Future connections automatically use stored token
const client = await new ClaspBuilder('ws://192.168.1.100:7330');
// Token loaded from storage automatically
```

## Revoking Paired Devices

### Router-Side

```bash
# List paired devices
clasp paired list

# Revoke specific device
clasp paired revoke "My Phone"

# Revoke all
clasp paired revoke-all
```

### Programmatic

```javascript
// List paired devices
const devices = await router.listPairedDevices();

// Revoke device
await router.revokePairedDevice('device-id');
```

## Custom Pairing Flow

Implement custom pairing logic:

```javascript
// Router setup
const router = new Router(config);

router.setPairingHandler(async (request) => {
  // Custom validation
  if (request.name.includes('unauthorized')) {
    return { approved: false, reason: 'Device name not allowed' };
  }

  // Generate custom token
  const token = generateClaspToken({
    read: ['/public/**'],
    write: []  // Read-only for new devices
  }, {
    subject: request.name,
    expiresIn: '30d'
  });

  return {
    approved: true,
    token: token,
    message: 'Welcome! You have read-only access.'
  };
});
```

## QR Code Pairing

Alternative to PIN:

```javascript
// Router generates QR code
const qrData = await router.generatePairingQR({
  expiresIn: 300,
  permissions: { read: ['/**'], write: ['/**'] }
});

// Display QR code
displayQRCode(qrData.image);

// Client scans QR code
const client = await Clasp.pairWithQR(qrData.code);
```

## Security Considerations

### PIN Security

- Use 6+ digit PINs
- Short expiration (5 minutes)
- Rate limit PIN attempts
- Display PIN only on trusted display

### Token Security

- Store tokens securely (encrypted at rest)
- Set appropriate expiration
- Use TLS for all connections
- Implement token refresh

### Physical Security

- Pairing requires physical access to see PIN
- Consider environment when displaying PIN
- Implement "pairing mode" with timeout

## Troubleshooting

### "Pairing not enabled"

Enable pairing in router configuration.

### "PIN expired"

Restart pairing process within timeout window.

### "Invalid PIN"

Verify PIN was entered correctly. PINs are case-sensitive if alphanumeric.

### "Stored token not working"

Token may have expired or been revoked. Re-pair the device.

## Next Steps

- [Enable TLS](enable-tls.md)
- [Capability Tokens](capability-tokens.md)
- [Manual Connection](../discovery/manual-connection.md)
