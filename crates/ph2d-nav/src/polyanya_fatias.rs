//! ⭐ (W15) **A procura em FATIAS** — a MESMA procura, parada entre dois `pop` do heap e continuada
//! noutro tique (plano 30 §23.2, o idioma do Detour `updateSlicedFindPath`).
//!
//! ⚠️ **A pausa é uma função do TRABALHO, e só do trabalho:** ela acontece logo DEPOIS de um `pop` em
//! que o trabalho desta procura ([`super::Stats::work`], contado desde zero) PASSOU o tecto (um tecto `W`
//! paga uma procura de `W`: o `pop` do alvo não custa nada). O trabalho cresce ESTRITAMENTE a cada `pop`
//! que não acaba a procura (um nó expandido soma `1`; uma raiz materializada, `≥ 4`) ⇒ correr de uma vez
//! com o tecto `W − 1`, sendo `W` o trabalho em que uma corrida em fatias parou, dá o MESMO estado, `pop`
//! a `pop`. É o que deixa um scrub refazer uma procura a meio sem a guardar no anel.

use std::cmp::Reverse;

use super::{Kind, NONE, NoPath, Node, Path, Polyanya, Root};
use crate::cost::{cost_of, path_cost};
use crate::geom::{EPS, V2, dist};
use crate::mesh::NavMesh;

/// O que uma fatia deu.
pub(crate) enum Fatia {
    Feita(Result<Path, NoPath>),
    /// O trabalho chegou ao tecto antes da resposta.
    Falta,
}

impl Polyanya {
    /// Prepara a procura de `s` a `t`; `Some` quando a resposta não precisa de nenhum `pop` (fora da
    /// malha, ilhas diferentes, o mesmo polígono).
    pub(super) fn begin(
        &mut self,
        mesh: &NavMesh,
        costs: &[f64],
        s: V2,
        t: V2,
    ) -> Option<Result<Path, NoPath>> {
        self.reset(mesh);
        self.alvo = t;
        self.costs.clear();
        self.costs.extend_from_slice(costs);
        self.wmin = costs.iter().copied().fold(1.0, f64::min);
        let mut ps = Vec::new();
        mesh.locate_all(s, &mut ps);
        if ps.is_empty() {
            return Some(Err(NoPath::StartOff));
        }
        mesh.locate_all(t, &mut self.target_polys);
        if self.target_polys.is_empty() {
            return Some(Err(NoPath::TargetOff));
        }
        // No mesmo polígono: a direito (convexo), ao custo do mais barato que tem os dois. ⚠️ (W7) Só
        // é a resposta se nenhuma área custa MENOS: dentro da lama, sair e contornar pode ser mais
        // barato (medido) — então a recta é uma candidata no heap, como qualquer outra.
        let directo = ps
            .iter()
            .filter(|p| self.target_polys.binary_search(p).is_ok())
            .map(|&p| self.cost(mesh, p))
            .reduce(f64::min);
        if let Some(c) = directo
            && c <= self.wmin
        {
            return Some(Ok(Path {
                points: vec![s, t],
                length: dist(s, t),
                cost: c * dist(s, t),
            }));
        }
        let same_island = directo.is_some()
            || ps.iter().any(|&a| {
                self.target_polys
                    .iter()
                    .any(|&b| mesh.island(a) == mesh.island(b))
            });
        if !same_island {
            return Some(Err(NoPath::Unreachable));
        }

        self.roots.push(Root {
            p: s,
            g: 0.0,
            prev: NONE,
            w_in: 0.0,
            range: None,
        });
        if let Some(c) = directo {
            self.push(
                c * dist(s, t),
                Node {
                    root: 0,
                    kind: Kind::Final { via: None },
                    w: c,
                },
            );
        }
        for &p in &ps {
            self.push_from_point(mesh, 0, p, None, t);
        }
        None
    }

    /// Continua a procura até à resposta, ou até o trabalho passar `ate` (`u64::MAX` = sem tecto).
    pub(super) fn resume(&mut self, mesh: &NavMesh, ate: u64) -> Fatia {
        let t = self.alvo;
        while let Some(Reverse((k, _, ni))) = self.open.pop() {
            let node = self.nodes[ni as usize];
            match node.kind {
                Kind::Final { via } => {
                    return Fatia::Feita(Ok(self.reconstruct(mesh, node.root, via, node.w, t)));
                }
                Kind::Interval {
                    poly,
                    entry,
                    left,
                    right,
                } => {
                    self.stats.expanded += 1;
                    if let Some(d) = self.diag.as_mut() {
                        let b = if k >> 63 == 1 { k & !(1 << 63) } else { !k };
                        let r = self.roots[node.root as usize];
                        let refrata = cost_of(&self.costs, mesh.area_id(poly)) != node.w;
                        d.push(super::Expandido {
                            f: f64::from_bits(b),
                            w: node.w,
                            poly,
                            entry,
                            rho: r.p,
                            g: r.g,
                            left,
                            right,
                            refrata,
                            ponderada: self.dominancia || self.sem_dominancia,
                        });
                    }
                    self.expand(mesh, node.root, poly, entry, left, right, node.w, t);
                }
                Kind::Pending { idx } => {
                    self.stats.pending += 1;
                    self.materialize(mesh, idx, t);
                }
            }
            if ate != u64::MAX && self.stats.work() > ate {
                return Fatia::Falta;
            }
        }
        Fatia::Feita(Err(NoPath::Unreachable))
    }
}

