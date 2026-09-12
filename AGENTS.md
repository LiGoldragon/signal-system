# signal-system — Agent Instructions

## Purpose

`signal-system` is the ordinary Signal contract for the system → router
focus-observation channel. It is a wire contract crate, not a runtime
component.

## Local Rules

- Use Jujutsu for version control.
- Keep runtime code out of this crate: no actors, sockets, tokio tasks, store
  handles, or filesystem mutation.
- The contract lives in `ethos/signal.ethos`. Change it there and regenerate
  `src/generated/signal.rs` with `ethos-zero`; never hand-edit generated Rust
  and never hand-write a `Datomic` implementation for a declared type.
- Every contract change needs a frame round-trip witness in `tests/contract.rs`
  and a canonical line in `examples/canonical.datom`.
- `SystemBackend` and the daemon's typed startup record are owned by
  `meta-signal-system`; import them rather than duplicating them.
- Pin every LiGoldragon dependency by immutable `rev`, never by branch.
