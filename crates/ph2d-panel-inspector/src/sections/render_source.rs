//! Render Source (+ Region) — Inspector section painter (split from sections.rs,
//! architecture_panel_loc_cap). Logic verbatim; behavior unchanged.

use super::render_source_precision::paint_precision_row;
use super::*;
use ph2d_editor_core::widget::SectionFold;

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_render_source_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorSpriteInfo,
) -> f32 {
    // Match Transform's row-label style — Sm font, Text2 color — so
    // Render Source feels visually identical (user feedback 2026-05-24).
    let line_font = TypeToken::Sm.px();
    let label_font = TypeToken::Sm.px();
    let row_gap = Spacing::Xs.px();
    let row_h = line_font + row_gap;
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: section header band height
    let color_id = ids::INSP_LIVE_RENDER_COLOR;
    let rgba = store
        .widget_color(color_id)
        .unwrap_or([0x88, 0x88, 0x88, 0xff]); // LITERAL-COLOR-OK: neutral default for unconfigured section accent
    let header = section_header(store, ids::INSP_LIVE_RENDER_SECTION, "Render Source").color(rgba);
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
        ids::INSP_LIVE_RENDER_SECTION,
        x,
        w,
        y + header_h,
        scene,
        hit_index,
    ) else {
        return y + header_h;
    };
    // No inner separator — orchestrator draws it AFTER section content.
    let mut cur_y = y + header_h;

    // ⛔ **A fileira adaptativa MORREU aqui em 2026-09-01.** Ela empilhava rótulo e valor quando
    // não cabiam lado a lado, e existia **só** para as duas linhas de proveniência — que hoje são
    // a ranhura da textura e a irmã do tamanho, ambas desenhadas por [`paint_provenance`]. *Uma
    // abstracção sem consumidor é código que a próxima pessoa lê e tenta perceber.*
    cur_y = paint_strategy_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        info,
        x,
        w,
        cur_y,
        label_font,
    );
    cur_y = paint_provenance(
        scene,
        text_system,
        theme,
        hit_index,
        info,
        x,
        w,
        cur_y,
        label_font,
        row_h,
        row_gap,
    );

    // A amostragem de REGIÃO mora num irmão — ver [`paint_region_rows`].
    cur_y = paint_region_rows(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        info,
        x,
        w,
        cur_y,
        label_font,
        row_gap,
    );

    // **O formato volta a ser uma ESCOLHA — porque agora existe modelo por trás dela** (plano
    // `docs/Sprite_projeto/18` W5).
    //
    // ⚠️ **A história completa, porque este controle já mentiu duas vezes:**
    //
    // 1. Nasceu como par segmentado **sem arm de dispatch em lado nenhum**: pintado, registado,
    //    hit-indexado, e clicar não fazia nada — nem um toast. O aceso era o literal `true`.
    // 2. O plano 17 §5 removeu-o e pôs uma linha de FACTO no lugar. Certo na altura — mas o facto
    //    era **derivado da estratégia**, e por isso dizia "RGBA8" para toda a gente.
    //
    // O que mudou não foi a opinião sobre o botão: foi o `Rgba16Float` no store, o
    // `Asset::ImageRgba16`, o `PixelPayload` no ficheiro e a conversão nos dois sentidos.
    // *Um controle nasce quando o modelo o entrega, não quando o desenho o imagina.*
    cur_y = paint_precision_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        info,
        x,
        w,
        cur_y,
        label_font,
    );

    // **A SPRITE COMO FONTE DE LUZ** (plano `docs/Sprite_projeto/18` W8). Vizinha do `Format` de
    // propósito: a emissão é a única coisa no app que precisa da folga acima de 1.0 que os 16 bits
    // dão. ⚠️ Vive num ficheiro IRMÃO porque este está no tecto de LOC (`emissive_row.rs`).
    cur_y = super::emissive_row::paint_emissive_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
    );

    let reimport_h = 30.0_f32; // LITERAL-PX-OK: Reimport button height
    let btn_rect = Rect::new(x, cur_y, w, reimport_h);
    let id = ids::INSP_RENDER_SOURCE_REIMPORT;
    let state = if !info.can_reimport {
        (ButtonState::Disabled, ph2d_editor_core::motion::SETTLED)
    } else {
        store.button_visual(id)
    };
    hit_index.register(id, btn_rect);
    let btn = Button::new(id, "Reimport at current px/m")
        .kind(ButtonKind::Default)
        .visual(state);
    paint_button(&btn, btn_rect, scene, text_system, theme);
    fold.finish(
        store,
        scene,
        hit_index,
        cur_y + reimport_h + SECTION_BOTTOM_PAD_PX,
    )
}

