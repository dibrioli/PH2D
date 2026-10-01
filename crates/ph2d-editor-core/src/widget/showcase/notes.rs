//! Sticky-note painters (single editable note + its title/body
//! sub-painters).
//!
//! Wave 8 Phase 2.A — hoisted from
//! `ph2d_editor::screens::hero::inspector::notes` to editor-core so
//! the showcase tree (now in editor-core::widget::showcase) and
//! eventually the panel-inspector crate can both use them without
//! depending on `ph2d-editor`.
//!
//! Notes are right-click-created highlights pinned above panel
//! sections. Each painted note registers three hit rects — the
//! whole slot for the right-click background-color menu, plus
//! title + body sub-rects for the TextInput focus + edit pipeline.

use super::read_text_input;
use crate::ids::NoteIds;
use crate::interaction::{
    HitIndex, NoteData, WidgetStore,
    dispatch::note_drag::{lugar_da_queda, seccoes_do_painel},
};
use crate::paint::{fill_rounded_rect, resolve, stroke_rounded_rect};
use crate::widget::panel_chrome::highlighter_rgba;
use crate::widget::section_grip::{grip_hit_rect, grip_rect, grip_slot_w_px, paint_grip};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Horizontal inset between a note's rect edge and where text drawn
/// inside it actually starts. Matches the non-hex `TextInput`
/// dispatch math in `byte_offset_from_click_xy` (`rect.x + 12.0`) so
/// click→caret + drag-select route to the byte under the visible
/// cursor.
pub(super) fn note_text_pad_x() -> f32 {
    Spacing::Lg.px()
}

/// Paint a single sticky-note. Editable: the title + body each
/// have their own TextInput state in the store
/// (`caixas.title[slot]` + `caixas.body[slot]`) — `caixas` são as ranhuras do PAINEL que pinta
/// ([`crate::ids::note_ids`]; até 2026-10-01 eram partilhadas entre a Galeria e o Inspector).
///
/// ⭐⭐ **A fileira do título** (2026-10-01): o botão de MINIMIZAR à esquerda (a dobra das secções,
/// em ponto pequeno), o título, e a pega de 10 pontos à direita. Minimizada, a nota é só esta
/// fileira. ⭐ **O corpo CRESCE com o texto** — a altura são as linhas visuais da quebra
/// ([`super::notes_text::linhas_do_corpo`]), nunca menos de três.
#[allow(clippy::too_many_arguments)]
pub fn paint_one_note(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: &mut f32,
    note: &NoteData,
    caixas: &NoteIds,
    slot: usize,
) {
    let pad = Spacing::Md.px();
    let title_font = TypeToken::Base.px();
    let body_font = TypeToken::Base.px();
    let title_h = title_font + Spacing::Md.px();
    let body_w = (w - pad * 2.0).max(0.0);
    let body_h = if note.minimized {
        0.0
    } else {
        let m = crate::widget::text_area_metrics(Rect::new(0.0, 0.0, body_w, 0.0));
        let texto = caixas
            .body
            .get(slot)
            .map_or("", |id| read_text_input(store, *id).1);
        let n = super::notes_text::linhas_do_corpo(text_system, texto, m.inner_w, body_font);
        // O recuo de cima e o de baixo do `TextArea` (a régua é a dele, a do despacho também).
        m.inner_y * 2.0 + m.line_h * n as f32
    };
    let note_h = title_h + body_h + pad * 2.0;
    let r = Rect::new(x, *y, w, note_h);
    if let Some(slot_id) = caixas.slot.get(slot) {
        hit_index.register(*slot_id, r);
    }
    let rgba = highlighter_rgba(note.color_idx);
    let bg = ph2d_vector::Color::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]); // LITERAL-COLOR-OK: user-color — HIGHLIGHTER_RGBA palette (note background)
    // ⛔ **A ÚNICA fill que fica FORA da porta do raio, e é declarada:** um post-it não é cromo —
    // ele tem cor de marcador fixa (`HIGHLIGHTER_RGBA`) e este pintor não recebe tema nenhum.
    // *Achatá-lo com o resto seria achatar a única superfície do app que é de propósito um objeto.*
    fill_rounded_rect(scene, r, Radius::Md.px(), bg);

    let dark = ph2d_vector::Color::from_rgba8(0x21, 0x21, 0x21, 0xFF); // LITERAL-COLOR-OK: note-text — dark glyph fixed across themes
    // Os pontos e a dobra na cor do texto da nota a meia força — a mesma do texto de espera.
    let meio = ph2d_vector::Color::from_rgba8(0x21, 0x21, 0x21, 0x80); // LITERAL-COLOR-OK: note-grip — the placeholder glyph tone
    let fila = Rect::new(r.x, r.y + pad, r.w, title_h);
    // ⭐ **O botão de MINIMIZAR** — o chevron da dobra das secções: para baixo aberta, para a
    //    direita minimizada. ⚠️ Regista-se DEPOIS do título (o hit-index resolve o último primeiro).
    let icon = ph2d_tokens::INLINE_ICON_PX;
    let dobra = Rect::new(r.x + pad, fila.y + (title_h - icon) * 0.5, icon, icon);
    let title_x = dobra.x + icon;
    let title_rect = Rect::new(
        title_x,
        fila.y,
        (r.x + r.w - grip_slot_w_px() - title_x).max(0.0),
        title_h,
    );
    if let Some(title_id) = caixas.title.get(slot) {
        hit_index.register(*title_id, title_rect);
        super::notes_text::paint_note_editable_line(
            scene,
            text_system,
            read_text_input(store, *title_id),
            title_rect,
            title_font,
            dark,
            "Title",
        );
    }
    if let Some(fold_id) = caixas.fold.get(slot) {
        hit_index.register(*fold_id, dobra);
        let glifo = if note.minimized {
            crate::icons::IconId::ChevronRight
        } else {
            crate::icons::IconId::ChevronDown
        };
        crate::paint::paint_icon(scene, glifo, dobra, meio, StrokeToken::Default.px());
    }
    // ⭐ **A PEGA da nota** (ordem do dono, 2026-09-30: *«Coloque os 10 pontinhos de arrastar
    //    também nas notas»*) — a mesma pega das secções, na ponta direita da fila do título.
    if let Some(grip_id) = caixas.grip.get(slot) {
        hit_index.register(*grip_id, grip_hit_rect(fila));
        paint_grip(scene, grip_rect(fila), meio);
    }
    if !note.minimized {
        let body_rect = Rect::new(r.x + pad, r.y + pad + title_h, body_w, body_h);
        if let Some(body_id) = caixas.body.get(slot) {
            hit_index.register(*body_id, body_rect);
            super::notes_text::paint_note_editable_multiline(
                scene,
                text_system,
                read_text_input(store, *body_id),
                body_rect,
                body_font,
                dark,
                "Notes…",
            );
        }
    }
    *y += note_h + Spacing::Md.px();
}

