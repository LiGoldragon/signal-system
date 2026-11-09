# Upgrades

## 4.1.0 → 4.1.1: generator ethos-zero 22.0.0

- Build-dependency ethos-zero ccacc21 (22.0.0), which brings its own datom-codec 5cd6219
  (0.33.1) into the lock as a build dependency only. ethos-zero 5a5daca (19.2.0) stays
  in the lock as the build dependency of signal and signal-persona.
- `ethos/signal.ethos` writes each variant named for the type it carries bare
  (`X.X` becomes `X`), which 22.0.0 requires. The generated Rust is byte-identical;
  the Rust API is unchanged.

## 4.1.0 — repin to the 0.33 cohort

- Pins: meta-signal-system dff620c (4.1.0, candidate branch protos033-923d14-meta-signal-system,
  pending main, which pins the signal-persona candidate 0f2b3e7), signal 7afb68b (10.3.0),
  datom-codec eabead4 (0.33.0), protos 3c7ade3 (0.33.0), and build-dependency ethos-zero
  5a5daca (19.2.0). One revision of each in the lock.
- Rust API only: `src/generated/signal.rs` is regenerated with ethos-zero 19.2.0, so the
  aliases NiriWindowId, ObservationGeneration and Focused are tuple structs. A consumer wraps a
  value with the constructor and reads it as `.0`. A test compares the archived bytes of all
  three with those of the wrapped types.
- `ReaderBudget` fields are private: build one with `ReaderBudget::try_from(n).unwrap()`.
- The contract holds no `Decimal`.

## 4.0.0 — repin to the current producer front, adopt the exchange layer

Repinned to `protos` 0.31.0 (rev `1febca7836bf8d5f5973a302fdd50aa9c84c159b`),
`datom-codec` 0.31.0 (rev `09e2a9d52bf7f2f11e51d15cb6c3177f72c6c927`),
`ethos-zero` 10.0.0 (rev `4bf73cae8d4f5a2072c76a11cd2f00aa3fe9f8e3`,
build-dependency only), `signal` 7.0.0 (rev
`66e7b153706696c2cfbb2abcf931a5e83aec91af`), and `meta-signal-system` (this
closure's own new revision — see its changelog). `src/generated/signal.rs`
was regenerated with the `ethos-zero` 10.0.0 CLI: every generated type gains
`Eq, Hash` in its derive list (`datom-codec` 0.31.0 gave the public data
types a `Hash`); no field, no kind and no trait name changed.

`signal` declares `links = "signal"`, which admits exactly one package per
`links` key in a graph, so `signal` and every one of its consumers in this
closure had to move together in this one pass — a stale `signal` pin
anywhere in the graph is a hard cargo resolution failure, not deferred debt.

`datom-codec` 0.31.0 removed every datom kind from `f64`; this contract
holds no `f64` and no `Decimal` anywhere in its ethos source, so nothing
here needed conversion.

`datom-codec`'s dependency list now includes `rkyv` (this crate's own
build-dependency on `ethos-zero` enables `datom-codec/rkyv`, unified into
the one resolved `datom-codec` package for this platform).

`signal` 7.0.0 adds the exchange layer (contract handshake by source
digest). This crate now carries the one-line
`impl signal::Contracted for Query { const CONTRACT_SOURCE: &'static str = ETHOS; }`
in `src/lib.rs` that the new handshake needs, per
`flows/f6db8d/reports/exchange-protocol.md` §4 and §9.

## 3.0.0 — portable Signal frame un-forked, current producer front

`signal-system` no longer declares its own `Signal<T>` frame or its
`Signalizable`, `ByteViewable`, `Restorable` traits. It now re-exports all
four from the new `signal` 5.0.0 crate, which owns the one portable rkyv
Signal frame shared across the estate — a second local copy was a different
Rust type forking the wire. Every consumer that named
`signal_system::Signal` (or the three traits) as a type still gets the same
paths through the re-export, but the concrete type identity changed: a
`Signal<T>` built against the old local definition is not the same type as
one built against `signal` 5.0.0, so anything that pinned an older
`signal-system` alongside a fresh one, or cached a build artifact, must
rebuild against 3.0.0 uniformly.

Repinned to `datom-codec` 0.27.0, `ethos-zero` 9.0.0 (build-dependency
only), `meta-signal-system` 3.0.0, and added `signal` 5.0.0 as a direct
dependency (`default-features = false`; enabled by this crate's own `datom`
feature via `signal/datom`). `protos` is unchanged at 0.30.1
(rev `171b21f65337983ab624b7b906397a4f1f92c5a3`).

`datom-codec` 0.27.0 renames the kind a datom composes into from
`Compositional` to `Composing` (ethos-zero 9.0.0 emits
`datom_codec::Composing` where 8.0.1 emitted `datom_codec::Compositional`).
`src/generated/signal.rs` is regenerated against the new ethos-zero output;
any consumer binding `datom_codec::Compositional` on a type from this crate
must move to `datom_codec::Composing`. No compatibility path is kept for the
old derive name.

Consumers: bump to `signal-system` 3.0.0, and if you also depend on `signal`
directly, pin it to
`{ git = "https://github.com/LiGoldragon/signal", rev = "7bcb0949cdaa4501651ed519625eba48e283fac0" }`
(no `.git` suffix) so cargo resolves a single copy — `signal` declares
`links = "signal"`, so a differing URL spelling for the same crate fails at
resolution rather than silently duplicating it.