/// **A linha "Strategy"** — o rótulo e o grupo segmentado de três (Atlas / Individual /
/// Hand-packed). Devolve o `y` a seguir a ela.
///
/// ⚠️ Saiu do [`paint_render_source_section`] por medição: o `cargo fmt --all` re-expandiu a
/// função-mãe de 199 para 205 linhas, contra um tecto de 200 — a memória
/// `feedback_loc_cap_split_not_allowlist_and_fmt_reexpands` avisa exatamente por este caminho, e
/// a cura registada é **cortar**, não tolerar. Este é o segundo corte desta função (o primeiro foi
/// a `paint_region_num_cell`), e ambos são o *"per-row split"* que a tolerância antiga nomeava
/// como diferido.
/// **A linha `Format` — o par de precisão** (plano `docs/Sprite_projeto/18` W5).
///
/// ⚠️ **Uma textura cozida não oferece escolha nenhuma**: ela é BC/ASTC/ETC2 e a precisão depende do
/// tier resolvido. Pintar-lhe o par seria a mesma mentira que o plano 17 §5 removeu, então ela cai
/// na linha de facto, como as outras de proveniência.
///
/// ⚠️ **A nota de custo é parte do controle, não decoração.** Converter para 16 bits **dobra a
/// memória** da imagem e **força a estratégia a `Individual`** (o atlas é uma textura com um
/// formato, §3.3). O artista tem de ler isso **antes** de carregar, não descobrir depois pelo painel
/// a mudar sozinho — *uma consequência que só aparece depois do clique lê-se como um bug*.
#[allow(clippy::too_many_arguments)]
fn paint_strategy_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    info: &InspectorSpriteInfo,
    x: f32,
    w: f32,
    mut cur_y: f32,
    label_font: f32,
) -> f32 {
    paint_text(
        text_system,
        scene,
        "Strategy",
        x,
        cur_y,
        label_font,
        w,
        resolve(ColorToken::Text2, theme),
    );
    cur_y += label_font + SECTION_LABEL_TO_CONTROL_PX;
    // ⚠️ **Uma textura cozida não tem estratégia autorável — e por isso não pinta botões.**
    //
    // Antes de 2026-08-21 os três botões saíam **igualmente acesos e com nada selecionado** (os
    // três `matches!` abaixo são falsos para `CookedTexture`), e o artista só descobria que eram
    // read-only depois de clicar. O par **Format**, quinze linhas acima, já resolve o mesmo caso
    // do mesmo modo: esconde o controlo e afirma o facto. *Duas linhas irmãs na mesma seção não
    // podem discordar sobre o que é editável* (auditoria `docs/Sprite_projeto/20` §4.6).
    //
    // ⛔ A alternativa — pintar os três a cinzento — exigia um eixo de `enabled` no
    // `paint_segmented_group_adaptive`, que é partilhado; e um controlo desactivado que continua
    // a despachar (o `strategy_click` roteia o cozido de propósito, para o toast sair) seria a
    // pior das três hipóteses: *dimmed que despacha mente*.
    if matches!(info.source_kind, InspectorSpriteSource::CookedTexture) {
        paint_text(
            text_system,
            scene,
            "From the asset pipeline \u{00b7} read-only",
            x,
            cur_y,
            label_font,
            w,
            resolve(ColorToken::Text3, theme),
        );
        return cur_y
            + label_font
            + ph2d_editor_core::widget::panel_chrome::SECTION_INNER_ROW_GAP_PX;
    }
    // Adaptive segmented GROUP — when the panel is narrow, drops
    // "Hand-packed" (the longest) to its own row instead of wrapping
    // the label. Returns the actual height used.
    let (strategy_w, strategy_dot) =
        ph2d_editor_core::widget::form_row_columns(x, w, cur_y, ROW_H_PX);
    let strategy_h = paint_segmented_group_adaptive(
        Rect::new(x, cur_y, strategy_w, ROW_H_PX),
        &[
            (
                "Atlas",
                matches!(info.source_kind, InspectorSpriteSource::Atlas { .. }),
                ids::INSP_RENDER_STRATEGY_ATLAS,
            ),
            (
                "Individual",
                matches!(info.source_kind, InspectorSpriteSource::Individual { .. }),
                ids::INSP_RENDER_STRATEGY_INDIVIDUAL,
            ),
            (
                "Hand-packed",
                matches!(info.source_kind, InspectorSpriteSource::HandPacked { .. }),
                ids::INSP_RENDER_STRATEGY_HANDPACKED,
            ),
        ],
        scene,
        text_system,
        theme,
        store,
        hit_index,
    );
    // Inter-row gap inside Render Source — matches Transform's row_gap
    // (SECTION_INNER_ROW_GAP_PX) so Render Source feels like Transform.
    ph2d_editor_core::widget::paint_decorator_dot(scene, theme, strategy_dot);
    cur_y + strategy_h + ph2d_editor_core::widget::panel_chrome::SECTION_INNER_ROW_GAP_PX
}

