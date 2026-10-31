//! **Sonda da secagem (30/10) — fora do produto.** Os cortes AO BIT da secagem do Wet Paint simulados
//! sobre a cena de cobertura total de `nucleos_wash.rs` (4096×2560, `Lei::Cobre`). A cópia de
//! `drying_pass_jacobi_rows` → `dry_cell` → `assenta`/`levanta` (commit base `207e660793`) vem com um
//! `const C: u8` de cortes; `C = 0` é a cópia verbatim e tem de dar o MESMO hash que a crate.
//!
//! - bit 1 — `LADO`: a metade constante de `diferenca_de_pigmento` (o perfil e a norma da cor que não
//!   muda no Newton; `od` da mesma) içada para fora do avaliador; `d(sc,uc)` partilhado entre o alvo e o
//!   palpite do assentar.
//! - bit 2 — `CANAL`: a derivada numérica do Newton avalia só o canal que ela lê (o `d` e o `a_e`
//!   continuam inteiros — dependem dos três).
//! - bit 4 — `MEMO`: o `powf` do tapar reaproveitado quando a cobertura repete (`a_f == a_f2`: a massa
//!   que desceu não mudou a casa inteira da tabela de `alpha_of_mass`).
//!
//! Doc: `docs/Painter/49_realshader/SECAGEM_diagnostico.md`.
//! ```text
//! bash scripts/ph2d-run.sh env PH2D_RELEASE="mede velocidade" cargo build --profile smoke \
//!   -p ph2d-wet-paint --example sonda_secagem
//! target/smoke/examples/sonda_secagem
//! ```
#![allow(clippy::too_many_arguments, clippy::manual_clamp)]

use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
use std::time::Instant;

use ph2d_wet_paint::assenta::Ecra;
use ph2d_wet_paint::colorops::{
    ColorMix, OUTRA_TINTA_MINIMA, VELA_DA_MESMA_TINTA, VELA_POR_DIFERENCA, a_mesma_tinta_filtrada,
    diferenca_de_pigmento, velatura,
};
use ph2d_wet_paint::grid::{
    Grid, GridSnapshot, aparencia, oleo_celula, restore_grid, snapshot_grid, span_x_of,
};
use ph2d_wet_paint::jsmath::clamp01;
use ph2d_wet_paint::opacity::alpha_of_mass;
use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::par::Rows;
use ph2d_wet_paint::sim::Params;
use ph2d_wet_paint::tuning::Knob;
use rayon::prelude::*;

const W: usize = 4096;
const H: usize = 2560;
const FAIXA: f64 = 12.0;
const RAIO: f64 = 60.0;
const PRESSAO: f64 = 6.0;
const N_HASH: usize = 6;

// ---- contadores (só na corrida de contagem, série) ----
static CNT: [AtomicU64; 12] = [const { AtomicU64::new(0) }; 12];
const K_CEL: usize = 0; // células visitadas
const K_ASS: usize = 1; // assenta Cobre
const K_LEV: usize = 2; // levanta Cobre
const K_EVAL: usize = 3; // avaliações do ecrã dentro do resolve
const K_POW: usize = 4; // powf do tapar
const K_DIF: usize = 5; // diferenca_de_pigmento
const K_SAME: usize = 6; // assenta com a_s2 == a_s e a_f2 == a_f
const K_VELA: usize = 7; // células com vela > 0
const K_ITER: usize = 8; // voltas do Newton
const K_NOLOST: usize = 9; // lost == 0 ou s_mass == 0 (não assenta)
const K_SAMEL: usize = 10; // levanta com coberturas iguais
const K_FILT: usize = 11; // a_mesma_tinta_filtrada (6 ln)
#[inline(always)]
fn cnt<const N: bool>(k: usize, v: u64) {
    if N {
        CNT[k].fetch_add(v, Relaxed);
    }
}

fn faixa(e: &mut Engine, y: f64) {
    let ppf = 16.0;
    let (x0, x1) = (-64.0, W as f64 + 64.0);
    e.pointer_down(x0, y, None);
    let mut x = x0;
    while x < x1 {
        x = (x + ppf).min(x1);
        e.dispatch_pressure_dab(x, y, PRESSAO, RAIO);
    }
    e.pointer_up();
    while e.release_frame(x, y) {}
}

/// VERBATIM a montagem de `nucleos_wash.rs`.
fn wash() -> Engine {
    let mut e = Engine::new(W, H);
    e.sim.glaze = ph2d_wet_paint::colorops::VELATURA_MEDIDA;
    e.sim.glaze_filme = ph2d_wet_paint::colorops::VELATURA_DO_FILME;
    e.sim.pickup_assentado = 0.0;
    e.sim.lei = ph2d_wet_paint::assenta::Lei::Cobre;
    e.sim.esconde = ph2d_wet_paint::colorops::TINTA_ESCONDE;
    e.sliders.water = 1.0;
    e.sliders.size = 1.0;
    e.wet_canvas_now();
    let mut y = -32.0;
    while y < H as f64 + 32.0 {
        faixa(&mut e, y);
        y += FAIXA;
    }
    e
}

