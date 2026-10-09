//! **Auditoria da W1c** (09/10, smoke do dono: a orla de baixo PIXELADA e a raiz ARTIFICIAL da gota) — a
//! régua da COBERTURA do pixel da orla e da RAIZ do fio, a mesma conta de
//! `docs/Painter/NewFeaturesWetPaint/ferramentas/borda_cobertura.py` (lá corre sobre o oráculo).
//!
//! A régua da W1c ([`super::borda_do_s::mede_borda`]) mede a POSIÇÃO da orla: uma escada de 1 px perfeita
//! passa «lisa» nela. Esta mede se o pixel da orla é 0/1 (sem meio-tom), o degrau da orla dos dois lados
//! de cada fio e a largura do fio junto à raiz.
//!
//! Os portões são `#[ignore]` e VERMELHOS hoje: são da janela de cura
//! (`docs/Painter/NewFeaturesWetPaint/10_PROMPT_janela_W1d_cura.md`), que lhes tira o `ignore`.
//!
//! ```text
//! PH2D_SONDA_OUT=/tmp/x cargo test -p ph2d-tool-painter --profile smoke --test it sonda_auditoria_w1c -- --ignored --nocapture
//! ```

use super::borda_do_s::{
    DESCIDA, cena, corpo_direito, corrida, escuro, fios_do_s, grava, mede_borda, mediana, orla,
};
use ph2d_brush_presets::Volumes;
use ph2d_tool_painter::PainterTool;
use ph2d_wet_paint::drips::{DripLaw, DripPaper};

/// O azul das capturas do dono (a faixa dele chega a `(4, 127, 255)`).
pub const AZUL_DO_DONO: [u8; 3] = [0, 128, 255];
const AZUL_DA_BANCADA: [u8; 3] = [23, 101, 179];
const Y_FIOS: usize = 330;
const KS: [usize; 6] = [1, 2, 4, 8, 16, 32];

/// A orla de baixo da descida, pixel a pixel.
#[derive(Debug)]
pub struct Cobertura {
    /// Colunas em que a orla é 0/1: o último pixel ≥ 0,85·L e o de baixo ≤ 0,15·L (L = o pico nos 5 px
    /// de cima da orla), %.
    pub binaria: f64,
    /// Colunas com um pixel PARCIAL (0,2·L < d < 0,8·L) na passagem orla → papel, %.
    pub meio_tom: f64,
    /// O contraste do degrau, `d[yb] − d[yb+1]`, ÷ `b0` (mediana).
    pub salto: f64,
    /// Px de tinta (> 25) de 3 a 20 px abaixo da orla, por 100 colunas, fora dos fios.
    pub pontos: f64,
}

pub fn cobertura(px: &[u8], b0: f64, fios_x: &[usize]) -> Cobertura {
    let t = 0.12 * b0;
    let (mut bin, mut meio, mut salto, mut pontos) = (0usize, 0usize, Vec::new(), 0usize);
    for x in DESCIDA {
        let yb = orla(px, x, t);
        let l = (yb - 4..=yb).map(|y| escuro(px, x, y)).fold(0.0, f64::max);
        let (a, b) = (escuro(px, x, yb), escuro(px, x, yb + 1));
        bin += usize::from(a >= 0.85 * l && b <= 0.15 * l);
        meio += usize::from((yb - 1..=yb + 2).any(|y| {
            let d = escuro(px, x, y);
            0.2 * l < d && d < 0.8 * l
        }));
        salto.push((a - b) / b0);
        if fios_x.iter().all(|&f| f.abs_diff(x) > 4) {
            pontos += (yb + 3..yb + 21)
                .filter(|&y| escuro(px, x, y) > 25.0)
                .count();
        }
    }
    let n = DESCIDA.len() as f64;
    Cobertura {
        binaria: 100.0 * bin as f64 / n,
        meio_tom: 100.0 * meio as f64 / n,
        salto: mediana(salto),
        pontos: 100.0 * pontos as f64 / n,
    }
}

