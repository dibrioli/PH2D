//! **A alça de fade do strip existe NA TELA e é CLICÁVEL** (ADR-0115 B4).
//!
//! Este gate pinta o painel de VERDADE (`MockPanelHost::paint` devolve o hit index que a pintura
//! registrou) e pergunta quem ganha o pixel da alça. É a costura que os testes puros do
//! `hit_plan` NÃO cobrem: eles provam a ORDEM de uma lista que alguém precisa construir e
//! entregar — e "alguém constrói e entrega" é exatamente o que pode não acontecer.
//!
//! O Enio achou isso no smoke: os pontinhos rosa apareciam nas pontas do clipe e **não
//! arrastavam**. Pintado ≠ populado, mais uma vez ([[feedback_painted_is_not_populated_paint_gate]]).

use ph2d_editor_core::ids;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_timeline::TimelinePanel;
use ph2d_panel_timeline::state::{TimelinePanelState, set_current_timeline};
use ph2d_panel_timeline::tab::Tab;
use ph2d_timeline::{
    ClipLane, ClipStrip, LaneMode, LaneView, StripId, StripLoop, StripSource, StripView,
    TimelineViewSnapshot,
};
use ph2d_ui_testkit::MockPanelHost;

/// A tela do smoke: uma faixa com UM strip (sozinho — é o caso em que a alça é o único jeito de
/// fazer fade).
fn one_lone_strip() -> TimelineViewSnapshot {
    let mut lane = ClipLane::new("Lane 1");
    lane.insert(ClipStrip::new(StripSource::Clip(0), 1.0, 3.0, 2.0));
    TimelineViewSnapshot {
        fps: 60.0,
        lanes: vec![LaneView {
            name: "Lane 1".into(),
            muted: false,
            weight: 1.0,
            mode: LaneMode::Override,
            strips: vec![StripView {
                id: StripId(1),
                clip_name: "Main".into(),
                container: None,
                t_start: 1.0,
                t_end: 3.0,
                blend_in: lane.blend_in(0),
                blend_out: lane.blend_out(0),
                lead_in: 0.0,
                lead_out: 0.0,
                marks: [0.0; 4],
                ease_locked_in: lane.neighbour_reach_in(0) > 0.0,
                ease_locked_out: lane.neighbour_reach_out(0) > 0.0,
                loop_mode: StripLoop::Once,
                speed: 1.0,
            }],
        }],
        ..TimelineViewSnapshot::default()
    }
}

/// Onde o ponteiro cai — a última registração sobre o ponto é quem ganha o clique.
fn hit_at(
    regs: &[(ph2d_editor_core::NodeId, Rect)],
    x: f32,
    y: f32,
) -> Option<ph2d_editor_core::NodeId> {
    regs.iter()
        .rev()
        .find(|(_, r)| x >= r.x && x < r.x + r.w && y >= r.y && y < r.y + r.h)
        .map(|(id, _)| *id)
}

/// **As duas alças de um strip sozinho são registradas no hit index** — e o clique nelas resolve
/// para a alça, não para o corpo nem para a borda de aparar.
#[test]
fn the_fade_grips_of_a_lone_strip_are_painted_and_clickable() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    // Strips live in the Arrange tab now (ADR-0115 R8, amended): a default panel opens
    // on Keys and would paint no strip at all.
    let mut state = TimelinePanelState {
        tab: Tab::Arrange,
        ..TimelinePanelState::default()
    };
    set_current_timeline(Some(one_lone_strip()));

    let regs = host.paint::<TimelinePanel>(&mut state, Rect::new(0.0, 0.0, 1600.0, 900.0));

    for edge in [3_u8, 4] {
        let id = ids::timeline_strip_hit_id(0, 1, edge);
        let r = regs
            .iter()
            .find(|(w, _)| *w == id)
            .map(|(_, r)| *r)
            .unwrap_or_else(|| {
                panic!(
                    "a alça de fade (edge {edge}) foi PINTADA e não registrada: o clique nela cai \
                     no corpo do strip e o artista arrasta o clipe inteiro"
                )
            });
        assert!(
            r.w > 0.0 && r.h > 0.0,
            "alça com área zero não é clicável: {r:?}"
        );
        // …e ela GANHA o pixel: uma alça registrada que outro retângulo cobre é uma alça morta.
        assert_eq!(
            hit_at(&regs, r.x + r.w * 0.5, r.y + r.h * 0.5),
            Some(id),
            "outra coisa ficou por cima da alça (edge {edge}) — last-wins, e ela não é a última"
        );
    }
}

