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

/// A borracha da água não pousa: o Pickup, o pó do pincel e os knobs que só agem a cada dab de tinta (Wet bloom, Tip
/// pickup, Re-wet, Drag, Solubility, Glaze, Cover, Dry cover, Wet soften) saem (`DepositoMolhado`); os da E10 também.
const DO_DEPOSITO: [ph2d_a11y::NodeId; 11] = [
    ph2d_tool_painter::ids::PAINTER_WETPAINT_PICKUP,
    ph2d_tool_painter::ids::PAINTER_WETPAINT_DUST,
    ph2d_tool_painter::ids::PAINTER_WETPAINT_WET_BLOOM,
    ph2d_tool_painter::ids::PAINTER_WETPAINT_TIP_PICKUP,
    ph2d_tool_painter::ids::PAINTER_WETPAINT_REWET,
    ph2d_tool_painter::ids::PAINTER_WETPAINT_REWET_DRAG,
    ph2d_tool_painter::ids::PAINTER_WETPAINT_SOLUBILITY,
    ph2d_tool_painter::ids::PAINTER_WETPAINT_GLAZE,
    ph2d_tool_painter::ids::PAINTER_WETPAINT_COVER,
    ph2d_tool_painter::ids::PAINTER_WETPAINT_DRY_COVER,
    ph2d_tool_painter::ids::PAINTER_WETPAINT_WET_SOFTEN,
];

/// **Os controlos NUMÉRICOS de lei da E10 no cartão** (F3 §9): `(depois de que linha, rótulo, id)`. A Hardness junto
/// do Pickup (o contacto com o papel); o Body junto do Leveling (o que nivela × o que resiste). O Medium é o menu
/// Paint type que abre o cartão ([`tipo_de_tinta`], E10f). Só no Pincel (`DepositoMolhado`).
const E10_NO_CARTAO: [(Option<ph2d_a11y::NodeId>, &str, ph2d_a11y::NodeId); 2] = [
    (
        Some(ph2d_tool_painter::ids::PAINTER_WETPAINT_PICKUP),
        "panel.painter_layers.wetpaint.hardness",
        ph2d_tool_painter::ids::PAINTER_WETPAINT_HARDNESS,
    ),
    (
        Some(ph2d_tool_painter::ids::PAINTER_WETPAINT_LEVELING),
        "panel.painter_layers.wetpaint.body",
        ph2d_tool_painter::ids::PAINTER_WETPAINT_BODY,
    ),
];

/// Um número do cartão: o nome, o id, o valor, a faixa, o passo, as casas. O painel de antes e os cartões leem esta
/// tabela; as condições moram nas funções que a montam.
pub(crate) struct Numero {
    pub(crate) chave: &'static str,
    pub(crate) id: ph2d_a11y::NodeId,
    pub(crate) valor: f32,
    pub(crate) min: f32,
    pub(crate) max: f32,
    pub(crate) passo: f64,
    pub(crate) casas: usize,
}

impl Numero {
    /// A linha do painel de antes.
    pub(crate) fn linha_de_antes(
        &self,
        ctx: &mut PaintCtx,
        theme: ph2d_tokens::Theme,
        x: f32,
        content_w: f32,
        y: f32,
    ) -> f32 {
        card_row(
            ctx, theme, x, content_w, y, self.chave, self.id, self.valor, self.min, self.max,
            self.passo, self.casas,
        )
    }

    /// Um knob da tabela do motor: as casas saem do passo.
    pub(crate) fn de_knob(
        chave: &'static str,
        id: ph2d_a11y::NodeId,
        v: f64,
        min: f64,
        max: f64,
        passo: f64,
    ) -> Self {
        let casas = ph2d_tool_painter::Escala {
            min: min as f32,
            max: max as f32,
            step: passo,
        }
        .decimals();
        Self {
            chave,
            id,
            valor: v as f32,
            min: min as f32,
            max: max as f32,
            passo,
            casas,
        }
    }
}

