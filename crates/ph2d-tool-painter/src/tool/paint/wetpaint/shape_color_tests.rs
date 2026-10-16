//! ⭐ **A Shape Color no Wet Paint** (prompt 41, lista 26 A7, plano 27 W6): as duas camadas (Texture
//! Color · Falloff Color) dão a cor de cada CÉLULA do depósito — e, desligadas, o traço de hoje ao byte.
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_color::{ColorRamp, RampColorMode, RampInterp, RampStop};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_painter_brush::texture::TextureKind;

const LADO: u32 = 128;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

pub(super) fn verde_vermelho() -> ColorRamp {
    ColorRamp::new(
        vec![
            RampStop::new(0.0, [0.0, 1.0, 0.0, 1.0]),
            RampStop::new(1.0, [1.0, 0.0, 0.0, 1.0]),
        ],
        RampColorMode::Rgb,
        RampInterp::Linear,
    )
}

/// A tela branca do Wet Paint com o pincel azul de raio 14 e o relógio da água fixo.
pub(super) fn tela() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (LADO * LADO * 4) as usize], LADO, LADO);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    t.set_brush_size_px(14.0);
    t.set_brush_color_srgb8([30, 60, 220]);
    t
}

/// As camadas da Shape Color com a rampa verde → vermelho; `None` = configuradas e DESLIGADAS.
pub(super) fn arma_rampa(t: &mut PainterTool, camadas: Option<(bool, bool)>) {
    t.set_shape_color_ramp(verde_vermelho());
    t.set_shape_falloff_ramp(verde_vermelho());
    t.set_shape_ramp_enabled(camadas.is_some());
    let (textura, queda) = camadas.unwrap_or((true, true));
    t.paint.shape_camadas.textura = textura;
    t.paint.shape_camadas.queda = queda;
    t.set_shape_falloff_ramp(verde_vermelho()); // re-assa as LUTs
}

/// Um traço recto de `de` a `ate` (4 px por evento, um quadro por evento) e `assenta` quadros depois.
pub(super) fn traco(t: &mut PainterTool, y: f32, de: f32, ate: f32, assenta: u32) {
    t.on_canvas_pointer(cp([de, y], PointerPhase::Down));
    t.on_tick(1000.0 / 60.0);
    let passos = ((ate - de) / 4.0).abs().round() as i32;
    let dir = (ate - de).signum();
    for k in 1..=passos {
        #[allow(clippy::cast_precision_loss)]
        t.on_canvas_pointer(cp([de + dir * 4.0 * k as f32, y], PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(cp([ate, y], PointerPhase::Up));
    for _ in 0..assenta {
        t.on_tick(1000.0 / 60.0);
    }
}

/// FNV-1a de 64 bits da tela — a impressão ao byte.
fn impressao(px: &[u8]) -> u64 {
    px.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    })
}

/// As cenas da identidade: a ponta lisa, o padrão procedural (o canal `padrao`), a cápsula (Continuous),
/// as cerdas, a rampa configurada e DESLIGADA, o modo TOM (B&W, que já agia antes) e a brocha suja (a
/// recolha da cor da tela por um 2.º traço vermelho que cruza o azul ainda molhado).
fn cena(n: usize) -> PainterTool {
    let mut t = tela();
    match n {
        0 => {}
        1 => t.set_brush_shape_kind(TextureKind::Voronoi.to_u8()),
        2 => {
            t.set_brush_shape_kind(TextureKind::Voronoi.to_u8());
            t.set_brush_spacing(1.5);
            t.toggle_brush_continuous_spacing();
        }
        3 => t.set_brush_shape_kind(TextureKind::Bristles.to_u8()),
        4 => {
            t.set_brush_shape_kind(TextureKind::Voronoi.to_u8());
            arma_rampa(&mut t, None);
        }
        5 => {
            t.set_brush_shape_kind(TextureKind::Voronoi.to_u8());
            arma_rampa(&mut t, Some((true, true)));
            t.paint.shape_color_ramp_bw = true;
        }
        6 => arma_rampa(&mut t, None),
        _ => unreachable!(),
    }
    let seed = t.paint.brush;
    t.paint.brush_by_mode.fill(seed);
    traco(&mut t, 64.0, 16.0, 112.0, if n == 6 { 4 } else { 60 });
    if n == 6 {
        t.set_brush_color_srgb8([220, 40, 30]);
        traco(&mut t, 70.0, 112.0, 16.0, 60);
    }
    t
}

/// As impressões de [`cena`], tiradas no `f6d501defb` (antes da Shape Color no Wet Paint).
const HOJE: [u64; 7] = [
    0x73f7_9de2_f437_ef58,
    0x0138_746a_c4e6_86d1,
    0x6e8a_9c8f_6636_7281,
    0x0157_c28e_024d_a8de,
    0x0138_746a_c4e6_86d1,
    0x0138_746a_c4e6_86d1,
    0xb3e4_1616_d447_4895,
];

/// ⭐ **A SECÇÃO DESLIGADA É O TRAÇO DE HOJE AO BYTE** (o portão de identidade do prompt 41, nascido
/// antes do motor mudar): cada rota do depósito — lisa, padrão, cápsula, cerdas, a rampa configurada e
/// desligada, o modo TOM e a brocha suja — pinta a mesma tela que pintava.
#[test]
fn a_secao_desligada_e_o_traco_de_hoje_ao_byte() {
    let agora: Vec<u64> = (0..HOJE.len())
        .map(|n| impressao(&cena(n).canvas_rgba))
        .collect();
    assert_eq!(agora, HOJE, "as impressões de hoje: {agora:#x?}");
}
