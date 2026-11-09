//! **Color cycling** (P13): os índices de uma rampa rodam no tempo; as células não mudam, muda
//! o índice que se MOSTRA. Só faz sentido em Indexed (quem compõe decide).

use crate::ramp::{PaletteRamps, RampId};

/// Para onde as cores andam ao longo da rampa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CycleDirection {
    /// A cor da posição `k` passa à posição `k + 1`.
    #[default]
    Forward,
    Reverse,
    /// Vai e volta: `0, 1, …, n−1, n−2, …, 1`.
    PingPong,
}

impl CycleDirection {
    #[must_use]
    pub const fn code(self) -> u8 {
        match self {
            Self::Forward => 0,
            Self::Reverse => 1,
            Self::PingPong => 2,
        }
    }
    #[must_use]
    pub const fn from_code(c: u8) -> Option<Self> {
        match c {
            0 => Some(Self::Forward),
            1 => Some(Self::Reverse),
            2 => Some(Self::PingPong),
            _ => None,
        }
    }
}

/// Um ciclo sobre uma rampa. `periodo_ms` é a volta INTEIRA: `ciclo_em(t + periodo) = ciclo_em(t)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cycle {
    pub ramp: RampId,
    pub periodo_ms: u32,
    pub direcao: CycleDirection,
}

/// A permutação do instante `t_ms`: `perm[i]` é o índice que a célula de índice `i` MOSTRA.
/// Índice fora de toda rampa com ciclo fica igual.
#[must_use]
pub fn ciclo_em(pal: &PaletteRamps, t_ms: u64) -> [u8; 256] {
    let mut perm: [u8; 256] = std::array::from_fn(|i| i as u8);
    for c in &pal.cycles {
        let Some(r) = pal.ramp(c.ramp) else { continue };
        let n = r.indices.len() as u64;
        if n < 2 || c.periodo_ms == 0 {
            continue;
        }
        let period = u64::from(c.periodo_ms);
        let steps = match c.direcao {
            CycleDirection::PingPong => 2 * (n - 1),
            CycleDirection::Forward | CycleDirection::Reverse => n,
        };
        let s = (t_ms % period) * steps / period;
        let off = match c.direcao {
            CycleDirection::Forward => s,
            CycleDirection::Reverse => (n - s) % n,
            CycleDirection::PingPong => s.min(steps - s),
        };
        for (k, &i) in r.indices.iter().enumerate() {
            let from = (k as u64 + n - off) % n;
            perm[usize::from(i)] = r.indices[from as usize];
        }
    }
    perm
}
