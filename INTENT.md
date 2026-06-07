# INTENT — signal-system

*The ordinary peer-callable wire contract between `system` (producer of
OS facts) and the router (consumer of focus observations). Defines the
typed subscribe/event/snapshot/status channel that lets the router watch
window-focus changes without polling. Companion to `ARCHITECTURE.md` and
`Cargo.toml`. Maintenance: `primary/skills/repo-intent.md`.*

## Repo-scope only

This file carries only the intent that is FOR this `signal-system`
contract. Workspace-shape intent stays in the primary workspace
`primary/INTENT.md`. Component daemon intent stays in `system/INTENT.md`.

## Why this repo exists

`signal-system` is the **ordinary peer-callable wire contract** for the
`system` observer daemon. It carries one bidirectional channel: the
router opens focus subscriptions and reads focus/status snapshots; the
system pushes focus observation events. The router subscribes once per
target and waits for events — this contract IS the push substrate, so
the router never polls. Runtime actors, the Niri adapter, sockets, and
subscription accounting live in `system`.

## The channel shape

The system channel carries:

- **Requests:** `Watch` (open a focus subscription), `Unwatch` (retract
  it), `Query` (read a focus snapshot or system status), plus the
  mandatory `Tap`/`Untap` standardized observer hook persona components
  carry.
- **Replies:** `SubscriptionAccepted`, `SubscriptionRetracted` (echoing
  the token), `SystemStatus`, `ObservationTargetMissing`,
  `QueryFocusReply`, and `SystemRequestUnimplemented` for
  skeleton-honest unbuilt behavior.
- **Events** (on `FocusEventStream`): `FocusObservation` and
  `WindowClosed`, pushed as focus state changes.

The wire vocabulary is contract-local — the daemon lowers these public
operations into component-local commands; Sema classification happens at
observation publish time, not on the wire.

## Channels are closed, boundaries are named

- Wire enums are closed. No `Unknown` escape hatch; absence is named
  positively (`ObservationTargetMissing`).
- Subscription close uses **Path A**: a request-side `Unwatch` carrying
  the per-stream token, plus a reply-side `SubscriptionRetracted` ack
  echoing that token. The token in the retract request matches the token
  in the ack.
- Request payloads do not mint observation generations, timestamps, or
  sequence numbers; `system` mints those at the daemon.
- No stringly-typed dispatch. Target, backend, health, readiness, and
  reason fields are typed closed enums.

## Wire vocabulary discipline

Per `primary/skills/contract-repo.md` §"Public contracts use
contract-local operation verbs":

- Operation roots are domain verbs in verb form (`Watch`, `Unwatch`,
  `Query`), not the Sema class words `Subscribe`/`Match`. The six Sema
  classification words must not appear as request roots on this wire.
- Reply success variants name the concrete outcome the daemon produced.
- Payload record names drop redundant `System*`/`Focus*` prefixes where
  the crate namespace already supplies them.

## Three-layer model

Layer 1 (this crate): contract operations on the wire (`Watch`,
`Unwatch`, `Query`).
Layer 2 (daemon): component-local `SystemCommand` records
(`OpenFocusSubscription`, `CloseFocusSubscription`, `ReadFocusSnapshot`,
`ReadSystemStatus`).
Layer 3 (observation): payloadless Sema class labels (`Subscribe`,
`Retract`, `Match`) computed daemon-side for cross-component
introspection.

The contract names the public action; the daemon decides internal work
and Sema class. Sema classification never appears on the wire.

## Constraints

- This crate carries only typed wire vocabulary, NOTA codecs, and
  round-trip witnesses.
- No runtime code: no actors, no tokio, no socket binding, no redb, no
  Niri adapter or focus-tracker logic.
- Contract types derive NOTA in this crate. Clients do not carry shadow
  types that re-derive the text surface.
- Every request, reply, and event variant round-trips through both rkyv
  frames and NOTA text; the full subscribe/event/retract/ack lifecycle
  is witnessed.
- A valid request whose runtime behavior is not built yet returns the
  typed `SystemRequestUnimplemented`, never a text error or a hang.
- Wire dependency pins use named branches or tags, not raw revision
  hashes.

## Non-ownership

This crate does not own:

- `system` daemon runtime, the Niri adapter, or the focus-tracker actor;
- subscription accounting or the open-stream state;
- terminal prompt-gate logic (that is `terminal` / `terminal-cell`);
- socket binding, transport, reconnect, or version handshake policy;
- runtime status handling — only the typed records.

## See also

- `ARCHITECTURE.md` — detailed channel shape, the Path A subscription
  lifecycle, closed-enum discipline, and the three-layer migration.
- `../system/INTENT.md` — daemon-side intent (OS observation, focus
  tracking, schema-driven planes).
- `primary/skills/push-not-pull.md` — Subscribe, not poll; the substrate
  this channel realizes.
- `primary/skills/contract-repo.md` — contract repo discipline and
  naming rules.
- `primary/skills/component-triad.md` — repo triad structure and wire
  layers.
