//! Material & Blend — Inspector §10 section painter (Sprite Inspector
//! v2, spec §3.10). The Blend Mode is a 6-way segmented control over the
//! optional `BlendMode` component (Mix is the default = component
//! absent). Material / InstanceShaderParams are a future shader runtime
//! (the renderer is fixed-function today), so the Material slot is a
//! read-only placeholder. Snapshot-driven like §9 Sampling.

use super::*;
use ph2d_editor_core::screens::hero::InspectorBlendInfo;
use ph2d_editor_core::widget::SectionFold;
use ph2d_i18n::TextKey;
use ph2d_i18n::tr;

/// Blend-mode segmented labels, indexed by `BlendMode::tag()` (0..5).
/// Hardcoded here (not from `ph2d_ecs::BlendMode`) so the panel crate
/// stays loose-coupled from `ph2d-ecs` — the snapshot carries only the
/// resolved tag. English per HR-15 (i18n migrates in W7).
pub(crate) const BLEND_LABELS: [TextKey; 6] = [
    TextKey::new("panel.inspector.material.mix"),
    TextKey::new("panel.inspector.material.add"),
    TextKey::new("panel.inspector.material.subtract"),
    TextKey::new("panel.inspector.material.multiply"),
    TextKey::new("panel.inspector.material.screen"),
    TextKey::new("panel.inspector.material.premult"),
];

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_material_blend_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorBlendInfo,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: section header band height
    let header = section_header(
        store,
        core_ids::INSP_LIVE_BLEND_SECTION,
        tr("panel.inspector.material.material_and_blend"),
    );
    let header_rect = Rect::new(x, y, w, header_h);
    paint_section_header(&header, header_rect, scene, text_system, theme);
    // ⚠️ **A DOBRA do corpo** — o escopo recorta a cena E o hit, e escala o `y` de saída, para
    //    que tudo o que está por baixo suba junto. Ver `SectionFold`.
    // ⚠️ **Pergunta o `t`, e NUNCA o `is_collapsed`:** ao clicar para fechar o flag semântico vira
    //    neste mesmo quadro enquanto o `t` ainda desce, então um corpo gateado no flag sumiria de
    //    repente por baixo de um chevron a rodar — as duas metades a discordar outra vez.
    let Some(fold) = SectionFold::begin(
        store,
        core_ids::INSP_LIVE_BLEND_SECTION,
        x,
        w,
        y + header_h,
        scene,
        hit_index,
    ) else {
        return y + header_h;
    };
    let mut yy = y + header_h;
    let h = ROW_H_PX;
    // ⚠️ **O vão entre dois controlos é a porta `control_gap_px` (3 px)**, e não o
    //    `Spacing::Xs` (4) escrito à mão — ordem do dono, 2026-09-07. Esta secção é
    //    anterior à porta. Ver `every_stack_of_rows_asks_the_rhythm`.
    let row_gap = ph2d_tokens::control_gap_px();
    let label_font = TypeToken::Sm.px();

    // Blend Mode — 6-way segmented (Mix/Add/Subtract/Multiply/Screen/
    // Premult); `SegmentedAdaptive` reflows the wider labels onto extra
    // rows at the Inspector's narrow width. Selecting Mix detaches the
    // optional component (default).
    // ⚠️ **A secção tem UMA linha** — a declaração é dela na mesma: ver
    //    [`ph2d_editor_core::property_row::Seccao`].
    let sec = ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        1,
        &[tr("component.field.blend_mode.1")],
    );
    // ⚠️ **Nenhum aceso = «misto»**: cinco sprites com três blends não acendem «Mix» como se
    // concordassem (auditoria `docs/Sprite_projeto/20` §3.3).
    let nomes = BLEND_LABELS.map(TextKey::tr);
    yy = super::rows::seg_row_misto(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        TextKey::new("component.field.blend_mode.1"),
        (
            core_ids::INSP_LIVE_BLEND_SECTION,
            &ids::INSP_SAMPLE_BLEND,
            &nomes,
        ),
        (!info.mixed.blend).then(|| usize::from(info.blend_tag).min(BLEND_LABELS.len() - 1)),
        sec,
        row_gap,
    );

    // Material slot — read-only placeholder. The renderer is
    // fixed-function (no material / shader runtime yet, spec §3.10);
    // Material + InstanceShaderParams land with that future runtime.
    paint_text(
        text_system,
        scene,
        tr("panel.inspector.material.material_default_shader_runtime_pending"),
        x,
        yy + (h - label_font) * 0.5,
        label_font,
        w,
        resolve(ColorToken::Text3, theme),
    );
    yy += h + SECTION_BOTTOM_PAD_PX;
    fold.finish(store, scene, hit_index, yy)
}
