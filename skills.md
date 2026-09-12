# skills — signal-system

Before editing this repo, read:

- the `ethos` skill — the contract is an ethos file, and the Rust is generated
- the `datom` skill — the text dialect the `datom` feature projects into
- `ARCHITECTURE.md`
- `meta-signal-system`'s `ethos/signal.ethos`, which owns `SystemBackend` and
  the daemon's typed startup record

This crate owns only the ordinary Signal contract for the system → router
focus-observation channel. Do not add runtime, storage, or CLI behavior here.

## Invariants

- The contract is changed by editing `ethos/signal.ethos` and regenerating with
  `ethos-zero`; never by editing `src/generated/signal.rs`.
- Regenerated output is committed in the same change; `build.rs` is the gate.
- `SystemBackend` is imported from `meta-signal-system`. The privileged plane
  owns backend selection and daemon configuration; a second declaration here
  would fork the wire.
- Subscription close uses both sides: the `UnwatchFocus` query carries the
  per-stream `FocusSubscriptionToken`, and `Response::SubscriptionRetracted`
  echoes it. Removing either breaks the Path A discipline consumers bind to.
- Wire enums are closed. No `Unknown` variant. Target absence is named
  positively (`ObservationTargetMissing`). New backends arrive through a
  coordinated schema bump of `SystemTarget` and, in the meta contract,
  `SystemBackend`.
- Every `Query`, `Response` and `SystemStreamEvent` variant needs a frame
  round-trip witness in `tests/contract.rs`.
- Every canonical line in `examples/canonical.datom` must actualize into a
  contract head; `every_canonical_datom_line_actualizes_into_a_contract_head`
  is the gate on that file. An ungated canonical file drifts silently.
- A bare ethos enum variant whose name matches a declared type generates a
  payload-carrying variant. Tag enums such as `SystemOperationKind` therefore
  use names that are not themselves declared types.
- The codec emits and reads guillemets `« »` for quoted strings. Build any
  canonical text from what the codec itself emits, never by hand.
- Pin every Git producer by immutable `rev`. Moving branches are not build
  inputs.
