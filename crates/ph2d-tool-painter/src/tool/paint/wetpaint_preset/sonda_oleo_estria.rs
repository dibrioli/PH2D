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
    /// A correlação do detalhe a 8 px AO LONGO do traço e ATRAVÉS (a estria: alta ao longo, ~0 através).
    pub ct8: f64,
    pub cn8: f64,
    /// Os picos isolados (máximo 5×5 com proeminência > 0,5) por 1000 px² do miolo — o grumo.
    pub picos: f64,
}

/// A régua do grumo sobre o detalhe `v` (passa-alto): correlação a `lag` px ao longo/através e picos por área.
fn grumo(v: &[f64], gw: usize, gh: usize, r: f32, raio: f32, lag: f32) -> (f64, f64, f64) {
    let at = |p: [f32; 2]| -> Option<f64> {
        let (x, y) = ((p[0] / r).floor() as isize, (p[1] / r).floor() as isize);
        (x >= 0 && y >= 0 && (x as usize) < gw && (y as usize) < gh)
            .then(|| v[x as usize + y as usize * gw])
    };
    let (mut v2, mut ct, mut cn, mut n, mut picos) = (0.0, 0.0, 0.0, 0usize, 0usize);
    for y in 2..gh - 2 {
        for x in 2..gw - 2 {
            let p = [(x as f32 + 0.5) * r, (y as f32 + 0.5) * r];
            let (dx, dy) = (p[0] - C[0], p[1] - C[1]);
            let dist = dx.hypot(dy);
            if dx.atan2(-dy).abs() >= 0.7 * ABRE || (dist - R).abs() >= 0.5 * raio {
                continue;
            }
            let (nx, ny) = (dx / dist, dy / dist);
            let a = v[x + y * gw];
            let (Some(t), Some(q)) = (
                at([p[0] - ny * lag, p[1] + nx * lag]),
                at([p[0] + nx * lag, p[1] + ny * lag]),
            ) else {
                continue;
            };
            v2 += a * a;
            ct += a * t;
            cn += a * q;
            n += 1;
            let viz = (-2isize..=2).flat_map(|j| (-2isize..=2).map(move |i| (i, j)));
            let (mut mn, mut mx) = (f64::INFINITY, f64::NEG_INFINITY);
            for (i, j) in viz {
                let w = v[(x as isize + i) as usize + (y as isize + j) as usize * gw];
                mn = mn.min(w);
                mx = mx.max(w);
            }
            if a >= mx && a - mn > 0.5 {
                picos += 1;
            }
        }
    }
    let area = n as f64 * f64::from(r * r) / 1000.0;
    (
        ct / v2.max(1e-12),
        cn / v2.max(1e-12),
        picos as f64 / area.max(1e-9),
    )
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
    t.sonda_raio(raio);
    t.set_brush_color_srgb8([0x2f, 0x8f, 0xd8]);
    t.handle_panel_event(PanelEvent::SetValue(
        crate::ids::PAINTER_WETPAINT_WATER,
        0.03,
    ));
    traco(&mut t, &|s| no_arco(-ABRE + 2.0 * ABRE * s), 2.0 * ABRE * R);
    espera(&mut t, 30);
    let (h, m, a, gw, gh, r) = campos(&mut t);
    let hp = passa_alto(&h, gw, gh, (4.0 / r).ceil() as isize);
    let (fina, rug) = anis(&hp, gw, gh, r, raio);
    let hp8 = passa_alto(&h, gw, gh, (8.0 / r).ceil() as isize);
    let (ct8, cn8, picos) = grumo(&hp8, gw, gh, r, raio, 8.0);
    let e = Estria {
        fina,
        massa: anis(&m, gw, gh, r, raio).0,
        fios: anis(&a, gw, gh, r, raio).0,
        rug,
        ct8,
        cn8,
        picos,
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

/// O brilho PICOTADO (smoke do dono 12/11: «o brilho da tinta da nova luz está picotado, pixelado e pontilhado»):
/// na tinta (alfa ≥ 128) da tela com luz, os px cuja luma passa a média dos 8 vizinhos em mais de 40 (pontinhos
/// soltos) e a energia de alta frequência média `|L − caixa 3×3|`.
pub(super) fn picotado(rgba: &[u8]) -> (usize, f64) {
    let (w, h) = (W as usize, H as usize);
    let l = |x: usize, y: usize| {
        let q = &rgba[(x + y * w) * 4..][..4];
        let a = f64::from(q[3]) / 255.0;
        let m = |c: u8| f64::from(c) * a + 250.0 * (1.0 - a);
        0.2126 * m(q[0]) + 0.7152 * m(q[1]) + 0.0722 * m(q[2])
    };
    let (mut soltos, mut hf, mut n) = (0usize, 0.0f64, 0usize);
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            if (0..9).any(|k| rgba[(x + k % 3 - 1 + (y + k / 3 - 1) * w) * 4 + 3] < 128) {
                continue;
            }
            let c = l(x, y);
            let viz = (0..9)
                .filter(|&k| k != 4)
                .map(|k| l(x + k % 3 - 1, y + k / 3 - 1))
                .sum::<f64>()
                / 8.0;
            soltos += usize::from(c - viz > 40.0);
            hf += (c - (viz * 8.0 + c) / 9.0).abs();
            n += 1;
        }
    }
    (soltos, hf / n.max(1) as f64)
}

