//! **A borda do S** (W1c) — a cena do oráculo `fixtures/rebelle_gotas/s_*.png` a 1:1 no nosso motor: a
//! tela 1000×640, o S de `ferramentas/gotas.py --forma s` (um ponto a cada 3 px de arco), o Wet Round a
//! 100 px, água 1,0, 30 s de água.
//!
//! As variantes da lei correm em [`super::sonda_lei_das_gotas`]; aqui ficam a cena, a régua e os portões
//! (`PH2D_SONDA_OUT` grava as quatro cenas em PPM).

use super::gotas_pela_lei_do_rebelle::inclina;
use super::sonda_gotas_do_dono::gesto;
use ph2d_brush_presets::{Biblioteca, Preset, Volumes};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::RasterEditTool;
use ph2d_tool_painter::{PaintMedia, PainterTool};
use ph2d_wet_paint::drips::DripPaper;

pub const W: u32 = 1000;
pub const H: u32 = 640;

/// O S do oráculo: `y = 180 − 85·sin θ`, `θ ∈ [−π/2, 2π + π/4]`, x 100 → 900, reamostrado a 3 px de arco.
pub fn o_s() -> Vec<[f32; 2]> {
    let (x0, x1, yc, amp) = (100.0f64, 900.0, 180.0, 85.0);
    let (t0, t1) = (
        -std::f64::consts::FRAC_PI_2,
        2.0 * std::f64::consts::PI + std::f64::consts::FRAC_PI_4,
    );
    let fino: Vec<(f64, f64)> = (0..=8000)
        .map(|k| {
            let u = f64::from(k) / 8000.0;
            (x0 + (x1 - x0) * u, yc - amp * (t0 + (t1 - t0) * u).sin())
        })
        .collect();
    let mut pts = vec![[fino[0].0 as f32, fino[0].1 as f32]];
    let mut acc = 0.0;
    for w in fino.windows(2) {
        acc += (w[1].0 - w[0].0).hypot(w[1].1 - w[0].1);
        if acc >= 3.0 {
            pts.push([w[1].0 as f32, w[1].1 as f32]);
            acc = 0.0;
        }
    }
    pts
}

/// A cena: a inclinação `anel` (`None` = direita), as gotas do papel, `ajusta` antes do traço e
/// `preset` sobre o Wet Round de fábrica.
pub fn cena(
    anel: Option<u8>,
    gotas: DripPaper,
    ajusta: &dyn Fn(&mut PainterTool),
    preset: &dyn Fn(&mut Preset),
    segundos: u32,
) -> Vec<u8> {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let b = Biblioteca::carregar(&raiz, None);
    let mut p = b
        .entradas
        .iter()
        .find(|e| e.preset.parametros.name == "Wet Round")
        .expect("o pincel de fábrica")
        .preset
        .clone();
    preset(&mut p);
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (W * H * 4) as usize], W, H);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    inclina(&mut t, anel);
    t.set_gotas(gotas);
    assert!(t.wet_preset_aplicar(&p).is_empty());
    t.wet_preset_aplicar_volumes(&Volumes {
        size_px: Some(100.0),
        water: Some(1.0),
        ..Volumes::default()
    });
    t.set_brush_color_srgb8([23, 101, 179]);
    ajusta(&mut t);
    gesto(&mut t, &o_s());
    for _ in 0..segundos * 60 {
        t.on_tick(1000.0 / 60.0);
    }
    t.run_full().0
}

pub fn grava(out: &str, nome: &str, px: &[u8]) {
    let mut f = format!("P6\n{W} {H}\n255\n").into_bytes();
    for p in px.as_chunks::<4>().0 {
        f.extend_from_slice(&p[..3]);
    }
    std::fs::write(format!("{out}/{nome}.ppm"), f).unwrap();
}

// ── A régua da borda (a de `ferramentas/borda.py`, a mesma conta sobre o oráculo e o nosso) ─────────
//
// Tudo em proporção a `b0`, o escuro do corpo do MESMO motor com a tela direita (o Rebelle ~102, o nosso
// Wet Round ~213): os dois pigmentam diferente, a FORMA da borda é o que se compara. A descida do S
// (x 440–632, declive 0,72–0,75) é a janela do README das fixtures.

