//! **O CHÃO COMO FORÇA** (janela 10, lei do dono 01/11 para a aguada contra o relevo): nas águas rasas a superfície
//! que a água nivela é `h + chão`, e o declive dela entra no MOMENTO a cada passo (`−G·∇(h + chão)`), como a gravidade
//! da tela — não no nivelamento clampado de um quadro em quatro ([`super::ladeira_do_oleo::LadeiraDoOleo::termo`]).
//! A água empoça contra o relevo, contorna-o pelas laterais e só passa por cima com volume. O chão é a mesma altura
//! do óleo assentado e SECO no início do traço ([`Grid::chao`]); sem ele, nada corre, ao bit.
//! Medição e escolha: `docs/Painter/Arquitetura/J10_chao_como_forca.md`.

use crate::grid::Grid;
use crate::par::{self, Rows};
use crate::sim::Params;
use ph2d_det_math::DetMath as _;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicU8, AtomicU32};

/// O ganho do chão, em velocidade por (unidade de altura ÷ célula) por unidade de `film`: o peso por passo do
/// nivelamento de fábrica (`Leveling` 0,5 num quadro em quatro) — a água nivela `h + chão` como nivela `h`. MEDIDO
/// (sonda J10): 0,05 deixa 65 % mais água em cima do óleo; 0,125 e 0,5 seguram e contornam igual.
pub(crate) const G_CHAO: f64 = 0.125;

/// Ferramenta de MEDIÇÃO: os bits de um `f32` — `0` = [`G_CHAO`] (o produto), `> 0` = outro ganho, `< 0` = desligado.
#[doc(hidden)]
pub static CHAO_FORCA: AtomicU32 = AtomicU32::new(0);
/// Ferramenta de MEDIÇÃO: bit 0 = a ladeira continua TAMBÉM no nivelamento; bit 1 = também com o Real Oil na mão.
#[doc(hidden)]
pub static CHAO_ROTA: AtomicU8 = AtomicU8::new(0);
/// Ferramenta de MEDIÇÃO: os bits de um `f32` — `0` = [`FILM_SECO`], senão outro.
#[doc(hidden)]
pub static CHAO_SECO: AtomicU32 = AtomicU32::new(0);

/// O líquido de ÓLEO em cima da tinta (`film·m`) que a faz deixar de ser chão: `seco = (1 − film·m ÷ FILM_SECO)₊`.
/// A tinta de óleo ainda fluida corre com o seu líquido; a água que chegou por cima (`m = 0`, J9) não lhe tira o chão.
pub(crate) const FILM_SECO: f64 = 0.1;

/// O ATRITO DO LEITO (J11, smoke do dono J10: «a água ataca o óleo mais depressa que escorre no papel»): a água
/// rasa sobre o relevo rugoso é segurada por ele, a funda corre — `v ÷= 1 + K·(|∇chão| ÷ f)²` (o atrito de leito de
/// Manning cresce com a rugosidade sobre a profundidade). Sem ele a frente da água atravessava o corpo do óleo mais
/// depressa que escorria no papel (sonda J11). Escolha e recusas: `J11_agua_no_oleo.md`.
pub(crate) const K_ATRITO: f64 = 0.3;

/// Ferramenta de MEDIÇÃO: os bits de um `f32` — `0` = [`K_ATRITO`], `> 0` = outro, `< 0` = sem atrito.
#[doc(hidden)]
pub static CHAO_ATRITO: AtomicU32 = AtomicU32::new(0);

/// Ferramenta de MEDIÇÃO: o expoente de `|∇chão| ÷ f` no atrito (`0` = o do produto, 2; `1` = linear).
#[doc(hidden)]
pub static CHAO_ATRITO_P: AtomicU8 = AtomicU8::new(0);

fn atrito() -> f64 {
    match CHAO_ATRITO.load(Relaxed) {
        0 => K_ATRITO,
        b => f64::from(f32::from_bits(b)).max(0.0),
    }
}

/// Ferramenta de MEDIÇÃO: `1` = acumula [`MEDIDA`].
#[doc(hidden)]
pub static CHAO_MEDE: AtomicU8 = AtomicU8::new(0);

/// O que a sonda lê: somas sobre as células molhadas com declive de chão, por passo.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct MedidaChao {
    /// Células·passo somadas.
    pub n: u64,
    /// `Σ|g·f|` — a gravidade da tela que a célula ganha neste passo.
    pub gravidade: f64,
    /// `Σ|G·∇chão·f|` — a força do chão com o ganho em vigor.
    pub chao: f64,
    /// `Σ|∇chão|` (por célula).
    pub declive: f64,
    /// `Σ|termo da ladeira|÷4` — o que o nivelamento daria (um quadro em quatro), clampado.
    pub ladeira: f64,
    /// `Σ|termo|÷4` sem o clamp.
    pub ladeira_livre: f64,
    /// Células·passo em que o clamp mordeu.
    pub clamp_morde: u64,
}