/// O controlo da E10 que vem depois de `depois`, se houver: faixa e passo da ferramenta (`WetKnobs::linha_e10`, a
/// `KNOB_DEFS`), condições de `leis`, valor de `valores`.
fn numero_e10_depois(
    leis: &BrushSettings,
    valores: &BrushSettings,
    depois: Option<ph2d_a11y::NodeId>,
) -> Option<Numero> {
    let &(_, rotulo, id) = E10_NO_CARTAO.iter().find(|(d, _, _)| *d == depois)?;
    // O Pincel lê os três; o Blend da água (3048 texels no censo `cada_ferramenta_da_agua_so_ve_o_que_le`) e a
    // borracha da água (`apply_erase`, censo `a_borracha_em_cada_meio_so_ve_o_que_o_meio_lhe_muda`) também lêem o
    // `paperGate` (a Hardness, E10h).
    let pousa = leis.le(ph2d_tool_painter::LinhaDaFerramenta::DepositoMolhado);
    let le_o_papel = leis.eraser || leis.wet_tool == ph2d_tool_painter::WetTool::Blend;
    if !(pousa || le_o_papel && id == ph2d_tool_painter::ids::PAINTER_WETPAINT_HARDNESS) {
        return None;
    }
    let (v, min, max, step) = valores.wet_knobs.linha_e10(id)?;
    Some(Numero::de_knob(rotulo, id, v, min, max, step))
}

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
            for n in numeros_da_ferramenta(&brush, &brush) {
                y = n.linha_de_antes(ctx, theme, x, content_w, y);
            }
        }
        // The TILT dial (the model's polar pad, copied faithfully).
        y = crate::paint_wetpaint_tilt::paint_tilt_card(ctx, theme, x, content_w, y, &brush);
        // A Dilution e o Volume são VOLUMES do topo (`paint_wetpaint_presets.rs`): um só controlo por facto. O
        // Paint type (Water · Oil) abre o cartão (E10f, no lugar do Pigment).
        let pousa = brush.le(ph2d_tool_painter::LinhaDaFerramenta::DepositoMolhado);
        if pousa {
            y = tipo_de_tinta::paint_tipo_de_tinta(ctx, theme, x, content_w, y, &brush);
        }
        for n in numeros_da_fisica(&brush, &brush).iter().flatten() {
            y = n.linha_de_antes(ctx, theme, x, content_w, y);
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

/// Os knobs da ferramenta da água na mão (dono 25/10): condição de `leis`, valor de `valores`.
pub(crate) fn numeros_da_ferramenta(leis: &BrushSettings, valores: &BrushSettings) -> Vec<Numero> {
    valores
        .wet_knobs
        .knobs_da_ferramenta(leis.wet_tool)
        .into_iter()
        .map(|(id, rotulo, v, min, max, step)| Numero::de_knob(rotulo, id, v, min, max, step))
        .collect()
}

/// Quantos nomes de [`numeros_da_fisica`] são do grupo da espessura: os restantes são os do pigmento.
const NOMES_DA_ESPESSURA: usize = 5;

/// Os números da física, pela ordem do cartão, por grupo: contacto e secagem · espessura · pigmento e camadas.
/// Condições de `leis`, valores de `valores`: os cartões leem a fábrica pela mesma tabela.
pub(crate) fn numeros_da_fisica(leis: &BrushSettings, valores: &BrushSettings) -> [Vec<Numero>; 3] {
    let k = valores.wet_knobs;
    let pousa = leis.le(ph2d_tool_painter::LinhaDaFerramenta::DepositoMolhado);
    let mut grupos: [Vec<Numero>; 3] = [Vec::new(), Vec::new(), Vec::new()];
    let rows: [(&'static str, ph2d_a11y::NodeId, f32, f32, f32, f64, usize); 5] = [
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
    for (chave, id, valor, min, max, passo, casas) in rows {
        if pousa || !DO_DEPOSITO.contains(&id) {
            grupos[0].push(Numero {
                chave,
                id,
                valor,
                min,
                max,
                passo,
                casas,
            });
        }
        grupos[0].extend(numero_e10_depois(leis, valores, Some(id)));
    }
    // A física da água que regula a «grossura» dela (saiu do Tuning, dono 13/10) e o pó do pincel (19/10): a faixa e o
    // passo vêm da tabela do motor, a mesma do Tuning.
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
        "panel.painter_wet_cartao.rewet",
        "panel.painter_wet_cartao.rewet_drag",
        "panel.painter_wet_cartao.solubility",
        "panel.painter_wet_cartao.staining",
        "panel.painter_wet_cartao.glaze",
        "panel.painter_wet_cartao.cover",
        "panel.painter_wet_cartao.dry_cover",
        "panel.painter_wet_cartao.wet_soften",
    ];
    for (i, (chave, (id, v, min, max, passo))) in
        nomes.into_iter().zip(k.knobs_no_cartao()).enumerate()
    {
        if !pousa && DO_DEPOSITO.contains(&id) {
            continue;
        }
        let grupo = usize::from(i >= NOMES_DA_ESPESSURA) + 1;
        grupos[grupo].push(Numero::de_knob(chave, id, v, min, max, passo));
        grupos[grupo].extend(numero_e10_depois(leis, valores, Some(id)));
    }
    grupos
}

/// Grid Size e Flow Grid, a mesma faixa das duas caras.
pub(crate) fn numeros_da_grade(b: &BrushSettings) -> [Numero; 2] {
    [
        Numero {
            chave: "panel.painter_layers.wetpaint.grid_size_px",
            id: ph2d_tool_painter::ids::PAINTER_WETPAINT_GRID,
            valor: f32::from(b.wet_grid_ratio),
            min: GRID_MIN,
            max: GRID_MAX,
            passo: 1.0,
            casas: 0,
        },
        Numero {
            chave: "panel.painter_layers.wetpaint.flow_grid_x",
            id: ph2d_tool_painter::ids::PAINTER_WETPAINT_FLOW,
            valor: f32::from(b.wet_flow_ratio),
            min: FLOW_MIN,
            max: FLOW_MAX,
            passo: 1.0,
            casas: 0,
        },
    ]
}

/// O texto do readout da resolução.
pub(crate) fn texto_da_resolucao(b: &BrushSettings) -> String {
    resolution_readout(b.wet_fluid_dims, b.wet_flow_dims)
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
    // A SEGUNDA metade da multi-resolução (plano 30): a velocidade e a
    // pressão ficam `Flow Grid` vezes mais grossas que o pigmento. Dois
    // números porque são duas perguntas — *quão fino é o pigmento?* e *quão
    // grosso é o fluxo?* — e um controle só governando os dois seria a
    // falha de duas-portas ao contrário.
    for n in numeros_da_grade(&brush) {
        y = n.linha_de_antes(ctx, theme, x, content_w, y);
    }
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

pub(crate) mod oleo;
pub(crate) mod tipo_de_tinta; // o menu Paint type (Water · Oil, E10f)
use oleo::paint_oleo_real;
pub(crate) use oleo::{MATERIAL, paint_wetpaint_popovers};

/// O fim da secção — Show Wet, Paint Texture (do traço), Live Water (W1g) e Tuning. Partida de
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
    // Paint Texture — o papel impresso DENTRO da tinta (o *Paint Texture* do Rebelle). É do TRAÇO (dono 28/10: «por
    // traço e não para o papel todo»): pousa na célula, vale para os seguintes, viaja com o preset e com a ferramenta.
    // (O Smooth Edges é da TELA desde a W3b — D1 do dono 27/10: mora no cartão Paper.)
    if brush.le(ph2d_tool_painter::LinhaDaFerramenta::DepositoMolhado) {
        y = crate::paint_brush_top::paint_checkbox_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            ph2d_tool_painter::ids::PAINTER_WETPAINT_PAPER_VISUAL,
            "panel.painter_layers.paper.texture_on_paint",
            brush.wet_paper_visual,
        );
    }
    y = paint_oleo_real(ctx, theme, x, content_w, y, brush);
    // Pigment mixing (K-M, dono 30/10): do pincel, ligado de fábrica; lei da sim — à vista de quem MOVE tinta por ela
    // (o Pincel e o Blow), não da Borracha nem do Smear/Blend/Wet/Dry da água (inertes).
    if brush.le(ph2d_tool_painter::LinhaDaFerramenta::MisturaDoPigmento) {
        y = crate::paint_brush_top::paint_checkbox_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            ph2d_tool_painter::ids::PAINTER_WETPAINT_PIGMENT_MIXING,
            "panel.painter_layers.wetpaint.pigment_mixing",
            brush.wet_km_mixing,
        );
    }
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
