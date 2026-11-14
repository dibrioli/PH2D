//! **Furos, caroços e branco do óleo sobre óleo na cena do dono** (31/10, `smoke_31_10_oleo/dono_3..5.png`): raio 32
//! (o Size 64), N passadas em vaivém deslocadas, com a PAUSA do dono entre elas (2 s: a tinta assenta — a
//! `sonda_oleo_grumos` pausava 10 ticks e via limpo), azul e depois amarelo por cima. Mede no miolo, onde todas as
//! passadas se cobrem: furos (massa < 5 % da mediana), papel à vista (alfa < 0,9), claros (mais claros que as duas
//! tintas) e os picos da altura. Variantes no MESMO processo (`PH2D_SONDA_ROTAS`, a ladeira: 5 = o produto, 2 = sem).
//! `bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib sonda_oleo_furos -- --ignored --nocapture`
use super::sonda_oleo_grumos::{AZ, alturas, grava_png, mede, preset};
use super::*;
use crate::{PaintMedia, PainterTool};
use ph2d_brush_presets::Biblioteca;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

const W: u32 = 360;
const H: u32 = 280;
const AM: [u8; 3] = [245, 205, 25];

fn ponto(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn traco(t: &mut PainterTool, a: [f32; 2], b: [f32; 2], pausa: u32) {
    let n = ((b[0] - a[0]).hypot(b[1] - a[1]) / 3.0).ceil() as u32;
    t.on_canvas_pointer(ponto(a, PointerPhase::Down));
    for k in 1..=n {
        let s = k as f32 / n as f32;
        t.on_canvas_pointer(ponto(
            [a[0] + (b[0] - a[0]) * s, a[1] + (b[1] - a[1]) * s],
            PointerPhase::Move,
        ));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(ponto(b, PointerPhase::Up));
    for _ in 0..pausa {
        t.on_tick(1000.0 / 60.0);
    }
}

pub(super) struct Furos {
    pub furos: usize,
    pub papel: usize,
    pub claros: usize,
    pub picos: usize,
    pub p99: f64,
    pub n: usize,
}

/// O miolo (px): x 100..260, y 122..158 — dentro de todas as passadas (y 140 ± 12, raio 32).
fn no_miolo(x: usize, y: usize) -> bool {
    (100..260).contains(&x) && (122..158).contains(&y)
}

fn conta(t: &mut PainterTool, cores: &[[u8; 3]]) -> Furos {
    let (h, m, gw, _gh, r) = alturas(t);
    let mut ms: Vec<f64> = vec![];
    for (i, v) in m.iter().enumerate() {
        let (x, y) = ((i % gw) as f32 * r, (i / gw) as f32 * r);
        if no_miolo(x as usize, y as usize) {
            ms.push(*v);
        }
    }
    let mut srt = ms.clone();
    srt.sort_by(f64::total_cmp);
    let med = srt[srt.len() / 2];
    let furos = ms.iter().filter(|v| **v < 0.05 * med).count();
    t.wet_recomposite_full();
    let lum = |c: [f64; 3]| 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
    let teto = cores
        .iter()
        .map(|c| lum(c.map(f64::from)))
        .fold(0.0, f64::max)
        + 12.0;
    let (mut papel, mut claros, mut n) = (0, 0, 0);
    for y in 0..H as usize {
        for x in 0..W as usize {
            if !no_miolo(x, y) {
                continue;
            }
            let q = &t.canvas_rgba[(x + y * W as usize) * 4..][..4];
            n += 1;
            let a = f64::from(q[3]) / 255.0;
            if a < 0.9 {
                papel += 1;
            }
            let c = [0, 1, 2].map(|k| f64::from(q[k]) * a + 236.0 * (1.0 - a));
            if a >= 0.9 && lum(c) > teto {
                claros += 1;
            }
        }
    }
    let md = mede(&h, &m, gw, r);
    Furos {
        furos,
        papel,
        claros,
        picos: md.picos,
        p99: md.p99,
        n,
    }
}

/// `n` passadas de `pincel` em vaivém com `cores` em blocos (`n` cada), pausa `pausa` ticks; mede no fim de cada bloco.
pub(super) fn cena(
    b: &Biblioteca,
    pincel: &str,
    cores: &[[u8; 3]],
    n: usize,
    pausa: u32,
    grava: Option<(&std::path::Path, &str)>,
) -> Vec<Furos> {
    let mut t = PainterTool::default();
    t.set_source(vec![250u8; (W * H * 4) as usize], W, H);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    t.set_semente_dos_tracos(0);
    let p = preset(b, pincel);
    let mut out = vec![];
    for (ci, cor) in cores.iter().enumerate() {
        for k in 0..n {
            let erros = t.wet_preset_aplicar(&p);
            assert!(erros.is_empty(), "{erros:?}");
            t.set_brush_size_px(32.0);
            t.set_brush_color_srgb8(*cor);
            let y = 140.0 + [-12.0, 4.0, 12.0, -4.0][k % 4];
            let (a, z) = ([50.0, y], [310.0, y + 6.0]);
            if k % 2 == 0 {
                traco(&mut t, a, z, pausa);
            } else {
                traco(&mut t, z, a, pausa);
            }
        }
        out.push(conta(&mut t, &cores[..=ci]));
        if let Some((dir, rot)) = grava {
            grava_png(
                &dir.join(format!("{rot}_c{ci}.png")),
                &t.canvas_rgba.to_vec(),
                W as usize,
                H as usize,
            );
        }
    }
    out
}

#[test]
#[ignore = "sonda — corre-se à mão no perfil smoke"]
fn sonda_oleo_furos() {
    let b = Biblioteca::carregar(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes"),
        None,
    );
    let dir = std::env::var("PH2D_SONDA_OUT")
        .ok()
        .map(std::path::PathBuf::from);
    if let Some(d) = &dir {
        std::fs::create_dir_all(d).expect("pasta");
    }
    let rotas: Vec<u8> = std::env::var("PH2D_SONDA_ROTAS")
        .unwrap_or_else(|_| "5,2".into())
        .split(',')
        .filter_map(|s| s.parse().ok())
        .collect();
    let pinceis = std::env::var("PH2D_SONDA_PINCEIS")
        .unwrap_or_else(|_| "Flat/Thick Impasto,Flat/Filbert".into());
    let ns: Vec<usize> = std::env::var("PH2D_SONDA_N")
        .unwrap_or_else(|_| "1,8,20".into())
        .split(',')
        .filter_map(|s| s.parse().ok())
        .collect();
    let pausa: u32 = std::env::var("PH2D_SONDA_PAUSA")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(120);
    for pincel in pinceis.split(',') {
        for &n in &ns {
            for &rota in &rotas {
                let _r = ph2d_wet_paint::solver::regua::Regua::poe(&ph2d_wet_paint::solver::LADEIRA_ROTA, rota);
                let rot = format!(
                    "{}_n{n}_r{rota}",
                    pincel.replace(['/', ' '], "_").to_lowercase()
                );
                let g = dir.as_deref().map(|d| (d, rot.as_str()));
                for (ci, f) in cena(&b, pincel, &[AZ, AM], n, pausa, g).iter().enumerate() {
                    println!(
                        "[furos] {rot:<34} {} furos {:>4} papel {:>5} claros {:>5} picos {:>4} p99 {:>6.2} /{}",
                        ["azul   ", "+amarelo"][ci],
                        f.furos,
                        f.papel,
                        f.claros,
                        f.picos,
                        f.p99,
                        f.n
                    );
                }
            }
        }
    }
}

/// ⭐ **O óleo sobre o óleo ASSENTADO não encaroça** (dono 31/10, `dono_4.png`): 8 passadas em vaivém a raio 32 com a
/// pausa do dono. Mecanismo: a ladeira da fotografia segurava a tinta grossa com o sinal ruidoso de `dfilm` e cada
/// passada cavava e empilhava (`solver::ladeira_do_oleo`). Antes da cura: Thick Impasto p99 19,7 → 106,5 (5,4×),
/// Filbert 47 picos; depois 31,1 (1,6×) e 0 — o mesmo que sem ladeira (30,0 e 0).
#[test]
fn o_oleo_sobre_o_oleo_assentado_nao_encaroca() {
    let b = Biblioteca::carregar(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes"),
        None,
    );
    let um = &cena(&b, "Flat/Thick Impasto", &[AZ], 1, 120, None)[0];
    let oito = &cena(&b, "Flat/Thick Impasto", &[AZ], 8, 120, None)[0];
    assert!(
        oito.p99 <= 2.0 * um.p99,
        "Thick Impasto: p99 da altura {:.1} → {:.1} em 8 passadas",
        um.p99,
        oito.p99
    );
    let f = &cena(&b, "Flat/Filbert", &[AZ], 8, 120, None)[0];
    assert!(f.picos <= 5, "Filbert: {} caroços em 8 passadas", f.picos);
    assert_eq!(oito.furos + f.furos, 0, "furos até ao papel");
}
