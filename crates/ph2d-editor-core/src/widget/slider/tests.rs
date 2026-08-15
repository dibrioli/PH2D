//! Gates do [`super::Slider`] — a lei da TRILHA e a reacção ao ponteiro.
//!
//! ⚠️ **Ficheiro IRMÃO, e a forma é o precedente do `tool_rail`/`command_palette`:** um
//! `slider_tests.rs` solto em `src/widget/` seria varrido pelo `ph2d-widget-sync` como se
//! fosse um WIDGET (o gerador lê os `*.rs` do topo), e nasceria sem showcase e sem a11y. O
//! par `slider.rs` + `slider/tests.rs` deduplica para um `mod` só.

use super::*;

/// **A rota do PAINEL é a lei que shipava, verbatim** (BUGS_vector #26).
///
/// ⚠️ `None` não é "um default razoável": é a política de LINHA — 25% com piso E teto — e
/// mudá-la re-dimensionaria todo slider do app. O gate a escreve por extenso, para que
/// alterá-la exija alterar as duas coisas.
#[test]
fn without_an_override_the_track_is_the_panel_law() {
    for across in [0.0, 4.0, 8.0, 28.0, 32.0, 96.0, 400.0] {
        assert_eq!(
            track_thickness(None, across),
            (across * 0.25).clamp(TRACK_MIN_PX, TRACK_MAX_PX), // CLAMP-OK: mirrors the law under test
            "a politica de linha mudou em across={across}"
        );
    }
}

/// **O TETO é do painel; o PISO é do pintor.** Quem informa a espessura escapa ao teto de
/// linha — e não escapa ao piso, porque uma trilha abaixo dele é invisível seja quem for o
/// chamador.
#[test]
fn an_override_escapes_the_ceiling_but_never_the_floor() {
    assert_eq!(track_thickness(Some(40.0), 160.0), 40.0);
    assert!(
        track_thickness(None, 160.0) < 40.0,
        "a fixture nao contem o fenomeno: o teto de linha nao mordeu em across=160"
    );
    assert_eq!(track_thickness(Some(0.0), 160.0), TRACK_MIN_PX);
    assert_eq!(track_thickness(Some(-3.0), 160.0), TRACK_MIN_PX);
}

/// **No ponto de operação do painel os dois caminhos COINCIDEM.** É esta igualdade que faz
/// da pele de canvas uma continuação da lei, e não uma segunda lei.
#[test]
fn at_a_panel_row_the_override_and_the_law_agree() {
    let row = ph2d_tokens::ROW_H_PX;
    assert_eq!(
        track_thickness(Some(row * 0.25), row),
        track_thickness(None, row)
    );
}

fn fixture() -> Slider {
    Slider::new(NodeId(1), "Opacity")
}

#[test]
fn defaults_match_spec() {
    let s = fixture();
    assert_eq!(s.id, NodeId(1));
    assert_eq!(s.label, "Opacity");
    assert!((s.value - 0.5).abs() < f32::EPSILON);
    assert_eq!(s.state, SliderState::Normal);
    assert_eq!(s.orientation, SliderOrientation::Horizontal);
    assert!(!s.accent);
    assert!(s.ticks.is_empty());
}

#[test]
fn set_value_clamps_below_zero() {
    let mut s = fixture();
    s.set_value(-0.5);
    assert_eq!(s.value, 0.0);
}

#[test]
fn set_value_clamps_above_one() {
    let mut s = fixture();
    s.set_value(1.5);
    assert_eq!(s.value, 1.0);
}

#[test]
fn ticks_setter_round_trips() {
    let s = fixture().ticks(vec![0.0, 0.25, 0.5, 0.75, 1.0]);
    assert_eq!(s.ticks.len(), 5);
}

#[test]
fn a11y_node_has_slider_role_and_value() {
    let s = fixture();
    let node = s.build_a11y(0.0, 0.0, 100.0, 30.0);
    assert_eq!(node.role(), Role::Slider);
    assert_eq!(node.label(), Some("Opacity"));
    assert_eq!(node.numeric_value(), Some(0.5));
    assert_eq!(node.min_numeric_value(), Some(0.0));
    assert_eq!(node.max_numeric_value(), Some(1.0));
}

fn smoke(slider: Slider, rect: Rect, theme: Theme) {
    let mut scene = VectorScene::new();
    paint_slider(&slider, rect, &mut scene, theme);
}

#[test]
fn paint_smoke_horizontal_default() {
    smoke(fixture(), Rect::new(0.0, 0.0, 200.0, 24.0), Theme::Forge);
}

#[test]
fn paint_smoke_horizontal_zero() {
    let mut s = fixture();
    s.set_value(0.0);
    smoke(s, Rect::new(0.0, 0.0, 200.0, 24.0), Theme::Forge);
}

#[test]
fn paint_smoke_horizontal_one() {
    let mut s = fixture();
    s.set_value(1.0);
    smoke(s, Rect::new(0.0, 0.0, 200.0, 24.0), Theme::Sunstone);
}

#[test]
fn paint_smoke_vertical_half() {
    smoke(
        fixture().orientation(SliderOrientation::Vertical),
        Rect::new(0.0, 0.0, 24.0, 200.0),
        Theme::Blueprint,
    );
}

#[test]
fn paint_smoke_dragging_with_ticks() {
    let s = fixture()
        .accent(true)
        .ticks(vec![0.0, 0.25, 0.5, 0.75, 1.0])
        .state(SliderState::Dragging);
    smoke(s, Rect::new(0.0, 0.0, 200.0, 24.0), Theme::Forge);
}

#[test]
fn paint_smoke_focused_draws_ring() {
    smoke(
        fixture().state(SliderState::Focused),
        Rect::new(0.0, 0.0, 200.0, 24.0),
        Theme::Workshop,
    );
}

