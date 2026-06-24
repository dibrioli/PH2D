//! Hierarchy panel `apply_event` — ADR-0029 Phase C.2 port.
//!
//! Migrated from `ph2d_editor_core::screens::hero::hierarchy::
//! apply_event_full` to the panel crate. Signature switched from
//! `(hero: &mut HeroScreen, ev: WidgetEvent)` to
//! `(state: &mut HierarchyState, host: &mut dyn PanelHostInternal,
//! ev: WidgetEvent)`. `hero.<field>` accesses route through
//! [`PanelHostInternal`] trait methods. `hero.hierarchy.live_entries`
//! reads go through [`state::live_entries_contains`] /
//! [`state::live_entry_for`] (thread-local snapshot owned by the
//! panel crate). `hero.hierarchy.rename_target_row` writes land on
//! the typed `state` parameter directly.

use crate::state;
use ph2d_editor_core::action_bus::{EditorAction, SelectModifier};
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{ContextMenuKind, InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_editor_core::screens::hero::{HeroSelection, HierReparentIntent, ViewFocusKind};

pub(crate) fn apply_event(
    state: &mut state::HierarchyState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    // Drag-reparent (M14.6B) — dispatcher emits one HierReparent per
    // drop; live (ECS) mode reads it via the bus.
    if let WidgetEvent::HierReparent {
        dragged,
        new_parent,
        before,
        after,
    } = ev
    {
        host.bus_mut()
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
            host.bus_mut()
                .push(EditorAction::HierToggleVisibility { row: row_id });
            return EventOutcome::Consumed;
        }
        // M14.6C — expand-toggle companion id (chevron click).
        if let Some(row_id) = ids::hier_expand_companion_to_row(id) {
            host.store_mut().toggle_hierarchy_collapsed(row_id);
            return EventOutcome::Consumed;
        }
        // 2026-05-26 — lock + group companion ids. Shell drains
        // HierToggleLock/HierToggleGroup and (un)inserts the matching
        // ECS component on the row's entity.
        if let Some(row_id) = ids::hier_lock_companion_to_row(id) {
            host.bus_mut()
                .push(EditorAction::HierToggleLock { row: row_id });
            return EventOutcome::Consumed;
        }
        if let Some(row_id) = ids::hier_group_companion_to_row(id) {
            host.bus_mut()
                .push(EditorAction::HierToggleGroup { row: row_id });
            return EventOutcome::Consumed;
        }
        // M14.6 F — per-row right-click context menu actions.
        if id == ids::CTX_MENU_HIER_DUPLICATE
            || id == ids::CTX_MENU_HIER_ADD_CHILD
            || id == ids::CTX_MENU_HIER_RESET_TRANSFORM
            || id == ids::CTX_MENU_HIER_DELETE
            || id == ids::CTX_MENU_HIER_RENAME
            || id == ids::CTX_MENU_HIER_MERGE_SPRITES
            || id == ids::CTX_MENU_HIER_USE_AS_BRUSH_TEXTURE
        {
            if let Some(req) = host.store_mut().consume_last_context_menu()
                && let ContextMenuKind::HierarchyRow { row } = req.kind
            {
                if id == ids::CTX_MENU_HIER_USE_AS_BRUSH_TEXTURE {
                    host.bus_mut()
                        .push(EditorAction::HierUseAsBrushTexture { row });
                } else if id == ids::CTX_MENU_HIER_DUPLICATE {
                    host.bus_mut().push(EditorAction::HierDuplicate { row });
                } else if id == ids::CTX_MENU_HIER_ADD_CHILD {
                    host.bus_mut().push(EditorAction::HierAddChild { row });
                } else if id == ids::CTX_MENU_HIER_RESET_TRANSFORM {
                    host.bus_mut()
                        .push(EditorAction::HierResetTransform { row });
                } else if id == ids::CTX_MENU_HIER_DELETE {
                    host.bus_mut().push(EditorAction::HierDelete { row });
                } else if id == ids::CTX_MENU_HIER_MERGE_SPRITES {
                    host.bus_mut().push(EditorAction::HierMergeSprites { row });
                } else if id == ids::CTX_MENU_HIER_RENAME {
                    ph2d_editor_core::screens::hero::open_rename_public(host.store_mut());
                    state.rename_target_row = Some(row);
                    host.bus_mut().push(EditorAction::HierRenameSeed { row });
                }
            }
            return EventOutcome::Consumed;
        }
        // Fase 0c — click on a live hierarchy row → raise the
        // multi-select-aware HierSelectRow / HierRangeSelect BEFORE
        // updating the header selection label, so the shell drain
        // resolves row → entity + mutates GizmoStateGroup according
        // to modifier semantics (Replace / Add / Toggle / Range).
        // Replaces the legacy HierRowClick emission.
        //
        // Onda 2 hotfix: Shift on the hierarchy emits HierRangeSelect
        // — selects-and-deselects (toggle each) every row between the
        // primary's row and the clicked row, inclusive. Cmd / Ctrl
        // toggle the single clicked row (canvas parity for solo
        // toggle). No modifier replaces the selection.
        let live_hit = state::live_entries_contains(id);
        if live_hit {
            let store = host.store();
            let shift = store.shift_held();
            let cmd = store.cmd_held();
            if shift && !cmd {
                host.bus_mut()
                    .push(EditorAction::HierRangeSelect { row: id });
            } else {
                let modifier = if cmd {
                    SelectModifier::Toggle
                } else {
                    SelectModifier::Replace
                };
                host.bus_mut()
                    .push(EditorAction::HierSelectRow { row: id, modifier });
            }
        }
        if let Some(entry) = state::live_entry_for(id) {
            *host.selection_mut() = Some(HeroSelection {
                label: entry.name.clone(),
                kind: entry.badge.clone().unwrap_or_else(|| "ENT".to_string()),
                world_pos: (0.0, 0.0),
            });
            return EventOutcome::Consumed;
        }
        if let Some(label) = ids::hierarchy_label_for_id(id) {
            *host.selection_mut() = Some(HeroSelection {
                label: label.into(),
                kind: ids::hierarchy_kind_for_label(label).into(),
                world_pos: (0.0, 0.0),
            });
            return EventOutcome::Consumed;
        }
        if live_hit {
            return EventOutcome::Consumed;
        }
    }
    // 2026-05-26 — double-click on the entity ICON (left glyph of a
    // row) → focus camera on the row (was: double-click on row name).
    // The icon has its own companion hit id since this commit.
    if let WidgetEvent::DoubleClick(id) = ev
        && let Some(row_id) = ids::hier_icon_companion_to_row(id)
        && state::live_entries_contains(row_id)
    {
        host.bus_mut().push(EditorAction::HierSelectRow {
            row: row_id,
            modifier: SelectModifier::Replace,
        });
        host.bus_mut().push(EditorAction::SetViewFocus {
            kind: ViewFocusKind::Selected,
        });
        return EventOutcome::Consumed;
    }
    // 2026-05-26 — double-click on a live hierarchy row's NAME body →
    // enter inline-rename mode (substitui o long-press). Pra não
    // disparar quando o duplo-clique cai numa companion (eye / lock /
    // group / icon), checa que `id` é o row id em si.
    if let WidgetEvent::DoubleClick(id) = ev
        && state::live_entries_contains(id)
    {
        ph2d_editor_core::screens::hero::open_rename_public(host.store_mut());
        state.rename_target_row = Some(id);
        host.bus_mut()
            .push(EditorAction::HierRenameSeed { row: id });
        return EventOutcome::Consumed;
    }
    // Inline-rename commit (Enter / Submit on HIER_RENAME_INPUT).
    if let WidgetEvent::Submit(id) = ev
        && id == ids::HIER_RENAME_INPUT
        && let Some(row) = state.rename_target_row.take()
    {
        let buf = match host.store().get(ids::HIER_RENAME_INPUT) {
            Some(InteractiveState::TextInput { text, .. }) => text.clone(),
            _ => String::new(),
        };
        let trimmed = buf.trim().to_owned();
        if !trimmed.is_empty() {
            host.bus_mut().push(EditorAction::HierRenameCommit {
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
        state.rename_target_row = None;
        return EventOutcome::Consumed;
    }
    // Implicit commit on Blur (Finder/macOS convention) — stage the
    // current buffer as a HierRenameCommit + drop rename mode.
    // Returns `Observed` so other panels may still see the Blur but
    // the host-level dispatch knows a side effect happened.
    if let WidgetEvent::Blur(id) = ev
        && id == ids::HIER_RENAME_INPUT
        && let Some(row) = state.rename_target_row.take()
    {
        let buf = match host.store().get(ids::HIER_RENAME_INPUT) {
            Some(InteractiveState::TextInput { text, .. }) => text.clone(),
            _ => String::new(),
        };
        let trimmed = buf.trim().to_owned();
        if !trimmed.is_empty() {
            host.bus_mut().push(EditorAction::HierRenameCommit {
                row,
                new_name: trimmed,
            });
        }
        return EventOutcome::Observed;
    }
    EventOutcome::Ignored
}
