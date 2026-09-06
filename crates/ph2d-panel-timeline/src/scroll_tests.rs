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
///
/// ⚠️⚠️ **A contagem que transborda é PROCURADA, não escrita** (2026-09-06): a redacção anterior
/// afirmava que `4` linhas transbordavam, e isso deixou de ser verdade no dia em que o dono pediu
/// linhas mais baixas (`24 → 22 px`) — *uma fixtura que nomeia um número descreve a árvore do dia
/// em que foi escrita.* O que este gate defende é o FENÓMENO: existe uma contagem a partir da
/// qual a barra aparece, e uma abaixo dela em que não aparece.
#[test]
fn the_bar_is_there_and_grabbable_exactly_when_the_rows_overflow() {
    let first_overflow = (1usize..64)
        .find(|n| scroll_after_a_wheel(*n, 0.0).2)
        .expect("nenhuma contagem ate' 64 transborda — a fixture perdeu o fenomeno");
    assert!(
        first_overflow > 1,
        "uma lista de UMA linha nao devia transbordar"
    );
    let (max_below, _, bar_below) = scroll_after_a_wheel(first_overflow - 1, 0.0);
    assert!(
        !bar_below && max_below == 0.0,
        "n={}: a barra apareceu logo abaixo do limiar",
        first_overflow - 1
    );
    let (max_over, _, bar_over) = scroll_after_a_wheel(first_overflow, 0.0);
    assert!(
        bar_over && max_over > 0.0,
        "n={first_overflow}: o desenho e o alcance discordam no limiar"
    );
    assert!(
        scroll_after_a_wheel(40, 0.0).2,
        "uma lista bem maior que a janela tem de continuar a oferecer a barra"
    );
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

// ── A lista do "+ Track" cabe na tela ────────────────────────────────────────

/// Abre o "+ Track" e devolve quantas das linhas da tabela ficaram alcançáveis (hit-registadas)
/// numa janela de `vh` px de altura, e se alguma delas caiu FORA da janela.
fn addprop_rows_reachable(vh: f32) -> (usize, bool) {
    crate::state::set_current_timeline(Some(snap_with(3)));
    let viewport = Rect::new(0.0, 0.0, 1600.0, vh);
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<crate::TimelinePanel>();
    let mut state = TimelinePanelState::default();
    // Abre a lista pela porta do produto — o clique no botão "+ Track".
    let _ = host.paint::<crate::TimelinePanel>(&mut state, viewport);
    let _ = host.apply_panel_event::<crate::TimelinePanel>(
        &mut state,
        ph2d_editor_core::interaction::WidgetEvent::Click(ids::TIMELINE_ADD_TRACK),
    );
    let hits = host.paint::<crate::TimelinePanel>(&mut state, viewport);
    let rows: Vec<_> = hits
        .iter()
        .filter(|(id, _)| ids::ADDPROP_BUTTONS.iter().any(|(b, _)| b == id))
        .collect();
    let outside = rows
        .iter()
        .any(|(_, r)| r.y < viewport.y || r.y + r.h > viewport.y + viewport.h);
    crate::state::set_current_timeline(None);
    (rows.len(), outside)
}

/// **Numa janela baixa a lista DESLIZA para dentro, e as treze continuam alcançáveis.**
///
/// A janela da foto do Enio (2026-08-15) tinha ~500 px: a lista larga-se do botão a meia altura,
/// `13 x 28 = 364 px` não cabem abaixo dele, e as últimas cinco — Morph e as quatro do joint —
/// acabavam no rodapé do ecrã, **pintadas e hit-registadas fora da janela**.
///
/// **Mutação que deve sangrar:** voltar a `Rect::new(anchor.x, anchor.y + anchor.h, ..)`, o
/// largar-direto-para-baixo.
#[test]
fn the_add_track_list_slides_into_a_short_window_and_stays_reachable() {
    let (n, outside) = addprop_rows_reachable(500.0);
    assert!(!outside, "uma linha do +Track ficou FORA da janela");
    assert_eq!(
        n,
        ids::ADDPROP_BUTTONS.len(),
        "so {n} das {} propriedades sao alcancaveis numa janela de 500 px",
        ids::ADDPROP_BUTTONS.len()
    );
}

/// **O CONTROLO: numa janela alta nada muda** — a lista continua a cair do botão para baixo, e as
/// treze estão lá. Sem esta metade, «não pintar nada» satisfaria o gate acima.
#[test]
fn a_tall_window_still_shows_every_property() {
    let (n, outside) = addprop_rows_reachable(1400.0);
    assert!(!outside);
    assert_eq!(n, ids::ADDPROP_BUTTONS.len());
}

/// **E uma janela mais baixa que a própria lista não oferece o que não alcança.**
///
/// O resto honesto: com 300 px não há deslize que ponha 364 px na tela. O que fica de fora tem de
/// ficar de fora **inteiro** — nem pintado nem registado —, porque uma linha viva onde o ponteiro
/// não chega conta como oferta e não é uma.
///
/// ⚠️⚠️ **A janela é PROCURADA, não escrita** — pela mesma razão da irmã acima: os `300 px` da
/// redacção anterior deixaram de cortar a lista quando a linha desceu para `22 px`.
#[test]
fn a_window_shorter_than_the_list_offers_only_what_it_can_reach() {
    let total = ids::ADDPROP_BUTTONS.len();
    let mut vh = 1400.0_f32;
    let (mut n, mut outside) = addprop_rows_reachable(vh);
    while n >= total && vh > 32.0 {
        vh *= 0.5;
        let r = addprop_rows_reachable(vh);
        n = r.0;
        outside = r.1;
    }
    assert!(
        n < total,
        "nenhuma janela ate' 32 px cortou as {total} linhas — a fixture perdeu o fenomeno"
    );
    assert!(!outside, "linha registada fora de uma janela de {vh} px");
    assert!(n > 0, "a lista desapareceu por inteiro");
}
