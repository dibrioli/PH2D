//! A cola entre dois OBJECTOS: `docs/BlenderConstraints/05_W5_diferenciais.md` §2. Uma relação SIMÉTRICA (arrastar
//! qualquer um leva o outro), resolvida por GRUPO — componente conexo do grafo de colas — antes da pilha, que
//! continua de um sentido (ADR-0192). Tudo em poses MUNDIAIS; quem converte para o `Transform` local é o ECS.
//!
//! Árvore com um condutor = fecho analítico (§2.3); laço ou ≥ 2 condutores = `kcl-ezpz`.

#![forbid(unsafe_code)]

mod ez;
mod grafo;

use std::collections::BTreeMap;

pub use ph2d_constraints::Pose2;
use serde::{Deserialize, Serialize};

/// A referência durável de um objecto (o `u64` do `StableId`).
pub type Corpo = u64;

/// Erro de composição `f64` é ~1e-15 relativo; seis ordens de folga para dizer «a mesma pose».
pub const TOL: f64 = 1e-9;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Tipo {
    /// A pose de `b` em `a` no momento de colar: `b = rel.under(a)`.
    Rigida { rel: Pose2 },
    /// O ponto do pino nas coordenadas locais de cada um; nada sobre a rotação.
    Pino { em_a: [f64; 2], em_b: [f64; 2] },
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Cola {
    pub id: u32,
    pub a: Corpo,
    pub b: Corpo,
    pub tipo: Tipo,
}

impl Cola {
    /// Solda `b` a `a` na pose relativa de agora (a conta do Child Of `keep_current`).
    #[must_use]
    pub fn rigida(id: u32, a: Corpo, b: Corpo, pa: Pose2, pb: Pose2) -> Option<Self> {
        let rel = pb.relative_to(pa)?;
        Some(Self {
            id,
            a,
            b,
            tipo: Tipo::Rigida { rel },
        })
    }

    /// Prende `a` e `b` no ponto `mundo` (onde o artista clicou).
    #[must_use]
    pub fn pino(
        id: u32,
        a: Corpo,
        b: Corpo,
        pa: Pose2,
        pb: Pose2,
        mundo: [f64; 2],
    ) -> Option<Self> {
        let em_a = local(pa, mundo)?;
        let em_b = local(pb, mundo)?;
        Some(Self {
            id,
            a,
            b,
            tipo: Tipo::Pino { em_a, em_b },
        })
    }

    #[must_use]
    pub fn toca(&self, c: Corpo) -> bool {
        self.a == c || self.b == c
    }

    /// O parceiro de `c` nesta cola.
    #[must_use]
    pub fn outro(&self, c: Corpo) -> Option<Corpo> {
        if self.a == c {
            Some(self.b)
        } else if self.b == c {
            Some(self.a)
        } else {
            None
        }
    }
}

/// O ponto `mundo` nas coordenadas locais de `p`.
#[must_use]
pub fn local(p: Pose2, mundo: [f64; 2]) -> Option<[f64; 2]> {
    Some(p.to_xform().inverse()?.apply(mundo))
}

/// Um membro: a pose MUNDIAL deste quadro (autorada, ou a da mão) e se ele CONDUZ (mão, curva, pilha, cadeado).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Entrada {
    pub pose: Pose2,
    pub condutor: bool,
}

/// O estado de um grupo — a cor do CAD no cartão.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Estado {
    /// Ninguém conduz: o documento já é coerente, nada a fazer.
    Parado,
    Definido,
    /// Condutores que discordam, laço que não fecha ou solda incoerente: cada um fica na sua pose.
    Conflito,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Grupo {
    pub membros: Vec<Corpo>,
    pub colas: Vec<u32>,
    pub estado: Estado,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Saida {
    /// A pose MUNDIAL de cada membro de um grupo `Definido`; condutores saem com a sua pose ao bit.
    pub poses: BTreeMap<Corpo, Pose2>,
    pub grupos: Vec<Grupo>,
}

/// ⭐ **A porta única da cola.** As colas cujo objecto falta em `corpos` (ou que colam um objecto a si mesmo) são
/// ignoradas.
#[must_use]
pub fn resolve(colas: &[Cola], corpos: &BTreeMap<Corpo, Entrada>) -> Saida {
    let mut saida = Saida::default();
    for g in grafo::grupos(colas, |c| corpos.contains_key(&c)) {
        let do_grupo: Vec<Cola> = g.colas.iter().map(|&i| colas[i]).collect();
        let estado = match grafo::resolve_grupo(&g.membros, &do_grupo, corpos) {
            Ok(Some(poses)) => {
                saida.poses.extend(poses);
                Estado::Definido
            }
            Ok(None) => Estado::Parado,
            Err(()) => Estado::Conflito,
        };
        saida.grupos.push(Grupo {
            membros: g.membros,
            colas: do_grupo.iter().map(|c| c.id).collect(),
            estado,
        });
    }
    saida
}

/// Duas poses iguais a menos de [`TOL`] (ângulo pelo arco curto).
#[must_use]
pub fn perto(p: Pose2, q: Pose2) -> bool {
    let d = (p.rot - q.rot).rem_euclid(std::f64::consts::TAU);
    let da = d.min(std::f64::consts::TAU - d);
    let esc = 1.0 + p.x.abs().max(p.y.abs());
    (p.x - q.x).abs() <= TOL * esc
        && (p.y - q.y).abs() <= TOL * esc
        && da <= TOL
        && (p.sx - q.sx).abs() <= TOL
        && (p.sy - q.sy).abs() <= TOL
}
