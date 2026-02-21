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

---

## Summary

| Phase | Files | Status |
|-------|-------|--------|
| 1. New Crate READMEs | 5 | Done |
| 2. Existing README Updates | 3 | Done |
| 3. Architecture Diagrams | 7 | Done |
| 4. Pentest Plan | 1 | Done |
| **Total** | **16** | **Complete** |
