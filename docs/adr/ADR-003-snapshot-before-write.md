# ADR-003: Mandatory Snapshot Before Every Write Operation

**Status:** Accepted

---

## Context

Writing to an ECU's as-built configuration carries a non-trivial risk of bricking the
module if the wrong value is written or if a communication error occurs mid-write. A
bricked BCM or PCM in a Ford Fusion requires dealer reprogramming and can cost hundreds
of euros. A tool that makes writes easy must make rollback equally easy.

Several strategies were considered:

- **No snapshot** — simplest, but no recovery path.
- **Explicit snapshot command only** — `asbuilt snapshot` before `asbuilt write`; recoverable
  only if the user remembered to snapshot first.
- **Mandatory automatic snapshot** — the write command always reads and saves the current
  value before writing, regardless of user action.

---

## Decision

Implement **mandatory automatic snapshot** as the write path's first step. Before any CAN
frame encoding a write is transmitted:

1. `read_asbuilt_block()` fetches the current block data.
2. `save_snapshot()` persists a timestamped JSON `AsBuiltSnapshot` to
   `~/.local/share/faraday/snapshots/`.
3. Only after the snapshot is confirmed written does the write sequence begin.

The snapshot path is also the output of the explicit `faraday asbuilt snapshot` command,
so a user who wants a manual snapshot before a batch of writes still can.

Rollback is `faraday asbuilt restore <snapshot-file>`, which reads the snapshot JSON and
re-writes the block.

---

## Consequences

**Positive:**
- Any write that corrupts configuration is recoverable without dealer intervention.
- The mandatory nature means users cannot accidentally skip the safety step.
- Snapshots are auditable JSON — human-readable, diffable.

**Negative:**
- Every write incurs one extra read round-trip (read current block, save snapshot) before
  the actual write. On MS-CAN this adds ~200 ms.
- Disk I/O to `~/.local/share/faraday/snapshots/` on every write. Negligible in practice.
- Snapshots accumulate on disk; no automatic pruning today.
