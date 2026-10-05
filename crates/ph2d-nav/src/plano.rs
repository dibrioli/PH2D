//! ⭐ (W15) **O PLANO EM FATIAS** — o caminho de um agente ([`crate::agent`]) como uma procura que pode
//! parar a meio e continuar noutro tique (plano 30 §23.2).
//!
//! O plano é o de sempre: o ponto da malha mais perto do agente; com atalhos, o grafo deles
//! ([`crate::link`]); senão (ou sem caminho por eles), a procura até ao ponto mais perto do alvo na ilha
//! do agente — PARCIAL se o alvo está noutra. Cada procura pode parar ([`crate::polyanya::fatias`]).
//!
//! ⚠️ **O trabalho de um plano conta-se desde ZERO** (as contagens de quem emprestou os buffers ficam
//! de lado e voltam somadas): é o que deixa um scrub refazê-lo até ao mesmo ponto com buffers novos.

use crate::geom::{EPS, V2, dist};
use crate::link::{Atalhos, Hop, Query};
use crate::mesh::NavMesh;
use crate::polyanya::fatias::Custos;
use crate::polyanya::{Path, Polyanya, Stats};

/// O caminho de um agente: os pontos, os atalhos e se é PARCIAL (o alvo está noutra ilha); `None` =
/// sem caminho nenhum.
pub type Planeado = Option<(Vec<V2>, Vec<Hop>, bool)>;

/// ⭐ Uma procura de caminho A MEIO, com os buffers dela.
#[derive(Debug)]
pub struct Plano {
    search: Polyanya,
    /// As contagens de quem emprestou os buffers.
    emprestadas: Stats,
    pos: V2,
    /// O ponto da malha mais perto do agente, e o polígono dele.
    s: (V2, u32),
    t: V2,
    etapa: Etapa,
}

#[derive(Debug)]
enum Etapa {
    Atalhos(Atalhos),
    Directo { c: Custos, partial: bool },
}

impl Plano {
    /// Começa a procura do caminho de `pos` até `t`. `Err` = a resposta já, com os buffers de volta.
    ///
    /// # Errors
    /// A resposta, quando nenhuma procura fica a meio.
    pub fn begin(
        mesh: &NavMesh,
        mut search: Polyanya,
        q: &Query<'_>,
        pos: V2,
        t: V2,
    ) -> Result<Self, (Planeado, Polyanya)> {
        // Um agente empurrado para fora da malha volta pelo ponto mais perto dela.
        let Some(s) = mesh.nearest_point(pos, None) else {
            return Err((None, search));
        };
        let emprestadas = std::mem::take(&mut search.stats);
        // (Um grafo sem atalhos é só o lugar da etapa até se saber qual é.)
        let sem_atalhos = Query {
            costs: q.costs,
            links: &[],
        };
        let mut plano = Self {
            search,
            emprestadas,
            pos,
            s,
            t,
            etapa: Etapa::Atalhos(Atalhos::new(mesh, &sem_atalhos, s.0, t)),
        };
        // (W7) Com atalhos, o alvo pode estar noutra ilha e ser alcançável (um teleporte liga-as): o
        // grafo dos atalhos tenta o ponto mais perto dele em QUALQUER ilha; sem caminho, o de sempre.
        if !q.links.is_empty()
            && let Some((t_any, _)) = mesh.nearest_point(t, None)
        {
            plano.etapa = Etapa::Atalhos(Atalhos::new(mesh, q, s.0, t_any));
            return Ok(plano);
        }
        match plano.directo(mesh, q) {
            Some(r) => Err((r, plano.into_search())),
            None => Ok(plano),
        }
    }

    /// A procura sem atalhos até ao ponto mais perto do alvo na ilha do agente; `Some` = a resposta já.
    fn directo(&mut self, mesh: &NavMesh, q: &Query<'_>) -> Option<Planeado> {
        let island = mesh.island(self.s.1);
        // ⚠️ «Parcial» quer dizer OUTRA ilha — o alvo fora da malha por estar encostado a uma parede está
        // na mesma ilha (o ponto mais perto dele, em qualquer ilha, é desta).
        let Some((_, tp_any)) = mesh.nearest_point(self.t, None) else {
            return Some(None);
        };
        let partial = mesh.island(tp_any) != island;
        let Some((t_in, _)) = mesh.nearest_point(self.t, Some(island)) else {
            return Some(None);
        };
        match Custos::begin(&mut self.search, mesh, q.costs, self.s.0, t_in) {
            Err(r) => Some(self.fim_directo(r.ok(), partial)),
            Ok(c) => {
                self.etapa = Etapa::Directo { c, partial };
                None
            }
        }
    }

    fn fim_directo(&self, p: Option<Path>, partial: bool) -> Planeado {
        let mut pts = p?.points;
        if dist(pts[0], self.pos) > EPS {
            pts.insert(0, self.pos);
        }
        Some((pts, Vec::new(), partial))
    }

    /// Continua até à resposta, ou até o trabalho DESTE plano passar `ate` (`u64::MAX` = sem tecto).
    /// ⚠️ `q` tem de ser a do começo (a ponte recomeça o plano quando as entradas mudam).
    pub fn run(&mut self, mesh: &NavMesh, q: &Query<'_>, ate: u64) -> Option<Planeado> {
        loop {
            match &mut self.etapa {
                Etapa::Atalhos(a) => match a.run(mesh, &mut self.search, q, ate)? {
                    Some((mut pts, mut hops, _)) => {
                        if dist(pts[0], self.pos) > EPS {
                            pts.insert(0, self.pos);
                            hops.iter_mut().for_each(|h| h.at += 1);
                        }
                        return Some(Some((pts, hops, false)));
                    }
                    None => {
                        if let Some(r) = self.directo(mesh, q) {
                            return Some(r);
                        }
                    }
                },
                Etapa::Directo { c, partial } => {
                    let partial = *partial;
                    let r = c.run(&mut self.search, mesh, q.costs, ate)?;
                    return Some(self.fim_directo(r.ok(), partial));
                }
            }
        }
    }

    /// O trabalho deste plano até agora ([`Stats::work`]).
    #[must_use]
    pub fn trabalho(&self) -> u64 {
        self.search.stats.work()
    }

    /// Os buffers de volta, com as contagens de quem os emprestou somadas às deste plano.
    #[must_use]
    pub fn into_search(self) -> Polyanya {
        let mut search = self.search;
        let deste = search.stats;
        search.stats = self.emprestadas;
        search.stats.soma(&deste);
        search
    }
}

/// O plano inteiro, de uma vez (a procura síncrona): o caminho e o trabalho dele.
pub(crate) fn plan(
    mesh: &NavMesh,
    search: &mut Polyanya,
    q: &Query<'_>,
    pos: V2,
    t: V2,
) -> (Planeado, u64) {
    match Plano::begin(mesh, std::mem::take(search), q, pos, t) {
        Err((r, s)) => {
            *search = s;
            (r, 0)
        }
        Ok(mut p) => loop {
            if let Some(r) = p.run(mesh, q, u64::MAX) {
                let w = p.trabalho();
                *search = p.into_search();
                return (r, w);
            }
        },
    }
}
