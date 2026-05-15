# ADR-005: Inline `#[cfg(test)]` Modules Over Separate `tests/` Directory

**Status:** Accepted

---

## Context

Rust supports two test locations: inline `#[cfg(test)]` modules within source files, and
integration test files in a crate's `tests/` directory (compiled as separate crates with
access only to the public API).

For a project where most testable behaviour involves internal state (ISO-TP frame
construction, seed-key XOR computation, bit-level decoder/encoder) a decision was needed
on where tests live.

---

## Decision

Use **inline `#[cfg(test)]` modules** for all unit and quasi-integration tests across
`faraday-core`, `faraday-asbuilt`, and `faraday-emu`. No separate `tests/` directory
exists at the crate level.

The `Makefile` `test/integration` target (`cargo test --test '*' --all-features`) is
defined for future integration tests but produces no output today.

---

## Consequences

**Positive:**
- Tests live next to the code they test. Refactoring a private function and its test
  stays in the same file.
- Private internals (e.g., `PciType`, individual frame builders in `isotp.rs`) are
  directly accessible to tests without requiring `pub(crate)` promotion.
- No need for a separate test fixture crate to share mock types — mocks are defined in
  the same `#[cfg(test)]` block.

**Negative:**
- Source files are longer — `isotp.rs` and `seed_key.rs` carry their test code inline.
- Test-only dependencies (e.g., `proptest`, `tokio-test`) are declared as
  `[dev-dependencies]` but apply to the whole crate, not just specific test files.
- Integration-level tests (e.g., running the full `faraday` binary against `faraday-emu`)
  require end-to-end tooling (`make tui/emu`) rather than `cargo test`.
- `faraday-cli` has no tests today — the inline pattern requires discipline to maintain
  as new command modules are added.
