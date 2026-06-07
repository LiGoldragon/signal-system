# signal-system

The Signal contract between **`system`** (producer
of OS facts) and **`persona-router`** (consumer of focus
observations).

Read `src/lib.rs` for the public interface — three enums
(`SystemRequest`, `SystemReply`, `SystemEvent`) declared via the
`signal_channel!` macro. The variants ARE the messages
this channel carries.

## Quick reference

```rust
use signal_frame::{
    ExchangeIdentifier, ExchangeLane, LaneSequence, RequestPayload, SessionEpoch,
};
use signal_system::{
    FocusSubscription, SystemFrame, SystemFrameBody, SystemRequest, SystemTarget,
};

// Router subscribes to focus events for a Niri window
let exchange = ExchangeIdentifier::new(
    SessionEpoch::new(1),
    ExchangeLane::Connector,
    LaneSequence::first(),
);
let request = SystemRequest::WatchFocus(FocusSubscription {
    target: SystemTarget::niri_window(223),
});
let frame = SystemFrame::new(SystemFrameBody::Request {
    exchange,
    request: request.into_request(),
});
let bytes = frame.encode_length_prefixed()?;
// send to system's UDS
```

The system replies with `SystemReply::SubscriptionAccepted`
followed by `SystemEvent::FocusObservation` events whenever
focus changes for the subscribed target.

The public operation heads are contract-local:
`WatchFocus`, `UnwatchFocus`, `QueryFocus`, and `QueryStatus`.
Sema classification words such as `Subscribe`, `Retract`, and
`Match` are daemon-side observation labels, not wire roots.

Prompt cleanliness, input gates, and programmatic write safety are terminal
transport facts. They live in `signal-persona-terminal`, not in this system
contract.

## See also

- `ARCHITECTURE.md` — channel role + boundaries
- `~/primary/skills/contract-repo.md` — contract-repo discipline
- `signal-frame` — kernel that supplies `Frame`, `Request`,
  `Reply`, `signal_channel!`
