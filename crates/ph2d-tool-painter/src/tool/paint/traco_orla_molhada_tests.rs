//! ⭐ **A orla molhada é o *Wet Edges* do Procreate** (bug #58, 4.ª volta — o dono 24/10: *«wet edges está deixando
//! transparente o centro do traço»*; o Brush Studio → Rendering: *«Wet Edges — Soften and blur the edges of your
//! brushstrokes to mimic pigment bleeding into paper»*). A lei de antes (`a·(1 − m(1 − e))`, o Wet Edges do
//! Photoshop) clareava o miolo até `1 − molhada`; agora o miolo fica intacto e a borda borra-se (`traco_borrao`).

use super::traco_orla_bordas_tests::{H, W, glaze_round, padrao};
use super::traco_orla_lei_tests::{caminho, cp, distancias, pinta, traco, z};
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase, RasterEditTool};

/// ⭐ **(a) O wet não clareia o miolo**: no Z do Glaze Round de fábrica e do pincel por omissão, todo texel a mais
/// de `W + √2` do contorno visível tem com Wet 1 a cor que tem com Wet 0 (`≤ 1/255`).
#[test]
fn o_wet_nao_clareia_o_miolo() {
    type Monta = fn(f32) -> PainterTool;
    let casos: [(&str, Monta); 2] = [
        ("Glaze Round", |m| glaze_round(|t| t.set_traco_molhada(m))),
        // O tecto 1 arma o plano sem tapar nada: Wet 0 e Wet 1 pintam o MESMO plano.
        ("omissão", |m| {
            padrao(|t| {
                t.set_traco_tecto(1.0);
                t.set_traco_molhada(m);
            })
        }),
    ];
    for (nome, mk) in casos {
        let (mut seco, mut molhado) = (mk(0.0), mk(1.0));
        let largura = molhado.paint.brush.traco.largura;
        let a = traco(&mut seco, &z());
        let b = traco(&mut molhado, &z());
        let d = distancias(&b.v, b.tau, largura.ceil() as i32 + 3);
        let fundo_lim = largura + std::f32::consts::SQRT_2 + 0.01;
        let (mut fundos, mut pior, mut claros) = (0usize, 0u8, 0usize);
        for i in (0..W * H).filter(|&i| d[i] > fundo_lim) {
            fundos += 1;
            let dd = (0..4)
                .map(|k| a.tela[i * 4 + k].abs_diff(b.tela[i * 4 + k]))
                .max()
                .unwrap();
            pior = pior.max(dd);
            claros += usize::from(dd > 1);
        }
        assert!(fundos > 2_000, "{nome}: o traço não tem miolo ({fundos})");
        assert_eq!(
            claros, 0,
            "{nome}: o Wet 1 mudou {claros} de {fundos} texels do miolo (pior |Δ| {pior})"
        );
    }
}

/// O alfa da tela em `(x, y)` sub-píxel (bilinear).
fn alfa(px: &[u8], w: usize, x: f32, y: f32) -> f32 {
    let (x0, y0) = (x.floor() as usize, y.floor() as usize);
    let (fx, fy) = (x - x0 as f32, y - y0 as f32);
    let a = |x: usize, y: usize| f32::from(px[(y * w + x) * 4 + 3]) / 255.0;
    let top = a(x0, y0) * (1.0 - fx) + a(x0 + 1, y0) * fx;
    let bot = a(x0, y0 + 1) * (1.0 - fx) + a(x0 + 1, y0 + 1) * fx;
    top * (1.0 - fy) + bot * fy
}

/// A largura (px) da subida 10–90 % do alfa através da borda de um recto do pincel por omissão (dureza 0,9) a `graus`, na tela
/// transparente: a média de 5 cortes ao longo do traço e dos dois lados.
fn largura_da_borda(molhada: f32, graus: f32) -> f32 {
    const L: usize = 320;
    let mut t = PainterTool::default();
    t.set_source(vec![0u8; L * L * 4], L as u32, L as u32);
    t.set_paint_media(crate::PaintMedia::Digital);
    t.set_brush_size_px(40.0);
    t.set_brush_color_srgb8([220, 30, 30]);
    t.set_brush_strength(0.6);
    // A ponta dura: a borda de Wet 0 é a do antialias, e a suavidade que se mede é a da orla.
    t.paint.brush.hardness = 0.9;
    t.set_traco_tecto(1.0);
    t.set_traco_molhada(molhada);
    let (s, c) = graus.to_radians().sin_cos();
    let m = L as f32 / 2.0;
    let pts = caminho(&[
        [m - 120.0 * c, m - 120.0 * s],
        [m + 120.0 * c, m + 120.0 * s],
    ]);
    pinta(&mut t, &pts, |_| 4);
    t.on_canvas_pointer(cp(*pts.last().unwrap(), PointerPhase::Up));
    let px = t.canvas_rgba.to_vec();
    let mut soma = 0.0;
    let mut n = 0;
    for ao_longo in [-40.0f32, -20.0, 0.0, 20.0, 40.0] {
        for lado in [-1.0f32, 1.0] {
            let (ox, oy) = (m + ao_longo * c, m + ao_longo * s);
            let (nx, ny) = (-s * lado, c * lado);
            let f = |d: f32| alfa(&px, L, ox + d * nx, oy + d * ny);
            let topo = f(0.0);
            let cruza = |nivel: f32| {
                (0..1600)
                    .map(|k| k as f32 * 0.05)
                    .find(|&d| f(d) < nivel * topo)
                    .expect("a borda")
            };
            soma += cruza(0.1) - cruza(0.9);
            n += 1;
        }
    }
    soma / n as f32
}

/// ⭐ **(b) O wet suaviza a borda**: a subida 10–90 % do alfa através da borda cresce com o Wet (0 · 0,3 · 0,6 · 1)
/// e é a mesma a 0°, 17° e 45° (±0,5 px).
#[test]
fn o_wet_suaviza_a_borda() {
    let mut antes = 0.0f32;
    for m in [0.0f32, 0.3, 0.6, 1.0] {
        let l: Vec<f32> = [0.0f32, 17.0, 45.0]
            .iter()
            .map(|&g| largura_da_borda(m, g))
            .collect();
        eprintln!("Wet {m}: subida 10–90 % {l:?} px");
        let (lo, hi) = l
            .iter()
            .fold((f32::MAX, 0.0f32), |(a, b), &x| (a.min(x), b.max(x)));
        assert!(hi - lo <= 0.5, "Wet {m}: a borda não é isotrópica {l:?}");
        assert!(
            lo > antes + 0.1,
            "Wet {m}: a borda não suavizou ({antes} → {l:?})"
        );
        antes = hi;
    }
}
