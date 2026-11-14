//! **O óleo sobreposto «derrete»** (queixa do dono 31/10: Thick Impasto sobre Thick Impasto — buracos escuros no
//! cruzamento e o relevo do traço INTEIRO ondulado, como se levasse água). Um traço longo horizontal; pausa; 3 traços
//! cruzados por cima no meio; e o relógio continua. Mede a altura fina do Óleo Real LONGE do cruzamento (deve ficar
//! igual), as cavidades na faixa do traço e a água na grade, e grava a tela (composite com luz) em cada fase.
//! `PH2D_SONDA_OUT=<pasta> bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib sonda_oleo_derrete -- --ignored --nocapture`
use super::*;
use crate::{PaintMedia, PainterTool};
use ph2d_brush_presets::Biblioteca;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

const W: u32 = 420;
const H: u32 = 280;

fn ponto(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn traco(t: &mut PainterTool, a: [f32; 2], b: [f32; 2]) {
    let n = ((b[0] - a[0]).hypot(b[1] - a[1]) / 3.0).ceil() as u32;
    t.on_canvas_pointer(ponto(a, PointerPhase::Down));
    for k in 1..=n {
        let s = k as f32 / n as f32;
        t.on_canvas_pointer(ponto(
            [a[0] + (b[0] - a[0]) * s, a[1] + (b[1] - a[1]) * s],
            PointerPhase::Move,
        ));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(ponto(b, PointerPhase::Up));
}

fn espera(t: &mut PainterTool, n: usize) {
    for _ in 0..n {
        t.on_tick(1000.0 / 60.0);
    }
}

/// (altura fina por célula, massa por célula, água total, gw, gh, razão)
fn estado(t: &mut PainterTool) -> (Vec<f64>, Vec<f64>, f64, usize, usize, f32) {
    let o = t
        .wet_oleo_real()
        .unwrap_or_else(|| t.wet_oleo_real_valores());
    let s = t.paint.wetpaint.session.as_mut().expect("sessão");
    s.bring_home();
    let r = f32::from(s.ratio);
    let (gw, gh) = s.grid;
    let grades: Vec<(&ph2d_wet_paint::grid::Grid, bool)> = s
        .engine
        .layers
        .iter()
        .map(|l| (&l.grid, l.visible))
        .collect();
    let st = grades[0].0.s;
    let campo =
        ph2d_wet_paint::oleo_real::CampoAltura::de(&grades, &o, usize::from(s.ratio), 1, 1, gw, gh);
    let mut h = vec![0.0; gw * gh];
    let mut m = vec![0.0; gw * gh];
    let agua: f64 = grades
        .iter()
        .map(|(g, _)| g.film.iter().map(|&f| f64::from(f)).sum::<f64>())
        .sum();
    let l = longe(gw, r);
    let mut fl = [0.0f64; 3];
    for &k in &l {
        let i = (k % gw + 1) + (k / gw + 1) * st;
        for (g, _) in &grades {
            fl[0] += f64::from(g.film[i]);
            fl[1] += f64::from(g.sett[i]);
            fl[2] += f64::from(g.susp[i]);
        }
    }
    let n = l.len() as f64;
    let _ = (fl, n);
    for y in 0..gh {
        for x in 0..gw {
            h[x + y * gw] = campo.at_fina(x as isize + 1, y as isize + 1);
            let i = (x + 1) + (y + 1) * st;
            m[x + y * gw] = grades
                .iter()
                .filter(|g| g.1)
                .map(|(g, _)| f64::from(g.sett[i]) + f64::from(g.susp[i]))
                .sum();
        }
    }
    (h, m, agua, gw, gh, r)
}

/// As células (px) longe do cruzamento: x 50..130 e 300..380, y 128..152.
fn longe(gw: usize, r: f32) -> Vec<usize> {
    let c = |p: f32| (p / r) as usize;
    let mut v = vec![];
    for y in c(128.0)..=c(152.0) {
        for x in (c(50.0)..=c(130.0)).chain(c(300.0)..=c(380.0)) {
            v.push(x + y * gw);
        }
    }
    v
}

/// Cavidades na faixa do traço (x 40..380, y 122..158): célula com tinta cuja altura cai abaixo de metade da
/// mediana 7×7 à volta, com a vizinhança em pé (> 1 px).
fn cavidades(h: &[f64], m: &[f64], gw: usize, r: f32) -> usize {
    let c = |p: f32| (p / r) as usize;
    let mut n = 0;
    for y in c(122.0)..=c(158.0) {
        for x in c(40.0)..=c(380.0) {
            let i = x + y * gw;
            let mut viz: Vec<f64> = (-3isize..=3)
                .flat_map(|dy| (-3isize..=3).map(move |dx| (dx, dy)))
                .map(|(dx, dy)| h[(x as isize + dx) as usize + (y as isize + dy) as usize * gw])
                .collect();
            viz.sort_by(f64::total_cmp);
            let med = viz[viz.len() / 2];
            if m[i] > 0.0 && med > 1.0 && h[i] < 0.5 * med {
                n += 1;
            }
        }
    }
    n
}

fn grava(t: &mut PainterTool, dir: Option<&std::path::Path>, nome: &str) {
    use crate::tool::paint::tests::look_watercolor_arc::{Canvas, write_png};
    let Some(dir) = dir else { return };
    t.wet_recomposite_full();
    let (w, h) = (W as usize, H as usize);
    let px = t.canvas_rgba[..w * h * 4]
        .chunks(4)
        .map(|q| {
            let a = f32::from(q[3]) / 255.0;
            let m = |c: u8| (f32::from(c) * a + 250.0 * (1.0 - a)).round() as u8;
            [m(q[0]), m(q[1]), m(q[2])]
        })
        .collect();
    write_png(&dir.join(format!("{nome}.png")), &Canvas { w, h, px }).expect("png");
}

pub(super) struct Derrete {
    /// Média de |Δaltura| longe do cruzamento: (antes → logo depois dos cruzados, antes → fim).
    pub dh_longe: (f64, f64),
    /// Altura média longe: antes, depois, fim.
    pub h_longe: (f64, f64, f64),
    /// Cavidades: antes, depois, fim.
    pub cav: (usize, usize, usize),
    /// Água total na grade: antes, depois, fim.
    pub agua: (f64, f64, f64),
}

pub(super) fn cena(
    b: &Biblioteca,
    rel: &str,
    passos: usize,
    dir: Option<&std::path::Path>,
    rot: &str,
    cruza: bool,
) -> Derrete {
    let p = b
        .entradas
        .iter()
        .find(|e| e.relativo == rel)
        .unwrap_or_else(|| panic!("preset {rel}"))
        .preset
        .clone();
    let mut t = PainterTool::default();
    t.set_source(vec![250u8; (W * H * 4) as usize], W, H);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    t.set_semente_dos_tracos(0);
    let erros = t.wet_preset_aplicar(&p);
    assert!(erros.is_empty(), "{erros:?}");
    t.sonda_raio(32.0); // o Size 64 do painel (diâmetro) que o dono usa
    t.set_brush_color_srgb8([20, 90, 200]);
    traco(&mut t, [30.0, 140.0], [390.0, 140.0]);
    espera(&mut t, 60);
    let (h0, m0, a0, gw, _gh, r) = estado(&mut t);
    grava(&mut t, dir, &format!("{rot}_1_antes"));
    if cruza {
        traco(&mut t, [200.0, 40.0], [200.0, 240.0]);
        traco(&mut t, [240.0, 240.0], [240.0, 40.0]);
        traco(&mut t, [160.0, 60.0], [280.0, 220.0]);
        espera(&mut t, 10);
    } else {
        espera(&mut t, 10 + 3 * 67);
    }
    let (h1, m1, a1, ..) = estado(&mut t);
    grava(&mut t, dir, &format!("{rot}_2_depois"));
    espera(&mut t, passos);
    let (h2, m2, a2, ..) = estado(&mut t);
    grava(&mut t, dir, &format!("{rot}_3_fim"));
    let l = longe(gw, r);
    let media = |h: &[f64]| l.iter().map(|&i| h[i]).sum::<f64>() / l.len() as f64;
    let dh =
        |a: &[f64], b: &[f64]| l.iter().map(|&i| (a[i] - b[i]).abs()).sum::<f64>() / l.len() as f64;
    Derrete {
        dh_longe: (dh(&h0, &h1), dh(&h0, &h2)),
        h_longe: (media(&h0), media(&h1), media(&h2)),
        cav: (
            cavidades(&h0, &m0, gw, r),
            cavidades(&h1, &m1, gw, r),
            cavidades(&h2, &m2, gw, r),
        ),
        agua: (a0, a1, a2),
    }
}

pub(super) fn biblioteca() -> Biblioteca {
    Biblioteca::carregar(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes"),
        None,
    )
}

#[test]
#[ignore = "sonda — corre-se à mão no perfil smoke"]
fn sonda_oleo_derrete() {
    let b = biblioteca();
    let dir = std::env::var("PH2D_SONDA_OUT")
        .ok()
        .map(std::path::PathBuf::from);
    if let Some(d) = &dir {
        std::fs::create_dir_all(d).expect("pasta");
    }
    let passos: usize = std::env::var("PH2D_SONDA_PASSOS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(600);
    let pinceis = [
        ("thick", "WetPainter/Oil/Flat/Thick Impasto.ph2d-brush.png"),
        ("filbert", "WetPainter/Oil/Flat/Filbert.ph2d-brush.png"),
        (
            "express",
            "WetPainter/ExpressOil/Round/Express Oil.ph2d-brush.png",
        ),
    ];
    // As rotas da ladeira no MESMO processo, intercaladas por pincel (`solver::LADEIRA_ROTA`).
    let rotas: [(&str, u8, bool); 7] = [
        ("semcruz", 0, false),
        ("hoje", 1, true),
        ("semladeira", 2, true),
        ("fotointeira", 3, true),
        ("pesoagora", 4, true),
        ("cura", 5, true),
        ("fotointeira_peso", 6, true),
    ];
    for (nome, rel) in pinceis {
        if !nome.contains(&std::env::var("PH2D_SONDA_CENA").unwrap_or_default()) {
            continue;
        }
        for (tag, rota, cruza) in rotas {
            let _r = ph2d_wet_paint::solver::regua::Regua::poe(&ph2d_wet_paint::solver::LADEIRA_ROTA, rota);
            let rot = format!("{nome}_{tag}");
            let d = cena(&b, rel, passos, dir.as_deref(), &rot, cruza);
            println!(
                "[derrete] {rot:<24} |dh| longe depois {:>6.3} fim {:>6.3} · h longe {:>6.2} {:>6.2} {:>6.2} · cav {:>4} {:>4} {:>4} · água {:>10.0} {:>10.0} {:>10.0}",
                d.dh_longe.0,
                d.dh_longe.1,
                d.h_longe.0,
                d.h_longe.1,
                d.h_longe.2,
                d.cav.0,
                d.cav.1,
                d.cav.2,
                d.agua.0,
                d.agua.1,
                d.agua.2
            );
        }
    }
}

/// A LEI (queixa do dono 31/10, Thick Impasto sobre Thick Impasto): o traço LONGE do cruzamento não derrete — a ladeira
/// do óleo não move a tinta ainda molhada. Altura média longe, 600 passos depois: sem cruzar 17,35; com os cruzados na
/// ladeira de 30/10 13,52 (Δ 3,8, vermelho); pesada pela fracção assente da fotografia 17,06 (Δ 0,29).
#[test]
fn o_traco_longe_do_cruzamento_nao_derrete() {
    let b = biblioteca();
    let rel = "WetPainter/Oil/Flat/Thick Impasto.ph2d-brush.png";
    let so = cena(&b, rel, 600, None, "lei", false);
    let cr = cena(&b, rel, 600, None, "lei", true);
    let d = (cr.h_longe.2 - so.h_longe.2).abs();
    assert!(
        d <= 0.6,
        "altura longe: sem cruzar {:.2}, com cruzados {:.2} (Δ {d:.2})",
        so.h_longe.2,
        cr.h_longe.2
    );
}

/// Px pretos da tela composta (31/10): `(px de tinta opaca, px com max(R,G,B) < 70)` — a tinta azul `[20, 90, 200]`
/// numa fenda funda fica azul-escura, nunca abaixo de 70 no canal dela.
fn pretos(t: &mut PainterTool) -> (usize, usize) {
    t.wet_recomposite_full();
    let (w, h) = (W as usize, H as usize);
    let (mut n, mut p) = (0, 0);
    for q in t.canvas_rgba[..w * h * 4].chunks(4) {
        if q[3] > 240 {
            n += 1;
            p += usize::from(q[0].max(q[1]).max(q[2]) < 70);
        }
    }
    (n, p)
}

/// SONDA dos pontos pretos (31/10): o traço ÚNICO de Thick Impasto (Water do preset e Water 0,03) e o cruzamento.
/// `PH2D_SONDA_OUT=<pasta> … --lib sonda_oleo_pretos -- --ignored --nocapture`
#[test]
#[ignore = "sonda — corre-se à mão no perfil smoke"]
fn sonda_oleo_pretos() {
    let b = biblioteca();
    let dir = std::env::var("PH2D_SONDA_OUT")
        .ok()
        .map(std::path::PathBuf::from);
    if let Some(d) = &dir {
        std::fs::create_dir_all(d).expect("pasta");
    }
    let rel = "WetPainter/Oil/Flat/Thick Impasto.ph2d-brush.png";
    let p = b
        .entradas
        .iter()
        .find(|e| e.relativo == rel)
        .expect("preset")
        .preset
        .clone();
    for (rot, agua, cruza) in [
        ("unico", None, false),
        ("unico_agua003", Some(0.03), false),
        ("cruza", None, true),
    ] {
        let mut t = PainterTool::default();
        t.set_source(vec![250u8; (W * H * 4) as usize], W, H);
        t.set_paint_media(PaintMedia::WetPaint);
        t.set_wet_relogio_fixo(true);
        t.set_semente_dos_tracos(0);
        assert!(t.wet_preset_aplicar(&p).is_empty());
        if let Some(a) = agua {
            t.paint.wetpaint.knobs.water = a;
        }
        t.sonda_raio(32.0); // o Size 64 do painel (diâmetro)
        t.set_brush_color_srgb8([20, 90, 200]);
        traco(&mut t, [30.0, 140.0], [390.0, 140.0]);
        espera(&mut t, 60);
        if cruza {
            traco(&mut t, [200.0, 40.0], [200.0, 240.0]);
            traco(&mut t, [240.0, 240.0], [240.0, 40.0]);
            traco(&mut t, [160.0, 60.0], [280.0, 220.0]);
        }
        espera(&mut t, 600);
        let (n, pr) = pretos(&mut t);
        grava(&mut t, dir.as_deref(), &format!("pretos_{rot}"));
        println!(
            "[pretos] {rot:<16} tinta {n:>6} · pretos {pr:>5} ({:.2} %)",
            100.0 * pr as f64 / n.max(1) as f64
        );
    }
}
