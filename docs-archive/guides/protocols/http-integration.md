---
title: "Http Integration"
description: "Documentation page for the CLASP protocol."
section: guides
order: 1
---
## HTTP Integration with CLASP

The HTTP bridge lets you expose CLASP signals as a REST API and/or drive CLASP from external HTTP clients.

### Server Mode: REST API → CLASP

In **server mode**, the bridge listens for HTTP requests and maps them to CLASP signals:

- **GET** `/api/signals/*path` → read a signal value.
- **PUT** `/api/signals/*path` → `SET` a Param value.
- **POST** `/api/signals/*path` → `PUBLISH` an Event‑style value.
- **DELETE** `/api/signals/*path` → clear a value (SET `null`).

By default, the bridge uses:

- Base path: `/api`
- Namespace prefix: `/http`
- JSON bodies and responses.

Example: a `PUT /api/signals/foo/bar` with body `{ "value": 0.75 }` maps to CLASP address `/foo/bar` with `Value::Float(0.75)`. The HTTP integration tests in `test-suite/src/bin/http_integration_tests.rs` exercise this behavior end‑to‑end.

### Client Mode: CLASP → HTTP

In **client mode**, the bridge makes outbound HTTP requests based on CLASP activity (polling, webhooks, etc.). This is useful for:

- Pushing state changes into existing REST backends.
- Polling third‑party APIs and reflecting results as CLASP signals.

See `crates/clasp-bridge/src/http.rs` for the full `HttpBridgeConfig` and server/client behaviors; tests and examples should be kept in sync with that file to ensure docs remain accurate.

