//! Canonical examples round-trip witness.
//!
//! Parses `examples/canonical.nota` end-to-end, decoding each record
//! as a `SystemRequest`, `SystemReply`, or `SystemEvent` and
//! asserting the re-encoded text equals the canonical form.

use nota_codec::{Decoder, Encoder, NotaDecode, NotaEncode};
use signal_system::{
    FocusObservation, FocusSnapshot, FocusSubscription, FocusSubscriptionToken,
    ObservationGeneration, ObservationTargetMissing, SubscriptionAccepted, SubscriptionKind,
    SubscriptionRetracted, SystemBackend, SystemEvent, SystemHealth, SystemOperationKind,
    SystemReadiness, SystemReply, SystemRequest, SystemRequestUnimplemented, SystemStatus,
    SystemStatusQuery, SystemTarget, SystemUnimplementedReason, WindowClosed,
};

const CANONICAL: &str = include_str!("../examples/canonical.nota");

fn target() -> SystemTarget {
    SystemTarget::niri_window(223)
}

fn token() -> FocusSubscriptionToken {
    FocusSubscriptionToken { target: target() }
}

#[test]
fn canonical_request_examples_round_trip() {
    let expected: Vec<(SystemRequest, &str)> = vec![
        (
            SystemRequest::WatchFocus(FocusSubscription { target: target() }),
            "(WatchFocus ((NiriWindow 223)))",
        ),
        (
            SystemRequest::UnwatchFocus(token()),
            "(UnwatchFocus ((NiriWindow 223)))",
        ),
        (
            SystemRequest::QueryFocus(FocusSnapshot { target: target() }),
            "(QueryFocus ((NiriWindow 223)))",
        ),
        (
            SystemRequest::QueryStatus(SystemStatusQuery {
                backend: SystemBackend::Niri,
            }),
            "(QueryStatus (Niri))",
        ),
    ];

    for (value, canonical_text) in expected {
        let mut encoder = Encoder::new();
        value.encode(&mut encoder).expect("encode");
        let text = encoder.into_string();
        assert_eq!(text, canonical_text, "encode for {value:?}");

        let mut decoder = Decoder::new(canonical_text);
        let decoded = SystemRequest::decode(&mut decoder).expect("decode");
        assert_eq!(decoded, value, "decode for {canonical_text}");

        assert!(
            CANONICAL.contains(canonical_text),
            "examples/canonical.nota missing line: {canonical_text}",
        );
    }
}

#[test]
fn canonical_reply_examples_round_trip() {
    let expected: Vec<(SystemReply, &str)> = vec![
        (
            SystemReply::SubscriptionAccepted(SubscriptionAccepted {
                target: target(),
                kind: SubscriptionKind::Focus,
            }),
            "(SubscriptionAccepted ((NiriWindow 223) Focus))",
        ),
        (
            SystemReply::SubscriptionRetracted(SubscriptionRetracted { token: token() }),
            "(SubscriptionRetracted (((NiriWindow 223))))",
        ),
        (
            SystemReply::ObservationTargetMissing(ObservationTargetMissing {
                target: SystemTarget::niri_window(999),
            }),
            "(ObservationTargetMissing ((NiriWindow 999)))",
        ),
        (
            SystemReply::SystemStatus(SystemStatus {
                backend: SystemBackend::Niri,
                health: SystemHealth::Running,
                readiness: SystemReadiness::Ready,
            }),
            "(SystemStatus (Niri Running Ready))",
        ),
        (
            SystemReply::SystemRequestUnimplemented(SystemRequestUnimplemented {
                operation: SystemOperationKind::QueryFocus,
                reason: SystemUnimplementedReason::NotBuiltYet,
            }),
            "(SystemRequestUnimplemented (QueryFocus NotBuiltYet))",
        ),
        (
            SystemReply::QueryFocusReply(FocusObservation {
                target: target(),
                focused: true,
                generation: ObservationGeneration::new(12),
            }),
            "(QueryFocusReply ((NiriWindow 223) True 12))",
        ),
    ];

    for (value, canonical_text) in expected {
        let mut encoder = Encoder::new();
        value.encode(&mut encoder).expect("encode");
        let text = encoder.into_string();
        assert_eq!(text, canonical_text, "encode for {value:?}");

        let mut decoder = Decoder::new(canonical_text);
        let decoded = SystemReply::decode(&mut decoder).expect("decode");
        assert_eq!(decoded, value, "decode for {canonical_text}");

        assert!(
            CANONICAL.contains(canonical_text),
            "examples/canonical.nota missing line: {canonical_text}",
        );
    }
}

#[test]
fn canonical_event_examples_round_trip() {
    let expected: Vec<(SystemEvent, &str)> = vec![
        (
            SystemEvent::FocusObservation(FocusObservation {
                target: target(),
                focused: true,
                generation: ObservationGeneration::new(12),
            }),
            "(FocusObservation ((NiriWindow 223) True 12))",
        ),
        (
            SystemEvent::WindowClosed(WindowClosed { target: target() }),
            "(WindowClosed ((NiriWindow 223)))",
        ),
    ];

    for (value, canonical_text) in expected {
        let mut encoder = Encoder::new();
        value.encode(&mut encoder).expect("encode");
        let text = encoder.into_string();
        assert_eq!(text, canonical_text, "encode for {value:?}");

        let mut decoder = Decoder::new(canonical_text);
        let decoded = SystemEvent::decode(&mut decoder).expect("decode");
        assert_eq!(decoded, value, "decode for {canonical_text}");

        assert!(
            CANONICAL.contains(canonical_text),
            "examples/canonical.nota missing line: {canonical_text}",
        );
    }
}
