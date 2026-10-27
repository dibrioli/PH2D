//! The **Wet Paint** section (ADR-0134 — the fluid engine as a paint mode): sibling of
//! `paint_watercolor.rs`. The **Enable** master checkbox — the persistent ARM that makes the
//! Brush paint WET and survives tool round-trips (eraser / selection and back; Enio 2026-07-21),
//! exactly like the Watercolor and Impasto enables — plus the W3 **curated knobs** (SPEC §16's
//! tuning table curated to the seven an artist reaches for), painted only while armed. Each knob
//! is a [`crate::card::card_row`] number chip forwarding as `SetValue` via
//! `PAINTER_WETPAINT_FIELDS` + `number_field::is_param_field`.

use crate::PaintCtx;
use crate::card::card_row;
use crate::number_field;
use ph2d_i18n::{tr, tr_with};
use ph2d_tool_painter::BrushSettings;

// Knob ranges/steps: each mirrors the `set_wet_knob_value` clamp in the tool
// (which mirrors SPEC §16's `KNOB_DEFS` slider table) — parameter domains of
// the fluid engine, not design values.
const PIGMENT_MAX: f32 = 2000.0; // LITERAL-PX-OK: wet-paint Pigment-per-dab range bound (SPEC §16)
const PIGMENT_STEP: f64 = 10.0; // LITERAL-PX-OK: wet-paint Pigment slider step (SPEC §16)
const PICKUP_MAX: f32 = 0.2; // LITERAL-PX-OK: wet-paint Pickup range bound (SPEC §16)
const PICKUP_STEP: f64 = 0.001; // LITERAL-PX-OK: wet-paint Pickup slider step (SPEC §16)
const DRY_SPEED_MAX: f32 = 8.0; // LITERAL-PX-OK: wet-paint Evaporation range bound (SPEC §16)
const DRY_SPEED_STEP: f64 = 0.05; // LITERAL-PX-OK: wet-paint Evaporation slider step (SPEC §16)
const EDGE_MAX: f32 = 200.0; // LITERAL-PX-OK: wet-paint Edge-darkening range bound (SPEC §16)
const GRAVITY_MAX: f32 = 0.05; // LITERAL-PX-OK: wet-paint Gravity range bound (SPEC §16)
const GRAVITY_STEP: f64 = 0.0005; // LITERAL-PX-OK: wet-paint Gravity slider step (SPEC §16)
// A faixa da grade do fluido (px por célula) — espelha `wetpaint::grid_map`
// MIN_RATIO/MAX_RATIO, do mesmo jeito que os knobs acima espelham os clamps do
// tool. Não é valor de design: é o domínio do parâmetro.
const GRID_MIN: f32 = 1.0; // LITERAL-PX-OK: fluid-grid ratio lower bound (grid_map::MIN_RATIO)
const GRID_MAX: f32 = 30.0; // LITERAL-PX-OK: fluid-grid ratio upper bound (grid_map::MAX_RATIO)
// A faixa da grade de FLUXO (células de fluido por célula de velocidade) —
// espelha `ph2d_wet_paint::flow` MIN/MAX_FLOW_RATIO, pelo mesmo motivo.
const FLOW_MIN: f32 = 1.0; // LITERAL-PX-OK: flow-grid ratio lower bound (flow::MIN_FLOW_RATIO)
const FLOW_MAX: f32 = 16.0; // LITERAL-PX-OK: flow-grid ratio upper bound (flow::MAX_FLOW_RATIO)

