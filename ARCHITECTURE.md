# signal-system — architecture

*The Signal contract between `system` (producer of OS facts) and `router`
(consumer of focus observations).*

## 0 · TL;DR

`signal-system` carries one bidirectional channel between the router (request
side, opens subscriptions) and the system observer (reply/event side, emits
focus observations). The router subscribes once per target and the system pushes
events; the router never polls.

`schema/lib.schema` is the source of truth. `build.rs` runs the TrueSchema
`schema-rust` contract build and freshness-checks `src/schema/lib.rs`; setting
`SIGNAL_SYSTEM_UPDATE_SCHEMA_ARTIFACTS=1` intentionally refreshes the generated
Rust artifact. Handwritten Rust in `src/lib.rs` only re-exports the generated
nouns and adds behavior/convenience methods on those nouns.

## Three-layer model

**Layer 1 — Contract operations on the wire (this crate).** The contract-local
operation heads are `WatchFocus`, `UnwatchFocus`, `QueryFocus`, and
`QueryStatus`. Payload names stay domain nouns (`FocusSubscription`,
`FocusSubscriptionToken`, `FocusSnapshot`, `SystemStatusQuery`).

**Layer 2 — Component commands (system daemon).** The system daemon owns its
typed command enum (for example `OpenFocusSubscription`,
`CloseFocusSubscription`, `ReadFocusSnapshot`, `ReadSystemStatus`) plus command
execution.

**Layer 3 — Sema classification.** Each daemon-side component command projects
to a payloadless Sema class label via `ToSemaOperation`; the wire form never
uses Sema words as request roots.

**Frame layer.** The schema-derived contract emits `SystemFrame` as a
`signal_frame::StreamingFrame<SystemRequest, SystemReply, SystemEvent>` with
length-prefixed rkyv archives and optional `nota-text` witnesses.

## 1 · Channel

| Side | Component |
|---|---|
| Request side | `router` |
| Reply / event side | `system` |

The router initiates subscriptions via `SystemRequest`; the system answers
direct requests with `SystemReply` and pushes `SystemEvent` values on the
`FocusEventStream` when focus state changes. The steady-state flow is system →
router (push events on an open stream).

## 2 · Wire vocabulary

Records local to this contract: `SystemTarget`, `NiriWindowId`,
`ObservationGeneration`, `FocusSubscription`, `FocusSubscriptionToken`,
`FocusSnapshot`, `SystemStatusQuery`, `SystemBackend`, `FocusObservation`,
`WindowClosed`, `SubscriptionAccepted`, `SubscriptionKind`,
`ObservationTargetMissing`, `SystemStatus`, `SystemHealth`, `SystemReadiness`,
`SubscriptionRetracted`, `SystemRequestUnimplemented`,
`SystemUnimplementedReason`, `SystemOperationKind`, and
`SystemDaemonConfiguration`.

Small role wrappers such as `Target`, `Backend`, `Generation`, and
`SystemSocketPath` are schema-emitted field roles. They keep positional schema
fields dimensionally typed while `src/lib.rs` exposes convenience constructors
such as `FocusSubscription::from_target` and `SystemDaemonConfigurationParts`.

## 3 · Messages

```text
SystemRequest                            SystemReply
├─ WatchFocus(FocusSubscription)         ├─ SubscriptionAccepted
├─ UnwatchFocus(FocusSubscriptionToken)  ├─ SubscriptionRetracted(token)
├─ QueryFocus(FocusSnapshot)             ├─ ObservationTargetMissing
└─ QueryStatus(SystemStatusQuery)        ├─ SystemStatus
                                         ├─ SystemRequestUnimplemented
                                         └─ QueryFocusReply

SystemEvent on FocusEventStream
├─ FocusObservation
└─ WindowClosed
```

The closing exchange follows the Path A discipline: a request-side
`UnwatchFocus` carries the per-stream `FocusSubscriptionToken`, and the
reply-side `SubscriptionRetracted` echoes that token as the final acknowledgement.

