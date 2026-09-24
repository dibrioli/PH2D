//! Color & Tint — Inspector section painter (split from sections.rs,
//! architecture_panel_loc_cap). Logic verbatim; behavior unchanged.

use super::*;
use ph2d_editor_core::widget::SectionFold;
use ph2d_i18n::tr;

/// Paint one labeled tint swatch (label left, swatch right) inside
/// `cell`. The swatch fill reads `widget_color(swatch_id)` (kept in
/// sync with the live `Sprite` channel by `sync.rs`), falling back to
/// `fallback_rgba` on the cold paint. The hit rect is the swatch only,
/// so the picker opens from the colored chip — matching grid-snap's
/// color row and the Widget Gallery "Tint" sample.
#[allow(clippy::too_many_arguments)]
/// ⚠️ `pub(super)` desde o TOP-20 #18: a secção PARTICLES pinta as duas cores dela por AQUI. Uma
/// segunda linha de cor escrita noutro sítio seria uma segunda resposta a *«como se pinta uma
/// amostra»* — e a coluna de animação (que este ajudante reserva) é exactamente o que a segunda
/// esqueceria.
/// ⭐ **A COLUNA de um bloco de linhas de cor** — medida sobre TODOS os nomes do bloco.
///
/// ⛔ Sem ela cada linha mede a coluna com o próprio nome e duas vizinhas caem em `x` diferentes,
/// que é metade do report do dono de 2026-09-21 (*«o alinhamento precisa melhorar em todos os
/// lugares»*). ⚠️ Ela nasceu porque a MESMA conta ia aparecer em **três** secções (tint ·
/// partículas · tween) no mesmo commit.
#[must_use]
pub(super) fn seccao_de_cores(
    text_system: &mut TextSystem,
    rotulos: &[&str],
) -> ph2d_editor_core::property_row::Seccao {
    ph2d_editor_core::property_row::Seccao::medida(text_system, 1, rotulos)
}

/// ⭐⭐⭐ **UM BLOCO DE LINHAS DE COR** — N amostras que partilham a coluna do bloco.
///
/// ⛔⛔ Ela nasceu de um TECTO: extrair o par do tween para um irmão levava o `tween_editor.rs`
/// acima do cap de ficheiro, e a mesma forma já existia nas PARTÍCULAS. ⇒ *duas cópias do mesmo
/// bloco, a nascer no mesmo commit* — a lei desta casa é que isso é uma PORTA.
///
/// ⚠️ **A coluna sai de TODOS os nomes do bloco**, que é o que alinha as linhas entre si — a outra
/// metade do report do dono de 2026-09-21 sobre alinhamento.
#[allow(clippy::too_many_arguments)]
pub(super) fn bloco_de_cores(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    mut y: f32,
    cores: &[(NodeId, &str, [f32; 4])],
) -> f32 {
    let rotulos: Vec<&str> = cores.iter().map(|(_, l, _)| *l).collect();
    let sec = seccao_de_cores(text_system, &rotulos);
    for (id, label, rgba) in cores {
        y = paint_tint_swatch_cell(
            Rect::new(x, y, w, ph2d_tokens::ROW_H_PX),
            label,
            *id,
            crate::state_tint::tint_f32_to_u8(*rgba),
            false,
            store,
            hit_index,
            scene,
            text_system,
            theme,
            sec,
        );
    }
    y
}

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_tint_swatch_cell(
    cell: Rect,
    label: &str,
    swatch_id: NodeId,
    fallback_rgba: [u8; 4],
    mixed: bool,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    seccao: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    ph2d_editor_core::property_row::paint_color_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        cell.x,
        cell.w,
        cell.y,
        label,
        swatch_id,
        fallback_rgba,
        mixed,
        seccao,
    )
}