#[test]
fn paint_smoke_disabled() {
    smoke(
        fixture().state(SliderState::Disabled),
        Rect::new(0.0, 0.0, 200.0, 24.0),
        Theme::Forge,
    );
}

// ---------------------------------------------------------------------------
// A TRILHA REAGE AO PONTEIRO
// ---------------------------------------------------------------------------
//
// ⚠️ **O oráculo é a TINTA, não o campo.** O defeito desta wave vivia um andar abaixo do store:
// o despachante escrevia `Hovered`/`Dragging`, a struct carregava-os, e o `paint_slider` **deitava-
// os fora** — só `Focused` e `Disabled` chegavam ao desenho. Um gate que lesse `slider.state`
// ficaria verde sobre exactamente esse produto, porque o campo estava certo. Por isso estes gates
// PINTAM e comparam a cena.

/// A tinta que o `paint_slider` produz para um par visual dado.
fn ink(visual: (SliderState, f32)) -> (Vec<u32>, Vec<u32>) {
    let mut scene = ph2d_vector::VectorScene::new();
    let s = Slider::new(NodeId(1), "x").visual(visual);
    paint_slider(
        &s,
        Rect::new(0.0, 0.0, 200.0, 28.0),
        &mut scene,
        Theme::Forge,
    );
    let e = scene.inner().encoding();
    (e.path_data.clone(), e.draw_data.clone())
}

/// **A trilha REAGE ao ponteiro — e antes desta wave não reagia.**
///
/// ⚠️ As três superfícies são DISTINTAS entre si, e isso é mais forte que «hover ≠ repouso»: um
/// pintor que colapsasse `Dragging` em `Hovered` passaria num par de `assert_ne!` contra o
/// repouso e deixaria o artista sem saber que a trilha está sob a mão dele.
///
/// **Mutação que deve sangrar:** `track_tint`/`fill_tint` devolverem o token de repouso para todo
/// estado — a lei que shipava.
#[test]
fn the_track_reacts_to_the_pointer() {
    let normal = ink((SliderState::Normal, crate::motion::SETTLED));
    let hovered = ink((SliderState::Hovered, crate::motion::SETTLED));
    let dragging = ink((SliderState::Dragging, crate::motion::SETTLED));
    assert_ne!(
        hovered, normal,
        "hover pinta igual ao repouso: o pintor descarta o estado"
    );
    assert_ne!(dragging, normal, "arrastar pinta igual ao repouso");
    assert_ne!(
        dragging, hovered,
        "arrastar e passar por cima sao a MESMA tinta: a superficie sob a mao nao se distingue"
    );
}

/// **O neutro é o mundo COMO ERA** — nas duas metades de que isso é feito.
///
/// ⚠️ **Metade 1, a FRONTEIRA:** `SETTLED` e o `t` mais baixo têm de pintar a MESMA coisa em
/// repouso. Se não pintassem, haveria um DEGRAU no instante em que o relógio desiste de um id, e
/// ele apareceria como um piscar em toda linha de painel que o rato acabou de deixar.
///
/// ⚠️ **Metade 2, os TOKENS de repouso, escritos por extenso** — `Bg2` na trilha, `Accent` no
/// preenchimento, exactamente a lei que shipava antes desta wave. Escrever o literal é o que
/// impede a coisa que um oráculo derivado não impede: uma escada de tons nova que tingisse TODO
/// slider do app no arranque passaria numa comparação contra as próprias funções.
///
/// **Mutação que deve sangrar:** `track_tint` devolver `Bg3` em repouso (a linha inteira do app
/// nasceria acesa).
#[test]
fn the_neutral_is_the_world_as_it_was() {
    assert_eq!(
        ink((SliderState::Normal, crate::motion::SETTLED)),
        ink((SliderState::Normal, 0.0)),
        "ha um degrau na fronteira do neutro: o painel pisca quando o relogio larga o id"
    );
    assert_eq!(track_tint(SliderState::Normal), ColorToken::Bg2);
    assert_eq!(fill_tint(SliderState::Normal), ColorToken::Accent);
}

/// **SAIR do hover anima — e é a metade que o estado sozinho não consegue exprimir.**
///
/// ⚠️ A lei é `soft = matches!(state, Normal | Hovered)`: com o dedo a sair, o estado já voltou a
/// `Normal` e só o `t` a descer conta a história. Se `Normal` não fosse macio, a saída seria um
/// CORTE e o gate mede exactamente isso — `normal@0.5` teria de ser a tinta de repouso.
///
/// **Mutação que deve sangrar:** tirar `Normal` de `soft`.
#[test]
fn leaving_the_hover_animates() {
    let mid_in = ink((SliderState::Hovered, 0.5));
    let mid_out = ink((SliderState::Normal, 0.5));
    assert_eq!(
        mid_out, mid_in,
        "a meio caminho o `t` tem de escolher a cor; se o ESTADO escolher, sair do hover e um corte"
    );
    assert_ne!(mid_in, ink((SliderState::Normal, crate::motion::SETTLED)));
    assert_ne!(mid_in, ink((SliderState::Hovered, crate::motion::SETTLED)));
}

/// **`Dragging` é estado DURO — não é meia quantidade.**
///
/// ⚠️ Um arrasto ou está a acontecer ou não; interpolar para ele desenharia uma superfície
/// «meio agarrada» que nenhum gesto produz. O gate afirma que o `t` não o move.
#[test]
fn dragging_is_a_hard_state() {
    assert_eq!(
        ink((SliderState::Dragging, 0.0)),
        ink((SliderState::Dragging, crate::motion::SETTLED)),
        "o `t` mexeu num estado duro"
    );
}
