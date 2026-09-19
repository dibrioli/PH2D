//! **A FILEIRA QUE CARREGA UMA QUEIXA** — a única row do painel de params que pode estar ERRADA.
//!
//! ⚠️ **Irmã por RESPONSABILIDADE do [`super`]** (tecto de `600` linhas por ficheiro de painel e
//! de `200` por função, 2026-09-19): lá mora *como cada espécie de fileira se dispõe*; aqui, *o que
//! uma fileira faz quando o que o artista escreveu não compila*. As duas crescem por motivos
//! diferentes — uma ganha uma espécie, a outra ganha uma forma de queixa.
//!
//! ⛔ *A cura de um tecto é o CORTE, nunca uma entrada numa lista de isenção.*

use super::*;

/// ⭐⭐ **A fileira de TEXTO que carrega uma QUEIXA** — a caixa das irmãs mais uma linha por
/// baixo, em `Danger`.
///
/// ⚠️ **Irmã por RESPONSABILIDADE do [`paint_one_row`]** (tecto de `200` linhas por função de
/// painel, 2026-09-19): ela é a única row que pode estar ERRADA, e o registo do que ela ainda
/// corta mora aqui, junto do que o descreve. *A cura de um tecto é o corte, nunca uma isenção.*
///
/// ⛔⛔ **Ela CORTA a frase em `176 px` e a cura óbvia está PROIBIDA:** trocar o pintor por um
/// que QUEBRA faz reprovar o `the_motion_chrome_never_gives_a_row_label_a_wrap_budget`, cuja lei
/// vem de um report do dono de 2026-08-30 — no chrome do Motion uma linha de `22 px` que quebra
/// derrama a segunda metade **por cima da entrada seguinte**. *A lei é mais velha e está medida.*
/// ⇒ o que fica por fazer é uma fileira que SAIBA que segura uma frase (altura própria, como o
/// aviso de secção do Inspector), e isso é desenho. Número e mecânica na dívida do
/// `nenhum_rotulo_do_app_pinta_nada::A_PASSAGEM_ARMADA_AINDA_CORTA`.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_queixa(
    row: &ParamRow,
    i: usize,
    msg: &str,
    inner_x: f32,
    inner_w: f32,
    mut y: f32,
    label_font: f32,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) -> f32 {
    let used = number::paint_box_row(
        row,
        i,
        inner_x,
        inner_w,
        y,
        store,
        hit_index,
        scene,
        text_system,
        theme,
    )
    .expect("o braço e a porta casam por construção");
    y += used;
    // ⚠️ **Alinhada com o CAMPO, não com a margem** — ela fala do que está na caixa, e uma linha
    //    à esquerda do rótulo leria-se como outra propriedade.
    paint_text_elided(
        text_system,
        scene,
        msg,
        inner_x + DEFAULT_LABEL_W,
        y + (ROW_H_PX - label_font) * 0.5,
        label_font,
        (inner_w - DEFAULT_LABEL_W).max(0.0),
        resolve(ColorToken::Danger, theme),
    );
    // ⚠️ **Nada é registado no `HitIndex`**: um aviso não se clica. Registá-lo poria um alvo
    //    mudo por cima do campo, que é o defeito que a caça aos knobs mortos nomeia.
    y + ph2d_tokens::row_pitch_px()
}