// ---------------------------------------------------------------------------
// A cópia das contas de `assenta.rs` (só a lei Cobre; as outras vão à crate)
// ---------------------------------------------------------------------------

const LADO: u8 = 1;
const CANAL: u8 = 2;
const MEMO: u8 = 4;
/// `levanta` com coberturas iguais devolve o palpite `x0` sem Newton (o alvo e o avaliador em `x0 = uc` são a
/// mesma conta ⇒ a 1.ª verificação sai).
const SALTA: u8 = 8;

/// A metade de `diferenca_de_pigmento` que só lê uma das cores (verbatim, as mesmas somas).
#[derive(Clone, Copy)]
struct Lado {
    p: [f64; 3],
    n: f64,
}
#[inline(always)]
fn lado(a: [f64; 3]) -> Lado {
    let p = a.map(|v| 255.0 - v);
    Lado {
        p,
        n: p.iter().map(|v| v * v).sum::<f64>(),
    }
}
#[inline(always)]
fn dif_lados(a: &Lado, b: &Lado) -> f64 {
    if a.n < 1e-9 || b.n < 1e-9 {
        return 0.0;
    }
    let dot = (0..3).map(|c| a.p[c] * b.p[c]).sum::<f64>();
    let cos2 = (dot * dot / (a.n * b.n)).min(1.0);
    (1.0 - cos2).sqrt()
}
#[inline(always)]
fn od(c: [f64; 3]) -> [f64; 3] {
    c.map(|v| 255.0 + (v.max(1.0) / 255.0).min(1.0).ln())
}

/// A cor que fica parada no avaliador, com os seus lados içados.
#[derive(Clone, Copy)]
struct Fixa {
    l: Lado,
    lod: Option<Lado>,
    /// `true`: a fixa é `uc` (assentar); `false`: é `sc` (re-molhar).
    uc: bool,
}
#[inline(always)]
fn fixa(c: [f64; 3], vela: f64, uc: bool) -> Fixa {
    Fixa {
        l: lado(c),
        lod: (vela > 0.0).then(|| lado(od(c))),
        uc,
    }
}

/// O `d` de `cor_e_alfa_tapando` (o tapar extra): `0` sob o Glaze se é a mesma tinta filtrada.
#[inline(always)]
fn d_pig<const C: u8, const N: bool>(sc: [f64; 3], uc: [f64; 3], vela: f64, f: &Fixa) -> f64 {
    if C & LADO == 0 {
        if vela > 0.0 {
            cnt::<N>(K_FILT, 1);
            if a_mesma_tinta_filtrada(sc, uc) {
                return 0.0;
            }
        }
        cnt::<N>(K_DIF, 1);
        return diferenca_de_pigmento(sc, uc);
    }
    let var = if f.uc { sc } else { uc };
    if vela > 0.0 {
        let lv = lado(od(var));
        let lf = f.lod.expect("lod com vela");
        let dd = if f.uc {
            dif_lados(&lv, &lf)
        } else {
            dif_lados(&lf, &lv)
        };
        if VELA_POR_DIFERENCA / (1.0 - VELA_DA_MESMA_TINTA) * dd < OUTRA_TINTA_MINIMA {
            return 0.0;
        }
    }
    let lv = lado(var);
    if f.uc {
        dif_lados(&lv, &f.l)
    } else {
        dif_lados(&f.l, &lv)
    }
}

#[inline(always)]
fn tapa<const N: bool>(esconde: f64, a_f: f64) -> f64 {
    cnt::<N>(K_POW, 1);
    1.0 - (1.0 - a_f).powf(esconde)
}

/// `sobre_branco_tapando` na lei Cobre SECA (`molhada = 1` ⇒ `p = 1`), canal `ch` ou os três.
/// `d` é o `d_pig` já calculado (ou `None`: calcular aqui).
#[inline(always)]
fn sobre_branco_c<const C: u8, const N: bool>(
    e: Ecra,
    sc: [f64; 3],
    uc: [f64; 3],
    a_s: f64,
    a_f: f64,
    h: &mut dyn FnMut() -> f64,
    d: Option<f64>,
    fx: &Fixa,
) -> [f64; 3] {
    let Ecra::Cobre {
        filme,
        molhada,
        vela,
        oleo,
        ..
    } = e
    else {
        unreachable!()
    };
    let (c, la) = 'k: {
        if a_f <= 0.0 {
            break 'k (sc, a_s);
        }
        let p = 1.0 + (molhada - 1.0) * a_f;
        let molha = |r: [f64; 3], la: f64| {
            let c = r.map(|v| ((v - 255.0 * (1.0 - la)) / la).clamp(0.0, 255.0));
            let la_w = if p == 1.0 {
                la
            } else {
                1.0 - (1.0 - la).powf(p)
            };
            (c, la_w)
        };
        let a_t = a_f;
        if a_s <= 0.0 {
            break 'k molha(uc.map(|v| 255.0 - a_t * (255.0 - v)), a_t);
        }
        let hh = h();
        let d = match d {
            Some(d) => d,
            None => d_pig::<C, N>(sc, uc, vela, fx),
        };
        let a_e = a_t + (hh - a_t).max(0.0) * d * (1.0 - oleo.max(0.0));
        let t_s = sc.map(|v| 255.0 - a_s * (255.0 - v));
        let c_eff = uc.map(|v| 255.0 - (a_t / a_e) * (255.0 - v));
        let mut r = [0, 1, 2].map(|k| t_s[k] * (1.0 - a_e) + c_eff[k] * a_e);
        cnt::<N>(K_DIF, u64::from(filme > 0.0));
        velatura(&mut r, a_e, filme, 1.0, t_s, c_eff);
        if vela > 0.0 {
            let m = vela;
            for k in 0..3 {
                let f = 255.0 - a_t * (255.0 - uc[k]);
                r[k] += m * (t_s[k] * f / 255.0 - r[k]);
            }
        }
        let r = [0, 1, 2].map(|k| r[k].min(t_s[k].max(uc[k])));
        let la = a_s + a_e * (1.0 - a_s);
        molha(r, la)
    };
    c.map(|v| 255.0 * (1.0 - la) + v * la)
}

