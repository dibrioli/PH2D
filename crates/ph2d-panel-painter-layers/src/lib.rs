//! `ph2d-panel-painter-layers` — typed `Panel<State>` para o painel de
//! camadas do Painter (ADR-0029 + W3.T3.4 plan §6, design 02_layers.md).
//!
//! Right-docked (Inspector geometry slot, mirror do `painter_sidebar`);
//! visível só quando o `painter` tool é ativo (shell drives
//! `panel_visible("painter_layers")` from the active-tool id, em
//! `render_loop/painter_bridge.rs`). Lista as layers do `LayerStack`:
//! thumb + name + visibility + opacity slider + blend dropdown.
//!
//! ## SCAFFOLD (Coordenador, caminho B)
//!
//! Este crate entrega: chrome canon (surface, corner dot, title "Layers",
//! close, drag/resize handles), o snapshot publish (`set_current_layers`),
//! a docagem (layout slot + visibility), o registro no panel-registry-init,
//! e um body **placeholder** ("No layers"). O Implementador preenche as
//! layer rows reais — ver `// TODO(impl W3.T3.4)` em `paint.rs`.
//!
//! ## Param canon location
//!
//! O `LayerStack` canônico vive no `PainterTool` (shell-side ToolRegistry).
//! Cada frame o shell publica um snapshot via [`set_current_layers`] →
//! `paint` lê. Edits sairão via `EditorAction::ToolPanelEvent` (ADR-0040
//! TG-B canal genérico) → shell chama `PainterTool::handle_panel_event`.

#![forbid(unsafe_code)]

mod adjust_menu;
/// Os nomes que as camadas de ajuste pintam: do rótulo da crate de efeitos para a tabela de strings.
pub mod adjust_nomes;
mod blend;
mod brush_fallback;

/// ⭐ **O instantâneo de pincel que o painel usa antes de a shell publicar um** — exposto para a
/// varredura de rótulos o poder ARMAR.
///
/// ⛔⛔ Sem isto o Painter é medido no estado de FÁBRICA, onde a textura é `None` e as fileiras
/// por-padrão **não são pintadas** — e foi por isso que o report do dono de 2026-09-21 (a secção
/// `SHAPE ▸ Texture` a mostrar `paint_brush.pattern_param.contrast`) atravessou 30 censos de
/// texto e uma varredura de ecrã. *Um painel cujas fileiras dependem de uma escolha tem de ser
/// armado, como o Inspector já é.*
pub use crate::brush_fallback::FALLBACK_BRUSH;
mod card; // the titled row-box shared by the brush panel technique sections
mod composite_picker;
mod dropdown_popover;
mod event;
mod event_brush_forward;
/// As barras do card Line: a tabela que o pintor, o registo e o encaminhamento leem.
pub mod line_barras;
mod number_field;
mod paint;
mod paint_adjust;
mod paint_brush;
/// Os primitivos de ROW que o card Brush empresta aos outros (rótulo, row de dropdown, chip).
mod paint_brush_rows;
mod paint_brush_sections;
mod paint_brush_top;
mod paint_clone;
mod paint_composite;
mod paint_composite_montagem;
mod paint_deform;
mod paint_falloff;
mod paint_impasto;
mod paint_impasto_rig; // Impasto: the Body (per-brush) + Lighting (per-canvas) cards
mod paint_impasto_tool;
mod paint_inpaint;
mod paint_layer_list;
mod paint_line; // o card Line (Style: Solid) — plano 38 §1
mod paint_mask;
mod paint_mask_row;
mod paint_pigment; // a fileira `Pigment`: uma lei, uma porta, os três meios que a sentem
mod paint_ramp_widget;
mod paint_rows;
mod paint_rows_relief;
mod paint_sculpt;
mod paint_seg_row; // the segmented row of a card — Impasto, the light rig and Wet Paint
mod paint_selection;
mod paint_shape;
mod paint_shape_dab;
mod paint_shape_layers;
mod paint_shape_ramp;
mod paint_stencil;
mod paint_stroke;
mod paint_symmetry;
mod paint_taper;
mod paint_texture;
mod paint_texture_ramp;
mod paint_texture_tiling; // the Grain's Offset / Size / Depth rows
mod paint_watercolor;
mod paint_watercolor_paper;
mod paint_wetpaint;
mod paint_wetpaint_tilt; // doc 22: the TILT dial (polar pad) of the Wet Paint section
mod populate;
mod populate_brush_chips;
mod populate_composite_chips;
mod populate_deform;
/// The Grid Stamp card's widgets (its four sliders + chips and the Show Grid checkbox).
mod populate_grid_stamp;
mod populate_sculpt;
mod populate_sections;
/// As secções que pintam linhas de marcar, e os rótulos de cada uma (a coluna do nome sai daqui).
pub mod seccoes;
pub mod state;
mod state_dropdowns;
mod state_ramp;
pub mod stroke_method_offer; // the Method dropdown's narrowing law — pure, gate-tested from `tests/`

