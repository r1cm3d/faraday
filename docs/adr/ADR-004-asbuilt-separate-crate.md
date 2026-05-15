# ADR-004: `faraday-asbuilt` as an Independent Data Library

**Status:** Accepted

---

## Context

The as-built block catalog (which ECU DIDs exist, what bit positions mean what feature,
how to decode raw bytes into human-readable values) is pure, static knowledge. It does not
need a serial port or a CAN bus to function — a catalog lookup is just a data query.

Two placement options were considered:

**Option A — Inside `faraday-core`:** The catalog, decoder, and encoder live as modules
in the core library alongside the transport and protocol layers.

**Option B — Separate `faraday-asbuilt` crate:** An independent library with no dependency
on `faraday-core`, focused solely on catalog data, bit decoding, bit encoding, and JSON
snapshots.

---

## Decision

Implement **Option B — separate `faraday-asbuilt` crate**. The crate has zero dependency
on `faraday-core`. Its only external dependencies are `serde`, `serde_json`, `thiserror`,
and `hex`.

`faraday-cli` depends on both `faraday-core` and `faraday-asbuilt`.

---

## Consequences

**Positive:**
- The catalog can be used in tools or scripts that do not need a live CAN connection (e.g.,
  a future web interface that renders as-built feature names from a snapshot file).
- Compilation of `faraday-asbuilt` is fast (no tokio, no serialport, no async-trait).
- Tests for catalog correctness (block count, DID values, feature names) are isolated from
  transport-layer mock complexity.
- Encourages the catalog to stay data-focused and not accumulate transport logic.

**Negative:**
- `faraday-tui` currently does not depend on `faraday-asbuilt`, meaning the TUI cannot
  display decoded as-built configuration without adding the dependency. This is a gap in
  the TUI feature set.
- Two crates to understand instead of one — slight onboarding overhead.
- If `AsBuiltBlock` needs a field from `faraday-core::types` (e.g., `Module`), the
  separation forces duplicating the module enum or using string identifiers instead.
  Currently resolved by using `String` for module names in the catalog.