/// W2 Sprite Inspector v2 — Color & Tint section (anatomia §03 §3.6).
/// Sub-tabs `[Tint] [Self] [Corners] [Effects]` (§3.0 D11 density fix),
/// one body at a time: Tint / Self Tint modulate swatches (OKLCH picker),
/// the per-corner 2×2 gradient grid (+ live preview + Equalize), and the
/// Effects body (Opacity slider-with-chip + Tint Fill silhouette). Every
/// channel is render-ready (`RenderInstance.tint` / per-corner / opacity).
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_color_tint_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let field_h = ROW_H_PX;
    // ⚠️ **O vão entre dois controlos é uma PORTA, não um `Spacing` escolhido aqui**
    //    (`ph2d_tokens::control_gap_px`, 3 px). Enio, 2026-09-07: *«entre grupos de botões
    //    temos um espaçamento, entre sliders outro. Para ambos vamos colocar o padrão de 3 px»*.
    //    Este ficheiro é anterior à porta e escrevia o `Spacing::Xs` (4) à mão.
    let row_gap = ph2d_tokens::control_gap_px();
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: section header band height
    let color_id = core_ids::INSP_LIVE_COLOR_COLOR;
    let rgba = store
        .widget_color(color_id)
        .unwrap_or([0x88, 0x88, 0x88, 0xff]); // LITERAL-COLOR-OK: neutral default for unconfigured section accent
    let header = section_header(
        store,
        core_ids::INSP_LIVE_COLOR_SECTION,
        tr("panel.inspector.color_tint.color_and_tint"),
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
        core_ids::INSP_LIVE_COLOR_SECTION,
        x,
        w,
        y + header_h,
        scene,
        hit_index,
    ) else {
        return y + header_h;
    };
    let mut cur_y = y + header_h;

    let sp = crate::state::current_inspector_sprite();

    // All Color & Tint controls stacked + visible at once (user
    // 2026-05-31: faster to reach than behind the old [Tint][Self]
    // [Corners][Effects] sub-tabs). Order: Tint · Self Tint · Per-corner
    // grid + Equalize · Opacity · Tint Fill. Each color swatch opens the
    // shared BlenderColorPicker (OKLCH); the chosen color round-trips via
    // `widget_color(id)` and is dispatched as a `SpriteFieldEdit` from
    // `sync.rs`. WHITE fallback covers the cold paint before first sync.

    // Tint — inherited modulate (cascades to children).
    let tint_seed = sp
        .as_ref()
        .map(|s| crate::state_tint::tint_f32_to_u8(s.tint))
        .unwrap_or([0xff, 0xff, 0xff, 0xff]); // LITERAL-COLOR-OK: WHITE = tint default
    // ⚠️ **A secção das duas linhas de cor é UMA** — a coluna sai dos DOIS nomes, senão a `Tint`
    //    e a `Self Tint` caem em `x` diferentes, que é metade do report do alinhamento.
    let sec_tint = ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        1,
        &[
            tr("panel.inspector.color_tint.tint"),
            tr("panel.inspector.color_tint.self_tint"),
        ],
    );
    cur_y = paint_tint_swatch_cell(
        Rect::new(x, cur_y, w, field_h),
        tr("panel.inspector.color_tint.tint"),
        ids::INSP_SPRITE_TINT_SWATCH,
        tint_seed,
        sp.as_ref().is_some_and(|s| s.mixed.tint),
        store,
        hit_index,
        scene,
        text_system,
        theme,
        sec_tint,
    );

    // Self Tint — local modulate (does NOT cascade).
    let self_seed = sp
        .as_ref()
        .map(|s| crate::state_tint::tint_f32_to_u8(s.self_tint))
        .unwrap_or([0xff, 0xff, 0xff, 0xff]); // LITERAL-COLOR-OK: WHITE = self_tint default
    cur_y = paint_tint_swatch_cell(
        Rect::new(x, cur_y, w, field_h),
        tr("panel.inspector.color_tint.self_tint"),
        ids::INSP_SPRITE_SELF_TINT_SWATCH,
        self_seed,
        sp.as_ref().is_some_and(|s| s.mixed.self_tint),
        store,
        hit_index,
        scene,
        text_system,
        theme,
        sec_tint,
    );

    // Per-corner — 2×2 swatch grid + live bilinear gradient preview +
    // Equalize. Renders via the shader's @location(9..12) attributes.
    cur_y = paint_per_corner_tab(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        sp.as_ref(),
    );
    cur_y += row_gap;

    // Opacity slider-with-chip.
    let (_, op_value) = store
        .slider(ids::INSP_SPRITE_OPACITY)
        .unwrap_or((SliderState::Normal, 1.0));
    let opacity_h = paint_slider_with_chip(
        Rect::new(x, cur_y, w, field_h),
        tr("panel.inspector.color_tint.opacity"),
        op_value,
        ids::INSP_SPRITE_OPACITY,
        ids::INSP_SPRITE_OPACITY_CHIP,
        store,
        hit_index,
        scene,
        text_system,
        theme,
    );
    cur_y += opacity_h + row_gap;

    // Tint Fill silhouette toggle — **pela porta** (2026-09-15).
    //
    // ⚠️ **A secção declara UM nome**, e não os das barras deslizantes acima: ali o nome vive
    // DENTRO da barra (spec §2), logo não partilha coluna nenhuma com esta linha.
    let (_, tf_value) = store
        .checkbox(ids::INSP_SPRITE_TINT_FILL)
        .unwrap_or((CheckboxState::Normal, CheckboxValue::Unchecked));
    let sec = ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        1,
        &[tr("panel.inspector.color_tint.tint_fill")],
    );
    cur_y = ph2d_editor_core::property_row::paint_check_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        (
            ids::INSP_SPRITE_TINT_FILL,
            tr("panel.inspector.color_tint.tint_fill"),
            matches!(tf_value, CheckboxValue::Checked),
        ),
        sec,
    );

    fold.finish(store, scene, hit_index, cur_y + SECTION_BOTTOM_PAD_PX)
}