/// **Uma célula X/Y/W/H da região** — rótulo do eixo + o campo numérico.
///
/// ⚠️ Era uma closure DENTRO do `paint_render_source_section`, e sair paga o tecto de 200 LOC que
/// a wave do hover fez a função cruzar. É também o *"per-row split"* que a tolerância dela nomeia
/// como diferido desde 2026-07-10 — feito para UMA row.
#[allow(clippy::too_many_arguments)]
fn paint_region_num_cell(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    cell: Rect,
    axis: &str,
    id: NodeId,
    label_font: f32,
    theme: Theme,
) {
    // ⚠️ Lido AQUI e não recebido: é um TOKEN, e o chamador não tem opinião sobre ele.
    let axis_w = Spacing::Lg.px(); // mini X/Y/W/H label column
    paint_text(
        text_system,
        scene,
        axis,
        cell.x,
        cell.y + (cell.h - label_font) * 0.5,
        label_font,
        axis_w,
        resolve(ColorToken::Text2, theme),
    );
    let input_rect = Rect::new(cell.x + axis_w, cell.y, (cell.w - axis_w).max(0.0), cell.h);
    hit_index.register(id, input_rect);
    let (state, value, buffer, caret, anchor) = read_number_input(store, id);
    let input = NumberInput::new(id, "", value)
        .step(1.0)
        .visual((state, store.hover_live(id)));
    paint_number_input_with_buffer(
        &input,
        Some(buffer),
        caret,
        anchor,
        input_rect,
        scene,
        text_system,
        theme,
    );
}

