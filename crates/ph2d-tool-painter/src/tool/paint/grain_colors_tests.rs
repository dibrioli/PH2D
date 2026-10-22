//! ⭐ **As Grain Colors nos QUATRO meios** (dono 22/10): o valor do grão indexa a rampa e a cor da
//! rampa é a da tinta — a lei do digital (`ph2d_painter_brush::RampPaint::cor`). No Digital e no
//! Impasto o grão do carimbo; na Aquarela o mapa de granulação (o Grain com «Same as Paper» desligado);
//! no Wet Paint o grão do dab. Desligadas, cada meio pinta ao byte o de antes (os fingerprints de cada
//! meio e os portões por foto da fábrica).

use super::measure_shape_system::{cp, tool};
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_color::{ColorRamp, RampColorMode, RampInterp, RampStop};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
use ph2d_painter_brush::TextureKind;

const MEIOS: [PaintMedia; 4] = [
    PaintMedia::Digital,
    PaintMedia::Watercolor,
    PaintMedia::Impasto,
    PaintMedia::WetPaint,
];

/// A rampa verde (grão `0`) → vermelho (grão `1`): as duas pontas na tinta provam que é o VALOR do grão
/// que a indexa, e não uma cor só.
fn verde_vermelho() -> ColorRamp {
    ColorRamp::new(
        vec![
            RampStop::new(0.0, [0.0, 1.0, 0.0, 1.0]),
            RampStop::new(1.0, [1.0, 0.0, 0.0, 1.0]),
        ],
        RampColorMode::Rgb,
        RampInterp::Linear,
    )
}

/// Um pincel azul com um Grain xadrez (na Aquarela: o mapa de granulação próprio), armado por `arma`,
/// num traço recto com o relógio a correr.
fn pinta_t(media: PaintMedia, arma: impl Fn(&mut PainterTool)) -> PainterTool {
    let mut t = tool(128, media, 12.0);
    t.set_wet_relogio_fixo(true);
    t.set_brush_color_srgb8([30, 60, 220]);
    t.set_brush_texture_kind(TextureKind::Checker.to_u8());
    t.paint.brush.granulation_use_paper = false;
    arma(&mut t);
    t.on_canvas_pointer(cp([20.0, 64.0], PointerPhase::Down));
    t.on_tick(1000.0 / 60.0);
    for k in 1..=22u8 {
        let x = 20.0 + 4.0 * f32::from(k);
        t.on_canvas_pointer(cp([x, 64.0], PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(cp([108.0, 64.0], PointerPhase::Up));
    for _ in 0..4 {
        t.on_tick(1000.0 / 60.0);
    }
    t
}

fn pinta(media: PaintMedia, arma: impl Fn(&mut PainterTool)) -> Vec<u8> {
    pinta_t(media, arma).canvas_rgba.to_vec()
}

fn liga(t: &mut PainterTool) {
    t.set_texture_ramp(verde_vermelho());
    t.set_texture_ramp_enabled(true);
}

/// Texels onde o canal `c` domina os outros dois por uma margem visível.
fn dominados_por(px: &[u8], c: usize) -> usize {
    px.chunks(4)
        .filter(|p| {
            let v = i32::from(p[c]);
            (0..3).filter(|&k| k != c).all(|k| v > i32::from(p[k]) + 40)
        })
        .count()
}

/// Quantas vezes, ao longo da linha `y` do traço, a cor dominante troca entre verde e vermelho — a
/// assinatura do xadrez do GRÃO (uma rampa indexada pela queda do carimbo é constante ao longo do
/// traço horizontal: zero trocas).
fn trocas_ao_longo(px: &[u8], y: usize) -> usize {
    let mut ultima = None;
    let mut n = 0;
    for x in 16..112 {
        let p = &px[(y * 128 + x) * 4..][..4];
        let (r, g) = (i32::from(p[0]), i32::from(p[1]));
        let c = if g > r + 40 {
            Some('g')
        } else if r > g + 40 {
            Some('r')
        } else {
            None
        };
        if let Some(c) = c {
            if ultima.is_some_and(|u| u != c) {
                n += 1;
            }
            ultima = Some(c);
        }
    }
    n
}

/// **A cor do grão MUDA a tinta em cada meio** — com as Grain Colors ligadas o traço azul sai verde nas
/// casas baixas do grão e vermelho nas altas (o xadrez troca a cor ao longo do traço); sem elas, nem um
/// texel verde ou vermelho.
///
/// **Mutações que sangram:** a aguada voltar a só dar a dona Shape (`rampa_da_aguada` com
/// `color_ramp_owner(false)`) — Aquarela e Wet Paint ficam azuis; a aguada ou o Wet Paint indexarem a
/// rampa pela queda (`grao` = `None`) — a cor deixa de seguir o xadrez.
#[test]
fn as_grain_colors_pintam_a_cor_do_grao_nos_quatro_meios() {
    let mut falhas = Vec::new();
    for m in MEIOS {
        let sem = pinta(m, |_| {});
        let com = pinta(m, liga);
        let (sv, sr) = (dominados_por(&sem, 1), dominados_por(&sem, 0));
        let (cv, cr) = (dominados_por(&com, 1), dominados_por(&com, 0));
        if sv + sr > 0 {
            falhas.push(format!(
                "{m:?}: o controlo azul já tinha {sv} verdes e {sr} vermelhos"
            ));
        }
        if cv < 40 || cr < 40 {
            falhas.push(format!(
                "{m:?}: as Grain Colors não chegaram à tinta ({cv} verdes, {cr} vermelhos)"
            ));
        }
        let n = trocas_ao_longo(&com, 64);
        if n < 3 {
            falhas.push(format!(
                "{m:?}: a cor não segue o grão ao longo do traço ({n} trocas verde↔vermelho)"
            ));
        }
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

/// **Onde o meio NÃO lê o grão, a linha esmaece com o motivo** — e a tinta não muda: a Aquarela com
/// «Same as Paper» (a lavagem assenta no papel, o slot Grain não é lido) e o Wet Paint fora da
/// ferramenta Paint (Smear, Blend, Wet, Dry, Blow não pousam a cor da rampa).
#[test]
fn as_grain_colors_esmaecem_onde_o_meio_nao_le_o_grao() {
    use crate::ids::PAINTER_BRUSH_TEXTURE_RAMP_ENABLE as ID;
    let dica = |t: &PainterTool| t.inercias().dica_de(ID);
    for m in MEIOS {
        let t = pinta_t(m, liga);
        assert_eq!(
            dica(&t),
            None,
            "{m:?}: as Grain Colors agem e a linha esmaeceu"
        );
    }
    let papel = |t: &mut PainterTool| {
        liga(t);
        t.paint.brush.granulation_use_paper = true;
    };
    let t = pinta_t(PaintMedia::Watercolor, papel);
    assert_eq!(
        dica(&t),
        Some("panel.painter_layers.inerte.same_as_paper_off"),
        "Aquarela com Same as Paper: a linha não esmaeceu"
    );
    assert_eq!(
        pinta(PaintMedia::Watercolor, papel),
        pinta(PaintMedia::Watercolor, |t| t
            .paint
            .brush
            .granulation_use_paper =
            true),
        "Aquarela com Same as Paper: esmaecida mas a tinta mudou"
    );
    let mut t = pinta_t(PaintMedia::WetPaint, liga);
    t.pick_wet_tool(2);
    assert_eq!(
        dica(&t),
        Some("panel.painter_layers.inerte.paint_tool_only"),
        "Wet Paint fora da ferramenta Paint: a linha não esmaeceu"
    );
}
