// ph2d-chrome-sync:z=210 (dispatch priority, ADR-0107; lower = earlier)
//! Image-edit actions on the bus — Trim, Make Square, Real Size,
//! Bg Removal, Padding, Undo. Wave 2.5 PR 11.8b migration from
//! `pending_X` fields; ADR-0040 TG-A made the dispatch generic
//! (`OneShotImageOp { tool_id, entity }` + `ActivateTool { tool_id }`)
//! so chrome no longer names per-tool variants.

use crate::action_bus::EditorAction;
use crate::ids;
use crate::interaction::WidgetEvent;
use crate::screens::hero::HeroScreen;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    // One-shot image ops (Trim / Make Square / Real Size) — push one
    // generic `OneShotImageOp` per selected sprite. Fase 0e broadcast:
    // with single selection this is identical to the pre-Fase-0
    // single-push path; with multi-select each selected sprite gets
    // its own action, and the per-tool drain function in the shell
    // applies the bake independently to each. Empty selection still
    // consumes the click (parity with the legacy per-variant arms).
    if let Some(tool_id) = oneshot_tool_for(id) {
        for entity_bits in hero.gizmo.iter_selected() {
            hero.bus.push(EditorAction::OneShotImageOp {
                tool_id,
                entity_bits,
            });
        }
        return true;
    }
    // Stateful tool activations — Bg Removal opens the live-preview
    // panel; Padding opens the per-edge fields panel. The apply
    // triggers live in each tool's panel.
    if let Some(tool_id) = stateful_tool_for(id) {
        hero.bus.push(EditorAction::ActivateTool { tool_id });
        return true;
    }
    // Image-edit Undo — TOOL_UNDO chip on the LeftRail (also bound to
    // Cmd+Z in the desktop shell). When no snapshot exists the shell's
    // drainer surfaces a "Nothing to undo" toast.
    if id == ids::TOOL_UNDO {
        hero.bus.push(EditorAction::UndoImageEdit);
        return true;
    }
    false
}

/// Map an `IMAGE_ACTION_*` pill id to the canonical tool id for the
/// one-shot image ops (those whose chrome is a single click → bake,
/// no modal panel). Returns `None` for non-one-shot ids.
fn oneshot_tool_for(id: ph2d_a11y::NodeId) -> Option<&'static str> {
    if id == ids::IMAGE_ACTION_TRIM {
        Some("trim_transparency")
    } else if id == ids::IMAGE_ACTION_MAKE_SQUARE {
        Some("make_square")
    } else if id == ids::IMAGE_ACTION_REAL_SIZE {
        Some("real_size")
    } else if id == ids::IMAGE_ACTION_RASTERIZE {
        Some("rasterize")
    } else {
        None
    }
}

/// Map an `IMAGE_ACTION_*` pill id to the canonical tool id for the
/// stateful image tools (those whose chrome click activates a modal
/// tool that opens its own panel). Returns `None` for non-stateful ids
/// (one-shot tools handled by [`oneshot_tool_for`]).
///
/// **Production path (audit 2026-05-26 F2):** data-driven via
/// `installed_registry().cluster("image_tools")`, filtrando por
/// `ToolHandler::Stateful` e casando `hash_node_id(manifest.id) == id`.
/// Tools dropped via fan-out drop-crate herdam automaticamente.
///
/// **Test/pre-registry-boot fallback:** quando `installed_registry()`
/// retorna `None` (testes isolados antes de `install_registry`), o lookup
/// usa a tabela `STATEFUL_TOOL_IDS` (lista de slugs auditável, espelha
/// o conjunto de stateful manifests). Tabela é mantida sincronizada com
/// os manifests; a fonte canônica é o `ToolHandler::Stateful` nos
/// `crates/ph2d-tool-*/src/lib.rs`. **Não é if-else hardcoded** — é uma
/// table-driven map auditável + sincronizável via `tool-sync` codegen
/// futuro (registered no plano como follow-up W1 T-codegen-stateful).
///
/// Audit anterior (memória `feedback_fanout_registry_init_friction`)
/// recomendou extender tool-sync; trabalho em batch futuro junto com
/// outros 2 sites hand-maintained em `tool-registry-init`.
fn stateful_tool_for(id: ph2d_a11y::NodeId) -> Option<&'static str> {
    use ph2d_tool_registry::{ToolHandler, hash_node_id};

    // Production path — data-driven via registry.
    if let Some(reg) = crate::installed_registry() {
        let found = reg.cluster("image_tools").iter().find_map(|m| {
            if !matches!(m.handler, ToolHandler::Stateful { .. }) {
                return None;
            }
            if hash_node_id(m.id) == id {
                Some(m.id)
            } else {
                None
            }
        });
        if found.is_some() {
            return found;
        }
    }

    // Test/pre-boot fallback — table-driven (não if-else hardcoded).
    // Lista canônica dos stateful image-tools; mantida em sync com os
    // manifests via tool-sync codegen (W1 T-codegen-stateful follow-up).
    const STATEFUL_TOOL_IDS: &[&str] = &[
        "bgremoval",
        "color_equalization",
        "equalize_sizes",
        "padding",
        "painter",
        "upscale",
    ];
    STATEFUL_TOOL_IDS
        .iter()
        .find(|slug| hash_node_id(slug) == id)
        .copied()
}