thread_local! {
    /// A medida do fio que chama o passo (as sondas correm casos em fios paralelos).
    #[doc(hidden)]
    pub static MEDIDA: std::cell::Cell<MedidaChao> = std::cell::Cell::new(MedidaChao::default());
}

fn ganho() -> f64 {
    match CHAO_FORCA.load(Relaxed) {
        0 => G_CHAO,
        b => f64::from(f32::from_bits(b)),
    }
}

fn film_seco() -> f64 {
    match CHAO_SECO.load(Relaxed) {
        0 => FILM_SECO,
        b => f64::from(f32::from_bits(b)),
    }
}

/// No início de um traço: com a força ligada e óleo na grade, fotografa o chão (a altura do óleo assentado × a
/// fracção assente × o quanto está seco); senão esvazia-o.
pub(crate) fn fotografa(g: &mut Grid, oleo_na_mao: bool) {
    if g.oleo.is_empty() {
        g.sett_do_traco = Vec::new();
    } else {
        g.sett_do_traco.clone_from(&g.sett);
    }
    if !(ativo(oleo_na_mao) || CHAO_MEDE.load(Relaxed) != 0) || g.oleo.is_empty() {
        g.chao = Vec::new();
        return;
    }
    let fs = film_seco();
    let chao: Vec<f32> = (0..g.cells)
        .map(|i| {
            let h = super::ladeira_do_oleo::altura(g, i, false);
            if h <= 0.0 {
                return 0.0;
            }
            let meio = f64::from(crate::grid::oleo_celula::da_celula(&g.oleo, i).meio);
            let seco = (1.0 - f64::from(g.film[i]) * meio / fs).clamp(0.0, 1.0);
            (f64::from(h) * f64::from(super::ladeira_do_oleo::fraccao_assente(g, i)) * seco) as f32
        })
        .collect();
    g.chao = if chao.iter().any(|&v| v > 0.0) {
        chao
    } else {
        Vec::new()
    };
}

fn na_mao_tambem() -> bool {
    CHAO_ROTA.load(Relaxed) & 2 != 0
}

/// O chão corre neste passo: ganho positivo e (sem o Real Oil na mão, salvo a rota de medição).
fn ativo(oleo_na_mao: bool) -> bool {
    ganho() > 0.0 && (!oleo_na_mao || na_mao_tambem())
}

/// A ladeira sai do nivelamento quando o chão a leva como força ([`CHAO_ROTA`] bit 0 a mantém).
pub(crate) fn tira_do_nivelamento(oleo_na_mao: bool) -> bool {
    ativo(oleo_na_mao) && CHAO_ROTA.load(Relaxed) & 1 == 0
}

/// O passe tem de correr nesta grade (a rota fundida cede-lhe); a medição corre-o também com o ganho a zero.
pub(crate) fn liga(g: &Grid, p: &Params) -> bool {
    (ativo(p.oleo > 0.0) || CHAO_MEDE.load(Relaxed) != 0) && g.chao.len() == g.cells
}

