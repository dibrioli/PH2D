//! O Smudge no 1.º traço sobre uma poça SECA da Aquarela, numa camada transparente (smoke do dono
//! 23/10, `docs/Painter/smoke_2026-10-23/`). Fotos: `PH2D_SONDA_DIR=<pasta>` grava cada cena em PPM.

use super::*;
use crate::tool::paint::media::PaintMedia;

const LADO: u32 = 256;

fn tela(fundo: u8) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![fundo; (LADO * LADO * 4) as usize], LADO, LADO);
    t.set_paint_media(PaintMedia::Watercolor);
    t.set_brush_size_px(60.0);
    t.set_brush_color_srgb8([250, 225, 20]);
    t.set_brush_wet_smudge(0.0);
    t
}

fn traco(t: &mut PainterTool, pts: &[[f32; 2]]) {
    t.on_canvas_pointer(cp(pts[0], PointerPhase::Down));
    for w in pts.windows(2) {
        let (a, b) = (w[0], w[1]);
        let n = ((b[0] - a[0]).hypot(b[1] - a[1]) / 3.0).ceil().max(1.0) as usize;
        for k in 1..=n {
            let f = k as f32 / n as f32;
            t.on_canvas_pointer(cp(
                [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f],
                PointerPhase::Move,
            ));
            frame(t);
        }
    }
    t.on_canvas_pointer(cp(*pts.last().unwrap(), PointerPhase::Up));
    frame(t);
    frame(t);
}

fn espera(t: &mut PainterTool, s: f32) {
    for _ in 0..(s / 0.25).round() as usize {
        t.paint_tick(0.25);
    }
}

fn salva(t: &PainterTool, nome: &str) {
    let Ok(dir) = std::env::var("PH2D_SONDA_DIR") else {
        return;
    };
    let mut out = format!("P6\n{LADO} {LADO}\n255\n").into_bytes();
    for p in t.canvas_rgba.as_chunks::<4>().0 {
        out.extend(sobre_branco(p));
    }
    std::fs::write(format!("{dir}/{nome}.ppm"), out).unwrap();
}

fn sobre_branco(p: &[u8]) -> [u8; 3] {
    let a = f32::from(p[3]) / 255.0;
    [0, 1, 2].map(|c| (f32::from(p[c]) * a + 255.0 * (1.0 - a)).round() as u8)
}

fn lum(p: &[u8]) -> f32 {
    let q = sobre_branco(p);
    0.299 * f32::from(q[0]) + 0.587 * f32::from(q[1]) + 0.114 * f32::from(q[2])
}

/// (luminância mínima, maior salto entre vizinhos) na janela `[x0,x1)×[y0,y1)`.
fn mede(t: &PainterTool, x0: u32, x1: u32, y0: u32, y1: u32) -> (f32, f32) {
    let (mut mn, mut salto) = (f32::MAX, 0.0f32);
    let at = |x: u32, y: u32| {
        let i = ((y * LADO + x) * 4) as usize;
        lum(&t.canvas_rgba[i..i + 4])
    };
    for y in y0..y1 {
        for x in x0..x1 {
            let l = at(x, y);
            mn = mn.min(l);
            salto = salto
                .max((l - at(x + 1, y)).abs())
                .max((l - at(x, y + 1)).abs());
        }
    }
    (mn, salto)
}

fn poca(t: &mut PainterTool) {
    traco(
        t,
        &[
            [30.0, 90.0],
            [110.0, 90.0],
            [110.0, 120.0],
            [30.0, 120.0],
            [30.0, 150.0],
            [110.0, 150.0],
        ],
    );
}

/// A poça seca e o 1.º traço com o Smudge a cruzar-lhe a orla, sobre a tela de fundo `fundo`.
fn cena(fundo: u8, smudge: f32) -> PainterTool {
    let mut t = tela(fundo);
    poca(&mut t);
    espera(&mut t, 40.0);
    assert!(!t.wet_session_continues(), "CONTROLO: a poça secou");
    t.set_brush_wet_smudge(smudge);
    traco(&mut t, &[[150.0, 30.0], [150.0, 230.0]]);
    salva(&t, &format!("fundo{fundo}_smudge{smudge}"));
    t
}

/// ⭐ **O Smudge sobre a poça seca numa CAMADA TRANSPARENTE não escurece nem serrilha a orla** (smoke
/// do dono 23/10: o 1.º traço deixava uma sombra oliva e degraus onde cruzava a orla da poça). O
/// arrasto da base misturava em alfa recto com o `0,0,0,0` do vazio da camada. A régua: a mesma cena
/// sobre papel BRANCO opaco, onde aquele vazio não existe.
#[test]
fn o_smudge_sobre_a_poca_seca_numa_camada_transparente_e_o_do_papel() {
    let janela = (120, 170, 70, 170);
    let mede_ = |t: &PainterTool| mede(t, janela.0, janela.1, janela.2, janela.3);
    let (lum_sem, _) = mede_(&cena(0, 0.0));
    for smudge in [0.5f32, 1.0] {
        let branco = cena(255, smudge);
        let transp = cena(0, smudge);
        let (lum_b, salto_b) = mede_(&branco);
        let (lum_t, salto_t) = mede_(&transp);
        let dif = branco
            .canvas_rgba
            .as_chunks::<4>()
            .0
            .iter()
            .zip(transp.canvas_rgba.as_chunks::<4>().0)
            .map(|(b, t)| {
                let (b, t) = (sobre_branco(b), sobre_branco(t));
                (0..3).map(|c| b[c].abs_diff(t[c])).max().unwrap_or(0)
            })
            .max()
            .unwrap_or(0);
        eprintln!(
            "smudge {smudge}: branco lum {lum_b:.1} salto {salto_b:.1} · transparente lum {lum_t:.1} \
             salto {salto_t:.1} · sem smudge lum {lum_sem:.1} · maior diferença {dif}"
        );
        assert!(
            lum_t >= lum_sem - 1.0,
            "smudge {smudge}: a orla ESCURECEU — luminância mínima {lum_t:.1} contra {lum_sem:.1} sem Smudge"
        );
        assert!(
            salto_t <= salto_b + 1.0,
            "smudge {smudge}: a orla SERRILHOU — salto {salto_t:.1} contra {salto_b:.1} no papel branco"
        );
        assert!(
            dif <= 4,
            "smudge {smudge}: a camada transparente pinta diferente do papel branco ({dif} níveis)"
        );
    }
}
