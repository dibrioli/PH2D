//! Pointer **Down** context-menu openers — secondary (right-click) menus +
//! the TopBar/chip primary-click popovers. Extracted from the
//! `dispatch_pointer_with_text` god-function (blindagem Fase 3.2). Each opener
//! returns `true` when it handled the event so the caller short-circuits the
//! rest of the Down arm (matching the original `return` shape). Pure move,
//! same behaviour (covered by `dispatch::tests`).

use super::is_section_header_id;
use crate::interaction::types::{ContextMenuKind, ContextMenuRequest};
use crate::interaction::{InteractiveState, WidgetStore};
use crate::zones::Rect;
use ph2d_host::PointerEvent;

/// Open the appropriate context menu / popover for a `Down` event and report
/// whether one was opened (or, for a secondary click, otherwise handled). On
/// `true` the caller returns immediately — no widget focus/drag is started.
pub(super) fn handle_down_menus(
    store: &mut WidgetStore,
    hit: Option<(ph2d_a11y::NodeId, Rect)>,
    event: PointerEvent,
) -> bool {
    // Right-click → open a context menu. We dispatch in two
    // shapes:
    //   - Secondary on a registered widget id whose role is
    //     "section header" (the inspector marks these via
    //     `is_collapsible_section_id` — currently any id in
    //     the `INSP_SECTION_*` range) → `SectionOutline` menu.
    //   - Secondary anywhere inside a panel rect → "CreateNote"
    //     menu parented to that panel.
    // Primary clicks fall through to the regular focus/click
    // path below. A right-click on a non-panel area closes any
    // currently-open menu.
    if event.button == ph2d_host::PointerButton::Secondary {
        // Secondary click INSIDE the BlenderColorPicker belongs to the picker's own
        // dispatch — right-click on a palette swatch REMOVES it (see `apply_blender_hit`).
        // The picker publishes its outer rect to `panel_rects`, so without this guard the
        // CreateNote fallback below (which doesn't exclude the picker) swallows the click
        // and `return true`s, and the swatch-remove never runs. Bail to the regular Down
        // path. Tested directly against the picker rect (not `panel_at`, whose HashMap
        // order could surface a panel the picker overlaps).
        if store
            .panel_rect(crate::ids::INSP_BLENDER_PICKER)
            .is_some_and(|r| r.contains(event.x, event.y))
        {
            return false;
        }
        let panel_under = store.panel_at(event.x, event.y);
        let hit_id = hit.map(|(id, _)| id);
        let is_section = hit_id.map(is_section_header_id).unwrap_or(false);
        // Note slot hit (id range 800..811): right-click on a
        // painted note opens the NoteBackground menu for that
        // slot's index. The inspector painter publishes the
        // slot→note-index mapping by always painting note
        // `i` at `NOTE_SLOT_IDS[i]`, so slot id - 800 IS the
        // note index.
        let note_slot = hit_id.and_then(|id| {
            let v = id.0;
            if (800..=811).contains(&v) {
                Some((v - 800) as u8)
            } else {
                None
            }
        });
        // M14.6 F: right-click on a hierarchy row opens the
        // per-entity actions menu. Resolved BEFORE the broader
        // panel-under fallback because the row lives inside
        // the hierarchy panel — the CreateNote menu must not
        // win over this more specific kind. Eye/chevron
        // companion ids are stripped first so a right-click on
        // those toggles still reaches the parent row.
        let hier_row_id = hit_id.and_then(|id| {
            if let Some(row) = crate::ids::hier_eye_companion_to_row(id) {
                Some(row)
            } else if let Some(row) = crate::ids::hier_expand_companion_to_row(id) {
                Some(row)
            } else if let Some(row) = crate::ids::hier_lock_companion_to_row(id) {
                Some(row)
            } else if let Some(row) = crate::ids::hier_group_companion_to_row(id) {
                Some(row)
            } else if let Some(row) = crate::ids::hier_icon_companion_to_row(id) {
                Some(row)
            } else {
                Some(id)
            }
            .filter(|row| store.is_hierarchy_row(*row))
        });
        if let Some(row) = hier_row_id {
            store.open_context_menu(ContextMenuRequest {
                x: event.x,
                y: event.y,
                kind: ContextMenuKind::HierarchyRow { row },
            });
        } else if let Some(note_index) = note_slot
            && let Some(panel) = panel_under
        {
            store.open_context_menu(ContextMenuRequest {
                x: event.x,
                y: event.y,
                kind: ContextMenuKind::NoteBackground { panel, note_index },
            });
        } else if is_section {
            let section_id = hit_id.unwrap();
            store.open_context_menu(ContextMenuRequest {
                x: event.x,
                y: event.y,
                kind: ContextMenuKind::SectionOutline {
                    section: section_id,
                },
            });
        } else if let Some(panel) = panel_under.filter(|p| {
            *p != crate::ids::HIER_PANEL
                && *p != crate::ids::PAD_PANEL
                && *p != crate::ids::BGR_PANEL
                && *p != crate::ids::CEQ_PANEL
                && *p != crate::ids::UPS_PANEL
                && *p != crate::ids::EQS_PANEL
                && *p != crate::ids::PAINTER_LAYERS_PANEL
                && *p != crate::grid_snap::ids::GS_PANEL
        }) {
            // `before_section` is filled in by apply_event
            // — only the inspector knows the screen→body
            // conversion + section y-ranges.
            //
            // Hierarchy + image-tool panels (PAD/BGR/CEQ/UPS/EQS)
            // are excluded by design — these are transient
            // operation surfaces, not annotation surfaces. UI
            // canon post-2026-05-24: notes + outlines live in
            // Inspector + Widget Gallery only.
            store.open_context_menu(ContextMenuRequest {
                x: event.x,
                y: event.y,
                kind: ContextMenuKind::CreateNote {
                    panel,
                    before_section: None,
                },
            });
        } else {
            store.close_context_menu();
        }
        return true;
    }
    // Primary click on the TopBar theme cluster opens the
    // ThemeSelector context menu (4 themes + 3 corner-radius
    // presets). Anchored just below the cluster's hit rect
    // so the popover doesn't overlap the cluster itself.
    // The `Plain` state check disambiguates from other
    // widgets that may happen to share the TOPBAR_THEME
    // NodeId numeric value in isolated unit tests (the
    // hero's real `populate` registers it as Plain).
    if event.button == ph2d_host::PointerButton::Primary
        && let Some((hit_id, hit_rect)) = hit
        && hit_id == crate::ids::TOPBAR_THEME
        && matches!(store.get(hit_id), Some(InteractiveState::Button { .. }))
    {
        store.open_context_menu(ContextMenuRequest {
            x: hit_rect.x,
            y: hit_rect.y + hit_rect.h + 4.0,
            kind: ContextMenuKind::ThemeSelector,
        });
        return true;
    }
    // Same pattern for the Save chip — Primary opens the
    // Save / Save As menu anchored below the chip.
    if event.button == ph2d_host::PointerButton::Primary
        && let Some((hit_id, hit_rect)) = hit
        && hit_id == crate::ids::TOPBAR_SAVE
        && matches!(store.get(hit_id), Some(InteractiveState::Button { .. }))
    {
        store.open_context_menu(ContextMenuRequest {
            x: hit_rect.x,
            y: hit_rect.y + hit_rect.h + 4.0,
            kind: ContextMenuKind::SaveMenu,
        });
        return true;
    }
    // Open chip — same anchor logic.
    if event.button == ph2d_host::PointerButton::Primary
        && let Some((hit_id, hit_rect)) = hit
        && hit_id == crate::ids::TOPBAR_OPEN
        && matches!(store.get(hit_id), Some(InteractiveState::Button { .. }))
    {
        store.open_context_menu(ContextMenuRequest {
            x: hit_rect.x,
            y: hit_rect.y + hit_rect.h + 4.0,
            kind: ContextMenuKind::OpenMenu,
        });
        return true;
    }
    // Settings cluster (gear) — opens the SettingsMenu with
    // px/m presets. Same anchor convention as Save/Open.
    if event.button == ph2d_host::PointerButton::Primary
        && let Some((hit_id, hit_rect)) = hit
        && hit_id == crate::ids::TOPBAR_SETTINGS
        && matches!(store.get(hit_id), Some(InteractiveState::Button { .. }))
    {
        store.open_context_menu(ContextMenuRequest {
            x: hit_rect.x,
            y: hit_rect.y + hit_rect.h + 4.0,
            kind: ContextMenuKind::SettingsMenu,
        });
        return true;
    }
    // Project chip → SceneList popover (search + scenes).
    if event.button == ph2d_host::PointerButton::Primary
        && let Some((hit_id, hit_rect)) = hit
        && hit_id == crate::ids::TOPBAR_PROJECT
        && matches!(store.get(hit_id), Some(InteractiveState::Button { .. }))
    {
        store.open_context_menu(ContextMenuRequest {
            x: hit_rect.x,
            y: hit_rect.y + hit_rect.h + 4.0,
            kind: ContextMenuKind::SceneList,
        });
        return true;
    }
    false
}