pub(crate) fn paint_wetpaint_section(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let (mut y, fold) = crate::paint_brush_top::paint_collapsible_section(
        ctx,
        theme,
        x,
        content_w,
        y,
        tr("panel.painter_layers.wetpaint.wet_paint"),
        ph2d_tool_painter::ids::PAINTER_WETPAINT_SECTION,
        ph2d_tool_painter::ids::PAINTER_WETPAINT_RESET,
    );
    let Some(fold) = fold else {
        return y;
    };
    // The W3 knobs — only while ARMED (a knob for an engine that is not running is a dead
    // control wearing a live one's clothes). Ranges mirror the tool's clamps (SPEC §16 /
    // `KNOB_DEFS` slider ranges); decimals sized to each knob's granularity.
    //
    // ⚠️ The arm is the **Paint Mode** dropdown at the head of the appearance half (2026-07-22) — the
    // section's own `Enable` checkbox is gone, because the section is only painted while this medium is
    // the one selected. The `brush.wetpaint` guard STAYS: `paint_media` derives the medium from this
    // very flag, so the two agree by construction, and one condition guarding the knobs is cheaper to
    // trust than a second predicate that could drift from it.
    if brush.wetpaint {
        y = paint_resolution_group(ctx, theme, x, content_w, y, brush);
        // As ferramentas da água moram no trilho da esquerda (dono 25/10); aqui, a linha da que está na mão.
        if !brush.eraser {
            for (id, rotulo, v, min, max, step) in
                brush.wet_knobs.knobs_da_ferramenta(brush.wet_tool)
            {
                let decimais = ph2d_tool_painter::Escala {
                    min: min as f32,
                    max: max as f32,
                    step,
                }
                .decimals();
                y = card_row(
                    ctx, theme, x, content_w, y, rotulo, id, v as f32, min as f32, max as f32,
                    step, decimais,
                );
            }
        }
        // The TILT dial (the model's polar pad, copied faithfully).
        y = crate::paint_wetpaint_tilt::paint_tilt_card(ctx, theme, x, content_w, y, &brush);
        let k = brush.wet_knobs;
        // O Water é um VOLUME do topo (W1, `paint_wetpaint_presets.rs`): um só controlo por facto.
        let rows: [(&str, ph2d_a11y::NodeId, f32, f32, f32, f64, usize); 6] = [
            (
                "panel.painter_layers.wetpaint.pigment",
                ph2d_tool_painter::ids::PAINTER_WETPAINT_PIGMENT,
                k.pigment() as f32,
                0.0,
                PIGMENT_MAX,
                PIGMENT_STEP,
                0,
            ),
            (
                "panel.painter_layers.wetpaint.pickup",
                ph2d_tool_painter::ids::PAINTER_WETPAINT_PICKUP,
                k.pickup() as f32,
                0.0,
                PICKUP_MAX,
                PICKUP_STEP,
                3,
            ),
            (
                "panel.painter_layers.wetpaint.dry_speed",
                ph2d_tool_painter::ids::PAINTER_WETPAINT_DRY_SPEED,
                k.dry_speed() as f32,
                0.0,
                DRY_SPEED_MAX,
                DRY_SPEED_STEP,
                2,
            ),
            (
                "panel.painter_layers.wetpaint.edge_darkening",
                ph2d_tool_painter::ids::PAINTER_WETPAINT_EDGE,
                k.edge_darkening() as f32,
                0.0,
                EDGE_MAX,
                1.0,
                0,
            ),
            (
                "panel.painter_layers.wetpaint.gravity",
                ph2d_tool_painter::ids::PAINTER_WETPAINT_GRAVITY,
                k.gravity() as f32,
                0.0,
                GRAVITY_MAX,
                GRAVITY_STEP,
                4,
            ),
            (
                "panel.painter_layers.wetpaint.erase_strength",
                ph2d_tool_painter::ids::PAINTER_WETPAINT_ERASE,
                k.erase as f32,
                0.0,
                1.0,
                number_field::FINE_STEP,
                2,
            ),
        ];
        // A borracha da água não pousa: o Pigment, o Pickup, o pó do pincel e os knobs que só agem a cada dab de tinta
        // (Wet bloom, Tip pickup) saem (`DepositoMolhado`).
        let pousa = brush.le(ph2d_tool_painter::LinhaDaFerramenta::DepositoMolhado);
        let do_deposito = [
            ph2d_tool_painter::ids::PAINTER_WETPAINT_PIGMENT,
            ph2d_tool_painter::ids::PAINTER_WETPAINT_PICKUP,
            ph2d_tool_painter::ids::PAINTER_WETPAINT_DUST,
            ph2d_tool_painter::ids::PAINTER_WETPAINT_WET_BLOOM,
            ph2d_tool_painter::ids::PAINTER_WETPAINT_TIP_PICKUP,
        ];
        for (label, id, value, min, max, step, decimals) in rows {
            if !pousa && do_deposito.contains(&id) {
                continue;
            }
            y = card_row(
                ctx, theme, x, content_w, y, label, id, value, min, max, step, decimals,
            );
        }
        // A física da água que regula a «grossura» dela (saiu do Tuning, dono 13/10) e o pó do pincel (19/10): a
        // faixa e o passo vêm da tabela do motor, a mesma do Tuning.
        let nomes = [
            "panel.painter_layers.wetpaint.leveling",
            "panel.painter_layers.wetpaint.level_clamp",
            "panel.painter_layers.wetpaint.capillary",
            "panel.painter_layers.wetpaint.brake",
            "panel.painter_layers.wetpaint.dust",
            "panel.painter_wet_cartao.backrun",
            "panel.painter_wet_cartao.granulation",
            "panel.painter_wet_cartao.diffusion",
            "panel.painter_wet_cartao.wet_bloom",
            "panel.painter_wet_cartao.tip_pickup",
        ];
        for (label, (id, v, min, max, step)) in nomes.into_iter().zip(k.knobs_no_cartao()) {
            if !pousa && do_deposito.contains(&id) {
                continue;
            }
            let decimais = ph2d_tool_painter::Escala {
                min: min as f32,
                max: max as f32,
                step,
            }
            .decimals();
            y = card_row(
                ctx, theme, x, content_w, y, label, id, v as f32, min as f32, max as f32, step,
                decimais,
            );
        }
        // Canvas actions (the model's bottom bar): three one-shots as a
        // momentary button row (the watercolor Dry/Wet pattern) + Show Wet
        // as an honest TOGGLE (its state is visible, not implied).
        y = crate::paint_seg_row::seg_row_owned(
            ctx,
            theme,
            x,
            content_w,
            y,
            ph2d_tool_painter::ids::PAINTER_WETPAINT_WETCANVAS,
            tr("panel.painter_layers.wetpaint.canvas_actions"),
            &[
                (
                    ph2d_tool_painter::ids::PAINTER_WETPAINT_WETCANVAS,
                    tr("panel.painter_layers.wetpaint.wet_canvas").into(),
                ),
                (
                    ph2d_tool_painter::ids::PAINTER_WETPAINT_DRYCANVAS,
                    tr("panel.painter_layers.wetpaint.dry_canvas").into(),
                ),
                (
                    ph2d_tool_painter::ids::PAINTER_WETPAINT_FASTDRY,
                    tr("panel.painter_layers.wetpaint.fast_dry").into(),
                ),
            ],
            usize::MAX,
        );
        y = paint_display_toggles(ctx, theme, x, content_w, y, &brush);
    }
    let out = y;
    crate::paint_brush_top::end_fold(ctx, fold, out)
}

