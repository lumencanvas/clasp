---
title: "Addressing"
description: "Documentation page for the CLASP protocol."
section: api
order: 1
---
## CLASP Addressing

CLASP uses hierarchical, UTF‑8 string addresses to identify signals:

```text
/namespace/category/instance/property
```

Examples:

- `/lumen/scene/0/layer/3/opacity`
- `/midi/launchpad/cc/74`
- `/dmx/1/47`

### Wildcards (Subscriptions Only)

Wildcards make it easy to subscribe to families of signals:

- `*` matches **exactly one** path segment.
- `**` matches **zero or more** segments.

Examples:

- `/lumen/scene/*/layer/**/opacity`
- `/controller/*`

Clients MAY use wildcards in subscribe patterns, but **addresses in SET/GET/PUBLISH messages must be concrete** (no wildcards).

See the main protocol specification for full matching rules; language‑specific APIs provide helpers for building and validating addresses.

