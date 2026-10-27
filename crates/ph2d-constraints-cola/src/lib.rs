//! A cola entre dois OBJECTOS: `docs/BlenderConstraints/05_W5_diferenciais.md` §2. Uma relação SIMÉTRICA (arrastar
//! qualquer um leva o outro), resolvida por GRUPO — componente conexo do grafo de colas — antes da pilha, que
//! continua de um sentido (ADR-0192). Tudo em poses MUNDIAIS; quem converte para o `Transform` local é o ECS.
//!
//! Árvore com um condutor = fecho analítico (§2.3); laço ou ≥ 2 condutores = `kcl-ezpz`.

#![forbid(unsafe_code)]

mod construcao;
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
    /// A MÃO de um gesto: quando o grupo não fecha com ela onde o gesto a pôs (um pino num chão com
    /// cadeado), ela CEDE — fica onde as colas deixam, o mais perto do gesto (a lei do arrasto da W4).
    pub cede: bool,
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
    resolve_com(&mut Cache::default(), colas, corpos)
}

/// A estrutura de cada grupo (sólidos, pinos nas raízes, a via e a receita) enquanto as colas, os objectos e quem
/// conduz não mudam. Só depende das entradas: com ou sem ela, o mesmo bit.
#[derive(Default)]
pub struct Cache {
    colas: Vec<Cola>,
    chaves: Vec<(Corpo, bool, bool)>,
    rank: BTreeMap<Corpo, usize>,
    grupos: Vec<Preparado>,
}

struct Preparado {
    membros: Vec<Corpo>,
    colas: Vec<Cola>,
    ids: Vec<u32>,
    normal: Result<grafo::Preparo, ()>,
    /// Há uma mão que pode ceder: o conflito tenta outra vez com ela como alvo.
    cede: bool,
}

impl Cache {
    fn serve(&self, colas: &[Cola], corpos: &BTreeMap<Corpo, Entrada>) -> bool {
        self.colas == colas
            && self.chaves.len() == corpos.len()
            && self
                .chaves
                .iter()
                .zip(corpos)
                .all(|(&(c, k, d), (&c2, e))| c == c2 && k == e.condutor && d == e.cede)
    }

    fn refaz(&mut self, colas: &[Cola], corpos: &BTreeMap<Corpo, Entrada>) {
        self.colas = colas.to_vec();
        self.chaves = corpos
            .iter()
            .map(|(&c, e)| (c, e.condutor, e.cede))
            .collect();
        self.rank = corpos.keys().enumerate().map(|(i, &c)| (c, i)).collect();
        self.grupos = grafo::grupos(colas, |c| corpos.contains_key(&c))
            .into_iter()
            .map(|g| {
                let do_grupo: Vec<Cola> = g.colas.iter().map(|&i| colas[i]).collect();
                Preparado {
                    normal: grafo::prepara(&g.membros, &do_grupo, corpos, &self.rank, false),
                    cede: g
                        .membros
                        .iter()
                        .any(|m| corpos[m].condutor && corpos[m].cede),
                    ids: do_grupo.iter().map(|c| c.id).collect(),
                    membros: g.membros,
                    colas: do_grupo,
                }
            })
            .collect();
    }
}

/// [`resolve`] com a estrutura guardada de um quadro para o outro (o passo do mundo).
#[must_use]
pub fn resolve_com(cache: &mut Cache, colas: &[Cola], corpos: &BTreeMap<Corpo, Entrada>) -> Saida {
    if !cache.serve(colas, corpos) {
        cache.refaz(colas, corpos);
    }
    let poses: Vec<Pose2> = corpos.values().map(|e| e.pose).collect();
    let mut todas = Vec::with_capacity(corpos.len());
    let mut grupos = Vec::with_capacity(cache.grupos.len());
    let mut r = grafo::Rascunho::default();
    for g in &cache.grupos {
        let feito = match g
            .normal
            .as_ref()
            .map_err(|_| ())
            .and_then(|p| grafo::executa(p, &poses, &mut r, &mut todas))
        {
            Err(()) if g.cede => grafo::prepara(&g.membros, &g.colas, corpos, &cache.rank, true)
                .and_then(|p| grafo::executa(&p, &poses, &mut r, &mut todas)),
            x => x,
        };
        let estado = match feito {
            Ok(true) => Estado::Definido,
            Ok(false) => Estado::Parado,
            Err(()) => Estado::Conflito,
        };
        grupos.push(Grupo {
            membros: g.membros.clone(),
            colas: g.ids.clone(),
            estado,
        });
    }
    Saida {
        poses: todas.into_iter().collect(),
        grupos,
    }
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
