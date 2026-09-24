//! Render Source (+ Region) — Inspector section painter (split from sections.rs,
//! architecture_panel_loc_cap). Logic verbatim; behavior unchanged.

use super::render_source_precision::paint_precision_row;
use super::render_source_regiao::paint_region_rows;
use super::*;
use ph2d_editor_core::widget::SectionFold;
use ph2d_i18n::tr;
use ph2d_i18n::tr_with;

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
    //
    // ⭐⭐⭐ **E a ALTURA DE LINHA desta secção MORREU em 2026-09-22.** Ela era
    //    `Sm + vão = 15` — a altura de um rótulo empilhado por cima de um valor —, e a da casa é
    //    a `ROW_H_PX = 22`. *Uma secção com altura própria não alinha com nenhuma vizinha, e esta
    //    é a que o dono fotografou.*
    let label_font = TypeToken::Sm.px();
    // ⚠️ **O vão entre dois controlos é a porta `control_gap_px` (3 px)**, e não o
    //    `Spacing::Xs` (4) escrito à mão — ordem do dono, 2026-09-07. Esta secção é
    //    anterior à porta. Ver `every_stack_of_rows_asks_the_rhythm`.
    let row_gap = ph2d_tokens::control_gap_px();
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: section header band height
    let color_id = core_ids::INSP_LIVE_RENDER_COLOR;
    let rgba = store
        .widget_color(color_id)
        .unwrap_or([0x88, 0x88, 0x88, 0xff]); // LITERAL-COLOR-OK: neutral default for unconfigured section accent
    let header = section_header(
        store,
        core_ids::INSP_LIVE_RENDER_SECTION,
        tr("panel.inspector.render_source.render_source"),
    )
    .color(rgba);
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
        core_ids::INSP_LIVE_RENDER_SECTION,
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
    let btn_rect = ph2d_editor_core::property_row::caixa_do_botao(
        text_system,
        x,
        w,
        cur_y,
        reimport_h,
        tr("panel.inspector.render_source.reimport_at_current_px_m"),
    );
    let id = ids::INSP_RENDER_SOURCE_REIMPORT;
    let state = if !info.can_reimport {
        (ButtonState::Disabled, ph2d_editor_core::motion::SETTLED)
    } else {
        store.button_visual(id)
    };
    hit_index.register(id, btn_rect);
    let btn = Button::new(
        id,
        tr("panel.inspector.render_source.reimport_at_current_px_m"),
    )
    .kind(ButtonKind::Default)
    .visual(state);
    paint_button(&btn, btn_rect, scene, text_system, theme);
    fold.finish(
        store,
        scene,
        hit_index,
        ph2d_editor_core::property_row::abaixo_do_botao(btn_rect) + SECTION_BOTTOM_PAD_PX,
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
        // ⚠️ Aqui o nome fica POR CIMA de propósito: o que vem por baixo não é um controlo, é uma
        //    FRASE que explica porque não há controlo, e ela precisa da largura inteira.
        paint_text(
            text_system,
            scene,
            tr("panel.inspector.render_source.strategy"),
            x,
            cur_y,
            label_font,
            w,
            resolve(ColorToken::Text2, theme),
        );
        cur_y += label_font + SECTION_LABEL_TO_CONTROL_PX;
        paint_text(
            text_system,
            scene,
            tr("panel.inspector.render_source.from_the_asset_pipeline_read"),
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
    // ⭐⭐ **Pela porta da ESCOLHA, com o nome ao LADO** (2026-09-23) — o nome ia POR CIMA e o
    //    grupo arrancava na borda do conteúdo. A porta escolhe a forma pela medida (ao lado quando
    //    cabe numa fileira, paleta quando não) e a coluna é a da secção.
    let seccao = seccao_do_render(text_system);
    ph2d_editor_core::property_row::paint_choice_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.render_source.strategy"),
        &[
            (
                tr("panel.inspector.render_source.atlas"),
                matches!(info.source_kind, InspectorSpriteSource::Atlas { .. }),
                core_ids::INSP_RENDER_STRATEGY_ATLAS,
            ),
            (
                tr("panel.inspector.render_source.individual"),
                matches!(info.source_kind, InspectorSpriteSource::Individual { .. }),
                crate::ids::INSP_RENDER_STRATEGY_INDIVIDUAL,
            ),
            (
                tr("panel.inspector.render_source.hand_packed"),
                matches!(info.source_kind, InspectorSpriteSource::HandPacked { .. }),
                crate::ids::INSP_RENDER_STRATEGY_HANDPACKED,
            ),
        ],
        seccao,
    )
}

