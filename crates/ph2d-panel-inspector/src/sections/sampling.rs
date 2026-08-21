//! Sampling — Inspector §9 section painter (Sprite Inspector v2 W3, spec §3.9 / §9.9).
//! Texture Filter + Texture Repeat como segmentados vindos do snapshot, mais o par UV.
//!
//! # Duas afirmações falsas saíram daqui em 2026-08-21 (auditoria, `docs/Sprite_projeto/20` §2)
//!
//! 1. ⚠️ **Este doc dizia que «mipmap/aniso map to their base filter, so the segmented mirrors what
//!    actually renders».** Não mapeava: `.min(2)` **clampava**, e como o renderer manda as tags
//!    ímpares `1 | 3 | 5` para `Nearest`, o painel acendia **«Linear»** exatamente sobre as sprites
//!    que desenhavam com pixel duro. Hoje há um segmento por variante e a posição **é** a tag —
//!    zero conversão, nada para desalinhar.
//! 2. ⛔ **A linha «Anti-halo: enabled (atlas-level)» foi REMOVIDA, não corrigida.** Ela afirmava um
//!    facto sobre um mecanismo que **não existe em lado nenhum do repo**: nem implementação, nem
//!    `struct SpriteAtlas`, nem crate de cooker de atlas. E era pintada para toda sprite, incluindo
//!    `Individual` e `CookedTexture`, que não estão em atlas nenhum. A spec §9.3
//!    ([`09_sampling_e_material.md`](../../../../docs/Sprite_projeto/09_sampling_e_material.md))
//!    define-a como flag **de asset**; quando o cooker a entregar, a linha volta **lendo-a**.
//!    *Um rótulo só pode prometer o que o modelo entrega — e não havia modelo.*
//!    ⚠️ Este módulo já tinha pago esta lição uma vez: o literal `"RGBA8"` que o plano 17 §5 apanhou.

use super::*;
use ph2d_editor_core::screens::hero::InspectorSamplingInfo;
use ph2d_editor_core::widget::SectionFold;
use ph2d_editor_core::widget::{SegmentedAdaptive, SegmentedOption, paint_segmented_adaptive};

/// **Um rótulo por variante de `ph2d_ecs::FilterMode`, na ordem das tags `0..=6`.**
///
/// ⚠️ Hardcoded aqui de propósito — o painel é chrome e não depende do `ph2d-ecs` (mesma razão do
/// `BLEND_LABELS` da §10). Quem guarda as duas contagens honestas é o gate
/// `the_filter_segmented_offers_every_mode_the_engine_has`, no shell, que é a única crate que vê
/// as duas.
pub const FILTER_LABELS: [&str; 7] = [
    "Inherit",
    "Nearest",
    "Linear",
    "Near+Mip",
    "Lin+Mip",
    "Near+Aniso",
    "Lin+Aniso",
];

/// **Qual segmento acende para uma tag de filtro.** Identidade, porque a posição **é** a tag.
///
/// # Por que isto é uma função nomeada e não uma expressão
///
/// ⚠️ O defeito que shipou era exatamente uma expressão: `.selected((filter_tag as usize).min(2))`.
/// Uma expressão dentro do pintor **não é observável de fora** — nenhum teste podia perguntar-lhe
/// nada, porque a seleção não sai no `paint`. Com nome, ela responde.
///
/// O `min` que sobrou não é o mesmo: com um segmento por variante ele só é alcançável por uma tag
/// que o `FilterMode` não tem, e o gate `the_position_in_the_segmented_is_the_tag_itself` (shell)
/// prende as duas contagens. *O clamp deixou de ser a regra e passou a ser a rede.*
pub(crate) fn filter_selected_index(filter_tag: u8) -> usize {
    usize::from(filter_tag).min(FILTER_LABELS.len() - 1)
}

