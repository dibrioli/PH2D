//! Os gates do [`super::NumberInput`] — irmão por RESPONSABILIDADE.
//!
//! ⚠️ **Corte forçado pelo tecto de 500 LOC do widget** (2026-09-14, ao pôr a unidade dentro da
//! caixa): o ficheiro foi a `594`. ⛔ A cura é o corte, nunca uma entrada no `FILE_OVERAGE_OK` —
//! e o corte honesto aqui é o mesmo que a `section_cards` já fez: *o que o widget FAZ e o que
//! prova que ele o faz crescem por motivos diferentes.*

use super::*;

/// **An unbounded value reads as the infinity glyph, not a saturated integer or "inf"**
/// (Enio, 2026-07-28: the timeline's Dur box shows ∞ for a `0` = infinite composition,
/// and the box formats the value it is handed). Finite values are untouched. (Mutation:
/// drop the `is_infinite` branch ⇒ `f64::INFINITY as i64` saturates to `i64::MAX`, so the
/// box would read "9223372036854775807", RED.)
#[test]
fn an_infinite_value_reads_as_the_infinity_glyph() {
    assert_eq!(format_number(f64::INFINITY), "\u{221E}");
    assert_eq!(format_number(f64::NEG_INFINITY), "\u{221E}");
    // Finite values are byte-identical to before (the branch is inert off the infinite path).
    assert_eq!(format_number(4.0), "4");
    assert_eq!(format_number(2.5), "2.500");
    assert_eq!(format_number(0.0), "0");
}

#[test]
fn defaults_match_spec() {
    let n = NumberInput::new(NodeId(1), "Width", 10.0);
    assert_eq!(n.value, 10.0);
    assert_eq!(n.step, 1.0);
    assert_eq!(n.min, None);
    assert_eq!(n.max, None);
}

#[test]
fn min_max_clamp_initial_value() {
    let n = NumberInput::new(NodeId(1), "x", -5.0).min(0.0);
    assert_eq!(n.value, 0.0);
    let n = NumberInput::new(NodeId(1), "x", 100.0).max(50.0);
    assert_eq!(n.value, 50.0);
}

#[test]
fn increment_respects_max() {
    let mut n = NumberInput::new(NodeId(1), "x", 9.5).step(1.0).max(10.0);
    n.increment();
    assert_eq!(n.value, 10.0);
    n.increment();
    assert_eq!(n.value, 10.0);
}

#[test]
fn decrement_respects_min() {
    let mut n = NumberInput::new(NodeId(1), "x", 0.5).step(1.0).min(0.0);
    n.decrement();
    assert_eq!(n.value, 0.0);
    n.decrement();
    assert_eq!(n.value, 0.0);
}

#[test]
fn a11y_role_is_number_input_with_value() {
    let n = NumberInput::new(NodeId(1), "x", 7.0).min(0.0).max(10.0);
    let node = n.build_a11y(0.0, 0.0, 100.0, 32.0);
    assert_eq!(node.role(), Role::NumberInput);
    assert_eq!(node.numeric_value(), Some(7.0));
    assert_eq!(node.min_numeric_value(), Some(0.0));
    assert_eq!(node.max_numeric_value(), Some(10.0));
}

#[test]
fn stepper_rects_split_vertically() {
    let n = NumberInput::new(NodeId(1), "x", 0.0);
    let host = Rect::new(0.0, 0.0, 100.0, 32.0);
    let up = n.up_rect(host);
    let down = n.down_rect(host);
    assert!(up.y < down.y);
    assert!((up.h - down.h).abs() < 0.01);
}

fn smoke(n: NumberInput, theme: Theme) {
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    paint_number_input(
        &n,
        Rect::new(0.0, 0.0, 120.0, 32.0),
        &mut scene,
        &mut text,
        theme,
    );
}

#[test]
fn paint_smoke_default() {
    smoke(NumberInput::new(NodeId(1), "x", 42.0), Theme::Forge);
}

#[test]
fn paint_smoke_focused() {
    smoke(
        NumberInput::new(NodeId(1), "x", 1.5).state(TextInputState::Focused),
        Theme::Sunstone,
    );
}

#[test]
fn paint_smoke_disabled() {
    smoke(
        NumberInput::new(NodeId(1), "x", 0.0).state(TextInputState::Disabled),
        Theme::Blueprint,
    );
}

#[test]
fn paint_smoke_error() {
    smoke(
        NumberInput::new(NodeId(1), "x", 99.0).state(TextInputState::Error),
        Theme::Workshop,
    );
}

/// ⭐⭐⭐ **A UNIDADE É TINTA DENTRO DA CAIXA, e não uma segunda caixa.**
///
/// ⛔⛔ **Ordem do dono, 2026-09-14:** a 1.ª entrega punha-a num chip com fundo próprio
/// encostado à borda direita — *«não ficou legal. Melhor junto ao número dentro da caixa»*.
///
/// ⚠️ **A régua é a CENA EMITIDA, e ela tem de dizer as duas coisas de uma vez:** mais
/// GLIFOS (a unidade foi de facto pintada) com o MESMO número de caminhos (não nasceu um
/// segundo rectângulo). Uma delas sozinha aprova o defeito — contar só glifos aprovaria o chip,
/// e contar só caminhos aprovaria uma unidade que não é pintada de todo.
#[test]
fn the_unit_is_ink_inside_the_box_and_never_a_second_box() {
    let medir = |suffix: Option<&'static str>| {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        paint_number_input_with_buffer(
            &NumberInput::new(NodeId(1), "", 1.25).suffix(suffix),
            None,
            0,
            None,
            Rect::new(0.0, 0.0, 160.0, 22.0),
            &mut scene,
            &mut text,
            Theme::Dark,
        );
        let e = scene.inner().encoding();
        (e.resources.glyph_runs.len(), e.n_paths)
    };
    let (glifos_sem, caminhos_sem) = medir(None);
    let (glifos_com, caminhos_com) = medir(Some("m/s"));
    assert!(
        glifos_com > glifos_sem,
        "a unidade nao foi pintada: {glifos_com} corridas de glifo contra {glifos_sem}"
    );
    assert_eq!(
        caminhos_com, caminhos_sem,
        "nasceu um CAMINHO novo com a unidade ({caminhos_com} contra {caminhos_sem}) — \
         isso e' um segundo rectangulo, que e' exactamente o chip que o dono recusou"
    );
}

/// ⚠️ **E a escrever, o campo mostra o que o artista escreveu** — nada mais.
///
/// O parser aceita o sufixo digitado (`"5m/s"`), logo pintar a unidade por cima do buffer
/// faria o texto discordar do que vai ser lido.
#[test]
fn while_typing_the_field_shows_only_what_was_typed() {
    let medir = |state: TextInputState| {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        paint_number_input_with_buffer(
            &NumberInput::new(NodeId(1), "", 1.25)
                .suffix(Some("m"))
                .state(state),
            Some("1.25"),
            4,
            None,
            Rect::new(0.0, 0.0, 160.0, 22.0),
            &mut scene,
            &mut text,
            Theme::Dark,
        );
        scene.inner().encoding().resources.glyph_runs.len()
    };
    assert!(
        medir(TextInputState::Focused) < medir(TextInputState::Normal),
        "o campo focado pintou a unidade por cima do que o artista escreve"
    );
}
