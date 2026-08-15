//! **A timeline rola as propriedades animadas, e a barra aparece quando elas transbordam.**
//!
//! Report do Enio (2026-08-15): *«a timeline está sem o scroll e a barra de scroll para as
//! propriedades animadas»*. Medido antes de qualquer hipótese, o report tinha **duas metades com
//! causas distintas**, e nenhuma delas era geometria (o `content_h`/`scroll_max` estavam certos, e
//! as unidades do `geom_tests` já os provavam):
//!
//! 1. **o scroll existia e estava numa tecla que ninguém descobre** — a roda simples ZOOMAVA o
//!    eixo do tempo, e rolar as linhas era `Shift+roda`. Esta era a única superfície do app em que
//!    a roda sobre o corpo de um painel não rolava o corpo. Curado no `dispatch/scroll.rs`, com a
//!    tabela nova e o preço escritos lá;
//! 2. **a barra existia e era INVISÍVEL** — o polegar era `border` (L 0.295) sobre uma pista
//!    `bg-2` (L 0.185): **ΔL 0.11** numa faixa de 10 px. Curado pela wave dos scrollbars, que a
//!    levou a `border-emph` (ΔL 0.365).
//!
//! ⚠️ **Estes gates correm pela porta do PRODUTO** (`MockPanelHost::paint::<TimelinePanel>`), e a
//! razão é que o defeito irmão desta sessão foi exactamente uma rota certa cujo consumidor deitava
//! fora o que ela entregava. O gate do despachante
//! (`a_plain_wheel_scrolls_the_property_rows_and_consumes`) prova o que ENTRA no store; estes
//! provam o que SAI dele — a costura é o `add_timeline_wheel`/`take_timeline_wheel`, e os dois
//! lados nomeiam a mesma função ([[feedback_layered_defenses_need_per_layer_gates]]).

use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::TimelineViewSnapshot;

use crate::ids;
use crate::state::TimelinePanelState;

/// A janela do smoke: um dock de timeline numa tela de trabalho.
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

fn snap_with(n: usize) -> TimelineViewSnapshot {
    TimelineViewSnapshot {
        tracks: (0..n)
            .map(|i| ph2d_timeline::TrackView {
                target: ph2d_timeline::AnimTarget::new(i as u64),
                prop: ph2d_timeline::PropKind::TranslationX,
                entity: 1,
                missing: false,
                buffer_ghost: None,
                pre: ph2d_timeline::Extrap::Hold,
                post: ph2d_timeline::Extrap::Hold,
                expr: None,
                keys: Vec::new(),
            })
            .collect(),
        ..TimelineViewSnapshot::default()
    }
}

/// Pinta o painel com `n` propriedades animadas, empurra `scroll` px de roda pela porta do store,
/// pinta outra vez (é o segundo paint que drena e aplica) e devolve
/// `(scroll_max, scroll_y, a barra registou?)`.
fn scroll_after_a_wheel(n: usize, scroll: f32) -> (f32, f32, bool) {
    crate::state::set_current_timeline(Some(snap_with(n)));
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<crate::TimelinePanel>();
    let mut state = TimelinePanelState::default();
    let _ = host.paint::<crate::TimelinePanel>(&mut state, VIEWPORT);
    if scroll != 0.0 {
        // O que o despachante põe no store para uma roda SIMPLES: zoom 0, pan 0, scroll = delta.
        host.store_mut()
            .add_timeline_wheel(ids::TIMELINE_PANEL, 0.0, 0.0, scroll, 800.0);
    }
    let hits = host.paint::<crate::TimelinePanel>(&mut state, VIEWPORT);
    let bar = hits.iter().any(|(id, _)| *id == ids::TIMELINE_SCROLLBAR);
    crate::state::set_current_timeline(None);
    (state.scroll_max, state.scroll_y, bar)
}

/// **Uma roda simples move as linhas de propriedade.**
///
/// **Mutação que deve sangrar:** apagar o braço do `scroll_delta` no `view::apply_wheel`, ou o
/// `take_timeline_wheel` do `interact` — a rota continua certa e nada se mexe, que é a forma
/// exacta do defeito que esta sessão já pagou duas vezes.
#[test]
fn a_plain_wheel_scrolls_the_animated_properties() {
    let (max, y, _) = scroll_after_a_wheel(16, -40.0);
    assert!(max > 0.0, "16 propriedades tinham de transbordar o dock");
    assert!(
        y > 0.0,
        "a roda nao moveu as linhas: scroll_max={max}, scroll_y={y}"
    );
}

/// **O CONTROLO: uma lista que CABE não rola.**
///
/// Sem esta metade, uma lei que rolasse sempre satisfaria o gate acima — e a timeline deslizaria
/// para longe das próprias linhas num documento de uma propriedade só.
#[test]
fn a_timeline_whose_rows_fit_has_nothing_to_scroll() {
    let (max, y, bar) = scroll_after_a_wheel(1, -40.0);
    assert_eq!(max, 0.0, "uma propriedade nao devia transbordar");
    assert_eq!(y, 0.0, "a roda moveu uma lista que cabe");
    assert!(!bar, "a barra apareceu sobre uma lista que cabe");
}

/// **A barra aparece exactamente quando há o que rolar** — e é hit-registada, senão é um desenho
/// que não se agarra.
#[test]
fn the_bar_is_there_and_grabbable_exactly_when_the_rows_overflow() {
    for (n, want) in [(1usize, false), (4, true), (40, true)] {
        let (max, _, bar) = scroll_after_a_wheel(n, 0.0);
        assert_eq!(
            bar, want,
            "n={n}: barra={bar} mas scroll_max={max} — o desenho e o alcance discordam"
        );
    }
}

/// **E o polegar não é da mesma cor que a pista.**
///
/// A metade (2) do report. Um gate de tinta e não de token: ele compara o que o pintor produz,
/// então continua a valer no dia em que a paleta se mexer.
#[test]
fn the_thumb_is_not_the_same_colour_as_the_track_it_sits_in() {
    use ph2d_editor_core::widget::{ScrollbarState, scrollbar_thumb_color};
    use ph2d_tokens::{ColorToken, Theme};
    let t = Theme::Forge;
    let track = ph2d_editor_core::paint::resolve(ColorToken::Bg2, t);
    for state in [
        ScrollbarState::Normal,
        ScrollbarState::Hovered,
        ScrollbarState::Dragging,
    ] {
        let thumb = scrollbar_thumb_color((state, ph2d_editor_core::motion::SETTLED), t);
        assert_ne!(
            thumb, track,
            "{state:?}: o polegar pinta a cor da pista — invisivel"
        );
    }
}
