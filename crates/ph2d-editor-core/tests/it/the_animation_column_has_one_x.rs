//! ⭐⭐⭐ **A coluna de animação tem UM `x`, seja qual for a família de linha.**
//!
//! Report do Enio (2026-09-03, com foto): *«várias não receberam pontos»* — as linhas do Transform
//! não são a caixa única nem a linha de verificação; são **construídas à mão dentro do painel**,
//! com a sua própria aritmética de larguras. E há ~20 construtores desses só no Inspector.
//!
//! ⛔ **O risco que este gate cobre não é «esquecer o ponto» — é DESENHÁ-LO EM SÍTIOS DIFERENTES.**
//! Vinte subtracções de `DECORATOR_W` são vinte oportunidades de a coluna ficar com um `x` por
//! painel, e uma coluna com dois `x` é pior que nenhuma: ela lê-se como desalinho, não como coluna.
//!
//! ⇒ a lei vive numa porta ([`form_row_columns`]) e este gate afirma o que ela garante.

use ph2d_editor_core::widget::{DECORATOR_W, form_row_columns, surface_rect};

/// ⚠️ **Este ficheiro afirma sobre o REDESENHO**, que desde 2026-09-03 é opcional. A porta devolve
/// a linha INTEIRA e uma coluna de zero px no clássico — medi-la sem escolher a aparência seria
/// medir a UI de sempre com o nome da coluna.
fn redesign() {
    ph2d_editor_core::paint::set_ui_look(ph2d_tokens::UiLook::Redesign);
}

/// **As três famílias põem o ponto no MESMO `x`.**
///
/// A caixa única recua a superfície por [`surface_rect`]; a linha à mão pergunta à
/// [`form_row_columns`]. Se as duas leis divergirem, a coluna parte-se ao meio do painel.
///
/// **Mutação que deve sangrar:** dar à porta uma margem própria (`w - DECORATOR_W - pad`) — a linha
/// à mão passa a pôr o ponto mais à esquerda que a caixa única, e o formulário fica com duas
/// colunas.
#[test]
fn the_hand_rolled_row_and_the_unified_box_agree_on_the_column() {
    redesign();
    for w in [120.0_f32, 184.0, 240.0, 268.0, 400.0] {
        let x = 37.0_f32;
        let (control_w, dot) = form_row_columns(x, w, 0.0, 22.0);
        let surface = surface_rect(
            ph2d_editor_core::zones::Rect::new(x, 0.0, w, 22.0),
            ph2d_editor_core::widget::FORM_ROWS_SHOW_DECORATOR,
        );
        assert!(
            (dot.x - (surface.x + surface.w)).abs() < 0.001,
            "a {w} px o ponto da linha a' mao cai em {} e a caixa unica acaba em {}: a coluna \
             partiu-se ao meio do painel",
            dot.x,
            surface.x + surface.w
        );
        assert!(
            (control_w - surface.w).abs() < 0.001,
            "as duas familias reservam larguras diferentes ({control_w} contra {}): uma delas \
             desenha por cima da coluna",
            surface.w
        );
    }
}

/// **A coluna encosta à direita e mede exactamente [`DECORATOR_W`].**
#[test]
fn the_column_is_flush_right_and_exactly_one_column_wide() {
    redesign();
    let (control_w, dot) = form_row_columns(10.0, 200.0, 5.0, 22.0);
    assert!(
        (dot.x + dot.w - 210.0).abs() < 0.001,
        "a coluna nao encosta a' direita"
    );
    assert!(
        (dot.w - DECORATOR_W).abs() < 0.001,
        "a coluna nao mede DECORATOR_W"
    );
    assert!(
        (control_w + dot.w - 200.0).abs() < 0.001,
        "os controlos e a coluna nao somam a largura da linha: ou sobra um vazio, ou sobrepoem-se"
    );
}

/// ⛔ **Uma linha estreita de mais não devolve largura NEGATIVA.**
///
/// ⚠️ Um `control_w` negativo propaga-se por dentro da aritmética de cada painel — larguras de
/// chip, posições de rótulo — e o sintoma aparece longe daqui, como texto desenhado à esquerda do
/// painel.
#[test]
fn a_row_narrower_than_the_column_still_returns_a_usable_width() {
    redesign();
    for w in [0.0_f32, 1.0, DECORATOR_W, DECORATOR_W + 0.5] {
        let (control_w, _) = form_row_columns(0.0, w, 0.0, 22.0);
        assert!(
            control_w >= 1.0,
            "a {w} px a porta devolveu {control_w}: uma largura nao-positiva propaga-se para \
             dentro da aritmetica do painel e reaparece longe daqui"
        );
    }
}