/// Per-corner tint sub-tab: a 2×2 swatch grid (`TL TR` / `BL BR`), a live
/// bilinear gradient preview to its right, and an "Equalize Corners"
/// button below. Each swatch opens the picker (dispatched as the whole
/// `PerCornerTint` array with one corner replaced — see `sync.rs`); the
/// preview re-bilerps the live (picker-overridden) corner colors so the
/// gradient updates while the user picks.
#[allow(clippy::too_many_arguments)]
fn paint_per_corner_tab(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    sp: Option<&InspectorSpriteInfo>,
) -> f32 {
    // ⭐⭐⭐ **O NOME À ESQUERDA, OS QUATRO CANTOS NA COLUNA DO VALOR** — ordem do dono,
    //    2026-09-21, depois de ver a versão anterior: *«vamos tirar o preview (rect maior) e no
    //    lugar colocar a label. Alinhar ao centro. Os 4 seletores de cor à direita com a altura
    //    padrão dos outros seletores de cor»*.
    //
    // ⛔⛔ **Ele deixou de ser um BLOCO COM LEGENDA e passou a ser uma LINHA DE PROPRIEDADE**, e o
    //    que o destravou foi tirar a prévia: a nota que aqui esteve media o grupo em `~144 px`
    //    (`2 × 32` de amostras `+ 8` de vão `+ 68` de PRÉVIA) contra uma coluna de `~120`, e sem
    //    os `68` ele cabe. *A prévia era o que o impedia de alinhar com as vizinhas.*
    //
    // ⚠️ **As quatro medem a ALTURA DE LINHA**, como a `Tint` e a `Self Tint` — a padronização que
    //    o mesmo report pediu —, e a largura sai da coluna a dividir por dois.
    let gap = ph2d_tokens::control_gap_px();
    let sec = seccao_de_cores(
        text_system,
        &[tr(
            "panel.inspector.color_tint.per_corner_tint_vertex_gradient",
        )],
    );
    let alto = ph2d_tokens::ROW_H_PX * 2.0 + gap;
    let row = ph2d_editor_core::widget::property_row_columns_for(x, w, y, alto, sec.nome_w(), None);
    // ⭐ O nome ao CENTRO do bloco — ele descreve as DUAS filas, não a de cima.
    let label_font = TypeToken::Sm.px();
    ph2d_editor_core::widget::paint_property_label(
        text_system,
        scene,
        tr("panel.inspector.color_tint.per_corner_tint_vertex_gradient"),
        row.label.x,
        row.label.y + (alto - label_font) * 0.5,
        label_font,
        row.label.w,
        resolve(ColorToken::Text2, theme),
    );
    let cell_w = ((row.control.w - gap) * 0.5).max(1.0);
    let cell_h = ph2d_tokens::ROW_H_PX;
    let committed = sp
        .map(|s| s.per_corner_tint)
        .unwrap_or([[1.0, 1.0, 1.0, 1.0]; 4]); // WHITE = per-corner default (no gradient)
    let corner_ids = [
        ids::INSP_SPRITE_CORNER_TL,
        ids::INSP_SPRITE_CORNER_TR,
        ids::INSP_SPRITE_CORNER_BL,
        ids::INSP_SPRITE_CORNER_BR,
    ];
    // ⚠️ **O nome acessível de cada canto** — sem ele os quatro são quadrados anónimos, e foi o
    //    censo de chaves que o apanhou quando a porta nova os engoliu.
    let a11y = [
        tr("panel.inspector.color_tint.top_left_corner_tint"),
        tr("panel.inspector.color_tint.top_right_corner_tint"),
        tr("panel.inspector.color_tint.bottom_left_corner_tint"),
        tr("panel.inspector.color_tint.bottom_right_corner_tint"),
    ];
    // TL, TR, BL, BR — a grelha 2×2 mapeia os cantos do quad, e é por isso que ela fica grelha.
    let positions = [
        (row.control.x, row.control.y),
        (row.control.x + cell_w + gap, row.control.y),
        (row.control.x, row.control.y + cell_h + gap),
        (row.control.x + cell_w + gap, row.control.y + cell_h + gap),
    ];
    // Any per-corner divergence across a multi-selection (BulkSelect) →
    // all four show the Mixed treatment (a single flag covers the array).
    let per_corner_mixed = sp.is_some_and(|s| s.mixed.per_corner);
    for i in 0..4 {
        let fallback = crate::state_tint::tint_f32_to_u8(committed[i]);
        let sr = Rect::new(positions[i].0, positions[i].1, cell_w, cell_h);
        let rgba =
            (!per_corner_mixed).then(|| store.widget_color(corner_ids[i]).unwrap_or(fallback));
        ph2d_editor_core::widget::paint_swatch_or_mixed(
            sr,
            corner_ids[i],
            a11y[i],
            rgba,
            store.picker_target() == Some(corner_ids[i]),
            scene,
            theme,
        );
        hit_index.register(corner_ids[i], sr);
    }
    ph2d_editor_core::widget::paint_decorator_dot(scene, theme, row.dot);
    let grid_h = alto;
    let mut cur_y = y + grid_h + Spacing::Sm.px();

    // Equalize Corners — copies TL → the other three (spec §3.6).
    let btn_h = ROW_H_PX;
    let eq_rect = ph2d_editor_core::property_row::caixa_do_botao(
        text_system,
        x,
        w,
        cur_y,
        btn_h,
        tr("panel.inspector.color_tint.equalize_corners"),
    );
    let eq_state = store.button_visual(ids::INSP_SPRITE_CORNER_EQUALIZE);
    hit_index.register(ids::INSP_SPRITE_CORNER_EQUALIZE, eq_rect);
    let eq = Button::new(
        ids::INSP_SPRITE_CORNER_EQUALIZE,
        tr("panel.inspector.color_tint.equalize_corners"),
    )
    .kind(ButtonKind::Default)
    .visual(eq_state);
    paint_button(&eq, eq_rect, scene, text_system, theme);
    cur_y += btn_h + ph2d_tokens::control_gap_px();
    cur_y
}
