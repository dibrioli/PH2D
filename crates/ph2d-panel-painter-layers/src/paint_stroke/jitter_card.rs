//! **O card JITTER** — o que randomiza uma MARCA, e quantas marcas um ponto do caminho deixa.
//!
//! Filho de [`super`] (a seção Stroke), cortado por ASSUNTO quando o pai cruzou o teto de LOC: a
//! seção responde *como este caminho é percorrido*, este card responde *com que aparência cada
//! marca cai*. A moldura do cartão, a unidade do espalhamento e a conversão da contagem moram aqui; as
//! fileiras e a ordem delas, em [`super::jitter_linhas`].

use super::jitter_linhas;
use ph2d_editor_core::paint::{fill_rounded_rect, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::DropdownOption;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken};
use ph2d_tool_painter::ids::painter_brush_jitter_unit_option_id;
use ph2d_tool_painter::{BrushSettings, CartaoDaMarca, JitterUnit, SPRAY_COUNT_MAX};

/// Paint the **Jitter** group inside a decorative rounded-rect card: the Unit menu on top, then every
/// numeric row as a slider-with-chip, in the order of [`super::jitter_linhas::FILEIRAS_DO_JITTER`]. The
/// card background is drawn first (its height summed from the same table), then the rows. Returns the next `y`.
pub(crate) fn paint_jitter_card(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let pad = Spacing::Sm.px();
    let xs = Spacing::Xs.px();
    let inner_w = (content_w - pad * 2.0).max(0.0);
    // A altura é CONTADA das fileiras que o cartão pinta (a mesma tabela, o mesmo avanço), nunca escolhida: um
    // número fixo por fileira fê-la transbordar em 2026-06-29.
    let rows_h: f32 = jitter_linhas::fileiras(&brush)
        .map(|f| jitter_linhas::altura(f, inner_w))
        .sum();
    let card_h = pad + rows_h + xs;
    let card = Rect::new(x, y, content_w, card_h);
    // ⭐ Raio e moldura pela porta do TEMA: o cartão é plano num tema moderno.
    let card_radius = ph2d_editor_core::paint::frame_radius(theme, Radius::Md.px());
    fill_rounded_rect(
        ctx.scene,
        card,
        card_radius,
        resolve(
            ph2d_editor_core::widget::section_cards::CardDepth::Subsection.token(),
            theme,
        ),
    );
    ph2d_editor_core::paint::stroke_frame(
        ctx.scene,
        card,
        card_radius,
        theme,
        ph2d_tokens::visuals::Feel::Rest,
        StrokeToken::Default.px(),
        resolve(ColorToken::Border, theme),
    );
    let inner_x = x + pad;
    // O título é o cabeçalho da sub-secção «Jitter» que o embrulha (`paint_stroke`): o cartão não o repete.
    // ⭐ As fileiras pela ordem do dono (23/10): uma tabela, `jitter_linhas::FILEIRAS_DO_JITTER`.
    jitter_linhas::paint_fileiras(ctx, theme, inner_x, inner_w, y + pad, &brush);
    y + card_h + ph2d_tokens::control_gap_px()
}

/// Os índices das linhas da marca que se pintam no cartão `c`.
pub(crate) fn linhas_do_cartao(c: CartaoDaMarca) -> impl Iterator<Item = usize> {
    ph2d_tool_painter::LINHAS_DA_MARCA
        .into_iter()
        .enumerate()
        .filter(move |(_, l)| l.cartao() == c)
        .map(|(i, _)| i)
}

/// Pinta as linhas da marca do cartão `c` (uma tabela no tool, `marca.rs`); devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_linhas_da_marca(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    w: f32,
    mut y: f32,
    brush: &BrushSettings,
    c: CartaoDaMarca,
    sec: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    for i in linhas_do_cartao(c) {
        let (lo, hi, passo) = ph2d_tool_painter::LINHAS_DA_MARCA[i].faixa();
        y = crate::number_field::paint_num_row(
            ctx,
            theme,
            x,
            w,
            y,
            tr(ROTULOS_DA_MARCA[i]),
            ph2d_tool_painter::ids::PAINTER_MARCA_FIELDS[i],
            brush.marca[i] as f32,
            lo as f32,
            hi as f32,
            passo,
            2,
            sec,
        );
    }
    y
}

/// O rótulo de cada linha da marca, pela ordem de [`ph2d_tool_painter::LINHAS_DA_MARCA`].
pub(crate) const ROTULOS_DA_MARCA: [&str; ph2d_tool_painter::N_LINHAS_DA_MARCA] = [
    "panel.painter_marca.opacity",
    "panel.painter_marca.count",
    "panel.painter_marca.roundness",
    "panel.painter_marca.flip_x",
    "panel.painter_marca.flip_y",
    "panel.painter_marca.angle_per_stroke",
    "panel.painter_marca.across",
    "panel.painter_marca.particle",
    "panel.painter_marca.secondary_jitter",
    "panel.painter_marca.pressure_secondary",
    "panel.painter_marca.pressure_hue",
    "panel.painter_marca.pressure_sat",
    "panel.painter_marca.pressure_val",
];

/// A contagem de marcas do spray na pista `0..1` do slider — a INVERSA exata do mapeamento que o
/// tool aplica (`set_brush_spray_count_norm`).
///
/// ⚠️ **Ela existe porque a pista tem de mostrar o que o tool guardou**, e as duas metades da mesma
/// conversão têm de ser inversas: uma pista derivada por outra fórmula faria o polegar pousar num
/// lugar e o número dizer outro. É a lei do *seed == sample* que esta casa já pagou várias vezes.
pub(super) fn spray_count_track(count: u32) -> f32 {
    let span = (SPRAY_COUNT_MAX - 1).max(1);
    // ⚠️ O marcador tem de estar NA linha do `clamp` (o gate lê a linha, não a de cima) — a mesma
    // cicatriz do `LITERAL-PX-OK`, que já foi reflowada para fora por um `rustfmt`.
    let n = count.clamp(1, SPRAY_COUNT_MAX); // CLAMP-OK: `u32` sem NaN, bordas constantes
    #[allow(clippy::cast_precision_loss)]
    let t = (n - 1) as f32 / span as f32;
    t
}

/// Display name for a jitter-unit wire discriminant.
pub(super) fn jitter_unit_name(u: u8) -> &'static str {
    match JitterUnit::from_u8(u) {
        JitterUnit::Brush => tr("panel.painter_layers.stroke.jitter.brush"),
        JitterUnit::View => tr("panel.painter_layers.stroke.jitter.view"),
    }
}

/// The two jitter units as dropdown options.
pub(super) fn jitter_unit_options() -> Vec<DropdownOption<u8>> {
    [0u8, 1]
        .into_iter()
        .map(|u| {
            DropdownOption::new(
                painter_brush_jitter_unit_option_id(u),
                u,
                jitter_unit_name(u),
            )
        })
        .collect()
}
