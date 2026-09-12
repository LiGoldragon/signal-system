# signal-system

The Signal contract between **`system`** (producer of OS facts) and
**`router`** (consumer of focus observations).

The contract source is `ethos/signal.ethos`. `ethos-zero` generates
`src/generated/signal.rs` from it; `build.rs` asserts the committed
generation matches a fresh one, so the ethos file is the single authority
for the wire. Read `src/lib.rs` for the frame surface and the three wire
enums the ethos declares: `Query`, `Response`, and `SystemStreamEvent`.

## Quick reference

```rust
use meta_signal_system::SystemBackend;
use signal_system::{
    ByteViewable, FocusSubscription, Query, Signalizable, SystemStatusQuery, SystemTarget,
};

// Router subscribes to focus events for a Niri window.
let query = Query::WatchFocus(FocusSubscription {
    system_target: SystemTarget::NiriWindow(223),
});
let frame = query.signalize().expect("query archives");
let _bytes = frame.bytes(); // send to system's UDS

// Router asks the system boundary for its readiness.
let _status = Query::QueryStatus(SystemStatusQuery {
    system_backend: SystemBackend::Niri,
});
```

The system answers `WatchFocus` with `Response::SubscriptionAccepted` and
then pushes `SystemStreamEvent::FocusObservation` frames whenever focus
changes for the subscribed target.

The public operation heads are contract-local: `WatchFocus`,
`UnwatchFocus`, `QueryFocus`, and `QueryStatus`.

`SystemBackend` and the daemon's typed startup record
(`SystemDaemonConfiguration`) belong to the privileged plane and live in
`meta-signal-system`. This contract imports `SystemBackend` from there; it
does not declare its own.

Prompt cleanliness, input gates, and programmatic write safety are terminal
transport facts. They live in `signal-terminal`, not in this contract.

## See also

- `ARCHITECTURE.md` — channel role + boundaries
- `ethos/signal.ethos` — the contract source
- `examples/canonical.datom` — one canonical Datom line per contract head
- `meta-signal-system` — the privileged configuration plane for `system`
