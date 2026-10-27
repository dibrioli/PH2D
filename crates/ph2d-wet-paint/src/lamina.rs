//! ⭐ **A LÂMINA da faca e o REBORDO** (Óleo Real W2 E4, CAP-01 / C38; medição em
//! `docs/Painter/49_realshader/E4_lamina.md`).
//!
//! A ORIENTAÇÃO da lâmina é do hospedeiro e já existe: `dab_flatten` × `dab_angle_deg` × o knob `dabRotation`
//! (`0` fixa na tela · `1` segue o traço · `2` a inclinação da caneta) — a lei L3 medida no Rebelle (06_LEIS §1:
//! fixa OU seguidora, sem inércia nem memória). O que faltava ao motor é o REBORDO: a lâmina arrastada empurra a
//! tinta para as PONTAS dela, e o traço fica com uma crista nas duas bordas laterais (Knife Smooth 2 / 3 do
//! Rebelle, folha F42: o declive escuro por dentro da borda de cima e o claro por dentro da de baixo, ~0,12 da
//! meia-largura, iguais nas duas facas largas).
//!
//! A lei: a altura ([`crate::grid::Grid::altura`]) de cada célula sob o dab é penteada
//! ([`crate::bristles_relevo::penteia`]) para `altura · perfil(u)`, com `u` = a distância ATRAVÉS do traço ÷ a
//! meia-largura que a silhueta do dab tem nessa direcção: `0` no miolo (a lâmina ALISA) e uma rampa até `altura`
//! na última [`FAIXA`] (a crista encostada à borda). A meia-largura mede-se na própria silhueta, por isso a lâmina
//! que gira com o traço (ou fixa e atravessada) dá a crista nas bordas do que ela varre, em qualquer braço do traço.
//! [`Rebordo::NEUTRO`] (altura `0`) = o hospedeiro nem chama a porta: a tinta de antes, ao bit.
//!
//! W3 K3a (`docs/Painter/Pinceis/W3_K3a_faca.md`): a crista vai também para o campo PRÓPRIO
//! [`crate::grid::Grid::crista`] (a sombra soma-o sem os fios do Oil Bristle), e a faca NIVELA ([`nivelar`], knob
//! `knifeLevel`, `0` = ao bit): a tinta em pé sob a lâmina vai para a média dela — o planalto no lugar do tubo.

use crate::painter::Engine;
use crate::par::Rows;

/// A fracção da meia-largura (a partir da borda) onde a crista sobe — medida na folha F42 do Rebelle: o declive
/// interior da borda ocupa 6 px de 45,5 (Knife Smooth 2) e 5–6 de 44 (Knife Smooth 3).
pub const FAIXA: f64 = 0.12;

/// O rebordo de um pincel (o knob `knifeRidge` do mapa `engine` do preset).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rebordo {
    /// A altura (px) da crista na borda (`0` = sem rebordo).
    pub altura_px: f64,
}

impl Rebordo {
    pub const NEUTRO: Rebordo = Rebordo { altura_px: 0.0 };

    /// Sem altura: a lei de antes.
    #[must_use]
    pub fn neutro(&self) -> bool {
        self.altura_px == 0.0
    }
}

/// O perfil ATRAVÉS (`0..1`) a `u` meias-larguras do eixo do traço: `0` no miolo, rampa recta na última
/// [`FAIXA`], `1` na borda (e fora dela, onde a silhueta já não pesa).
#[must_use]
pub fn perfil(u: f64) -> f64 {
    ((u.abs() - (1.0 - FAIXA)) / FAIXA).clamp(0.0, 1.0)
}

/// A normal unitária ao rumo `rumo` (`None` sem rumo — o 1.º dab antes de o traço ter direcção).
#[must_use]
pub fn normal(rumo: [f64; 2]) -> Option<[f64; 2]> {
    let n = rumo[0].hypot(rumo[1]);
    (n > 1e-9).then(|| [-rumo[1] / n, rumo[0] / n])
}

