//! ⭐ **Molhado × molhado** ([`Knob::WetOpen`], J5 — `docs/Painter/Pinceis/exames/aguarela.md` «Motor J5»): a tinta
//! que entra na molhada de OUTRA tinta abre nela uma mancha REDONDA e empurra a de baixo para os lados.
//!
//! Medido no Rebelle (cena `sobre`, foto A, `blot.py`): o azul de 40 px que atravessa a faixa vermelha molhada de 60 abre
//! uma mancha de 54–60 px no meio da faixa (Liner 1,42 × a largura no papel, Mop 1,44, Wash 1,17, Gouache Round 1,25),
//! mais estreita junto às bordas da faixa (redonda), e tão escura como o azul no papel (1,06) — o vermelho saiu dela.
//! O nosso: um quadrado da largura do traço e 1,27 × mais escuro (as duas tintas somadas na célula).
//!
//! A lei, em dois tempos:
//! 1. **o cruzamento** — a cada dab a sonda olha `r + 2` à frente da pegada (onde o traço ainda não pousou nada); a
//!    primeira vez que ela cai na tinta molhada de outra cor, mede pela linha do traço a faixa que ele vai atravessar
//!    (`[ta, tb]`) e guarda a mancha: o disco centrado no meio dela, de raio `min(meia faixa, r · (1 + WetOpen))`; e
//!    EMPURRA logo, pela normal do traço, a fracção [`Knob::WetOpenPush`] da tinta de baixo de dentro do disco para a
//!    tinta molhada logo fora dele (a orla do cruzamento);
//! 2. **a mancha** — cada janela que pousa dentro da faixa estica o seu pigmento (e a água) pela normal até à meia-largura
//!    do disco naquela linha: a massa é a mesma, espalhada (a mancha é do azul, mais clara que a soma).
//!
//! `WetOpen = 0` = nada disto corre (o campo fica vazio): o cruzamento de sempre, ao bit.

use super::Trail;
use crate::grid::Grid;
use crate::sim::Params;
use crate::tuning::Knob;

/// A água mínima de uma célula «molhada» (a de [`crate::tools`]).
const MOLHADA: f32 = 0.01;
/// Quanto a sonda anda pela linha do traço a medir a faixa, em raios do dab.
const ALCANCE_RAIOS: f64 = 8.0;
/// A orla que recebe a tinta empurrada: até esta fracção do raio fora do disco, um píxel de cada vez, com pesos que descem.
const ORLA_RAIO: f64 = 2.0;
/// O empurrão desce do centro a zero em `BORDA_FIM` da meia-largura, `1 − (|s| ÷ BORDA_FIM·meia)²` (a borda esbatida
/// da mancha: o azul esticado chega lá ralo, e o vermelho tirado por inteiro deixava um aro claro).
const BORDA_FIM: f64 = 0.8;
/// A mancha esbate: cada célula da pegada reparte-se por [`ESBATE_PASSOS`] destinos pela normal, de onde estava até
/// `meia + ESBATE_ALEM · (meia − r)` (contínuo onde o disco encontra a pegada; o degradê da borda do Rebelle).
const ESBATE_ALEM: f64 = 1.0;
const ESBATE_PASSOS: usize = 8;
/// O alcance do triângulo de pesos ÷ o do esticão (a média fica a meio dele, como com pesos iguais).
const ESBATE_CAUDA: f64 = 1.5;

/// Uma mancha do traço vivo: o eixo (origem e direcção), o centro `tm` no eixo, o raio e o raio do dab.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Mancha {
    ox: f64,
    oy: f64,
    tx: f64,
    ty: f64,
    tm: f64,
    raio: f64,
    r: f64,
}

impl Mancha {
    /// `(t, s)` do ponto no eixo da mancha (ao longo, pela normal).
    fn coords(&self, x: f64, y: f64) -> (f64, f64) {
        let (ax, ay) = (x - self.ox, y - self.oy);
        (ax * self.tx + ay * self.ty, -ax * self.ty + ay * self.tx)
    }

