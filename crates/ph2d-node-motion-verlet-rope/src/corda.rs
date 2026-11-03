//! ⭐ **A corda do nó FORA do grafo** — a MESMA lei ([`super::step`]: Verlet + distância + flexão),
//! presa nas DUAS pontas a alvos que andam, sem contacto e de perfil uniforme. É a corda que a UI
//! desenha entre um cartão solto e o objeto dele (T44, `docs/CartoesUI/01_decisoes_do_dono.md`).
//!
//! O espaço é o do nó: `y` para CIMA (a gravidade puxa `−y`). Quem desenha em ecrã inverte o `y`.

use super::{Params, step};

/// O feitio da corda (o que no nó são os params).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Feitio {
    /// Quantos pontos (≥ 2).
    pub pontos: usize,
    /// Aceleração da gravidade (unidades/s²).
    pub gravidade: f32,
    /// Passes de relaxação por passo.
    pub iteracoes: usize,
    /// `0..1` da velocidade perdida por passo.
    pub amortecimento: f32,
    /// `0..1` da resistência à dobra.
    pub flexao: f32,
}

/// Uma corda viva: pontos e a memória de Verlet.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Corda {
    pos: Vec<[f32; 2]>,
    prev: Vec<[f32; 2]>,
}

impl Corda {
    /// Os pontos de agora, de `a` a `b`.
    #[must_use]
    pub fn pontos(&self) -> &[[f32; 2]] {
        &self.pos
    }

    /// Um passo de `dt` segundos com as pontas em `a` e `b` e o comprimento `comprimento`. A
    /// primeira vez (ou outro número de pontos) semeia a corda RETA de `a` a `b`, parada.
    pub fn passo(&mut self, a: [f32; 2], b: [f32; 2], comprimento: f32, dt: f32, f: &Feitio) {
        let n = f.pontos.max(2);
        if self.pos.len() != n {
            #[allow(clippy::cast_precision_loss)]
            let reta = (0..n).map(|i| {
                let t = i as f32 / (n - 1) as f32;
                [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
            });
            self.pos = reta.collect();
            self.prev = self.pos.clone();
        }
        #[allow(clippy::cast_precision_loss)]
        let p = Params {
            count: n,
            seg_rest: comprimento / (n - 1) as f32,
            rest: None,
            gravity: f.gravidade,
            iterations: f.iteracoes,
            damping: f.amortecimento,
            pin_tail: true,
            bend: f.flexao,
            substeps: 1,
        };
        let pos = std::mem::take(&mut self.pos);
        let (pos, prev) = step(pos, &self.prev, &[], &[], a, b, dt, &p, None);
        self.pos = pos;
        self.prev = prev;
    }
}

#[cfg(test)]
#[path = "corda_tests.rs"]
mod tests;
