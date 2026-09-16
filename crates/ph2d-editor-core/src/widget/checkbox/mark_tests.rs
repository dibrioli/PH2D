//! Os testes do [`super::mark`] — a geometria e a tinta de uma marca booleana.
//!
//! ⚠️ **Ficheiro irmão pelo tecto de 500 LOC do widget** (2026-09-15, quando a marca ganhou a
//! folga dentro da caixa): é o mesmo molde do `number_input/tests.rs` e do `text_input/tests.rs`.
//! ⛔ *A cura de um tecto estourado é o corte, nunca uma entrada na lista de folgas.*

use super::*;
// ⚠️ O pintor que COMPÕE a marca com o nome vive no irmão desde 2026-09-15 (tecto de LOC) —
//    estes testes exercitam-no de propósito: é ele o caminho do produto.
use super::super::Checkbox;
use super::super::label::paint_checkbox;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::CHECKBOX_BOX_PX as CHROME_CHECKBOX_BOX;

/// ⭐⭐⭐ **A MARCA NUNCA TOCA A CAIXA, E ENCOSTA MAIS À ESQUERDA DO QUE O TEXTO DE UM CAMPO.**
///
/// ⛔⛔ **Report do dono, 2026-09-15, com foto:** *«checkbox ficou maior que a caixa e não foi
/// bem alinhado à esquerda. Godot melhor»*.
///
/// Duas causas, e as duas aritméticas:
///
/// | | antes | agora |
/// |---|---|---|
/// | altura da linha | `18` (o mesmo literal em **treze** secções) | [`ph2d_tokens::ROW_H_PX`] = `22` |
/// | lado da marca | `min(18, 18) = 18` — a caixa TODA | `min(18, 22 − 2×Xs) = 14` |
/// | recuo à esquerda | `field_pad_x` = `12` (onde o VALOR começa) | `Spacing::Xs` = `4` |
///
/// ⚠️ **A segunda metade compara com OUTRA porta** (`field_pad_x`), logo não é a fórmula
/// repetida: ela diz *a marca encosta mais do que o texto*, e a razão é que **um valor precisa
/// de folga para o caret e para a selecção e uma marca não**.
///
/// **Mutação que deve sangrar:** repor `Rect::new(campo.x + field_pad_x(), box_y, box_size,
/// box_size)` — a marca volta a `18` numa caixa de `22` (folga `4`, abaixo dos `8` que dois
/// degraus pedem) e volta a começar em `12`.
#[test]
fn a_marca_nunca_toca_a_caixa_e_encosta_mais_que_o_texto() {
    crate::paint::set_ui_look(ph2d_tokens::UiLook::Redesign);
    let mut scene = VectorScene::new();
    let faixa = Rect::new(0.0, 0.0, 260.0, ph2d_tokens::ROW_H_PX);
    let pintada = paint_boolean_mark(
        faixa,
        BooleanMark {
            value: CheckboxValue::Unchecked,
            state: CheckboxState::Normal,
            hover_t: crate::motion::SETTLED,
            box_px: None,
            decorator: true,
            linha: Some(crate::widget::Seccao::apenas_campos(1)),
        },
        &mut scene,
        Theme::Forge,
    );
    let caixa = pintada
        .caixa
        .expect("uma linha de formulario tem de devolver a CAIXA");
    let folga = caixa.h - pintada.marca.h;
    assert!(
        folga >= 2.0 * ph2d_tokens::Spacing::Xs.px() - 0.01,
        "a marca mede {:.1} numa caixa de {:.1} — folga {folga:.1}, e dois degraus pedem {:.1}",
        pintada.marca.h,
        caixa.h,
        2.0 * ph2d_tokens::Spacing::Xs.px()
    );
    let recuo = pintada.marca.x - caixa.x;
    assert!(
        recuo < crate::widget::field_pad_x(),
        "a marca comeca a {recuo:.1} da borda e o TEXTO de um campo comeca a {:.1}: ela nao              encosta mais a' esquerda do que o valor",
        crate::widget::field_pad_x()
    );
}

/// A caixa de partida dos testes de tinta — um sítio só.
fn fixture() -> Checkbox {
    Checkbox::new(NodeId(1), "Snap to grid")
}

fn smoke(c: Checkbox, theme: Theme) {
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    paint_checkbox(
        &c,
        Rect::new(0.0, 0.0, 200.0, 18.0),
        &mut scene,
        &mut text,
        theme,
    );
}

#[test]
fn half_a_hover_moves_the_unchecked_box_between_the_two_ends() {
    use super::*;
    let theme = Theme::Forge;
    let rest = ColorToken::Bg1.resolve(theme);
    let hot = ColorToken::Bg2.resolve(theme);
    let mid =
        crate::motion::hover_axis(true, 0.5, Some(rest), Some(hot)).expect("o eixo macio mistura");
    assert_ne!(mid, rest);
    assert_ne!(mid, hot);
    // O neutro sai como `None` ⇒ o chamador cai no token DURO.
    assert!(
        crate::motion::hover_axis(true, crate::motion::SETTLED, Some(rest), Some(hot)).is_none()
    );
    // Estado duro (ou caixa MARCADA) não é uma quantidade: fora do eixo.
    assert!(crate::motion::hover_axis(false, 0.5, Some(rest), Some(hot)).is_none());
}

/// **Sem override, a caixa é o TOKEN** — a lei de todo painel do app, ao bit
/// (BUGS_vector #26).
///
/// ⚠️ `box_px: None` não é "um default razoável": é o que faz cada checkbox do app ter
/// exactamente o mesmo tamanho, e é a razão de um formulário ler como formulário. Este gate
/// existe para que mexer nisso exija mexer nele.
#[test]
fn without_an_override_the_box_is_the_token() {
    let c = fixture();
    assert_eq!(c.box_px, None, "o default deixou de ser o token");

    let tall = Rect::new(0.0, 0.0, 200.0, CHROME_CHECKBOX_BOX * 8.0);
    let mut a = VectorScene::new();
    let mut ts = TextSystem::without_system_fonts();
    paint_checkbox(&c, tall, &mut a, &mut ts, Theme::Forge);

    let mut explicit = c.clone();
    explicit.box_px = Some(CHROME_CHECKBOX_BOX);
    let mut b = VectorScene::new();
    paint_checkbox(&explicit, tall, &mut b, &mut ts, Theme::Forge);

    let (ea, eb) = (a.inner().encoding(), b.inner().encoding());
    assert_eq!(
        (ea.n_paths, ea.path_data.clone()),
        (eb.n_paths, eb.path_data.clone()),
        "pedir o proprio token divergiu de nao pedir nada — o canal nao e' neutro"
    );
}

#[test]
fn paint_smoke_normal_unchecked() {
    smoke(fixture(), Theme::Forge);
}

#[test]
fn paint_smoke_hovered_checked() {
    smoke(
        fixture()
            .value(CheckboxValue::Checked)
            .state(CheckboxState::Hovered),
        Theme::Sunstone,
    );
}

#[test]
fn paint_smoke_pressed_indeterminate() {
    smoke(
        fixture()
            .value(CheckboxValue::Indeterminate)
            .state(CheckboxState::Pressed),
        Theme::Blueprint,
    );
}

#[test]
fn paint_smoke_focused_unchecked() {
    smoke(fixture().state(CheckboxState::Focused), Theme::Workshop);
}

#[test]
fn paint_smoke_disabled_checked() {
    smoke(
        fixture()
            .value(CheckboxValue::Checked)
            .state(CheckboxState::Disabled),
        Theme::Forge,
    );
}
