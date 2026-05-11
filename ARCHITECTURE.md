# ARCHITECTURE — signal-persona-system

The Signal contract between `persona-system` (producer of
OS facts) and `persona-router` (consumer — uses focus +
input-buffer state to gate message delivery). The whole
channel is one `signal_channel!` invocation in `src/lib.rs`.
It relates one router subscription client to the system observer:
the router names observation targets and the system mints
observation generations.

## Channel

| Side | Component |
|---|---|
| Sender (event side) | `persona-system` |
| Receiver (request side) | `persona-router` |

The router initiates subscriptions via `SystemRequest`;
`persona-system` accepts and pushes `SystemEvent` events as
focus + input-buffer state changes. The channel is
**bidirectional** but the steady-state flow is system →
router (push events).

Per `~/primary/skills/push-not-pull.md`, this channel IS
the push substrate. The router never polls; it subscribes
once per target then waits for events.

## Record source

This contract defines its records locally
(`SystemTarget`, `NiriWindowId`, `FocusObservation`,
`InputBufferObservation`, `ObservationGeneration`, etc.) because they're the
channel's vocabulary, not records that travel beyond.

If a future channel needs `SystemTarget` (e.g. a harness-discovery channel),
make or update the relation-specific `signal-persona-*` contract for that
relation. Do not lift system observation payloads into `signal-persona`; that
crate is the top-level engine-manager contract.

## Messages

```
SystemRequest                    SystemEvent
├─ FocusSubscription             ├─ FocusObservation
├─ FocusUnsubscription           ├─ InputBufferObservation
├─ FocusSnapshot                 ├─ WindowClosed
├─ InputBufferSubscription       ├─ SubscriptionAccepted
├─ InputBufferUnsubscription     └─ ObservationTargetMissing
└─ InputBufferSnapshot
```

Closed enums; no `Unknown` variant on the wire (the
`InputBufferState::Unknown` variant is a domain value
meaning "system can't tell," not a wire-level
"forward-compatible new variant").

## Versioning

`signal_core::Frame` carries the protocol version.
Schema-level changes (adding a new subscription kind or
event variant) are breaking; coordinate `persona-system` +
`persona-router` upgrades.

## Examples

```text
;; router → system: subscribe to focus events for Niri window 223
SystemRequest::FocusSubscription(FocusSubscription {
    target: SystemTarget::niri_window(223),
})

;; system → router: subscription accepted
SystemEvent::SubscriptionAccepted(SubscriptionAccepted {
    target: SystemTarget::niri_window(223),
    kind: SubscriptionKind::Focus,
})

;; system → router: focus changed (this Pi window now focused by user)
SystemEvent::FocusObservation(FocusObservation {
    target: SystemTarget::niri_window(223),
    focused: true,
    generation: ObservationGeneration::new(12),
})

;; system → router: input buffer is now non-empty (user typing)
SystemEvent::InputBufferObservation(InputBufferObservation {
    target: SystemTarget::niri_window(223),
    state: InputBufferState::Occupied,
    generation: ObservationGeneration::new(13),
})
```

## Round trips

14 round-trip tests in `tests/round_trip.rs` cover all 6
request variants, all 5 event variants, every
`InputBufferState`, both `SubscriptionKind` values, and
representative `From` impl witnesses.

The `ObservationGeneration` field on focus + input-buffer observations
is the monotonic counter the system mints; the router uses
it to discard stale events when subscriptions race.

Architectural-truth tests fire when:
- A new variant is added without a round-trip test.
- The Frame's encode/decode bytes don't match.
- A consumer tries to dispatch on a variant that isn't in
  the closed enum.

## Non-ownership

- No Niri adapter — that's `persona-system`.
- No focus-tracker actor — that's `persona-system`.
- No router gate logic — that's `persona-router`.
- No transport (UDS path, reconnect, timeouts).
- No subscription accounting — that's `persona-system`'s
  actor.

## Code map

```
src/
└── lib.rs    — payloads + signal_channel! invocation
tests/
└── round_trip.rs — per-variant wire-form round trips
```

## See also

- `~/primary/reports/designer/72-harmonized-implementation-plan.md`
  §2.1 — channel inventory
- `~/primary/reports/designer/73-signal-derive-research.md`
  — the `signal_channel!` macro decision
- `~/primary/reports/operator/67-signal-actor-messaging-gap-audit.md`
  — the safety property that drives this channel's design
- `~/primary/reports/operator/54-niri-focus-source-vision.md`
  — operator's earlier vision for the focus-source side
- `signal-core/src/channel.rs` — the macro
- `signal-persona-message/ARCHITECTURE.md` — companion
  channel that the router consumes alongside this one