/// ⭐ A [`Polyanya::find_path_costs`] em fatias: a procura inteira quando os custos não distinguem
/// nada; senão a GERAL e, se a lama tocar o caminho dela, a PONDERADA — cada uma pode parar a meio.
#[derive(Debug)]
pub(crate) struct Custos {
    s: V2,
    t: V2,
    fase: Fase,
}

#[derive(Debug)]
enum Fase {
    Inteira,
    Geral { wmin: f64 },
    Ponderada { geral: Path },
}

impl Custos {
    /// Começa; `Err` = a resposta, já (sem nenhum `pop`).
    pub(crate) fn begin(
        search: &mut Polyanya,
        mesh: &NavMesh,
        costs: &[f64],
        s: V2,
        t: V2,
    ) -> Result<Self, Result<Path, NoPath>> {
        search.stats.searches += 1;
        let wmin = costs.iter().copied().fold(1.0, f64::min);
        // (W14) Uma tabela toda a `1` (a da ponte sem lama é `[1.0]`) é uniforme sem varrer a malha.
        let uniforme = costs.iter().all(|&c| c == 1.0)
            || (0..mesh.poly_count() as u32).all(|p| cost_of(costs, mesh.area_id(p)) == wmin);
        if uniforme {
            return match search.begin(mesh, costs, s, t) {
                Some(r) => Err(r),
                None => Ok(Self {
                    s,
                    t,
                    fase: Fase::Inteira,
                }),
            };
        }
        match search.begin(mesh, &[], s, t) {
            Some(Err(e)) => Err(Err(e)),
            Some(Ok(p0)) => Self::depois_da_geral(search, mesh, costs, (s, t), wmin, p0),
            None => Ok(Self {
                s,
                t,
                fase: Fase::Geral { wmin },
            }),
        }
    }

    /// A geral acabou: se ela não toca nada mais caro que o mínimo, é o óptimo; senão começa a
    /// ponderada.
    fn depois_da_geral(
        search: &mut Polyanya,
        mesh: &NavMesh,
        costs: &[f64],
        (s, t): (V2, V2),
        wmin: f64,
        p0: Path,
    ) -> Result<Self, Result<Path, NoPath>> {
        let c0 = path_cost(mesh, costs, &p0.points).unwrap_or(f64::INFINITY);
        let geral = Path { cost: c0, ..p0 };
        if c0 <= wmin * geral.length * (1.0 + 1e-12) + EPS {
            return Err(Ok(geral));
        }
        search.dominancia = !search.sem_dominancia;
        let r = search.begin(mesh, costs, s, t);
        search.dominancia = false;
        match r {
            Some(p) => Err(Ok(escolhe(p, geral))),
            None => Ok(Self {
                s,
                t,
                fase: Fase::Ponderada { geral },
            }),
        }
    }

    /// Continua até à resposta, ou até o trabalho passar `ate`.
    pub(crate) fn run(
        &mut self,
        search: &mut Polyanya,
        mesh: &NavMesh,
        costs: &[f64],
        ate: u64,
    ) -> Option<Result<Path, NoPath>> {
        loop {
            match &self.fase {
                Fase::Inteira => {
                    return match search.resume(mesh, ate) {
                        Fatia::Feita(r) => Some(r),
                        Fatia::Falta => None,
                    };
                }
                &Fase::Geral { wmin } => match search.resume(mesh, ate) {
                    Fatia::Falta => return None,
                    Fatia::Feita(Err(e)) => return Some(Err(e)),
                    Fatia::Feita(Ok(p0)) => {
                        match Self::depois_da_geral(search, mesh, costs, (self.s, self.t), wmin, p0)
                        {
                            Err(r) => return Some(r),
                            Ok(seguinte) => *self = seguinte,
                        }
                    }
                },
                Fase::Ponderada { geral } => {
                    search.dominancia = !search.sem_dominancia;
                    let r = search.resume(mesh, ate);
                    search.dominancia = false;
                    return match r {
                        Fatia::Falta => None,
                        Fatia::Feita(p) => Some(Ok(escolhe(p, geral.clone()))),
                    };
                }
            }
        }
    }
}

/// A ponderada só ganha se custa MENOS que a geral.
fn escolhe(ponderado: Result<Path, NoPath>, geral: Path) -> Path {
    match ponderado {
        Ok(p) if p.cost < geral.cost => p,
        _ => geral,
    }
}
