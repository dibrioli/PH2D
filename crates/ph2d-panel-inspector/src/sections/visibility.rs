//! Visibility — Inspector §8 section painter (Sprite Inspector v2 W3,
//! spec §3.8 / §6). Renders the optional-component controls BELOW the
//! always-on "Visible" toggle row (`paint_visibility_row`): the
//! `VisibilityLayer` 4×8 bitmask, the `ClipChildren` + `MaskInteraction`
//! segmented modes, the mask `alpha_cutoff` (when Mask != None), and the
//! `OnScreenEnabler` toggle + its Rect2 editor (when on). Snapshot-driven
//! like §9 Sampling; each control dispatches an `InspectorVisibilitySectionEdit`.

use super::*;
use ph2d_editor_core::screens::hero::InspectorVisibilitySectionInfo;
use ph2d_editor_core::widget::SectionFold;
use ph2d_editor_core::widget::section_cards::close_section;
use ph2d_editor_core::widget::{SegmentedAdaptive, SegmentedOption, paint_segmented_adaptive};
use ph2d_i18n::tr;

/// Os nomes que esta secção pinta à esquerda — a fonte da coluna dela.
const NOMES: [&str; 6] = [
    "panel.inspector.visibility.clip_children",
    "panel.inspector.visibility.mask_interaction",
    "panel.inspector.visibility.mask_alpha_cutoff",
    "panel.inspector.visibility.enabler_rect",
    // ⭐ As linhas de MARCAR partilham a coluna.
    "panel.inspector.visibility.mask_source_mask2d",
    "panel.inspector.visibility.on_screen_enabler",
];
/// Quantas componentes tem a linha que esta secção não quer ver quebrar.
const CAMPOS: usize = 2;

/// ⭐⭐ **A COLUNA DESTA SECÇÃO — uma só, para as TRÊS famílias de linha que ela desenha.**
///
/// ⛔ Report do dono, 2026-09-15: *«a caixa recua quando na verdade o nome deveria criar as
/// colunas»*. Esta secção pinta um segmentado, um campo e uma linha de quatro componentes; sem uma
/// declaração comum cada família responderia à pergunta por sua conta — e três respostas à mesma
/// pergunta são três colunas. Ver [`ph2d_editor_core::property_row::Seccao`].
fn seccao(text_system: &mut TextSystem) -> ph2d_editor_core::property_row::Seccao {
    ph2d_editor_core::property_row::Seccao::medida(text_system, CAMPOS, &NOMES.map(tr))
}

/// Label-above row with a single NumberInput. Returns the next `y`.
/// Mirrors §9 Sampling's `uv_pair_row` but for one value (cutoff, a rect
/// component).
#[allow(clippy::too_many_arguments)]
fn number_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    label: &str,
    id: NodeId,
    // ⚠️ **A coluna é da SECÇÃO** — ver [`seccao`]. Sem ela esta linha respondia pela metade da
    //    linha enquanto a vizinha de quatro componentes cedia, e a secção saía esfarrapada.
    sec: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    // ⭐ **Rótulo à ESQUERDA, pela porta** (report do dono, 14/09) — esta era a SEGUNDA das duas
    // funções do Inspector que empilhavam o rótulo, gémea da `sections/rows::num_row`.
    let h = ROW_H_PX;
    let row = ph2d_editor_core::property_row::colunas_da_linha(x, w, y, h, sec);
    let label_font = TypeToken::Sm.px();
    ph2d_editor_core::widget::paint_property_label(
        text_system,
        scene,
        label,
        row.label.x,
        row.label.y + (row.label.h - label_font) * 0.5,
        label_font,
        row.label.w,
        resolve(ColorToken::Text2, theme),
    );
    hit_index.register(id, row.control);
    let (state, value, buffer, caret, anchor) = read_number_input(store, id);
    let input = NumberInput::new(id, "", value)
        .step(0.1) // LITERAL-PX-OK: cutoff/rect step
        .visual((state, store.hover_live(id)));
    paint_number_input_with_buffer(
        &input,
        Some(buffer),
        caret,
        anchor,
        row.control,
        scene,
        text_system,
        theme,
    );
    ph2d_editor_core::widget::paint_decorator_dot(scene, theme, row.dot);
    y + h + ph2d_tokens::control_gap_px()
}

