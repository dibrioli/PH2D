//! **A cor do Shape (rampa sem B&W) e o Grain** — prompt 36 (dono, 14/10): «ao usar Shape color parece
//! haver uma mudança no Mapping de Shape para Tiled» e «o grain muda de comportamento com cores na rampa».
//! A régua mede pelo PRODUTO (o traço pintado numa tela transparente: alfa = cobertura).

use super::*;
use ph2d_color::{ColorRamp, RampColorMode, RampInterp, RampStop};
use ph2d_painter_brush::texture::{TextureKind, TextureMapping};

const W: u32 = 192;
const H: u32 = 96;
const R: f32 = 18.0;

/// Rampa da Shape: B&W ligado (controlo) ou desligado (a rampa é a dona da cor), ou a rampa desligada.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Rampa {
    Off,
    Bw,
    Cor,
}

/// O Grain do traço: nenhum, Noise num mapeamento, ou o PAPEL a fazer de Grain no Digital (Relief
/// ligado + Tooth; [`PainterTool::papel_como_grain`] — sempre ancorado à tela).
#[derive(Clone, Copy, Debug, PartialEq)]
enum Grao {
    Sem,
    Noise(TextureMapping),
    /// Xadrez miúdo (Size 0,3): o padrão que denuncia na hora se a cor está presa à tela.
    Xadrez(TextureMapping),
    Papel,
}

fn tela(shape: Option<TextureMapping>, grao: Grao, rampa: Rampa) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![0u8; (W * H * 4) as usize], W, H);
    t.paint.brush = BrushSpec {
        radius_px: R,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.0, 0.0, 0.0],
        space_attenuation: false,
        ..Default::default()
    };
    let seed = t.paint.brush;
    t.paint.brush_by_mode.fill(seed);
    match grao {
        Grao::Sem => {}
        Grao::Noise(m) => {
            t.set_brush_texture_kind(TextureKind::Noise.to_u8());
            t.set_brush_texture_mapping(m.to_u8());
        }
        Grao::Xadrez(m) => {
            t.set_brush_texture_kind(TextureKind::Checker.to_u8());
            t.set_brush_texture_mapping(m.to_u8());
            t.paint.brush.texture.size = [miudo(m); 2];
        }
        Grao::Papel => {
            t.paint.brush.paper.kind = TextureKind::Noise;
            t.paint.brush.paper.mapping = TextureMapping::Tiled;
            t.paint.brush.paper_depth = ph2d_painter_brush::PAPER_TOOTH_MAX;
            t.paint.substrate_depth = 0.5;
            assert!(t.papel_como_grain());
        }
    }
    if let Some(m) = shape {
        t.set_brush_shape_kind(TextureKind::Checker.to_u8());
        t.paint.brush.shape.mapping = m;
        t.paint.brush.shape.size = [miudo(m); 2];
    }
    t.set_shape_color_ramp(ColorRamp::new(
        vec![
            RampStop::new(0.0, [1.0, 0.0, 0.0, 1.0]),
            RampStop::new(1.0, [0.0, 0.0, 1.0, 1.0]),
        ],
        RampColorMode::Rgb,
        RampInterp::Linear,
    ));
    t.set_shape_ramp_enabled(rampa != Rampa::Off);
    t.paint.shape_color_ramp_bw = rampa == Rampa::Bw;
    t
}

/// Um Size que põe vários ladrilhos dentro do carimbo (o Size MULTIPLICA a coordenada: maior = mais
/// miúdo): no View a unidade é o raio do dab; preso à tela ela é `TEX_TILE_BASE_PX`.
fn miudo(m: TextureMapping) -> f32 {
    if m.is_canvas_fixed() { 8.0 } else { 1.5 }
}

/// A diferença média entre os dois recortes: `0` ⇒ o padrão andou com o pincel.
fn diferenca(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| (x - y).abs()).sum::<f32>() / a.len() as f32
}

fn carimbo(t: &mut PainterTool, x: f32, y: f32) {
    t.on_canvas_pointer(cp([x, y], PointerPhase::Down));
    t.on_canvas_pointer(cp([x, y], PointerPhase::Up));
}

