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