/// Só o canal `ch` de [`sobre_branco_c`] (o corte `CANAL`) — a mesma aritmética por canal.
#[inline(always)]
fn sobre_branco_canal<const C: u8, const N: bool>(
    e: Ecra,
    sc: [f64; 3],
    uc: [f64; 3],
    a_s: f64,
    a_f: f64,
    h: f64,
    fx: &Fixa,
    ch: usize,
) -> f64 {
    let Ecra::Cobre {
        filme,
        molhada,
        vela,
        oleo,
        ..
    } = e
    else {
        unreachable!()
    };
    let p = 1.0 + (molhada - 1.0) * a_f;
    if a_f <= 0.0 || a_s <= 0.0 || p != 1.0 {
        return sobre_branco_c::<C, N>(e, sc, uc, a_s, a_f, &mut || h, None, fx)[ch];
    }
    let a_t = a_f;
    let d = d_pig::<C, N>(sc, uc, vela, fx);
    let a_e = a_t + (h - a_t).max(0.0) * d * (1.0 - oleo.max(0.0));
    let t_s = sc.map(|v| 255.0 - a_s * (255.0 - v));
    let c_eff = uc.map(|v| 255.0 - (a_t / a_e) * (255.0 - v));
    let mut r = t_s[ch] * (1.0 - a_e) + c_eff[ch] * a_e;
    // `velatura` (colorops.rs:206) num canal.
    if filme > 0.0 {
        cnt::<N>(K_DIF, 1);
        let k = a_e * filme * 1.0 * diferenca_de_pigmento(t_s, c_eff);
        r = (r - k * c_eff[ch] * (1.0 - t_s[ch] / 255.0)).max(0.0);
    }
    if vela > 0.0 {
        let f = 255.0 - a_t * (255.0 - uc[ch]);
        r += vela * (t_s[ch] * f / 255.0 - r);
    }
    let r = r.min(t_s[ch].max(uc[ch]));
    let la = a_s + a_e * (1.0 - a_s);
    let c = ((r - 255.0 * (1.0 - la)) / la).clamp(0.0, 255.0);
    255.0 * (1.0 - la) + c * la
}

/// VERBATIM `resolve` (assenta.rs:399), com o avaliador de canal opcional.
#[inline(always)]
fn resolve<const N: bool>(
    mut x: [f64; 3],
    alvo: [f64; 3],
    cego: f64,
    ecra: impl Fn([f64; 3]) -> [f64; 3],
    canal: Option<&dyn Fn([f64; 3], usize) -> f64>,
) -> [f64; 3] {
    const H: f64 = 0.5;
    cnt::<N>(K_EVAL, 1);
    let mut y = ecra(x);
    for _ in 0..4 {
        if (0..3).all(|c| (y[c] - alvo[c]).abs() < 0.01) {
            break;
        }
        cnt::<N>(K_ITER, 1);
        for c in 0..3 {
            let mut xh = x;
            xh[c] = if x[c] + H <= 255.0 {
                x[c] + H
            } else {
                x[c] - H
            };
            cnt::<N>(K_EVAL, 1);
            let yh = match canal {
                Some(f) => f(xh, c),
                None => ecra(xh)[c],
            };
            let d = (yh - y[c]) / (xh[c] - x[c]);
            if d.abs() < cego {
                continue;
            }
            let passo = ((y[c] - alvo[c]) / d).clamp(-48.0, 48.0);
            x[c] = (x[c] - passo).clamp(0.0, 255.0);
            cnt::<N>(K_EVAL, 1);
            y = ecra(x);
        }
    }
    x
}

const CEGO: f64 = 0.1;
const CEGO_COM_GLAZE: f64 = 0.01;

