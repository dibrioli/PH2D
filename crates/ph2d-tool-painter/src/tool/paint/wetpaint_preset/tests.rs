//! Os gates do preset do Wet Painter do lado da ferramenta.
//!
//! - `todo_campo_do_pincel_tem_destino`: cada campo do `BrushSpec` (lido do FONTE) está em
//!   [`CAMPOS_DO_PINCEL`] ou em [`CAMPOS_FORA`], e só numa.
//! - `o_pincel_guardado_pinta_o_mesmo_traco`: um pincel com TODOS os campos mexidos (os de fora
//!   incluídos), capturado, gravado em ficheiro e relido, aplicado a uma ferramenta nova, pinta o
//!   mesmo traço ao byte — o preset leva tudo o que o traço lê, e o que fica de fora não o muda.

use super::*;
use ph2d_brush_presets::{Miniatura, Origem};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_painter_brush::{
    BrushBlend, BrushSpec, Falloff, FalloffCurve, FalloffPoint, FootprintCurve, HandleType,
    JitterUnit, TextureMapping, TextureSettings,
};
use ph2d_painter_brush::height_modes::{DepthSource, DrawTo};
use ph2d_painter_brush::taper::Taper;
use ph2d_wet_paint::tuning::Knob;
use std::collections::BTreeSet;

const W: u32 = 240;
const H: u32 = 120;

#[test]
fn todo_campo_do_pincel_tem_destino() {
    let fonte = include_str!("../../../../../ph2d-painter-brush/src/spec.rs");
    let corpo = fonte
        .split("pub struct BrushSpec {")
        .nth(1)
        .and_then(|r| r.split("\n}\n").next())
        .expect("a struct BrushSpec no spec.rs");
    let campos: BTreeSet<&str> = corpo
        .lines()
        .filter_map(|l| l.strip_prefix("    pub "))
        .filter_map(|l| l.split(':').next())
        .collect();
    let dentro: BTreeSet<&str> = CAMPOS_DO_PINCEL.iter().copied().collect();
    let fora: BTreeSet<&str> = CAMPOS_FORA.iter().map(|(c, _)| *c).collect();
    assert_eq!(dentro.len(), CAMPOS_DO_PINCEL.len(), "repetido em CAMPOS_DO_PINCEL");
    assert_eq!(fora.len(), CAMPOS_FORA.len(), "repetido em CAMPOS_FORA");
    assert!(dentro.is_disjoint(&fora), "um campo nas duas listas: {:?}", dentro.intersection(&fora));
    let declarados: BTreeSet<&str> = dentro.union(&fora).copied().collect();
    assert_eq!(
        campos.difference(&declarados).collect::<Vec<_>>(),
        Vec::<&&str>::new(),
        "campo novo do BrushSpec sem destino no preset: ponha-o em CAMPOS_DO_PINCEL ou em CAMPOS_FORA"
    );
    assert_eq!(
        declarados.difference(&campos).collect::<Vec<_>>(),
        Vec::<&&str>::new(),
        "campo que já não existe no BrushSpec"
    );
}