/// O MIOLO do carimbo à volta de `c` (raio `R − 3`: o contorno do disco ficaria a dominar a
/// correlação), pré-multiplicado (RGB·α, α), em `[0, 1]`.
fn recorte(t: &PainterTool, c: [u32; 2]) -> Vec<f32> {
    let r = R as i64;
    let mut v = Vec::new();
    for y in (c[1] as i64 - r)..=(c[1] as i64 + r) {
        for x in (c[0] as i64 - r)..=(c[0] as i64 + r) {
            let (dx, dy) = ((x - c[0] as i64) as f32, (y - c[1] as i64) as f32);
            if dx * dx + dy * dy >= (R - 3.0) * (R - 3.0) {
                continue;
            }
            let i = ((y as u32 * W + x as u32) * 4) as usize;
            let a = f32::from(t.canvas_rgba[i + 3]) / 255.0;
            for k in 0..3 {
                v.push(f32::from(t.canvas_rgba[i + k]) / 255.0 * a);
            }
            v.push(a);
        }
    }
    v
}

fn correlacao(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len() as f32;
    let (ma, mb) = (a.iter().sum::<f32>() / n, b.iter().sum::<f32>() / n);
    let (mut sab, mut saa, mut sbb) = (0.0, 0.0, 0.0);
    for (x, y) in a.iter().zip(b) {
        sab += (x - ma) * (y - mb);
        saa += (x - ma) * (x - ma);
        sbb += (y - mb) * (y - mb);
    }
    if saa * sbb <= 0.0 {
        return 1.0;
    }
    sab / (saa * sbb).sqrt()
}

/// D1: o MESMO carimbo em `x = 48` e `x = 48 + k`. `0` ⇒ o padrão anda com o pincel; alto ⇒ preso à tela
/// (ou sorteado por carimbo, no Random).
fn anda_com_o_pincel(shape: Option<TextureMapping>, grao: Grao, rampa: Rampa) -> f32 {
    [37u32, 61].map(|k| {
        let mut t = tela(shape, grao, rampa);
        carimbo(&mut t, 48.5, 48.5);
        carimbo(&mut t, 48.5 + k as f32, 48.5);
        diferenca(&recorte(&t, [48, 48]), &recorte(&t, [48 + k, 48]))
    })
    .into_iter()
    .fold(0.0, f32::max)
}

/// D2: a pegada do grão no α (a cobertura) de um carimbo — média, buracos (α < ½ do máximo dentro do
/// disco) e contraste (desvio padrão).
fn pegada(shape: Option<TextureMapping>, grao: Grao, rampa: Rampa) -> (Vec<f32>, f32, f32, f32) {
    let mut t = tela(shape, grao, rampa);
    carimbo(&mut t, 48.5, 48.5);
    let alfa: Vec<f32> = recorte(&t, [48, 48]).chunks(4).map(|p| p[3]).collect();
    let dentro = &alfa;
    let n = dentro.len() as f32;
    let media = dentro.iter().sum::<f32>() / n;
    let max = dentro.iter().copied().fold(0.0_f32, f32::max);
    let buracos = dentro.iter().filter(|a| **a < 0.5 * max).count() as f32 / n;
    let dp = (dentro.iter().map(|a| (a - media) * (a - media)).sum::<f32>() / n).sqrt();
    (alfa, media, buracos, dp)
}