/// Memória de UM `powf` do tapar (corte `MEMO`).
#[derive(Default)]
struct Memo {
    a: f64,
    esc: f64,
    h: f64,
    ok: bool,
}
#[inline(always)]
fn tapa_m<const C: u8, const N: bool>(m: &mut Memo, esconde: f64, a: f64) -> f64 {
    if C & MEMO != 0 {
        if m.ok && m.a.to_bits() == a.to_bits() && m.esc.to_bits() == esconde.to_bits() {
            return m.h;
        }
        let h = tapa::<N>(esconde, a);
        *m = Memo {
            a,
            esc: esconde,
            h,
            ok: true,
        };
        return h;
    }
    tapa::<N>(esconde, a)
}

/// VERBATIM `assenta` (assenta.rs:437), ramo Cobre.
fn assenta_c<const C: u8, const N: bool>(
    e: Ecra,
    sc: [f64; 3],
    uc: [f64; 3],
    a_s: f64,
    a_f: f64,
    a_s2: f64,
    a_f2: f64,
    m: &mut Memo,
) -> [f64; 3] {
    let e = e.seco();
    let Ecra::Cobre { vela, esconde, .. } = e else {
        return ph2d_wet_paint::assenta::assenta(e, sc, uc, a_s, a_f, a_s2, a_f2);
    };
    cnt::<N>(K_ASS, 1);
    if N && a_s.to_bits() == a_s2.to_bits() && a_f.to_bits() == a_f2.to_bits() {
        cnt::<N>(K_SAME, 1);
    }
    let fx = if C & LADO != 0 {
        fixa(uc, vela, true)
    } else {
        Fixa {
            l: Lado {
                p: [0.0; 3],
                n: 0.0,
            },
            lod: None,
            uc: true,
        }
    };
    // `d(sc, uc)`: o alvo e o palpite leem a MESMA (corte LADO: uma vez).
    let mut d_scuc: Option<f64> = None;
    let mut dd = |sc: [f64; 3]| -> Option<f64> {
        if C & LADO == 0 {
            return None;
        }
        Some(*d_scuc.get_or_insert_with(|| d_pig::<C, N>(sc, uc, vela, &fx)))
    };
    let alvo = sobre_branco_c::<C, N>(
        e,
        sc,
        uc,
        a_s,
        a_f,
        &mut || tapa_m::<C, N>(m, esconde, a_f),
        if a_s > 0.0 && a_f > 0.0 { dd(sc) } else { None },
        &fx,
    );
    if a_s <= 0.0 && a_f2 <= 0.0 {
        return uc;
    }
    let a_dm = if a_s < 1.0 {
        (1.0 - (1.0 - a_s2) / (1.0 - a_s)).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let mut x0 = if a_s <= 0.0 || a_s2 <= 0.0 {
        uc
    } else {
        let r = sobre_branco_c::<C, N>(
            e,
            sc,
            uc,
            a_s,
            a_dm,
            &mut || tapa::<N>(esconde, a_dm),
            if a_dm > 0.0 { dd(sc) } else { None },
            &fx,
        );
        r.map(|v| (255.0 - (255.0 - v) / a_s2).clamp(0.0, 255.0))
    };
    if vela > 0.0 && a_s > 0.0 && a_s2 > 0.0 {
        let mm = vela;
        for k in 0..3 {
            let t_s = 255.0 - a_s * (255.0 - sc[k]);
            let (f, f2) = (
                255.0 - a_f * (255.0 - uc[k]),
                255.0 - a_f2 * (255.0 - uc[k]),
            );
            let t_s2 = if f2 > 0.0 { t_s * f / f2 } else { t_s };
            let produto = (255.0 - (255.0 - t_s2) / a_s2).clamp(0.0, 255.0);
            x0[k] += mm * (produto - x0[k]);
        }
    }
    let cego = if vela > 0.0 { CEGO_COM_GLAZE } else { CEGO };
    // ⚠️ O original calcula o `h2` aqui, sempre (assenta.rs:484) — o memo só o reaproveita.
    let h2 = tapa_m::<C, N>(m, esconde, a_f2);
    let canal =
        |x: [f64; 3], ch: usize| sobre_branco_canal::<C, N>(e, x, uc, a_s2, a_f2, h2, &fx, ch);
    resolve::<N>(
        x0,
        alvo,
        cego,
        |x| sobre_branco_c::<C, N>(e, x, uc, a_s2, a_f2, &mut || h2, None, &fx),
        if C & CANAL != 0 { Some(&canal) } else { None },
    )
}

/// VERBATIM `levanta` (assenta.rs:559) + `levanta_cobre` + `na_caixa`, ramo Cobre.
fn levanta_c<const C: u8, const N: bool>(
    e: Ecra,
    sc: [f64; 3],
    uc: [f64; 3],
    a_s: f64,
    a_f: f64,
    a_s2: f64,
    a_f2: f64,
    m: &mut Memo,
) -> [f64; 3] {
    let e = e.seco();
    let Ecra::Cobre { vela, esconde, .. } = e else {
        return ph2d_wet_paint::assenta::levanta(e, sc, uc, a_s, a_f, a_s2, a_f2);
    };
    cnt::<N>(K_LEV, 1);
    if N && a_s.to_bits() == a_s2.to_bits() && a_f.to_bits() == a_f2.to_bits() {
        cnt::<N>(K_SAMEL, 1);
    }
    let x = 'l: {
        if a_f2 <= 0.0 {
            break 'l uc;
        }
        let fx = if C & LADO != 0 {
            fixa(sc, vela, false)
        } else {
            Fixa {
                l: Lado {
                    p: [0.0; 3],
                    n: 0.0,
                },
                lod: None,
                uc: false,
            }
        };
        let x0 = if a_f <= 0.0 {
            sc
        } else {
            let w = ((a_f2 - a_f) / a_f2).clamp(0.0, 1.0);
            [0, 1, 2].map(|c| uc[c] + (sc[c] - uc[c]) * w)
        };
        if C & SALTA != 0
            && a_f > 0.0
            && a_f.to_bits() == a_f2.to_bits()
            && a_s.to_bits() == a_s2.to_bits()
        {
            break 'l x0;
        }
        let alvo = sobre_branco_c::<C, N>(
            e,
            sc,
            uc,
            a_s,
            a_f,
            &mut || tapa_m::<C, N>(m, esconde, a_f),
            None,
            &fx,
        );
        let h2 = tapa_m::<C, N>(m, esconde, a_f2);
        let canal =
            |x: [f64; 3], ch: usize| sobre_branco_canal::<C, N>(e, sc, x, a_s2, a_f2, h2, &fx, ch);
        resolve::<N>(
            x0,
            alvo,
            CEGO,
            |x| sobre_branco_c::<C, N>(e, sc, x, a_s2, a_f2, &mut || h2, None, &fx),
            if C & CANAL != 0 { Some(&canal) } else { None },
        )
    };
    if a_f <= 0.0 {
        return sc;
    }
    [0, 1, 2].map(|c| x[c].clamp(uc[c].min(sc[c]), uc[c].max(sc[c])))
}