    /// A meia-largura do disco à altura `t` do eixo (`None` fora dele).
    fn meia(&self, t: f64) -> Option<f64> {
        let d = t - self.tm;
        let q = self.raio * self.raio - d * d;
        (q > 0.0).then(|| q.sqrt())
    }
}

/// As manchas do traço vivo e o dab anterior (a direcção). Vazio sem a lei.
#[derive(Clone, Default)]
pub(crate) struct Molhado {
    manchas: Vec<Mancha>,
    anterior: Option<[f64; 2]>,
}

impl Molhado {
    pub(crate) fn limpa(&mut self) {
        self.manchas.clear();
        self.anterior = None;
    }
}

/// A célula `(x, y)` tem tinta MOLHADA de outra cor que `cor`?
fn outra(g: &Grid, x: f64, y: f64, cor: [f64; 3]) -> bool {
    celula(g, x, y).is_some_and(|i| {
        g.film[i] > MOLHADA
            && g.susp[i] > 0.0
            && crate::colorops::outra_tinta(g.susp_rgb[i].map(f64::from), cor) > 0.0
    })
}

fn celula(g: &Grid, x: f64, y: f64) -> Option<usize> {
    let (cx, cy) = (x.round() as i64, y.round() as i64);
    (cx >= 2 && cy >= 2 && cx < g.w as i64 - 1 && cy < g.h as i64 - 1).then(|| cx as usize + cy as usize * g.s)
}

impl Trail {
    /// O tempo 1 (ver o módulo), a cada dab de tinta, ANTES de ele acumular.
    pub(super) fn molhado_dab(&mut self, g: &mut Grid, p: &Params, x: f64, y: f64, r: f64) {
        let abre = p.k(Knob::WetOpen);
        if abre <= 0.0 || r <= 0.0 {
            return;
        }
        let Some([px, py]) = self.molhado.anterior.replace([x, y]) else {
            return;
        };
        let passo = (x - px).hypot(y - py);
        if passo < 1e-6 {
            return;
        }
        let (tx, ty) = ((x - px) / passo, (y - py) / passo);
        let cor = [self.base_r, self.base_g, self.base_b];
        let frente = r + 2.0;
        let (sx, sy) = (x + frente * tx, y + frente * ty);
        if self.molhado.manchas.last().is_some_and(|m| {
            let (t, s) = m.coords(sx, sy);
            s.abs() <= m.r && t <= m.tm + m.raio
        }) || !outra(g, sx, sy, cor)
        {
            return;
        }
        // A faixa pela linha do traço: para trás só até à pegada (atrás dela o traço já pousou), para a frente até ao fim.
        let mut ta = frente;
        while ta > r && outra(g, x + (ta - 1.0) * tx, y + (ta - 1.0) * ty, cor) {
            ta -= 1.0;
        }
        let mut tb = frente;
        while tb < frente + ALCANCE_RAIOS * r && outra(g, x + (tb + 1.0) * tx, y + (tb + 1.0) * ty, cor) {
            tb += 1.0;
        }
        let raio = (0.5 * (tb - ta + 1.0)).min(r * (1.0 + abre));
        if raio <= r {
            return;
        }
        let m = Mancha { ox: x, oy: y, tx, ty, tm: 0.5 * (ta + tb), raio, r };
        empurra(g, p, &m, cor, p.k(Knob::WetOpenPush));
        self.molhado.manchas.push(m);
    }

