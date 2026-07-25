#![forbid(unsafe_code)]

//! `ph2d-tool-flip` — a ferramenta de desenho do Flip (ADR-0114 W2).
//!
//! A ferramenta de primeira classe do módulo Flip (animação quadro-a-quadro): um
//! pill do cluster `flip_tools` que ativa o desenho sobre o `FlipDoc`. Modos
//! **Select/Draw/Erase** (a arbitragem espelha o Vector, ADR-0112: gizmo só no
//! Select).
//!
//! Por ADR-0040 (tool-as-isolated-feature-crate): a tool carrega só o **modelo de
//! brush** (cor / largura / dureza / opacidade / smoothing + modo). O documento
//! (`FlipDoc`) e a interação (traço em curso, pointer→mundo) vivem no shell
//! (`flip_bridge`), que lê o estilo por downcast via [`Tool::as_any_mut`] —
//! mesmo shape das tools Vector / Painter.
//!
//! ## Smoke contract
//!
//! Clique no pill FLIP → o desenho ativa → o painel docado `ph2d-panel-flip`
//! aparece (slot do Inspector) → em Draw, arrastar no canvas cria um traço no
//! desenho ativo → o painel edita cor/largura/dureza ao vivo.

pub mod icon;
pub mod params;
pub mod tool;

pub use params::{
    DOT_SPACING_MAX, EditDomain, EraseMode, FillMode, FlipMode, FlipStyleSnapshot, GAP_MAX_WORLD,
    GROW_MAX, GROW_MIN, OPACITY_SLIDER_SCALE, PRECISION_MAX, PRECISION_MIN, ReshapeKind,
    SIZE_PX_PER_WORLD, TRAP_MAX_PX, WIDTH_MAX_PX, WIDTH_MIN_PX, WIDTH_SLIDER_OFFSET,
    WIDTH_SLIDER_SCALE, px_to_slider, size_to_world, slider_to_px, slider_to_unit,
};
/// O enum do *tip* pontilhado vem do MODELO (`ph2d-flip`) — re-exportado aqui para o painel
/// (que já depende deste crate) nomeá-lo sem uma aresta nova de dependência.
pub use ph2d_flip::{DEFAULT_DOT_SPACING, StrokeTip};
pub use tool::{
    DEFAULT_FILL, DEFAULT_HARDNESS, DEFAULT_OPACITY, DEFAULT_PRECISION, DEFAULT_SMOOTHING,
    DEFAULT_STROKE, DEFAULT_WIDTH_PX, FlipTool,
};

use ph2d_a11y::Role;
use ph2d_core::MemoryBudget;
use ph2d_tool_registry::{HandlerFn, McpExposure, Registry, ToolHandler, ToolManifest, Zone};

/// Constrói a tool Flip como trait object boxed. Alvo do codegen do
/// `ph2d-tool-sync` (`register_all_tools`).
pub fn make() -> Box<dyn ph2d_editor_core::tool::Tool> {
    Box::new(FlipTool::default())
}

/// Handler placeholder do slot `ToolHandler::Stateful` — nunca roda; a semântica
/// real está no `impl Tool for FlipTool` + no `flip_bridge` do shell.
fn noop_manifest_handler() {}

/// Manifest da tool. Cluster `flip_tools` (novo medium); `order = 10`.
pub const MANIFEST: ToolManifest = ToolManifest {
    id: "flip",
    label_key: "tool.flip.label",
    icon_fn: icon::flip_bezpath,
    zone: Zone::TopRight,
    cluster: "flip_tools",
    order: 10,
    a11y_role: Role::Button,
    handler: ToolHandler::Stateful {
        on_activate: noop_manifest_handler as HandlerFn,
        on_deactivate: noop_manifest_handler as HandlerFn,
        on_panel_event: noop_manifest_handler as HandlerFn,
    },
    // A tool guarda só alguns escalares de estilo; o documento vive no shell.
    // Reserva 1 MB contra o boot-check HR-13.
    memory_budget: MemoryBudget::new(0, 1, 0),
    touches_sim: false,
    mcp: McpExposure::reserved(),
};

/// Registra a tool Flip num `Registry` de manifests.
pub fn register(reg: &mut Registry) {
    reg.register(&MANIFEST);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_attaches_manifest() {
        let mut reg = Registry::default();
        register(&mut reg);
        assert_eq!(reg.manifests().len(), 1);
        assert_eq!(reg.manifests()[0].id, "flip");
        assert_eq!(reg.manifests()[0].cluster, "flip_tools");
    }

    #[test]
    fn make_builds_flip_tool() {
        let t = make();
        assert_eq!(
            t.id(),
            ph2d_editor_core::floating_panel::ToolId::new("flip")
        );
    }
}