/// TODOS os campos mexidos. ⚠️ Sem `..default()`: um campo novo no `BrushSpec` não compila aqui até
/// ter um valor (e um destino em `CAMPOS_*`). Os da OBRA, do GESTO e do MEIO ficam no de fábrica —
/// são os mesmos nas duas ferramentas; todos os outros mudam.
fn pincel_mexido() -> BrushSpec {
    let d = BrushSpec::default();
    let tex = |kind: u8, ang: u16| TextureSettings {
        kind: ph2d_painter_brush::TextureKind::from_u8(kind),
        mapping: TextureMapping::Tiled,
        angle_deg: ang,
        rake: false,
        flow: false,
        offset: [0.1, -0.2],
        size: [0.7, 0.9],
        stencil_offset: [0.05, 0.0],
        stencil_size: [0.4, 0.45],
        stencil_angle_deg: 12,
        params: [0.4, 0.6, 0.55, 0.5, 0.3, 0.7],
    };
    let curva = FalloffCurve::from_points(&[
        FalloffPoint { id: 0, x: 0.0, y: 1.0, handle: HandleType::Auto },
        FalloffPoint { id: 3, x: 0.45, y: 0.8, handle: HandleType::Vector },
        FalloffPoint { id: 1, x: 1.0, y: 0.0, handle: HandleType::Auto },
    ])
    .unwrap();
    BrushSpec {
        // ── o preset ──
        hardness: 0.35,
        falloff: Falloff::Custom,
        custom_falloff: curva,
        spacing: 0.15,
        jitter: 0.12,
        jitter_unit: JitterUnit::View,
        jitter_absolute_px: 3.0,
        jitter_scale: 0.2,
        jitter_rotate: 0.3,
        jitter_spacing: 0.1,
        spray_count: 2,
        input_samples: 3,
        stabilizer: 0.25,
        taper: Taper { start: 1.5, tip_start: 0.2, opacity: 0.4 },
        texture: tex(3, 30),
        grain_depth: 0.6,
        shape: tex(5, 45),
        dab_flatten: 0.3,
        dab_angle_deg: 25,
        dab_curve: FootprintCurve::from_rows([[0.05, 0.0, 0.02, 0.0, 0.0, 0.0, 0.0]; 2]),
        color_jitter_enabled: true,
        color_jitter_hue: 0.05,
        color_jitter_sat: 0.1,
        color_jitter_val: 0.08,
        blend: BrushBlend::Multiply,
        pigment: true,
        pigment_mix: 0.4,
        paper_depth: 0.3,
        // ── volumes ──
        radius_px: 9.0,
        strength: 0.85,
        // ── da obra · do gesto · do meio · do sculpt: os mesmos nas duas ──
        color: d.color,
        paper: d.paper,
        symmetry: d.symmetry,
        stroke_method: d.stroke_method,
        grid_cell_px: d.grid_cell_px,
        grid_offset_px: d.grid_offset_px,
        grid_fit: d.grid_fit,
        edge_to_edge: d.edge_to_edge,
        line_kind: d.line_kind,
        style_solid: d.style_solid,
        sketchy_reach: d.sketchy_reach,
        sketchy_density: d.sketchy_density,
        thread_width_px: d.thread_width_px,
        thread_opacity: d.thread_opacity,
        sketchy_magnetify: d.sketchy_magnetify,
        wire_history: d.wire_history,
        wire_connection_line: d.wire_connection_line,
        ribbon_weight: d.ribbon_weight,
        ribbon_friction: d.ribbon_friction,
        ribbon_gravity: d.ribbon_gravity,
        ribbon_rungs: d.ribbon_rungs,
        rough_amount: d.rough_amount,
        rough_bowing: d.rough_bowing,
        rough_passes: d.rough_passes,
        dash_ratio: d.dash_ratio,
        dash_samples: d.dash_samples,
        needs_heading: d.needs_heading,
        watercolor: d.watercolor,
        impasto: d.impasto,
        // ── os de fora que o Wet Paint não deve ler: mexidos ──
        edge_gain: 2.5,
        edge_spread: 5.0,
        smooth_edges: !d.smooth_edges,
        granulation: 0.7,
        fill: 0.4,
        depth: 0.6,
        opacity: 0.5,
        warp: 3.0,
        edge_flow: tex(2, 10),
        paper_edge: 0.6,
        wet_smudge: 0.5,
        wet_rewet: 0.4,
        wet_charge: 0.3,
        wet_dilution: 0.6,
        wet_pull: 0.7,
        granulation_use_paper: !d.granulation_use_paper,
        watercolor_shape_auto: !d.watercolor_shape_auto,
        impasto_smooth_edges: !d.impasto_smooth_edges,
        impasto_depth: 0.8,
        film_depth: 0.4,
        impasto_source: DepthSource::Grain,
        impasto_draw_to: DrawTo::Depth,
        impasto_smoothing: 0.6,
        impasto_body: 0.7,
        impasto_plow: 0.5,
        impasto_push: 0.4,
        impasto_shine: 0.9,
        impasto_roughness: 0.3,
        impasto_metallic: 0.6,
        impasto_wax: 0.5,
        impasto_wax_color: [0.2, 0.4, 0.6],
        flow: 0.45,
        accumulate: !d.accumulate,
        airbrush_rate_s: 0.05,
        space_attenuation: !d.space_attenuation,
    }
}

fn mascara(w: u32, h: u32, fase: u32) -> Vec<u8> {
    (0..w * h)
        .map(|i| {
            let (x, y) = (i % w, i / w);
            (((x * 13 + y * 7 + fase) % 64) * 4) as u8
        })
        .collect()
}

fn ferramenta() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (W * H * 4) as usize], W, H);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    t
}

/// O pincel de A, inteiro: o `BrushSpec` mexido, as máscaras, os knobs e o modo.
fn ferramenta_mexida() -> PainterTool {
    let mut t = ferramenta();
    t.paint.brush = pincel_mexido();
    let forma = t.paint.brush.shape;
    let grao = t.paint.brush.texture;
    t.set_brush_shape_image(mascara(17, 13, 0), 17, 13);
    t.set_brush_texture_image(mascara(32, 32, 5), 32, 32);
    t.paint.brush.shape = TextureSettings { kind: TextureKind::Image, ..forma };
    t.paint.brush.texture = TextureSettings { kind: TextureKind::Image, ..grao };
    let k = &mut t.paint.wetpaint.knobs;
    k.set(Knob::PigmentPerDab, 1200.0);
    k.set(Knob::Pickup, 0.02);
    k.set(Knob::EdgeDarkening, 90.0);
    k.set(Knob::Gravity, 0.01);
    k.set(Knob::BristleSize, 1.5);
    k.set(Knob::BristleCount, 1500.0);
    k.erase = 0.6;
    k.water = 0.7;
    t
}

