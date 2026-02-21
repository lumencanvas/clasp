# Documentation Plan - Distributed Infrastructure

**Created:** 2026-02-21
**Branch:** `feat/distributed-infrastructure`
**Status:** Complete

---

## Phase 1: New Crate READMEs

- [x] `crates/clasp-caps/README.md` -- Capability tokens
- [x] `crates/clasp-registry/README.md` -- Entity registry
- [x] `crates/clasp-journal/README.md` -- Append-only journal
- [x] `crates/clasp-rules/README.md` -- Rules engine
- [x] `crates/clasp-federation/README.md` -- Federation

## Phase 2: Update Existing READMEs

- [x] `README.md` -- Main project README (distributed infra section, features, crates table)
- [x] `crates/clasp-router/README.md` -- Feature flags, federation/journal/rules sections, architecture diagram
- [x] `crates/clasp-cli/README.md` -- Key management, cap token, entity token commands

## Phase 3: Architecture and State Diagrams

- [x] `docs/explanation/distributed-architecture.md` -- Overall system diagram and message flow
- [x] `docs/explanation/federation-state-machine.md` -- PeerState transition diagram
- [x] `docs/explanation/federation-message-sequence.md` -- Hub/Leaf handshake sequence
- [x] `docs/explanation/token-validation-flow.md` -- ValidatorChain dispatch flowchart
- [x] `docs/explanation/capability-delegation.md` -- Delegation chain and attenuation rules
- [x] `docs/explanation/security-model.md` -- Updated to reflect actual token types (CPSK/Cap/Entity)
- [x] `docs/explanation/README.md` -- Added entries for new docs

## Phase 4: Pentest Plan

- [x] `docs/security/pentest-plan.md` -- 8 attack categories, 55 test cases

## Phase 5: Inline Security Documentation (16-Phase Plan)

- [x] Pentest cross-references in security-critical code paths (`See pentest <ID>` comments)
- [x] Accuracy verification pass (router README version bump 3.1->3.5, added `metrics` feature flag)

Files modified:
- `crates/clasp-caps/src/token.rs` -- CAP-02, CAP-03, CAP-04, CAP-10, PAT-04
- `crates/clasp-core/src/security.rs` -- TTL rationale, expiry semantics
- `crates/clasp-router/src/handlers/federation.rs` -- FED-02, FED-03, FED-05, FED-06, FED-07, FED-08
- `crates/clasp-router/src/handlers/set.rs` -- PAT-05, FED-01, FED-10
- `crates/clasp-router/src/handlers/publish.rs` -- PAT-01, FED-01, FED-10
- `crates/clasp-router/src/handlers/hello.rs` -- CAP-01, ENT-01, ENT-04, FED-09
- `crates/clasp-journal/src/sqlite.rs` -- JNL-01 (compute + verify), migration note
- `deploy/relay/src/cpsk.rs` -- ADM-06
- `deploy/relay/src/auth.rs` -- ADM-01, Argon2id rationale
- `deploy/relay/src/validator/write.rs` -- PAT-01..PAT-05 module-level ref
- `crates/clasp-router/README.md` -- version 3.5, added `metrics` feature flag

---

## Summary

| Phase | Files | Status |
|-------|-------|--------|
| 1. New Crate READMEs | 5 | Done |
| 2. Existing README Updates | 3 | Done |
| 3. Architecture Diagrams | 7 | Done |
| 4. Pentest Plan | 1 | Done |
| 5. Inline Security Docs | 11 | Done |
| **Total** | **27** | **Complete** |
