---
title: "Token Validation Flow"
description: "How CLASP validates authentication tokens using the ValidatorChain dispatch mechanism, supporting multiple token types with a single unified pipeline."
section: explanation
order: 14
---
# Token Validation Flow

How CLASP validates authentication tokens using the ValidatorChain dispatch mechanism, supporting multiple token types with a single unified pipeline.

## ValidatorChain Dispatch

CLASP uses a chain-of-responsibility pattern to validate tokens. Each validator checks whether a token belongs to it (by prefix) and either validates it or passes it along:

```mermaid
flowchart TD
    Token[Incoming Token String] --> Chain[ValidatorChain]

    Chain --> V1[CpskValidator]
    V1 -->|"starts with cpsk_"| CPSK_Validate[Validate CPSK Token]
    V1 -->|NotMyToken| V2[CapabilityValidator]

    V2 -->|"starts with cap_"| CAP_Validate[Validate Capability Token]
    V2 -->|NotMyToken| V3[EntityValidator]

    V3 -->|"starts with ent_"| ENT_Validate[Validate Entity Token]
    V3 -->|NotMyToken| NoMatch[Invalid: No validator matched]

    CPSK_Validate --> Result{Result}
    CAP_Validate --> Result
    ENT_Validate --> Result

    Result -->|Valid| Session[Create Session with Scopes]
    Result -->|Expired| Reject1[Reject: Token expired]
    Result -->|Invalid| Reject2[Reject: Validation failed]
```

The ValidatorChain iterates through registered validators. Each returns one of:

| Result | Meaning |
|--------|---------|
| `Valid(TokenInfo)` | Token is valid, includes scopes and metadata |
| `NotMyToken` | Token prefix not recognized, try next validator |
| `Expired` | Token was recognized but has expired |
| `Invalid(reason)` | Token was recognized but failed validation |

## CPSK Token Validation

Pre-shared key tokens (`cpsk_`) are the simplest token type, used for the register/login flow:

```mermaid
flowchart TD
    T["cpsk_..."] --> Prefix{"Starts with cpsk_?"}
    Prefix -->|No| NMT[NotMyToken]
    Prefix -->|Yes| Lookup["Look up in token store"]
    Lookup --> Found{"Token exists?"}
    Found -->|No| Invalid["Invalid: unknown token"]
    Found -->|Yes| Expiry{"Expired?"}
    Expiry -->|Yes| Expired["Expired"]
    Expiry -->|No| Valid["Valid(scopes from store)"]
```

CPSK tokens carry scopes assigned at registration time (e.g., `admin:/**`, `write:/lights/**`).

## Capability Token Validation

Capability tokens (`cap_`) are Ed25519-signed tokens with delegation chains:

```mermaid
flowchart TD
    T["cap_..."] --> Prefix{"Starts with cap_?"}
    Prefix -->|No| NMT[NotMyToken]
    Prefix -->|Yes| Decode["Decode base64url + msgpack"]
    Decode -->|Error| Invalid1["Invalid: encoding error"]
    Decode -->|OK| Expiry{"expires_at < now?"}
    Expiry -->|Yes| Expired["Expired"]
    Expiry -->|No| Depth{"chain_depth <= max_depth?"}
    Depth -->|No| Invalid2["Invalid: chain too deep"]
    Depth -->|Yes| Sig["Verify Ed25519 signature"]
    Sig -->|Fail| Invalid3["Invalid: bad signature"]
    Sig -->|OK| Anchor{"Root issuer in trust_anchors?"}
    Anchor -->|No| Invalid4["Invalid: untrusted issuer"]
    Anchor -->|Yes| Attenuation["Verify attenuation chain:<br/>each child scope subset of parent"]
    Attenuation -->|Fail| Invalid5["Invalid: attenuation violation"]
    Attenuation -->|OK| Valid["Valid(scopes, chain_depth in metadata)"]
```

The root issuer is determined from the proof chain: if proofs exist, the root is `proofs[0].issuer`; for root tokens (no proofs), it is `token.issuer`.

## Entity Token Validation

Entity tokens (`ent_`) are Ed25519-signed identity tokens backed by the entity registry:

```mermaid
flowchart TD
    T["ent_..."] --> Prefix{"Starts with ent_?"}
    Prefix -->|No| NMT[NotMyToken]
    Prefix -->|Yes| Parse["Parse base64url + msgpack"]
    Parse -->|Error| Invalid1["Invalid: parse error"]
    Parse -->|OK| Age{"Token age within max_token_age?<br/>(if configured)"}
    Age -->|No| Expired["Expired"]
    Age -->|Yes| Lookup["Look up entity by ID in store"]
    Lookup --> Found{"Entity exists?"}
    Found -->|No| Invalid2["Invalid: entity not found"]
    Found -->|Yes| Status{"Entity status == Active?"}
    Status -->|No| Invalid3["Invalid: entity not active"]
    Status -->|Yes| Verify["Verify Ed25519 signature<br/>against entity's public key"]
    Verify -->|Fail| Invalid4["Invalid: signature mismatch"]
    Verify -->|OK| Scopes["Resolve scopes:<br/>1. entity.scopes (if non-empty)<br/>2. entity.namespaces -> admin:/**"]
    Scopes --> Valid["Valid(scopes, entity_type in metadata)"]
```

## Scope Conversion

When an entity has no explicit scopes but has namespace patterns, they are converted:

| Entity Namespace | Resulting Scope |
|------------------|----------------|
| `/lights` | `admin:/lights/**` |
| `/lights/` | `admin:/lights/**` |
| `/lights/**` | `admin:/lights/**` |

This gives entities full admin access within their declared namespaces.

## Session Creation

After successful validation, the router creates a session with the token's scopes:

```mermaid
flowchart LR
    Valid["Valid(TokenInfo)"] --> Extract["Extract scopes:<br/>action:pattern pairs"]
    Extract --> Session["Create Session"]
    Session --> Enforce["Scope enforcement on<br/>every SET/SUBSCRIBE/PUBLISH"]
```

Scope enforcement checks:
- **SET**: Requires `write:` or `admin:` scope matching the address
- **SUBSCRIBE**: Requires `read:` or `admin:` scope matching the pattern
- **PUBLISH**: Requires `write:` or `admin:` scope matching the address

## See Also

- [Capability Delegation](capability-delegation.md) -- Delegation chains and scope attenuation
- [Security Model](security-model.md) -- Encryption, tokens, and threat model
- [Distributed Architecture](distributed-architecture.md) -- Overall system architecture