const DESCIDA: std::ops::Range<usize> = 440..633;

fn escuro(px: &[u8], x: usize, y: usize) -> f64 {
    let i = (y * W as usize + x) * 4;
    f64::from(255 - px[i..i + 3].iter().copied().min().unwrap_or(255))
}

/// O eixo do S na coluna `x`.
fn eixo(x: f64) -> f64 {
    let (t0, t1) = (
        -std::f64::consts::FRAC_PI_2,
        2.0 * std::f64::consts::PI + std::f64::consts::FRAC_PI_4,
    );
    180.0 - 85.0 * (t0 + (t1 - t0) * (x - 100.0) / 800.0).sin()
}

/// A largura da corrida horizontal de tinta (`> t`) que contém `(x, y)`.
fn corrida(px: &[u8], x: usize, y: usize, t: f64) -> usize {
    if escuro(px, x, y) <= t {
        return 0;
    }
    let (mut l, mut r) = (x, x);
    while l > 0 && escuro(px, l - 1, y) > t {
        l -= 1;
    }
    while r + 1 < W as usize && escuro(px, r + 1, y) > t {
        r += 1;
    }
    r - l + 1
}

/// A orla de baixo da coluna `x`: a linha de tinta mais baixa (corrida > 8 px: não um fio) a até 75 px
/// abaixo do eixo, sem um buraco de mais de 3 linhas desde o eixo.
fn orla(px: &[u8], x: usize, t: f64) -> usize {
    let c = eixo(x as f64).round() as usize;
    let (mut yb, mut buraco) = (c, 0);
    for y in c..(c + 75).min(H as usize) {
        if escuro(px, x, y) > t && corrida(px, x, y, t) > 8 {
            (yb, buraco) = (y, 0);
        } else {
            buraco += 1;
            if buraco > 3 {
                break;
            }
        }
    }
    yb
}

fn mediana(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    let n = v.len();
    if n == 0 {
        return 0.0;
    }
    if n % 2 == 1 {
        v[n / 2]
    } else {
        0.5 * (v[n / 2 - 1] + v[n / 2])
    }
}

/// Uma coluna da descida: a orla, o corpo (mediana a ±10 do eixo), o pico (os 5 px da orla), a rampa
/// 90 → 10 % entre o corpo e o pico, a faixa chapada (linhas contíguas desde a orla com escuro ≥ `b0`) e
/// se há um DEGRAU (um salto ≥ 0,4·`b0` em 2 px acima dos 4 px da linha da orla — o topo da faixa).
struct Coluna {
    yb: usize,
    corpo: f64,
    rampa: Option<usize>,
    faixa: usize,
    degrau: bool,
}

fn coluna(px: &[u8], x: usize, b0: f64) -> Coluna {
    let yb = orla(px, x, 0.12 * b0);
    let c = eixo(x as f64).round() as usize;
    let corpo = mediana((c - 10..=c + 10).map(|y| escuro(px, x, y)).collect());
    let pico = (yb - 4..=yb).map(|y| escuro(px, x, y)).fold(0.0, f64::max);
    let perfil: Vec<f64> = (c - 10..=yb).map(|y| escuro(px, x, y)).collect();
    let k: Vec<f64> = (0..perfil.len())
        .map(|i| {
            let (a, b) = (i.saturating_sub(1), (i + 1).min(perfil.len() - 1));
            perfil[a..=b].iter().sum::<f64>() / (b - a + 1) as f64
        })
        .collect();
    let amp = pico - corpo;
    let rampa = (amp >= 0.15 * b0 && k.len() >= 3).then(|| {
        let (n90, n10) = (corpo + 0.9 * amp, corpo + 0.1 * amp);
        let mut i = k.len() - 2;
        while i > 0 && k[i] >= n90 {
            i -= 1;
        }
        let mut j = i;
        while j > 0 && k[j] >= n10 {
            j -= 1;
        }
        i - j
    });
    let faixa = (c - 10..=yb)
        .rev()
        .take_while(|&y| escuro(px, x, y) >= b0)
        .count();
    let z: Vec<f64> = (c - 10..yb.saturating_sub(4).max(c - 9))
        .map(|y| escuro(px, x, y))
        .collect();
    let degrau = z.len() > 2 && z.windows(3).any(|w| w[2] - w[0] >= 0.4 * b0);
    Coluna {
        yb,
        corpo,
        rampa,
        faixa,
        degrau,
    }
}