#[test]
#[ignore = "régua do prompt 36 — corra por nome com --ignored --nocapture"]
fn regua_cor_do_shape_e_o_grao() {
    use TextureMapping::{Random, Tiled, ViewPlane};
    let nome = |m: Option<TextureMapping>| m.map_or("—", |m| match m {
        ViewPlane => "View",
        Tiled => "Tiled",
        Random => "Random",
        _ => "Stencil",
    });
    println!("\n## D1 — o padrão anda com o pincel? (diferença média dos dois recortes; 0 = anda)\n");
    println!("| Shape | Grain | rampa Off | B&W | Cor |\n|---|---|---|---|---|");
    for shape in [Some(ViewPlane), Some(Random), Some(Tiled), None] {
        for grao in [Grao::Sem, Grao::Xadrez(Tiled), Grao::Xadrez(ViewPlane), Grao::Papel] {
            let v: Vec<String> = [Rampa::Off, Rampa::Bw, Rampa::Cor]
                .iter()
                .map(|r| format!("{:.3}", anda_com_o_pincel(shape, grao, *r)))
                .collect();
            println!("| {} | {grao:?} | {} |", nome(shape), v.join(" | "));
        }
    }
    println!("\n## D2 — a pegada do grão no α (média · buracos · desvio · correlação com a rampa Off)\n");
    println!("| Shape | Grain | rampa | média | buracos | desvio | corr(Off) |\n|---|---|---|---|---|---|---|");
    for shape in [None, Some(ViewPlane)] {
        for grao in [Grao::Noise(Tiled), Grao::Noise(ViewPlane), Grao::Papel] {
            let (base, ..) = pegada(shape, grao, Rampa::Off);
            for r in [Rampa::Off, Rampa::Bw, Rampa::Cor] {
                let (a, m, b, d) = pegada(shape, grao, r);
                println!(
                    "| {} | {grao:?} | {r:?} | {m:.3} | {b:.3} | {d:.3} | {:.3} |",
                    nome(shape),
                    correlacao(&base, &a)
                );
            }
        }
    }
}

/// A tela sobre branco, em PPM (a prancha converte-se com `magick`).
fn ppm(t: &PainterTool, caminho: &std::path::Path) {
    let mut out = format!("P6\n{W} {H}\n255\n").into_bytes();
    for p in t.canvas_rgba.chunks(4) {
        let a = f32::from(p[3]) / 255.0;
        for k in 0..3 {
            out.push((f32::from(p[k]) * a + 255.0 * (1.0 - a)).round() as u8);
        }
    }
    std::fs::write(caminho, out).unwrap();
}

#[test]
#[ignore = "prancha do prompt 36 — PH2D_PRANCHA_DIR=<pasta> e --ignored"]
fn prancha_cor_do_shape_e_o_grao() {
    use TextureMapping::{Random, Tiled, ViewPlane};
    let dir = std::path::PathBuf::from(std::env::var("PH2D_PRANCHA_DIR").expect("PH2D_PRANCHA_DIR"));
    std::fs::create_dir_all(&dir).unwrap();
    let casos = [
        ("shapeView_sem", Some(ViewPlane), Grao::Sem),
        ("shapeTiled_sem", Some(Tiled), Grao::Sem),
        ("shapeRandom_sem", Some(Random), Grao::Sem),
        ("shapeView_noiseTiled", Some(ViewPlane), Grao::Noise(Tiled)),
        ("shapeView_xadrezTiled", Some(ViewPlane), Grao::Xadrez(Tiled)),
        ("semShape_noiseTiled", None, Grao::Noise(Tiled)),
        ("semShape_papel", None, Grao::Papel),
    ];
    for (nome, shape, grao) in casos {
        for r in [Rampa::Off, Rampa::Bw, Rampa::Cor] {
            let mut t = tela(shape, grao, r);
            // O mesmo carimbo em três sítios + um traço: o carimbo mostra se o padrão anda com o
            // pincel, o traço mostra o grão ao longo dele.
            for x in [24.5, 61.5, 98.5] {
                carimbo(&mut t, x, 24.5);
            }
            t.on_canvas_pointer(cp([20.0, 70.0], PointerPhase::Down));
            for i in 1..=40 {
                t.on_canvas_pointer(cp([20.0 + 4.0 * i as f32, 70.0], PointerPhase::Move));
            }
            t.on_canvas_pointer(cp([180.0, 70.0], PointerPhase::Up));
            ppm(&t, &dir.join(format!("{nome}_{r:?}.ppm")));
        }
    }
}

/// O α (a cobertura) de um carimbo inteiro, byte a byte.
fn alfa_da_tela(t: &PainterTool) -> Vec<u8> {
    t.canvas_rgba.chunks(4).map(|p| p[3]).collect()
}