/// Paint a 3-tab segmented control (label above), registering each tab's
/// hit rect. Returns the next `y`.
#[allow(clippy::too_many_arguments)]
fn segmented_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    label: &str,
    ids3: [NodeId; 3],
    labels3: [&str; 3],
    // `None` = a seleção diverge; nenhum segmento acende.
    selected: Option<usize>,
    // ⚠️ **A coluna é da SECÇÃO** — ver [`seccao`].
    sec: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    let h = ROW_H_PX;
    // ⭐⭐ **O nome à ESQUERDA** (2026-09-15): esta porta é irmã da `rows::seg_row`, que já o fazia,
    //    e a única diferença dela é poder dizer «misto» — não a disposição da linha.
    let row = super::rows::property_label_row(scene, text_system, theme, x, w, y, h, label, sec);
    // ⚠️ **`SegmentedAdaptive` e não `Tabs`, para poder dizer «misto».** O `Tabs::selected()`
    // clampa (`idx.min(len-1)`), por isso é **incapaz** de renderizar «nenhum aceso» — e era isso
    // que fazia estas duas rows acenderem o valor da primária como se toda a seleção concordasse,
    // enquanto o host já calculava a divergência e a deitava fora
    // (auditoria `docs/Sprite_projeto/20` §3.3).
    let seg = SegmentedAdaptive::new(
        NodeId(0),
        label,
        ids3.iter()
            .zip(labels3)
            .map(|(&id, l)| SegmentedOption::new(id, l))
            .collect(),
    )
    .selected(selected.unwrap_or(usize::MAX));
    let seg_h = paint_segmented_adaptive(
        &seg,
        row.control,
        scene,
        text_system,
        theme,
        store,
        hit_index,
    );
    ph2d_editor_core::widget::paint_decorator_dot(scene, theme, row.dot);
    y + seg_h.max(h) + ph2d_tokens::control_gap_px()
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_visibility_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorVisibilitySectionInfo,
) -> f32 {
    let h = ROW_H_PX;
    // ⚠️ **O vão entre dois controlos é a porta `control_gap_px` (3 px)**, e não o
    //    `Spacing::Xs` (4) escrito à mão — ordem do dono, 2026-09-07. Esta secção é
    //    anterior à porta. Ver `every_stack_of_rows_asks_the_rhythm`.
    let row_gap = ph2d_tokens::control_gap_px();
    let mut yy = y;
    let sec = seccao(text_system);

    // Visibility Layer — collapsible sub-section using the CANONICAL
    // section header (a divider above + UPPERCASE title + chevron), so it
    // matches every other section. The 32-bit 4×8 grid is tall + advanced
    // (camera cull mask, not z-order), so it defaults COLLAPSED
    // (`set_collapsed` in `pre_populate`). Clicking the header toggles
    // `is_collapsed(INSP_VIS_LAYER_HEADER)` via `apply_click` (the id is
    // marked collapsible). Bit `n` = layer `n+1`; absent component → ALL.
    yy = close_section(scene, theme, x, w, yy);
    let layer_header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: section header band height
    let layer_header = section_header(
        store,
        core_ids::INSP_VIS_LAYER_HEADER,
        tr("panel.inspector.visibility.visibility_layer"),
    )
    .open_t(store.section_open_live(core_ids::INSP_VIS_LAYER_HEADER));
    let layer_header_rect = Rect::new(x, yy, w, layer_header_h);
    paint_section_header(&layer_header, layer_header_rect, scene, text_system, theme);
    hit_index.register(core_ids::INSP_VIS_LAYER_HEADER, layer_header_rect);
    yy += layer_header_h;
    // ⚠️ A grade de 32 bits e' ALTA, entao ela e' a sub-seccao onde a dobra do corpo mais se ve'.
    // O `begin` devolve `None` so' quando a seccao esta' fechada **e parada** — e' ai' que o corpo
    // nao e' percorrido de todo, exactamente o `if layer_collapsed` de sempre.
    match SectionFold::begin(
        store,
        core_ids::INSP_VIS_LAYER_HEADER,
        x,
        w,
        yy,
        scene,
        hit_index,
    ) {
        None => yy += row_gap,
        Some(fold) => {
            let grid = BitmaskGrid32::new(
                core_ids::INSP_LIVE_VISIBILITY_SECTION,
                tr("panel.inspector.visibility.visibility_layer"),
                ids::INSP_VIS_LAYER_BIT,
                info.layer_mask,
            );
            for (bit, id) in ids::INSP_VIS_LAYER_BIT.iter().enumerate() {
                hit_index.register(*id, BitmaskGrid32::cell_rect(x, yy, w, h, bit));
            }
            paint_bitmask_grid32(&grid, x, yy, w, h, scene, text_system, theme);
            let inner = yy + BitmaskGrid32::grid_height(h) + row_gap;
            yy = fold.finish(store, scene, hit_index, inner);
        }
    }

    // Clip Children — Disabled / ClipOnly / ClipAndDraw (tags 0/1/2).
    yy = segmented_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        tr("panel.inspector.visibility.clip_children"),
        ids::INSP_VIS_CLIP,
        [
            tr("panel.inspector.visibility.disabled"),
            tr("panel.inspector.visibility.clip"),
            tr("panel.inspector.visibility.clip_plus_draw"),
        ],
        (!info.mixed.clip_mode).then_some(usize::from(info.clip_mode)),
        sec,
    );

    // Mask Interaction — None / VisibleInside / VisibleOutside (0/1/2).
    yy = segmented_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        tr("panel.inspector.visibility.mask_interaction"),
        ids::INSP_VIS_MASK,
        [
            tr("panel.inspector.visibility.none"),
            tr("panel.inspector.visibility.inside"),
            tr("panel.inspector.visibility.outside"),
        ],
        (!info.mixed.mask_mode).then_some(usize::from(info.mask_mode)),
        sec,
    );

    // Mask alpha cutoff — only meaningful when the sprite obeys a mask.
    if info.mask_mode != 0 {
        yy = number_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            tr("panel.inspector.visibility.mask_alpha_cutoff"),
            ids::INSP_VIS_ALPHA_CUTOFF,
            sec,
        );
    }

    // Mask Source toggle — makes this sprite a Mask2D source (its silhouette
    // masks sibling VisibleInside/Outside responders).
    let src_rect = Rect::new(x, yy, w, h);
    hit_index.register(ids::INSP_VIS_MASK_SOURCE, src_rect);
    let src_cb = Checkbox::new(
        ids::INSP_VIS_MASK_SOURCE,
        tr("panel.inspector.visibility.mask_source_mask2d"),
    )
    .visual(store.checkbox_visual(ids::INSP_VIS_MASK_SOURCE))
    .seccao(sec)
    .value(if info.mask_source {
        CheckboxValue::Checked
    } else {
        CheckboxValue::Unchecked
    });
    paint_checkbox(&src_cb, src_rect, scene, text_system, theme);
    yy += h + row_gap;

    yy = paint_enabler_rows(scene, text_system, theme, hit_index, store, x, w, yy, info);

    yy + SECTION_BOTTOM_PAD_PX
}

