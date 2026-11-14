//! **Sonda (ignorada) do Bug #71: a tinta parece afundada no papel de alto relevo.** Um traço horizontal, a luz vem de
//! cima à esquerda: o papel subido clareia em cima e escurece em baixo. `S` = luz da orla de baixo − luz da orla de cima,
//! cada uma contra o papel nu do mesmo sítio, menos o mesmo `S` com o papel liso (a cor não entra). `S > 0` = afundada.
//! `PH2D_SONDA_SO=a,b` restringe; `PH2D_SONDA_MEIO=WetPaint|Watercolor`; `PH2D_SONDA_FOTOS=dir` grava os PPM.

use super::offthread::otica::forca_no_teste;
use super::sonda_bordas_longe::{H, W, ecra, grava_ppm, tiques, traco};
use crate::tool::PainterTool;
use ph2d_brush_presets::{Biblioteca, Preset};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

const COR: [u8; 3] = [235, 215, 40];

fn lum(p: &[u8]) -> f64 {
    let a = f64::from(p[3]) / 255.0;
    let c = |k: usize| f64::from(p[k]) * a + 255.0 * (1.0 - a);
    0.2126 * c(0) + 0.7152 * c(1) + 0.0722 * c(2)
}

fn tela(b: Option<&Preset>, relevo: f32, ajuste: &dyn Fn(&mut PainterTool)) -> Vec<u8> {
    forca_no_teste(Some(false));
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.set_brush_paper_kind(ph2d_painter_brush::TextureKind::PaperCold.to_u8());
    t.set_substrate_depth(relevo);
    if let Some(b) = b {
        t.preset_aplicar(b).expect("preset");
        t.set_brush_paper_kind(ph2d_painter_brush::TextureKind::PaperCold.to_u8());
        t.set_substrate_depth(relevo);
        ajuste(&mut t);
        t.sonda_raio(40.0);
        t.set_brush_color_srgb8(COR);
        traco(&mut t, 70.0, false);
    }
    tiques(&mut t, 300, false);
    forca_no_teste(None);
    ecra(&mut t)
}

/// A luz de baixo menos a de cima, nas orlas do traço (5 px em volta da 1.ª linha com tinta).
fn s_da_orla(com: &[u8], nu: &[u8]) -> Option<(f64, f64)> {
    let at = |v: &[u8], x: usize, y: usize| lum(&v[(y * W + x) * 4..(y * W + x) * 4 + 4]);
    let tem = |x: usize, y: usize| {
        let (a, b) = ((y * W + x) * 4, (y * W + x) * 4);
        (0..3).any(|k| com[a + k].abs_diff(nu[b + k]) > 24)
    };
    let (mut topo, mut base, mut n) = (0.0, 0.0, 0usize);
    for x in (140..340).step_by(1) {
        let (Some(y0), Some(y1)) = (
            (20..140).find(|&y| tem(x, y)),
            (20..140).rev().find(|&y| tem(x, y)),
        ) else {
            continue;
        };
        if y1 < y0 + 12 {
            continue;
        }
        let razao = |y: usize| at(com, x, y) / at(nu, x, y).max(1.0);
        for k in 0..5 {
            topo += razao(y0 + k - 2);
            base += razao(y1 + 2 - k);
            n += 1;
        }
    }
    (n > 200).then(|| (topo / n as f64, base / n as f64))
}

fn mede(b: &Preset, nu_alto: &[u8], nu_liso: &[u8], ajuste: &dyn Fn(&mut PainterTool)) -> String {
    let alto = tela(Some(b), 1.0, ajuste);
    let liso = tela(Some(b), 0.0, ajuste);
    match (s_da_orla(&alto, nu_alto), s_da_orla(&liso, nu_liso)) {
        (Some((ta, ba)), Some((tl, bl))) => format!(
            "{:+.3}\tcima {ta:.3} baixo {ba:.3}\tliso cima {tl:.3} baixo {bl:.3}",
            (ba - ta) - (bl - tl)
        ),
        _ => "sem orla medível".to_owned(),
    }
}

#[test]
#[ignore = "sonda: corre à mão"]
fn sonda_relevo_negativo() {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let lib = Biblioteca::carregar(&raiz, None);
    let so = std::env::var("PH2D_SONDA_SO").ok();
    let meio = std::env::var("PH2D_SONDA_MEIO").ok();
    let fotos = std::env::var("PH2D_SONDA_FOTOS").ok();
    let bs: Vec<&Preset> = lib
        .entradas
        .iter()
        .map(|e| &e.preset)
        .filter(|p| matches!(p.parametros.media.as_str(), "WetPaint" | "Watercolor"))
        .filter(|p| meio.as_ref().is_none_or(|m| *m == p.parametros.media))
        .filter(|p| {
            so.as_ref()
                .is_none_or(|s| s.split(',').any(|n| n == p.parametros.name))
        })
        .collect();
    let (nu_alto, nu_liso) = (tela(None, 1.0, &|_| {}), tela(None, 0.0, &|_| {}));
    let prox = AtomicUsize::new(0);
    let saida = Mutex::new(Vec::new());
    std::thread::scope(|sc| {
        for _ in 0..8 {
            sc.spawn(|| {
                while let Some(b) = bs.get(prox.fetch_add(1, Ordering::SeqCst)) {
                    let linha = mede(b, &nu_alto, &nu_liso, &|_| {});
                    if let Some(d) = &fotos {
                        grava_ppm(d, &b.parametros.name, &tela(Some(b), 1.0, &|_| {}));
                    }
                    saida.lock().unwrap().push(format!(
                        "{linha}\t{}\t{}",
                        b.parametros.media, b.parametros.name
                    ));
                }
            });
        }
    });
    let mut v = saida.into_inner().unwrap();
    v.sort();
    for l in v {
        println!("RN\t{l}");
    }
}

/// O controlo de amassar o papel (dono 28/10) tem de continuar a agir: Real Oil, Relief alto, Paper Press 0 × 1.
#[test]
#[ignore = "sonda: corre à mão"]
fn sonda_paper_press_continua_a_agir() {
    use ph2d_wet_paint::tuning::Knob;
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let lib = Biblioteca::carregar(&raiz, None);
    for n in ["Knife Stain", "Knife Oily"] {
        let b = super::sonda_bordas_longe::por_nome(&lib, n);
        let com = |p: f64| {
            tela(Some(b), 1.0, &move |t: &mut PainterTool| {
                t.set_wet_oleo_real_ligado(true);
                t.paint.wetpaint.knobs.set(Knob::PaperPress, p);
            })
        };
        let (a, c) = (com(0.0), com(1.0));
        println!(
            "PRESS\t{n}\t{:?}",
            super::sonda_bordas_longe::dif(&a, &c, 0, H)
        );
    }
}

/// Bug #71 (dono 05/11: «melhorou quase 100%»): a tinta não fica afundada no papel de alto relevo. Com o degrau, +0,080.
#[test]
fn a_tinta_nao_fica_afundada_no_papel_de_alto_relevo() {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let lib = Biblioteca::carregar(&raiz, None);
    let b = super::sonda_bordas_longe::por_nome(&lib, "Wet Liner");
    let alto = s_da_orla(&tela(Some(b), 1.0, &|_| {}), &tela(None, 1.0, &|_| {})).expect("orla");
    let liso = s_da_orla(&tela(Some(b), 0.0, &|_| {}), &tela(None, 0.0, &|_| {})).expect("orla");
    let s = (alto.1 - alto.0) - (liso.1 - liso.0);
    assert!(s.abs() < 0.03, "S = {s:+.3}");
}