    /// O tempo 2 (ver o módulo): a janela que pousa estica-se pela normal até ao disco de cada mancha.
    pub(super) fn molhado_estica(&mut self) {
        if self.molhado.manchas.is_empty() || self.lx1 < self.lx0 || self.tinta_na_janela {
            return;
        }
        let (n, size, half) = (self.pig.len(), self.size, self.half);
        let mut pig = vec![0.0f32; n];
        let mut agua = vec![0.0f32; n];
        let mut cor = vec![[0.0f64; 3]; n];
        let mut ext = (self.lx0, self.ly0, self.lx1, self.ly1);
        for ly in self.ly0..=self.ly1 {
            for lx in self.lx0..=self.lx1 {
                let l = (lx + ly * size) as usize;
                let (v, a) = (self.pig[l], self.water[l]);
                if v <= 0.0 && a <= 0.0 {
                    continue;
                }
                let tip = [self.tip_r[l], self.tip_g[l], self.tip_b[l]].map(f64::from);
                let x = f64::from(self.anchor_x + lx - half);
                let y = f64::from(self.anchor_y + ly - half);
                // Os destinos: na primeira mancha que contém a célula, a normal esticada de 1 até ao disco (esbatido).
                let mancha = self.molhado.manchas.iter().find_map(|m| {
                    let (t, s) = m.coords(x, y);
                    let meia = m.meia(t)?;
                    (s.abs() <= m.r && meia > m.r).then_some((m, s, (meia - m.r) * (1.0 + ESBATE_ALEM) / m.r))
                });
                // Pesos que descem (o triângulo) até `ESBATE_CAUDA` do esticão: a cauda vai a zero, sem degrau na borda.
                let total = (ESBATE_PASSOS * (ESBATE_PASSOS + 1) / 2) as f64;
                let destinos: Vec<(f64, f64, f64)> = match mancha {
                    Some((m, s, k)) => (0..ESBATE_PASSOS)
                        .map(|j| {
                            let d = s * k * ESBATE_CAUDA * (j as f64 + 0.5) / ESBATE_PASSOS as f64;
                            (x - d * m.ty, y + d * m.tx, (ESBATE_PASSOS - j) as f64 / total)
                        })
                        .collect(),
                    None => vec![(x, y, 1.0)],
                };
                for (dx, dy, parte) in destinos {
                let (fx, fy) = (dx - f64::from(self.anchor_x - half), dy - f64::from(self.anchor_y - half));
                let (x0, y0) = (fx.floor(), fy.floor());
                let (ax, ay) = (fx - x0, fy - y0);
                for (qx, qy, peso) in [
                    (x0, y0, parte * (1.0 - ax) * (1.0 - ay)),
                    (x0 + 1.0, y0, parte * ax * (1.0 - ay)),
                    (x0, y0 + 1.0, parte * (1.0 - ax) * ay),
                    (x0 + 1.0, y0 + 1.0, parte * ax * ay),
                ] {
                    let (qx, qy) = (qx as i32, qy as i32);
                    if peso <= 0.0 || !(0..size).contains(&qx) || !(0..size).contains(&qy) {
                        continue;
                    }
                    let q = (qx + qy * size) as usize;
                    let dv = f64::from(v) * peso;
                    let antes = f64::from(pig[q]);
                    if dv > 0.0 {
                        let w = dv / (antes + dv);
                        for k in 0..3 {
                            cor[q][k] += (tip[k] - cor[q][k]) * w;
                        }
                    } else if antes <= 0.0 {
                        cor[q] = tip;
                    }
                    pig[q] = (antes + dv) as f32;
                    agua[q] = (f64::from(agua[q]) + f64::from(a) * peso) as f32;
                    super::touch_ext(&mut ext, qx, qy);
                }
                }
            }
        }
        (self.lx0, self.ly0, self.lx1, self.ly1) = ext;
        for ly in self.ly0..=self.ly1 {
            for lx in self.lx0..=self.lx1 {
                let l = (lx + ly * size) as usize;
                self.pig[l] = pig[l];
                self.water[l] = agua[l];
                if pig[l] > 0.0 {
                    [self.tip_r[l], self.tip_g[l], self.tip_b[l]] = cor[l].map(|c| c as f32);
                }
            }
        }
    }
}