/// ⭐⭐⭐ **A PROVENIÊNCIA** — a ranhura de *de onde os pixels vêm* (e onde se largam outros),
/// mais o tamanho que eles tinham na origem.
///
/// # Ela era um FACTO e passou a ser um ALVO (plano `docs/Components/07`, wave B3)
///
/// A linha *Storage* dizia `Individual · texture 5` e mais nada. O plano pede *«queda num campo do
/// Inspector ⇒ preenche»*, e este é o campo: o que ele nomeia é exactamente o que uma imagem
/// largada substitui.
///
/// ⚠️ **A moldura é a affordance, e ela é honesta**: uma ranhura diz *«põe aqui»* sem prometer um
/// clique. ⛔ **Não é um `Button`** — pintá-la como botão prometeria a acção que ela não tem, que é
/// a 1.ª espécie de knob morto da caça de 2026-08-30.
///
/// ⚠️ **O texto continua a ser o MESMO facto**, formatado no mesmo sítio: a ranhura não inventa um
/// segundo vocabulário para dizer de onde vêm os pixels.
#[allow(clippy::too_many_arguments)]
fn paint_provenance(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    info: &InspectorSpriteInfo,
    x: f32,
    w: f32,
    y: f32,
    label_font: f32,
    row_h: f32,
    row_gap: f32,
) -> f32 {
    // Cleaner phrasing — strategy name + key/id separated by middle dot (the only ASCII-safe
    // non-ASCII glyph allowed in UI strings; vide no_tofu_glyphs gate).
    let detail = match info.source_kind {
        InspectorSpriteSource::Atlas { key } => format!("Atlas \u{00b7} key {key}"),
        InspectorSpriteSource::Individual { texture_id } => {
            format!("Individual \u{00b7} texture {texture_id}")
        }
        // ⚠️ O NOME, não os índices: é por ele que o artista reencontra o desenho no Aseprite. Os
        // números só aparecem se o rótulo faltar (uma folha que o projeto trouxe mas a sessão não
        // tem), e aí eles são a informação honesta que sobra.
        InspectorSpriteSource::HandPacked { sheet, region } => match &info.sheet_label {
            Some(label) => format!("Hand-packed \u{00b7} {label}"),
            None => format!("Hand-packed \u{00b7} sheet {sheet} \u{00b7} region {region}"),
        },
        // W2.T2: tier-cooked KTX2 — read-only marker, no key/id shown.
        InspectorSpriteSource::CookedTexture => "Cooked texture".to_string(),
    };
    paint_text(
        text_system,
        scene,
        STORAGE_LABEL,
        x,
        y,
        label_font,
        w,
        resolve(ColorToken::Text2, theme),
    );
    let slot = Rect::new(x, y + label_font + row_gap, w, row_h);
    // ⭐ Raio e moldura pela porta do TEMA: o slot é plano num tema moderno.
    let slot_radius = ph2d_editor_core::paint::frame_radius(theme, Radius::Sm.px());
    fill_rounded_rect(scene, slot, slot_radius, resolve(ColorToken::Bg2, theme));
    ph2d_editor_core::paint::stroke_frame(
        scene,
        slot,
        slot_radius,
        theme,
        ph2d_tokens::visuals::Feel::Rest,
        SLOT_BORDER_PX,
        resolve(ColorToken::Border, theme),
    );
    paint_text(
        text_system,
        scene,
        &detail,
        slot.x + Spacing::Xs.px(),
        slot.y + (slot.h - label_font) * 0.5,
        label_font,
        (slot.w - Spacing::Xs.px() * 2.0).max(0.0),
        resolve(ColorToken::Text1, theme),
    );
    // ⭐⭐⭐ **É AQUI que ela vira alvo.** O `HitIndex` é a porta única de *«o que está debaixo do
    // cursor»*, e é ela que dá de graça o recorte do corpo e a oclusão por um painel de cima.
    // ⛔ Sem `populate`: quem consome este id é o caminho da QUEDA, não o de clique — a mesma
    // classe das *swatches* do picker, e o `HIT_PARITY_ALLOW` nomeia-a.
    hit_index.register(ids::INSP_RENDER_TEXTURE_SLOT, slot);
    let mut cur_y = slot.y + slot.h + row_gap;
    // ⚠️ **O TAMANHO de origem fica ao lado da ranhura**, e não numa função irmã: as duas são a
    // mesma pergunta — *de onde vêm estes pixels, e que tamanho tinham* —, e separá-las custou ao
    // pai o tecto de 200 LOC por uma chamada.
    if let Some((pw, ph)) = info.source_pixels {
        paint_text(
            text_system,
            scene,
            SOURCE_SIZE_LABEL,
            x,
            cur_y,
            label_font,
            w,
            resolve(ColorToken::Text2, theme),
        );
        paint_text(
            text_system,
            scene,
            &format!("{pw} \u{00d7} {ph} px"),
            x,
            cur_y + label_font + row_gap,
            label_font,
            w,
            resolve(ColorToken::Text1, theme),
        );
        cur_y += (label_font + row_gap) * 2.0;
    }
    cur_y
}