/// A largura a meia altura e o centro da corrida em volta de `c` (±6 colunas) na linha `y`.
fn meia_altura(px: &[u8], y: usize, c: usize) -> (usize, f64) {
    let lo = c.saturating_sub(6);
    let row: Vec<f64> = (lo..=c + 6).map(|x| escuro(px, x, y)).collect();
    let m = row.iter().copied().fold(0.0, f64::max);
    let k = row.iter().position(|&v| v == m).unwrap_or(0);
    let (mut a, mut b) = (k, k);
    while a > 0 && row[a - 1] >= m / 2.0 {
        a -= 1;
    }
    while b + 1 < row.len() && row[b + 1] >= m / 2.0 {
        b += 1;
    }
    (b - a + 1, lo as f64 + (a + b) as f64 / 2.0)
}

/// A recta de mínimos quadrados da orla nas colunas `xs`, avaliada em `c`.
fn orla_em(px: &[u8], t: f64, xs: std::ops::RangeInclusive<usize>, c: usize) -> f64 {
    let pts: Vec<(f64, f64)> = xs.map(|x| (x as f64, orla(px, x, t) as f64)).collect();
    let n = pts.len() as f64;
    let (sx, sy) = pts
        .iter()
        .fold((0.0, 0.0), |(a, b), &(x, y)| (a + x, b + y));
    let (mx, my) = (sx / n, sy / n);
    let (num, den) = pts.iter().fold((0.0, 0.0), |(a, b), &(x, y)| {
        (a + (x - mx) * (y - my), b + (x - mx) * (x - mx))
    });
    my + num / den * (c as f64 - mx)
}

/// O degrau da orla em `c`: |a recta da direita (c+4..c+20) − a da esquerda (c−20..c−4)| em `c`.
fn degrau_em(px: &[u8], t: f64, c: usize) -> f64 {
    (orla_em(px, t, c + 4..=c + 20, c) - orla_em(px, t, c - 20..=c - 4, c)).abs()
}

/// A raiz de um fio que cruza `y = 330`.
#[derive(Debug)]
pub struct Raiz {
    pub x: usize,
    pub raiz: usize,
    pub degrau: f64,
    /// A largura a meia altura `k` px abaixo da raiz (`KS`), quando acima de `y = 330`.
    pub larg: Vec<(usize, usize)>,
    /// A coluna do fio `k` px abaixo da raiz − a coluna em `y = 330`.
    pub desvio: Vec<(usize, f64)>,
}

pub fn raizes(px: &[u8], b0: f64) -> Vec<Raiz> {
    let t = 0.12 * b0;
    let w = super::borda_do_s::W as usize;
    let mut out = Vec::new();
    let mut x = 0;
    while x < w {
        if escuro(px, x, Y_FIOS) <= t {
            x += 1;
            continue;
        }
        let a = x;
        while x < w && escuro(px, x, Y_FIOS) > t {
            x += 1;
        }
        let topo = (a..x).map(|k| escuro(px, k, Y_FIOS)).fold(0.0, f64::max);
        let c = (a..x).find(|&k| escuro(px, k, Y_FIOS) == topo).unwrap_or(a);
        let (mut cc, mut yy) = (c, Y_FIOS);
        while yy > 1 {
            yy -= 1;
            let lo = cc.saturating_sub(2);
            let seg: Vec<f64> = (lo..=cc + 2).map(|k| escuro(px, k, yy)).collect();
            let m = seg.iter().copied().fold(0.0, f64::max);
            if m <= t {
                break;
            }
            cc = lo + seg.iter().position(|&v| v == m).unwrap_or(0);
            if corrida(px, cc, yy, t) > 8 {
                break;
            }
        }
        let c330 = meia_altura(px, Y_FIOS, c).1;
        let abaixo: Vec<(usize, (usize, f64))> = KS
            .iter()
            .filter(|&&k| yy + k < Y_FIOS)
            .map(|&k| (k, meia_altura(px, yy + k, cc)))
            .collect();
        if (20..w - 21).contains(&c) {
            out.push(Raiz {
                x: c,
                raiz: yy,
                degrau: degrau_em(px, t, c),
                larg: abaixo.iter().map(|&(k, (l, _))| (k, l)).collect(),
                desvio: abaixo.iter().map(|&(k, (_, m))| (k, m - c330)).collect(),
            });
        }
        x += 1;
    }
    out
}

