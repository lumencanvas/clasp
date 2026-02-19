# State & Lifecycle Diagrams

All diagrams use [Mermaid](https://mermaid.js.org/) syntax.

## Authentication Flow

```mermaid
flowchart TD
    A[User opens app] --> B{Has token in localStorage?}
    B -->|Yes| C[Navigate to /chat]
    B -->|No| D[Navigate to /auth]

    D --> E{User choice}
    E -->|Register| F[POST /auth/register]
    E -->|Login| G[POST /auth/login]
    E -->|Guest| H[POST /auth/guest]

    F -->|Rate limit check| F1{IP blocked?}
    F1 -->|Yes| F2[429 Too Many Requests]
    F1 -->|No| F3{Username taken?}
    F3 -->|Yes| F4[409 Conflict]
    F3 -->|No| F5[Argon2 hash password]
    F5 --> F6[Insert into SQLite]
    F6 --> F7[Generate CPSK token with scopes]
    F7 --> I

    G -->|Rate limit check| G1{IP or username blocked?}
    G1 -->|Yes| G2[429 Too Many Requests]
    G1 -->|No| G3[Lookup user in SQLite]
    G3 -->|Not found| G4[401 Unauthorized]
    G3 -->|Found| G5[Argon2 verify password]
    G5 -->|Wrong| G6[401 + record failed attempt]
    G5 -->|Correct| G7[Clear rate limit + generate token]
    G7 --> I

    H -->|Rate limit check| H1{IP blocked?}
    H1 -->|Yes| H2[429 Too Many Requests]
    H1 -->|No| H3{user_id provided?}
    H3 -->|Yes| H4{Valid format? Not taken?}
    H4 -->|Invalid| H5[400/409 Error]
    H4 -->|Valid| H6[Generate token with scopes]
    H3 -->|No| H6
    H6 --> I

    I[Store token + userId in localStorage] --> C
    C --> J[useClasp.connect with token]
    J --> K[WebSocket HELLO + token]
    K --> L[Relay validates token]
    L --> M[WELCOME + SNAPSHOT]
    M --> N[App ready]
```

## Room Lifecycle

```mermaid
stateDiagram-v2
    [*] --> Creating: User fills RoomCreateDialog

    Creating --> Active: createRoom()
    note right of Creating
        1. Generate roomId (UUID)
        2. Set room meta via CLASP
        3. Register in room/ns registry
        4. Auto-join locally
    end note

    Active --> Encrypted: enableEncryption(roomId)
    note right of Encrypted
        1. Generate AES-256-GCM key
        2. Persist to IndexedDB
        3. Publish ECDH pubkey
    end note

    Active --> Active: Members join/leave
    Encrypted --> Encrypted: Members join/leave

    Encrypted --> KeyRotation: Admin bans user
    note right of KeyRotation
        1. Remove banned peer's cached key
        2. Generate new AES room key
        3. Distribute to remaining peers
    end note
    KeyRotation --> Encrypted: New key active

    Active --> Deleted: Creator deletes room
    Encrypted --> Deleted: Creator deletes room
    note right of Deleted
        1. Set registry entry to null
        2. Remove from local state
        3. Leave room
    end note

    Deleted --> [*]
```

## E2E Key Exchange Sequence

```mermaid
sequenceDiagram
    participant A as Alice (key holder)
    participant R as CLASP Relay
    participant B as Bob (new member)

    Note over A: Has room key + ECDH key pair
    Note over B: Joins encrypted room, no key

    B->>R: subscribe(/room/R1/crypto/keyex/bob)
    B->>R: set(/room/R1/crypto/pubkey/bob, {publicKey})

    R->>A: UPDATE: /room/R1/crypto/pubkey/bob

    Note over A: Derive shared secret
    A->>A: ECDH(alice.private, bob.public)
    A->>A: HKDF-SHA256(shared, info="clasp-chat-keyex-v1")
    A->>A: AES-GCM encrypt room key with shared secret

    A->>R: emit(/room/R1/crypto/keyex/bob, {encryptedKey, iv, senderPublicKey})

    R->>B: EVENT: /room/R1/crypto/keyex/bob

    Note over B: Derive same shared secret
    B->>B: ECDH(bob.private, alice.public)
    B->>B: HKDF-SHA256(shared, info="clasp-chat-keyex-v1")
    B->>B: AES-GCM decrypt room key

    Note over B: Can now encrypt/decrypt messages
```

## Password-Gated Key Exchange

```mermaid
sequenceDiagram
    participant J as Joining User
    participant R as CLASP Relay
    participant H as Key Holder

    J->>J: PBKDF2(password, room salt) -> hash
    J->>R: set(/room/R1/crypto/proof/joiner, {hash, timestamp})
    J->>R: set(/room/R1/crypto/pubkey/joiner, {publicKey})

    R->>H: UPDATE: /room/R1/crypto/pubkey/joiner

    Note over H: Password room detected
    H->>R: subscribe(/room/R1/crypto/proof/joiner)
    R->>H: UPDATE: proof data

    alt proof.hash === expectedHash
        H->>H: Derive shared secret + encrypt room key
        H->>R: emit(/room/R1/crypto/keyex/joiner, {encryptedKey, ...})
        R->>J: EVENT: encrypted room key
        J->>J: Decrypt room key
    else proof.hash !== expectedHash OR timeout (2s)
        Note over H: Skip key exchange
        Note over J: Remains without key
    end
```

## Presence & Typing State Machine

```mermaid
stateDiagram-v2
    state "Presence" as P {
        [*] --> Offline
        Offline --> Online: joinChat() -> announcePresence()
        Online --> Online: Heartbeat every 10s
        Online --> Stale: No heartbeat for 25s
        Stale --> Offline: pruneStale()
        Online --> Offline: leaveChat() -> set(null)
    }

    state "Typing" as T {
        [*] --> NotTyping
        NotTyping --> Typing: handleTyping()
        note right of Typing
            set(/room/{rid}/typing/{uid}, {name, timestamp})
        end note
        Typing --> NotTyping: stopTyping() after 2s idle
        note right of NotTyping
            set(/room/{rid}/typing/{uid}, null)
        end note
        Typing --> NotTyping: sendMessage()
    }
```

### Presence Timing Constants

| Constant | Value | Purpose |
|----------|-------|---------|
| `PRESENCE_HEARTBEAT` | 10,000 ms | Interval between presence announcements |
| `PRESENCE_STALE` | 25,000 ms | Time before a participant is considered stale |
| `TYPING_TIMEOUT` | 2,000 ms | Idle time before typing indicator clears |
| `TYPING_EXPIRE` | 3,000 ms | Remote typing indicator auto-expiry |
| `DISCONNECT_GRACE` | 5,000 ms | Grace period before marking disconnected |

## Message Flow (Encrypted Room)

```mermaid
flowchart TD
    A[User types message] --> B{Rate limit check}
    B -->|Blocked| C[Show throttle warning]
    B -->|OK| D{Starts with /?}
    D -->|Yes| E[Route to plugin system]
    D -->|No| F{Waiting for key?}
    F -->|Yes| G[Show key-wait warning]
    F -->|No| H[Build message payload]

    H --> I{Room encrypted?}
    I -->|No| J[Emit plaintext payload]
    I -->|Yes| K[AES-GCM encrypt text]
    K --> L{Has image?}
    L -->|Yes| M[AES-GCM encrypt image separately]
    L -->|No| N[Emit encrypted payload]
    M --> N

    H --> O[Add optimistic local message]
    H --> P[Persist to IndexedDB]

    J --> Q[CLASP emit to /room/rid/messages]
    N --> Q

    Q --> R[Relay fans out to subscribers]
    R --> S[Other clients receive]
    S --> T{encrypted: true?}
    T -->|No| U[Display plaintext]
    T -->|Yes| V[AES-GCM decrypt]
    V --> W{Decrypt success?}
    W -->|Yes| U
    W -->|No| X[Show key-unavailable placeholder]
```

## WebSocket Connection State

```mermaid
stateDiagram-v2
    [*] --> Disconnected

    Disconnected --> Connecting: useClasp.connect()
    Connecting --> Connected: WELCOME received
    Connecting --> Disconnected: Connection error

    Connected --> Disconnected: onDisconnect
    Connected --> Reconnecting: Connection lost (auto-reconnect enabled)
    Reconnecting --> Connected: onReconnect
    Reconnecting --> Disconnected: Max retries exceeded

    Connected --> Disconnected: useClasp.disconnect()
```

## Room Join Flow (with Crypto)

```mermaid
flowchart TD
    A[joinChat called] --> B[Load cached messages from IndexedDB]
    B --> C[Subscribe to /room/rid/messages]
    C --> D[Subscribe to /room/rid/presence/*]
    D --> E[Subscribe to /room/rid/typing/*]
    E --> F[Load room key from IndexedDB]

    F --> G[Subscribe to /room/rid/meta]
    G --> H{meta.encrypted?}
    H -->|No| I[Announce presence + heartbeat]
    H -->|Yes| J{Have room key?}

    J -->|Yes| K[Subscribe key exchange]
    J -->|No| L[Set waitingForKey = true]
    L --> M[Subscribe key exchange]
    M --> N[Request room key - publish ECDH pubkey]
    N --> O[Start retry interval every 5s]

    K --> I
    O --> I

    I --> P[Room active]

    Q[Key exchange response received] --> R[Decrypt room key]
    R --> S[Store in-memory + IndexedDB]
    S --> T[waitingForKey = false]
```

## DM Lifecycle

```mermaid
sequenceDiagram
    participant A as User A (initiator)
    participant R as CLASP Relay
    participant B as User B (recipient)

    Note over A,B: Prerequisite: A and B are friends

    A->>A: createDM(B.id, B.name)
    Note over A: roomId = dm-{sort(A.id, B.id)}

    A->>R: set(/chat/room/{roomId}/meta, {type:DM, dmUsers})
    A->>R: set(/chat/user/{B.id}/dms/{roomId}, {fromId, fromName, timestamp})

    R->>B: UPDATE: /chat/user/{B.id}/dms/{roomId}

    Note over B: handleIncomingDM()
    B->>B: Add room to local state + joinedRoomIds
    B->>R: get(/chat/room/{roomId}/meta)
    R->>B: Room metadata (dmUsers map)

    Note over A,B: Both can now exchange messages at /chat/room/{roomId}/messages
```

### DM Reconnect (Page Refresh)

```mermaid
flowchart TD
    A[Page loads] --> B[joinedRoomIds restored from localStorage]
    B --> C[rooms Map is empty]
    C --> D[CLASP connects + receives snapshot]
    D --> E[subscribeDMInbox replays cached /dms/* entries]
    E --> F{joinedRoomIds.has AND rooms.has?}
    F -->|Both true| G[Skip - already synced]
    F -->|rooms missing| H[handleIncomingDM populates rooms Map]
    H --> I[fetchRoomMeta for full dmUsers data]
    I --> J[DM visible in sidebar]
```

## Namespace Tree

```mermaid
graph TD
    R["/chat/registry/ns"] --> G["gaming"]
    R --> M["music"]
    R --> D["dev"]

    G --> G1["gaming/minecraft"]
    G --> G2["gaming/factorio"]
    G1 --> G1R1["room: survival-server"]
    G1 --> G1R2["room: creative-builds"]
    G2 --> G2R1["room: megabase"]

    M --> MR1["room: listening-party"]
    M --> MR2["room: production-tips"]

    D --> D1["dev/rust"]
    D --> D2["dev/web"]
    D1 --> D1R1["room: async-questions"]
    D2 --> D2R1["room: vue-help"]

    style R fill:#1a1a2e,stroke:#457b9d
    style G fill:#1a1a2e,stroke:#2a9d8f
    style M fill:#1a1a2e,stroke:#9b5de5
    style D fill:#1a1a2e,stroke:#f77f00
```

Namespace metadata is stored at `/chat/registry/ns-meta/{path}`:
```json
{
  "description": "All things Minecraft",
  "isPublic": true,
  "createdBy": "u-1234-abc",
  "createdAt": 1708123456789
}
```

Password protection is stored separately at `/chat/registry/ns-meta/{path}/__auth`:
```json
{
  "passwordHash": "<PBKDF2 hash>",
  "passwordSalt": "<random salt>"
}
```
