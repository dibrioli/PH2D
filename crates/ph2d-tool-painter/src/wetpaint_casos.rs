//! `PH2D_WETPAINT_SMOKE=4` — **os casos da lei antiga** (dono 05/11): traços reais dos presets de fábrica, no
//! papel texturizado padrão e na lei de omissão (`LeiDaTinta::Velha`), para ele decidir o ADR-0197 pelo olho.
//! Receitas: a da F0 (`sonda_varredura_oleo`) e a da prancha `branco_em_papel_de_cor.png`
//! (`sonda_fotos_do_dono`). Nada aqui muda motor nem preset.

use crate::ids::{PAINTER_WETPAINT_DILUTION, PAINTER_WETPAINT_WATER};
use crate::tool::PainterTool;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase};

/// A tela: 3 × 2 lugares de 400 × 360.
pub const LARGURA: u32 = 1200;
pub const ALTURA: u32 = 720;
const LUGAR: [f32; 2] = [400.0, 360.0];

const AZUL: [u8; 3] = [40, 90, 220];
const AMARELO: [u8; 3] = [245, 205, 25];
const BRANCO: [u8; 3] = [255, 255, 255];
const LARANJA: [u8; 3] = [240, 130, 60];

const THICK_IMPASTO: &str = "WetPainter/Oil/Flat/Thick Impasto.ph2d-brush.png";
const WET_FLAT: &str = "WetPainter/Watercolor/Fluid/Wet Flat.ph2d-brush.png";
const WET_ROUND: &str = "WetPainter/Watercolor/Fluid/Wet Round.ph2d-brush.png";
const DABBER: &str = "WetPainter/Oil/Dabber/Dabber.ph2d-brush.png";
const MARKER: &str = "WetPainter/Marker/Marker/Marker Bullet.ph2d-brush.png";

/// Um quadro de 60 Hz, em ms.
const QUADRO: f32 = 1000.0 / 60.0;

/// `(Dilution, Water)` do painel: a Dilution que `receita::migra_para_controlos` dá ao Pigment da F0 com
/// `WaterPerDab = 1` nestes pincéis, `1 − P / (P + 350 W²)`.
const P10_W1: (f64, f64) = (0.972_222, 1.0);
const P40_W1: (f64, f64) = (0.897_436, 1.0);

/// O caso `n` (1..=6) e o lugar dele, para o roteiro e a sonda: `[coluna, linha]`.
pub const CASOS: [(u8, [u8; 2]); 6] = [
    (1, [0, 0]),
    (2, [1, 0]),
    (3, [2, 0]),
    (6, [0, 1]),
    (4, [1, 1]),
    (5, [2, 1]),
];

fn preset(rel: &str) -> Option<ph2d_brush_presets::Preset> {
    let f = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/brushes")
        .join(rel);
    ph2d_brush_presets::ler(&std::fs::read(f).ok()?).ok()
}

/// O pincel de fábrica na mão, pela porta do navegador, com a tinta e o tamanho; os controlos só nos casos da F0.
fn arma(
    t: &mut PainterTool,
    rel: &str,
    cor: [u8; 3],
    raio: f32,
    controlos: Option<(f64, f64)>,
) -> bool {
    let Some(p) = preset(rel) else {
        eprintln!("PH2D_WETPAINT_SMOKE=4: o preset {rel} não foi encontrado");
        return false;
    };
    let _ = t.preset_aplicar(&p);
    if let Some((diluicao, agua)) = controlos {
        t.handle_panel_event(PanelEvent::SetValue(PAINTER_WETPAINT_DILUTION, diluicao));
        t.handle_panel_event(PanelEvent::SetValue(PAINTER_WETPAINT_WATER, agua));
    }
    t.set_brush_size_px(raio);
    t.set_brush_color_srgb8(cor);
    true
}

fn traco(t: &mut PainterTool, a: [f32; 2], z: [f32; 2], pressure: f32) {
    let p = |pos, phase| CanvasPointer {
        pos,
        pressure,
        tilt: [0.0, 0.0],
        phase,
    };
    let n = ((z[0] - a[0]).hypot(z[1] - a[1]) / 3.0).ceil() as u32;
    t.on_canvas_pointer(p(a, PointerPhase::Down));
    for k in 1..=n {
        let s = k as f32 / n as f32;
        t.on_canvas_pointer(p(
            [a[0] + (z[0] - a[0]) * s, a[1] + (z[1] - a[1]) * s],
            PointerPhase::Move,
        ));
        t.on_tick(QUADRO);
    }
    t.on_canvas_pointer(p(z, PointerPhase::Up));
}

/// `n` quadros sem ninguém a pintar, em blocos de 30 (o relógio fixo dá os mesmos passos).
fn espera(t: &mut PainterTool, n: u32) {
    for _ in 0..n / 30 {
        t.on_tick(30.0 * QUADRO);
    }
}

/// Azul assentado e amarelo por baixo, meio por cima dele: a receita da F0, com `n` passadas por cor.
fn faixas(t: &mut PainterTool, o: [f32; 2], rel: &str, ctl: (f64, f64), pressure: f32, n: usize) {
    for (cor, y0) in [(AZUL, 150.0), (AMARELO, 180.0)] {
        for k in 0..n {
            if !arma(t, rel, cor, 32.0, Some(ctl)) {
                return;
            }
            let y = o[1] + y0 + [0.0, 6.0, -6.0][k % 3];
            let (a, z) = ([o[0] + 70.0, y], [o[0] + 330.0, y + 4.0]);
            if k % 2 == 0 {
                traco(t, a, z, pressure);
            } else {
                traco(t, z, a, pressure);
            }
            espera(t, 60);
        }
        if cor == AZUL {
            espera(t, 300);
        }
    }
}

