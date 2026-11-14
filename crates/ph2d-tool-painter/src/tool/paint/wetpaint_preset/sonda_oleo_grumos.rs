//! **Grumos do óleo sobre óleo** (queixa do dono 31/10: «a reação está muito exagerada e forma grumos
//! exagerados»): N = 1·3·6·10 pinceladas sobrepostas com os óleos de fábrica — o mesmo pincel e dois a alternar,
//! a mesma cor e outra, ida e volta e cruzadas — e mede por N, na faixa central, a altura fina do Óleo Real
//! (`CampoAltura::at_fina`, a que dá a normal): máx, p99, a rugosidade (desvio-padrão local 5×5, média) e os picos
//! isolados (máximo 5×5 com proeminência > 0,5 px). Grava a tela (o composite do produto, com luz) e o mapa da
//! altura em `PH2D_SONDA_OUT`.
//! `bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib sonda_oleo_grumos -- --ignored --nocapture`
use super::*;
use crate::{PaintMedia, PainterTool};
use ph2d_brush_presets::Biblioteca;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

const W: u32 = 360;
const H: u32 = 280;

fn ponto(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

pub(super) fn preset(b: &Biblioteca, rel: &str) -> ph2d_brush_presets::Preset {
    let rel = format!("WetPainter/Oil/{rel}.ph2d-brush.png");
    b.entradas
        .iter()
        .find(|e| e.relativo == rel)
        .unwrap_or_else(|| panic!("preset {rel}"))
        .preset
        .clone()
}

/// Um traço de `a` a `b`, passo 3 px, e uma pausa curta (molhado sobre molhado).
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
    for _ in 0..10 {
        t.on_tick(1000.0 / 60.0);
    }
}

pub(super) struct Medida {
    pub max: f64,
    pub p99: f64,
    pub p50: f64,
    pub rug: f64,
    pub picos: usize,
    pub massa_p99: f64,
}

fn pct(v: &mut [f64], q: f64) -> f64 {
    v.sort_by(f64::total_cmp);
    v[((v.len() - 1) as f64 * q).round() as usize]
}

/// A altura fina por célula da tela inteira (grade `gw×gh`), e a massa visível.
pub(super) fn alturas(t: &mut PainterTool) -> (Vec<f64>, Vec<f64>, usize, usize, f32) {
    let o = t.wet_oleo_real().expect("o preset liga o Real Oil");
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
    (h, m, gw, gh, r)
}

pub(super) fn mede(h: &[f64], m: &[f64], gw: usize, r: f32) -> Medida {
    // A faixa central: x 120..240, y 110..170 (px), onde todas as cenas se sobrepõem.
    let c = |p: f32| (p / r) as usize;
    let (x0, x1, y0, y1) = (c(120.0), c(240.0), c(110.0), c(170.0));
    let (mut v, mut ms, mut sd) = (vec![], vec![], vec![]);
    let mut picos = 0;
    for y in y0..=y1 {
        for x in x0..=x1 {
            let i = x + y * gw;
            v.push(h[i]);
            ms.push(m[i]);
            let viz: Vec<f64> = (-2isize..=2)
                .flat_map(|dy| (-2isize..=2).map(move |dx| (dx, dy)))
                .map(|(dx, dy)| h[(x as isize + dx) as usize + (y as isize + dy) as usize * gw])
                .collect();
            let mu = viz.iter().sum::<f64>() / viz.len() as f64;
            sd.push(
                (viz.iter().map(|a| (a - mu) * (a - mu)).sum::<f64>() / viz.len() as f64).sqrt(),
            );
            let mn = viz.iter().copied().fold(f64::INFINITY, f64::min);
            let mx = viz.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            if h[i] >= mx && h[i] - mn > 0.5 {
                picos += 1;
            }
        }
    }
    let rug = sd.iter().sum::<f64>() / sd.len() as f64;
    Medida {
        max: pct(&mut v.clone(), 1.0),
        p99: pct(&mut v.clone(), 0.99),
        p50: pct(&mut v, 0.5),
        rug,
        picos,
        massa_p99: pct(&mut ms, 0.99),
    }
}

/// Uma cena: `pinceis` alternam por traço, `cores` alternam, `cruza` alterna horizontal e vertical; a ida e volta
/// troca o sentido a cada traço. Devolve as medidas em N = 1, 3, 6, 10 e grava os PNG.
pub(super) fn cena(
    b: &Biblioteca,
    rot: &str,
    pinceis: &[&str],
    cores: &[[u8; 3]],
    cruza: bool,
    grava: Option<&std::path::Path>,
) -> Vec<(usize, Medida)> {
    let mut t = PainterTool::default();
    t.set_source(vec![250u8; (W * H * 4) as usize], W, H);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    t.set_semente_dos_tracos(0);
    let ps: Vec<_> = pinceis.iter().map(|p| preset(b, p)).collect();
    let mut out = vec![];
    for k in 0..10usize {
        let erros = t.wet_preset_aplicar(&ps[k % ps.len()]);
        assert!(erros.is_empty(), "{erros:?}");
        t.sonda_raio(32.0); // o Size 64 do painel (diâmetro) que o dono usa
        t.set_brush_color_srgb8(cores[k % cores.len()]);
        let (a, z) = if cruza && k % 2 == 1 {
            ([180.0, 50.0], [180.0, 230.0])
        } else {
            ([50.0, 140.0], [310.0, 140.0])
        };
        if k % 2 == 1 && !cruza || cruza && k % 4 >= 2 {
            traco(&mut t, z, a);
        } else {
            traco(&mut t, a, z);
        }
        let n = k + 1;
        if [1, 3, 6, 10].contains(&n) {
            let (h, m, gw, gh, r) = alturas(&mut t);
            if let Some(dir) = grava {
                t.wet_recomposite_full();
                let tela = t.canvas_rgba.to_vec();
                grava_png(
                    &dir.join(format!("{rot}_n{n:02}_tela.png")),
                    &tela,
                    W as usize,
                    H as usize,
                );
                let hmax = 12.0;
                let mapa: Vec<u8> = h
                    .iter()
                    .flat_map(|v| {
                        let g = ((v / hmax).clamp(0.0, 1.0) * 255.0) as u8;
                        [g, g, g, 255]
                    })
                    .collect();
                grava_png(
                    &dir.join(format!("{rot}_n{n:02}_altura.png")),
                    &mapa,
                    gw,
                    gh,
                );
            }
            out.push((n, mede(&h, &m, gw, r)));
        }
    }
    out
}