// ---------------------------------------------------------------------------
// A cópia de `drying.rs` (Jacobi)
// ---------------------------------------------------------------------------

fn staining_multiplier(s: f64) -> f64 {
    if s < 0.5 {
        1.0 + (0.5 - s) * 14.0
    } else {
        (1.0 - s) * 2.0
    }
}

struct DryConst {
    retention: f64,
    edge_darkening: f64,
    base_evaporation: f64,
    ext_granulation: f64,
    ext_staining: f64,
    evap_base: f64,
    rewet_base: f64,
    ext_bypass: bool,
    mix: ColorMix,
    glaze: f64,
    ecra: Ecra,
}

impl DryConst {
    fn new(p: &Params, evap_base: f64, rewet_base: f64, ext_bypass: bool) -> Self {
        DryConst {
            retention: p.k(Knob::Retention),
            edge_darkening: p.k(Knob::EdgeDarkening),
            base_evaporation: p.k(Knob::BaseEvaporation),
            ext_granulation: p.k(Knob::ExtGranulation),
            ext_staining: p.k(Knob::ExtStaining),
            evap_base,
            rewet_base,
            ext_bypass,
            mix: p.mix,
            glaze: p.glaze,
            ecra: p.ecra,
        }
    }
}

/// VERBATIM `lift_settled` (drying.rs:41).
#[inline(always)]
fn lift_settled<const C: u8, const N: bool>(
    b: f64,
    susp: &mut f32,
    sett: &mut f32,
    susp_rgb: &mut [f32; 3],
    sett_rgb: [f32; 3],
    mix: ColorMix,
    glaze: f64,
    ecra: Ecra,
    out: &mut [f64; 3],
    m: &mut Memo,
) {
    let st = *sett as f64;
    let lift = st * b;
    if ecra != Ecra::Porto && lift > 0.0 {
        let sf = *susp as f64;
        *susp_rgb = levanta_c::<C, N>(
            ecra,
            sett_rgb.map(f64::from),
            susp_rgb.map(f64::from),
            alpha_of_mass(st),
            alpha_of_mass(sf),
            alpha_of_mass(st - lift),
            alpha_of_mass(sf + lift),
            m,
        )
        .map(|v| v as f32);
        *susp = (sf + lift) as f32;
        *sett = (st - lift) as f32;
        return;
    }
    let a_in = alpha_of_mass(st) * b;
    let a_susp = alpha_of_mass(*susp as f64);
    let u = a_susp * (1.0 - a_in);
    if u + a_in > 0.0 {
        let w = a_in / (u + a_in);
        mix.mix(
            susp_rgb[0] as f64,
            susp_rgb[1] as f64,
            susp_rgb[2] as f64,
            sett_rgb[0] as f64,
            sett_rgb[1] as f64,
            sett_rgb[2] as f64,
            w,
            out,
        );
        velatura(
            out,
            w,
            glaze,
            a_susp,
            susp_rgb.map(f64::from),
            sett_rgb.map(f64::from),
        );
        *susp_rgb = [out[0] as f32, out[1] as f32, out[2] as f32];
    } else {
        *susp_rgb = sett_rgb;
    }
    *susp = (*susp as f64 + lift) as f32;
    *sett = (st - lift) as f32;
}