/// **A moldura de ecrã** — o interruptor do `OnScreenEnabler` e o rectângulo dele.
///
/// ⚠️ **Função irmã por CAP** (200): com este bloco inline a [`paint_visibility_section`] media
/// **213** depois de os rótulos passarem pela tabela — o `tr("…")` alonga a chamada e o `rustfmt`
/// parte-a. *A cura de um tecto estourado é o CORTE.* E a fronteira é a que a secção já desenha: as
/// rows de cima dizem *o que este objecto mostra*, estas *quando ele deixa de ser desenhado*.
#[allow(clippy::too_many_arguments)]
fn paint_enabler_rows(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorVisibilitySectionInfo,
) -> f32 {
    let h = ROW_H_PX;
    let row_gap = ph2d_tokens::control_gap_px();
    let mut yy = y;
    let sec = seccao(text_system);
    // On-Screen Enabler toggle (presence of the component).
    let on_rect = Rect::new(x, yy, w, h);
    hit_index.register(ids::INSP_VIS_ON_SCREEN, on_rect);
    let on_cb = Checkbox::new(
        ids::INSP_VIS_ON_SCREEN,
        tr("panel.inspector.visibility.on_screen_enabler"),
    )
    .visual(store.checkbox_visual(ids::INSP_VIS_ON_SCREEN))
    .seccao(sec)
    .value(if info.on_screen {
        CheckboxValue::Checked
    } else {
        CheckboxValue::Unchecked
    });
    paint_checkbox(&on_cb, on_rect, scene, text_system, theme);
    yy += h + row_gap;

    // Enabler Rect — canonical Rect2Editor (X/Y/W/H in one row), only
    // when the enabler is on.
    if info.on_screen {
        // ⭐⭐⭐ **O nome à ESQUERDA e as quatro componentes na coluna do controlo** (2026-09-15).
        //
        // ⚠️⚠️ **O comentário que aqui estava já tinha chegado à LEI desta wave, para uma row só:**
        //    *«2×2 grid: the Inspector column is too narrow for four number inputs in one row (each
        //    would fall below NumberInput's usable minimum width)»*. Era verdade e estava escrita à
        //    mão num sítio — e as outras dezanove rows de N campos não a conheciam. Hoje quem a diz
        //    é a [`ph2d_editor_core::widget::property_fields_layout`], e ela reflui sozinha.
        //
        // ⛔ **O [`Rect2Editor`] sai daqui e fica sem consumidor de produto** (resta-lhe a bancada
        //    de widgets). *Um retângulo não é uma família de controlo à parte: são quatro números
        //    de uma propriedade*, que é exactamente o que a porta nova desenha.
        const RECT_STEP: f64 = 0.1; // LITERAL-PX-OK: passo de nudge do rect do enabler
        yy = super::rows::fields_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            tr("panel.inspector.visibility.enabler_rect"),
            &[
                ids::INSP_VIS_RECT_X,
                ids::INSP_VIS_RECT_Y,
                ids::INSP_VIS_RECT_W,
                ids::INSP_VIS_RECT_H,
            ],
            RECT_STEP,
            None,
            sec,
        );
    }
    yy
}
