# CLASP Protocol

CLASP (Collaborative Live Application State Protocol) is a real-time state synchronization protocol. This document covers how the chat app uses CLASP and the protocol's internals.

## Protocol Overview

CLASP provides three communication primitives over a single WebSocket connection:

| Operation | Method | Persistence | Pattern Matching | Use Case |
|-----------|--------|-------------|-----------------|----------|
| **set** | `client.set(address, value)` | Persistent (param) | Wildcard subscribers notified | Room metadata, presence, typing, profiles, bans |
| **emit** | `client.emit(address, payload)` | Ephemeral (event) | Wildcard subscribers notified | Chat messages, edit/delete, key exchange events |
| **subscribe** | `client.on(pattern, callback)` | N/A | `*` and `**` globs | Listen for changes on address patterns |

Additional operations:

| Operation | Method | Description |
|-----------|--------|-------------|
| **get** | `client.get(address)` | One-shot read of current param value |
| **stream** | `client.stream(address, value)` | High-frequency ephemeral updates (throttled) |
| **bundle** | `client.bundle(messages, opts)` | Batch multiple operations in a single frame |

### set vs emit

- **`set(address, value)`**: Stores `value` at `address` in the relay's state. All subscribers matching that address are notified. Setting `value` to `null` deletes the entry. The value persists across sessions and is included in state snapshots.

- **`emit(address, payload)`**: Sends `payload` to all subscribers matching `address`. Nothing is stored. If no one is subscribed, the event is lost. Used for messages because they are persisted client-side in IndexedDB, not on the relay.

### Subscription Patterns

CLASP subscriptions use glob-style pattern matching:

| Pattern | Matches |
|---------|---------|
| `/chat/room/abc/messages` | Exact address only |
| `/chat/room/abc/presence/*` | Any single segment: `/chat/room/abc/presence/user-1`, etc. |
| `/chat/room/*/meta` | Any room's meta: `/chat/room/abc/meta`, `/chat/room/xyz/meta` |
| `/chat/registry/**` | All addresses under `/chat/registry/` at any depth |

The callback receives `(value, address, meta)`:
- `value`: The data (for `set`: the new value or `null` on delete; for `emit`: the event payload)
- `address`: The exact address that triggered the callback
- `meta`: Protocol metadata (timestamps, session info)

## Connection Lifecycle

```
Client                                      Relay
  │                                           │
  │──── WebSocket connect ───────────────────>│
  │                                           │
  │──── HELLO { name, token, features } ─────>│
  │                                           │  Token validation
  │                                           │  Scope extraction
  │<──── WELCOME { session, config } ─────────│
  │                                           │
  │<──── SNAPSHOT { params } ─────────────────│  Current state dump
  │                                           │
  │──── SUBSCRIBE { pattern } ───────────────>│
  │──── SET { address, value } ──────────────>│  Scope check
  │──── PUBLISH { address, payload } ────────>│  Scope check
  │                                           │
  │<──── UPDATE { address, value } ───────────│  Subscription match
  │<──── EVENT { address, payload } ──────────│  Subscription match
  │                                           │
  │  ... bidirectional real-time exchange ...  │
```

### HELLO Handshake

The client initiates with a HELLO message containing:
- `name`: Display name
- `token`: CPSK auth token (from `/auth/register`, `/auth/login`, or `/auth/guest`)
- `features`: Requested protocol features (`['param', 'event', 'stream', 'gesture', 'timeline']`)
- `reconnect`: Whether automatic reconnection is enabled

The relay validates the token against the CpskValidator, extracts scopes, and responds with WELCOME containing the session ID and configuration.

### SNAPSHOT

After WELCOME, the relay sends a SNAPSHOT containing all current param values that the client is authorized to read. This populates the client-side cache so that subsequent subscriptions fire immediately for existing state.

## Binary Codec

CLASP uses a compact binary frame format over WebSocket binary frames:

```
┌─────────┬──────────┬─────────────┬──────────┐
│ Type(1B)│ Flags(1B)│ Length(var)  │ Payload  │
└─────────┴──────────┴─────────────┴──────────┘
```

- **Type**: Message type ID (HELLO=1, WELCOME=2, SET=3, PUBLISH=4, SUBSCRIBE=5, etc.)
- **Flags**: Compression, ack request, etc.
- **Length**: Variable-length integer encoding of payload size
- **Payload**: MessagePack-encoded data

The binary codec is implemented in the `@clasp-to/core` npm package (client) and `clasp-core`/`clasp-router` Rust crates (server).

## Message Types