/// VERBATIM `dry_cell` (drying.rs:145).
#[inline(always)]
fn dry_cell<const C: u8, const N: bool>(
    c: &DryConst,
    paper: f32,
    count: u32,
    vela: f32,
    vidrado: f32,
    oleo: oleo_celula::OleoCelula,
    film: &mut f32,
    susp: &mut f32,
    sett: &mut f32,
    susp_rgb: &mut [f32; 3],
    sett_rgb: &mut [f32; 3],
    out: &mut [f64; 3],
) {
    let mut memo = Memo::default();
    cnt::<N>(K_CEL, 1);
    cnt::<N>(K_VELA, u64::from(vela > 0.0));
    let film0 = *film;
    let f = film0 as f64;
    let s_mass = *susp as f64;
    let mut susp_c = *susp;
    let mut sett_c = *sett;
    let mut susp_rgb_c = *susp_rgb;
    let mut sett_rgb_c = *sett_rgb;
    let mix = c.mix;
    let ecra = c.ecra.com_vela(vela).com_vidrado(vidrado).com_oleo(oleo);
    let mut e = 1.0;
    if film0 > 0.0 && sett_c < 1000.0 {
        e = if count == 9 { 1.0 } else { count as f64 / 9.0 };
    }
    let mut new_film =
        f * c.retention - c.evap_base * ((1.0 - e) * c.edge_darkening + c.base_evaporation);
    if new_film < 0.0001 {
        new_film = 0.0;
    }
    let lost = if f > 0.0 {
        1.0 - clamp01(new_film / f)
    } else {
        1.0
    };
    let film_c: f32 = new_film as f32;
    if lost > 0.0 && s_mass > 0.0 {
        let mut dm = s_mass * lost;
        if !c.ext_bypass {
            let mut bias = 1.0 + c.ext_granulation * 0.6 * (0.5 - paper as f64) * 2.0;
            if bias < 0.3 {
                bias = 0.3;
            } else if bias > 1.7 {
                bias = 1.7;
            }
            dm *= bias;
            if dm > s_mass {
                dm = s_mass;
            }
        }
        let a_sett = alpha_of_mass(sett_c as f64);
        let a_in = alpha_of_mass(dm);
        if ecra != Ecra::Porto {
            sett_rgb_c = assenta_c::<C, N>(
                ecra,
                sett_rgb_c.map(f64::from),
                susp_rgb_c.map(f64::from),
                a_sett,
                alpha_of_mass(s_mass),
                alpha_of_mass(sett_c as f64 + dm),
                alpha_of_mass(s_mass - dm),
                &mut memo,
            )
            .map(|v| v as f32);
        } else if a_sett > 0.0 {
            let u = a_sett * (1.0 - a_in);
            let w = a_in / (u + a_in);
            mix.mix(
                sett_rgb_c[0] as f64,
                sett_rgb_c[1] as f64,
                sett_rgb_c[2] as f64,
                susp_rgb_c[0] as f64,
                susp_rgb_c[1] as f64,
                susp_rgb_c[2] as f64,
                w,
                out,
            );
            velatura(
                out,
                w,
                c.glaze,
                a_sett,
                sett_rgb_c.map(f64::from),
                susp_rgb_c.map(f64::from),
            );
            sett_rgb_c = [out[0] as f32, out[1] as f32, out[2] as f32];
        } else {
            sett_rgb_c = susp_rgb_c;
        }
        sett_c = (sett_c as f64 + dm) as f32;
        let rem = s_mass - dm;
        susp_c = if rem < 0.0 { 0.0 } else { rem as f32 };
    } else {
        cnt::<N>(K_NOLOST, 1);
    }
    let st = sett_c as f64;
    if film_c > 0.0 && st > 0.0 {
        let excess = (film_c as f64 - alpha_of_mass(susp_c as f64)).max(0.0);
        let mut b = c.rewet_base * (1.0 + excess * 50.0);
        if !c.ext_bypass {
            b *= staining_multiplier(c.ext_staining);
        }
        b = clamp01(b);
        if b > 0.0 {
            lift_settled::<C, N>(
                b,
                &mut susp_c,
                &mut sett_c,
                &mut susp_rgb_c,
                sett_rgb_c,
                mix,
                c.glaze,
                ecra,
                out,
                &mut memo,
            );
        }
    }
    *film = film_c;
    *susp = susp_c;
    *sett = sett_c;
    *susp_rgb = susp_rgb_c;
    *sett_rgb = sett_rgb_c;
}

const PIGMENT: f32 = 10.0;

