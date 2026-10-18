//! **«Randomize Color não funciona se uso Shape Color»** (dono, 17/10): com uma rampa dona da cor (a Shape
//! Color — Texture Color / Falloff Color — ou a do Grain) a cor de cada texel vinha da LUT e o desvio
//! sorteado do carimbo (`Dab::color_shift`) não chegava a ela. A lei: a cor da rampa desloca-se pelo
//! desvio DESTE carimbo (`apply_color_shift`, a do Bug #51), em todas as rotas — o núcleo por pixel, o
//! carimbo em cache com a LUT, o carimbo de cor assado, o Wet Paint e a aquarela.
//!
//! A régua: o MESMO traço com o Randomize ligado e desligado; conta as amostras cujo matiz mudou. Antes
//! da lei, zero em todas as rotas. Cada caso digital prova a ROTA que tomou pelo cache que ela enche.

use super::super::media::PaintMedia;
use super::*;
use ph2d_color::{ColorRamp, RampColorMode, RampInterp, RampStop};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::CanvasPaintTool;
use ph2d_painter_brush::texture::{TextureKind, TextureMapping};

const LARGO: u32 = 128;
const ALTO: u32 = 64;

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

/// A tela branca e o pincel do `meio`: o Wet Paint com o dele (raio 14, relógio fixo, como em
/// `wetpaint::shape_color_tests`), os outros com o pincel duro de raio 10.
fn tela(meio: PaintMedia) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (LARGO * ALTO * 4) as usize], LARGO, ALTO);
    t.set_paint_media(meio);
    t.paint.brush.color = [0.2, 0.2, 0.9];
    if meio == PaintMedia::WetPaint {
        t.set_wet_relogio_fixo(true);
        t.set_brush_size_px(14.0);
        return t;
    }
    t.paint.brush.radius_px = 10.0;
    t.paint.brush.hardness = 1.0;
    t.paint.brush.falloff = Falloff::Constant;
    t.paint.brush.space_attenuation = false;
    t.paint.brush.spacing = 0.6;
    t
}

/// A Shape Color em cor (B&W desligado) com a rampa verde → vermelho nas camadas pedidas.
fn arma_shape_color(t: &mut PainterTool, textura: bool, queda: bool) {
    t.set_shape_color_ramp(verde_vermelho());
    t.set_shape_falloff_ramp(verde_vermelho());
    t.set_shape_ramp_enabled(true);
    t.paint.shape_color_ramp_bw = false;
    t.paint.shape_camadas.textura = textura;
    t.paint.shape_camadas.queda = queda;
    t.set_shape_falloff_ramp(verde_vermelho()); // re-assa as LUTs
}

/// O padrão procedural do Shape (xadrez largo) no mapeamento `m`.
fn shape_xadrez(t: &mut PainterTool, m: TextureMapping) {
    t.set_brush_shape_kind(TextureKind::Checker.to_u8());
    t.paint.brush.shape.mapping = m;
}