/// **O GRUPO DE RESOLUÇÃO** — os dois números que decidem de que tamanho é cada
/// grade, e o readout que os torna legíveis.
///
/// Extraído do corpo da seção pelo teto de 200 LOC de função dos painéis, e o
/// corte é por RESPONSABILIDADE: aqui mora *de que tamanho é a simulação*, e no
/// que sobra *o que ela faz* (as tools, os knobs de física).
fn paint_resolution_group(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let mut y = y;
    // ⚠️ **A GRADE é o PRIMEIRO widget da seção** (Enio 2026-07-29), acima
    // do rádio de tools: o custo do solver é linear nas células, então este
    // número decide a taxa VISUAL da água antes de qualquer knob de física
    // — e trocá-lo encerra a água viva (o bake), o que é uma decisão de
    // sessão, não de pincelada. Faixa e passo espelham `grid_map`
    // (MIN/MAX_RATIO) como os knobs espelham `KNOB_DEFS`.
    y = card_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "panel.painter_layers.wetpaint.grid_size_px",
        ph2d_tool_painter::ids::PAINTER_WETPAINT_GRID,
        f32::from(brush.wet_grid_ratio),
        GRID_MIN,
        GRID_MAX,
        1.0,
        0,
    );
    // A SEGUNDA metade da multi-resolução (plano 30): a velocidade e a
    // pressão ficam `Flow Grid` vezes mais grossas que o pigmento. Dois
    // números porque são duas perguntas — *quão fino é o pigmento?* e *quão
    // grosso é o fluxo?* — e um controle só governando os dois seria a
    // falha de duas-portas ao contrário.
    y = card_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "panel.painter_layers.wetpaint.flow_grid_x",
        ph2d_tool_painter::ids::PAINTER_WETPAINT_FLOW,
        f32::from(brush.wet_flow_ratio),
        FLOW_MIN,
        FLOW_MAX,
        1.0,
        0,
    );
    y = paint_resolution_readout(ctx, theme, x, content_w, y, brush);
    y
}