## 4 · Sema-class projections

```text
WatchFocus (FocusSubscription)          -> Subscribe   (opens FocusEventStream)
UnwatchFocus (FocusSubscriptionToken)   -> Retract     (closes FocusEventStream)
QueryFocus (FocusSnapshot)              -> Match
QueryStatus (SystemStatusQuery)         -> Match
```

`SystemStatusQuery` and `SystemStatus` are the daemon-skeleton readiness surface.
A valid request whose runtime behavior is not built yet returns
`SystemReply::SystemRequestUnimplemented` carrying typed
`SystemUnimplementedReason`.

## 5 · Closed-enum integrity

```text
SystemTarget
  | NiriWindow(NiriWindowId)

SystemBackend
  | Niri

SystemHealth
  | Running
  | Degraded
  | Stopped

SystemReadiness
  | Ready
  | Starting
  | Unavailable

SubscriptionKind
  | Focus

SystemUnimplementedReason
  | NotBuiltYet
  | BackendUnavailable

SystemOperationKind
  | WatchFocus
  | UnwatchFocus
  | QueryFocus
  | QueryStatus
```

The contract has no `Unknown` variant on any wire enum. Future backends add
variants through a coordinated schema upgrade.

## 6 · Constraints

| Constraint | Witness |
|---|---|
| Schema is the contract source of truth. | `build.rs` uses `schema_rust::build::ContractCrateBuild` with `SIGNAL_SYSTEM_UPDATE_SCHEMA_ARTIFACTS`; a normal build fails if `src/schema/lib.rs` is stale. |
| Subscription close uses Path A — request-side `UnwatchFocus` plus reply-side `SubscriptionRetracted`. | `schema/lib.schema` declares `WatchFocus ... opens FocusEventStream`, `UnwatchFocus FocusSubscriptionToken`, and `FocusEventStream`; tests cover request, event, retract, and ack round trips. |
| Wire enums contain no `Unknown` variant. | Closed enum declarations in `schema/lib.schema`; tests exercise every current variant. |
| Round-trip witnesses cover rkyv frames and NOTA text. | `tests/round_trip.rs`, `tests/canonical_examples.rs`, and `examples/canonical.nota`. |
| Runtime code stays out of the contract. | Source scan: no Kameo, Tokio, socket, or redb runtime ownership in this crate. |
| Persona/system startup consumes a binary configuration contract. | `SystemDaemonConfiguration` is schema-derived and round-trips through rkyv; handwritten code only provides `SystemDaemonConfigurationParts` and archive helpers. |

## 7 · Versioning

`signal_frame::Frame` carries the protocol versioning boundary. This TrueSchema
port changes the generated Rust and frame surface, so the crate version is
`0.2.0`. Schema-level changes remain breaking and require coordinated upgrades
of `system`, `router`, and Persona runtime consumers.

## 8 · Non-ownership

- No Niri adapter — that is `system`.
- No focus-tracker actor — that is `system`.
- No terminal prompt-gate logic — that is `terminal` / `terminal-cell`.
- No transport (UDS path, reconnect, timeouts).
- No subscription accounting — that is `system`'s actor.
- No runtime implementation of status handling — the contract owns only typed
  records.

## 9 · Code map

```text
schema/
└── lib.schema            — TrueSchema source of the wire vocabulary
src/
├── lib.rs                — public re-exports + methods on generated nouns
└── schema/
    ├── mod.rs
    └── lib.rs            — schema-rust generated artifact
examples/
└── canonical.nota        — canonical text examples
tests/
├── canonical_examples.rs — examples parser/encoder witness
└── round_trip.rs         — frame, stream, NOTA, configuration witnesses
```

## See also

- `signal-message/ARCHITECTURE.md` and `signal-terminal/ARCHITECTURE.md` —
  sibling schema-derived Signal contracts.
- `schema/ARCHITECTURE.md` — TrueSchema and schema-rust source/generation model.
- `signal-frame` — streaming frame envelope kernel.