/// O escuro do corpo com a tela direita (o `b0` das réguas).
pub fn corpo_direito(px: &[u8]) -> f64 {
    mediana(DESCIDA.map(|x| coluna(px, x, 213.0).corpo).collect())
}

/// A borda da descida do S.
#[derive(Debug)]
pub struct Borda {
    /// O corpo inclinado ÷ o direito.
    pub corpo: f64,
    /// A rampa 90 → 10 % (px, mediana).
    pub rampa: f64,
    /// A faixa chapada (px, mediana).
    pub faixa: f64,
    /// As colunas com degrau (%).
    pub degraus: f64,
    /// A orla: o maior salto entre colunas (px) e o resíduo rms de uma cúbica.
    pub salto: f64,
    pub rms: f64,
}

pub fn mede_borda(px: &[u8], b0: f64) -> Borda {
    let cols: Vec<Coluna> = DESCIDA.map(|x| coluna(px, x, b0)).collect();
    let yb: Vec<f64> = cols.iter().map(|c| c.yb as f64).collect();
    let xs: Vec<f64> = DESCIDA.map(|x| (x as f64 - 536.0) / 100.0).collect();
    Borda {
        corpo: mediana(cols.iter().map(|c| c.corpo).collect()) / b0,
        rampa: mediana(
            cols.iter()
                .filter_map(|c| c.rampa.map(|r| r as f64))
                .collect(),
        ),
        faixa: mediana(cols.iter().map(|c| c.faixa as f64).collect()),
        degraus: 100.0 * cols.iter().filter(|c| c.degrau).count() as f64 / cols.len() as f64,
        salto: yb
            .windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .fold(0.0, f64::max),
        rms: residuo_cubico(&xs, &yb),
    }
}

/// O resíduo rms da cúbica de mínimos quadrados (as equações normais, 4×4).
fn residuo_cubico(xs: &[f64], ys: &[f64]) -> f64 {
    let mut a = [[0.0f64; 5]; 4];
    for (&x, &y) in xs.iter().zip(ys) {
        let p = [1.0, x, x * x, x * x * x];
        for i in 0..4 {
            for j in 0..4 {
                a[i][j] += p[i] * p[j];
            }
            a[i][4] += p[i] * y;
        }
    }
    for c in 0..4 {
        let piv = (c..4)
            .max_by(|&i, &j| a[i][c].abs().total_cmp(&a[j][c].abs()))
            .unwrap();
        a.swap(c, piv);
        let pivo = a[c];
        for (r, linha) in a.iter_mut().enumerate() {
            if r != c {
                let f = linha[c] / pivo[c];
                for (v, p) in linha.iter_mut().zip(pivo).skip(c) {
                    *v -= f * p;
                }
            }
        }
    }
    let coef: Vec<f64> = (0..4).map(|i| a[i][4] / a[i][i]).collect();
    let se: f64 = xs
        .iter()
        .zip(ys)
        .map(|(&x, &y)| {
            let f = coef[0] + coef[1] * x + coef[2] * x * x + coef[3] * x * x * x;
            (y - f).powi(2)
        })
        .sum();
    (se / xs.len() as f64).sqrt()
}

/// Um fio que cruza a linha `y = 330` (abaixo de todo o S): a largura a meia altura,
/// o comprimento desde a orla (px), a inclinação do S onde ele nasceu e o funil (o escuro da raiz, 2–12
/// px abaixo da orla, ÷ a mediana do fio em 330–380).
#[derive(Debug)]
pub struct Fio {
    pub largura: usize,
    pub comprimento: usize,
    pub declive: f64,
    pub funil: f64,
}

