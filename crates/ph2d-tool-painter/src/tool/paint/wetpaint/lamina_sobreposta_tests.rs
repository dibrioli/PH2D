//! **BUG do smoke 27/10 — a Knife Textured sobreposta** (`docs/Painter/Pinceis/W3_P_Knife.md` §BUG): o preset de
//! fábrica, três traços diagonais sobrepostos no papel com grão, a foto do produto (Óleo Real ligado + o relevo do
//! papel) depois de cada traço e a régua da espessura: degraus entre células vizinhas no miolo e cortes.

use crate::PainterTool;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_wet_paint::tuning::Knob;

pub(super) const TW: u32 = 700;
pub(super) const TH: u32 = 560;

/// O preset de fábrica `nome`, a tela do papel, o papel com grão (`papel`: o Cold do painel com o Relief 0,6) e o
/// Óleo Real ligado desde o início (a tela é a do produto a cada quadro).
pub(super) fn pincel(nome: &str, papel: bool) -> PainterTool {
    pincel_com(nome, papel, |_| {})
}

/// [`pincel`] com o preset mudado por `muda` ANTES de aplicado (a porta do ficheiro: o que a fábrica gravaria).
pub(super) fn pincel_com(
    nome: &str,
    papel: bool,
    muda: impl Fn(&mut ph2d_brush_presets::Preset),
) -> PainterTool {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let b = ph2d_brush_presets::Biblioteca::carregar(&raiz, None);
    let mut p = b
        .entradas
        .iter()
        .find(|e| e.preset.parametros.name == nome)
        .unwrap_or_else(|| panic!("{nome}: não está na fábrica"))
        .preset
        .clone();
    muda(&mut p);
    let p = &p;
    let mut t = PainterTool::default();
    t.set_source(vec![236u8; (TW * TH * 4) as usize], TW, TH);
    t.set_wet_relogio_fixo(true);
    t.set_semente_dos_tracos(0);
    assert!(t.preset_aplicar(p).expect("meio").is_empty());
    t.set_brush_color_srgb8([30, 80, 220]);
    t.set_brush_size_px(45.0);
    t.paint.wetpaint.tilt_on = false;
    if papel {
        t.set_brush_paper_kind(1);
        t.set_substrate_depth(0.6);
    }
    t.set_wet_oleo_real_ligado(true);
    t
}

