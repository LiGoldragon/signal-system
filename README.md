# signal-system

The Signal contract between **`system`** (producer of OS facts) and
**`router`** (consumer of focus observations).

`schema/lib.schema` is the source of truth. `src/lib.rs` re-exports the
schema-derived public interface: `SystemRequest`, `SystemReply`,
`SystemEvent`, and the streaming `SystemFrame` aliases.

## Quick reference

```rust
use signal_frame::{ExchangeIdentifier, ExchangeLane, LaneSequence, SessionEpoch};
use signal_system::{FocusSubscription, SystemRequest, SystemTarget};

let exchange = ExchangeIdentifier::new(
    SessionEpoch::new(1),
    ExchangeLane::Connector,
    LaneSequence::first(),
);
let request = SystemRequest::WatchFocus(FocusSubscription::from_target(
    SystemTarget::niri_window(223),
));
let frame = request.into_frame(exchange);
let bytes = frame.encode_length_prefixed()?;
// send to system's UDS
```

The system replies with `SystemReply::SubscriptionAccepted` followed by
`SystemEvent::FocusObservation` events whenever focus changes for the subscribed
target.

The public operation heads are contract-local: `WatchFocus`, `UnwatchFocus`,
`QueryFocus`, and `QueryStatus`. Sema classification words such as `Subscribe`,
`Retract`, and `Match` are daemon-side observation labels, not wire roots.

Prompt cleanliness, input gates, and programmatic write safety are terminal
transport facts. They live in `signal-terminal`, not in this system contract.

## See also

- `ARCHITECTURE.md` — channel role + boundaries
- `schema/lib.schema` — schema-derived wire vocabulary
- `signal-frame` — frame, request, reply, and streaming envelope kernel
