# CLASP Chat

[CLASP Chat](apps/chat/) is a production chat application built entirely on top of CLASP. It demonstrates what happens when you push the protocol to its limits: real-time chat with E2E encryption, video calling, namespaces, and a plugin system running on a **generic CLASP relay that has zero knowledge of chat**.

## No Chat Server

Traditional chat apps require a dedicated server with REST endpoints, message databases, and chat-specific logic. CLASP Chat has none of that. The relay is a pub/sub router. Every chat concept is expressed as a CLASP address:

```
/chat/room/{roomId}/messages          -> EMIT (ephemeral messages)
/chat/room/{roomId}/presence/{userId} -> SET  (who's online, persisted)
/chat/room/{roomId}/typing/{userId}   -> SET  (typing indicators, auto-expire)
/chat/room/{roomId}/meta              -> SET  (room name, type, settings)
/chat/registry/rooms/{roomId}         -> SET  (public room discovery)
/chat/user/{userId}/profile           -> SET  (display name, avatar)
/chat/user/{userId}/friends/{friendId}-> SET  (friend list)
/chat/requests/{targetId}             -> EMIT (friend request handshake)
```

Messages are `EMIT` (fire-and-forget events), while presence and metadata are `SET` (persisted state that late-joiners receive automatically). Clients subscribe with wildcards (`/chat/room/*/presence/*`) and receive updates instantly. No polling, no REST calls.

## End-to-End Encryption

Encryption is layered on top of the same relay paths. When a room is encrypted:

1. Each member publishes an ECDH public key to `/chat/room/{roomId}/crypto/pubkey/{userId}`
2. Existing members derive a shared secret and send the AES-256-GCM room key via `/chat/room/{roomId}/crypto/keyex/{peerId}`
3. All messages are encrypted client-side before emit. The relay only sees ciphertext.
4. Messages are signed with ECDSA (P-256) for authenticity verification
5. Banning a user triggers key rotation. The banned peer's cached key is pruned and a new room key is distributed to remaining members.

## Video Calling via CLASP Signaling

WebRTC video calls use the same relay as a signaling channel:

```
/chat/room/{roomId}/video/presence/{sessionId} -> SET  (who's in the call)
/chat/room/{roomId}/video/signal/{recipientId} -> EMIT (offer/answer/ICE)
```

The relay carries only tiny signaling messages. Actual audio/video streams go peer-to-peer via WebRTC. No media server required.

## Namespaces

Rooms are organized into hierarchical namespaces (similar to Discord servers), stored as nested CLASP paths:

```
/chat/registry/ns/gaming/minecraft/{roomId}     -> room in gaming/minecraft
/chat/registry/ns-meta/gaming/minecraft         -> namespace metadata
/chat/registry/ns-meta/gaming/minecraft/__auth   -> password gate (hidden from wildcards)
```

Wildcard subscriptions (`/chat/registry/ns/gaming/**`) let clients discover all rooms in a namespace tree with a single subscribe.

## Why This Matters

The relay never needs to be updated for new chat features. Admin controls, friend requests, typing indicators, key rotation: all client-side logic over generic pub/sub paths. Scaling the chat means scaling the relay, which knows nothing about chat. Any CLASP relay can serve any CLASP Chat instance out of the box.