/// O fator de borda (`edge_rows`, drying.rs:392) — a contagem 3×3 direta, o mesmo número.
fn edge_rows(g: &mut Grid, par: bool) {
    let s = g.s;
    let (by0, by1) = (g.by0, g.by1);
    let Grid {
        scratch,
        susp,
        row_lo,
        row_hi,
        spans_enabled,
        bx0,
        bx1,
        ..
    } = g;
    let (spans_enabled, bx0, bx1) = (*spans_enabled, *bx0, *bx1);
    let susp: &[f32] = susp;
    let band = by0 as usize * s..(by1 as usize + 1) * s;
    let f = |ri: usize, row: &mut [u8]| {
        let y = by0 + ri as i32;
        let (lo, hi) = span_x_of(row_lo, row_hi, spans_enabled, bx0, bx1, y);
        if lo > hi {
            return;
        }
        for x in lo..=hi {
            let i = x as usize + y as usize * s;
            let mut n = 0u32;
            for j in [i - s, i, i + s] {
                n += u32::from(susp[j - 1] > PIGMENT)
                    + u32::from(susp[j] > PIGMENT)
                    + u32::from(susp[j + 1] > PIGMENT);
            }
            row[x as usize] = n as u8;
        }
    };
    let ed = &mut scratch.edge[band];
    if par {
        ed.par_chunks_mut(s).enumerate().for_each(|(r, a)| f(r, a));
    } else {
        ed.chunks_mut(s).enumerate().for_each(|(r, a)| f(r, a));
    }
}

/// VERBATIM `drying_pass_jacobi_rows` (drying.rs:448).
fn drying_copy<const C: u8, const N: bool>(
    g: &mut Grid,
    p: &Params,
    evap_base: f64,
    rewet_base: f64,
    ext_bypass: bool,
    par: bool,
) {
    if g.by0 > g.by1 {
        return;
    }
    g.scratch.ensure(g.cells);
    edge_rows(g, par);
    let s = g.s;
    let c = DryConst::new(p, evap_base, rewet_base, ext_bypass);
    let (by0, by1) = (g.by0, g.by1);
    let Grid {
        scratch,
        film,
        susp,
        sett,
        susp_rgb,
        sett_rgb,
        paper,
        vela,
        aparencia: ap,
        oleo,
        row_lo,
        row_hi,
        spans_enabled,
        bx0,
        bx1,
        ..
    } = g;
    let (spans_enabled, bx0, bx1) = (*spans_enabled, *bx0, *bx1);
    let edge: &[u8] = &scratch.edge;
    let paper: &[f32] = paper;
    let vela: &[f32] = vela;
    let ap: &[aparencia::AparenciaCelula] = ap;
    let oleo: &[oleo_celula::OleoCelula] = oleo;
    let band = by0 as usize * s..(by1 as usize + 1) * s;
    type Row<'a> = (
        &'a mut [f32],
        &'a mut [f32],
        &'a mut [f32],
        &'a mut [[f32; 3]],
        &'a mut [[f32; 3]],
    );
    let body = |ri: usize, (fr, sr, tr, cr, dr): Row| {
        let y = by0 + ri as i32;
        let (lo, hi) = span_x_of(row_lo, row_hi, spans_enabled, bx0, bx1, y);
        if lo > hi {
            return;
        }
        let mut out = [0.0f64; 3];
        let base = y as usize * s;
        for x in lo as usize..=hi as usize {
            if fr[x] <= 0.0 && sr[x] <= 0.0 {
                continue;
            }
            let i = base + x;
            dry_cell::<C, N>(
                &c,
                paper[i],
                u32::from(edge[i]),
                vela[i],
                aparencia::vidrado(ap, i),
                oleo_celula::da_celula(oleo, i),
                &mut fr[x],
                &mut sr[x],
                &mut tr[x],
                &mut cr[x],
                &mut dr[x],
                &mut out,
            );
        }
    };
    let (f0, s0, t0, c0, d0) = (
        &mut film[band.clone()],
        &mut susp[band.clone()],
        &mut sett[band.clone()],
        &mut susp_rgb[band.clone()],
        &mut sett_rgb[band],
    );
    if par {
        f0.par_chunks_mut(s)
            .zip(s0.par_chunks_mut(s))
            .zip(t0.par_chunks_mut(s))
            .zip(c0.par_chunks_mut(s))
            .zip(d0.par_chunks_mut(s))
            .enumerate()
            .for_each(|(r, ((((a, b), cc), d), e))| body(r, (a, b, cc, d, e)));
    } else {
        f0.chunks_mut(s)
            .zip(s0.chunks_mut(s))
            .zip(t0.chunks_mut(s))
            .zip(c0.chunks_mut(s))
            .zip(d0.chunks_mut(s))
            .enumerate()
            .for_each(|(r, ((((a, b), cc), d), e))| body(r, (a, b, cc, d, e)));
    }
}

// ---------------------------------------------------------------------------

