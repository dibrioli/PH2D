//! **F0 — a régua-mãe da refatoração do Wet Paint** (`docs/Painter/Arquitetura/F0_varredura.md`; dono 31/10,
//! `smoke_31_10_oleo/dono_6.png`). Para cada pincel de óleo de fábrica: Pigment × Water (as faixas do painel) ×
//! pressão × N passadas; azul assenta (pausa longa), amarelo por cima meio deslocado (sobre o azul e sobre o papel),
//! papel creme. Por célula: distância à curva K-M azul–amarelo (máx e p99), manchas de concentração (mais escuras
//! que o azul sozinho), ferrugem (matiz fora do segmento azul–verde–amarelo), furos e rugosidade. Grava TSV e PNG.
//! `PH2D_SONDA_OUT=<dir> bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib sonda_varredura_oleo -- --ignored --nocapture`
use super::sonda_oleo_grumos::{alturas, mede};
use super::*;
use crate::PaintMedia;
use ph2d_brush_presets::Biblioteca;
use ph2d_editor_core::tool::{
    CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase, RasterEditTool,
};
use ph2d_wet_paint::colorops::{ks_of_srgb255, reflectance_of_ks, srgb255_of_linear};

const W: u32 = 360;
const H: u32 = 240;
const AZ: [u8; 3] = [40, 90, 220];
const AM: [u8; 3] = [245, 205, 25];
const CREME: [f64; 3] = [236.0, 226.0, 206.0];
/// O recorte gravado por célula (px): x 30..330, y 70..200.
const RX: (usize, usize) = (30, 330);
const RY: (usize, usize) = (70, 200);

/// O Pigment (0..2000, `PIGMENT_MAX` do painel) e o Water (0..1, `volume_row` do painel).
const PIGMENTOS: [f64; 3] = [40.0, 600.0, 2000.0];
const AGUAS: [f64; 3] = [0.0, 0.5, 1.0];
const PRESSOES: [f32; 3] = [0.2, 0.6, 1.0];
const NS: [usize; 2] = [1, 3];

pub(super) const PINCEIS: [&str; 15] = [
    "Oil/Flat/Thick Impasto",
    "Oil/Flat/Filbert",
    "Oil/Flat/Filbert Dry",
    "Oil/Flat/Flat Bristle",
    "Oil/Flat/Tilted Shade",
    "Oil/Dabber/Dabber",
    "Oil/Dabber/Dabber 4",
    "Oil/Knife/Knife Smooth 3",
    "Oil/Knife/Knife Oily",
    "Oil/Knife/Knife Grainy",
    "Oil/Knife/Knife Dry",
    "Oil/Knife/Knife Textured",
    "Oil/Knife/Knife Stain",
    "Oil/Metallic/Metallic Flat",
    "ExpressOil/Round/Express Oil",
];

fn preset(b: &Biblioteca, rel: &str) -> ph2d_brush_presets::Preset {
    let rel = format!("WetPainter/{rel}.ph2d-brush.png");
    b.entradas
        .iter()
        .find(|e| e.relativo == rel)
        .unwrap_or_else(|| panic!("preset {rel}"))
        .preset
        .clone()
}