/// O TEXTO do readout — extraído para ser afirmável.
///
/// ⚠️ Ele existe como função porque a primeira versão shipou **"fluido/fluxo"**
/// no meio de "Grid Size", "Paint", "Erase" e "Smear" — duas línguas na mesma
/// seção, e nenhum gate do repo alcança essa classe: o `hr15_no_hardcoded_ui_strings`
/// varre `.label(...)` dentro de `ph2d-editor-core/src/widget/`, não `paint_text`
/// num painel. Um gate GERAL de idioma seria uma lista de palavras que nunca
/// fica completa; este pina a string que de fato falhou.
fn resolution_readout(fluid: (u32, u32), flow: (u32, u32)) -> String {
    let ((gw, gh), (fw, fh)) = (fluid, flow);
    if gw == 0 || gh == 0 {
        tr("panel.painter_layers.wetpaint.no_canvas").to_string()
    } else {
        tr_with(
            "panel.painter_layers.wetpaint.resolution",
            &[("gw", &gw), ("gh", &gh), ("fw", &fw), ("fh", &fh)],
        )
    }
}

/// **O readout DERIVADO das duas razões** — `fluid 2048x2048 - flow 512x512`.
///
/// ⚠️ **O texto é INGLÊS** porque a UI do app é inglês (o comentário é
/// português porque o código é lido por quem trabalha nele; a STRING é lida
/// pelo artista). A primeira versão shipou "fluido/fluxo" no meio de
/// "Grid Size" e "Paint/Erase/Smear" — duas línguas na mesma seção.
///
/// ⚠️ Ele não é decoração: sem ele o artista não tem como saber que *Grid 2 +
/// Flow 4* dá uma grade de fluxo de 512², e a lei desta linha é que **um limite
/// que não se vê é um limite que o artista descobre por acidente** (a duração
/// autorada da timeline, o teto do impasto). É derivado das MESMAS portas que o
/// motor usa (`grid_dims` e `flow_dims`) — uma segunda aritmética aqui
/// mostraria um tamanho que o solver não tem.
fn paint_resolution_readout(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    use ph2d_editor_core::paint::{paint_text, resolve};
    use ph2d_tokens::{ColorToken, Spacing, TypeToken};
    let text = resolution_readout(brush.wet_fluid_dims, brush.wet_flow_dims);
    let font = TypeToken::Sm.px();
    paint_text(
        ctx.text_system,
        ctx.scene,
        &text,
        x + Spacing::Sm.px(),
        y,
        font,
        content_w - 2.0 * Spacing::Sm.px(),
        resolve(ColorToken::Text2, theme),
    );
    y + font + ph2d_tokens::control_gap_px()
}

