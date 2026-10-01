//! ⭐⭐⭐ **O CROMO de uma secção e de um painel — as NOTAS e o CONTORNO, em qualquer painel**
//! (2026-10-01, ordem do dono: *«a possibilidade de criar notas deve existir em quaisquer painéis de
//! qualquer tipo. Outline também deve funcionar em qualquer painel. Em physics não está
//! funcionando. … a outline não está englobando as notas da seção. faça englobar.»*).
//!
//! Até aqui só o Inspector e a Galeria pintavam notas e contornos, cada um com a sua cópia: os
//! outros painéis de secções (Física, Vector, Painter, Escultura, Grade, Áudio, Wet Paint) mudavam
//! o tema e arrastavam, e o contorno escolhido no menu do título **não se pintava** — um item de
//! menu mudo.
//!
//! ⭐ **Duas portas, e todo painel passa por uma delas:**
//!
//! - [`fecha_seccao`] — no FIM de cada secção, dentro do cartão dela: pinta as notas presas à
//!   secção e, POR FORA de tudo, o contorno — a secção **e** as notas dela (o contorno ficava curto
//!   e deixava as notas de fora). É chamada pelos laços de secções (o `PlanoCtx`, o do Inspector,
//!   o do Vector e o da Galeria).
//! - [`pinta_as_que_sobram`] — no FIM do corpo, pela porta de rolagem
//!   ([`crate::panel::scroll_area::close`]): as notas que nenhuma secção pintou (sem secção, ou com
//!   a secção fora de vista) e o fantasma da nota arrastada. É por ela que um painel SEM secções
//!   (a Hierarquia, as ferramentas de imagem…) tem notas, e é ela que declara o painel como
//!   anfitrião ([`HitIndex::mark_note_host`]) — o botão direito só oferece *Create Note* onde ela
//!   vai aparecer.
//!
//! ⚠️ **O livro das notas pintadas é o do quadro** ([`HitIndex::mark_note_painted`]): uma nota cuja
//! secção pintou não pode voltar a ser pintada no fim, e uma nota cuja secção não está à vista não
//! pode desaparecer.

use crate::interaction::{HitIndex, WidgetStore};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{Radius, Spacing, StrokeToken, Theme};
use ph2d_vector::VectorScene;

/// ⭐⭐ **Fecha uma secção**: as notas dela e o contorno à volta da secção E das notas. Devolve o
/// `y` depois da última nota (o da secção, se ela não tem notas).
#[allow(clippy::too_many_arguments)]
pub fn fecha_seccao(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    panel: NodeId,
    section: NodeId,
    inner_x: f32,
    inner_w: f32,
    y_before: f32,
    y: f32,
) -> f32 {
    let caixas = crate::ids::note_ids(panel);
    let mut fim = y;
    let mut pintou = false;
    for (slot, note) in store.notes_for_panel(panel).iter().enumerate() {
        if note.section == Some(section) && !hit_index.note_painted(panel, slot) {
            super::paint_one_note(
                scene,
                text_system,
                hit_index,
                store,
                inner_x,
                inner_w,
                &mut fim,
                note,
                &caixas,
                slot,
            );
            hit_index.mark_note_painted(panel, slot);
            pintou = true;
        }
    }
    if let Some(color_idx) = store.section_outline_color(section) {
        let caixa = caixa_do_contorno(inner_x, inner_w, y_before, y, fim, pintou);
        contorno(scene, color_idx, caixa);
    }
    fim
}

/// ⭐⭐ **A caixa do contorno de uma secção** — do topo dela até ao fim da ÚLTIMA nota (`fim`, o `y`
/// que o laço das notas devolveu), ou até ao fim da secção (`y`) quando ela não tem notas; com a
/// folga `Xs` à volta. ⚠️ O `paint_one_note` soma o vão até à nota SEGUINTE, que não é desta — o
/// fundo desconta-o. *Até 2026-10-01 o contorno era pintado ANTES das notas e acabava em `y`: as
/// notas da secção ficavam de fora* (o report do dono).
#[must_use]
pub fn caixa_do_contorno(
    inner_x: f32,
    inner_w: f32,
    y_before: f32,
    y: f32,
    fim: f32,
    pintou: bool,
) -> Rect {
    let fundo = if pintou { fim - Spacing::Md.px() } else { y };
    let pad = Spacing::Xs.px();
    Rect::new(
        inner_x - pad,
        y_before - pad,
        inner_w + pad * 2.0,
        (fundo - y_before + pad * 2.0).max(0.0),
    )
}

/// O contorno de marcador de uma secção, na caixa dada.
fn contorno(scene: &mut VectorScene, color_idx: u8, block: Rect) {
    let rgba = crate::widget::panel_chrome::highlighter_rgba(color_idx);
    let cor = ph2d_vector::Color::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]); // LITERAL-COLOR-OK: HIGHLIGHTER_RGBA palette — a marca que o artista escolheu
    // FRAME-RAW-OK: a MARCA de realce que o utilizador escolheu (cor de marcador): conteúdo autorado.
    // ⛔ **FORA da porta do raio, e é a mesma família do post-it:** cor de highlighter fixa, e este
    //    pintor não recebe tema nenhum. *Achatá-lo com o cromo seria achatar a única marca que é de
    //    propósito um objecto do dono.*
    crate::paint::stroke_rounded_rect(scene, block, Radius::Md.px(), StrokeToken::Thick.px(), cor);
}

/// ⭐⭐ **As notas que sobram no FIM do corpo de `panel`** — as que nenhuma secção pintou neste
/// quadro — e o fantasma da nota arrastada por cima. Declara o painel como anfitrião de notas.
/// Devolve o `y` depois da última. ⚠️ Idempotente no quadro: a 2.ª chamada para o mesmo painel
/// não faz nada.
#[allow(clippy::too_many_arguments)]
pub fn pinta_as_que_sobram(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    panel: NodeId,
    theme: Theme,
    inner_x: f32,
    inner_w: f32,
    y: f32,
) -> f32 {
    // ⭐ UMA vez por painel por quadro: um painel que a chama do próprio corpo (para a altura que
    //    publica contar com as notas) e depois fecha pela porta de rolagem não pinta o fantasma
    //    duas vezes.
    if hit_index.is_note_host(panel) {
        return y;
    }
    hit_index.mark_note_host(panel);
    let caixas = crate::ids::note_ids(panel);
    let mut fim = y;
    for (slot, note) in store.notes_for_panel(panel).iter().enumerate() {
        if !hit_index.note_painted(panel, slot) {
            super::paint_one_note(
                scene,
                text_system,
                hit_index,
                store,
                inner_x,
                inner_w,
                &mut fim,
                note,
                &caixas,
                slot,
            );
            hit_index.mark_note_painted(panel, slot);
        }
    }
    super::paint_note_drag_ghost(scene, text_system, hit_index, store, panel, theme);
    fim
}
