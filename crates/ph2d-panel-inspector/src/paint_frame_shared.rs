//! **As quatro seções COMPARTILHADAS do Inspector** — §5 9-Slice, §7 Ordering, §9 Sampling e
//! §10 Material & Blend.
//!
//! ⚠️ **Irmão de [`super::paint_frame`] por CAP de FICHEIRO** (600): a família cabia lá dentro
//! logicamente, mas levava-o a 628. *Um cap de ficheiro e um cap de função medem grandezas
//! diferentes*, e extrair para o mesmo ficheiro curaria um e estouraria o outro — a lição que o
//! par de PRECISAO já pagou em 2026-08-20.
//!
//! Elas andam juntas porque partilham a mesma porta — **qualquer entidade com `Transform`** — e
//! porque manter os quatro slots de nota (6..9) adjacentes é o que os torna obviamente distintos.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{HitIndex, NoteData, WidgetStore};
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

use super::paint_frame::{begin_section, finish_section};

/// **§5 9-Slice + §7 Ordering + §9 Sampling + §10 Material & Blend**, moldura e tudo.
///
/// Levantadas do `paint_inspector` pelo mesmo motivo da família da física: aquele orquestrador
/// está numa tolerância de LOC que **só encolhe**, e a §5 (2026-08-21) empurrou-o para 436 contra
/// 414. As quatro vivem juntas porque partilham a mesma porta — qualquer entidade com `Transform`
/// — e porque manter os seus quatro slots de nota (6..9) adjacentes é o que os torna obviamente
/// distintos.
///
/// Devolve o novo `y`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_shared_sections(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: ph2d_tokens::Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    section_tops_y: &mut Vec<f32>,
    inner_x: f32,
    inner_w: f32,
    body_top_y: f32,
    mut y: f32,
    header_h: f32,
    slice: Option<&ph2d_editor_core::screens::hero::InspectorSliceInfo>,
    ordering: Option<&ph2d_editor_core::screens::hero::InspectorOrderingInfo>,
    sampling: Option<&ph2d_editor_core::screens::hero::InspectorSamplingInfo>,
    blend: Option<&ph2d_editor_core::screens::hero::InspectorBlendInfo>,
    notes: &[Vec<(usize, NoteData)>],
) -> f32 {
    let slot = |i: usize| notes.get(i).map_or(&[][..], |v| &v[..]);
    // §5 9-Slice — LOGO A SEGUIR à Sprite Sheet, que é a vizinhança que a explica: as duas
    // descrevem como os pixels da fonte se distribuem pelo quad. ⚠️ Aparece para toda sprite,
    // COM ou SEM o componente — sem ele mostra só o «+ Add 9-Slice». Uma seção que só existe
    // depois de a feature estar ligada é uma feature que ninguém descobre.
    if let Some(sl) = slice {
        y = crate::paint::paint_section_separator_at(scene, theme, inner_x, inner_w, y);
        let y_before = y;
        begin_section(
            section_tops_y,
            hit_index,
            inner_x,
            inner_w,
            body_top_y,
            y_before,
            ids::INSP_LIVE_SLICE_SECTION,
            header_h,
        );
        let new_y = crate::sections::slice_nine::paint_slice_section(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            inner_x,
            inner_w,
            y,
            sl,
        );
        y = finish_section(
            scene,
            text_system,
            hit_index,
            store,
            inner_x,
            inner_w,
            ids::INSP_LIVE_SLICE_SECTION,
            y_before,
            new_y,
            slot(6),
        );
    }
    // §7 Ordering / Sorting — vale para qualquer entidade com Transform, não só sprites.
    if let Some(ord) = ordering {
        y = crate::paint::paint_section_separator_at(scene, theme, inner_x, inner_w, y);
        let y_before = y;
        begin_section(
            section_tops_y,
            hit_index,
            inner_x,
            inner_w,
            body_top_y,
            y_before,
            ids::INSP_LIVE_ORDERING_SECTION,
            header_h,
        );
        let new_y = crate::sections::paint_ordering_section(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            inner_x,
            inner_w,
            y,
            ord,
        );
        y = finish_section(
            scene,
            text_system,
            hit_index,
            store,
            inner_x,
            inner_w,
            ids::INSP_LIVE_ORDERING_SECTION,
            y_before,
            new_y,
            slot(7),
        );
    }
    // §9 Sampling — irmã da §7.
    if let Some(samp) = sampling {
        y = crate::paint::paint_section_separator_at(scene, theme, inner_x, inner_w, y);
        let y_before = y;
        begin_section(
            section_tops_y,
            hit_index,
            inner_x,
            inner_w,
            body_top_y,
            y_before,
            ids::INSP_LIVE_SAMPLING_SECTION,
            header_h,
        );
        let new_y = crate::sections::paint_sampling_section(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            inner_x,
            inner_w,
            y,
            samp,
        );
        y = finish_section(
            scene,
            text_system,
            hit_index,
            store,
            inner_x,
            inner_w,
            ids::INSP_LIVE_SAMPLING_SECTION,
            y_before,
            new_y,
            slot(8),
        );
    }
    // §10 Material & Blend — irmã da §9.
    if let Some(bl) = blend {
        y = crate::paint::paint_section_separator_at(scene, theme, inner_x, inner_w, y);
        let y_before = y;
        begin_section(
            section_tops_y,
            hit_index,
            inner_x,
            inner_w,
            body_top_y,
            y_before,
            ids::INSP_LIVE_BLEND_SECTION,
            header_h,
        );
        let new_y = crate::sections::paint_material_blend_section(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            inner_x,
            inner_w,
            y,
            bl,
        );
        y = finish_section(
            scene,
            text_system,
            hit_index,
            store,
            inner_x,
            inner_w,
            ids::INSP_LIVE_BLEND_SECTION,
            y_before,
            new_y,
            slot(9),
        );
    }
    y
}