pub(super) fn grava_png(p: &std::path::Path, rgba: &[u8], w: usize, h: usize) {
    use crate::tool::paint::tests::look_watercolor_arc::{Canvas, write_png};
    // Sobre o branco do papel: a tinta tem alfa.
    let px = rgba[..w * h * 4]
        .chunks(4)
        .map(|q| {
            let a = f32::from(q[3]) / 255.0;
            let m = |c: u8| (f32::from(c) * a + 250.0 * (1.0 - a)).round() as u8;
            [m(q[0]), m(q[1]), m(q[2])]
        })
        .collect();
    write_png(p, &Canvas { w, h, px }).expect("png");
}

pub(super) const AZ: [u8; 3] = [20, 90, 200];
pub(super) const OC: [u8; 3] = [210, 140, 40];

/// `(nome, presets, cores, cruz)` de uma cena de [`cenas`].
pub(super) type CenaGrumos = (&'static str, Vec<&'static str>, Vec<[u8; 3]>, bool);

pub(super) fn cenas() -> Vec<CenaGrumos> {
    vec![
        ("filbert_ida_volta", vec!["Flat/Filbert"], vec![AZ], false),
        (
            "filbert_cores_cruz",
            vec!["Flat/Filbert"],
            vec![AZ, OC],
            true,
        ),
        (
            "flat_bristle_ida_volta",
            vec!["Flat/Flat Bristle"],
            vec![AZ],
            false,
        ),
        (
            "thick_impasto_ida_volta",
            vec!["Flat/Thick Impasto"],
            vec![AZ],
            false,
        ),
        (
            "knife_oily_ida_volta",
            vec!["Knife/Knife Oily"],
            vec![AZ],
            false,
        ),
        (
            "metallic_cruz",
            vec!["Metallic/Metallic Flat"],
            vec![AZ],
            true,
        ),
        ("dabber_ida_volta", vec!["Dabber/Dabber"], vec![AZ], false),
        (
            "filbert_x_knife_cores",
            vec!["Flat/Filbert", "Knife/Knife Oily"],
            vec![AZ, OC],
            false,
        ),
    ]
}

#[test]
#[ignore = "sonda — corre-se à mão no perfil smoke"]
fn sonda_oleo_grumos() {
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
    let filtro = std::env::var("PH2D_SONDA_CENA").unwrap_or_default();
    // As variantes no MESMO processo, intercaladas por cena: a fracção que o 1.º toque espalha (0 = antes).
    let variantes = [
        ("antes", 0.0),
        ("espalha30", 0.3),
        ("espalha50", 0.5),
        ("espalha70", 0.7),
    ];
    for (rot, ps, cs, cruza) in cenas() {
        if !rot.contains(&filtro) {
            continue;
        }
        for (tag, a) in variantes {
            ph2d_wet_paint::trail::espalha::sobrepoe(Some(a));
            let rot2 = format!("{tag}_{rot}");
            for (n, m) in cena(&b, &rot2, &ps, &cs, cruza, dir.as_deref()) {
                println!(
                    "[grumos] {rot2:<38} N={n:>2} max {:>6.2} p99 {:>6.2} p50 {:>6.2} rug {:>6.3} picos {:>4} massa p99 {:>8.0}",
                    m.max, m.p99, m.p50, m.rug, m.picos, m.massa_p99
                );
            }
        }
    }
    ph2d_wet_paint::trail::espalha::sobrepoe(None);
}

/// A LEI (queixa dos grumos, 31/10): óleo molhado sobre óleo molhado ESPALHA — o relevo satura com as passadas em vez
/// de crescer linear, e o grão não se soma em montes. Antes da cura (Filbert ida e volta): p99 1,84 → 13,71 px (7,4×)
/// e rugosidade 0,091 → 0,410 (4,5×) em 10 passadas, +57 % de N = 6 a 10; depois: 4,45 px (2,4×), 0,124 (1,4×), +14 %.
#[test]
fn o_oleo_sobre_oleo_molhado_satura_o_relevo() {
    let b = Biblioteca::carregar(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes"),
        None,
    );
    let m = cena(&b, "lei", &["Flat/Filbert"], &[AZ], false, None);
    let (m1, m6, m10) = (&m[0].1, &m[2].1, &m[3].1);
    assert!(m10.p99 <= 3.0 * m1.p99, "p99 {} → {}", m1.p99, m10.p99);
    assert!(m10.rug <= 2.0 * m1.rug, "rug {} → {}", m1.rug, m10.rug);
    assert!(m10.p99 <= 1.3 * m6.p99, "N6 {} → N10 {}", m6.p99, m10.p99);
    assert!(m10.picos <= 5, "picos {}", m10.picos);
}
