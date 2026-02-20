# Architecture Overview

## System Topology

```
+------------------+          wss://           +------------------+
|   Vue 3 SPA      | <======================> |   CLASP Relay    |
|   (Browser)      |     CLASP Protocol       |   (Rust)         |
|                  |     Binary WebSocket      |                  |
|  - Chat UI       |                           |  - State store   |
|  - E2E crypto    |                           |  - Pub/sub       |
|  - IndexedDB     |          https://         |  - Scope enforce |
|  - Web Crypto    | -----------------------> |  - Auth API      |
|                  |     REST (register/       |  - SQLite users  |
+------------------+     login/guest)          +------------------+
                                                       |
                                                 State snapshot
                                                       |
                                                 /data/state.json
```

The system has two main components:

1. **Frontend** (`apps/chat/`): A Vue 3 single-page application that handles all UI, E2E encryption, and state management via composables.

2. **Relay Server** (`deploy/relay/`): A Rust binary that runs two services:
   - **WebSocket server** (default port 7330): The CLASP protocol endpoint that handles real-time state synchronization and pub/sub messaging.
   - **HTTP auth server** (default port 7350): REST endpoints for user registration, login, and guest access. Issues CPSK tokens with scoped permissions.

All real-time communication flows through the CLASP protocol over a single WebSocket connection per client. The relay is *not* application-aware -- it stores and forwards state based on address paths and enforces scope-based access control.

## Frontend Architecture

### Pages

The app has three route pages:

| Route | Page | Purpose |
|-------|------|---------|
| `/` | `JoinPage.vue` | Landing page with relay URL input and join-link handling |
| `/auth` | `AuthPage.vue` | Registration, login, and guest access forms |
| `/chat` | `ChatPage.vue` | Main chat interface (requires auth token) |

Navigation guard in `router.js` redirects unauthenticated users from `/chat` to `/auth`, forwarding any `?join=` query parameter.

### Component Tree

```
ChatPage.vue
├── AppLayout.vue
│   ├── AppSidebar.vue
│   │   ├── RoomList.vue
│   │   │   └── RoomListItem.vue (per room)
│   │   ├── FriendList.vue
│   │   │   └── FriendItem.vue (per friend)
│   │   ├── RoomDiscovery.vue
│   │   │   ├── NamespaceCard.vue (per namespace)
│   │   │   └── NamespaceCreateDialog.vue
│   │   ├── NamespaceGroup.vue (per pinned namespace)
│   │   ├── RoomCreateDialog.vue
│   │   └── StatusPicker.vue
│   │
│   ├── AppHeader.vue
│   │   └── UserProfilePopup.vue
│   │
│   └── [Main Content Area]
│       ├── ChatView.vue (text rooms)
│       │   ├── MessageList.vue / VirtualMessageList.vue
│       │   │   ├── MessageItem.vue (per message)
│       │   │   │   ├── ReactionBadge.vue
│       │   │   │   └── ReactionPicker.vue
│       │   │   └── SystemMessage.vue
│       │   ├── MessageComposer.vue
│       │   │   └── EmojiPicker.vue
│       │   ├── MemberList.vue
│       │   │   └── MemberItem.vue
│       │   │       └── UserAvatar.vue
│       │   ├── TypingIndicator.vue
│       │   ├── KeyChangeWarning.vue
│       │   └── AdminPanel.vue
│       │
│       ├── VideoChannelView.vue (video rooms)
│       │   ├── VideoGrid.vue
│       │   │   └── VideoTile.vue (per participant)
│       │   ├── VideoControls.vue
│       │   ├── LocalPreview.vue
│       │   └── AdminPanel.vue
│       │
│       └── ComboChannelView.vue (combo rooms)
│           ├── [Video section]
│           │   ├── LocalPreview.vue
│           │   ├── VideoGrid.vue
│           │   │   └── VideoTile.vue (per participant)
│           │   └── VideoControls.vue
│           ├── [Chat section — inlined, not ChatView]
│           │   ├── MessageList.vue
│           │   ├── MessageComposer.vue
│           │   └── TypingIndicator.vue
│           └── AdminPanel.vue
```

### Composable Layer