/// O **Real Oil** (Óleo Real, doc 49): a caixa e, só com ela ligada, as oito linhas da óptica do relevo —
/// Depth, Shadow, Gloss, a luz (altura e direcção, em GRAUS), a fenda (força e alcance, W2 E1) e a força do
/// ambiente (W2 E2), por cima do Environment; depois o MATERIAL (W2 E3-B, [`paint_oleo_material`]). As
/// faixas são as da ferramenta (`PAINTER_WETPAINT_OLEO_FAIXAS`), uma lei para quem pinta e quem prende.
fn paint_oleo_real(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: &BrushSettings,
) -> f32 {
    use ph2d_tool_painter::ids::{PAINTER_WETPAINT_OLEO_FAIXAS, PAINTER_WETPAINT_OLEO_FIELDS};
    let mut y = crate::paint_brush_top::paint_checkbox_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        ph2d_tool_painter::ids::PAINTER_WETPAINT_OLEO_REAL,
        "panel.painter_layers.wetpaint.oleo_real",
        brush.wet_oleo_real,
    );
    if !brush.wet_oleo_real {
        return y;
    }
    let linhas = [
        (
            "panel.painter_layers.wetpaint.oleo_depth",
            brush.wet_oleo_profundidade,
            number_field::FINE_STEP,
            2,
        ),
        (
            "panel.painter_layers.wetpaint.oleo_shadow",
            brush.wet_oleo_sombra,
            number_field::FINE_STEP,
            2,
        ),
        (
            "panel.painter_layers.wetpaint.oleo_gloss",
            brush.wet_oleo_brilho,
            number_field::FINE_STEP,
            2,
        ),
        (
            "panel.painter_layers.wetpaint.oleo_light_height",
            brush.wet_oleo_altitude_graus,
            number_field::ANGLE_STEP,
            0,
        ),
        (
            "panel.painter_layers.wetpaint.oleo_light_direction",
            brush.wet_oleo_azimute_graus,
            number_field::ANGLE_STEP,
            0,
        ),
        (
            "panel.painter_layers.wetpaint.oleo_crevice",
            brush.wet_oleo_fenda,
            number_field::FINE_STEP,
            2,
        ),
        (
            "panel.painter_layers.wetpaint.oleo_crevice_reach",
            brush.wet_oleo_alcance_fenda,
            number_field::CELL_STEP,
            0,
        ),
        (
            "panel.painter_layers.wetpaint.oleo_environment_light",
            brush.wet_oleo_forca_ambiente,
            number_field::FINE_STEP,
            2,
        ),
    ];
    for (((chave, valor, passo, casas), id), (lo, hi)) in linhas
        .into_iter()
        .zip(
            &PAINTER_WETPAINT_OLEO_FIELDS
                [..ph2d_tool_painter::ids::PAINTER_WETPAINT_OLEO_MATERIAL_DE],
        )
        .zip(PAINTER_WETPAINT_OLEO_FAIXAS)
    {
        y = card_row(
            ctx, theme, x, content_w, y, chave, *id, valor, lo as f32, hi as f32, passo, casas,
        );
    }
    let cur = brush.wet_oleo_ambiente;
    let (y, open) = crate::paint_brush_rows::paint_dropdown_row_de_caixa(
        ctx,
        theme,
        x,
        content_w,
        y,
        "panel.painter_layers.wetpaint.oleo_environment",
        ph2d_tool_painter::ids::PAINTER_WETPAINT_OLEO_AMBIENTE,
        cur,
        nome_do_ambiente(cur),
    );
    if let Some(r) = open {
        PENDING_AMBIENTE_DD.with(|c| c.set(Some((r, cur))));
    }
    paint_oleo_material(ctx, theme, x, content_w, y, brush)
}

/// Os rótulos das linhas do material (nomes do Blender), pela ordem de `MaterialCartao::CAMPOS`; o que não cabe
/// parte-se em duas linhas ([`crate::duas_linhas`]).
pub(crate) const MATERIAL: [&str; ph2d_tool_painter::ids::PAINTER_WETPAINT_OLEO_MATERIAL.len()] = [
    "panel.painter_layers.wetpaint.oleo_metallic",
    "panel.painter_layers.wetpaint.oleo_roughness",
    "panel.painter_layers.wetpaint.oleo_ior_level",
    "panel.painter_layers.wetpaint.oleo_coat",
    "panel.painter_layers.wetpaint.oleo_coat_roughness",
    "panel.painter_layers.wetpaint.oleo_sheen",
    "panel.painter_layers.wetpaint.oleo_subsurface",
    "panel.painter_layers.wetpaint.oleo_subsurface_radius",
    "panel.painter_layers.wetpaint.oleo_transparency",
];

/// O **material** global do Real Oil (W3): os 8 controlos do Principled simplificado, com os nomes do Blender. O
/// reflexo usa o **Environment** (o chip acima).
fn paint_oleo_material(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: &BrushSettings,
) -> f32 {
    use ph2d_tool_painter::ids::{
        PAINTER_WETPAINT_OLEO_FAIXAS, PAINTER_WETPAINT_OLEO_MATERIAL,
        PAINTER_WETPAINT_OLEO_MATERIAL_DE,
    };
    let mut y = y;
    for k in 0..MATERIAL.len() {
        let (lo, hi) = PAINTER_WETPAINT_OLEO_FAIXAS[PAINTER_WETPAINT_OLEO_MATERIAL_DE + k];
        y = card_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            MATERIAL[k],
            PAINTER_WETPAINT_OLEO_MATERIAL[k],
            brush.wet_oleo_material[k],
            lo as f32,
            hi as f32,
            number_field::FINE_STEP,
            2,
        );
    }
    y
}

