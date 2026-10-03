//! **O smoke do dono do ADR-0177** — o MESMO documento do Painter composto pelo compositor de
//! produção, para olhar antes × depois da lei «as camadas juntam-se em tons de ecrã». O script
//! `docs/Painter/ferramentas/smoke_camadas_antes_e_depois.sh` corre-o aqui (a lei nova) e num
//! worktree temporário no merge-base (a lei velha, em luz) e grava os PNG.
//!
//! A pilha (de baixo para cima): um fundo opaco (creme → céu, uma faixa escura em baixo) que deixa
//! o quarto direito TRANSPARENTE; três manchas translúcidas de bordo suave (Normal); uma sombra em
//! Multiplicar a 70 %; um Desfoque Gaussiano de raio 3 e umas Curvas em S por cima de tudo.
//!
//! ```text
//! cargo run -p ph2d-tool-painter --example camadas_antes_e_depois -- <saida.rgba>
//! ```
//! Escreve `W·H·4` bytes RGBA straight (sRGB8), `W = 640`, `H = 400`.

use ph2d_painter_effects::BlendMode;
use ph2d_painter_effects::adjustments::{
    AdjustmentParams, ControlPoints, CurvesParams, GaussianBlurParams,
};
use ph2d_tool_painter::{LayerImage, LayerKind, LayerStack, MapPixelSource, composite};

const W: u32 = 640;
const H: u32 = 400;

fn suave(d: f32, raio: f32, borda: f32) -> f32 {
    ((raio - d) / borda).clamp(0.0, 1.0)
}

fn fundo() -> Vec<u8> {
    let mut px = Vec::with_capacity((W * H * 4) as usize);
    for y in 0..H {
        for x in 0..W {
            let t = x as f32 / (W - 1) as f32;
            let lerp = |a: f32, b: f32| (a + (b - a) * t).round() as u8;
            let rgb = if y > H * 7 / 10 {
                [30, 40, 70]
            } else {
                [lerp(240.0, 120.0), lerp(228.0, 170.0), lerp(200.0, 220.0)]
            };
            let a = if x >= W * 3 / 4 { 0 } else { 255 };
            px.extend_from_slice(&[rgb[0], rgb[1], rgb[2], a]);
        }
    }
    px
}

/// Três discos translúcidos (alfa até 0,6, bordo suave de 30 px), juntos dentro da camada pelo
/// `over` straight em tons de ecrã — o que o pincel faria.
fn manchas() -> Vec<u8> {
    const DISCOS: [([f32; 2], [f32; 3]); 3] = [
        ([230.0, 170.0], [220.0, 40.0, 40.0]),
        ([330.0, 240.0], [40.0, 180.0, 70.0]),
        ([430.0, 170.0], [40.0, 80.0, 220.0]),
    ];
    let mut px = Vec::with_capacity((W * H * 4) as usize);
    for y in 0..H {
        for x in 0..W {
            let (mut c, mut a) = ([0.0f32; 3], 0.0f32);
            for (centro, cor) in DISCOS {
                let d = ((x as f32 - centro[0]).powi(2) + (y as f32 - centro[1]).powi(2)).sqrt();
                let s = 0.6 * suave(d, 120.0, 30.0);
                let ao = s + a * (1.0 - s);
                if ao > 0.0 {
                    for k in 0..3 {
                        c[k] = (cor[k] / 255.0 * s + c[k] * a * (1.0 - s)) / ao;
                    }
                }
                a = ao;
            }
            let b = |v: f32| (v * 255.0).round() as u8;
            px.extend_from_slice(&[b(c[0]), b(c[1]), b(c[2]), b(a)]);
        }
    }
    px
}

fn sombra() -> Vec<u8> {
    let mut px = Vec::with_capacity((W * H * 4) as usize);
    for y in 0..H {
        for x in 0..W {
            let d = (((x as f32 - 330.0) / 1.8).powi(2) + (y as f32 - 320.0).powi(2)).sqrt();
            let a = (suave(d, 70.0, 40.0) * 255.0).round() as u8;
            px.extend_from_slice(&[90, 90, 110, a]);
        }
    }
    px
}

fn main() {
    let saida = std::env::args()
        .nth(1)
        .expect("uso: camadas_antes_e_depois <saida.rgba>");
    let mut s = LayerStack::new();
    let mut src = MapPixelSource::default();
    let mut camada = |s: &mut LayerStack, nome: &str, px: Vec<u8>, modo: BlendMode, op: f32| {
        let id = s.add_raster(nome, W, H).expect("camada");
        let l = s.get_mut(id).expect("camada");
        l.blend_mode = modo;
        l.opacity = op;
        src.insert(
            id,
            LayerImage {
                width: W,
                height: H,
                rgba8: px,
            },
        );
    };
    camada(&mut s, "fundo", fundo(), BlendMode::Normal, 1.0);
    camada(&mut s, "manchas", manchas(), BlendMode::Normal, 1.0);
    camada(&mut s, "sombra", sombra(), BlendMode::Multiply, 0.7);
    let ajuste = |s: &mut LayerStack, p: AdjustmentParams| {
        let id = s.add_adjustment(p.kind()).expect("ajuste");
        if let Some(LayerKind::Adjustment(adj)) = s.get_mut(id).map(|l| &mut l.kind) {
            adj.params = p;
        }
    };
    ajuste(
        &mut s,
        AdjustmentParams::GaussianBlur(GaussianBlurParams { radius: 3.0 }),
    );
    ajuste(
        &mut s,
        AdjustmentParams::Curves(CurvesParams {
            points_rgb: ControlPoints {
                points: vec![[0.0, 0.0], [0.25, 0.18], [0.75, 0.85], [1.0, 1.0]],
            },
            ..Default::default()
        }),
    );
    std::fs::write(&saida, composite(&s, &src, W, H)).expect("gravar");
    eprintln!("{saida}: {W}x{H} RGBA");
}