/// O resumo da raiz: o degrau mediano nas raízes e nos pontos da descida longe dos fios (o controlo), e a
/// largura mediana a `k` px.
#[derive(Debug)]
pub struct Raizes {
    pub fios: usize,
    pub degrau_raiz: f64,
    pub degrau_ctrl: f64,
    pub larg: Vec<(usize, f64)>,
    pub desvio: Vec<(usize, f64)>,
}

pub fn resume_raizes(px: &[u8], b0: f64, rz: &[Raiz]) -> Raizes {
    let t = 0.12 * b0;
    let ctrl = (450..625)
        .step_by(12)
        .filter(|c| rz.iter().all(|r| r.x.abs_diff(*c) > 24))
        .map(|c| degrau_em(px, t, c))
        .collect();
    let por_k = |f: &dyn Fn(&Raiz, usize) -> Option<f64>| -> Vec<(usize, f64)> {
        KS.iter()
            .map(|&k| (k, mediana(rz.iter().filter_map(|r| f(r, k)).collect())))
            .collect()
    };
    Raizes {
        fios: rz.len(),
        degrau_raiz: mediana(rz.iter().map(|r| r.degrau).collect()),
        degrau_ctrl: mediana(ctrl),
        larg: por_k(&|r, k| r.larg.iter().find(|l| l.0 == k).map(|l| l.1 as f64)),
        desvio: por_k(&|r, k| r.desvio.iter().find(|l| l.0 == k).map(|l| l.1.abs())),
    }
}

/// Uma cena do S com a cor, o tamanho e a água (`None` = a tela direita), a lei e a grade.
#[derive(Clone, Copy)]
pub struct Cena {
    pub cor: [u8; 3],
    pub size_px: f64,
    pub agua: f64,
    pub gotas: DripPaper,
    pub lei: Option<DripLaw>,
    pub ratio: f64,
}

impl Cena {
    pub const BANCADA: Cena = Cena {
        cor: AZUL_DA_BANCADA,
        size_px: 100.0,
        agua: 1.0,
        gotas: DripPaper::DEFAULT,
        lei: None,
        ratio: 1.0,
    };

    pub fn pinta(&self, anel: Option<u8>) -> Vec<u8> {
        let c = *self;
        cena(
            anel,
            c.gotas,
            &move |t: &mut PainterTool| {
                t.wet_preset_aplicar_volumes(&Volumes {
                    size_px: Some(c.size_px),
                    water: Some(c.agua),
                    ..Volumes::default()
                });
                t.set_brush_color_srgb8(c.cor);
                if let Some(l) = c.lei {
                    t.set_wet_drip_law(l);
                }
                if c.ratio > 1.0 {
                    t.set_wet_grid_ratio(c.ratio);
                }
            },
            &|_| {},
            30,
        )
    }
}

/// A cena do dono mais provável (a captura: o cursor de ~60 px, o azul dele, água no máximo).
pub const DONO: Cena = Cena {
    cor: AZUL_DO_DONO,
    size_px: 60.0,
    ..Cena::BANCADA
};

/// Tudo o que a auditoria mede numa cena inclinada no máximo, em proporção ao `b0` da direita.
pub fn mede(nome: &str, lei: &[u8], b0: f64) -> (Cobertura, Raizes) {
    let rz = raizes(lei, b0);
    let xs: Vec<usize> = rz.iter().map(|r| r.x).collect();
    let cob = cobertura(lei, b0, &xs);
    let res = resume_raizes(lei, b0, &rz);
    let bd = mede_borda(lei, b0);
    let fios = fios_do_s(lei, b0);
    println!(
        "[aud] {nome:<16} b0 {b0:5.1} | degraus {:4.1} % faixa {:4.1} rampa {:4.1} corpo {:.2} | \
         binária {:5.1} % meio-tom {:5.1} % salto {:.2} pontos {:5.1} | fios {} degrau raiz {:.2} ctrl {:.2} | \
         larg {:?} | desvio {:?} | funil {:.2}",
        bd.degraus,
        bd.faixa,
        bd.rampa,
        bd.corpo,
        cob.binaria,
        cob.meio_tom,
        cob.salto,
        cob.pontos,
        res.fios,
        res.degrau_raiz,
        res.degrau_ctrl,
        res.larg,
        res.desvio,
        mediana(fios.iter().map(|f| f.funil).collect()),
    );
    for r in &rz {
        println!(
            "[raiz] {nome:<16} x {} y {} degrau {:.1} larg {:?}",
            r.x, r.raiz, r.degrau, r.larg
        );
    }
    (cob, res)
}

