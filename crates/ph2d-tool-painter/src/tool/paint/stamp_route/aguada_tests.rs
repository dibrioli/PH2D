//! B25.3 (DF-21, IN-5, T1-36): a aguada com Tiling publica o mapa das cópias da costura (`dab_groups`) para o
//! lote DELA; um Smear/Mask seguinte não pode lê-lo contra a sua lista (o `DabRng::enter` repetia sorteios).

use super::super::super::PainterTool;
use super::super::PaintMode;
use ph2d_editor_core::tool::{
    CanvasPaintTool as _, CanvasPointer, PointerPhase, RasterEditTool as _,
};
use ph2d_painter_brush::{BrushSpec, TextureKind, TextureMapping, TextureSettings};

const N: u32 = 96;

fn ponto(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn traco(t: &mut PainterTool, y: f32, de: f32, ate: f32) {
    t.on_canvas_pointer(ponto([de, y], PointerPhase::Down));
    let mut x = de;
    while x < ate {
        x += 3.0;
        t.on_canvas_pointer(ponto([x, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(ponto([x, y], PointerPhase::Up));
}

/// A aguada que acaba sobre a borda direita com Tiling X, e o pincel do Smear com o Grain em Random Offset.
fn depois_da_aguada() -> PainterTool {
    let mut src = vec![255u8; (N * N * 4) as usize];
    for (i, p) in src.chunks_exact_mut(4).enumerate() {
        if (i as u32 % N) / 8 % 2 == 0 {
            p.copy_from_slice(&[200, 30, 40, 255]); // listras: o Smear tem o que arrastar
        }
    }
    let mut t = PainterTool::default();
    t.set_source(src, N, N);
    t.paint.tiling = [true, false];
    t.paint.brush = BrushSpec {
        radius_px: 8.0,
        color: [0.1, 0.2, 0.85],
        watercolor: true,
        fill: 0.3,
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    traco(&mut t, 30.0, 60.0, 93.0); // o último lote atravessa a borda direita
    t.paint.paint_mode = PaintMode::Smear;
    t.paint.brush.watercolor = false;
    t.paint.brush.texture = TextureSettings {
        kind: TextureKind::Noise,
        mapping: TextureMapping::Random,
        ..t.paint.brush.texture
    };
    t
}

#[test]
fn o_mesmo_traco_da_o_mesmo_byte_depois_de_uma_aguada() {
    let esfrega = |t: &mut PainterTool| {
        traco(t, 60.0, 10.0, 80.0);
        (*t.canvas_rgba).clone()
    };
    let mut limpo = depois_da_aguada();
    limpo.paint.dab_groups.clear();
    let referencia = esfrega(&mut limpo);
    // Controlo: o byte do Smear DEPENDE do mapa (cópias a repetir o sorteio mudam-no) — senão a régua é vácuo.
    let mut repetido = depois_da_aguada();
    repetido.paint.dab_groups = vec![0; 4096];
    assert_ne!(
        esfrega(&mut repetido),
        referencia,
        "o Smear desta fixtura não lê o mapa das cópias: a régua passaria por vácuo"
    );
    let mut t = depois_da_aguada();
    let n = esfrega(&mut t)
        .iter()
        .zip(&referencia)
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(
        n, 0,
        "o Smear depois de uma aguada com Tiling difere em {n} bytes do mesmo Smear sem o mapa da aguada"
    );
}