/// **D2 — o Grain morde a cobertura com a rampa do Shape em cor** (prompt 36). A rampa do Shape escolhe a
/// COR; a cobertura tem de ser a mesmíssima de sem rampa, em cada rota: por pixel (Grain Tiled, o papel a
/// fazer de Grain) e o carimbo assado (Grain View). Vermelho antes da cura: com cor, α = 1 sem buracos.
#[test]
fn a_rampa_do_shape_em_cor_nao_desliga_o_grain() {
    use TextureMapping::{Tiled, ViewPlane};
    for shape in [None, Some(ViewPlane)] {
        for grao in [Grao::Noise(Tiled), Grao::Noise(ViewPlane), Grao::Papel] {
            let alfa = |r: Rampa| {
                let mut t = tela(shape, grao, r);
                carimbo(&mut t, 48.5, 48.5);
                alfa_da_tela(&t)
            };
            let (sem, cor) = (alfa(Rampa::Off), alfa(Rampa::Cor));
            let (_, _, buracos, desvio) = pegada(shape, grao, Rampa::Off);
            assert!(
                buracos > 0.0 || desvio > 0.01,
                "controlo: o Grain tem de morder sem rampa ({shape:?}, {grao:?})"
            );
            let difere = sem.iter().zip(&cor).filter(|(a, b)| a != b).count();
            assert_eq!(
                difere, 0,
                "com a rampa do Shape em cor a cobertura mudou em {difere} px ({shape:?}, {grao:?}) — o \
                 Grain deixou de morder"
            );
        }
    }
}

/// **D1 — a cor da rampa do Shape anda com o pincel** (prompt 36: «parece mudar o Mapping para Tiled»).
/// Shape View, Grain preso à tela (xadrez miúdo, Tiled) e um falloff macio, para a rampa variar dentro do
/// dab: o MESMO carimbo em dois sítios tem de dar a mesma cor em cada ponto relativo ao dab. Vermelho
/// antes da cura: a cor era a do Grain, presa à tela.
#[test]
fn a_cor_da_rampa_do_shape_anda_com_o_pincel() {
    let k = 48u32; // 1,5 período do xadrez preso à tela (32 px): em contrafase
    let mut t = tela(Some(TextureMapping::ViewPlane), Grao::Xadrez(TextureMapping::Tiled), Rampa::Cor);
    t.paint.brush.falloff = Falloff::Smooth;
    t.paint.brush.hardness = 0.0;
    t.paint.brush.grain_depth = 0.5; // o Grain modula sem zerar: os dois carimbos partilham texels
    carimbo(&mut t, 48.5, 48.5);
    carimbo(&mut t, 48.5 + k as f32, 48.5);
    let (a, b) = (recorte(&t, [48, 48]), recorte(&t, [48 + k, 48]));
    let matiz = |p: &[f32]| p[0] / (p[0] + p[2]).max(1e-6);
    let (mut n, mut pior, mut dalfa) = (0, 0.0_f32, 0.0_f32);
    for (pa, pb) in a.chunks(4).zip(b.chunks(4)) {
        dalfa += (pa[3] - pb[3]).abs();
        if pa[3] > 0.1 && pb[3] > 0.1 {
            n += 1;
            pior = pior.max((matiz(pa) - matiz(pb)).abs());
        }
    }
    let dalfa = dalfa / (a.len() / 4) as f32;
    assert!(dalfa > 0.02, "controlo: o Grain tem de estar preso à tela (Δα médio {dalfa:.3}; 0 se andasse)");
    assert!(n > 50, "controlo: os dois carimbos têm de partilhar texels pintados ({n})");
    let (lo, hi) = a
        .chunks(4)
        .filter(|p| p[3] > 0.1)
        .map(matiz)
        .fold((1.0_f32, 0.0_f32), |(lo, hi), m| (lo.min(m), hi.max(m)));
    assert!(hi - lo > 0.2, "controlo: a rampa tem de variar dentro do dab ({lo:.2}..{hi:.2})");
    assert!(
        pior <= 0.03,
        "a cor da rampa do Shape ficou presa à tela: o mesmo ponto do dab muda de matiz {pior:.3} entre \
         dois sítios"
    );
}