/// ⭐⭐ **A nota arrastada: o contorno dela no sítio de onde saiu, a marca de onde vai cair e o
/// FANTASMA** — ela, menor e meio transparente, colada ao cursor (ordem do dono, 2026-09-30).
///
/// Corre no FIM do corpo do painel, depois de todas as notas pintadas: a geometria é a do
/// hit-index DESTE quadro (a ranhura da nota e os cabeçalhos), e o fantasma fica por cima de tudo.
/// ⚠️ **A queda é a MESMA lei do despacho** ([`lugar_da_queda`]) — uma porta, dois leitores: o
/// `pointer_up` que a grava e este pintor que a mostra.
#[allow(clippy::too_many_arguments)]
pub fn paint_note_drag_ghost(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    hit_index: &HitIndex,
    store: &WidgetStore,
    panel: NodeId,
    theme: Theme,
) {
    let Some(d) = store.note_drag().filter(|d| d.active && d.panel == panel) else {
        return;
    };
    let (Some(note), Some(painel)) = (
        store.notes_for_panel(panel).get(d.index).cloned(),
        store.panel_rect(panel),
    ) else {
        return;
    };
    let caixas = crate::ids::note_ids(panel);
    let dentro = |r: &Rect| painel.contains(r.x + r.w * 0.5, r.y + r.h * 0.5);
    let ranhura = |i: usize| {
        hit_index
            .iter_registrations()
            .find(|(id, r)| *id == caixas.slot[i] && dentro(r))
            .map(|(_, r)| r)
    };
    let Some(original) = ranhura(d.index) else {
        return;
    };
    let acento = resolve(ColorToken::Accent, theme);
    let espessura = StrokeToken::Thick.px();
    let raio = Radius::Md.px();
    // FRAME-RAW-OK: o contorno de ARRASTO — um estado do gesto, da cor do acento (irmão do das secções).
    stroke_rounded_rect(scene, original, raio, espessura, acento);
    if let Some((seccao, rank)) = lugar_da_queda(store, hit_index, &d, d.cursor_y) {
        let notas = store.notes_for_panel(panel);
        let mut membros: Vec<Rect> = (0..notas.len())
            .filter(|&i| i != d.index && notas[i].section == seccao)
            .filter_map(ranhura)
            .collect();
        membros.sort_by(|a, b| a.y.total_cmp(&b.y));
        let meio = Spacing::Md.px() * 0.5;
        let y = match (membros.get(rank), membros.last()) {
            (Some(m), _) => Some(m.y - meio),
            (None, Some(u)) => Some(u.y + u.h + meio),
            (None, None) => {
                let heads = seccoes_do_painel(hit_index, painel);
                let proxima = heads
                    .iter()
                    .skip_while(|(id, _)| Some(*id) != seccao)
                    .nth(1);
                proxima.map(|(_, top)| top - ph2d_tokens::section_gap_px() * 0.5)
            }
        };
        if let Some(y) = y {
            fill_rounded_rect(
                scene,
                Rect::new(original.x, y - espessura * 0.5, original.w, espessura),
                espessura * 0.5,
                acento,
            );
        }
    }
    let mut conteudo = VectorScene::new();
    let mut descartado = HitIndex::default();
    let mut y = original.y;
    paint_one_note(
        &mut conteudo,
        text_system,
        &mut descartado,
        store,
        original.x,
        original.w,
        &mut y,
        &note,
        &caixas,
        d.index,
    );
    crate::widget::paint_card_ghost(
        scene,
        &conteudo,
        original,
        raio,
        None,
        (d.down_x, d.down_y),
        (d.cursor_x, d.cursor_y),
    );
}