pub fn fios_do_s(px: &[u8], b0: f64) -> Vec<Fio> {
    const Y: usize = 330;
    let t = 0.12 * b0;
    let mut out = Vec::new();
    let mut x = 0;
    while x < W as usize {
        if escuro(px, x, Y) <= t {
            x += 1;
            continue;
        }
        let a = x;
        while x < W as usize && escuro(px, x, Y) > t {
            x += 1;
        }
        let seg: Vec<f64> = (a..x).map(|k| escuro(px, k, Y)).collect();
        let topo = seg.iter().copied().fold(0.0, f64::max);
        let c = a + seg.iter().position(|&v| v == topo).unwrap_or(0);
        let largura = seg.iter().filter(|&&v| v >= topo / 2.0).count();
        let max_em = |y: usize, r: usize| {
            (c.saturating_sub(r)..=(c + r).min(W as usize - 1))
                .map(|k| escuro(px, k, y))
                .fold(0.0, f64::max)
        };
        let mut fim = Y;
        while fim + 1 < H as usize && max_em(fim + 1, 2) > t {
            fim += 1;
        }
        let yb = orla(px, c, t);
        let raiz = (yb + 2..=yb + 12).map(|y| max_em(y, 2)).fold(0.0, f64::max);
        let fio = mediana((Y..Y + 50).map(|y| max_em(y, 1)).collect());
        out.push(Fio {
            largura,
            comprimento: fim.saturating_sub(yb),
            declive: (eixo(c as f64 + 1.0) - eixo(c as f64 - 1.0)) / 2.0,
            funil: raiz / fio.max(1.0),
        });
        x += 1;
    }
    out
}

// ── Os portões ───────────────────────────────────────────────────────────────────────────────────

/// As cenas do S partilhadas pelos portões (uma vez por binário): a tela direita e a inclinada no
/// máximo, nas grades 1 e 2.
struct Cenas {
    direita: Vec<u8>,
    lei: Vec<u8>,
    direita_2: Vec<u8>,
    lei_2: Vec<u8>,
}

fn cenas() -> &'static Cenas {
    static C: std::sync::OnceLock<Cenas> = std::sync::OnceLock::new();
    C.get_or_init(|| {
        let s = |anel, ratio: f64| {
            cena(
                anel,
                DripPaper::DEFAULT,
                &|t| {
                    if ratio > 1.0 {
                        t.set_wet_grid_ratio(ratio);
                    }
                },
                &|_| {},
                30,
            )
        };
        let c = Cenas {
            direita: s(None, 1.0),
            lei: s(Some(8), 1.0),
            direita_2: s(None, 2.0),
            lei_2: s(Some(8), 2.0),
        };
        if let Ok(out) = std::env::var("PH2D_SONDA_OUT") {
            for (n, px) in [
                ("borda_s_direita", &c.direita),
                ("borda_s_lei", &c.lei),
                ("borda_s_direita_2", &c.direita_2),
                ("borda_s_lei_2", &c.lei_2),
            ] {
                grava(&out, n, px);
            }
        }
        c
    })
}

