# ADR-002: Generic `CommandExecutor<T>` Over Boxed Trait Objects

**Status:** Accepted

---

## Context

`CommandExecutor` is the top-level API that CLI and TUI binaries use. It must hold an
`IsoTpTransport` implementation. Two implementation strategies were considered:

**Option A — Generic:** `CommandExecutor<T: IsoTpTransport>` — monomorphized at compile
time. The concrete type is `CommandExecutor<IsoTp<VLinkerFs>>` in the CLI, and
`CommandExecutor<IsoTp<Box<dyn LinkLayer>>>` in the CLI adapter factory.

**Option B — Boxed:** `CommandExecutor { transport: Box<dyn IsoTpTransport> }` — dynamic
dispatch at runtime, single concrete `CommandExecutor` type everywhere.

---

## Decision

Use **Option A — generics**. `CommandExecutor<T: IsoTpTransport>` is generic over the
transport. The `faraday-cli` adapter factory (`commands::create_executor`) returns
`CommandExecutor<IsoTp<Box<dyn LinkLayer>>>` to allow runtime `LinkLayer` selection while
keeping the transport layer concrete.

The `LinkLayer` trait does have a blanket `Box<dyn LinkLayer>` implementation, enabling the
CLI to use dynamic dispatch at the link layer only (where the overhead is dwarfed by I/O
latency).

---

## Consequences

**Positive:**
- Zero runtime overhead in the hot path — no vtable dispatch between protocol, transport,
  and command layers.
- Compile-time verification that the concrete stack satisfies all trait bounds.
- `J1979<T>` and `Uds<T>` borrow `&mut T: IsoTpTransport` for their lifetime, which
  enforces single-use access without locks.

**Negative:**
- `faraday-tui` instantiates `CommandExecutor<IsoTp<VLinkerFs>>` with a concrete type —
  the TUI cannot swap adapters at runtime without a refactor or introducing a boxed variant.
- Each distinct `T` produces a separate monomorphized binary artifact, increasing binary
  size slightly (acceptable for a CLI/TUI tool).
- A blanket `impl IsoTpTransport for Box<dyn IsoTpTransport>` does not exist, so boxed
  transport cannot be passed where a generic bound is expected.