pub use state::{
    FalloffHit, PainterLayersPanelState, falloff_canvas_norm, falloff_hit_test, last_content_h,
    last_visible_h, selected_falloff_point, set_current_brush, set_current_brush_paper_image,
    set_current_brush_shape_color_preview, set_current_brush_shape_image,
    set_current_brush_texture_image, set_current_dock_shows_layers, set_current_layers,
    set_current_mask_grayscale_view, set_current_selection, set_selected_falloff_point,
};

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal, TextKey};

/// Zero-size marker que implementa o contrato typed Painter Layers panel.
pub struct PainterLayersPanel;

impl Panel for PainterLayersPanel {
    type State = PainterLayersPanelState;

    const ID: &'static str = "painter_layers";
    const NODE_ID: NodeId = ph2d_editor_core::ids::PAINTER_LAYERS_PANEL;
    const DEFAULT_VISIBLE: bool = false;
    /// ⭐⭐ **O nome da ABA é o do MÓDULO, não o de um dos dois modos** (report do Enio,
    /// 2026-09-09, com foto: *«a aba ficou com nome de Layers em vez de algo como painter»*).
    ///
    /// ⛔ Este painel hospeda **dois** corpos — *Brush* e *Layers* — e o título dizia o nome de um
    /// deles. Com a fileira de abas isso passou a ser visível e contraditório: a aba dizia
    /// «Layers» enquanto o cabeçalho por baixo dela dizia «Brush». *Uma aba nomeia o que ela
    /// ABRE; se o que ela abre tem dois modos, o nome é o do que os contém.*
    ///
    /// ⚠️ O título do CABEÇALHO continua a seguir o modo (`paint_brush_top::header_title`) — são
    /// duas perguntas: *«que painel é este?»* e *«que modo estou a ver?»*.
    const TITLE: TextKey = TextKey::new("panel.painter_layers.title");
    const ICON: ph2d_editor_core::icons::IconId = ph2d_editor_core::icons::IconId::Painter;
    /// ⚠️ **Um painel de COLUNA não cabe na faixa de baixo.** Ela tem 240 px de altura e a
    /// largura da área: uma lista de propriedades ali fica com duas linhas visíveis. ⇒ as duas
    /// colunas, e o gesto que o levaria ao fundo não é oferecido (decisão D1).
    const ALLOWED_SLOTS: ph2d_editor_core::screens::slot::SlotSet =
        ph2d_editor_core::screens::slot::SlotSet::SIDES;
    const DEFAULT_SLOT: ph2d_editor_core::screens::slot::Slot =
        ph2d_editor_core::screens::slot::Slot::RightTop;

    fn paint(state: &mut PainterLayersPanelState, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut PainterLayersPanelState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        event::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        populate::populate(store);
    }
}