/// O arco do Thick Impasto, vermelho escuro, com o Óleo Real a raio 48, a óptica de fábrica mexida por `ajuste`: a tela
/// com luz.
fn arco_com_brilho(
    p: &ph2d_brush_presets::Preset,
    ajuste: fn(&mut ph2d_wet_paint::oleo_real::OleoReal),
) -> Vec<u8> {
    let mut t = PainterTool::default();
    ajuste(&mut t.paint.wetpaint.oleo_real_valores);
    t.set_source(vec![250u8; (W * H * 4) as usize], W, H);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    t.set_semente_dos_tracos(0);
    assert!(t.wet_preset_aplicar(p).is_empty());
    t.set_wet_oleo_real_ligado(true);
    t.sonda_raio(48.0);
    t.set_brush_color_srgb8([0x8a, 0x16, 0x1c]);
    traco(&mut t, &|s| no_arco(-ABRE + 2.0 * ABRE * s), 2.0 * ABRE * R);
    espera(&mut t, 60);
    t.wet_recomposite_full();
    t.canvas_rgba.to_vec()
}

/// ⭐ **O brilho do óleo não é picotado** (smoke do dono 12/11: «picotado, pixelado e pontilhado»): no arco do Thick
/// Impasto os pontinhos soltos da tela com luz ficam abaixo de 2× os da mesma tela sem brilho. Medido: sem brilho 123;
/// o reflexo do ambiente sem o antisserrilhado (`antisserrilhado_ambiente = 0`, o de 27/10) 553 — vermelho; a `1`, 173.
#[test]
fn o_brilho_do_oleo_nao_e_picotado() {
    let b = Biblioteca::carregar(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes"),
        None,
    );
    let p = thick_impasto(&b);
    let (com, _) = picotado(&arco_com_brilho(&p, |_| {}));
    let (sem, _) = picotado(&arco_com_brilho(&p, |o| o.brilho = 0.0));
    assert!(sem > 50, "premissa: o relevo tem detalhe ({sem})");
    assert!(
        com <= 2 * sem,
        "{com} pontinhos soltos com brilho × {sem} sem"
    );
}

/// `PH2D_SONDA_OUT=<pasta> bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib
/// sonda_brilho_picotado -- --ignored --nocapture`: o [`arco_com_brilho`] por variante da óptica; imprime os
/// [`picotado`] e grava `brilho_<variante>.png`.
#[test]
#[ignore = "sonda — corre-se à mão no perfil smoke"]
fn sonda_brilho_picotado() {
    let b = Biblioteca::carregar(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes"),
        None,
    );
    let p = thick_impasto(&b);
    let out = std::env::var("PH2D_SONDA_OUT")
        .ok()
        .map(std::path::PathBuf::from);
    type Ajuste = fn(&mut ph2d_wet_paint::oleo_real::OleoReal);
    let variantes: [(&str, Ajuste); 4] = [
        ("fabrica", |_| {}),
        ("sem_brilho", |o| o.brilho = 0.0),
        ("amb_0", |o| o.af.antisserrilhado_ambiente = 0.0),
        ("amb_0_5", |o| o.af.antisserrilhado_ambiente = 0.5),
    ];
    for (nome, ajuste) in variantes {
        let px = arco_com_brilho(&p, ajuste);
        let (soltos, hf) = picotado(&px);
        println!("BRILHO {nome}: {soltos} pontinhos soltos · alta frequência {hf:.3}");
        if let Some(d) = &out {
            std::fs::create_dir_all(d).expect("pasta");
            grava_png(&d.join(format!("brilho_{nome}.png")), &px);
        }
    }
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
        ("pig250", vec![("pigmentPerDab", 250.0)]),
        ("capmax0", vec![("capsuleMaxPx", 0.0)]),
        ("capmax2", vec![("capsuleMaxPx", 2.0)]),
        ("capmax8", vec![("capsuleMaxPx", 8.0)]),
        ("capsp05", vec![("capsuleSpacing", 0.05)]),
        ("capsp01", vec![("capsuleSpacing", 0.0125)]),
        ("capperto", vec![("capsuleRoundNearest", 1.0)]),
    ];
    let filtro = std::env::var("PH2D_SONDA_VAR").unwrap_or_default();
    for raio in [16.0f32, 24.0, 32.0, 40.0, 50.0, 64.0] {
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
                "[estria] {rot:<16} anis fina {:>6.2} massa {:>6.2} fios {:>6.2} rug {:>6.3} ct8 {:>5.2} cn8 {:>5.2} picos/kpx² {:>5.2} pretos {pr:>5} escuros {es:>5}",
                e.fina, e.massa, e.fios, e.rug, e.ct8, e.cn8, e.picos
            );
        }
    }
}

/// ⭐ **A LEI (smoke do dono 31/10, Size 64 = raio 32)**: o Thick Impasto não escurece em grumos em tamanho nenhum. O
/// sub-passo da cápsula arredondado a células inteiras pousava até 2× mais sub-dabs por célula entre raio 24 e 40
/// (64 a r32 e 80 a r40 contra 50 a r50), e com eles a tinta e o grumo; a cura pesa cada sub-dab pelo sub-passo natural.
/// Px escuros por px de raio, arco + cruzamento: antes r16 4,2 · r32 16,1 · r40 24,1 · r50 6,4 · r64 11,0; depois
/// 4,2 · 3,9 · 4,9 · 4,7 · 2,5. Pretos: 0 antes e depois.
#[test]
fn o_impasto_nao_encaroca_em_tamanho_nenhum() {
    let b = Biblioteca::carregar(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes"),
        None,
    );
    let p = thick_impasto(&b);
    for raio in [16.0f32, 32.0, 64.0] {
        let (_, (pretos, escuros)) = arco(&p, raio, None);
        assert_eq!(pretos, 0, "raio {raio}");
        let densidade = escuros as f32 / raio;
        assert!(
            densidade <= 7.0,
            "raio {raio}: {escuros} px escuros ({densidade:.1} por px de raio)"
        );
    }
}
