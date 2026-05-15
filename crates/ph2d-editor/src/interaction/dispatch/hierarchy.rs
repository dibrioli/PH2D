//! Hierarchy panel drag-and-drop resolution.
//!
//! Extracted from [`super`] (Track A9). One responsibility — given
//! the cursor's y-position over the hierarchy panel and the row
//! being dragged, decide where the drop should land. Row geometry
//! is split into three vertical bands (top 30% → sibling above,
//! middle 40% → re-parent inside, bottom 30% → sibling below).
//!
//! [`HierDrop`] is `pub(crate)` because the host's pointer-Up
//! handler in `screens::hero` matches on it to issue the actual
//! ECS mutation.

use super::super::{HitIndex, WidgetStore};
use ph2d_a11y::NodeId;

/// Drop kind resolved at the end of a hierarchy DnD: a sibling
/// insertion (above or below the given row), or a re-parent inside
/// the given row.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum HierDrop {
    /// Drop dragged just before this row as a sibling.
    Before(NodeId),
    /// Drop dragged just after this row as a sibling. Resolved by
    /// the host to `(target's parent, before = target's next
    /// sibling, or None for "append at end")`. Used for the bottom-
    /// 30% drop band on the LAST visible child of a parent — the
    /// pre-M14.7-polish `End` fallthrough turned that into a root
    /// promotion instead of "append in this parent".
    After(NodeId),
    /// Drop dragged as a child of this row.
    Inside(NodeId),
    /// Drop at the very bottom (root level, end of list).
    End,
}

/// Resolve the drop position for a hierarchy DnD using the cursor's
/// vertical position vs each row rect. Row y is split into three
/// bands:
///   - top 30% → drop above this row (sibling)
///   - middle 40% → drop inside this row (child)
///   - bottom 30% → continue scanning (drop below this row)
///
/// Horizontal position (cursor_x) is intentionally NOT used to gate
/// `Inside` vs `Before` — that previous attempt blocked the common
/// case "drag X onto Y to make X a child of Y when Y is already
/// someone's child". Users naturally hold the cursor near where they
/// pressed down, which may sit left of an indented target row;
/// requiring `cursor_x >= row.x` then forced sibling semantics even
/// when the user clearly aimed at the row vertically. Painter
/// indicator mirrors this same y-only logic, so what the user sees
/// is what they get.
///
/// When the cursor lands below every row, returns `End` — a true
/// root append (now safe thanks to the `RootOrder` component; before
/// that, `End` snapped back to `Entity::to_bits` sort). The
/// in-row bottom-30% band still resolves to `After(id)` so dropping
/// at the foot of a nested last-child still appends inside that
/// child's parent — the user's "drop after t preserves parent"
/// behavior. Skips the dragged row itself.
pub(super) fn find_hierarchy_drop(
    hit_index: &HitIndex,
    store: &WidgetStore,
    cursor_y: f32,
    dragged: NodeId,
) -> HierDrop {
    for (id, rect) in hit_index.iter_registrations() {
        // Live mode: the static fixture range (400..=411) misses
        // every ECS-bridge row, so consult the store's per-frame
        // row set instead. The fixture range stays valid for the
        // demo's prepopulated ids — both pass `is_hierarchy_row`
        // because `populate_live` and `populate` both call
        // `set_hierarchy_row_ids`.
        if !store.is_hierarchy_row(id) {
            continue;
        }
        if id == dragged {
            continue;
        }
        let top = rect.y;
        let bot = rect.y + rect.h;
        let inside_top = top + rect.h * 0.3;
        let inside_bot = top + rect.h * 0.7;
        if cursor_y < top || cursor_y >= bot {
            continue;
        }
        if cursor_y < inside_top {
            return HierDrop::Before(id);
        } else if cursor_y < inside_bot {
            return HierDrop::Inside(id);
        } else {
            // Bottom band: drop AS THE NEXT SIBLING of this row.
            // Host resolves "after t" to t's parent + slot just past
            // t in the Children list. Last-child-of-parent + drop
            // here = append to parent's Children.
            return HierDrop::After(id);
        }
    }
    // Cursor is below every visible row → root append. Host clears
    // `ChildOf` on the dragged entity and writes a fresh `RootOrder`
    // index past the last existing root, so the panel will paint it
    // at the very bottom on the next frame.
    HierDrop::End
}
