//! ⭐⭐⭐ **O QUE UM PAINEL DIZ QUANDO NÃO TEM NADA A DIZER.**
//!
//! # ⛔⛔ O defeito que isto cura, medido
//!
//! Um painel encaixado que se recusa a pintar **não deixa um espaço vazio: leva a coluna
//! inteira**. Medido em 2026-09-09 com o `sculpt3d` e o `inspector` a partilharem a coluna da
//! direita e o módulo 3D desarmado:
//!
//! | quem está à frente | rect do sculpt | rect do inspector | glifos do quadro |
//! |---|---|---|---|
//! | `inspector` | — | publicado | 269 |
//! | `sculpt3d`  | — | **—** | **218** (só o cromo de base) |
//!
//! O segundo caso é o ecrã que o artista vê: ele clica na aba *Sculpt 3D* e **a coluna da direita
//! desaparece**, com a fileira de abas dentro dela — logo não há aba nenhuma para clicar de volta.
//! A cadeia é a do modelo de encaixes: quem está à frente esconde os outros
//! ([`slot_tabs::hidden_by_tabs`]), quem não pinta não publica rect, e o
//! `DockSides::from_published` responde *«esta coluna está ocupada?»* cruzando os rects
//! **publicados** com o rect da coluna. *Zero rects publicados = coluna livre.*
//!
//! # ⚠️ A recusa de pintar estava CERTA antes das abas
//!
//! O doc do painel de escultura escreve-a: *«um painel que pinta sobre o vazio é a forma de chrome
//! morto que esta casa varre a cada wave»*. Isso era verdade quando um painel encaixado era **o
//! dono da coluna** — não pintar libertava o espaço, e o espaço voltava para a área de desenho.
//! As abas mudaram o preço: hoje ele arrasta um vizinho VIVO consigo. ⇒ `CLAUDE.md` §0.0: *quem
//! move o número que tornava algo inalcançável tem de reconferir a nota.*
//!
//! # A lei que fica
//!
//! *Um painel FECHADO cala-se; um painel ABERTO deve ao artista uma superfície.* Se ele não tem
//! conteúdo, diz **porquê** e **o que fazer** — e continua a publicar o rect, que é o que segura
//! a coluna, as abas e o vizinho.

use super::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_TITLE_BASELINE, paint_panel_close_button,
    paint_panel_surface, paint_panel_title,
};
use crate::interaction::HitIndex;
use crate::paint::{paint_text_block, resolve};
use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Pinta a face vazia de um painel encaixado: a superfície, o título, o fecho e **uma frase**.
///
/// ⚠️ **Quem chama continua a ser o dono do rect** — esta porta desenha, não publica. O painel
/// publica o `ctx.slot` como em qualquer outro quadro, porque é isso que mantém a coluna de pé.
///
/// ⚠️ **A frase é do PAINEL, não daqui.** Um texto genérico («nothing to show») diria ao artista
/// exactamente o que ele já vê; o que falta é *porquê* e *o que fazer*, e isso só o painel sabe.
// ⚠️ **Oito argumentos porque a face vazia é um painel INTEIRO** — superfície, título, fecho e
// frase. Empacotá-los num struct seria uma segunda forma de dizer o que o `PaintCtx` já diz,
// e esta porta vive abaixo dele de propósito (as crates de painel chamam-na sem o `ctx`).
#[allow(clippy::too_many_arguments)]
pub fn paint_panel_empty(
    rect: Rect,
    title: &str,
    close_id: ph2d_a11y::NodeId,
    message: &str,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    hit_index: &mut HitIndex,
    theme: Theme,
) {
    paint_panel_surface(rect, scene, theme);
    let title_size = paint_panel_title(
        rect,
        title,
        PANEL_HEADER_CLOSE_RESERVE,
        scene,
        text_system,
        theme,
    );
    paint_panel_close_button(rect, close_id, hit_index, scene, theme);

    let body_top = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();
    let body_h = (rect.y + rect.h - body_top - PANEL_HEAD_PAD).max(0.0);
    let w = (rect.w - PANEL_HEAD_PAD * 2.0).max(0.0);
    if body_h <= 0.0 || w <= 0.0 {
        return;
    }
    // ⚠️ **A frase QUEBRA, não elide.** Ela é a única coisa no painel: cortá-la a `...` deixaria o
    // artista com a mesma pergunta com que entrou, e há altura de sobra para as duas linhas.
    let font = TypeToken::Sm.px();
    paint_text_block(
        text_system,
        scene,
        message,
        rect.x + PANEL_HEAD_PAD,
        body_top,
        font,
        w,
        resolve(ColorToken::Text3, theme),
    );
}
