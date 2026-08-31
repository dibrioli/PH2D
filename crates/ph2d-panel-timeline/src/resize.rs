//! Chrome resizes of the timeline panel: its own edges, the track-name column,
//! and the height of the expanded graph bands. Split from `view` (the time-axis
//! camera) under the HR-18 panel LOC cap.
//!
//! All three share one shape: capture the value and the pointer at Begin, apply
//! the delta to THAT on every Update, and let the port clamp the result. Applying
//! deltas to the live value instead would accumulate rounding across a slow drag.
//! None of them raise intents — none is undoable.
//!
//! ⚠️ **A terceira deixou de escrever um rect e passou a escrever uma MEDIDA** (2026-08-31) — ver
//! [`apply_resize`].

use ph2d_editor_core::interaction::{GesturePhase, TimelineGesture};
use ph2d_editor_core::zones::Rect;

use crate::geom;
use crate::state::{ResizeDrag, TimelinePanelState};

/// Splitter drag: widen or narrow the track-name column. The width applies to
/// the one captured at Begin (no drift), and `paint` clamps it into the panel.
pub(crate) fn apply_label_drag(state: &mut TimelinePanelState, g: TimelineGesture) {
    match g.phase {
        GesturePhase::Begin => state.label_drag = Some((state.label_w, g.x)),
        GesturePhase::Update => {
            if let Some((w0, x0)) = state.label_drag {
                state.label_w = w0 + (g.x - x0);
            }
        }
        _ => state.label_drag = None,
    }
}

/// Graph-band grip drag: taller or shorter curves. Applies to the height captured
/// at Begin (no drift); `paint` clamps it.
pub(crate) fn apply_graph_resize(state: &mut TimelinePanelState, g: TimelineGesture) {
    match g.phase {
        GesturePhase::Begin => state.graph_resize = Some((state.graph_h, g.y)),
        GesturePhase::Update => {
            if let Some((h0, y0)) = state.graph_resize {
                state.graph_h = h0 + (g.y - y0);
            }
        }
        _ => state.graph_resize = None,
    }
}

