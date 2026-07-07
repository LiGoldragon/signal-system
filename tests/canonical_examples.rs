//! Canonical examples round-trip witness.
//!
//! Parses `examples/canonical.nota` end-to-end, decoding each record as a
//! `SystemRequest`, `SystemReply`, or `SystemEvent` and asserting the re-encoded
//! text equals the canonical form.

#![cfg(feature = "nota-text")]

use nota::{NotaEncode, NotaSource};
use signal_system::{
    Backend, FocusObservation, FocusSnapshot, FocusSubscription, FocusSubscriptionToken, Health,
    Kind, ObservationTargetMissing, Operation, Readiness, Reason, SubscriptionAccepted,
    SubscriptionKind, SubscriptionRetracted, SystemBackend, SystemEvent, SystemHealth,
    SystemOperationKind, SystemReadiness, SystemReply, SystemRequest, SystemRequestUnimplemented,
    SystemStatus, SystemStatusQuery, SystemTarget, SystemUnimplementedReason, Target, WindowClosed,
};

const CANONICAL: &str = include_str!("../examples/canonical.nota");

fn target() -> SystemTarget {
    SystemTarget::niri_window(223)
}

fn token() -> FocusSubscriptionToken {
    FocusSubscriptionToken::from_target(target())
}

fn observation(generation: u64) -> FocusObservation {
    FocusObservation::new(target(), true, generation)
}

#[test]
fn canonical_request_examples_round_trip() {
    let expected: Vec<(SystemRequest, &str)> = vec![
        (
            SystemRequest::WatchFocus(FocusSubscription::from_target(target())),
            "(WatchFocus (NiriWindow 223))",
        ),
        (
            SystemRequest::UnwatchFocus(token()),
            "(UnwatchFocus (NiriWindow 223))",
        ),
        (
            SystemRequest::QueryFocus(FocusSnapshot::from_target(target())),
            "(QueryFocus (NiriWindow 223))",
        ),
        (
            SystemRequest::QueryStatus(SystemStatusQuery::from_backend(SystemBackend::Niri)),
            "(QueryStatus Niri)",
        ),
    ];

    for (value, canonical_text) in expected {
        let text = value.to_nota();
        assert_eq!(text, canonical_text, "encode for {value:?}");

        let decoded = NotaSource::new(canonical_text)
            .parse::<SystemRequest>()
            .expect("decode");
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
                target: Target::new(target()),
                kind: Kind::new(SubscriptionKind::Focus),
            }),
            "(SubscriptionAccepted ((NiriWindow 223) Focus))",
        ),
        (
            SystemReply::SubscriptionRetracted(SubscriptionRetracted::from_token(token())),
            "(SubscriptionRetracted (NiriWindow 223))",
        ),
        (
            SystemReply::ObservationTargetMissing(ObservationTargetMissing::from_target(
                SystemTarget::niri_window(999),
            )),
            "(ObservationTargetMissing (NiriWindow 999))",
        ),
        (
            SystemReply::SystemStatus(SystemStatus {
                backend: Backend::new(SystemBackend::Niri),
                health: Health::new(SystemHealth::Running),
                readiness: Readiness::new(SystemReadiness::Ready),
            }),
            "(SystemStatus (Niri Running Ready))",
        ),
        (
            SystemReply::SystemRequestUnimplemented(SystemRequestUnimplemented {
                operation: Operation::new(SystemOperationKind::QueryFocus),
                reason: Reason::new(SystemUnimplementedReason::NotBuiltYet),
            }),
            "(SystemRequestUnimplemented (QueryFocus NotBuiltYet))",
        ),
        (
            SystemReply::QueryFocusReply(observation(12)),
            "(QueryFocusReply ((NiriWindow 223) True 12))",
        ),
    ];

    for (value, canonical_text) in expected {
        let text = value.to_nota();
        assert_eq!(text, canonical_text, "encode for {value:?}");

        let decoded = NotaSource::new(canonical_text)
            .parse::<SystemReply>()
            .expect("decode");
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
            SystemEvent::FocusObservation(observation(12)),
            "(FocusObservation ((NiriWindow 223) True 12))",
        ),
        (
            SystemEvent::WindowClosed(WindowClosed::from_target(target())),
            "(WindowClosed (NiriWindow 223))",
        ),
    ];

    for (value, canonical_text) in expected {
        let text = value.to_nota();
        assert_eq!(text, canonical_text, "encode for {value:?}");

        let decoded = NotaSource::new(canonical_text)
            .parse::<SystemEvent>()
            .expect("decode");
        assert_eq!(decoded, value, "decode for {canonical_text}");

        assert!(
            CANONICAL.contains(canonical_text),
            "examples/canonical.nota missing line: {canonical_text}",
        );
    }
}