/// **§12 Sockets / Named Anchors** (ADR-0072) — a última seção, e a única que precisa do
/// **estado do painel**: qual linha da lista está aberta.
///
/// ⚠️ O índice é **saturado aqui** contra o tamanho da lista. Apagar a última âncora deixa-o a
/// apontar para além do fim, e um editor aberto sobre uma linha que já não existe é a forma mais
/// direta de escrever na âncora errada.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_anchor_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: ph2d_tokens::Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    section_tops_y: &mut Vec<f32>,
    inner_x: f32,
    inner_w: f32,
    body_top_y: f32,
    mut y: f32,
    header_h: f32,
    anchor: Option<&ph2d_editor_core::screens::hero::InspectorAnchorInfo>,
    selected: &mut usize,
    notes: &[Vec<(usize, NoteData)>],
) -> f32 {
    let Some(anch) = anchor else {
        // Sem snapshot não há ficha aberta — e o gizmo do canvas tem de saber disso, senão ele
        // continua a oferecer alças de uma âncora que a seção já não mostra.
        crate::state::set_open_anchor_row(None);
        return y;
    };
    *selected = (*selected).min(anch.rows.len().saturating_sub(1));
    // ⚠️ **A linha aberta viaja para a SHELL aqui** — é o que dá alças ao gizmo do canvas. Sai da
    // PINTURA e não do despacho de propósito: a pintura corre todo o quadro e conhece o estado
    // final (já corrigido contra o tamanho da lista, na linha acima), enquanto o despacho só
    // corre quando alguém clica.
    crate::state::set_open_anchor_row((!anch.rows.is_empty()).then_some(*selected));
    y = crate::paint::paint_section_separator_at(scene, theme, inner_x, inner_w, y);
    let y_before = y;
    begin_section(
        section_tops_y,
        hit_index,
        inner_x,
        inner_w,
        body_top_y,
        y_before,
        ids::INSP_LIVE_ANCHOR_SECTION,
        header_h,
    );
    let new_y = crate::sections::anchors::paint_anchors_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        inner_x,
        inner_w,
        y,
        anch,
        *selected,
    );
    finish_section(
        scene,
        text_system,
        hit_index,
        store,
        inner_x,
        inner_w,
        ids::INSP_LIVE_ANCHOR_SECTION,
        y_before,
        new_y,
        notes.get(14).map_or(&[][..], |v| &v[..]),
    )
}

/// **§3 Render Source + §6 Color & Tint + §4 Sprite Sheet** — as três que só existem quando há
/// sprite. Moldura e tudo, como as irmãs deste ficheiro.
///
/// Saíram do `paint_inspector` pelo mesmo cap que levou lá as compartilhadas: a §12
/// Sockets/Anchors empurrou-o para 403 contra uma catraca de 387. *A cura de um teto estourado
/// é o corte.*
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_sprite_sections(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: ph2d_tokens::Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    section_tops_y: &mut Vec<f32>,
    inner_x: f32,
    inner_w: f32,
    body_top_y: f32,
    mut y: f32,
    header_h: f32,
    sprite: Option<&ph2d_editor_core::screens::hero::InspectorSpriteInfo>,
    notes: &[Vec<(usize, NoteData)>],
) -> f32 {
    let Some(info) = sprite else {
        return y;
    };
    let slot = |i: usize| notes.get(i).map_or(&[][..], |v| &v[..]);
    for (section_id, note_slot, which) in [
        (ids::INSP_LIVE_RENDER_SECTION, 3usize, 0u8),
        (ids::INSP_LIVE_COLOR_SECTION, 4, 1),
        (ids::INSP_LIVE_SHEET_SECTION, 5, 2),
    ] {
        if which > 0 {
            y = crate::paint::paint_section_separator_at(scene, theme, inner_x, inner_w, y);
        }
        let y_before = y;
        begin_section(
            section_tops_y,
            hit_index,
            inner_x,
            inner_w,
            body_top_y,
            y_before,
            section_id,
            header_h,
        );
        let new_y = match which {
            0 => crate::sections::paint_render_source_section(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                inner_x,
                inner_w,
                y,
                info,
            ),
            1 => crate::sections::paint_color_tint_section(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                inner_x,
                inner_w,
                y,
            ),
            _ => crate::sections::paint_sprite_sheet_section(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                inner_x,
                inner_w,
                y,
            ),
        };
        y = finish_section(
            scene,
            text_system,
            hit_index,
            store,
            inner_x,
            inner_w,
            section_id,
            y_before,
            new_y,
            slot(note_slot),
        );
    }
    y
}
