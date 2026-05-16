//! Generic action queue — replaces the `pending_X` proliferation in
//! `shells/desktop/src/main.rs` (PR 9). PR 1 lands the types only;
//! the host-side queue field + drain loop arrive with PR 9.
//!
//! Design constraint: HR-3 zero-alloc hot path. The queue itself is a
//! pre-allocated `VecDeque<ActionInvocation>` (capacity set at boot);
//! payloads live in a per-frame `bumpalo::Bump` arena. Enqueue does
//! not allocate on the heap once steady state is reached.
//!
//! See `docs/Migracao/2026-05-convention-by-discovery.md` §4.2 (fluxo
//! de dados) and Appendix F (`tool_actions.rs` shape).

use crate::manifest::ToolId;

/// One queued click / panel event to be drained next frame. Payload
/// is `&'static [u8]` *interpreted as a borrow into the per-frame
/// arena* — the producer of the invocation guarantees the bytes live
/// for the duration of the frame.
///
/// Safety / lifetime: this struct is `Copy` because it carries only
/// the borrow handle, not ownership. The arena is reset at frame end
/// (after the drain loop runs), so all live `ActionInvocation`s must
/// be drained before reset.
#[derive(Copy, Clone, Debug)]
pub struct ActionInvocation {
    /// Tool that owns this invocation. Looked up in the registry to
    /// find the handler.
    pub action_id: ToolId,

    /// Opaque payload bytes, decoded by the handler. Encoding is a
    /// per-tool contract (postcard is the recommended choice — HR-6
    /// asset format already uses it, MSRV-compatible, deterministic).
    /// Empty slice if the handler takes no parameters.
    pub payload: &'static [u8],
}

impl ActionInvocation {
    /// Construct a parameter-less invocation. Common case for one-shot
    /// actions (e.g. `[Make Square]` button → no payload, handler
    /// reads the current selection).
    pub const fn no_payload(action_id: ToolId) -> Self {
        Self {
            action_id,
            payload: &[],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_payload_invocation_is_const() {
        const INV: ActionInvocation = ActionInvocation::no_payload("make_square");
        assert_eq!(INV.action_id, "make_square");
        assert!(INV.payload.is_empty());
    }
}
