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
use ph2d_painter_brush::height_modes::{DepthSource, DrawTo};
use ph2d_painter_brush::taper::Taper;
use ph2d_painter_brush::{
    BrushBlend, BrushSpec, Falloff, FalloffCurve, FalloffPoint, FootprintCurve, HandleType,
    JitterUnit, TextureMapping, TextureSettings,
};
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
    assert_eq!(
        dentro.len(),
        CAMPOS_DO_PINCEL.len(),
        "repetido em CAMPOS_DO_PINCEL"
    );
    assert_eq!(fora.len(), CAMPOS_FORA.len(), "repetido em CAMPOS_FORA");
    assert!(
        dentro.is_disjoint(&fora),
        "um campo nas duas listas: {:?}",
        dentro.intersection(&fora)
    );
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
        lei: ph2d_painter_brush::GrainLaw {
            mode: ph2d_painter_brush::GrainMode::Height,
            depth_min: 0.2,
            depth_jitter: 0.7,
            depth_pressure: true,
        },
    };
    let curva = FalloffCurve::from_points(&[
        FalloffPoint {
            id: 0,
            x: 0.0,
            y: 1.0,
            handle: HandleType::Auto,
        },
        FalloffPoint {
            id: 3,
            x: 0.45,
            y: 0.8,
            handle: HandleType::Vector,
        },
        FalloffPoint {
            id: 1,
            x: 1.0,
            y: 0.0,
            handle: HandleType::Auto,
        },
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
        dab_jitter: ph2d_painter_brush::DabJitter {
            opacity: 0.3,
            count: 0.4,
            flip_x: 0.5,
            flip_y: 0.25,
            roundness: 0.6,
            angle_per_stroke: 0.7,
            across: 1.2,
            particle: 0.5,
        },
        hose: ph2d_painter_brush::ImageHose {
            n: 1,
            mode: ph2d_painter_brush::HoseMode::Sequence,
            pesos: [5, 2, 1, 1, 1, 1, 0, 1],
        },
        input_samples: 3,
        stabilizer: 0.25,
        taper: Taper {
            start: 1.5,
            tip_start: 0.2,
            opacity: 0.4,
            end: 1.25,
            tip_end: 0.3,
            fall_off: 9.0,
            fall_off_start: 6.5,
        },
        texture: tex(3, 30),
        grain_depth: 0.6,
        shape: tex(5, 45),
        dab_flatten: 0.3,
        dab_angle_deg: 25,
        dab_quina: 0.6,
        dab_curve: FootprintCurve::from_rows([[0.05, 0.0, 0.02, 0.0, 0.0, 0.0, 0.0]; 2]),
        color_jitter_enabled: true,
        color_jitter_hue: 0.05,
        color_jitter_sat: 0.1,
        color_jitter_val: 0.08,
        color_jitter_scope: ph2d_painter_brush::ColorJitterScope::PerStroke,
        cor_mao: ph2d_painter_brush::CorDaMao {
            secondary: [0.9, 0.8, 0.1],
            secondary_jitter: 0.2,
            pressure_secondary: 0.3,
            pressure_hue: -0.1,
            pressure_sat: 0.55,
            pressure_val: 0.33,
        },
        blend: BrushBlend::Multiply,
        blend_sobre_vazio: true,
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
        dab_tilt: d.dab_tilt,
        pen_tilt: d.pen_tilt,
        watercolor: d.watercolor,
        impasto: d.impasto,
        // ── os de fora que o Wet Paint não deve ler: mexidos ──
        pixel: ph2d_painter_brush::PixelTip::Perfect,
        traco: ph2d_painter_brush::TracoComposto {
            tecto: 0.5,
            molhada: 0.3,
            queimada: 0.2,
            largura: 9.0,
        },
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
        continuous_spacing: !d.continuous_spacing,
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

pub(super) fn ferramenta() -> PainterTool {
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
    t.paint.brush.shape = TextureSettings {
        kind: TextureKind::Image,
        ..forma
    };
    t.paint.brush.texture = TextureSettings {
        kind: TextureKind::Image,
        ..grao
    };
    let k = &mut t.paint.wetpaint.knobs;
    k.set(Knob::PigmentPerDab, 1200.0);
    k.set(Knob::Pickup, 0.02);
    k.set(Knob::EdgeDarkening, 90.0);
    k.set(Knob::Gravity, 0.01);
    k.set(Knob::BristleSize, 1.5);
    k.set(Knob::BristleCount, 1500.0);
    k.erase = 0.6;
    k.water = 0.7;
    // O Tilt (W1e, F4): o pincel guardado leva a inclinação — mexida dos três lados do de fábrica.
    let w = &mut t.paint.wetpaint;
    (w.tilt_on, w.tilt_ring, w.tilt_spoke) = (true, 7, 2);
    // As opções da W1g viajam no preset.
    t.set_wet_agua_viva(true);
    t.set_wet_borda_que_pesa(0.7);
    t
}

/// **Um preset de ANTES das opções da W1g não mexe nelas** (a regra do Tilt). Mutação que sangra:
/// aplicar o neutro quando o ficheiro não as traz.
#[test]
fn um_preset_sem_as_opcoes_da_w1g_nao_mexe_nelas() {
    let mut p = ferramenta_mexida()
        .wet_preset_capturar(&Parametros::novo(MEIO_DO_PRESET, "T", "Antigo"))
        .0;
    assert_eq!(
        (p.live_water, p.heavy_edge),
        (Some(true), Some(0.7)),
        "a captura leva as opções"
    );
    (p.live_water, p.heavy_edge) = (None, None);
    let lido = pelo_ficheiro(p, BTreeMap::new());
    assert_eq!(
        (lido.parametros.live_water, lido.parametros.heavy_edge),
        (None, None)
    );
    let mut t = ferramenta();
    t.set_wet_agua_viva(true);
    t.set_wet_borda_que_pesa(0.3);
    let _ = t.wet_preset_aplicar(&lido);
    assert!(t.wet_agua_viva(), "um preset sem a Live Water desligou-a");
    assert_eq!(
        t.wet_borda_que_pesa(),
        0.3,
        "um preset sem a Heavy Edge mexeu nela"
    );
}

/// **Um preset de ANTES do Tilt não mexe na tela** (W1e, F4: o campo é opcional, o formato não sobe de
/// versão). Mutação que sangra: aplicar um Tilt de fábrica quando o ficheiro não o traz.
#[test]
fn um_preset_sem_tilt_nao_mexe_na_inclinacao() {
    let mut p = ferramenta_mexida()
        .wet_preset_capturar(&Parametros::novo(MEIO_DO_PRESET, "T", "Antigo"))
        .0;
    p.tilt = None;
    let lido = pelo_ficheiro(p, BTreeMap::new());
    assert_eq!(
        lido.parametros.tilt, None,
        "o ficheiro sem tilt lê-se sem tilt"
    );
    let mut t = ferramenta();
    let w = &mut t.paint.wetpaint;
    (w.tilt_on, w.tilt_ring, w.tilt_spoke) = (false, 2, 5);
    let _ = t.wet_preset_aplicar(&lido);
    let w = &t.paint.wetpaint;
    assert_eq!(
        (w.tilt_on, w.tilt_ring, w.tilt_spoke),
        (false, 2, 5),
        "um preset sem tilt mexeu na inclinação"
    );
}

pub(super) fn pinta(t: &mut PainterTool) -> Vec<u8> {
    risca(t, [23, 101, 179]);
    t.run_full().0
}

/// O traço de [`pinta`] com a cor `cor`, sem tirar a tela da ferramenta (lê-se em `canvas_rgba`).
fn risca(t: &mut PainterTool, cor: [u8; 3]) {
    t.set_brush_color_srgb8(cor);
    let p = |x: f32, pr: f32, phase| CanvasPointer {
        pos: [x, 60.0 + 10.0 * (x * 0.03).sin()],
        pressure: pr,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(p(20.0, 0.2, PointerPhase::Down));
    for k in 1..=66 {
        let x = 20.0 + 3.0 * k as f32;
        t.on_canvas_pointer(p(
            x,
            0.2 + 0.8 * (k as f32 / 66.0 * std::f32::consts::PI).sin(),
            PointerPhase::Move,
        ));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(p(218.0, 0.2, PointerPhase::Up));
    for _ in 0..60 {
        t.on_tick(1000.0 / 60.0);
    }
}

pub(super) fn pelo_ficheiro(prm: Parametros, texturas: BTreeMap<String, Vec<u8>>) -> Preset {
    let p = Preset {
        parametros: prm,
        origem: Origem::Utilizador,
        miniatura: Miniatura {
            largura: 1,
            altura: 1,
            rgba: vec![255; 4],
        },
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
    let diferentes = px_a
        .chunks(4)
        .zip(px_b.chunks(4))
        .filter(|(x, y)| x != y)
        .count();
    assert_eq!(
        diferentes, 0,
        "{diferentes} px diferem: o preset não leva tudo o que o traço lê, ou um campo de fora o muda"
    );

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

/// ⭐ A macro leva o pincel INTEIRO (W6c): cada campo do `BrushSpec` vai ao JSON e volta igual. O
/// controlo: a fixtura mexe em TODOS (o JSON de cada um difere do de fábrica), senão uma conversão em
/// falta passava calada pelo valor de `base`.
#[test]
fn o_pincel_inteiro_vai_e_volta_pelo_json_da_macro() {
    use ph2d_painter_brush::line_kind::LineKind;
    use ph2d_painter_brush::{MirrorAxis, StrokeMethod};
    let d = BrushSpec::default();
    let mut b = pincel_mexido();
    b.color = [0.1, 0.5, 0.9];
    b.paper = TextureSettings {
        angle_deg: 33,
        ..b.texture
    };
    b.symmetry.enabled = !d.symmetry.enabled;
    b.symmetry.circular = !d.symmetry.circular;
    b.symmetry.axis = MirrorAxis::from_u8(1);
    b.symmetry.radial_segments = d.symmetry.radial_segments + 3;
    b.symmetry.center = [11.0, 7.0];
    b.symmetry.custom_dir = [0.6, 0.8];
    b.stroke_method = (0..StrokeMethod::COUNT)
        .map(StrokeMethod::from_u8)
        .find(|m| *m != d.stroke_method)
        .unwrap();
    b.line_kind = (0..6)
        .map(LineKind::from_wire)
        .find(|k| *k != d.line_kind)
        .unwrap();
    b.grid_cell_px = [d.grid_cell_px[0] + 3.0, d.grid_cell_px[1] + 5.0];
    b.grid_offset_px = [d.grid_offset_px[0] + 1.0, d.grid_offset_px[1] + 2.0];
    b.grid_fit = d.grid_fit + 0.25;
    b.edge_to_edge = !d.edge_to_edge;
    b.style_solid = !d.style_solid;
    b.sketchy_reach = d.sketchy_reach + 0.25;
    b.sketchy_density = d.sketchy_density + 0.25;
    b.thread_width_px = d.thread_width_px + 0.25;
    b.thread_opacity = d.thread_opacity + 0.25;
    b.sketchy_magnetify = !d.sketchy_magnetify;
    b.wire_history = d.wire_history + 0.25;
    b.wire_connection_line = !d.wire_connection_line;
    b.ribbon_weight = d.ribbon_weight + 0.25;
    b.ribbon_friction = d.ribbon_friction + 0.25;
    b.ribbon_gravity = d.ribbon_gravity + 0.25;
    b.ribbon_rungs = d.ribbon_rungs + 0.25;
    b.rough_amount = d.rough_amount + 0.25;
    b.rough_bowing = d.rough_bowing + 0.25;
    b.rough_passes = d.rough_passes + 1;
    b.dash_ratio = d.dash_ratio + 0.25;
    b.dash_samples = d.dash_samples + 1;
    b.needs_heading = !d.needs_heading;
    b.pen_tilt = ph2d_painter_brush::PenTilt {
        rotation: ph2d_painter_brush::DabRotation::PenTilt,
        start_deg: 30.0,
        full_deg: 50.0,
        width: 1.25,
        length: 3.25,
        smooth: 0.5,
        lag_px: 170.0,
        opacity: 0.25,
    };
    b.watercolor = !d.watercolor;
    b.impasto = !d.impasto;

    let campos = campos_da_macro();
    let fabrica = json::pincel_para_json(&d, &campos);
    let m = json::pincel_para_json(&b, &campos);
    // `get`: a ponta de píxel e a de N imagens (L18/L19) só se gravam fora do neutro — na fábrica faltam.
    let parados: Vec<&&str> = campos.iter().filter(|c| fabrica.get(**c) == m.get(**c)).collect();
    assert!(parados.is_empty(), "a fixtura não mexe em {parados:?}");
    assert_eq!(json::pincel_de_json(&m, &campos, d), b);
}

#[test]
fn campos_de_ondas_futuras_sobrevivem_a_captura() {
    let mut base = Parametros::novo(MEIO_DO_PRESET, "Teste", "Futuro");
    base.extra
        .insert("dual_brush".into(), serde_json::json!({"count": 3}));
    base.brush
        .insert("bristle_thin".into(), serde_json::json!(0.52));
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
    assert_eq!(
        t.paint.wetpaint.opacidade, 0.5,
        "a Opacity é do Wet Paint, não a força partilhada"
    );
    assert_eq!(t.paint.wetpaint.knobs.water, 0.25);
    let v = t.wet_preset_volumes();
    assert_eq!(
        (v.size_px, v.opacity, v.water),
        (Some(40.0), Some(0.5), Some(0.25))
    );
}

/// **A Opacity do Wet Paint é a QUANTIDADE de pigmento** (dono 10/10): o traço a 0,26 é ~um quarto do escuro
/// do traço cheio (a mediana do escuro nos píxeis pintados; Rebelle: a Opacity clareia o traço). Mutação que
/// sangra: a porta do dab ignorar o `pigmento` (os dois traços iguais).
#[test]
fn a_opacity_do_wet_paint_e_a_quantidade_de_pigmento() {
    let escuro = |op: f32| {
        let mut t = ferramenta();
        t.paint.wetpaint.opacidade = op;
        let px = pinta(&mut t);
        let mut v: Vec<u32> = px
            .as_chunks::<4>()
            .0
            .iter()
            .map(|p| 255 - u32::from(p[0].min(p[1]).min(p[2])))
            .filter(|&e| e > 8)
            .collect();
        v.sort_unstable();
        f64::from(v[v.len() / 2])
    };
    let (cheia, clara) = (escuro(1.0), escuro(0.26));
    let razao = clara / cheia;
    assert!(
        (0.18..=0.36).contains(&razao),
        "a Opacity 0,26 deu {clara} contra {cheia} da cheia (razão {razao:.2})"
    );
}

/// O escuro do MIOLO do traço de [`pinta`] (a mediana na linha central, x 80…160) e a cor lá.
fn miolo_do_traco(px: &[u8]) -> (f64, [f64; 3]) {
    let mut v: Vec<(u32, [f64; 3])> = (80..160u32)
        .map(|x| {
            let y = (60.0 + 10.0 * (x as f32 * 0.03).sin()).round() as u32;
            let i = ((y * W + x) * 4) as usize;
            let c = [px[i], px[i + 1], px[i + 2]];
            (255 - u32::from(c[0].min(c[1]).min(c[2])), c.map(f64::from))
        })
        .collect();
    v.sort_by_key(|e| e.0);
    let m = v[v.len() / 2];
    (f64::from(m.0), m.1)
}

/// **Com a Opacity baixa a cor ACUMULA até à escolhida** (dono 10/10: «a cor nunca vai chegar à cor
/// escolhida no painel … o correto para clarear não seria usar pigment?»; escolhido por ele: como o Rebelle,
/// onde a tinta acumula até ao amarelo puro abaixo do próprio traço — `fixtures/rebelle_camadas/`). O mesmo
/// traço, Opacity 0,26, seco entre passadas (lido 1 s depois de cada uma): o miolo escurece a cada passada, passa o tecto da cor diluída
/// e fica no TOM do pincel. ⚠️ O CONTROLO está dentro: a lei recusada (a cor do pincel diluída para o papel,
/// o pigmento cheio) pára no tecto dela e reprova a mesma barra.
#[test]
fn com_a_opacity_baixa_a_cor_acumula_ate_a_escolhida() {
    const PASSADAS: usize = 12;
    let azul = [23.0, 101.0, 179.0];
    let tecto = 0.26 * (255.0 - 23.0);
    let camadas = |op: f32, cor: [u8; 3]| {
        let mut t = ferramenta();
        t.paint.wetpaint.opacidade = op;
        let mut out = Vec::new();
        for _ in 0..PASSADAS {
            risca(&mut t, cor);
            out.push(miolo_do_traco(&t.canvas_rgba));
            assert!(
                t.route_brush_wetpaint_event(&ph2d_editor_core::tool::PanelEvent::Click(
                    crate::ids::PAINTER_WETPAINT_DRYCANVAS
                ))
            );
        }
        out
    };
    let barra = |v: &[(f64, [f64; 3])]| -> Result<(), String> {
        let escuros: Vec<f64> = v.iter().map(|e| e.0).collect();
        let (ultimo, tom) = (
            escuros[PASSADAS - 1],
            ph2d_wet_paint::colorops::diferenca_de_pigmento(v[PASSADAS - 1].1, azul),
        );
        let sobe = escuros.windows(2).all(|w| w[1] >= w[0] - 1.0);
        if sobe && ultimo >= 1.5 * tecto && tom <= 0.15 {
            Ok(())
        } else {
            Err(format!(
                "escuro por passada {escuros:?} (tecto da diluída {tecto:.0}, barra {:.0}); tom {tom:.3}",
                1.5 * tecto
            ))
        }
    };
    let diluida = azul.map(|c| (255.0 - 0.26 * (255.0 - c)).round() as u8);
    let c = camadas(1.0, diluida);
    eprintln!(
        "[acumula] controlo (diluída): {:?}",
        c.iter().map(|e| e.0).collect::<Vec<_>>()
    );
    assert!(
        barra(&c).is_err(),
        "CONTROLO: a cor diluída passou a barra — a régua não discrimina"
    );
    let p = camadas(0.26, [23, 101, 179]);
    eprintln!(
        "[acumula] produto: {:?}",
        p.iter().map(|e| e.0).collect::<Vec<_>>()
    );
    if let Err(e) = barra(&p) {
        panic!("a Opacity 0,26 não acumula até à cor escolhida: {e}");
    }
}

/// O painel mostra um volume só quando o traço o LÊ. A Opacity (a força) não tem leitor no Wet Paint
/// (dono, 2026-08-12: escondida pela mesma medição): quando a W5 lhe der um, este gate fica vermelho e
/// o volume Opacity entra no painel. O Water e o Size são lidos — o controlo do gesto.
#[test]
fn a_opacidade_nao_tem_leitor_no_wet_paint() {
    let traco = |f: &dyn Fn(&mut PainterTool)| {
        let mut t = ferramenta();
        f(&mut t);
        pinta(&mut t)
    };
    let fraca = traco(&|t| t.set_brush_strength(0.25));
    let forte = traco(&|t| t.set_brush_strength(0.95));
    assert_eq!(
        fraca, forte,
        "a força ganhou leitor no Wet Paint: o volume Opacity entra no painel"
    );
    let seca = traco(&|t| t.paint.wetpaint.knobs.water = 0.2);
    assert_ne!(seca, forte, "controlo: o Water é lido");
    let grossa = traco(&|t| t.set_brush_size_px(20.0));
    assert_ne!(grossa, forte, "controlo: o Size é lido");
}