/// Um traço horizontal com um quadro por evento e `assenta` quadros depois; devolve as 11 amostras da
/// linha do meio.
fn traco(t: &mut PainterTool, randomize: bool, assenta: u32) -> Vec<[u8; 4]> {
    if randomize {
        t.paint.brush.color_jitter_hue = 1.0;
    }
    let seed = t.paint.brush;
    t.paint.brush_by_mode.fill(seed);
    let y = ALTO as f32 / 2.0;
    t.on_canvas_pointer(cp([16.0, y], PointerPhase::Down));
    t.on_tick(1000.0 / 60.0);
    for k in 1..=24 {
        #[allow(clippy::cast_precision_loss)]
        t.on_canvas_pointer(cp([16.0 + 4.0 * k as f32, y], PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(cp([112.0, y], PointerPhase::Up));
    for _ in 0..assenta {
        t.on_tick(1000.0 / 60.0);
    }
    (24..=104)
        .step_by(8)
        .map(|x| px(t, LARGO, x, ALTO / 2))
        .collect()
}

/// O matiz `[0, 1)` de um pixel e se ele tem cor bastante para ter matiz.
fn matiz(p: [u8; 4]) -> Option<f32> {
    let [r, g, b] = [p[0], p[1], p[2]].map(|v| f32::from(v) / 255.0);
    let (max, min) = (r.max(g).max(b), r.min(g).min(b));
    let d = max - min;
    if d < 0.15 {
        return None;
    }
    let h = if max == r {
        (g - b) / d
    } else if max == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };
    Some((h / 6.0).rem_euclid(1.0))
}

/// Quantas amostras do traço com Randomize mudaram de matiz face ao MESMO traço sem ele.
fn mudaram(sem: &[[u8; 4]], com: &[[u8; 4]]) -> usize {
    sem.iter()
        .zip(com)
        .filter(|(a, b)| match (matiz(**a), matiz(**b)) {
            (Some(x), Some(y)) => {
                let d = (x - y).abs();
                d.min(1.0 - d) > 0.08
            }
            (None, None) => false,
            _ => true,
        })
        .count()
}

/// O CONTROLO: o traço sem Randomize pinta as cores da rampa (verde ou vermelho), nunca a do pincel azul
/// — com a margem de uma velatura (o Wet Paint pinta a rampa vermelha como `[248, 216, 187]`).
fn pinta_a_rampa(sem: &[[u8; 4]]) {
    let da_rampa = sem
        .iter()
        .filter(|p| i32::from(p[0].max(p[1])) > i32::from(p[2]) + 30)
        .count();
    assert!(
        da_rampa >= 9,
        "controlo: sem Randomize o traço pinta a rampa: {sem:?}"
    );
}

/// A rota que o traço tomou, pelo cache que ela enche.
#[derive(Debug, PartialEq)]
enum Rota {
    PorPixel,
    CacheComLut,
    CarimboAssado,
}

fn rota(t: &PainterTool) -> Rota {
    match (
        t.paint.ramp_color_stamp_cache.is_some(),
        t.paint.stamp_cache.is_some(),
    ) {
        (true, false) => Rota::CarimboAssado,
        (false, true) => Rota::CacheComLut,
        (false, false) => Rota::PorPixel,
        (true, true) => panic!("as duas caches cheias: rota ambígua"),
    }
}

/// O par de traços (sem e com Randomize) de um `arma`; devolve as amostras e a rota de cada um.
fn par(
    meio: PaintMedia,
    arma: impl Fn(&mut PainterTool),
) -> ((Vec<[u8; 4]>, Rota), (Vec<[u8; 4]>, Rota)) {
    let corre = |randomize| {
        let mut t = tela(meio);
        arma(&mut t);
        let assenta = if meio == PaintMedia::WetPaint { 60 } else { 4 };
        let a = traco(&mut t, randomize, assenta);
        (a, rota(&t))
    };
    (corre(false), corre(true))
}

fn exige_o_randomize(nome: &str, sem: &[[u8; 4]], com: &[[u8; 4]]) {
    pinta_a_rampa(sem);
    let n = mudaram(sem, com);
    assert!(
        n >= 4,
        "{nome}: o Randomize Color com a Shape Color muda o matiz em só {n}/11 amostras\n sem: {sem:?}\n com: {com:?}"
    );
}

/// Rota 1 — o núcleo POR PIXEL (`stamp_dabs_ramped`): o xadrez em Tiled não cabe em cache nenhum.
#[test]
fn o_randomize_desloca_a_shape_color_no_nucleo_por_pixel() {
    let ((sem, r0), (com, r1)) = par(PaintMedia::Digital, |t| {
        shape_xadrez(t, TextureMapping::Tiled);
        arma_shape_color(t, true, false);
    });
    assert_eq!((r0, r1), (Rota::PorPixel, Rota::PorPixel));
    exige_o_randomize("por pixel", &sem, &com);
}

/// Rota 2 — o carimbo em cache com a LUT (`stamp_dabs_cached_ramped`): sem Grain e só a Falloff Color,
/// a cobertura indexa a rampa.
#[test]
fn o_randomize_desloca_a_falloff_color_no_carimbo_em_cache() {
    let ((sem, r0), (com, r1)) = par(PaintMedia::Digital, |t| {
        arma_shape_color(t, false, true);
    });
    assert_eq!((r0, r1), (Rota::CacheComLut, Rota::CacheComLut));
    exige_o_randomize("cache com a LUT", &sem, &com);
}

/// Rota 3 — o carimbo de cor ASSADO (`stamp_dabs_cached_ramp_color`): o padrão procedural vira cor pela
/// Texture Color. O assado é do traço inteiro; o desvio entra no blit, carimbo a carimbo.
#[test]
fn o_randomize_desloca_a_texture_color_no_carimbo_assado() {
    let ((sem, r0), (com, r1)) = par(PaintMedia::Digital, |t| {
        shape_xadrez(t, TextureMapping::ViewPlane);
        arma_shape_color(t, true, false);
    });
    assert_eq!((r0, r1), (Rota::CarimboAssado, Rota::CarimboAssado));
    exige_o_randomize("carimbo assado", &sem, &com);
}

/// A rampa do GRAIN dona da cor, pelo carimbo assado — a mesma porta (`ramp_texel`), o índice do Grain.
#[test]
fn o_randomize_desloca_a_rampa_do_grain() {
    let ((sem, r0), (com, r1)) = par(PaintMedia::Digital, |t| {
        t.set_brush_texture_kind(TextureKind::Checker.to_u8());
        t.set_brush_texture_mapping(TextureMapping::ViewPlane.to_u8());
        t.set_shape_ramp_enabled(false);
        t.set_texture_ramp(verde_vermelho());
        t.set_texture_ramp_enabled(true);
        t.paint.texture_ramp_bw = false;
        t.paint.texture_ramp_dirty = true;
    });
    assert_eq!((r0, r1), (Rota::CarimboAssado, Rota::CarimboAssado));
    exige_o_randomize("rampa do Grain", &sem, &com);
}

/// Wet Paint — a cor de cada célula pela `RampaDaAguada` (Falloff Color).
#[test]
fn o_randomize_desloca_a_shape_color_no_wet_paint() {
    let ((sem, _), (com, _)) = par(PaintMedia::WetPaint, |t| {
        arma_shape_color(t, false, true);
    });
    exige_o_randomize("Wet Paint", &sem, &com);
}

/// Aquarela — o splat da cor pela `RampaDaAguada` (Falloff Color).
#[test]
fn o_randomize_desloca_a_shape_color_na_aquarela() {
    let ((sem, _), (com, _)) = par(PaintMedia::Watercolor, |t| {
        arma_shape_color(t, false, true);
    });
    exige_o_randomize("aquarela", &sem, &com);
}
