//! **As estrias das cerdas por TAMANHO** (smoke do dono 31/10: Thick Impasto com Size 64 no painel = raio 32, Water
//! 0,03 sai ENCAROÇADO — grumos sem estrias — e com raio 50 as estrias voltam; as sondas de antes passavam raio 50/64).
//! Um traço CURVO (arco de círculo) com o Thick Impasto de fábrica a raio 16·24·32·50·64, e mede no miolo do traço a
//! **anisotropia** do relevo: a energia do gradiente da altura fina do Óleo Real na NORMAL do traço ÷ a energia ao
//! LONGO dele, depois de tirar o perfil médio por distância ao eixo (a cúpula do traço). As estrias das cerdas são
//! longas no rumo do traço (variam através: razão ≫ 1); o grumo é isotrópico (razão ~1). Mede também a mesma razão na
//! massa e no relevo dos fios (`Grid::altura`) e os px pretos/escuros da tela com luz, depois de um 2.º traço cruzado.
//! `PH2D_SONDA_OUT=<pasta> bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib sonda_oleo_estria -- --ignored --nocapture`
use super::*;
use crate::{PaintMedia, PainterTool};
use ph2d_brush_presets::Biblioteca;
use ph2d_editor_core::tool::{
    CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase, RasterEditTool,
};

const W: u32 = 440;
const H: u32 = 320;
/// O arco: centro e raio (px) e o meio-ângulo.
const C: [f32; 2] = [220.0, 470.0];
const R: f32 = 330.0;
const ABRE: f32 = 0.5;

