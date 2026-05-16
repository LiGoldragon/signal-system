# ARCHITECTURE — signal-persona-system

The Signal contract between `persona-system` (producer of
OS facts) and `persona-router` (consumer of focus observations). The whole
channel is one `signal_channel!` invocation in `src/lib.rs`.
It relates one router subscription client to the system observer:
the router names observation targets and the system mints
observation generations.

> Status: persona-system is paused (per `persona-system/ARCHITECTURE.md`
> §0.7). This contract holds the Path A reply-side close shape; the
> system unpauses with a real consumer reading
> `SystemReply::SubscriptionRetracted` to terminate its in-flight
> `FocusSubscription`.

## Channel

| Side | Component |
|---|---|
| Sender (event side) | `persona-system` |
| Receiver (request side) | `persona-router` |

The router initiates subscriptions via `SystemRequest`;
`persona-system` answers direct requests with `SystemReply` and
pushes `SystemEvent` events as focus state changes. The channel is
**bidirectional** but the steady-state flow is system →
router (push events).

Per `~/primary/skills/push-not-pull.md`, this channel IS
the push substrate. The router never polls; it subscribes
once per target then waits for events.

## Record source

This contract defines its records locally
(`SystemTarget`, `NiriWindowId`, `FocusObservation`,
`ObservationGeneration`, etc.) because they're the
channel's vocabulary, not records that travel beyond.

If a future channel needs `SystemTarget` (e.g. a harness-discovery channel),
make or update the relation-specific `signal-persona-*` contract for that
relation. Do not lift system observation payloads into `signal-persona`; that
crate is the top-level engine-manager contract.

## Messages

```
SystemRequest                            SystemReply
├─ FocusSubscription                     ├─ SubscriptionAccepted
├─ FocusSubscriptionRetraction(token)    ├─ SubscriptionRetracted(token)
├─ FocusSnapshot                         ├─ ObservationTargetMissing
└─ SystemStatusQuery                     ├─ SystemStatus
                                         ├─ SystemRequestUnimplemented
                                         └─ FocusSnapshotReply

SystemEvent
├─ FocusObservation
└─ WindowClosed
```

Subscription close follows the **Path A** discipline per /181: the
router sends `Retract FocusSubscriptionRetraction(FocusSubscriptionToken)`
naming the subscription it wants to close; the system responds with
`SystemReply::SubscriptionRetracted(SubscriptionRetracted { token })`
carrying the same token. `FocusSubscriptionToken` is the per-stream
identity — structurally `{ target: SystemTarget }`, the same shape as
`FocusSubscription`'s payload, but a distinct type so subscribe / close
sites don't conflate "open this stream" with "name the stream to close."

Closed enums; no `Unknown` variant on the wire (the
target-missing event is an explicit typed fact, not a wire-level
"forward-compatible new variant").

### Signal root verbs

Every `SystemRequest` variant declares its root verb in the
`signal_channel!` declaration. `signal-core` generates
`SystemRequest::signal_verb()` and `SystemRequest::into_request()`
from that declaration.

```text
FocusSubscription             -> Subscribe
FocusSubscriptionRetraction   -> Retract
FocusSnapshot                 -> Match
SystemStatusQuery             -> Match
```

Subscriptions establish a push stream. Retractions close that stream
and the system acks with `SystemReply::SubscriptionRetracted` carrying
the token (Path A). One-shot observations and status reads use `Match`,
not `Assert`.

Prompt cleanliness, typed write leases, and programmatic write-injection
acknowledgements are terminal transport records. They live in
`signal-persona-terminal` and are enforced by `persona-terminal` /
`terminal-cell`, not by this system observation contract.

`SystemStatusQuery` and `SystemStatus` are the daemon-skeleton
readiness surface for the component itself. A valid request whose
runtime behavior is not built yet returns
`SystemReply::SystemRequestUnimplemented`; it is a typed reply, not a text error
or a hang.

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
SystemReply::SubscriptionAccepted(SubscriptionAccepted {
    target: SystemTarget::niri_window(223),
    kind: SubscriptionKind::Focus,
})

;; system → router: focus changed (this Pi window now focused by user)
SystemEvent::FocusObservation(FocusObservation {
    target: SystemTarget::niri_window(223),
    focused: true,
    generation: ObservationGeneration::new(12),
})

```

## Round trips

Round-trip tests in `tests/round_trip.rs` cover all request variants, all
event variants, `SubscriptionKind`, and representative `From` impl witnesses.
NOTA text witnesses cover every request and event variant. `SystemTarget` has a
manual NOTA codec so the text form preserves the target head, for example
`(NiriWindow 223)`.
Request frame tests assert each variant's `signal_verb()` mapping.

The `ObservationGeneration` field on focus observations is the monotonic
counter the system mints; the router uses
it to discard stale events when subscriptions race.

Architectural-truth tests fire when:
- A new variant is added without a round-trip test.
- The Frame's encode/decode bytes don't match.
- A consumer tries to dispatch on a variant that isn't in
  the closed enum.

## Constraints

- Subscription close uses **Path A** reply-side variant. The
  `FocusSubscription` request opens the focus-observation stream;
  the `FocusSubscriptionRetraction` retract request (carrying a
  `FocusSubscriptionToken`) closes it; the system acks with
  `SystemReply::SubscriptionRetracted` echoing the token.
- Wire enums are closed; no `Unknown` variants travel on the wire.
- Every `SystemRequest` variant declares its root verb in the
  `signal_channel!` declaration.
- One-shot reads (`FocusSnapshot`, `SystemStatusQuery`) use the
  `Match` verb; only `FocusSubscription` uses `Subscribe`.

## Non-ownership

- No Niri adapter — that's `persona-system`.
- No focus-tracker actor — that's `persona-system`.
- No terminal prompt gate logic — that's `persona-terminal` / `terminal-cell`.
- No transport (UDS path, reconnect, timeouts).
- No subscription accounting — that's `persona-system`'s
  actor.
- No runtime implementation of status handling — the contract owns
  only the typed records.

## Code map

```
src/
└── lib.rs    — payloads + signal_channel! invocation
tests/
└── round_trip.rs — per-variant frame round trips + NOTA text witnesses
```

## See also

- `signal-core/src/channel.rs` — the macro
- `signal-persona-message/ARCHITECTURE.md` — companion
  channel that the router consumes alongside this one