/// ⭐⭐ **A COSTURA DO TOPO: arrastar a borda muda a ALTURA DA BANDA, nunca solta o painel dela.**
///
/// Enio, 2026-08-31, com foto e duas setas: *«em nodes, arrastar a timeline na vertical deve
/// ajustar o canvas dos nós e não deixar espaços vazios nem sobrepor os nodes»*.
///
/// ⛔⛔ **O que ele arrastava não era uma banda — era o painel a soltar-se dela.** Esta função
/// escrevia `state.rect`, um rect LIVRE, e a partir do primeiro arrasto o painel deixava de ler a
/// faixa que o layout lhe dava: daí o espaço vazio por cima dele (a faixa ficava onde estava, o
/// painel foi-se embora) e a sobreposição no sentido contrário. *Uma borda de painel docado que
/// devolve um rect livre é um painel que deixa de estar docado quando se lhe toca.*
///
/// ⇒ hoje ela escreve **a medida** (`WidgetStore::set_dock_bottom_h`), como a borda interior de
/// uma coluna. Quem partilha a banda — o grafo de nós, via `HeroLayout::dock_timeline_into_motion`
/// — segue **por construção**, e não por uma segunda conta que possa discordar.
///
/// ⚠️ **Só o TOPO agarra**, e as outras três bordas desapareceram (ver `geom::resize_grips`): numa
/// faixa docada elas são inexprimíveis — os lados são as costuras das colunas, e o fundo é a borda
/// da janela.
///
/// ⚠️ A altura sai da captura do Begin, e não do valor vivo: aplicar deltas ao vivo acumula
/// arredondamento ao longo de um arrasto lento (é a lei das outras duas funções deste ficheiro).
pub(crate) fn apply_resize(
    state: &mut TimelinePanelState,
    store: &mut ph2d_editor_core::interaction::WidgetStore,
    rect: Rect,
    edges: u8,
    g: TimelineGesture,
) {
    if edges & geom::EDGE_T == 0 {
        return; // uma borda que já não existe; ver o doc acima
    }
    match g.phase {
        GesturePhase::Begin => {
            state.resize = Some(ResizeDrag {
                start_h: rect.h,
                start_y: g.y,
            });
        }
        GesturePhase::Update => {
            if let Some(d) = state.resize {
                // Puxar para CIMA (dy negativo) faz a faixa crescer — ela está ancorada no fundo.
                store.set_dock_bottom_h(d.start_h - (g.y - d.start_y));
            }
        }
        _ => state.resize = None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_editor_core::interaction::{GestureMods, TimelineHitKind};
    use ph2d_host::PointerButton;

    fn drag(button: PointerButton, phase: GesturePhase, x: f32, y: f32) -> TimelineGesture {
        TimelineGesture {
            surface: ph2d_a11y::NodeId(0),
            kind: TimelineHitKind::Lane,
            phase,
            x,
            y,
            button,
            mods: GestureMods::default(),
        }
    }

    #[test]
    fn dragging_the_splitter_widens_the_label_column() {
        let mut st = TimelinePanelState {
            label_w: 132.0,
            ..TimelinePanelState::default()
        };
        apply_label_drag(
            &mut st,
            drag(PointerButton::Primary, GesturePhase::Begin, 200.0, 0.0),
        );
        apply_label_drag(
            &mut st,
            drag(PointerButton::Primary, GesturePhase::Update, 260.0, 0.0),
        );
        assert_eq!(st.label_w, 192.0);
        // A second Update still measures from Begin, never from the live width.
        apply_label_drag(
            &mut st,
            drag(PointerButton::Primary, GesturePhase::Update, 150.0, 0.0),
        );
        assert_eq!(st.label_w, 82.0, "no drift accumulation");
        apply_label_drag(
            &mut st,
            drag(PointerButton::Primary, GesturePhase::End, 150.0, 0.0),
        );
        assert!(st.label_drag.is_none());
    }

    #[test]
    fn an_update_without_a_begin_moves_nothing() {
        let mut st = TimelinePanelState::default();
        let before = st.label_w;
        apply_label_drag(
            &mut st,
            drag(PointerButton::Primary, GesturePhase::Update, 900.0, 0.0),
        );
        assert_eq!(st.label_w, before);
    }

    #[test]
    fn dragging_the_graph_grip_resizes_every_expanded_band() {
        let mut st = TimelinePanelState {
            graph_h: 132.0,
            ..TimelinePanelState::default()
        };
        apply_graph_resize(
            &mut st,
            drag(PointerButton::Primary, GesturePhase::Begin, 0.0, 400.0),
        );
        apply_graph_resize(
            &mut st,
            drag(PointerButton::Primary, GesturePhase::Update, 0.0, 460.0),
        );
        assert_eq!(st.graph_h, 192.0);
        // Measured from Begin, never from the live height.
        apply_graph_resize(
            &mut st,
            drag(PointerButton::Primary, GesturePhase::Update, 0.0, 380.0),
        );
        assert_eq!(st.graph_h, 112.0, "no drift accumulation");
        apply_graph_resize(
            &mut st,
            drag(PointerButton::Primary, GesturePhase::End, 0.0, 380.0),
        );
        assert!(st.graph_resize.is_none());
    }

    #[test]
    fn the_graph_height_stays_between_its_bounds() {
        use crate::graph::clamp_graph_h;
        assert_eq!(clamp_graph_h(200.0), 200.0);
        assert!(clamp_graph_h(-500.0) > 0.0, "a band is never inverted");
        assert!(clamp_graph_h(10_000.0) < 10_000.0, "and never unbounded");
    }

    /// ⭐⭐ **A costura do topo escreve a MEDIDA da banda** — e a medida é a única coisa que ela
    /// escreve (o painel não tem rect próprio desde 2026-08-31).
    #[test]
    fn dragging_the_top_edge_writes_the_band_height_captured_at_begin() {
        let mut st = TimelinePanelState::default();
        let mut store = ph2d_editor_core::interaction::WidgetStore::default();
        let band = Rect::new(100.0, 600.0, 800.0, 240.0);
        let g = |phase, y| drag(PointerButton::Primary, phase, 400.0, y);

        assert_eq!(
            store.dock_bottom_h_choice(),
            None,
            "controlo: já havia uma escolha e o gate mediria a de outro"
        );
        apply_resize(
            &mut st,
            &mut store,
            band,
            geom::EDGE_T,
            g(GesturePhase::Begin, 600.0),
        );
        apply_resize(
            &mut st,
            &mut store,
            band,
            geom::EDGE_T,
            g(GesturePhase::Update, 550.0),
        );
        assert_eq!(store.dock_bottom_h(), 290.0, "puxar para cima faz crescer");

        // Um segundo Update mede a partir do Begin, nunca do valor vivo.
        apply_resize(
            &mut st,
            &mut store,
            band,
            geom::EDGE_T,
            g(GesturePhase::Update, 500.0),
        );
        assert_eq!(store.dock_bottom_h(), 340.0, "sem acumular arredondamento");

        apply_resize(
            &mut st,
            &mut store,
            band,
            geom::EDGE_T,
            g(GesturePhase::End, 500.0),
        );
        assert!(st.resize.is_none());
    }

    /// ⛔ **As outras três bordas já não existem, e uma que chegue é ignorada.**
    #[test]
    fn no_other_edge_moves_the_band() {
        let mut st = TimelinePanelState::default();
        let mut store = ph2d_editor_core::interaction::WidgetStore::default();
        let band = Rect::new(100.0, 600.0, 800.0, 240.0);
        // ⚠️ Vindos do editor-core, e não do `geom`: este painel já não os re-exporta (ele não os
        // oferece), e o gate mede exactamente isso — uma borda que chegue de fora é ignorada.
        use ph2d_editor_core::interaction::{TIMELINE_EDGE_B, TIMELINE_EDGE_L, TIMELINE_EDGE_R};
        for edges in [TIMELINE_EDGE_B, TIMELINE_EDGE_L, TIMELINE_EDGE_R] {
            apply_resize(
                &mut st,
                &mut store,
                band,
                edges,
                drag(PointerButton::Primary, GesturePhase::Begin, 400.0, 600.0),
            );
            apply_resize(
                &mut st,
                &mut store,
                band,
                edges,
                drag(PointerButton::Primary, GesturePhase::Update, 400.0, 400.0),
            );
        }
        assert_eq!(
            store.dock_bottom_h_choice(),
            None,
            "uma borda que já não é oferecida moveu a banda"
        );
    }

    /// ⭐ **E a faixa tem piso** — abaixo dele não sobra borda para a agarrar de volta.
    #[test]
    fn the_band_never_shrinks_past_its_floor() {
        let mut st = TimelinePanelState::default();
        let mut store = ph2d_editor_core::interaction::WidgetStore::default();
        let band = Rect::new(100.0, 600.0, 800.0, 240.0);
        apply_resize(
            &mut st,
            &mut store,
            band,
            geom::EDGE_T,
            drag(PointerButton::Primary, GesturePhase::Begin, 400.0, 600.0),
        );
        apply_resize(
            &mut st,
            &mut store,
            band,
            geom::EDGE_T,
            drag(PointerButton::Primary, GesturePhase::Update, 400.0, 5000.0),
        );
        assert!(
            store.dock_bottom_h() >= geom::MIN_H,
            "a faixa encolheu para {} — abaixo de {} o cabeçalho e o transporte não cabem",
            store.dock_bottom_h(),
            geom::MIN_H
        );
    }
}
