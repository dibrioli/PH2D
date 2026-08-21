//! `ph2d-panel-inspector` — ADR-0029 Phase C.1 typed panel crate.
//!
//! Owns the Inspector panel's:
//! - retained state ([`InspectorState`]) — held by `ErasedPanel` in
//!   the runtime `panel::PANEL_REGISTRY`.
//! - paint thunk ([`paint`]), event router ([`event`]),
//!   snapshot sync ([`sync`]), widget registration ([`populate`]).
//! - typed contract via [`InspectorPanel`] implementing
//!   [`ph2d_editor_core::panel::Panel`].
//!
//! Per-frame snapshots the host publishes to drive the live sections
//! flow through thread-local setters re-exported below
//! ([`set_current_inspector_sprite`] etc.); these supersede the
//! pre-C.1 `hero.inspector.*` field writes.

#![forbid(unsafe_code)]

mod event;
mod event_joint;
mod event_ordering;
mod event_physics;
mod event_player;
mod event_precision;
/// Os dois sliders-com-chip da sprite (Opacidade + Emissive) — irmão do `event`, que estava no tecto.
mod event_sprite_value;
mod event_wheel;
mod paint;
mod paint_frame;
mod populate;
mod populate_physics;
mod sections;
pub mod state;
mod sync;
mod sync_physics;
/// Os dois sliders-com-chip da sprite (Opacidade + Emissive) — irmão do `sync`, que estava no tecto.
mod sync_sprite_value;

/// The §11 Bake button's label — exported so a gate can hold the claim that
/// the button shows the range it would cover.
pub use sections::bake_label;
pub use sections::paste_label;
pub use sections::rig_button_label;
/// **Quantas rows numéricas a §14 Platform Player pinta.**
///
/// ⚠️ Exportado para que a varredura de seam possa afirmar que cobre a tabela
/// INTEIRA. Sem ele o gate iteraria a própria lista que testa — o oráculo
/// auto-referente que a `line/Painter` já pagou: encolher o array encolhe a
/// varredura, e a mutação passa.
pub const PLAYER_ROW_COUNT: usize = crate::sections::player::player_row_count();

/// Os RÓTULOS que a §14 pinta, na ordem da tabela.
///
/// ⚠️ Existe para a cena de smoke poder afirmar que o roteiro dela nomeia um
/// controle que o painel de fato desenha — um roteiro que cita um nome que a UI
/// não usa faz o artista procurar o que não existe.
/// **Todo id que a §14 pinta e que precisa de dica** — os dezenove números e os
/// três botões, na ordem da tabela.
///
/// ⚠️ Exportado pelo mesmo motivo do [`PLAYER_ROW_COUNT`]: sem ele a varredura
/// de dicas iteraria a própria lista que testa. Aqui a lista vem da tabela do
/// pintor, e a contagem é conferida contra ela.
pub fn player_control_ids() -> Vec<ph2d_a11y::NodeId> {
    crate::sections::player::PLAYER_CARDS
        .iter()
        .flat_map(|(_, _, rows)| rows.iter().map(|(_, id, _)| *id))
        .chain(
            crate::sections::player::PLAYER_BUTTON_TIPS
                .iter()
                .map(|(id, _)| *id),
        )
        .collect()
}

/// O passo de um card da §14 ao próximo — a régua que o pintor TEM de usar.
///
/// ⚠️ Exportada para o gate de geometria: ele compara onde as rows de fato
/// caíram contra este passo. Sem ela o gate só poderia afirmar *"as rows estão
/// em ordem"*, que continua verdade quando o pintor avança pela soma das rows e
/// as MOLDURAS passam a se sobrepor.
pub fn player_card_pitch(n_rows: usize) -> f32 {
    crate::sections::rows::card_pitch(n_rows)
}

/// Os cards da §14 e os ids das rows de cada um — o que a varredura de
/// GEOMETRIA precisa para afirmar que a moldura contém o que emoldura.
pub fn player_card_spans() -> Vec<(&'static str, Vec<ph2d_a11y::NodeId>)> {
    crate::sections::player::PLAYER_CARDS
        .iter()
        .map(|(title, _, rows)| (*title, rows.iter().map(|(_, id, _)| *id).collect()))
        .collect()
}

pub fn player_row_labels() -> Vec<&'static str> {
    crate::sections::player::PLAYER_CARDS
        .iter()
        .flat_map(|(_, _, rows)| rows.iter().map(|(label, _, _)| *label))
        .collect()
}

pub use state::{
    InspectorState, last_inspector_content_h, last_inspector_visible_h, set_current_display_unit,
    set_current_inspector_blend, set_current_inspector_joint, set_current_inspector_name,
    set_current_inspector_ordering, set_current_inspector_physics, set_current_inspector_player,
    set_current_inspector_sampling, set_current_inspector_sprite, set_current_inspector_transform,
    set_current_inspector_visibility, set_current_inspector_visibility_section,
    set_current_inspector_wheel,
};

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};

/// Zero-size marker implementing the typed Inspector panel contract.
pub struct InspectorPanel;

impl Panel for InspectorPanel {
    type State = InspectorState;

    const ID: &'static str = "inspector";
    const NODE_ID: NodeId = ids::INSP_PANEL;
    const DEFAULT_VISIBLE: bool = true;

    fn paint(state: &mut InspectorState, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut InspectorState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        event::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        populate::populate(store);
    }
}