/// ⭐⭐ **A coluna do nome da secção Render Source — UMA para as linhas de nome ao lado.**
///
/// ⚠️ Antes de 2026-09-23 a `Storage`/`Source size` mediam-se entre si e a `Strategy`/`Format`
/// pintavam o nome POR CIMA; com as duas escolhas ao lado, as quatro caem no mesmo `x`.
pub(super) fn seccao_do_render(
    text_system: &mut TextSystem,
) -> ph2d_editor_core::property_row::Seccao {
    ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        1,
        &[
            tr("panel.inspector.render_source.strategy"),
            STORAGE_LABEL.tr(),
            SOURCE_SIZE_LABEL.tr(),
            tr("panel.inspector.render_source.format"),
        ],
    )
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
    row_gap: f32,
) -> f32 {
    // Cleaner phrasing — strategy name + key/id separated by middle dot (the only ASCII-safe
    // non-ASCII glyph allowed in UI strings; vide no_tofu_glyphs gate).
    let detail = match info.source_kind {
        InspectorSpriteSource::Atlas { key } => {
            tr_with("panel.inspector.render_source.atlas_key", &[("key", &key)])
        }
        InspectorSpriteSource::Individual { texture_id } => tr_with(
            "panel.inspector.render_source.individual_texture",
            &[("texture_id", &texture_id)],
        ),
        // ⚠️ O NOME, não os índices: é por ele que o artista reencontra o desenho no Aseprite. Os
        // números só aparecem se o rótulo faltar (uma folha que o projeto trouxe mas a sessão não
        // tem), e aí eles são a informação honesta que sobra.
        InspectorSpriteSource::HandPacked { sheet, region } => match &info.sheet_label {
            Some(label) => tr_with(
                "panel.inspector.render_source.hand_packed_label",
                &[("label", &label)],
            ),
            None => tr_with(
                "panel.inspector.render_source.hand_packed_sheet_region",
                &[("sheet", &sheet), ("region", &region)],
            ),
        },
        // W2.T2: tier-cooked KTX2 — read-only marker, no key/id shown.
        InspectorSpriteSource::CookedTexture => {
            tr("panel.inspector.render_source.cooked_texture").to_string()
        }
    };
    // ⭐⭐⭐ **A PROVENIÊNCIA É UMA LINHA DE PROPRIEDADE — nome à ESQUERDA, valor à DIREITA.**
    //
    // ⛔⛔⛔ **Report do dono, 2026-09-21, com FOTO desta secção:** *«várias seções muito confusas
    //    e desorganizadas»* e *«quanto ao alinhamento precisamos melhorar em todos os lugares»*.
    //    Esta secção falava outra LÍNGUA: ela punha o nome POR CIMA do valor enquanto todas as
    //    vizinhas o põem à esquerda ⇒ nada aqui alinhava com nada, e a «linha» media
    //    `Sm + vão = 15` contra a `ROW_H_PX = 22` da casa.
    //
    // ⛔⛔ **E o gate que proíbe exactamente isto
    //    (`no_row_paints_its_name_above_its_control`) NUNCA a viu**: ele procura o idioma pelo
    //    nome da variável (`label_h`) e aqui ela chamava-se `label_font + row_gap`. *O limite
    //    estava escrito no próprio doc-comment dele* — *«uma secção que empilhe por outro caminho
    //    e com outro nome continua invisível a uma régua textual»* — e a secção que o dono
    //    fotografou era essa.
    //
    // ⚠️ **A coluna sai dos DOIS nomes do bloco**, senão a `Storage` e a `Source size` caem em `x`
    //    diferentes — a outra metade do mesmo report.
    let sec = seccao_do_render(text_system);
    let linha = ph2d_editor_core::property_row::paint_label_row(
        scene,
        text_system,
        theme,
        x,
        w,
        y,
        ROW_H_PX,
        STORAGE_LABEL.tr(),
        sec,
    );
    let slot = linha.control;
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
    // ⭐⭐⭐ **O BALÃO** — ordem do dono, 2026-09-19: *«encurtar · balão ao passar o rato»*.
    //
    // ⛔ Com a linha a falar a língua da casa o valor deixou de ter a largura do painel e passou a
    //    ter a da coluna do controlo (`112 px` no encaixe do dono) ⇒ um nome de folha comprido
    //    (`Hand-packed · hero · idle_0`) passa a ser cortado. *O nome da folha é texto do ARTISTA:
    //    encurtá-lo não é uma saída*, e a lei da casa para isso é o balão.
    //
    // ⚠️ **O âmbito é a porta, e não uma segunda lista:** quem decide o corte é a lei da reticência
    //    ([`ph2d_editor_core::text_elide`]), e ela só sabe ONDE o texto caiu se o pintor lho disser.
    ph2d_editor_core::text_elide::balao::na_area(slot, || {
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
    });
    // ⭐⭐⭐ **É AQUI que ela vira alvo.** O `HitIndex` é a porta única de *«o que está debaixo do
    // cursor»*, e é ela que dá de graça o recorte do corpo e a oclusão por um painel de cima.
    // ⛔ Sem `populate`: quem consome este id é o caminho da QUEDA, não o de clique — a mesma
    // classe das *swatches* do picker, e o `HIT_PARITY_ALLOW` nomeia-a.
    hit_index.register(ids::INSP_RENDER_TEXTURE_SLOT, slot);
    let mut cur_y = y + ROW_H_PX + row_gap;
    // ⚠️ **O TAMANHO de origem fica ao lado da ranhura**, e não numa função irmã: as duas são a
    // mesma pergunta — *de onde vêm estes pixels, e que tamanho tinham* —, e separá-las custou ao
    // pai o tecto de 200 LOC por uma chamada.
    if let Some((pw, ph)) = info.source_pixels {
        let tamanho = ph2d_editor_core::property_row::paint_label_row(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            ROW_H_PX,
            SOURCE_SIZE_LABEL.tr(),
            sec,
        );
        paint_text(
            text_system,
            scene,
            &tr_with(
                "panel.inspector.render_source.size_px",
                &[("pw", &pw), ("ph", &ph)],
            ),
            tamanho.control.x,
            tamanho.control.y + (ROW_H_PX - label_font) * 0.5,
            label_font,
            tamanho.control.w,
            resolve(ColorToken::Text1, theme),
        );
        cur_y += ROW_H_PX + row_gap;
    }
    cur_y
}