/// Base horizontal e o 2.º traço na diagonal por cima: a prancha, ×1,5.
fn cruza(t: &mut PainterTool, o: [f32; 2], base: (&str, [u8; 3], u32), pincel: (&str, [u8; 3])) {
    let (rel, cor, pausa) = base;
    if !arma(t, rel, cor, 24.0, None) {
        return;
    }
    traco(
        t,
        [o[0] + 80.0, o[1] + 180.0],
        [o[0] + 320.0, o[1] + 180.0],
        1.0,
    );
    espera(t, pausa);
    if !arma(t, pincel.0, pincel.1, 24.0, None) {
        return;
    }
    traco(
        t,
        [o[0] + 110.0, o[1] + 240.0],
        [o[0] + 290.0, o[1] + 120.0],
        1.0,
    );
    espera(t, 60);
}

/// Pinta só o caso `n`. Devolve nada: a tinta fica na ferramenta.
pub fn pinta_caso(t: &mut PainterTool, n: u8) {
    let Some(&(_, [c, l])) = CASOS.iter().find(|(k, _)| *k == n) else {
        return;
    };
    let o = [f32::from(c) * LUGAR[0], f32::from(l) * LUGAR[1]];
    match n {
        1 => faixas(t, o, THICK_IMPASTO, P10_W1, 0.6, 3),
        2 => cruza(t, o, (WET_FLAT, AZUL, 40), (MARKER, LARANJA)),
        3 => cruza(t, o, (WET_FLAT, AZUL, 40), (MARKER, BRANCO)),
        4 => cruza(t, o, (DABBER, AZUL, 120), (MARKER, LARANJA)),
        5 => cruza(t, o, (WET_ROUND, AZUL, 40), (WET_ROUND, AMARELO)),
        6 => faixas(t, o, THICK_IMPASTO, P40_W1, 1.0, 3),
        _ => {}
    }
}

/// Pinta os casos no relógio fixo (o tempo da secagem passa sem esperar) e devolve a água ao relógio real.
pub fn pinta(t: &mut PainterTool) {
    let inicio = std::time::Instant::now();
    t.set_wet_relogio_fixo(true);
    for (n, _) in CASOS {
        pinta_caso(t, n);
    }
    espera(t, 300);
    t.set_wet_relogio_fixo(false);
    println!(
        "PH2D_WETPAINT_SMOKE=4: os casos pintados em {:.1} s.\n\
  Em cima: esquerda = Thick Impasto azul e amarelo muito aguados, pressao media (o da captura) | meio = Marker \
laranja sobre Wet Flat azul | direita = Marker branco sobre Wet Flat azul.\n\
  Em baixo: esquerda = Thick Impasto azul e amarelo aguados, pressao toda | meio = Marker laranja sobre Dabber \
azul | direita = o controlo, Wet Round azul e amarelo.",
        inicio.elapsed().as_secs_f32()
    );
}

/// A sonda: cada caso numa tela só dele contra a cena inteira, no mesmo processo. Sem isto, um preset posterior
/// que mude a luz da sessão inteira passaria por defeito da lei.
#[cfg(test)]
mod tests {
    use super::*;

    fn tela() -> PainterTool {
        let mut t = PainterTool::default();
        let px = PainterTool::pixeis_do_papel_texturizado(LARGURA, ALTURA);
        t.bind_document(1, px, LARGURA, ALTURA);
        assert!(t.papel().is_some(), "o papel padrão não chegou");
        t
    }

    fn ppm(dir: &str, nome: &str, t: &mut PainterTool) {
        let (px, w, h) = (t as &mut dyn ph2d_editor_core::tool::RasterEditTool).run_full();
        let mut v = format!("P6\n{w} {h}\n255\n").into_bytes();
        for p in px.chunks_exact(4) {
            v.extend_from_slice(&p[..3]);
        }
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(format!("{dir}/{nome}.ppm"), v).unwrap();
    }

    #[test]
    #[ignore = "sonda: grava a cena e cada caso sozinho em PH2D_SONDA_DIR"]
    fn sonda_casos_da_lei_antiga() {
        let dir = std::env::var("PH2D_SONDA_DIR").expect("PH2D_SONDA_DIR");
        let mut t = tela();
        pinta(&mut t);
        ppm(&dir, "cena", &mut t);
        // A água voltou ao relógio real: um traço do artista depois da cena ainda pinta.
        let antes = (&mut t as &mut dyn ph2d_editor_core::tool::RasterEditTool)
            .run_full()
            .0;
        traco(&mut t, [20.0, 20.0], [90.0, 20.0], 1.0);
        std::thread::sleep(std::time::Duration::from_millis(500));
        t.on_tick(QUADRO);
        let depois = (&mut t as &mut dyn ph2d_editor_core::tool::RasterEditTool)
            .run_full()
            .0;
        assert_ne!(antes, depois, "o traço depois da cena não pintou");
        std::thread::scope(|s| {
            for (n, _) in CASOS {
                let dir = &dir;
                s.spawn(move || {
                    let mut t = tela();
                    t.set_wet_relogio_fixo(true);
                    pinta_caso(&mut t, n);
                    espera(&mut t, 300);
                    ppm(dir, &format!("so_{n}"), &mut t);
                });
            }
        });
    }
}