/// Label-above row with two NumberInputs (X / Y) for a UV scale/offset
/// pair. Returns the next `y`.
#[allow(clippy::too_many_arguments)]
fn uv_pair_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    label: &str,
    id_x: NodeId,
    id_y: NodeId,
) -> f32 {
    let h = ROW_H_PX;
    let label_font = TypeToken::Sm.px();
    let label_h = label_font + Spacing::Xs.px();
    paint_text(
        text_system,
        scene,
        label,
        x,
        y + (label_h - label_font) * 0.5,
        label_font,
        w,
        resolve(ColorToken::Text2, theme),
    );
    let row_y = y + label_h;
    let gap = Spacing::Sm.px();
    let cw = ((w - gap) * 0.5).max(0.0);
    for (i, id) in [id_x, id_y].into_iter().enumerate() {
        let rect = Rect::new(x + (cw + gap) * i as f32, row_y, cw, h);
        hit_index.register(id, rect);
        let (state, value, buffer, caret, anchor) = read_number_input(store, id);
        let input = NumberInput::new(id, "", value)
            .step(0.1) // LITERAL-PX-OK: UV step
            .visual((state, store.hover_live(id)));
        paint_number_input_with_buffer(
            &input,
            Some(buffer),
            caret,
            anchor,
            rect,
            scene,
            text_system,
            theme,
        );
    }
    row_y + h + Spacing::Sm.px()
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_sampling_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorSamplingInfo,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: section header band height
    let color_id = ids::INSP_LIVE_SAMPLING_COLOR;
    let rgba = store
        .widget_color(color_id)
        .unwrap_or([0x88, 0x88, 0x88, 0xff]); // LITERAL-COLOR-OK: neutral default section accent
    let header = section_header(store, ids::INSP_LIVE_SAMPLING_SECTION, "Sampling").color(rgba);
    let header_rect = Rect::new(x, y, w, header_h);
    paint_section_header(&header, header_rect, scene, text_system, theme);
    if let Some(circle_rect) = ph2d_editor_core::widget::color_circle_hit_rect(&header, header_rect)
    {
        hit_index.register(color_id, circle_rect);
    }
    // ⚠️ **A DOBRA do corpo** — o escopo recorta a cena E o hit, e escala o `y` de saída, para
    //    que tudo o que está por baixo suba junto. Ver `SectionFold`.
    // ⚠️ **Pergunta o `t`, e NUNCA o `is_collapsed`:** ao clicar para fechar o flag semântico vira
    //    neste mesmo quadro enquanto o `t` ainda desce, então um corpo gateado no flag sumiria de
    //    repente por baixo de um chevron a rodar — as duas metades a discordar outra vez.
    let Some(fold) = SectionFold::begin(
        store,
        ids::INSP_LIVE_SAMPLING_SECTION,
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
    let row_gap = Spacing::Sm.px();
    let label_font = TypeToken::Sm.px();
    let label_color = resolve(ColorToken::Text2, theme);
    // The label sits on its OWN short row ABOVE the segmented (label-
    // above layout) — using a fraction of the control row overlapped
    // the text onto the buttons (smoke 2026-05-30).
    let label_h = label_font + Spacing::Xs.px();

    // Texture Filter — um segmento por variante de `FilterMode`, tags `0..=6` (vide FILTER_LABELS).
    paint_text(
        text_system,
        scene,
        "Texture Filter",
        x,
        yy + (label_h - label_font) * 0.5,
        label_font,
        w,
        label_color,
    );
    yy += label_h;
    // ⚠️ **Segmentado ADAPTATIVO, como o Blend Mode da §10** — sete opções não cabem numa fila de
    // `Tabs` num painel estreito, e o adaptativo reflui. A alternativa (manter três abas) foi o que
    // produziu o `.min(2)` que mentia.
    let seg = SegmentedAdaptive::new(
        ids::INSP_LIVE_SAMPLING_SECTION,
        "Texture Filter",
        ids::INSP_SAMPLE_FILTER
            .iter()
            .zip(FILTER_LABELS)
            .map(|(&id, label)| SegmentedOption::new(id, label))
            .collect(),
    )
    .selected(filter_selected_index(info.filter_tag));
    let seg_h = paint_segmented_adaptive(
        &seg,
        Rect::new(x, yy, w, h),
        scene,
        text_system,
        theme,
        store,
        hit_index,
    );
    yy += seg_h + row_gap;

    // Texture Repeat — Inherit / Disabled / Enabled / Mirror (0/1/2/3).
    paint_text(
        text_system,
        scene,
        "Texture Repeat",
        x,
        yy + (label_h - label_font) * 0.5,
        label_font,
        w,
        label_color,
    );
    yy += label_h;
    let repeat_rect = Rect::new(x, yy, w, h);
    let repeat_tabs = Tabs::new(
        NodeId(0),
        "",
        vec![
            TabItem::new(ids::INSP_SAMPLE_REPEAT[0], "Inherit"),
            TabItem::new(ids::INSP_SAMPLE_REPEAT[1], "Clamp"),
            TabItem::new(ids::INSP_SAMPLE_REPEAT[2], "Repeat"),
            TabItem::new(ids::INSP_SAMPLE_REPEAT[3], "Mirror"),
        ],
    )
    .variant(TabsVariant::Segmented)
    .selected((info.repeat_tag as usize).min(3));
    paint_tabs(&repeat_tabs, repeat_rect, scene, text_system, theme);
    for (i, item) in repeat_tabs.items.iter().enumerate() {
        hit_index.register(item.id, repeat_tabs.tab_rect(repeat_rect, i));
    }
    yy += h + row_gap;

    // UV tiling (scale > 1 tiles) + scroll (offset) — W3 UvTransform. The
    // repeat segmented above picks how the tiled UV wraps.
    yy = uv_pair_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        "UV Scale",
        ids::INSP_SAMPLE_UV_SCALE_X,
        ids::INSP_SAMPLE_UV_SCALE_Y,
    );
    yy = uv_pair_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        "UV Offset",
        ids::INSP_SAMPLE_UV_OFFSET_X,
        ids::INSP_SAMPLE_UV_OFFSET_Y,
    );

    // ⛔ **AQUI ficava «Anti-halo: enabled (atlas-level)», e não volta como literal.** O doc de
    // módulo no topo conta o mecanismo; o gatilho para a linha renascer é o cooker de atlas passar
    // a carregar a flag da spec §9.3 — e então ela **lê** a flag em vez de a afirmar.
    yy += SECTION_BOTTOM_PAD_PX;
    fold.finish(store, scene, hit_index, yy)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Cada tag acende o SEU segmento** — a identidade que o `.min(2)` quebrava.
    ///
    /// ⚠️ Este é o teste que não podia existir enquanto a regra era uma expressão dentro do
    /// pintor: a seleção não sai no `paint`, por isso nenhum gate de costura a alcançava. Ele
    /// mata a mutação exata que shipou (`.min(2)` → tags 3 e 5 acendendo «Linear»).
    #[test]
    fn every_filter_tag_lights_its_own_segment() {
        for tag in 0..FILTER_LABELS.len() {
            let t = u8::try_from(tag).expect("as tags cabem num u8");
            assert_eq!(
                filter_selected_index(t),
                tag,
                "a tag {tag} acende o segmento «{}» em vez do seu — foi assim que as tags 3 e 5 \
                 (que o renderer desenha com pixel duro) acenderam «Linear» durante meses",
                FILTER_LABELS[filter_selected_index(t)]
            );
        }
    }

    /// **Uma tag impossível não escolhe às cegas** — ela cai no último segmento em vez de indexar
    /// fora do array. A rede, não a regra.
    #[test]
    fn a_tag_the_engine_cannot_produce_is_clamped_not_indexed_out_of_range() {
        assert_eq!(filter_selected_index(200), FILTER_LABELS.len() - 1);
    }
}
