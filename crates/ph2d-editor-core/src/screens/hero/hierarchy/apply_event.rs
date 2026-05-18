//! Hierarchy panel — apply_event handler (Wave 6+7 Phase 4
//! distribution). Owns every hierarchy-related event branch that
//! previously lived in `HeroScreen::apply_event`'s god-match.

use crate::action_bus::EditorAction;
use crate::interaction::{ContextMenuKind, InteractiveState, WidgetEvent};
use crate::panel_registry::EventOutcome;
use crate::screens::hero::{HeroScreen, HeroSelection, HierReparentIntent, ViewFocusKind, ids};

/// Try to handle `ev` against the hierarchy panel. Returns true iff
/// claimed. Covers:
///
/// - Live hierarchy row click → push HierRowClick + update selection.
/// - Fixture hierarchy row click → update selection label.
/// - `WidgetEvent::HierReparent` drag-reparent → push HierReparent.
/// - Eye-toggle companion id → push HierToggleVisibility.
/// - Expand-toggle companion id → toggle store.hierarchy_collapsed.
/// - Right-click context menu actions (CTX_MENU_HIER_*) → push Hier*.
/// - DoubleClick on live row → push HierRowClick + SetViewFocus.
/// - LongPress on live row → enter inline-rename mode.
/// - Submit / Cancel / Blur on `HIER_RENAME_INPUT`.
pub fn apply_event_full(hero: &mut HeroScreen, ev: WidgetEvent) -> EventOutcome {
    // M14.6B — drag-reparent. Dispatcher emits one HierReparent per
    // drop; live (ECS) mode reads it via the bus.
    if let WidgetEvent::HierReparent {
        dragged,
        new_parent,
        before,
        after,
    } = ev
    {
        hero.bus
            .push(EditorAction::HierReparent(HierReparentIntent {
                dragged,
                new_parent,
                before,
                after,
            }));
        return EventOutcome::Consumed;
    }
    if let WidgetEvent::Click(id) = ev {
        // M14.6A — eye-toggle companion id.
        if let Some(row_id) = ids::hier_eye_companion_to_row(id) {
            hero.bus
                .push(EditorAction::HierToggleVisibility { row: row_id });
            return EventOutcome::Consumed;
        }
        // M14.6C — expand-toggle companion id (chevron click).
        if let Some(row_id) = ids::hier_expand_companion_to_row(id) {
            hero.store.toggle_hierarchy_collapsed(row_id);
            return EventOutcome::Consumed;
        }
        // M14.6 F — per-row right-click context menu actions.
        if id == ids::CTX_MENU_HIER_DUPLICATE
            || id == ids::CTX_MENU_HIER_ADD_CHILD
            || id == ids::CTX_MENU_HIER_RESET_TRANSFORM
            || id == ids::CTX_MENU_HIER_DELETE
            || id == ids::CTX_MENU_HIER_RENAME
        {
            if let Some(req) = hero.store.consume_last_context_menu()
                && let ContextMenuKind::HierarchyRow { row } = req.kind
            {
                if id == ids::CTX_MENU_HIER_DUPLICATE {
                    hero.bus.push(EditorAction::HierDuplicate { row });
                } else if id == ids::CTX_MENU_HIER_ADD_CHILD {
                    hero.bus.push(EditorAction::HierAddChild { row });
                } else if id == ids::CTX_MENU_HIER_RESET_TRANSFORM {
                    hero.bus.push(EditorAction::HierResetTransform { row });
                } else if id == ids::CTX_MENU_HIER_DELETE {
                    hero.bus.push(EditorAction::HierDelete { row });
                } else if id == ids::CTX_MENU_HIER_RENAME {
                    crate::screens::hero::open_rename_public(&mut hero.store);
                    hero.hierarchy.rename_target_row = Some(row);
                    hero.bus.push(EditorAction::HierRenameSeed { row });
                }
            }
            return EventOutcome::Consumed;
        }
        // M14.6 D — click on a live hierarchy row → raise
        // HierRowClick BEFORE the selection-label update below so
        // the shell resolves row → entity in the next drain.
        let live_hit = hero
            .hierarchy
            .live_entries
            .as_ref()
            .is_some_and(|live| live.contains_key(&id));
        if live_hit {
            hero.bus.push(EditorAction::HierRowClick { row: id });
            // Continue — also update selection label below (existing
            // hierarchy_label_for_id branch handles live mode via
            // live_entries lookup).
        }
        // Selection update — live mode (live_entries lookup) +
        // fixture fallback (hierarchy_label_for_id).
        if let Some(live) = hero.hierarchy.live_entries.as_ref()
            && let Some(entry) = live.get(&id)
        {
            hero.selection = Some(HeroSelection {
                label: entry.name.clone(),
                kind: entry.badge.clone().unwrap_or_else(|| "ENT".to_string()),
                world_pos: (0.0, 0.0),
            });
            return EventOutcome::Consumed;
        }
        if let Some(label) = ids::hierarchy_label_for_id(id) {
            hero.selection = Some(HeroSelection {
                label: label.into(),
                kind: ids::hierarchy_kind_for_label(label).into(),
                world_pos: (0.0, 0.0),
            });
            return EventOutcome::Consumed;
        }
        if live_hit {
            // Live row click had no live_entries match (defensive);
            // still consumed.
            return EventOutcome::Consumed;
        }
    }
    // M14.7 polish — double-click on a live hierarchy row → push
    // HierRowClick + SetViewFocus(Selected).
    if let WidgetEvent::DoubleClick(id) = ev
        && let Some(live) = hero.hierarchy.live_entries.as_ref()
        && live.contains_key(&id)
    {
        hero.bus.push(EditorAction::HierRowClick { row: id });
        hero.bus.push(EditorAction::SetViewFocus {
            kind: ViewFocusKind::Selected,
        });
        return EventOutcome::Consumed;
    }
    // M14.7 polish — long-press on a live hierarchy row → enter
    // inline-rename mode.
    if let WidgetEvent::LongPress(id) = ev
        && let Some(live) = hero.hierarchy.live_entries.as_ref()
        && live.contains_key(&id)
    {
        crate::screens::hero::open_rename_public(&mut hero.store);
        hero.hierarchy.rename_target_row = Some(id);
        hero.bus.push(EditorAction::HierRenameSeed { row: id });
        return EventOutcome::Consumed;
    }
    // Inline-rename commit (Enter / Submit on HIER_RENAME_INPUT).
    if let WidgetEvent::Submit(id) = ev
        && id == ids::HIER_RENAME_INPUT
        && let Some(row) = hero.hierarchy.rename_target_row.take()
    {
        let buf = match hero.store.get(ids::HIER_RENAME_INPUT) {
            Some(InteractiveState::TextInput { text, .. }) => text.clone(),
            _ => String::new(),
        };
        let trimmed = buf.trim().to_owned();
        if !trimmed.is_empty() {
            hero.bus.push(EditorAction::HierRenameCommit {
                row,
                new_name: trimmed,
            });
        }
        return EventOutcome::Consumed;
    }
    // Inline-rename cancel (Esc) — drop rename mode, no commit.
    if let WidgetEvent::Cancel(id) = ev
        && id == ids::HIER_RENAME_INPUT
    {
        hero.hierarchy.rename_target_row = None;
        return EventOutcome::Consumed;
    }
    // Implicit commit on Blur (Finder/macOS convention) — stage the
    // current buffer as a HierRenameCommit + drop rename mode.
    // Wave 8 Phase 4: returns `Observed` instead of the old
    // `false`-with-side-effect (audit B2). Other panels may still
    // see the Blur; we did our side-effect but don't claim
    // exclusivity. The dispatcher's `Observed` flag ensures the
    // outer chrome dispatch still sees the event was acted upon.
    if let WidgetEvent::Blur(id) = ev
        && id == ids::HIER_RENAME_INPUT
        && let Some(row) = hero.hierarchy.rename_target_row.take()
    {
        let buf = match hero.store.get(ids::HIER_RENAME_INPUT) {
            Some(InteractiveState::TextInput { text, .. }) => text.clone(),
            _ => String::new(),
        };
        let trimmed = buf.trim().to_owned();
        if !trimmed.is_empty() {
            hero.bus.push(EditorAction::HierRenameCommit {
                row,
                new_name: trimmed,
            });
        }
        return EventOutcome::Observed;
    }
    EventOutcome::Ignored
}
