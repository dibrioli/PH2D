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

/// **Um rótulo por TAG de `ph2d_ecs::FilterMode`, indexado pela tag `0..=6` — `None` = a tag não é
/// oferecida.**
///
/// ⚠️ Hardcoded aqui de propósito — o painel é chrome e não depende do `ph2d-ecs` (mesma razão do
/// `BLEND_LABELS` da §10). Quem guarda as duas listas honestas é o gate
/// `the_filter_segmented_offers_exactly_the_modes_that_render_differently`, no shell, que é a única
/// crate que vê o painel **e** as leis do renderer.
///
/// # ⛔⛔ Por que isto é `Option` e não uma lista mais curta
///
/// **A POSIÇÃO É A TAG** — o despacho (`event_ordering.rs`) deriva o que escrever de
/// `INSP_SAMPLE_FILTER.position(|&o| o == id)`, e aquele array vive no `ph2d-editor-core` com as
/// sete entradas. Encurtar esta lista faria o `zip` casar o rótulo `n+1` com o id `n`: o segmento
/// rotulado *«Lin+Aniso»* passaria a escrever a tag do vizinho, **sem uma linha de erro**. O buraco
/// explícito é o que mantém as duas listas indexadas pela mesma coisa.
///
/// # ⛔ O buraco no `5`
///
/// Era *«Near+Aniso»*, e é um item de menu **fisicamente inalcançável**: o wgpu recusa
/// `anisotropy_clamp > 1` sem os três filtros `Linear`, e *ampliar por ponto* é o que aquele nome
/// promete. O sampler dela é campo a campo o da `3 Near+Mip` — dois segmentos com nomes diferentes
/// e o mesmo desenho, que é a doença que a §4 desta seção já mediu com os rótulos repetidos.
pub const FILTER_LABELS: [Option<&str>; 7] = [
    Some("Inherit"),
    Some("Nearest"),
    Some("Linear"),
    Some("Near+Mip"),
    Some("Lin+Mip"),
    // ⛔ 5 — `Near+Aniso`, RETIRADO. ⚠️ Não reaproveite o slot: a tag é o formato de arquivo.
    None,
    Some("Lin+Aniso"),
];

/// **Um rótulo por variante de `ph2d_ecs::RepeatMode`**, na ordem das tags `0..=3`.
pub const REPEAT_LABELS: [&str; 4] = ["Inherit", "Clamp", "Repeat", "Mirror"];

/// **O índice que significa «nenhum segmento aceso»** — a afordância de divergência.
///
/// ⚠️ O `SegmentedAdaptive` trata um índice fora de alcance como *nenhuma seleção*; o `Tabs` **não
/// sabe**, porque o `selected()` dele clampa (`idx.min(len-1)`). Foi por isso que as duas rows desta
/// seção passaram a adaptativas em 2026-08-21: enquanto a Repeat era um `Tabs`, ela era
/// **incapaz** de dizer «misto», e o painel acendia o valor da primária como se toda a seleção
/// concordasse (auditoria `docs/Sprite_projeto/20` §3.3).
const NOTHING_LIT: usize = usize::MAX;

