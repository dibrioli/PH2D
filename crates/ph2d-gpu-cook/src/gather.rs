//! The **identity** questions a dispatch asks — split from `lib.rs` at the HR-18
//! LOC cap, and a coherent seam rather than a slice: which port keys an id
//! gather, whether a column is readable at this dispatch length, and how long
//! the prior state was. All three exist because ADR-0130 decoupled a state
//! port's length from the dispatch's.

use crate::stream::GpuStream;
use ph2d_nodegraph::gpu::ColumnBinding;

/// The base port of an ACTIVE id-gather (ADR-0130) for this input set, or `None`
/// when there is none: the kernel declares a [`ColumnBinding`] whose access
/// [`is_gather_key`](ph2d_nodegraph::gpu::ColumnAccess::is_gather_key) AND that
/// port's stream carries the key column at the dispatch length (a dense window —
/// the plan already refused a non-dense one). The state ports (any other) are
/// then paired by id rather than by position, so their length is decoupled from
/// the dispatch.
pub(crate) fn gather_key_port(
    bindings: &[ColumnBinding],
    inputs: &[GpuStream],
    count: u32,
) -> Option<usize> {
    let key = bindings.iter().find(|b| b.access.is_gather_key())?;
    let active = inputs
        .get(key.port)
        .is_some_and(|s| s.count == count && s.cols.contains_key(key.column));
    active.then_some(key.port)
}

/// Is `b`'s column readable off its port for a `count`-element dispatch? A
/// column is "present" only if its port carries it AND is a length this dispatch
/// can PAIR with:
///
/// - **Positional** (the default): the port must be the DISPATCH length. A state
///   whose count no longer matches the live set was rebuilt, so pair nothing and
///   re-seed — the CPU's `pairing` `_ if sn == n` arm, expressed once.
/// - **Under an active id-gather** (`gather_port`): the STATE ports (any but the
///   key's) are paired by id, not position, so `prev_n ≠ n` is NORMAL. Presence
///   is simply "carries the column" (any non-empty length) — the length-decouple
///   ADR-0130 D4 names. The gather's OWN base-port columns stay dispatch-length.
/// - **Broadcast** ([`ph2d_nodegraph::gpu::ColumnAccess::ReadBroadcast`]): the
///   dispatch length OR
///   exactly one, which element `i` then reads row `0` of. A length-1 port judged
///   ABSENT here would read its identity, which is the difference between a flock
///   facing the point the artist animated and a flock facing the origin.
pub(crate) fn column_present(
    gather_port: Option<usize>,
    count: u32,
    inputs: &[GpuStream],
    b: &ColumnBinding,
) -> bool {
    match inputs.get(b.port) {
        None => false,
        Some(s) if gather_port.is_some_and(|kp| b.port != kp) => {
            s.count > 0 && s.cols.contains_key(b.column)
        }
        Some(s) if b.access.broadcasts() => {
            (s.count == count || s.count == 1) && s.cols.contains_key(b.column)
        }
        Some(s) => s.count == count && s.cols.contains_key(b.column),
    }
}

/// The prior state's element count (`prev_n`) an active gather reads through the
/// `gather_prev_n` uniform: the length of the STATE port (the first input that is
/// not the gather's base). `0` at tick 0 (the `pre` is Empty), where nothing
/// pairs and every element seeds.
pub(crate) fn gather_prev_n(inputs: &[GpuStream], key_port: usize) -> u32 {
    inputs
        .iter()
        .enumerate()
        .find(|(p, _)| *p != key_port)
        .map(|(_, s)| s.count)
        .unwrap_or(0)
}