fn pinta(t: &mut PainterTool) -> Vec<u8> {
    t.set_brush_color_srgb8([23, 101, 179]);
    let p = |x: f32, pr: f32, phase| CanvasPointer {
        pos: [x, 60.0 + 10.0 * (x * 0.03).sin()],
        pressure: pr,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(p(20.0, 0.2, PointerPhase::Down));
    for k in 1..=66 {
        let x = 20.0 + 3.0 * k as f32;
        t.on_canvas_pointer(p(x, 0.2 + 0.8 * (k as f32 / 66.0 * std::f32::consts::PI).sin(), PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(p(218.0, 0.2, PointerPhase::Up));
    for _ in 0..60 {
        t.on_tick(1000.0 / 60.0);
    }
    t.run_full().0
}

fn pelo_ficheiro(prm: Parametros, texturas: BTreeMap<String, Vec<u8>>) -> Preset {
    let p = Preset {
        parametros: prm,
        origem: Origem::Utilizador,
        miniatura: Miniatura { largura: 1, altura: 1, rgba: vec![255; 4] },
        texturas,
    };
    ph2d_brush_presets::ler(&ph2d_brush_presets::escrever(&p).unwrap()).unwrap()
}

#[test]
fn o_pincel_guardado_pinta_o_mesmo_traco() {
    let a = ferramenta_mexida();
    let base = Parametros::novo(MEIO_DO_PRESET, "Teste", "Mexido");
    let (prm, texturas) = a.wet_preset_capturar(&base);
    assert_eq!(texturas.len(), 2, "as duas máscaras viajam");
    let lido = pelo_ficheiro(prm, texturas);

    let mut b = ferramenta();
    b.wet_preset_aplicar(&lido);
    let mut a = a;
    let px_a = pinta(&mut a);
    let px_b = pinta(&mut b);
    let diferentes = px_a.chunks(4).zip(px_b.chunks(4)).filter(|(x, y)| x != y).count();
    assert_eq!(diferentes, 0, "{diferentes} px diferem: o preset não leva tudo o que o traço lê, ou um campo de fora o muda");

    // Controlo: a mesma ferramenta nova SEM o preset pinta outro traço (o gesto é sensível ao pincel).
    let px_c = pinta(&mut ferramenta());
    assert_ne!(px_a, px_c, "controlo: sem o preset o traço tem de mudar");
}

#[test]
fn capturar_o_aplicado_devolve_o_mesmo_preset() {
    let a = ferramenta_mexida();
    let base = Parametros::novo(MEIO_DO_PRESET, "Teste", "Mexido");
    let (prm, texturas) = a.wet_preset_capturar(&base);
    let lido = pelo_ficheiro(prm.clone(), texturas.clone());
    let mut b = ferramenta();
    b.wet_preset_aplicar(&lido);
    let (de_novo, tex2) = b.wet_preset_capturar(&base);
    assert_eq!(de_novo, prm);
    assert_eq!(tex2, texturas);
}

#[test]
fn campos_de_ondas_futuras_sobrevivem_a_captura() {
    let mut base = Parametros::novo(MEIO_DO_PRESET, "Teste", "Futuro");
    base.extra.insert("dual_brush".into(), serde_json::json!({"count": 3}));
    base.brush.insert("bristle_thin".into(), serde_json::json!(0.52));
    base.engine.insert("oiliness".into(), 0.8);
    base.volumes.pressure = Some(0.7);
    let (p, _) = ferramenta().wet_preset_capturar(&base);
    assert_eq!(p.extra["dual_brush"], serde_json::json!({"count": 3}));
    assert_eq!(p.brush["bristle_thin"], serde_json::json!(0.52));
    assert_eq!(p.engine["oiliness"], 0.8);
    assert_eq!(p.volumes.pressure, Some(0.7));
}

#[test]
fn os_volumes_sao_o_que_o_painel_mostra() {
    let mut t = ferramenta();
    t.wet_preset_aplicar_volumes(&Volumes {
        size_px: Some(40.0),
        opacity: Some(0.5),
        water: Some(0.25),
        ..Volumes::default()
    });
    assert_eq!(t.paint.brush.radius_px, 20.0, "Size px é o DIÂMETRO");
    assert_eq!(t.paint.brush.strength, 0.5);
    assert_eq!(t.paint.wetpaint.knobs.water, 0.25);
    let v = t.wet_preset_volumes();
    assert_eq!((v.size_px, v.opacity, v.water), (Some(40.0), Some(0.5), Some(0.25)));
}