/// ⭐ **L1-1 — a borda do S como a do Rebelle** (auditoria de 08/10, o oráculo `s_inclinado.png` a 1:1):
/// o corpo clareia, o escuro sobe em RAMPA até uma linha fina na orla, sem a faixa chapada cujo topo
/// era a escada, e a orla continua lisa. Barras no vale entre o nosso de antes da W1c e o Rebelle
/// (descida do S, em proporção ao corpo com a tela direita):
///
/// | | degraus | faixa | rampa | corpo | salto / rms |
/// |---|---|---|---|---|---|
/// | antes (W1b) | 84–88 % | 11–12 px | 14 px | 0,43 | 5 / 0,9 |
/// | Rebelle | 8 % | 0 | 28 | 0,25 | 4 / 1,1 |
/// | barra | ≤ 20 % | ≤ 6 | ≥ 18 | ≤ 0,55 | ≤ 8 / ≤ 1,6 |
///
/// Mutação que sangra: a gota sem o canal (`recede = hold`, `drain_after = false`).
#[test]
fn a_borda_do_s_e_a_do_rebelle() {
    let c = cenas();
    let b0 = corpo_direito(&c.direita);
    let b = mede_borda(&c.lei, b0);
    println!("[borda] b0 {b0:.0} · {b:.2?}");
    assert!(
        b.degraus <= 20.0,
        "{:.1} % de colunas com degrau (a escada)",
        b.degraus
    );
    assert!(b.faixa <= 6.0, "faixa chapada de {} px na orla", b.faixa);
    assert!(b.rampa >= 18.0, "rampa de {} px (o Rebelle 28)", b.rampa);
    assert!(
        b.corpo <= 0.55,
        "o corpo não clareia ({:.2} do direito)",
        b.corpo
    );
    assert!(b.salto <= 8.0 && b.rms <= 1.6, "orla irregular: {b:.2?}");
}

/// ⭐ **L1-3 — as gotas saem de TODA a orla** (o Rebelle: 10–11 no S, em encostas, no arco e no vale; 3
/// px a meia altura; 228–430 px; um funil escuro na raiz, ~2× o fio). Antes da W1c nasciam só no ponto
/// baixo (2 fios no S, nenhum em encosta, 1–2 px, sem funil: 1,1).
#[test]
fn as_gotas_saem_de_toda_a_orla_do_s() {
    let c = cenas();
    let b0 = corpo_direito(&c.direita);
    let f = fios_do_s(&c.lei, b0);
    println!("[fios] {f:?}");
    assert!(
        (8..=14).contains(&f.len()),
        "{} fios no S (Rebelle 10–11)",
        f.len()
    );
    let encosta = f.iter().filter(|f| f.declive.abs() > 0.4).count();
    assert!(encosta >= 3, "{encosta} fios em encostas (Rebelle 6–8)");
    let larg = mediana(f.iter().map(|f| f.largura as f64).collect());
    assert!(
        (3.0..=4.0).contains(&larg),
        "fios de {larg} px a meia altura (Rebelle 3–4)"
    );
    assert!(
        f.iter().all(|f| (2..=6).contains(&f.largura)),
        "um fio fora de 2–6 px: {f:?}"
    );
    let comp = mediana(f.iter().map(|f| f.comprimento as f64).collect());
    assert!(
        (228.0..=430.0).contains(&comp),
        "fios de {comp} px (Rebelle 228–430)"
    );
    let funil = mediana(f.iter().map(|f| f.funil).collect());
    assert!(
        funil >= 1.5,
        "a raiz do fio tem {funil:.2} do escuro dele (Rebelle ~2)"
    );
}

/// **L1-5 — a lei em px, não em células**: com a grade a 2 (cada célula 2 px) os fios correm o mesmo em
/// px que na grade 1 (antes da W1c corriam o dobro: as constantes em px eram lidas como células), e a
/// borda passa as mesmas barras contra a SUA tela direita.
#[test]
fn a_grade_2_da_os_mesmos_fios_em_px() {
    let c = cenas();
    let (b1, b2) = (corpo_direito(&c.direita), corpo_direito(&c.direita_2));
    let comp = |px: &[u8], b0| {
        mediana(
            fios_do_s(px, b0)
                .iter()
                .map(|f| f.comprimento as f64)
                .collect(),
        )
    };
    let (l1, l2) = (comp(&c.lei, b1), comp(&c.lei_2, b2));
    let b = mede_borda(&c.lei_2, b2);
    println!("[grade 2] fios {l1} px → {l2} px · {b:.2?}");
    assert!(
        (0.85..=1.15).contains(&(l2 / l1)),
        "grade 2: fios de {l2} px contra {l1} na grade 1"
    );
    assert!(
        b.degraus <= 20.0 && b.faixa <= 6.0,
        "grade 2: a borda {b:.2?}"
    );
}