/// **Qual segmento acende para uma tag de filtro** — a contagem dos rótulos oferecidos ABAIXO
/// dela, porque a lista pintada salta o buraco do `5`.
///
/// Devolve [`NOTHING_LIT`] para uma tag que não é oferecida (o `5` aposentado, ou qualquer coisa
/// fora de alcance): o `SegmentedAdaptive` trata isso como *nenhum segmento aceso*.
///
/// # Por que isto é uma função nomeada e não uma expressão
///
/// ⚠️ O defeito que shipou era exatamente uma expressão: `.selected((filter_tag as usize).min(2))`.
/// Uma expressão dentro do pintor **não é observável de fora** — nenhum teste podia perguntar-lhe
/// nada, porque a seleção não sai no `paint`. Com nome, ela responde.
///
/// # ⛔ E por que o `min` MORREU
///
/// Com um buraco na lista, um clamp acenderia o **último** segmento (*«Lin+Aniso»*) para uma tag
/// que não é ele — a mesma família do `.min(2)` que acendia *«Linear»* sobre pixel duro. *Um clamp
/// é uma rede enquanto a lista é densa; sobre uma lista com buraco ele volta a ser uma mentira.*
/// Não acender nada é a resposta honesta, e é a afordância que esta seção já usa para «misto».
pub(crate) fn filter_selected_index(filter_tag: u8) -> usize {
    let tag = usize::from(filter_tag);
    if FILTER_LABELS.get(tag).copied().flatten().is_none() {
        return NOTHING_LIT;
    }
    FILTER_LABELS[..tag].iter().filter(|l| l.is_some()).count()
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
        // ⚠️ **`zip` com o array INTEIRO e depois `filter_map`** — o par `(id, rótulo)` tem de ser
        // formado ANTES de descartar o buraco, senão o rótulo `n+1` casa com o id `n` e o segmento
        // passa a escrever o modo do vizinho (vide o doc de `FILTER_LABELS`).
        ids::INSP_SAMPLE_FILTER
            .iter()
            .zip(FILTER_LABELS)
            .filter_map(|(&id, label)| label.map(|l| SegmentedOption::new(id, l)))
            .collect(),
    )
    .selected(if info.mixed.filter {
        NOTHING_LIT
    } else {
        filter_selected_index(info.filter_tag)
    });
    let (control_w, dot) = ph2d_editor_core::widget::form_row_columns(x, w, yy, h);
    let seg_h = paint_segmented_adaptive(
        &seg,
        Rect::new(x, yy, control_w, h),
        scene,
        text_system,
        theme,
        store,
        hit_index,
    );
    ph2d_editor_core::widget::paint_decorator_dot(scene, theme, dot);
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
    let repeat_seg = SegmentedAdaptive::new(
        ids::INSP_LIVE_SAMPLING_SECTION,
        "Texture Repeat",
        ids::INSP_SAMPLE_REPEAT
            .iter()
            .zip(REPEAT_LABELS)
            .map(|(&id, label)| SegmentedOption::new(id, label))
            .collect(),
    )
    .selected(if info.mixed.repeat {
        NOTHING_LIT
    } else {
        usize::from(info.repeat_tag).min(REPEAT_LABELS.len() - 1)
    });
    let (repeat_w, repeat_dot) = ph2d_editor_core::widget::form_row_columns(x, w, yy, h);
    let repeat_h = paint_segmented_adaptive(
        &repeat_seg,
        Rect::new(x, yy, repeat_w, h),
        scene,
        text_system,
        theme,
        store,
        hit_index,
    );
    ph2d_editor_core::widget::paint_decorator_dot(scene, theme, repeat_dot);
    yy += repeat_h + row_gap;

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

    /// **A lista OFERECIDA, na ordem em que o pintor a monta** — a mesma expressão do
    /// `paint_sampling_section`, para o teste falar da mesma coisa que o ecrã.
    fn oferecidos() -> Vec<&'static str> {
        FILTER_LABELS.iter().flatten().copied().collect()
    }

    /// **Cada tag acende o SEU segmento** — a identidade que o `.min(2)` quebrava, agora contada
    /// por cima do buraco do `5`.
    ///
    /// ⚠️ Este é o teste que não podia existir enquanto a regra era uma expressão dentro do
    /// pintor: a seleção não sai no `paint`, por isso nenhum gate de costura a alcançava. Ele
    /// mata a mutação exata que shipou (`.min(2)` → tags 3 e 5 acendendo «Linear»).
    ///
    /// ⚠️ **E mede o RÓTULO, não o índice.** Um gate sobre o número passaria numa lista
    /// re-ordenada; o que o artista lê é o nome que acende.
    #[test]
    fn every_offered_filter_tag_lights_the_segment_that_carries_its_own_label() {
        let offered = oferecidos();
        for (tag, label) in FILTER_LABELS.iter().enumerate() {
            let Some(label) = label else { continue };
            let t = u8::try_from(tag).expect("as tags cabem num u8");
            let i = filter_selected_index(t);
            assert_eq!(
                offered.get(i).copied(),
                Some(*label),
                "a tag {tag} («{label}») acende o segmento {i} («{}») — foi assim que as tags 3 e \
                 5 (que o renderer desenha com pixel duro) acenderam «Linear» durante meses",
                offered.get(i).copied().unwrap_or("<fora de alcance>")
            );
        }
    }

    /// ⛔ **O TAG APOSENTADO NÃO ACENDE NADA** — nem o vizinho, nem o último.
    ///
    /// ⚠️ Um clamp devolveria o **último** segmento (*«Lin+Aniso»*) e o painel voltaria a afirmar
    /// um modo que não é o do objecto — a mesma família do `.min(2)`. E o `5` não é hipotético: é
    /// o que um `.ph2dproj` gravado antes desta cura tem escrito.
    #[test]
    fn the_retired_and_the_impossible_tags_light_nothing() {
        const RETIRED: u8 = 5;
        assert!(
            FILTER_LABELS[RETIRED as usize].is_none(),
            "o slot aposentado voltou a ter rotulo"
        );
        assert_eq!(filter_selected_index(RETIRED), NOTHING_LIT);
        assert_eq!(filter_selected_index(200), NOTHING_LIT);
        // ⚠️ **A metade JUSTA:** as tags que EXISTEM continuam a acender alguma coisa. Sem ela um
        // `filter_selected_index` que devolvesse sempre `NOTHING_LIT` passaria este gate.
        for (tag, label) in FILTER_LABELS.iter().enumerate() {
            if label.is_some() {
                assert_ne!(
                    filter_selected_index(tag as u8),
                    NOTHING_LIT,
                    "a tag {tag} e' oferecida e nao acende segmento nenhum"
                );
            }
        }
    }
}