struct Ctx {
    p: Params,
    evap: f64,
    rewet: f64,
    bypass: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum V {
    Orig,
    Copia,
    Lado,
    LadoCanal,
    Tudo,
    Salta,
}
const VS: [V; 6] = [V::Orig, V::Copia, V::Lado, V::LadoCanal, V::Tudo, V::Salta];

fn dry(v: V, g: &mut Grid, c: &Ctx, par: bool) {
    let (p, ev, rw, by) = (&c.p, c.evap, c.rewet, c.bypass);
    match v {
        V::Orig => ph2d_wet_paint::drying::drying_pass_jacobi_rows(
            g,
            p,
            ev,
            rw,
            by,
            if par { Rows::Parallel } else { Rows::Serial },
        ),
        V::Copia => drying_copy::<0, false>(g, p, ev, rw, by, par),
        V::Lado => drying_copy::<LADO, false>(g, p, ev, rw, by, par),
        V::LadoCanal => drying_copy::<{ LADO | CANAL }, false>(g, p, ev, rw, by, par),
        V::Tudo => drying_copy::<{ LADO | CANAL | MEMO }, false>(g, p, ev, rw, by, par),
        V::Salta => drying_copy::<{ LADO | CANAL | MEMO | SALTA }, false>(g, p, ev, rw, by, par),
    }
}

fn fnv(g: &Grid) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut eat = |v: f32| {
        for b in v.to_bits().to_le_bytes() {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    };
    for a in [&g.film, &g.susp, &g.sett] {
        a.iter().for_each(|&v| eat(v));
    }
    for a in [&g.susp_rgb, &g.sett_rgb] {
        a.iter().flatten().for_each(|&v| eat(v));
    }
    h
}

fn stats(v: &mut [f64]) -> (f64, f64) {
    v.sort_by(f64::total_cmp);
    (v[0], v[v.len() / 2])
}

fn main() {
    let rounds: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(5);
    let t0 = Instant::now();
    let mut e = wash();
    let pp = e.sim.gather_params(&e.tuning);
    let c = Ctx {
        evap: e.sim.evap_scale * pp.k(Knob::Evaporation),
        rewet: e.sim.rewet_base * pp.k(Knob::Rewet),
        bypass: e.sim.ext_bypass,
        p: pp,
    };
    let g = e.active_grid_mut();
    let snap: GridSnapshot = snapshot_grid(g);
    println!(
        "wash {}x{} montado em {:.1} s · janela x {}..{} y {}..{} · ecra {:?} · rayon {} fios · loadavg {}",
        g.w,
        g.h,
        t0.elapsed().as_secs_f64(),
        g.bx0,
        g.bx1,
        g.by0,
        g.by1,
        c.p.ecra,
        rayon::current_num_threads(),
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );

    // Contagem (série, 1 secagem sobre o instantâneo).
    restore_grid(g, &snap);
    drying_copy::<0, true>(g, &c.p, c.evap, c.rewet, c.bypass, false);
    let k = |i: usize| CNT[i].load(Relaxed);
    let n = k(K_CEL).max(1) as f64;
    println!(
        "por célula ({} células): assenta {:.3} · levanta {:.3} · avaliações do ecrã {:.2} · voltas Newton {:.2} · powf {:.2} · dif_pigmento {:.2} · filtrada(6 ln) {:.2} · vela>0 {:.3} · sem assentar {:.3} · assenta c/ coberturas iguais {:.3} · levanta c/ coberturas iguais {:.3}",
        k(K_CEL),
        k(K_ASS) as f64 / n,
        k(K_LEV) as f64 / n,
        k(K_EVAL) as f64 / n,
        k(K_ITER) as f64 / n,
        k(K_POW) as f64 / n,
        k(K_DIF) as f64 / n,
        k(K_FILT) as f64 / n,
        k(K_VELA) as f64 / n,
        k(K_NOLOST) as f64 / n,
        k(K_SAME) as f64 / n,
        k(K_SAMEL) as f64 / n,
    );

    // Hash após N_HASH secagens encadeadas (paralelo e série).
    for par in [true, false] {
        for v in VS {
            restore_grid(g, &snap);
            for _ in 0..N_HASH {
                dry(v, g, &c, par);
            }
            println!(
                "hash N={N_HASH} {} {v:?}: {:016x}",
                if par { "par" } else { "ser" },
                fnv(g)
            );
        }
    }

    // Régua intercalada: rodada 0 aquece; início rodado; cada medida sobre o instantâneo restaurado.
    for par in [true, false] {
        let rr = if par { rounds } else { rounds.div_ceil(2) };
        let mut md: Vec<Vec<f64>> = vec![Vec::new(); VS.len()];
        for round in 0..=rr {
            for kk in 0..VS.len() {
                let vi = (kk + round) % VS.len();
                restore_grid(g, &snap);
                let t = Instant::now();
                dry(VS[vi], g, &c, par);
                if round > 0 {
                    md[vi].push(t.elapsed().as_secs_f64() * 1e3);
                }
            }
        }
        println!(
            "{} — ms por secagem, {rr} rodadas (mín / mediana):",
            if par { "PARALELO" } else { "SÉRIE" }
        );
        for (vi, v) in VS.iter().enumerate() {
            let (a, b) = stats(&mut md[vi]);
            println!("  {v:?}: {a:.1} / {b:.1}");
        }
    }
    println!(
        "loadavg fim {} · total {:.1} s",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim(),
        t0.elapsed().as_secs_f64()
    );
}