/// A rodada única da auditoria (§0.10): a bancada, as variantes da lei para a raiz (B1), a grade 2, o
/// escorrimento natural e três hipóteses da cena do dono (A3), no MESMO processo.
#[test]
#[ignore = "sonda: a rodada da auditoria da W1c; corre-se à mão"]
fn sonda_auditoria_w1c() {
    let out = std::env::var("PH2D_SONDA_OUT").unwrap_or_else(|_| "/tmp".into());
    let lei = DripLaw::DEFAULT;
    let sem = DripPaper {
        create: false,
        ..DripPaper::DEFAULT
    };
    let cenas: Vec<(&str, Cena)> = vec![
        ("banc", Cena::BANCADA),
        (
            "banc_natural",
            Cena {
                gotas: sem,
                ..Cena::BANCADA
            },
        ),
        (
            "banc_sem_dren",
            Cena {
                lei: Some(DripLaw {
                    drain_after: false,
                    ..lei
                }),
                ..Cena::BANCADA
            },
        ),
        (
            "banc_recede_alto",
            Cena {
                lei: Some(DripLaw {
                    recede: lei.hold,
                    ..lei
                }),
                ..Cena::BANCADA
            },
        ),
        (
            "banc_sem_os_dois",
            Cena {
                lei: Some(DripLaw {
                    recede: lei.hold,
                    drain_after: false,
                    ..lei
                }),
                ..Cena::BANCADA
            },
        ),
        (
            "banc_grade2",
            Cena {
                ratio: 2.0,
                ..Cena::BANCADA
            },
        ),
        ("dono60", DONO),
        ("dono60_agua07", Cena { agua: 0.7, ..DONO }),
        (
            "dono100",
            Cena {
                size_px: 100.0,
                ..DONO
            },
        ),
    ];
    // Com a tela direita a lei e as gotas não agem: um `b0` por pincel (cor, tamanho, água, grade).
    let mut b0s: Vec<(Cena, f64)> = Vec::new();
    for (nome, c) in cenas {
        let pincel = |o: &Cena| (o.cor, o.size_px, o.agua, o.ratio);
        let b0 = match b0s.iter().find(|b| pincel(&b.0) == pincel(&c)) {
            Some(b) => b.1,
            None => {
                let d = c.pinta(None);
                grava(&out, &format!("aud_{nome}_direita"), &d);
                let b = corpo_direito(&d);
                b0s.push((c, b));
                b
            }
        };
        let px = c.pinta(Some(8));
        grava(&out, &format!("aud_{nome}"), &px);
        mede(nome, &px, b0);
    }
}

// ── Os portões (VERMELHOS hoje; a janela de cura tira-lhes o `ignore`) ─────────────────────────────

fn par(c: Cena) -> (Vec<u8>, f64) {
    (c.pinta(Some(8)), corpo_direito(&c.pinta(None)))
}

/// A orla de baixo do S tem MEIO-TOM como a do Rebelle. Oráculo (`s_inclinado.png`, `_1`, `s_sem_gotas`,
/// `s_plano`): binária 5,7 · 19,7 · 21,8 · 0 % e meio-tom 89 · 67 · 59 · 98 %. Barras: binária ≤ 25 %,
/// meio-tom ≥ 55 %, na bancada e na cena do dono.
#[test]
#[ignore = "portão VERMELHO da auditoria da W1c — a janela de cura (10_PROMPT) tira o ignore"]
fn a_orla_do_s_tem_meio_tom_como_a_do_rebelle() {
    for (nome, c) in [("bancada", Cena::BANCADA), ("dono", DONO)] {
        let (px, b0) = par(c);
        let cob = cobertura(&px, b0, &[]);
        assert!(
            cob.binaria <= 25.0 && cob.meio_tom >= 55.0,
            "{nome}: a orla é 0/1 em {:.1} % das colunas e tem meio-tom em {:.1} % (Rebelle 5,7 / 89)",
            cob.binaria,
            cob.meio_tom
        );
    }
}

