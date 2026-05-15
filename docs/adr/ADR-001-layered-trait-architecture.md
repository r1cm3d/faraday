# ADR-001: Layered Trait Architecture

**Status:** Accepted

---

## Context

`faraday` must communicate with a physical CAN bus adapter over serial, implement two
protocol stacks (J1979 and UDS), and reassemble multi-frame ISO-TP payloads. Hardware
availability is limited: the OBD-II adapter and the vehicle are not always present during
development. Testing every layer in isolation without hardware is a hard requirement.

The design must also accommodate future adapter types (Bluetooth via btleplug, SocketCAN
via the Linux kernel) without rewriting the protocol or command layers.

---

## Decision

Define an explicit trait boundary at each of the five architecture layers:

```
Commands   ─ CommandExecutor<T: IsoTpTransport>
Protocol   ─ J1979<T>, Uds<T>  (both borrow &mut T: IsoTpTransport)
Transport  ─ IsoTp<L: LinkLayer>  (implements IsoTpTransport)
Link       ─ VLinkerFs  (implements LinkLayer)
Physical   ─ tokio-serial (external crate)
```

`LinkLayer` exposes: `connect`, `disconnect`, `send_frame`, `receive_frame`, `set_can_bus`,
`is_connected`.

`IsoTpTransport` exposes: `send`, `receive`, `request_response`, `set_timeout`, `set_can_bus`.

Both traits are `Send + Sync` and use `async-trait` to support `async fn` methods.

---

## Consequences

**Positive:**
- Each layer is independently testable with a mock (e.g., `MockLinkLayer` injects frames
  without a serial port; `MockTransport` injects UDS responses without ISO-TP).
- New adapters (Bluetooth, SocketCAN) require only a new `LinkLayer` implementation — all
  layers above are unaffected.
- The compile-time trait bounds prevent protocol-layer code from accidentally calling
  transport internals.

**Negative:**
- Five distinct abstractions increase the surface area a new contributor must understand.
- `async-trait` adds a heap allocation per async call at the trait boundary (overhead is
  negligible for serial I/O but nonzero).
- Adding a sixth layer (e.g., multiplexing across two buses simultaneously) would require
  extending both traits or introducing a new one.