/// **A amostragem de REGIÃO** (spec §3.3) — o toggle + os quatro campos px + o Filter Clip.
///
/// Saiu do corpo de [`paint_render_source_section`] pelo cap de fn do painel, e é o *per-row
/// split* que a nota do allowlist prescreve: é o maior bloco da seção e o único que fala um
/// vocabulário próprio (um sub-retângulo do asset), enquanto o resto descreve a PROVENIÊNCIA.
///
/// ⚠️ Escondido para `HandPacked` — aquele traz o próprio rect do asset, então o controle aqui
/// seria um knob que o extract ignora.
#[allow(clippy::too_many_arguments)]
fn paint_region_rows(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    info: &InspectorSpriteInfo,
    x: f32,
    w: f32,
    y: f32,
    label_font: f32,
    row_gap: f32,
) -> f32 {
    let mut cur_y = y;
    // Region sampling (spec §3.3) — hidden for Hand-packed (it brings its
    // own rect from the asset). Toggle + (when on) X/Y/W/H px inputs +
    // Filter Clip. Renders via the extract `region_subrect` (W2.T2.4).
    if !matches!(info.source_kind, InspectorSpriteSource::HandPacked { .. }) {
        let cb_h = 18.0_f32; // LITERAL-PX-OK: Checkbox visual height
        let re_value = store
            .checkbox(ids::INSP_REGION_ENABLED)
            .map_or(CheckboxValue::Unchecked, |(_, v)| v);
        let re_rect = Rect::new(x, cur_y, w, cb_h);
        hit_index.register(ids::INSP_REGION_ENABLED, re_rect);
        paint_checkbox(
            &Checkbox::new(ids::INSP_REGION_ENABLED, "Region")
                .visual(store.checkbox_visual(ids::INSP_REGION_ENABLED))
                .value(re_value),
            re_rect,
            scene,
            text_system,
            theme,
        );
        cur_y += cb_h + row_gap;

        if matches!(re_value, CheckboxValue::Checked) {
            let field_h = ROW_H_PX;
            let cell_gap = Spacing::Md.px();
            let (region_w, region_dot) =
                ph2d_editor_core::widget::form_row_columns(x, w, cur_y, field_h);
            let cell_w = ((region_w - cell_gap) * 0.5).max(0.0);
            paint_region_num_cell(
                scene,
                text_system,
                hit_index,
                store,
                Rect::new(x, cur_y, cell_w, field_h),
                "X",
                ids::INSP_REGION_X,
                label_font,
                theme,
            );
            paint_region_num_cell(
                scene,
                text_system,
                hit_index,
                store,
                Rect::new(x + cell_w + cell_gap, cur_y, cell_w, field_h),
                "Y",
                ids::INSP_REGION_Y,
                label_font,
                theme,
            );
            ph2d_editor_core::widget::paint_decorator_dot(scene, theme, region_dot);
            cur_y += field_h + row_gap;
            paint_region_num_cell(
                scene,
                text_system,
                hit_index,
                store,
                Rect::new(x, cur_y, cell_w, field_h),
                "W",
                ids::INSP_REGION_W,
                label_font,
                theme,
            );
            paint_region_num_cell(
                scene,
                text_system,
                hit_index,
                store,
                Rect::new(x + cell_w + cell_gap, cur_y, cell_w, field_h),
                "H",
                ids::INSP_REGION_H,
                label_font,
                theme,
            );
            cur_y += field_h + row_gap;

            let fc_value = store
                .checkbox(ids::INSP_REGION_FILTER_CLIP)
                .map_or(CheckboxValue::Checked, |(_, v)| v);
            let fc_rect = Rect::new(x, cur_y, w, cb_h);
            hit_index.register(ids::INSP_REGION_FILTER_CLIP, fc_rect);
            paint_checkbox(
                &Checkbox::new(ids::INSP_REGION_FILTER_CLIP, "Filter Clip")
                    .visual(store.checkbox_visual(ids::INSP_REGION_FILTER_CLIP))
                    .value(fc_value),
                fc_rect,
                scene,
                text_system,
                theme,
            );
            cur_y += cb_h + row_gap;
        }
    }
    cur_y
}
