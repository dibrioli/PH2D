//! **As fases de UM evento da pilha que acumula** — a sonda de teste do
//! [`super::super::composite_acumulado`], cortada dele pelo tecto de LOC (a lei e a sonda que a
//! mede são duas responsabilidades). O doc da sonda mora na declaração do módulo.

use super::Region;
use std::cell::Cell;

pub(in crate::tool::paint) const PRE: usize = 0;
pub(in crate::tool::paint) const ACUMULAR: usize = 1;
pub(in crate::tool::paint) const COMPOR: usize = 2;
pub(in crate::tool::paint) const COPIAS: usize = 3;

thread_local! {
    static US: Cell<[u64; 4]> = const { Cell::new([0; 4]) };
    static EVENTOS: Cell<u64> = const { Cell::new(0) };
    /// A soma das áreas de `alvo`, em fracção da TELA — *o que decide se o custo segue a
    /// FIGURA ou a TELA*.
    static AREA: Cell<f64> = const { Cell::new(0.0) };
    /// A área relativa de um cover por blocos de [`BLOCOS`], somada por evento.
    static COBERTURA: Cell<[f64; 5]> = const { Cell::new([0.0; 5]) };
}

/// Os lados de bloco que a sonda da cobertura varre.
pub(in crate::tool::paint) const BLOCOS: [u32; 5] = [16, 32, 64, 128, 256];

/// A cobertura acumulada desde a última leitura — e ZERA.
pub(in crate::tool::paint) fn take_cobertura() -> [f64; 5] {
    COBERTURA.with(Cell::take)
}

pub(in crate::tool::paint) fn soma(i: usize, t: std::time::Instant) {
    US.with(|c| {
        let mut v = c.get();
        v[i] += t.elapsed().as_micros() as u64;
        c.set(v);
    });
}

pub(in crate::tool::paint) fn conta_area(alvo: Region, w: u32, h: u32) {
    EVENTOS.with(|c| c.set(c.get() + 1));
    let tela = f64::from(w) * f64::from(h);
    if tela > 0.0 {
        let a = f64::from(alvo.w) * f64::from(alvo.h) / tela;
        AREA.with(|c| c.set(c.get() + a));
    }
}

/// **A área que um cover POR BLOCOS pagaria, contra a caixa envolvente** — a medição que
/// decide se vale a pena compor a FIGURA em vez do RECTÂNGULO dela.
///
/// Um anel ocupa `~5 %` da caixa dele, e a composição percorre a caixa inteira. Mas cada bloco
/// paga o **avental** do borrão (`pad` de cada lado), logo um bloco pequeno é quase todo
/// avental: *o tamanho óptimo do bloco é uma medição, não uma escolha.*
pub(in crate::tool::paint) fn conta_cobertura(
    camadas: &[Vec<ph2d_painter_brush::Dab>],
    pad: u32,
    alvo: Region,
    w: u32,
    h: u32,
) {
    let bbox = f64::from(alvo.w) * f64::from(alvo.h);
    if bbox <= 0.0 {
        return;
    }
    let mut out = [0f64; BLOCOS.len()];
    for (bi, &b) in BLOCOS.iter().enumerate() {
        let nx = w.div_ceil(b) as usize;
        let ny = h.div_ceil(b) as usize;
        let mut marca = vec![false; nx * ny];
        for lista in camadas {
            for d in lista {
                let r = d.radius_px + pad as f32;
                let x0 = ((d.center[0] - r).max(0.0) as u32 / b) as usize;
                let x1 = ((d.center[0] + r).max(0.0) as u32 / b).min(nx as u32 - 1) as usize;
                let y0 = ((d.center[1] - r).max(0.0) as u32 / b) as usize;
                let y1 = ((d.center[1] + r).max(0.0) as u32 / b).min(ny as u32 - 1) as usize;
                for gy in y0..=y1 {
                    for gx in x0..=x1 {
                        marca[gy * nx + gx] = true;
                    }
                }
            }
        }
        let n = marca.iter().filter(|m| **m).count();
        // Cada bloco compõe-se com o avental dele: `(b + 2·pad)²`.
        let lado = f64::from(b + 2 * pad);
        out[bi] = n as f64 * lado * lado / bbox;
    }
    COBERTURA.with(|c| {
        let mut v = c.get();
        for (i, o) in out.iter().enumerate() {
            v[i] += o;
        }
        c.set(v);
    });
}

/// O que a pilha fez desde a última leitura — e ZERA.
pub(in crate::tool::paint) fn take() -> ([u64; 4], u64, f64) {
    (
        US.with(|c| c.take()),
        EVENTOS.with(Cell::take),
        AREA.with(Cell::take),
    )
}