All state management uses Vue 3 composables (the Composition API). Each composable is a module-level singleton -- calling `useFoo()` from any component returns the same shared reactive state.

| Composable | Responsibility |
|------------|----------------|
| `useClasp` | WebSocket connection lifecycle, CLASP `set`/`emit`/`subscribe`/`get`/`stream`/`bundle` wrappers |
| `useIdentity` | User identity (userId, displayName, avatarColor, status), profile announcement |
| `useAuth` | HTTP auth API calls (register, login, logout), token/credential storage |
| `useRooms` | Room CRUD, join/leave, room registry discovery, DM creation |
| `useChat` | Per-room messaging: send/receive/edit/delete, presence, typing, message signing and verification |
| `useCrypto` | E2E encryption: AES-256-GCM room keys, ECDH key exchange, key rotation, password proof gating |
| `useSigning` | ECDSA P-256 message signing: non-extractable keypair generation, sign/verify, peer key cache |
| `useKeyVerification` | TOFU key verification: peer signing key fingerprints, key change warnings, per-room warning state |
| `useAdmin` | Room administration: kick/ban/unban, promote/demote admins, room meta updates |
| `useNamespaces` | Namespace tree management: create/browse/subscribe, password protection, nested hierarchy |
| `useFriends` | Friend requests: send/accept/reject, two-step handshake via CLASP state |
| `useVideoRoom` | WebRTC video calls: SFU-less mesh via CLASP signaling, track management, per-peer audio level detection |
| `useVideoLayout` | Video layout modes (grid/spotlight/sidebar), peer pinning, active speaker auto-spotlight |
| `useAudioLevel` | Audio level detection via Web Audio API AnalyserNode; returns reactive `isSpeaking` and `audioLevel` |
| `useReactions` | Message reactions: add/remove, per-message reaction aggregation |
| `useNotifications` | Browser notifications, per-room unread counts |
| `useStorage` | IndexedDB persistence for messages, crypto keys, and TOFU fingerprints |

### Dependency Graph

```
useChat ──────> useClasp
    │           useIdentity
    │           useCrypto ──────> useClasp, useIdentity
    │           useSigning ────> useClasp, useIdentity
    │           useNotifications
    │           useStorage
    │           useRooms
    │
useAdmin ─────> useClasp, useIdentity, useRooms, useCrypto
useNamespaces > useClasp, useIdentity
useFriends ──> useClasp, useIdentity
useVideoRoom > useClasp, useIdentity, useAudioLevel
useVideoLayout > (receives isScreenSharing + speakingPeerIds from useVideoRoom)
useAudioLevel  > (standalone, used internally by useVideoRoom)
useKeyVerification > (standalone, uses IndexedDB via storage.js)
useAuth ──────> (standalone HTTP, no CLASP dependency)
```

### Utility Modules

| Module | Location | Purpose |
|--------|----------|---------|
| `crypto.js` | `src/lib/` | Web Crypto API wrappers: AES-GCM, ECDH P-256, ECDSA P-256, HKDF-SHA256, PBKDF2, key import/export |
| `storage.js` | `src/lib/` | IndexedDB operations: message cache, crypto key persistence, TOFU fingerprint storage |
| `constants.js` | `src/lib/` | CLASP address prefixes, TTL values, room types, avatar colors |
| `plugins.js` | `src/lib/` | Slash command system for chat (extensible plugin architecture) |
| `markdown.js` | `src/lib/` | Lightweight markdown renderer (bold, italic, code, links, strikethrough) with XSS protection |
| `utils.js` | `src/lib/` | Helper functions: `generateId`, `formatTime`, `formatRelativeTime`, `getInitials`, `getAvatarColor`, `truncate` |
| `emoji-data.js` | `src/lib/` | Emoji dataset for the emoji picker component |

## Relay Server Architecture

### Binary Structure

The relay is a single Rust binary (`clasp-relay`) with CLI arguments:

```
clasp-relay
  --port <PORT>           WebSocket port (default: 7330)
  --auth-port <PORT>      Auth HTTP port (default: 7350)
  --data-dir <PATH>       State persistence directory
  --cors-origin <ORIGINS> Comma-separated allowed CORS origins
```

### Internal Components

