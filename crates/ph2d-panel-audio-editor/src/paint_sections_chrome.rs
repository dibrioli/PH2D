//! **O CHROME das secções deste painel** — o cabeçalho dobrável, a dobra do corpo e a régua
//! entre duas secções.
//!
//! Irmão (`#[path]`) do [`super::paint_sections`], e o corte é o que o cap de LOC do painel
//! pediu — mas ele é honesto por conta própria: aqui mora *como uma secção SE DESENHA*, e lá
//! *que secções existem e em que ordem*. Os dois crescem por motivos diferentes.

use super::*;

/// Height of a section header band (matches the Sprite Inspector's).
pub(super) fn section_h() -> f32 {
    TypeToken::Md.px() + Spacing::Md.px()
}

/// Paint one collapsible section header — the app's canonical chrome: chevron +
/// UPPERCASE label, and a darker plate when it is folded — with the block's readout
/// right-aligned in the same band. Registers the click that folds it (the dispatch does
/// the folding, via the `mark_collapsible_section` set) and returns `(open, y below)`.
///
/// The readout rides **in** the header rather than on a row of its own: "Variations" and
/// "3 clips" are one fact, and a folded section still has to say what is inside it.
#[allow(clippy::too_many_arguments)]
pub(super) fn section(
    y: f32,
    x: f32,
    w: f32,
    id: NodeId,
    label: &str,
    readout: Option<&str>,
    // ⚠️ **O PAR, num argumento só** — o estado semântico e o `t` VIVO da dobra. É a lei que a
    //    wave dos botões deixou (`visual: (ButtonState, f32)`): com duas entradas separadas, um
    //    chamador esquece a segunda e a secção fica **silenciosamente discreta** no meio de
    //    vizinhas que rodam, com a suíte inteira verde.
    fold: (bool, f32),
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut ClippedHits,
) -> (Option<SectionFold>, f32) {
    let h = section_h();
    let rect = Rect::new(x, y, w, h);
    paint_section_header(
        &SectionHeader::new(id, label)
            .collapsible(fold.0)
            .open_t(fold.1),
        rect,
        scene,
        text_system,
        theme,
    );
    hit_index.register(id, rect);

    if let Some(text) = readout.filter(|t| !t.is_empty()) {
        // Right-aligned: the label grows from the left, so anything centred would sooner
        // or later collide with it. Measure, then place against the right edge.
        let font = TypeToken::Xs.px();
        let tw = text_system.layout(text, font, w).width();
        let pad = Spacing::Md.px();
        paint_text(
            text_system,
            scene,
            text,
            (x + w - pad - tw).max(x),
            y + (h - font) * 0.5,
            font,
            w,
            resolve(ColorToken::Text2, theme),
        );
    }
    let body_top = y + h + Spacing::Xs.px();
    // ⚠️ O `t` vem do PAR que o chamador já tem — este painel fotografa a dobra antes de o paint
    // tomar os empréstimos, e reler o store aqui seria a segunda resposta sobre a mesma seção.
    let (store, hits) = hit_index.store_and_index_mut();
    let opened = SectionFold::begin_at(store, id, fold.1, x, w, body_top, scene, hits);
    (opened, body_top)
}

/// Fecha a dobra aberta por [`section`] e devolve o `y` de saída.
pub(super) fn end_fold(
    fold: SectionFold,
    y: f32,
    scene: &mut VectorScene,
    hit_index: &mut ClippedHits,
) -> f32 {
    let (store, hits) = hit_index.store_and_index_mut();
    fold.finish(store, scene, hits, y)
}

/// The rule between two sections — `paint_section_separator`, the SAME one the Sprite
/// Inspector and the Widget Gallery draw: a 1 px accent-coloured line, not the neutral
/// `Divider`. The Gallery is the single source of truth for chrome (DIRETRIZ §5.2), and
/// a panel that invents its own line is a panel that looks like it belongs to a
/// different app (Enio, 2026-07-12).
pub(super) fn separator(y: f32, x: f32, w: f32, scene: &mut VectorScene, theme: Theme) -> f32 {
    paint_section_separator(scene, theme, x, w, y)
}