/// O traço `k` (0, 1, 2): a diagonal de baixo-esquerda para cima-direita, cada um deslocado ~14 px para baixo e um
/// pouco ao comprido (as fotos do dono); 6 px por evento, a pressão a subir no primeiro 15 % e a descer no último 10 %.
pub(super) fn traco(t: &mut PainterTool, k: usize) {
    let d = k as f32;
    let (a, b) = (
        [110.0 - 12.0 * d, 450.0 + 14.0 * d],
        [600.0 + 8.0 * d, 100.0 + 16.0 * d],
    );
    let n = ((b[0] - a[0]).hypot(b[1] - a[1]) / 6.0).ceil() as usize;
    let pressao = |f: f32| (0.25 + 0.65 * (f / 0.15).min(1.0)) * ((1.0 - f) / 0.1).clamp(0.3, 1.0);
    let pt = |f: f32, phase| CanvasPointer {
        pos: [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f],
        pressure: pressao(f),
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(pt(0.0, PointerPhase::Down));
    for i in 1..=n {
        t.on_canvas_pointer(pt(i as f32 / n as f32, PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(pt(1.0, PointerPhase::Up));
    for _ in 0..30 {
        t.on_tick(1000.0 / 60.0);
    }
}

/// A régua da espessura no MIOLO (células com tinta e as 8 vizinhas também, cobertura > 0,5): a mediana da
/// espessura, o p99 e o máximo do degrau |ΔT| entre vizinhas (÷ a mediana), a fracção de pares com degrau > 0,5 e os
/// CORTES (células com massa < ¼ da mediana com tinta cheia a ≤ 3 células dos dois lados, numa linha ou coluna).
#[derive(Debug, Clone)]
pub(super) struct Regua {
    /// As células do miolo (`x`, `y`, ratio 1 = o pixel `x − 1`, `y − 1`).
    pub miolo: Vec<(usize, usize)>,
    pub mediana: f64,
    pub p99: f64,
    pub max: f64,
    pub frac_05: f64,
    pub cortes: usize,
    pub marcadas: f64,
    /// A maior espessura numa célula com tinta (em toda a tela, bordas incluídas) ÷ a mediana do miolo, e quantas
    /// passam de 4 × a mediana.
    pub tmax: f64,
    pub t4: usize,
}

pub(super) fn regua(t: &mut PainterTool) -> Regua {
    let s = t.paint.wetpaint.session.as_mut().expect("sessão");
    s.bring_home();
    let g = s.engine.active_grid();
    let (w, h, st) = (g.w, g.h, g.s);
    let m = |x: usize, y: usize| f64::from(g.sett[x + y * st]) + f64::from(g.susp[x + y * st]);
    let cheio = |x: usize, y: usize| ph2d_wet_paint::opacity::alpha_of_mass(m(x, y)) > 0.5;
    let miolo = |x: usize, y: usize| {
        (x > 1 && y > 1 && x < w && y < h) && (0..9).all(|k| cheio(x + k % 3 - 1, y + k / 3 - 1))
    };
    let tt = |x: usize, y: usize| ph2d_wet_paint::grid::espessura::em(g, x + y * st);
    if let Ok(c) = std::env::var("PH2D_SONDA_MAPA") {
        let v: Vec<usize> = c.split(',').map(|n| n.parse().expect("x,y")).collect();
        for y in v[1] - 8..=v[1] + 8 {
            let linha: Vec<String> = (v[0] - 10..=v[0] + 10)
                .map(|x| format!("{:>5.0}", tt(x, y)))
                .collect();
            println!("[mapa] y {y}: {}", linha.join(""));
        }
    }
    let mut ts = Vec::new();
    let mut cel = Vec::new();
    let mut ms = Vec::new();
    let mut marc = 0usize;
    for y in 2..h {
        for x in 2..w {
            if miolo(x, y) {
                cel.push((x, y));
                ts.push(tt(x, y));
                ms.push(m(x, y));
                if ph2d_wet_paint::grid::nivelado::em(g, x + y * st) > 0.5 {
                    marc += 1;
                }
            }
        }
    }
    let n_miolo = ts.len().max(1);
    let mediana = |mut v: Vec<f64>| {
        v.sort_by(f64::total_cmp);
        v.get(v.len() / 2).copied().unwrap_or(0.0)
    };
    let med = mediana(ts.clone()).max(1e-9);
    let med_m = mediana(ms).max(1e-9);
    let mut degraus = Vec::new();
    for y in 2..h {
        for x in 2..w {
            if !miolo(x, y) {
                continue;
            }
            for (vx, vy) in [(x + 1, y), (x, y + 1)] {
                if miolo(vx, vy) {
                    degraus.push((tt(x, y) - tt(vx, vy)).abs() / med);
                }
            }
        }
    }
    degraus.sort_by(f64::total_cmp);
    let nd = degraus.len().max(1);
    let mut cortes = 0;
    for y in 4..h.saturating_sub(3) {
        for x in 4..w.saturating_sub(3) {
            if m(x, y) >= 0.25 * med_m {
                continue;
            }
            let lados = |dx: i32, dy: i32| {
                let tem = |s: i32| {
                    (1..=3).any(|k| {
                        let (px, py) = (x as i32 + s * k * dx, y as i32 + s * k * dy);
                        m(px as usize, py as usize) > 0.5 * med_m
                    })
                };
                tem(1) && tem(-1)
            };
            if lados(1, 0) || lados(0, 1) {
                if std::env::var("PH2D_SONDA_CORTES").is_ok() && cortes < 40 {
                    println!(
                        "[corte] célula ({x}, {y}) massa {:.1} ratio {}",
                        m(x, y),
                        s.ratio
                    );
                }
                cortes += 1;
            }
        }
    }
    Regua {
        miolo: cel,
        mediana: med,
        p99: degraus.get(nd * 99 / 100).copied().unwrap_or(0.0),
        max: degraus.last().copied().unwrap_or(0.0),
        frac_05: degraus.iter().filter(|&&d| d > 0.5).count() as f64 / nd as f64,
        cortes,
        marcadas: marc as f64 / n_miolo as f64,
        tmax: (0..g.cells)
            .map(|i| ph2d_wet_paint::grid::espessura::em(g, i))
            .fold(0.0, f64::max)
            / med,
        t4: (0..g.cells)
            .filter(|&i| ph2d_wet_paint::grid::espessura::em(g, i) > 4.0 * med)
            .count(),
    }
}

/// A foto do produto: a tela composta do Wet Paint (com o Óleo Real) e o relevo do papel por cima, em RGB.
pub(super) fn foto(t: &PainterTool) -> Vec<u8> {
    let mut px: Vec<u8> = t.canvas_rgba.as_ref().clone();
    t.apply_impasto_light(
        &mut px,
        crate::Region {
            x: 0,
            y: 0,
            w: TW,
            h: TH,
        },
    );
    px.as_chunks::<4>()
        .0
        .iter()
        .flat_map(|p| [p[0], p[1], p[2]])
        .collect()
}

/// A foto sem um campo do render (`limpa` esvazia-o na grade): K7 sem a marca, K6 sem a crista. Recompõe, fotografa
/// e devolve o campo e a tela como estavam.
fn foto_sem(
    t: &mut PainterTool,
    limpa: fn(&mut ph2d_wet_paint::grid::Grid) -> Vec<f32>,
    poe: fn(&mut ph2d_wet_paint::grid::Grid, Vec<f32>),
) -> Vec<u8> {
    let guardado = {
        let s = t.paint.wetpaint.session.as_mut().expect("sessão");
        s.bring_home();
        limpa(s.engine.active_grid_mut())
    };
    t.wet_recomposite_full();
    let f = foto(t);
    {
        let s = t.paint.wetpaint.session.as_mut().expect("sessão");
        poe(s.engine.active_grid_mut(), guardado);
    }
    t.wet_recomposite_full();
    f
}

/// Na FOTO, as fracções do miolo com a luminância 60 níveis acima da mediana dele (as fendas claras, o brilho na
/// parede) e 60 abaixo (as placas de sombra).
pub(super) fn fendas(rgb: &[u8], miolo: &[(usize, usize)]) -> (f64, f64) {
    let lum: Vec<f64> = miolo
        .iter()
        .map(|&(x, y)| {
            let k = 3 * ((x - 1) + (y - 1) * TW as usize);
            0.2126 * f64::from(rgb[k])
                + 0.7152 * f64::from(rgb[k + 1])
                + 0.0722 * f64::from(rgb[k + 2])
        })
        .collect();
    let mut o = lum.clone();
    o.sort_by(f64::total_cmp);
    let med = o.get(o.len() / 2).copied().unwrap_or(0.0);
    let n = lum.len().max(1) as f64;
    (
        lum.iter().filter(|&&l| l > med + 60.0).count() as f64 / n,
        lum.iter().filter(|&&l| l < med - 60.0).count() as f64 / n,
    )
}

fn grava(dir: &std::path::Path, nome: &str, rgb: &[u8]) {
    let mut ppm = format!("P6\n{TW} {TH}\n255\n").into_bytes();
    ppm.extend_from_slice(rgb);
    std::fs::write(dir.join(format!("{nome}.ppm")), ppm).expect("grava");
}

/// **Sonda (ignorada):** as variantes no MESMO processo, 1/2/3 traços cada; a régua e as fotos em
/// `$PH2D_SONDA_OUT`. `PH2D_SONDA_SO=<nome,…>` corre só essas.
/// `PH2D_SONDA_OUT=<pasta> bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib
/// sonda_knife_textured_sobreposta -- --ignored --nocapture`
#[test]
#[ignore = "sonda: o BUG do smoke 27/10; corre-se à mão"]
fn sonda_knife_textured_sobreposta() {
    let dir = std::path::PathBuf::from(std::env::var("PH2D_SONDA_OUT").expect("PH2D_SONDA_OUT"));
    std::fs::create_dir_all(&dir).expect("pasta");
    let so = std::env::var("PH2D_SONDA_SO").unwrap_or_default();
    type Ajuste = fn(&mut PainterTool);
    let variantes: [(&str, &str, bool, Ajuste); 10] = [
        ("semtex", "Knife Textured", true, |t| {
            t.paint.brush.texture.kind = ph2d_painter_brush::TextureKind::None;
        }),
        ("semtex_nivel0", "Knife Textured", true, |t| {
            t.paint.brush.texture.kind = ph2d_painter_brush::TextureKind::None;
            t.paint.wetpaint.knobs.set(Knob::KnifeLevel, 0.0);
        }),
        ("fabrica", "Knife Textured", true, |_| {}),
        ("liso", "Knife Textured", false, |_| {}),
        ("nivel0", "Knife Textured", true, |t| {
            t.paint.wetpaint.knobs.set(Knob::KnifeLevel, 0.0);
        }),
        ("ridge0", "Knife Textured", true, |t| {
            t.paint.wetpaint.knobs.set(Knob::KnifeRidge, 0.0);
        }),
        ("nivel0_ridge0", "Knife Textured", true, |t| {
            t.paint.wetpaint.knobs.set(Knob::KnifeLevel, 0.0);
            t.paint.wetpaint.knobs.set(Knob::KnifeRidge, 0.0);
        }),
        ("smooth3", "Knife Smooth 3", true, |_| {}),
        ("oily", "Knife Oily", true, |_| {}),
        ("stain", "Knife Stain", true, |_| {}),
    ];
    for (nome, preset, papel, ajuste) in variantes {
        if !so.is_empty() && !so.split(',').any(|s| s == nome) {
            continue;
        }
        let mut t = pincel(preset, papel);
        ajuste(&mut t);
        for k in 0..3 {
            traco(&mut t, k);
            let r = regua(&mut t);
            println!(
                "[sobreposta] {nome:<14} {} traço(s): T med {:>8.1} · degrau p99 {:.3} máx {:.3} · >0,5 {:.4} · cortes {} · marcadas {:.2} · tmax {:.3e} · t>4 {}",
                k + 1,
                r.mediana,
                r.p99,
                r.max,
                r.frac_05,
                r.cortes,
                r.marcadas,
                r.tmax,
                r.t4
            );
            let f = foto(&t);
            let (claros, escuros) = fendas(&f, &r.miolo);
            println!(
                "[foto] {nome:<14} {} traço(s): claros {claros:.4} · escuros {escuros:.4}",
                k + 1
            );
            grava(&dir, &format!("{nome}_{}", k + 1), &f);
            if nome == "fabrica" {
                let sem_marca = foto_sem(
                    &mut t,
                    |g| std::mem::take(&mut g.nivelado),
                    |g, v| g.nivelado = v,
                );
                grava(&dir, &format!("semK7_{}", k + 1), &sem_marca);
                let sem_crista = foto_sem(
                    &mut t,
                    |g| std::mem::take(&mut g.crista),
                    |g, v| g.crista = v,
                );
                grava(&dir, &format!("semK6_{}", k + 1), &sem_crista);
            }
        }
    }
}

/// ⭐ **BUG do smoke 27/10 — a Knife Textured sobreposta** (gate): o preset de fábrica, três traços diagonais
/// sobrepostos no papel com grão. A lâmina que nivela ([`ph2d_wet_paint::lamina::nivelar`]) leva o vale do grão (massa
/// ~10) à altura da média; guardada como FACTOR, a massa que lhe chegava depois multiplicava-se por ele (~10²) e o
/// factor ia a ∞ no 2.º traço — a espessura saía 10²¹ × a mediana e a foto em placas negras. Com o excesso ABSOLUTO
/// abaixo da massa que cobre ([`ph2d_wet_paint::grid::espessura::MASSA_QUE_COBRE`]), medido: a maior espessura ≤ 2,3 ×
/// a mediana do miolo e o degrau máx entre vizinhas ≤ 1,22 × a mediana nos três traços (sem nível, 0,74).
#[test]
fn tres_tracos_sobrepostos_da_faca_textured_ficam_no_planalto() {
    let mut t = pincel("Knife Textured", true);
    t.set_wet_oleo_real_ligado(false);
    for k in 0..3 {
        traco(&mut t, k);
        let r = regua(&mut t);
        assert!(
            r.tmax <= 4.0,
            "{} traço(s): a espessura subiu a {:.3e} × a mediana",
            k + 1,
            r.tmax
        );
        assert!(
            r.max <= 2.0,
            "{} traço(s): degrau máx {:.3e} × a mediana",
            k + 1,
            r.max
        );
    }
}

/// **Sonda (ignorada) — os cortes do 1.º traço da Knife Textured** (`W3_P_Knife.md` §BUG, «o que fica»): a textura
/// `Noise` 60 × 2 do preset (Contrast 0,5) quase parava a tinta em 1–2 colunas; a fábrica leva 0,25 desde então
/// (todas as variantes ≤ 0,25 dão 0 cortes e a MESMA foto: a tinta satura acima do piso). As variantes da afinação (Contrast/Brightness = os
/// `params` 0/1, a escala `size`) no MESMO processo, 1/2/3 traços no papel com grão e o Óleo Real ligado: os cortes
/// da régua, as fendas claras/escuras e a luminância mediana do miolo; as fotos em `$PH2D_SONDA_OUT/cortes_*`.
#[test]
#[ignore = "sonda: os cortes da Knife Textured; corre-se à mão"]
fn sonda_knife_textured_cortes() {
    let dir = std::path::PathBuf::from(std::env::var("PH2D_SONDA_OUT").expect("PH2D_SONDA_OUT"));
    std::fs::create_dir_all(&dir).expect("pasta");
    let variantes: [(&str, f32, f32, f32); 8] = [
        ("fabrica", 0.5, 0.5, 2.0),
        // O preset de antes (Contrast 0,5: 6 cortes no 1.º traço) e dois extremos (a textura só morde nos mínimos).
        ("antes_c50", 0.5, 0.5, 2.0),
        ("c00", 0.0, 0.5, 2.0),
        ("b100", 0.5, 1.0, 2.0),
        ("c25", 0.25, 0.5, 2.0),
        ("c15", 0.15, 0.5, 2.0),
        ("c25b60", 0.25, 0.6, 2.0),
        ("c25s4", 0.25, 0.5, 4.0),
    ];
    for (nome, contraste, brilho, alto) in variantes {
        let mut t = pincel_com("Knife Textured", true, |p| {
            if nome == "fabrica" {
                return;
            }
            let tex = p
                .parametros
                .brush
                .get_mut("texture")
                .expect("a textura do preset");
            tex["params"][0] = serde_json::json!(contraste);
            tex["params"][1] = serde_json::json!(brilho);
            tex["size"][1] = serde_json::json!(alto);
        });
        for k in 0..3 {
            traco(&mut t, k);
            let r = regua(&mut t);
            let f = foto(&t);
            let (claros, escuros) = fendas(&f, &r.miolo);
            let mut lum: Vec<f64> = r
                .miolo
                .iter()
                .map(|&(x, y)| {
                    let i = 3 * ((x - 1) + (y - 1) * TW as usize);
                    0.2126 * f64::from(f[i])
                        + 0.7152 * f64::from(f[i + 1])
                        + 0.0722 * f64::from(f[i + 2])
                })
                .collect();
            lum.sort_by(f64::total_cmp);
            let med = lum.get(lum.len() / 2).copied().unwrap_or(0.0);
            let p02 = lum.get(lum.len() / 50).copied().unwrap_or(0.0);
            println!(
                "[cortes] {nome:<8} {} traço(s): cortes {} · claros {claros:.4} · escuros {escuros:.4} · lum med {med:.1} p02 {p02:.1} · tmax {:.2} · degrau máx {:.2}",
                k + 1,
                r.cortes,
                r.tmax,
                r.max
            );
            grava(&dir, &format!("cortes_{nome}_{}", k + 1), &f);
        }
    }
}