/// A orla não é um degrau de COR PURA: a faixa de ≥ 88 % da cor pura acima da orla (mediana) e o salto
/// orla → papel ÷ `b0`. Oráculo: faixa 0 px, salto 0,32–0,44 (com a tela direita o nosso dá 0,57). Hoje:
/// bancada 4 px / 1,04, o azul do dono a 100 px 4 px / 1,04 (a captura do dono: 4–5 px). Barras: faixa ≤ 1,
/// salto ≤ 0,6.
#[test]
#[ignore = "portão VERMELHO da auditoria da W1c — a janela de cura (10_PROMPT) tira o ignore"]
fn a_orla_nao_e_um_degrau_de_cor_pura() {
    let dono_100 = Cena {
        size_px: 100.0,
        ..DONO
    };
    for (nome, c) in [("bancada", Cena::BANCADA), ("dono a 100 px", dono_100)] {
        let (px, b0) = par(c);
        let pura = f64::from(255 - c.cor.iter().copied().min().unwrap_or(255));
        let faixa = faixa_pura(&px, b0, pura);
        let cob = cobertura(&px, b0, &[]);
        assert!(
            faixa <= 1.0 && cob.salto <= 0.6,
            "{nome}: faixa de cor pura {faixa:.1} px e salto {:.2}·B0 na orla (Rebelle 0 / 0,44)",
            cob.salto
        );
    }
}

/// A faixa de ≥ 88 % da cor pura (`pura` = `255 − min(cor)`) desde a orla para cima, mediana na descida.
pub fn faixa_pura(px: &[u8], b0: f64, pura: f64) -> f64 {
    mediana(
        DESCIDA
            .map(|x| {
                let yb = orla(px, x, 0.12 * b0);
                (0..=yb)
                    .rev()
                    .take_while(|&y| escuro(px, x, y) >= 0.88 * pura)
                    .count() as f64
            })
            .collect(),
    )
}

/// A raiz do fio não faz degrau na orla: a orla dos dois lados do fio, extrapolada à coluna dele, difere
/// como em qualquer ponto da descida. Oráculo: raiz 1,8 · 3,85 px contra o controlo 1,29 · 4,44. Barra:
/// raiz ≤ 1,5 × controlo + 1 px. GUARDA: verde hoje (0,74 contra 1,26) e nunca vista vermelha — o funil da
/// cura não pode criar o degrau.
#[test]
#[ignore = "guarda da auditoria da W1c (verde hoje) — a janela de cura tira o ignore e vê-a vermelha por mutação"]
fn a_raiz_do_fio_nao_faz_degrau_na_orla() {
    for (nome, c) in [("bancada", Cena::BANCADA), ("dono", DONO)] {
        let (px, b0) = par(c);
        let rz = raizes(&px, b0);
        let r = resume_raizes(&px, b0, &rz);
        assert!(
            r.degrau_raiz <= 1.5 * r.degrau_ctrl + 1.0,
            "{nome}: degrau na raiz {:.2} px contra {:.2} longe dos fios ({} fios)",
            r.degrau_raiz,
            r.degrau_ctrl,
            r.fios
        );
    }
}

/// O fio ALARGA para a orla (o funil do Rebelle): a largura a 1 px da raiz passa a de 16 px por ≥ 1,5 px.
/// Oráculo: 5 → 3 e 4,5 → 3.
#[test]
#[ignore = "portão VERMELHO da auditoria da W1c — a janela de cura (10_PROMPT) tira o ignore"]
fn o_fio_alarga_para_a_orla_como_o_funil_do_rebelle() {
    for (nome, c) in [("bancada", Cena::BANCADA), ("dono", DONO)] {
        let (px, b0) = par(c);
        let rz = raizes(&px, b0);
        let r = resume_raizes(&px, b0, &rz);
        let em = |k| r.larg.iter().find(|l| l.0 == k).map_or(0.0, |l| l.1);
        assert!(
            em(1) - em(16) >= 1.5,
            "{nome}: o fio tem {:.1} px a 1 px da raiz e {:.1} a 16 (Rebelle 5 → 3)",
            em(1),
            em(16)
        );
    }
}