impl Engine {
    /// ⭐ **A lâmina de UM dab**: o rebordo e o nivelamento sob o dab de centro `(x, y)` (células), na caixa de
    /// meia-extensão `meia`, pela cobertura `sil`; `u` mede-se ATRAVÉS de `rumo` (a direcção do traço) contra a
    /// meia-largura que a própria silhueta tem nessa direcção (as células que ela cobre a metade ou mais, mais meia
    /// célula). Sem rumo, nada.
    ///
    /// - **Rebordo** (`altura` > 0, a unidade de [`crate::grid::Grid::altura`]): penteia a `altura` (a normal) E a
    ///   [`crate::grid::Grid::crista`] (só a faca: a sombra) para `altura · perfil(u)`; `passos` = o pente por
    ///   comprimento ([`crate::bristles_relevo::penteia`]).
    /// - **Nivelamento** (o knob [`crate::tuning::Knob::KnifeLevel`] > 0): a tinta EM PÉ sob a lâmina vai para a
    ///   média dela ([`nivelar`]).
    #[allow(clippy::too_many_arguments)]
    pub fn pentear_rebordo(
        &mut self,
        x: f64,
        y: f64,
        meia: [f64; 2],
        sil: crate::brush::CellFn<'_>,
        rumo: [f64; 2],
        altura: f64,
        passos: f64,
    ) {
        let Some(n) = normal(rumo) else {
            return;
        };
        let nivel = self.tuning.get(crate::tuning::Knob::KnifeLevel);
        let (ax, ay) = (meia[0].ceil() as i32 + 1, meia[1].ceil() as i32 + 1);
        let g = &mut self.layers[self.active_layer].grid;
        let (cx, cy) = (x.round() as i32, y.round() as i32);
        let (x0, x1) = ((cx - ax).max(1), (cx + ax).min(g.w as i32));
        let (y0, y1) = ((cy - ay).max(1), (cy + ay).min(g.h as i32));
        if x1 < x0 || y1 < y0 {
            return;
        }
        let atraves = |px: i32, py: i32| (f64::from(px) - x) * n[0] + (f64::from(py) - y) * n[1];
        let mut largura = 0.0f64;
        for py in y0..=y1 {
            for px in x0..=x1 {
                if sil(px, py) >= 0.5 {
                    largura = largura.max(atraves(px, py).abs());
                }
            }
        }
        if largura <= 0.0 {
            return;
        }
        let inv = 1.0 / (largura + 0.5);
        let s = g.s;
        let (linhas, colunas) = ((y1 - y0 + 1) as usize, (x1 - x0 + 1) as usize);
        let mode = Rows::pick(linhas, colunas, crate::par::MIN_CELLS_PENTE);
        if altura != 0.0 {
            let pente = |campo: &mut [f32]| {
                let faixa = &mut campo[y0 as usize * s..(y1 as usize + 1) * s];
                crate::par::walk_rows(mode, faixa, s, |k, linha| {
                    let py = y0 + k as i32;
                    for px in x0..=x1 {
                        let w = sil(px, py);
                        if w <= 0.0 {
                            continue;
                        }
                        let alvo = altura * perfil(atraves(px, py) * inv);
                        let c = &mut linha[px as usize];
                        *c = crate::bristles_relevo::penteia(*c, alvo, w, passos);
                    }
                });
            };
            pente(crate::grid::altura::garante(g));
            pente(crate::grid::crista::garante(g));
        }
        if nivel > 0.0 {
            nivelar(g, (x0, y0, x1, y1), sil, nivel);
            // Os sulcos dos fios (o relevo do Shape) também se alisam sob a lâmina: o miolo vai ao perfil do rebordo
            // (`0` sem ele) pela mesma fracção — o pente do rebordo sozinho fica em equilíbrio com o relevo que o dab
            // acabou de pentear.
            if !g.altura.is_empty() {
                let k = nivel.clamp(0.0, 1.0);
                for py in y0..=y1 {
                    for px in x0..=x1 {
                        let w = sil(px, py).min(1.0);
                        if w <= 0.0 {
                            continue;
                        }
                        let alvo = altura * perfil(atraves(px, py) * inv);
                        let c = &mut g.altura[px as usize + py as usize * s];
                        *c = (f64::from(*c) + (alvo - f64::from(*c)) * k * w) as f32;
                    }
                }
            }
        }
        self.merge_dirty(x0, y0, x1, y1);
    }
}

/// ⭐ **A faca NIVELA** (W3 K3a, `docs/Painter/Pinceis/W3_K3a_faca.md` §1): a espessura ([`crate::grid::espessura`])
/// de cada célula com tinta na caixa `(x0, y0, x1, y1)` vai, pela cobertura `w` da silhueta e a fracção `nivel` por
/// dab, para a MÉDIA (pesada por `w`) da espessura sob a lâmina — o topo plano do Rebelle no lugar dos sulcos e do
/// tubo. Só a altura muda (o factor pode ficar < 1, [`crate::grid::espessura::nivela`]): a massa e a cor são as de
/// antes, ao bit.
pub fn nivelar(
    g: &mut crate::grid::Grid,
    (x0, y0, x1, y1): (i32, i32, i32, i32),
    sil: crate::brush::CellFn<'_>,
    nivel: f64,
) {
    let s = g.s;
    let tem_tinta = |g: &crate::grid::Grid, i: usize| g.sett[i] + g.susp[i] > 0.0;
    let (mut soma, mut peso) = (0.0f64, 0.0f64);
    for py in y0..=y1 {
        for px in x0..=x1 {
            let i = px as usize + py as usize * s;
            let w = sil(px, py);
            if w > 0.0 && tem_tinta(g, i) {
                soma += w * crate::grid::espessura::em(g, i);
                peso += w;
            }
        }
    }
    if peso <= 0.0 {
        return;
    }
    let media = soma / peso;
    let k = nivel.clamp(0.0, 1.0);
    for py in y0..=y1 {
        for px in x0..=x1 {
            let i = px as usize + py as usize * s;
            let w = sil(px, py);
            if w > 0.0 && tem_tinta(g, i) {
                let t = crate::grid::espessura::em(g, i);
                crate::grid::espessura::nivela(g, i, t + (media - t) * k * w.min(1.0));
            }
        }
    }
}

#[cfg(test)]
#[path = "lamina_tests.rs"]
mod tests;
