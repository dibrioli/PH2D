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
use ph2d_editor_core::widget::showcase::take_pending_dropdown_chip;
use ph2d_editor_core::widget::{self, Dropdown, DropdownOption};

use crate::{sections, state};
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

/// **OS TRÊS POPOVERS DIFERIDOS** — a §9 Sampling, a §7 Sorting Layer e a §12 «Rides Parent
/// Anchor». Pintam-se DEPOIS de todas as seções, para ficarem acima de tudo.
///
/// ⚠️ **Irmãos por uma LEI, não por vizinhança:** um popover aberto tem de sair da ordem em que a
/// sua seção foi pintada, senão a seção seguinte desenha-lhe por cima. Cada um guarda o seu rect
/// num slot próprio durante o passe normal e é resgatado aqui.
///
/// Saíram do `paint_inspector` em 2026-08-23, quando o seletor da §12 o levou de 380 a 403 contra
/// uma tolerância que **só desce**. ⚠️ Levar só o novo devolveria o número a 380 exactos, e *ficar
/// no mesmo sítio não é encolher* — a mesma lição que o par de PRECISÃO e o par de sliders da
/// sprite já pagaram nesta família.
pub(crate) fn paint_deferred_popovers(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: ph2d_tokens::Theme,
    hit_index: &mut HitIndex,
) {
    if let Some((sel_idx, chip)) = take_pending_dropdown_chip() {
        let labels = ["Front", "Side", "Top"];
        let selected_label = labels.get(sel_idx).copied().unwrap_or("Front");
        let dd = Dropdown::new(
            ids::INSP_SAMPLE_DROPDOWN,
            "View",
            vec![
                DropdownOption::new(ids::INSP_SAMPLE_DD_OPT_A, "front", "Front"),
                DropdownOption::new(ids::INSP_SAMPLE_DD_OPT_B, "side", "Side"),
                DropdownOption::new(ids::INSP_SAMPLE_DD_OPT_C, "top", "Top"),
            ],
        )
        .selected(selected_label)
        .open(true);
        widget::paint_dropdown_popover(&dd, chip, scene, text_system, theme);
        for (i, opt) in dd.options.iter().enumerate() {
            hit_index.register(opt.id, dd.option_rect(chip, i));
        }
    }
    // W3 §7 Sorting Layer dropdown popover — same deferred-paint pass,
    // panel-local pending slot so it never collides with the sample dd.
    if let Some((sel_idx, chip)) = state::take_pending_ordering_dd() {
        let label = sections::ordering::LAYER_LABELS
            .get(sel_idx)
            .copied()
            .unwrap_or("Default");
        let dd = Dropdown::new(
            ids::INSP_ORDER_SORTING_LAYER,
            "",
            sections::ordering::layer_options(),
        )
        .selected(label)
        .open(true);
        widget::paint_dropdown_popover(&dd, chip, scene, text_system, theme);
        for (i, opt) in dd.options.iter().enumerate() {
            hit_index.register(opt.id, dd.option_rect(chip, i));
        }
    }

    // §12 «Rides Parent Anchor» — mesmo passe diferido, slot próprio.
    //
    // ⚠️ **As opções rederivam-se do snapshot aqui**, e não vêm no slot: guardá-las seria uma
    // segunda cópia da mesma verdade, e as duas divergiriam no quadro em que a seleção muda.
    if let Some(chip) = state::take_pending_mount_dd()
        && let Some(info) = state::current_inspector_anchor()
    {
        let mut dd = Dropdown::new(
            ids::INSP_MOUNT_PICK,
            "",
            sections::anchor_mount_row::mount_options(&info),
        )
        .open(true)
        .placeholder(sections::anchor_mount_row::mount_placeholder(&info));
        if let Some(i) = info.mount_index() {
            dd.select(Some(i));
        }
        widget::paint_dropdown_popover(&dd, chip, scene, text_system, theme);
        for (i, opt) in dd.options.iter().enumerate() {
            hit_index.register(opt.id, dd.option_rect(chip, i));
        }
    }
}

/// **§11 Animation** — moldura e tudo. Irmã da `paint_anchor_section`, e igual a ela na única
/// coisa que as distingue das outras: ela também precisa do **estado do painel** (qual animação
/// está aberta no editor).
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_anim_section(
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
    anim: Option<&ph2d_editor_core::screens::hero::InspectorAnimInfo>,
    selected: &mut usize,
) -> f32 {
    let Some(an) = anim else {
        return y;
    };
    *selected = (*selected).min(an.rows.len().saturating_sub(1));
    y = crate::paint::paint_section_separator_at(scene, theme, inner_x, inner_w, y);
    let y_before = y;
    begin_section(
        section_tops_y,
        hit_index,
        inner_x,
        inner_w,
        body_top_y,
        y_before,
        ids::INSP_LIVE_ANIM_SECTION,
        header_h,
    );
    let new_y = crate::sections::anim::paint_anim_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        inner_x,
        inner_w,
        y,
        an,
        *selected,
    );
    // ⚠️ **Sem slot de NOTA**, e é deliberado: os slots são uma lista posicional que as doze
    // seções partilham, e acrescentar um a meio renumeraria as notas que os artistas já colaram.
    // A §11 nasce sem nota; dar-lhe uma é um passo à parte, com a renumeração feita de uma vez.
    finish_section(
        scene,
        text_system,
        hit_index,
        store,
        inner_x,
        inner_w,
        ids::INSP_LIVE_ANIM_SECTION,
        y_before,
        new_y,
        &[],
    )
}

/// **As duas seções que precisam do ESTADO do painel** — a §11 Animation e a §12
/// Sockets/Anchors, na ordem em que se pintam.
///
/// ⚠️ **Elas andam juntas por uma PROPRIEDADE, não por vizinhança:** são as únicas do Inspector
/// cuja pintura depende de qual LINHA está aberta — um facto que vive no `InspectorState` e que
/// nenhuma outra seção conhece. As dez restantes leem só o snapshot.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_stateful_sections(
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
    anim: Option<&ph2d_editor_core::screens::hero::InspectorAnimInfo>,
    anim_selected: &mut usize,
    anchor: Option<&ph2d_editor_core::screens::hero::InspectorAnchorInfo>,
    anchor_selected: &mut usize,
    notes: &[Vec<(usize, NoteData)>],
) -> f32 {
    y = paint_anim_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        header_h,
        anim,
        anim_selected,
    );
    paint_anchor_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        header_h,
        anchor,
        anchor_selected,
        notes,
    )
}