/// Os rótulos do **Environment**, pela ordem de `Ambiente::TODOS` (o wire do chip).
const AMBIENTES: [&str; ph2d_tool_painter::ids::PAINTER_WETPAINT_OLEO_AMBIENTE_COUNT as usize] = [
    "panel.painter_layers.wetpaint.oleo_env_classic",
    "panel.painter_layers.wetpaint.oleo_env_studio",
    "panel.painter_layers.wetpaint.oleo_env_interior",
    "panel.painter_layers.wetpaint.oleo_env_courtyard",
];

fn nome_do_ambiente(i: u8) -> &'static str {
    tr(AMBIENTES
        .get(usize::from(i))
        .copied()
        .unwrap_or(AMBIENTES[1]))
}

thread_local! {
    /// O menu do Environment aberto neste quadro (o chip e o índice de agora), pintado por cima do corpo.
    static PENDING_AMBIENTE_DD: std::cell::Cell<Option<(ph2d_editor_core::zones::Rect, u8)>> =
        const { std::cell::Cell::new(None) };
}

/// O popover do **Environment** (chamado de `paint_brush_popovers`, depois do clip do corpo).
pub(crate) fn paint_wetpaint_popovers(ctx: &mut PaintCtx, theme: ph2d_tokens::Theme) {
    if let Some((r, cur)) = PENDING_AMBIENTE_DD.with(std::cell::Cell::take) {
        let opcoes = (0..ph2d_tool_painter::ids::PAINTER_WETPAINT_OLEO_AMBIENTE_COUNT)
            .map(|i| {
                ph2d_editor_core::widget::DropdownOption::new(
                    ph2d_tool_painter::ids::painter_wetpaint_oleo_ambiente_option_id(i),
                    i,
                    nome_do_ambiente(i),
                )
            })
            .collect();
        crate::dropdown_popover::paint_dropdown_popover(
            ctx,
            theme,
            ph2d_tool_painter::ids::PAINTER_WETPAINT_OLEO_AMBIENTE,
            opcoes,
            r,
            cur,
        );
    }
}

/// O fim da secção — Show Wet, Smooth Edges (a intensidade, `0..1`), Live Water (W1g) e Tuning. Partida de
/// [`paint_wetpaint_section`] pelo tecto de 200 LOC de função dos painéis.
fn paint_display_toggles(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: &BrushSettings,
) -> f32 {
    let mut y = y;
    y = crate::paint_brush_top::paint_checkbox_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        ph2d_tool_painter::ids::PAINTER_WETPAINT_SHOWWET,
        "panel.painter_layers.wetpaint.show_wet",
        brush.wet_show_wet,
    );
    // (O «Paper» que morava aqui é a Texture on Paint da secção Paper desde a W1e: um papel, uma secção.)
    // Smooth Edges — a intensidade do meio-tom na silhueta (a lei do Watercolor; `0` de fábrica).
    y = card_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "panel.painter_layers.wetpaint.smooth_edges",
        ph2d_tool_painter::ids::PAINTER_WETPAINT_SMOOTH_EDGES,
        brush.wet_smooth_edges,
        0.0,
        1.0,
        number_field::FINE_STEP,
        2,
    );
    y = paint_oleo_real(ctx, theme, x, content_w, y, brush);
    // Live Water (W1g) — a água corre com o traço; desligada de fábrica (o custo está na dica).
    if brush.le(ph2d_tool_painter::LinhaDaFerramenta::AguaViva) {
        y = crate::paint_brush_top::paint_checkbox_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            ph2d_tool_painter::ids::PAINTER_WETPAINT_LIVE_WATER,
            "panel.painter_layers.wetpaint.live_water",
            brush.wet_live_water,
        );
    }
    // Tuning — opens the side panel with the engine's full knob table.
    y = crate::paint_brush_top::paint_checkbox_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        ph2d_tool_painter::ids::PAINTER_WETPAINT_TUNING,
        "panel.painter_layers.wetpaint.tuning",
        brush.wet_tuning_open,
    );
    y
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **O readout é INGLÊS**, como todo o resto da UI do app.
    ///
    /// Mutação: voltar a "fluido {gw}x{gh} - fluxo {fw}x{fh}" sangra aqui.
    #[test]
    fn the_resolution_readout_speaks_the_apps_language() {
        assert_eq!(
            resolution_readout((1024, 1024), (256, 256)),
            "fluid 1024x1024 - flow 256x256"
        );
        assert_eq!(resolution_readout((0, 0), (0, 0)), "no canvas");
    }
}
