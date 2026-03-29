---
title: Access Control (ACP)
description: Relationship-based access control for DefraDB config documents
order: 8
---

# Access Control (ACP)

When DefraDB is enabled, CLASP configurations (routers, connections, bridges, rules) are stored as documents that can be replicated across peers. Access control determines who can read, write, and delete these documents.

## How It Works

DefraDB uses a Zanzibar-style relationship-based access control system. Instead of traditional role-based permissions, access is determined by relationships between actors and documents:

- **Owner**: The person who created the document. Can read, write, and delete.
- **Operator**: A person explicitly granted access by the owner. Can read and write, but not delete.

Every config schema includes an `owner` field that records who created the document. When ACP is enabled, DefraDB enforces these relationships at the query/mutation level.

## Policy Structure

The CLASP ACP policy defines five resource types, one for each config schema:

```yaml
resources:
  router_config:
    permissions:
      read:
        expr: owner + operator
      write:
        expr: owner + operator
      delete:
        expr: owner
    relations:
      owner:
        types:
          - actor
      operator:
        types:
          - actor
```

The `+` operator means union: either an owner OR an operator can read/write. Only the owner can delete.

## Identity

DefraDB ACP uses secp256k1 keys for identity. This is a different curve from CLASP's native Ed25519 identity system. ACP-enabled deployments need a secp256k1 key for DefraDB operations.

Identity is passed with every GraphQL request as an `Authorization: bearer <token>` header. Without identity, documents are created as public (no access control).

## Enabling ACP

ACP is optional. Without it, all DefraDB documents are public and any connected peer can read/write them. This is fine for single-node and trusted-team deployments.

To enable ACP:

1. Start DefraDB with ACP enabled (this is the default in recent versions)
2. CLASP registers the ACP policy on startup
3. All mutations include the operator's identity
4. DefraDB enforces the policy automatically

## Sharing Configurations

Once ACP is enabled, you can share specific configs with other users:

```
Owner creates router config
  -> Document is private to owner

Owner grants "operator" relation to alice
  -> alice can now read and modify the router config

Owner revokes alice's access
  -> alice can no longer see the config
```

Public access can be granted by using `*` as the actor:

```
Owner grants "operator" relation to "*"
  -> Anyone can read and modify the config
```

## Limitations

- ACP requires secp256k1 identity (not Ed25519) due to DefraDB's identity model
- Local ACP mode prevents P2P replication on permissioned collections
- Secondary indexes, aggregations, and type joins are not supported with ACP
- Relationship management is per-document, not per-collection

## Without ACP

If ACP is not enabled (the default for most CLASP deployments), all documents are public within the DefraDB instance. Access control is handled at the network level (who can connect to the DefraDB instance) rather than the document level.

The existing SecurityPanel features (scopes, write rules, visibility rules) continue to work regardless of ACP status.