fn traco(t: &mut PainterTool, a: [f32; 2], b: [f32; 2], pressure: f32, pausa: u32) {
    let p = |pos, phase| CanvasPointer {
        pos,
        pressure,
        tilt: [0.0, 0.0],
        phase,
    };
    let n = ((b[0] - a[0]).hypot(b[1] - a[1]) / 3.0).ceil() as u32;
    t.on_canvas_pointer(p(a, PointerPhase::Down));
    for k in 1..=n {
        let s = k as f32 / n as f32;
        t.on_canvas_pointer(p(
            [a[0] + (b[0] - a[0]) * s, a[1] + (b[1] - a[1]) * s],
            PointerPhase::Move,
        ));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(p(b, PointerPhase::Up));
    for _ in 0..pausa {
        t.on_tick(1000.0 / 60.0);
    }
}

/// A distância (níveis) de `x` à curva Kubelka–Munk entre `a` e `b` — a de `assenta/oleo_sobre_oleo_tests.rs`.
fn fora_da_mistura(x: [f64; 3], a: [f64; 3], b: [f64; 3]) -> f64 {
    (0..=200)
        .map(|i| {
            let c = f64::from(i) / 200.0;
            (0..3)
                .map(|k| {
                    let ks = (1.0 - c) * ks_of_srgb255(a[k]) + c * ks_of_srgb255(b[k]);
                    (x[k] - srgb255_of_linear(reflectance_of_ks(ks))).powi(2)
                })
                .sum::<f64>()
                .sqrt()
        })
        .fold(f64::INFINITY, f64::min)
}

fn lum(c: [f64; 3]) -> f64 {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

/// Matiz (graus) e saturação HSV.
fn matiz(c: [f64; 3]) -> (f64, f64) {
    let mx = c[0].max(c[1]).max(c[2]);
    let mn = c[0].min(c[1]).min(c[2]);
    let d = mx - mn;
    if d < 1e-9 || mx < 1e-9 {
        return (0.0, 0.0);
    }
    let h = if (mx - c[0]).abs() < 1e-9 {
        60.0 * ((c[1] - c[2]) / d).rem_euclid(6.0)
    } else if (mx - c[1]).abs() < 1e-9 {
        60.0 * ((c[2] - c[0]) / d + 2.0)
    } else {
        60.0 * ((c[0] - c[1]) / d + 4.0)
    };
    (h, d / mx)
}

/// Ferrugem: matiz fora do arco azul(~220°)–verde–amarelo(~49°) — vermelho, laranja, castanho, magenta —, com
/// saturação que não seja a do papel creme (0,13).
fn ferrugem(c: [f64; 3]) -> bool {
    let (h, s) = matiz(c);
    s > 0.2 && (h < 40.0 || h > 250.0)
}

fn sobre_creme(q: &[u8]) -> ([f64; 3], f64) {
    let a = f64::from(q[3]) / 255.0;
    (
        [0, 1, 2].map(|k| f64::from(q[k]) * a + CREME[k] * (1.0 - a)),
        a,
    )
}

/// As métricas de UM render (com luz ou plano) sobre o recorte.
#[derive(Clone, Copy, Default)]
pub(super) struct Leitura {
    pub km_max: f64,
    pub km_p99: f64,
    /// % dos píxeis cobertos (alfa ≥ 0,9) mais escuros que o azul sozinho (p50) − 6 níveis.
    pub manchas: f64,
    /// O mesmo, mas no azul SOZINHO antes do amarelo.
    pub manchas_azul: f64,
    pub ferrugem: f64,
    pub ref_azul: f64,
    /// Degraus de matiz no verde: saltos > 3° entre linhas vizinhas (perfil vertical, x 150..210, através das
    /// passadas) e entre colunas vizinhas (perfil ao longo do eixo do traço, na faixa y 130..146).
    pub degraus_y: usize,
    pub degraus_x: usize,
}

pub(super) struct Celula {
    pub luz: Leitura,
    pub plano: Leitura,
    /// % de papel à vista (alfa < 0,9) no miolo do azul (x 100..260, y 118..142).
    pub furos: f64,
    pub rug: f64,
    pub p99_altura: f64,
}

fn pct(v: &mut [f64], q: f64) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(f64::total_cmp);
    v[((v.len() - 1) as f64 * q).round() as usize]
}

/// O render PLANO: sem o Real Oil (a luz, o verniz, a sombra da crista) e com o Emboss a 0 — a cor K-M do
/// assentado. A próxima passada volta a aplicar o preset inteiro (`wet_preset_aplicar`), por isso não se repõe.
fn render_plano(t: &mut PainterTool) {
    use ph2d_editor_core::Tool;
    t.set_wet_oleo_real(None);
    t.handle_panel_event(PanelEvent::SetValue(
        crate::ids::wet_tuning_slider_id("emboss"),
        0.0,
    ));
    t.wet_recomposite_full();
}

/// Luminância de referência do azul sozinho (p50 dos cobertos) e % de manchas.
fn le_azul(t: &PainterTool) -> (f64, f64) {
    let mut la = vec![];
    for y in RY.0..RY.1 {
        for x in RX.0..RX.1 {
            let (c, a) = sobre_creme(&t.canvas_rgba[(x + y * W as usize) * 4..][..4]);
            if a >= 0.9 {
                la.push(lum(c));
            }
        }
    }
    let r = pct(&mut la.clone(), 0.5);
    let m = if la.is_empty() {
        0.0
    } else {
        100.0 * la.iter().filter(|l| **l < r - 6.0).count() as f64 / la.len() as f64
    };
    (r, m)
}

/// Conta saltos > 3° num perfil de matiz médio (posições sem verde ficam de fora e partem a contagem).
fn saltos(perfil: &[Option<f64>]) -> usize {
    perfil
        .windows(2)
        .filter(|w| matches!(w, [Some(a), Some(b)] if (a - b).abs() > 3.0))
        .count()
}

/// O verde: matiz entre 65° e 190° com saturação > 0,15 (nem o amarelo puro ~49°, nem o azul ~220°).
fn verde(c: [f64; 3]) -> Option<f64> {
    let (h, s) = matiz(c);
    (s > 0.15 && (65.0..190.0).contains(&h)).then_some(h)
}

fn le_tudo(
    t: &PainterTool,
    ref_azul: f64,
    manchas_azul: f64,
    px: Option<&mut Vec<[u8; 3]>>,
) -> Leitura {
    let (af, bf) = (AZ.map(f64::from), AM.map(f64::from));
    let (mut km, mut cob, mut tinta, mut esc, mut fer) = (vec![], 0usize, 0usize, 0usize, 0usize);
    let mut px = px;
    let at = |x: usize, y: usize| sobre_creme(&t.canvas_rgba[(x + y * W as usize) * 4..][..4]);
    for y in RY.0..RY.1 {
        for x in RX.0..RX.1 {
            let (c, a) = at(x, y);
            if let Some(v) = px.as_deref_mut() {
                v.push(c.map(|v| v.round().clamp(0.0, 255.0) as u8));
            }
            if a >= 0.3 {
                tinta += 1;
                if ferrugem(c) {
                    fer += 1;
                }
            }
            if a >= 0.9 {
                cob += 1;
                km.push(fora_da_mistura(c, af, bf));
                if lum(c) < ref_azul - 6.0 {
                    esc += 1;
                }
            }
        }
    }
    let media = |it: &mut dyn Iterator<Item = (usize, usize)>| {
        let v: Vec<f64> = it.filter_map(|(x, y)| verde(at(x, y).0)).collect();
        (v.len() >= 8).then(|| v.iter().sum::<f64>() / v.len() as f64)
    };
    let py: Vec<Option<f64>> = (RY.0..RY.1)
        .map(|y| media(&mut (150..210).map(|x| (x, y))))
        .collect();
    let pxp: Vec<Option<f64>> = (60..300)
        .map(|x| media(&mut (130..146).map(|y| (x, y))))
        .collect();
    let pc = |a: usize, b: usize| {
        if b == 0 {
            0.0
        } else {
            100.0 * a as f64 / b as f64
        }
    };
    Leitura {
        km_max: km.iter().copied().fold(0.0, f64::max),
        km_p99: pct(&mut km, 0.99),
        manchas: pc(esc, cob),
        manchas_azul,
        ferrugem: pc(fer, tinta),
        ref_azul,
        degraus_y: saltos(&py),
        degraus_x: saltos(&pxp),
    }
}

/// O cenário da célula para lá da grelha: papel, ordem das cores.
#[derive(Clone, Copy, Default)]
pub(super) struct Cena {
    /// Depth do substrato (`0` = o do app por omissão: nenhum papel escolhido; `> 0` arma o papel de fábrica,
    /// `PaperCold`, como o slider Depth do app).
    pub papel: f32,
    /// Azul → amarelo → azul (a 3.ª mão volta pela orla).
    pub aya: bool,
}

/// Uma célula da grelha. Grava o recorte com luz em `png` (e o plano ao lado, `…_plano.png`) se pedido.
#[allow(clippy::too_many_arguments)]
pub(super) fn celula(
    b: &Biblioteca,
    pincel: &str,
    pig: f64,
    agua: f64,
    pressao: f32,
    n: usize,
    pausa_assenta: u32,
    cena: Cena,
    png: Option<&std::path::Path>,
) -> Celula {
    let mut t = PainterTool::default();
    t.set_source(vec![0u8; (W * H * 4) as usize], W, H);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    t.set_semente_dos_tracos(0);
    if cena.papel > 0.0 {
        t.set_substrate_depth(cena.papel);
    }
    let p = preset(b, pincel);
    let passadas = |t: &mut PainterTool, cor: [u8; 3], y0: f32| {
        for k in 0..n {
            let erros = t.wet_preset_aplicar(&p);
            assert!(erros.is_empty(), "{erros:?}");
            for (id, v) in [
                (crate::ids::PAINTER_WETPAINT_PIGMENT, pig),
                (crate::ids::PAINTER_WETPAINT_WATER, agua),
            ] {
                assert!(t.route_brush_wetpaint_event(&PanelEvent::SetValue(id, v)));
            }
            t.set_brush_size_px(32.0);
            t.set_brush_color_srgb8(cor);
            let y = y0 + [0.0, 6.0, -6.0][k % 3];
            let (a, z) = ([50.0, y], [310.0, y + 4.0]);
            if k % 2 == 0 {
                traco(t, a, z, pressao, 60);
            } else {
                traco(t, z, a, pressao, 60);
            }
        }
    };
    passadas(&mut t, AZ, 120.0);
    for _ in 0..pausa_assenta {
        t.on_tick(1000.0 / 60.0);
    }
    t.wet_recomposite_full();
    let (ref_l, man_l) = le_azul(&t);
    render_plano(&mut t);
    let (ref_p, man_p) = le_azul(&t);
    passadas(&mut t, AM, 150.0);
    if cena.aya {
        for _ in 0..pausa_assenta {
            t.on_tick(1000.0 / 60.0);
        }
        passadas(&mut t, AZ, 132.0);
    }
    for _ in 0..120 {
        t.on_tick(1000.0 / 60.0);
    }
    t.wet_recomposite_full();
    let mut px_l = Vec::with_capacity((RX.1 - RX.0) * (RY.1 - RY.0));
    let luz = le_tudo(&t, ref_l, man_l, Some(&mut px_l));
    let (mut miolo, mut papel) = (0usize, 0usize);
    for y in 118..142 {
        for x in 100..260 {
            miolo += 1;
            if t.canvas_rgba[(x + y * W as usize) * 4 + 3] < 230 {
                papel += 1;
            }
        }
    }
    let (rug, p99_altura) = if t.wet_oleo_real().is_some() {
        let (h, m, gw, _gh, r) = alturas(&mut t);
        let md = mede(&h, &m, gw, r);
        (md.rug, md.p99)
    } else {
        (f64::NAN, f64::NAN)
    };
    render_plano(&mut t);
    let mut px_p = Vec::with_capacity(px_l.len());
    let plano = le_tudo(&t, ref_p, man_p, Some(&mut px_p));
    if let Some(f) = png {
        use crate::tool::paint::tests::look_watercolor_arc::{Canvas, write_png};
        let (w, h) = (RX.1 - RX.0, RY.1 - RY.0);
        write_png(f, &Canvas { w, h, px: px_l }).expect("png");
        let fp = f.with_file_name(format!(
            "{}_plano.png",
            f.file_stem().and_then(|s| s.to_str()).unwrap_or("c")
        ));
        write_png(&fp, &Canvas { w, h, px: px_p }).expect("png");
    }
    Celula {
        luz,
        plano,
        furos: if miolo == 0 {
            0.0
        } else {
            100.0 * papel as f64 / miolo as f64
        },
        rug,
        p99_altura,
    }
}

fn lista<T: std::str::FromStr>(var: &str, omissao: &[T]) -> Vec<T>
where
    T: Copy,
{
    std::env::var(var).ok().map_or_else(
        || omissao.to_vec(),
        |s| s.split(',').filter_map(|v| v.trim().parse().ok()).collect(),
    )
}

#[test]
#[ignore = "sonda — corre-se à mão no perfil smoke"]
fn sonda_varredura_oleo() {
    let b = Biblioteca::carregar(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes"),
        None,
    );
    let dir = std::path::PathBuf::from(
        std::env::var("PH2D_SONDA_OUT").unwrap_or_else(|_| "/tmp/f0_varredura".into()),
    );
    std::fs::create_dir_all(dir.join("celulas")).expect("pasta");
    let pinceis: Vec<String> = std::env::var("PH2D_SONDA_PINCEIS").map_or_else(
        |_| PINCEIS.iter().map(|s| (*s).to_string()).collect(),
        |s| s.split(',').map(str::to_string).collect(),
    );
    let pigs = lista("PH2D_SONDA_PIGMENTOS", &PIGMENTOS);
    let aguas = lista("PH2D_SONDA_AGUAS", &AGUAS);
    let pressoes = lista("PH2D_SONDA_PRESSOES", &PRESSOES);
    let ns = lista("PH2D_SONDA_NS", &NS);
    let assenta: u32 = std::env::var("PH2D_SONDA_ASSENTA")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(300);
    let cena = Cena {
        papel: std::env::var("PH2D_SONDA_PAPEL")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.0),
        aya: std::env::var("PH2D_SONDA_ORDEM").is_ok_and(|v| v == "aya"),
    };
    let tsv_path =
        dir.join(std::env::var("PH2D_SONDA_TSV").unwrap_or_else(|_| "varredura.tsv".into()));
    let mut tsv = String::from(
        "pincel\tpigment\twater\tpressao\tn\tkm_max\tkm_p99\tmanchas_pct\tmanchas_azul_pct\tferrugem_pct\tfuros_pct\trug\tp99_altura\tref_azul\tpng\
         \tkm_p99_plano\tkm_max_plano\tmanchas_plano\tmanchas_azul_plano\tferrugem_plano\tref_azul_plano\tdegraus_y\tdegraus_x\tdegraus_y_plano\tdegraus_x_plano\tpausa\tpapel\tordem\n",
    );
    for pincel in &pinceis {
        let slug = pincel
            .rsplit('/')
            .next()
            .unwrap_or(pincel)
            .replace(' ', "_")
            .to_lowercase();
        for &pig in &pigs {
            for &agua in &aguas {
                for &pr in &pressoes {
                    for &n in &ns {
                        let ordem = if cena.aya { "aya" } else { "ay" };
                        let nome = format!(
                            "{slug}_p{pig}_w{agua}_pr{pr}_n{n}_s{assenta}_pa{}_{ordem}.png",
                            cena.papel
                        );
                        let t0 = std::time::Instant::now();
                        let c = celula(
                            &b,
                            pincel,
                            pig,
                            agua,
                            pr,
                            n,
                            assenta,
                            cena,
                            Some(&dir.join("celulas").join(&nome)),
                        );
                        let (l, q) = (c.luz, c.plano);
                        let linha = format!(
                            "{pincel}\t{pig}\t{agua}\t{pr}\t{n}\t{:.1}\t{:.1}\t{:.2}\t{:.2}\t{:.2}\t{:.2}\t{:.3}\t{:.2}\t{:.1}\t{nome}\
                             \t{:.1}\t{:.1}\t{:.2}\t{:.2}\t{:.2}\t{:.1}\t{}\t{}\t{}\t{}\t{assenta}\t{}\t{ordem}\n",
                            l.km_max,
                            l.km_p99,
                            l.manchas,
                            l.manchas_azul,
                            l.ferrugem,
                            c.furos,
                            c.rug,
                            c.p99_altura,
                            l.ref_azul,
                            q.km_p99,
                            q.km_max,
                            q.manchas,
                            q.manchas_azul,
                            q.ferrugem,
                            q.ref_azul,
                            l.degraus_y,
                            l.degraus_x,
                            q.degraus_y,
                            q.degraus_x,
                            cena.papel
                        );
                        print!("[f0 {:>5.1}s] {linha}", t0.elapsed().as_secs_f64());
                        tsv.push_str(&linha);
                        std::fs::write(&tsv_path, &tsv).expect("tsv");
                    }
                }
            }
        }
    }
}