/// Soma `−G·∇(h + chão)·min(1, f ÷ FILM_CHEIO)` ([`Grid::chao`]) à velocidade, nas células com declive de chão, persistente, na grade de FLUXO, depois do momento.
pub(crate) fn aplica(g: &mut Grid, p: &Params, gx: f64, gy: f64, mode: Rows) {
    let geom = g.flow;
    let (fbx0, fby0, fbx1, fby1) = crate::flow::flow_bbox((g.bx0, g.by0, g.bx1, g.by1), geom.rf);
    if fby0 > fby1 || fbx0 > fbx1 {
        return;
    }
    let na_mao = p.oleo > 0.0;
    let k = if ativo(na_mao) { ganho() } else { 0.0 };
    let ka = if k > 0.0 { atrito() } else { 0.0 };
    let dif = crate::flow::diff_scale(geom.rf);
    let max_v = p.k(crate::tuning::Knob::MaxVelocity);
    let mede = CHAO_MEDE.load(Relaxed) != 0;
    let (level_k, clamp) = (
        p.k(crate::tuning::Knob::Leveling),
        p.k(crate::tuning::Knob::LevelClamp),
    );
    let (s, gw, gh, rf) = (g.s, g.w as i32, g.h as i32, geom.rf);
    let (by0, by1) = (g.by0, g.by1);
    let Grid {
        film,
        active,
        chao,
        sett,
        sett_do_traco,
        vel_x,
        vel_y,
        ..
    } = g;
    let (film, active, b): (&[f32], &[u8], &[f32]) = (film, active, chao);
    let (sett, s0): (&[f32], &[f32]) = (sett, sett_do_traco);
    // O chão de agora: o da fotografia × o que resta do assentado dela (o dissolvido baixa o relevo).
    let em = |i: usize| {
        let c = f64::from(b[i]);
        match s0.get(i) {
            Some(&a) if a > 0.0 && c > 0.0 => c * (f64::from(sett[i]) / f64::from(a)).min(1.0),
            _ => c,
        }
    };
    let soma = std::sync::Mutex::new(MedidaChao::default());
    let band = fby0 as usize * geom.s..(fby1 as usize + 1) * geom.s;
    par::walk_rows2(
        mode,
        &mut vel_x[band.clone()],
        &mut vel_y[band],
        geom.s,
        |r, vx, vy| {
            let cy = fby0 + r as i32;
            let py = crate::flow::flow_probe(cy, gh, rf);
            if py < by0 || py > by1 {
                return;
            }
            let pu = crate::flow::flow_probe(cy - 1, gh, rf) as usize;
            let pd = crate::flow::flow_probe(cy + 1, gh, rf) as usize;
            let mut m = MedidaChao::default();
            for cx in fbx0..=fbx1 {
                let px = crate::flow::flow_probe(cx, gw, rf);
                let i = px as usize + py as usize * s;
                let f = f64::from(film[i]);
                if active[i] == 0 || f <= 0.0 {
                    continue;
                }
                let pl = crate::flow::flow_probe(cx - 1, gw, rf) as usize;
                let pr = crate::flow::flow_probe(cx + 1, gw, rf) as usize;
                let row = py as usize * s;
                let bx = em(row + pl) - em(row + pr);
                let by = em(pu * s + px as usize) - em(pd * s + px as usize);
                if bx == 0.0 && by == 0.0 {
                    continue;
                }
                // A SUPERFÍCIE `h + chão`: a poça mais alta que a parede passa por cima. A parte da água pesa pelo
                // relevo da vizinhança (um relevo de uma água cheia pesa 1): sem parede, o nivelamento de sempre.
                let alto = [
                    i,
                    row + pl,
                    row + pr,
                    pu * s + px as usize,
                    pd * s + px as usize,
                ]
                .into_iter()
                .map(em)
                .fold(0.0, f64::max);
                let wh = (alto / super::ladeira_do_oleo::FILM_CHEIO).min(1.0);
                let ox = bx + wh * (f64::from(film[row + pl]) - f64::from(film[row + pr]));
                let oy = by
                    + wh * (f64::from(film[pu * s + px as usize])
                        - f64::from(film[pd * s + px as usize]));
                let col = cx as usize;
                let wf = (f / super::ladeira_do_oleo::FILM_CHEIO).min(1.0);
                let (ax, ay) = (k * ox * dif * wf, k * oy * dif * wf);
                let rf_ = 0.5 * bx.det_hypot(by) * dif / f.max(0.05);
                let freio = 1.0
                    + ka * match CHAO_ATRITO_P.load(Relaxed) {
                        1 => rf_,
                        _ => rf_ * rf_,
                    };
                vx[col] = ((f64::from(vx[col]) + ax) / freio).clamp(-max_v, max_v) as f32;
                vy[col] = ((f64::from(vy[col]) + ay) / freio).clamp(-max_v, max_v) as f32;
                if mede {
                    let dfx = f64::from(film[row + pl]) - f64::from(film[row + pr]);
                    let dfy = f64::from(film[pu * s + px as usize])
                        - f64::from(film[pd * s + px as usize]);
                    let tx = (dfx + bx) * level_k * dif;
                    let ty = (dfy + by) * level_k * dif;
                    m.n += 1;
                    m.gravidade += (gx * gx + gy * gy).sqrt() * f;
                    m.chao += (ax * ax + ay * ay).sqrt();
                    m.declive += bx.det_hypot(by) * dif;
                    m.ladeira += tx.clamp(-clamp, clamp).det_hypot(ty.clamp(-clamp, clamp)) / 4.0;
                    m.ladeira_livre += tx.det_hypot(ty) / 4.0;
                    m.clamp_morde += u64::from(tx.abs() > clamp || ty.abs() > clamp);
                }
            }
            if mede {
                let mut t = soma.lock().expect("a soma");
                junta(&mut t, &m);
            }
        },
    );
    if mede {
        let m = soma.into_inner().expect("a soma");
        MEDIDA.with(|c| {
            let mut t = c.get();
            junta(&mut t, &m);
            c.set(t);
        });
    }
}

fn junta(a: &mut MedidaChao, b: &MedidaChao) {
    a.n += b.n;
    a.gravidade += b.gravidade;
    a.chao += b.chao;
    a.declive += b.declive;
    a.ladeira += b.ladeira;
    a.ladeira_livre += b.ladeira_livre;
    a.clamp_morde += b.clamp_morde;
}