fn ponto(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn no_arco(th: f32) -> [f32; 2] {
    [C[0] + R * th.sin(), C[1] - R * th.cos()]
}

/// Um traço pela curva `f(s)`, `s ∈ 0..1`, passo 3 px.
fn traco(t: &mut PainterTool, f: &dyn Fn(f32) -> [f32; 2], len: f32) {
    let n = (len / 3.0).ceil() as u32;
    t.on_canvas_pointer(ponto(f(0.0), PointerPhase::Down));
    for k in 1..=n {
        t.on_canvas_pointer(ponto(f(k as f32 / n as f32), PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(ponto(f(1.0), PointerPhase::Up));
}

fn espera(t: &mut PainterTool, n: usize) {
    for _ in 0..n {
        t.on_tick(1000.0 / 60.0);
    }
}

pub(super) struct Estria {
    /// Anisotropia da altura fina (normal ÷ rumo).
    pub fina: f64,
    pub massa: f64,
    pub fios: f64,
    /// A rugosidade (desvio-padrão do resíduo) da altura fina.
    pub rug: f64,
}

/// Os campos por célula: (altura fina, massa, altura dos fios, gw, gh, razão).
fn campos(t: &mut PainterTool) -> (Vec<f64>, Vec<f64>, Vec<f64>, usize, usize, f32) {
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
    let (mut h, mut m, mut a) = (vec![0.0; gw * gh], vec![0.0; gw * gh], vec![0.0; gw * gh]);
    for y in 0..gh {
        for x in 0..gw {
            h[x + y * gw] = campo.at_fina(x as isize + 1, y as isize + 1);
            let i = (x + 1) + (y + 1) * st;
            for (g, v) in &grades {
                if *v {
                    m[x + y * gw] += f64::from(g.sett[i]) + f64::from(g.susp[i]);
                    if !g.altura.is_empty() {
                        a[x + y * gw] += f64::from(g.altura[i]);
                    }
                }
            }
        }
    }
    (h, m, a, gw, gh, r)
}

/// A razão normal ÷ rumo do gradiente de `v` no miolo do arco (|d| < 0,5 raio, ângulo dentro de 80 % da abertura),
/// depois de tirar o perfil médio por distância ao eixo; e o desvio-padrão do resíduo.
fn anis(v: &[f64], gw: usize, gh: usize, r: f32, raio: f32) -> (f64, f64) {
    let dentro = |x: usize, y: usize| -> Option<(f32, f32, f32)> {
        let p = [(x as f32 + 0.5) * r, (y as f32 + 0.5) * r];
        let (dx, dy) = (p[0] - C[0], p[1] - C[1]);
        let dist = dx.hypot(dy);
        let th = dx.atan2(-dy);
        (th.abs() < 0.8 * ABRE && (dist - R).abs() < 0.5 * raio).then_some((
            dist - R,
            dx / dist,
            dy / dist,
        ))
    };
    // O perfil médio por faixa de 1 célula através do traço.
    let nb = (raio / r).ceil() as usize + 2;
    let (mut soma, mut cont) = (vec![0.0; 2 * nb + 1], vec![0usize; 2 * nb + 1]);
    let faixa = |d: f32| ((d / r).round() as isize + nb as isize) as usize;
    for y in 1..gh - 1 {
        for x in 1..gw - 1 {
            if let Some((d, _, _)) = dentro(x, y) {
                soma[faixa(d)] += v[x + y * gw];
                cont[faixa(d)] += 1;
            }
        }
    }
    let media = |x: usize, y: usize| -> f64 {
        let p = [(x as f32 + 0.5) * r, (y as f32 + 0.5) * r];
        let d = (p[0] - C[0]).hypot(p[1] - C[1]) - R;
        let b = faixa(d.clamp(-(nb as f32) * r, nb as f32 * r));
        if cont[b] == 0 {
            0.0
        } else {
            soma[b] / cont[b] as f64
        }
    };
    let res = |x: usize, y: usize| v[x + y * gw] - media(x, y);
    let (mut en, mut et, mut s2, mut n) = (0.0, 0.0, 0.0, 0usize);
    for y in 2..gh - 2 {
        for x in 2..gw - 2 {
            if let Some((_, nx, ny)) = dentro(x, y) {
                let gx = 0.5 * (res(x + 1, y) - res(x - 1, y));
                let gy = 0.5 * (res(x, y + 1) - res(x, y - 1));
                let gn = gx * f64::from(nx) + gy * f64::from(ny);
                let gt = -gx * f64::from(ny) + gy * f64::from(nx);
                en += gn * gn;
                et += gt * gt;
                s2 += res(x, y).powi(2);
                n += 1;
            }
        }
    }
    (en / et.max(1e-12), (s2 / n.max(1) as f64).sqrt())
}

/// `v` menos a média da caixa `(2k+1)²`: o detalhe (estria ou grumo) sem a forma do traço.
fn passa_alto(v: &[f64], gw: usize, gh: usize, k: isize) -> Vec<f64> {
    let mut o = vec![0.0; v.len()];
    for y in 0..gh as isize {
        for x in 0..gw as isize {
            let (mut s, mut n) = (0.0, 0.0);
            for dy in -k..=k {
                for dx in -k..=k {
                    let (xx, yy) = (x + dx, y + dy);
                    if xx >= 0 && yy >= 0 && xx < gw as isize && yy < gh as isize {
                        s += v[xx as usize + yy as usize * gw];
                        n += 1.0;
                    }
                }
            }
            o[x as usize + y as usize * gw] = v[x as usize + y as usize * gw] - s / n;
        }
    }
    o
}

/// Px pretos (máx dos canais < 40) e escuros (luma < 60) da tela composta sobre o papel.
fn escuros(t: &mut PainterTool) -> (usize, usize) {
    t.wet_recomposite_full();
    let (mut p, mut e) = (0, 0);
    for q in t.canvas_rgba.chunks(4) {
        if q[3] < 128 {
            continue;
        }
        let a = f32::from(q[3]) / 255.0;
        let m = |c: u8| f32::from(c) * a + 250.0 * (1.0 - a);
        let (r, g, b) = (m(q[0]), m(q[1]), m(q[2]));
        if r.max(g).max(b) < 40.0 {
            p += 1;
        }
        if 0.2126 * r + 0.7152 * g + 0.0722 * b < 60.0 {
            e += 1;
        }
    }
    (p, e)
}

pub(super) fn thick_impasto(b: &Biblioteca) -> ph2d_brush_presets::Preset {
    let rel = "WetPainter/Oil/Flat/Thick Impasto.ph2d-brush.png";
    b.entradas
        .iter()
        .find(|e| e.relativo == rel)
        .unwrap_or_else(|| panic!("preset {rel}"))
        .preset
        .clone()
}

/// Um arco com o preset `p` ao `raio`, Water 0,03; depois um 2.º arco que o cruza. Devolve a estria do 1.º e os
/// px (pretos, escuros) depois do cruzamento.
pub(super) fn arco(
    p: &ph2d_brush_presets::Preset,
    raio: f32,
    grava: Option<(&std::path::Path, &str)>,
) -> (Estria, (usize, usize)) {
    let mut t = PainterTool::default();
    t.set_source(vec![250u8; (W * H * 4) as usize], W, H);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    t.set_semente_dos_tracos(0);
    let erros = t.wet_preset_aplicar(p);
    assert!(erros.is_empty(), "{erros:?}");
    t.set_brush_size_px(raio);
    t.set_brush_color_srgb8([0x2f, 0x8f, 0xd8]);
    t.handle_panel_event(PanelEvent::SetValue(
        crate::ids::PAINTER_WETPAINT_WATER,
        0.03,
    ));
    traco(&mut t, &|s| no_arco(-ABRE + 2.0 * ABRE * s), 2.0 * ABRE * R);
    espera(&mut t, 30);
    let (h, m, a, gw, gh, r) = campos(&mut t);
    let (fina, rug) = anis(
        &passa_alto(&h, gw, gh, (4.0 / r).ceil() as isize),
        gw,
        gh,
        r,
        raio,
    );
    let e = Estria {
        fina,
        massa: anis(&m, gw, gh, r, raio).0,
        fios: anis(&a, gw, gh, r, raio).0,
        rug,
    };
    if let Some((dir, rot)) = grava {
        t.wet_recomposite_full();
        grava_png(&dir.join(format!("{rot}_arco.png")), &t.canvas_rgba);
    }
    // O cruzamento: um arco quase vertical pelo meio.
    let cruz = |s: f32| {
        [
            C[0] - 40.0 + 80.0 * (s - 0.5).powi(2) * 2.0,
            40.0 + 200.0 * s,
        ]
    };
    traco(&mut t, &cruz, 210.0);
    espera(&mut t, 60);
    let px = escuros(&mut t);
    if let Some((dir, rot)) = grava {
        grava_png(&dir.join(format!("{rot}_cruz.png")), &t.canvas_rgba);
    }
    (e, px)
}

fn grava_png(p: &std::path::Path, rgba: &[u8]) {
    use crate::tool::paint::tests::look_watercolor_arc::{Canvas, write_png};
    let (w, h) = (W as usize, H as usize);
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

#[test]
#[ignore = "sonda — corre-se à mão no perfil smoke"]
fn sonda_oleo_estria() {
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
    let base = thick_impasto(&b);
    // As variantes no MESMO processo: um knob do motor sobreposto ao de fábrica.
    let variantes: Vec<(&str, Vec<(&str, f64)>)> = vec![
        ("fabrica", vec![]),
        ("altura0", vec![("bristleHeight", 0.0)]),
        ("jitter0", vec![("@jitter", 0.0)]),
        ("pig250", vec![("pigmentPerDab", 250.0)]),
        ("emboss0", vec![("embossSizeExp", 0.0)]),
        ("visgrain0", vec![("visualGrain", 0.0)]),
        ("spacing1", vec![("@spacing", 0.05)]),
    ];
    let filtro = std::env::var("PH2D_SONDA_VAR").unwrap_or_default();
    for raio in [32.0f32, 50.0] {
        for (tag, kn) in &variantes {
            if !tag.contains(&filtro) {
                continue;
            }
            let mut p = base.clone();
            for (k, v) in kn {
                if let Some(b) = k.strip_prefix('@') {
                    p.parametros
                        .brush
                        .insert(b.to_string(), serde_json::json!(*v));
                } else {
                    p.parametros.engine.insert((*k).to_string(), *v);
                }
            }
            let rot = format!("{tag}_r{raio:02}");
            let (e, (pr, es)) = arco(&p, raio, dir.as_deref().map(|d| (d, rot.as_str())));
            println!(
                "[estria] {rot:<16} anis fina {:>6.2} massa {:>6.2} fios {:>6.2} rug {:>6.3} pretos {pr:>5} escuros {es:>5}",
                e.fina, e.massa, e.fios, e.rug
            );
        }
    }
}
