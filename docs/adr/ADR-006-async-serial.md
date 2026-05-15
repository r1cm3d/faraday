# ADR-006: Async Serial I/O with `tokio-serial` and `async-trait`

**Status:** Accepted

---

## Context

OBD-II communication is inherently latency-bound: each request waits for a vehicle ECU to
respond (typically 50–500 ms). The TUI must remain responsive during these waits, and the
CLI must support future multiplexing (reading multiple PIDs or polling multiple modules).

Two I/O models were considered:

**Option A — Synchronous / blocking:** Use `serialport` crate directly with blocking reads.
Simpler, but blocks the thread during every wait.

**Option B — Async with tokio-serial:** Use `tokio-serial::SerialStream` with
`AsyncReadExt` / `AsyncWriteExt`, and `async-trait` for trait method signatures.

---

## Decision

Use **Option B — async with tokio-serial**. All `LinkLayer` and `IsoTpTransport` methods
are `async fn`. The tokio runtime is the exclusive executor (`#[tokio::main]` in both
binaries). `async-trait` provides the procedural macro that enables `async fn` in trait
definitions (a limitation lifted in Rust 1.75+ with return-position `impl Trait`, but
`async-trait` remains for maximum stable Rust compatibility).

---

## Consequences

**Positive:**
- The TUI event loop (`crossterm::event::poll`) and CAN frame polling run concurrently
  within the same tokio runtime without additional threads.
- Background tasks (TesterPresent keepalive) are `tokio::spawn`-ed `JoinHandle`s,
  cancellable on `Drop`.
- Future Bluetooth (btleplug) and SocketCAN transports are naturally async, so they will
  fit the existing trait surface.

**Negative:**
- `VLinkerFs::receive_frame()` currently uses `tokio::time::sleep(Duration::from_millis(10))`
  in a polling loop while waiting for bytes from the adapter. This is not truly reactive —
  it wakes up 100 times per second even when idle. A proper solution would use
  `AsyncReadExt::read()` with a future that resolves when bytes arrive, eliminating the
  sleep loop.
- `async-trait` adds a `Box<dyn Future>` allocation per async call at trait-object
  boundaries. This is negligible for serial I/O latency but is worth noting.
- The mandatory tokio runtime prevents embedding `faraday-core` in `no_std` or
  single-threaded environments without significant refactoring.