/// O EMPURRÃO (tempo 1): a fracção `forca × outra_tinta` da tinta molhada de dentro do disco sai pela normal do traço
/// para a tinta molhada da orla logo fora dele ([`ORLA_RAIO`]); o que não tem célula molhada para onde ir fica.
fn empurra(g: &mut Grid, p: &Params, m: &Mancha, cor: [f64; 3], forca: f64) {
    if forca <= 0.0 {
        return;
    }
    // Um destino por píxel da orla (sem anéis), com pesos que descem (o triângulo), somando 1.
    let passos = (ORLA_RAIO * m.raio).ceil().max(1.0) as usize;
    let largura = passos as f64;
    let total = (passos * (passos + 1) / 2) as f64;
    let orla: Vec<(f64, f64)> = (0..passos).map(|k| (1.0 + k as f64, (passos - k) as f64 / total)).collect();
    let alcance = m.raio + largura + 2.0;
    let (cx, cy) = (m.ox + m.tm * m.tx, m.oy + m.tm * m.ty);
    let mut levar: Vec<(usize, f64, [f32; 3])> = Vec::new();
    let (y0, y1) = ((cy - alcance).floor() as i64, (cy + alcance).ceil() as i64);
    let (x0, x1) = ((cx - alcance).floor() as i64, (cx + alcance).ceil() as i64);
    for yy in y0..=y1 {
        for xx in x0..=x1 {
            let (x, y) = (xx as f64, yy as f64);
            let Some(i) = celula(g, x, y) else { continue };
            let (t, s) = m.coords(x, y);
            let Some(meia) = m.meia(t) else { continue };
            if s.abs() >= meia || g.film[i] <= MOLHADA || g.susp[i] <= 0.0 {
                continue;
            }
            let o = crate::colorops::outra_tinta(g.susp_rgb[i].map(f64::from), cor);
            if o <= 0.0 {
                continue;
            }
            // A borda da mancha esbate ([`BORDA_FIM`]).
            let q = (1.0 - (s / (BORDA_FIM * meia)).powi(2)).max(0.0);
            let massa = f64::from(g.susp[i]) * (forca * o * q).min(1.0);
            let sinal = if s >= 0.0 { 1.0 } else { -1.0 };
            let mut levou = 0.0;
            for &(d, peso) in &orla {
                let off = sinal * (meia + d) - s;
                let (fx, fy) = (x - off * m.ty, y + off * m.tx);
                let (bx, by) = (fx.floor(), fy.floor());
                let (ax, ay) = (fx - bx, fy - by);
                for (qx, qy, b) in [
                    (bx, by, (1.0 - ax) * (1.0 - ay)),
                    (bx + 1.0, by, ax * (1.0 - ay)),
                    (bx, by + 1.0, (1.0 - ax) * ay),
                    (bx + 1.0, by + 1.0, ax * ay),
                ] {
                    if let Some(j) = celula(g, qx, qy).filter(|&j| b > 0.0 && j != i && g.film[j] > MOLHADA) {
                        levar.push((j, massa * peso * b, g.susp_rgb[i]));
                        levou += massa * peso * b;
                    }
                }
            }
            g.susp[i] = (f64::from(g.susp[i]) - levou).max(0.0) as f32;
        }
    }
    let mut out = [0.0f64; 3];
    for (j, dm, c) in levar {
        let antes = f64::from(g.susp[j]);
        if antes > 0.0 {
            let tc = g.susp_rgb[j];
            p.mix.mix(
                f64::from(tc[0]),
                f64::from(tc[1]),
                f64::from(tc[2]),
                f64::from(c[0]),
                f64::from(c[1]),
                f64::from(c[2]),
                dm / (antes + dm),
                &mut out,
            );
            g.susp_rgb[j] = out.map(|v| v as f32);
        } else {
            g.susp_rgb[j] = c;
        }
        g.susp[j] = (antes + dm) as f32;
    }
}
