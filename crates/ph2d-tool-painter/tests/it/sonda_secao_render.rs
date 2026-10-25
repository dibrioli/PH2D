//! **Sonda da retirada da secção Rendering do Digital** (dono 24/10): os pincéis de fábrica que armavam o plano do
//! traço (Glaze Round, Ballpoint, Marker Liner, Burn, Dodge, Color Glaze) e o Soft Round de controlo, cada um num
//! laço que volta sobre si sobre uma tela de duas metades (branco · azul-acinzentado). Escreve `<nome>.ppm` em
//! `PH2D_RENDER_OUT` e imprime o perfil do flanco (o falloff, num corte do braço recto) e o texel do cruzamento
//! contra o de uma passagem — corre-se antes e depois da retirada e compara-se.

use ph2d_brush_presets::Biblioteca;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_tool_painter::{PaintMedia, PainterTool};
use std::path::PathBuf;

const W: usize = 480;
const H: usize = 240;
const NOMES: [&str; 7] = [
    "Soft Round",
    "Glaze Round",
    "Ballpoint",
    "Marker Liner",
    "Burn",
    "Dodge",
    "Color Glaze",
];

fn tela() -> Vec<u8> {
    let mut v = vec![255u8; W * H * 4];
    for y in 0..H {
        for x in W / 2..W {
            let i = (y * W + x) * 4;
            v[i..i + 3].copy_from_slice(&[96, 120, 150]);
        }
    }
    v
}

/// O laço: um braço recto em `y = 60` de 40 a 440 e a volta que o cruza em `x = 120` e `x = 360`.
fn caminho() -> Vec<[f32; 2]> {
    let mut p = Vec::new();
    for k in 0..=100 {
        p.push([40.0 + 4.0 * k as f32, 60.0]);
    }
    for k in 1..=120 {
        let a = std::f32::consts::PI * k as f32 / 120.0;
        p.push([240.0 + 200.0 * a.cos(), 60.0 + 120.0 * a.sin()]);
    }
    for k in 1..=100 {
        p.push([40.0 + 4.0 * k as f32, 60.0]);
    }
    p
}

fn pinta(nome: &str) -> Vec<u8> {
    let raiz = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let p = Biblioteca::carregar(&raiz, None)
        .entradas
        .into_iter()
        .find(|e| e.preset.parametros.name == nome)
        .unwrap_or_else(|| panic!("{nome} de fábrica"))
        .preset;
    let mut t = PainterTool::default();
    t.set_source(tela(), W as u32, H as u32);
    t.set_paint_media(PaintMedia::Digital);
    t.set_brush_color_srgb8([30, 80, 200]);
    t.preset_aplicar(&p).expect("aplica");
    t.set_brush_size_px(40.0);
    let ponto = |pos, phase| CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    let pts = caminho();
    t.on_canvas_pointer(ponto(pts[0], PointerPhase::Down));
    for &q in &pts[1..] {
        t.on_canvas_pointer(ponto(q, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(ponto(*pts.last().unwrap(), PointerPhase::Up));
    for _ in 0..5 {
        t.on_tick(1000.0 / 60.0);
    }
    t.run_full().0
}

/// O último quadro do preview VIVO do painel (o S do pincel), como o painel o mostra.
fn preview(nome: &str) -> (Vec<u8>, u32, u32) {
    let raiz = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let p = Biblioteca::carregar(&raiz, None)
        .entradas
        .into_iter()
        .find(|e| e.preset.parametros.name == nome)
        .unwrap()
        .preset;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_media(PaintMedia::Digital);
    t.set_brush_color_srgb8([30, 80, 200]);
    t.preset_aplicar(&p).expect("aplica");
    let cena = t.wet_preview_cena();
    let mut ultimo = (Vec::new(), 0, 0);
    ph2d_tool_painter::pintar_preview(&cena, &|| false, &mut |px, w, h| {
        ultimo = (px.to_vec(), w, h);
    });
    ultimo
}

fn ppm(px: &[u8], w: u32, h: u32) -> Vec<u8> {
    let mut v = format!("P6\n{w} {h}\n255\n").into_bytes();
    for c in px.as_chunks::<4>().0 {
        v.extend_from_slice(&c[..3]);
    }
    v
}

fn luma(px: &[u8], x: usize, y: usize) -> f32 {
    let i = (y * W + x) * 4;
    0.2126 * f32::from(px[i]) + 0.7152 * f32::from(px[i + 1]) + 0.0722 * f32::from(px[i + 2])
}

#[test]
#[ignore = "sonda: PH2D_RENDER_OUT=<pasta> — a secção Rendering antes e depois"]
fn sonda_secao_render() {
    let dir = std::env::var("PH2D_RENDER_OUT").expect("PH2D_RENDER_OUT");
    std::fs::create_dir_all(&dir).unwrap();
    for nome in NOMES {
        let px = pinta(nome);
        let mut ppm = format!("P6\n{W} {H}\n255\n").into_bytes();
        for c in px.as_chunks::<4>().0 {
            ppm.extend_from_slice(&c[..3]);
        }
        std::fs::write(PathBuf::from(&dir).join(format!("{nome}.ppm")), ppm).unwrap();
        let (pv, pw, ph) = preview(nome);
        std::fs::write(
            PathBuf::from(&dir).join(format!("{nome} preview.ppm")),
            self::ppm(&pv, pw, ph),
        )
        .unwrap();
        // O flanco de UMA passagem: o fundo do arco (x = 240, no azul), de y = 180 para baixo.
        let perfil: Vec<String> = (0..26)
            .map(|d| format!("{:.0}", luma(&px, 240, 180 + d)))
            .collect();
        // Uma passagem (o arco) contra duas (o braço, ida e volta), no branco e no azul.
        let (uma_b, cruza_b) = (luma(&px, 99, 145), luma(&px, 120, 60));
        let (uma_a, cruza_a) = (luma(&px, 381, 145), luma(&px, 360, 60));
        println!(
            "[render] {nome:<12} uma/cruza branco {uma_b:.1}/{cruza_b:.1} · azul {uma_a:.1}/{cruza_a:.1} · flanco {}",
            perfil.join(" ")
        );
    }
}