```
main.rs
├── CLI arg parsing (clap)
├── WebSocket server (axum + tokio-tungstenite)
│   ├── CLASP Router (clasp-router crate)
│   │   ├── State store (HashMap<String, Value>)
│   │   ├── Subscription matching (glob patterns)
│   │   └── Scope enforcement (CpskValidator)
│   ├── Connection handler
│   │   ├── HELLO/WELCOME handshake
│   │   ├── Token validation
│   │   ├── Rate limiting (30 msg/sec per session)
│   │   └── Binary frame codec
│   └── State persistence
│       ├── Periodic snapshot (60s interval)
│       └── Atomic write (tmp + rename)
│
auth.rs
├── Auth HTTP server (axum)
│   ├── POST /auth/register
│   ├── POST /auth/login
│   └── POST /auth/guest
├── SQLite user database
│   └── users(id, username, password_hash, created_at)
├── Argon2 password hashing
├── CPSK token generation with scoped permissions
├── Rate limiting (per-IP and per-username)
└── Configurable CORS
```

### State Persistence

The relay maintains all CLASP state in memory as a `HashMap<String, serde_json::Value>`. This state is periodically serialized to disk:

1. Every 60 seconds, the full state snapshot is written to `{data-dir}/state.json`
2. Writes are atomic: data goes to a `.tmp` file first, then `rename()` replaces the live file
3. On startup, the relay loads the snapshot to restore state

Events (`emit()`) are not persisted -- they are fire-and-forget. Only `set()` params are included in the snapshot.

## Data Flow

### Message Send (Encrypted Room)

```
User types message
        │
        v
  useChat.sendMessage()
        │
        ├── Client-side rate limit check (5 msg/sec)
        ├── Slash command routing (if starts with /)
        │
        v
  useCrypto.encrypt(roomId, text)
        │
        ├── Lookup AES-256-GCM room key from in-memory Map
        ├── Generate random 12-byte IV
        └── AES-GCM encrypt -> { ciphertext, iv } (base64)
        │
        v
  useClasp.emit(address, payload)
        │
        ├── Payload: { from, fromId, msgId, text: ciphertext, iv, encrypted: true, ... }
        └── CLASP PUBLISH message -> binary frame -> WebSocket
        │
        v
  CLASP Relay
        │
        ├── Scope check: write:/chat/room/*/messages (allowed)
        ├── Rate limit: 30 msg/sec per session
        └── Fan out to all subscribers of /chat/room/{roomId}/messages
        │
        v
  Other clients receive PUBLISH
        │
        v
  useChat.handleIncomingMessage()
        │
        ├── Detect encrypted: true
        ├── useCrypto.decrypt(roomId, ciphertext, iv)
        │   └── AES-GCM decrypt with room key
        └── Display plaintext in MessageList
```

### Room Creation

```
User fills RoomCreateDialog
        │
        v
  useRooms.createRoom({ name, type, isPublic, encrypted, ... })
        │
        ├── Generate roomId (crypto.randomUUID)
        ├── set(/chat/registry/rooms/{roomId}, roomData)    [if public]
        ├── set(/chat/registry/ns/{ns}/{roomId}, roomData)  [if in namespace]
        ├── set(/chat/room/{roomId}/meta, { name, type, encrypted, ... })
        ├── Add to local rooms Map
        └── Auto-join (add to joinedRoomIds, persist to localStorage)
        │
        v
  If encrypted:
        │
        v
  useCrypto.enableEncryption(roomId)
        │
        ├── generateRoomKey() -> AES-256-GCM CryptoKey
        ├── Export to JWK, persist to IndexedDB
        └── publishPublicKey() -> set ECDH pubkey to CLASP state
```

## Client-Side Storage

| Store | Technology | Contents |
|-------|-----------|----------|
| localStorage | Web Storage | Auth token, userId, display name, avatar color, status, joined rooms, pinned namespaces, relay URL |
| IndexedDB `clasp-chat` | IndexedDB | `messages` store (roomId-indexed message cache), `crypto-keys` store (roomId -> AES JWK) |
| In-memory Maps | JavaScript | Active room keys (`roomKeys`), peer ECDH public keys (`peerPublicKeys`), encrypted room set, password rooms, participant lists, typing indicators |

## Room Types