| ID | Name | Direction | Description |
|----|------|-----------|-------------|
| 1 | HELLO | C -> S | Connection init with auth token |
| 2 | WELCOME | S -> C | Connection accepted, session ID |
| 3 | SET | C -> S | Store persistent param |
| 4 | PUBLISH | C -> S | Fire ephemeral event |
| 5 | SUBSCRIBE | C -> S | Register pattern subscription |
| 6 | UNSUBSCRIBE | C -> S | Remove subscription |
| 7 | UPDATE | S -> C | Param change notification |
| 8 | EVENT | S -> C | Event notification |
| 9 | SNAPSHOT | S -> C | Full state dump on connect |
| 10 | GET | C -> S | One-shot param read |
| 11 | RESULT | S -> C | Response to GET |
| 12 | STREAM | C -> S | High-frequency ephemeral update |
| 13 | BUNDLE | C -> S | Batched operations |
| 14 | GESTURE | C/S | Cursor/pointer position (for collaborative features) |
| 15 | ERROR | S -> C | Error response |

## Address Space

The chat app uses a structured address space under `/chat/`:

### User Addresses

```
/chat/user/{userId}/profile          User profile (name, avatarColor, status)
```

### Room Addresses

```
/chat/room/{roomId}/meta             Room metadata (name, type, encrypted, password, namespace)
/chat/room/{roomId}/messages         Message events (emit only, not persisted on relay)
/chat/room/{roomId}/presence/{uid}   User presence (set with heartbeat, null on leave)
/chat/room/{roomId}/typing/{uid}     Typing indicators (set true/null)
/chat/room/{roomId}/admin/{uid}      Admin role records
/chat/room/{roomId}/bans/{uid}       Ban records
/chat/room/{roomId}/reactions/{mid}  Message reactions
/chat/room/{roomId}/video/{uid}      WebRTC signaling for video rooms
```

### Crypto Addresses

```
/chat/room/{roomId}/crypto/pubkey/{userId}   ECDH public key for key exchange
/chat/room/{roomId}/crypto/proof/{userId}    Password proof hash (for gated rooms)
/chat/room/{roomId}/crypto/keyex/{peerId}    Encrypted room key delivery (emit)
```

### Registry Addresses

```
/chat/registry/rooms/{roomId}                Public room listing
/chat/registry/ns/{namespace}/{roomId}       Room in namespace
/chat/registry/ns-meta/{namespace}           Namespace metadata
/chat/registry/ns-meta/{namespace}/__auth    Namespace password data
```

### Friend Addresses

```
/chat/requests/{targetUserId}                Friend request inbox
```

## Scope Enforcement

Every CLASP operation is checked against the token's scope list before execution. Scopes follow the format `action:pattern` where:

- `action` is `read` or `write`
- `pattern` is a glob path (supports `*` for single segment, `**` for recursive)

A `set()` or `emit()` requires a `write` scope matching the target address. A `subscribe()` requires a `read` scope matching the pattern.

### Current Scope Set (per user)

```
read:/chat/**                                    Read all chat state
write:/chat/user/{userId}/**                     Own profile
write:/chat/requests/*                           Friend requests
write:/chat/room/*/messages                      Send messages (any room)
write:/chat/room/*/presence/{userId}             Own presence only
write:/chat/room/*/typing/{userId}               Own typing only
write:/chat/room/*/reactions/**                  Reactions
write:/chat/room/*/video/**                      Video signaling
write:/chat/room/*/crypto/pubkey/{userId}        Own ECDH public key only
write:/chat/room/*/crypto/proof/{userId}         Own password proof only
write:/chat/room/*/crypto/keyex/*                Key exchange delivery
write:/chat/room/*/admin/*                       Admin operations
write:/chat/room/*/bans/*                        Ban operations
write:/chat/room/*/meta                          Room metadata
write:/chat/registry/rooms/*                     Room registry
write:/chat/registry/ns/**                       Namespace registry
write:/chat/registry/ns-meta/**                  Namespace metadata
```

Key security restrictions:
- Crypto pubkey and proof paths are restricted to the user's own userId, preventing MITM key injection
- Presence and typing paths are userId-scoped, preventing impersonation
- All profile paths are userId-scoped

## Client SDK Usage

The `@clasp-to/core` package is used via `ClaspBuilder`:

```javascript
const builder = new ClaspBuilder('wss://relay.clasp.chat')
  .name('Alice')
  .token(cpskToken)
  .features(['param', 'event', 'stream', 'gesture', 'timeline'])
  .reconnect(true)

const client = await builder.connect()

// Persistent state
client.set('/chat/room/abc/presence/user-1', { name: 'Alice', ... })

// Ephemeral event
client.emit('/chat/room/abc/messages', { text: 'Hello', ... })

// Subscribe with glob
const unsub = client.on('/chat/room/abc/presence/*', (value, address) => {
  // value: the data, address: exact path that changed
})

// One-shot read
const meta = await client.get('/chat/room/abc/meta')

// Cleanup
unsub()
client.close()
```

## Rate Limiting

| Layer | Limit | Scope |
|-------|-------|-------|
| Client JS | 5 messages/sec | Per composable instance |
| CLASP relay | 30 messages/sec | Per WebSocket session |
| Auth login | 5 attempts/15 min | Per IP + per username |
| Auth register/guest | 10 attempts/15 min | Per IP |