/// **A alça tem de ser TÃO AGARRÁVEL quanto a borda de aparar** — medido na pintura real.
///
/// O primeiro corte fazia um quadradinho de 7×7 na quina de cima de um strip de 240×22, ao lado
/// de uma borda de aparar de 6×22: **um décimo da área**. O gate de hit passava (o alvo existia e
/// ganhava o pixel) e o Enio não conseguia acertá-lo — o ponteiro caía no corpo e arrastava o
/// clipe. Um alvo que passa no teste e não passa no dedo é um alvo que não existe
/// ([[feedback_ergonomics_verdict_is_a_design_bug]]).
#[test]
fn the_fade_grip_is_as_grabbable_as_the_trim_grip() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    // Strips live in the Arrange tab now (ADR-0115 R8, amended): a default panel opens
    // on Keys and would paint no strip at all.
    let mut state = TimelinePanelState {
        tab: Tab::Arrange,
        ..TimelinePanelState::default()
    };
    set_current_timeline(Some(one_lone_strip()));
    let regs = host.paint::<TimelinePanel>(&mut state, Rect::new(0.0, 0.0, 1600.0, 900.0));

    let rect_of = |edge: u8| {
        let id = ids::timeline_strip_hit_id(0, 1, edge);
        regs.iter()
            .find(|(w, _)| *w == id)
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("nada registrado para a borda {edge}"))
    };
    let trim = rect_of(0);
    for edge in [3_u8, 4] {
        let grip = rect_of(edge);
        assert!(
            grip.h >= trim.h,
            "a alça (edge {edge}) tem {:.0} px de altura contra {:.0} da borda de aparar —              o ponteiro vai cair no corpo e arrastar o clipe",
            grip.h,
            trim.h
        );
        assert!(
            grip.w * grip.h >= trim.w * trim.h,
            "a alça (edge {edge}) tem área menor que a borda de aparar: {:.0}px² vs {:.0}px²",
            grip.w * grip.h,
            trim.w * trim.h
        );
    }
}

/// **O nome do strip nunca pode ser da cor do corpo do strip.**
///
/// O corpo é pintado com `TimelineKey`, que é um token de PRIMEIRO PLANO: ele vale exatamente o
/// mesmo que `Text1` em TODOS os temas (forge: os dois `oklch(0.940 …)`; sunstone: os dois
/// `oklch(0.220 …)`). Escrever o rótulo em `Text1` era escrever branco no branco — o nome só
/// aparecia onde a cunha do fade escurecia o fundo (Enio, smoke).
///
/// O gate compara os tokens RESOLVIDOS, em todos os temas: é a pergunta de verdade ("dá para
/// ler?"), e não "o código cita o token X" — que ficaria verde se alguém trocasse `Text1` por
/// outro apelido do mesmo valor.
#[test]
fn the_strip_name_is_never_the_colour_of_the_strip_body() {
    use ph2d_editor_core::paint::resolve;
    use ph2d_tokens::{ColorToken, Theme};

    for theme in [
        Theme::Forge,
        Theme::Workshop,
        Theme::Sunstone,
        Theme::Blueprint,
    ] {
        let body = resolve(ColorToken::TimelineKey, theme).components;
        let name = resolve(ColorToken::Accent, theme).components; // o que `paint_strip` usa
        assert_ne!(
            body, name,
            "{theme:?}: o nome do strip tem a cor do corpo do strip — invisível"
        );
        // …e a armadilha nomeada: `Text1` É o corpo. Se um dia deixar de ser, este par cai e
        // alguém relê a decisão em vez de herdá-la.
        let text1 = resolve(ColorToken::Text1, theme).components;
        assert_eq!(
            body, text1,
            "{theme:?}: `TimelineKey` deixou de ser igual a `Text1` — reveja a cor do rótulo"
        );
    }
}