| Type | Constant | Description |
|------|----------|-------------|
| Text | `text` | Text-only chat room with message history |
| Video | `video` | WebRTC video/audio room with mesh topology |
| Combo | `combo` | Combined text chat + video grid |
| DM | `dm` | Direct message (two users, deterministic room ID, no registry listing) |

### Video Features

Video rooms (`video` and `combo` types) support multiple layout modes, active speaker detection, and peer pinning.

#### Layout Modes

| Mode | Description |
|------|-------------|
| Grid | Auto-fit responsive grid. All peers shown at equal size, tiles reflow based on container dimensions. |
| Spotlight | Large main tile + horizontal strip of thumbnails. The main tile shows the pinned or active speaker. |
| Sidebar | Large main tile + vertical strip of thumbnails. Same priority as spotlight, vertical layout. |

The layout is controlled by `useVideoLayout`, which exposes `setLayout(mode)`. Screen sharing automatically switches to spotlight and restores the previous layout when sharing stops.

#### Active Speaker Detection

`useAudioLevel` uses the Web Audio API `AnalyserNode` to detect speech:

- Samples audio at ~15fps (`fftSize: 256`, `getByteFrequencyData`)
- Speaking threshold: average frequency bin value > 15
- Debounce: 300ms timeout before marking a peer as no longer speaking
- `useVideoRoom` creates an `useAudioLevel` instance for each peer's audio track (local and remote)
- Speaking peer IDs are collected in a reactive `speakingPeerIds` Set (`'__local__'` for local user)

#### Peer Pinning

- Click or double-tap a video tile to pin that peer as the spotlight focus
- Pinned peer overrides automatic active speaker selection
- Click again to unpin and return to auto-speaker mode

#### Swipe Navigation

In spotlight mode on mobile, horizontal swipe on the main tile cycles through peers.

#### Resizable Combo Split

Combo rooms (`ComboChannelView`) render video and chat in a split layout with a draggable resize handle. The split is clamped between 20% and 80% of the container.

### DM Flow

DMs require an existing friendship. The "Message" button only appears on friend profiles.

```
User A clicks "Message" on friend B's profile
        │
        v
  useRooms.createDM(targetUserId, targetName)
        │
        ├── Generate deterministic roomId: dm-{first8(A)}-{first8(B)}
        │   (sorted so both users generate the same ID)
        ├── If room already exists in rooms Map → return existing roomId
        ├── set(/chat/room/{roomId}/meta, { name, type:DM, dmUsers })
        ├── set(/chat/user/{B}/dms/{roomId}, { fromId, fromName, roomId, timestamp })
        │   ↑ DM inbox notification (requires write:/chat/user/*/dms/* scope)
        ├── Add to joinedRoomIds + persist to localStorage
        └── Return roomId → ChatPage switches to it
        │
        v
  User B's client (subscribeDMInbox)
        │
        ├── Subscription: /chat/user/{B}/dms/*
        ├── handleIncomingDM(data, address)
        │   ├── Extract roomId from address
        │   ├── Skip if already in joinedRoomIds AND rooms Map
        │   ├── Add room to local rooms Map
        │   ├── Add to joinedRoomIds + persist
        │   └── Async fetchRoomMeta(roomId) for full dmUsers data
        └── DM appears in B's sidebar
```

On page refresh, `joinedRoomIds` is restored from localStorage but `rooms` is empty. The DM inbox subscription replays cached entries from the CLASP snapshot, repopulating the rooms Map. The early-return guard checks both `joinedRoomIds` and `rooms` to handle this reconnect case.

## Namespace System

Namespaces provide hierarchical room organization (like Discord servers or Slack workspaces):

```
/chat/registry/ns/{namespace}/{roomId}         Room listing within namespace
/chat/registry/ns/{namespace}/{child}/{roomId} Nested namespace
/chat/registry/ns-meta/{namespace}             Namespace metadata
/chat/registry/ns-meta/{namespace}/__auth      Namespace password protection
```

Features:
- Nested hierarchy (e.g., `gaming/minecraft/survival`)
- Public/private visibility
- Optional password protection (PBKDF2-SHA256 hashed)
- Path sanitization (prevents traversal and wildcard injection)
- Pin namespaces to sidebar (persisted in localStorage)
