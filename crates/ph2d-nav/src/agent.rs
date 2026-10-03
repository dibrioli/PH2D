//! ⭐⭐ **A CONDUÇÃO DE UM AGENTE** — o que ele faz NESTE tique (plano 30 §3, W3).
//!
//! A lei é pura: recebe *onde estou · onde está o alvo · a malha · a memória*, e devolve *para onde
//! quero ir* (uma direcção unitária, ou zero) e *o que aconteceu* (chegou · sem caminho · preso).
//! ⚠️ **Ela não move nada**: quem anda é o `TopDownPlayer` da ponte (§2.1 do plano), que acelera,
//! trava e desliza na parede — a navegação é a intenção, nunca a pose.
//!
//! # As cinco queixas da pesquisa que esta lei existe para não ter
//!
//! | queixa | o que a cura |
//! |---|---|
//! | **Q4** caminho vazio no 1.º quadro | a procura é síncrona: o 1.º tique já tem caminho |
//! | **Q5** a orbitar um ponto sem o apanhar | um ponto conta como alcançado a `velocidade · dt` |
//! | **Q6** recalcular a cada tique | só quando o alvo andou mais que [`AgentConfig::repath_distance`], quando o agente saiu do corredor, ou quando ficou preso |
//! | **Q8** o agente como obstáculo de si mesmo | a malha é de colisores ESTÁTICOS; o agente é cinemático |
//! | **Q12** os estados calados | [`Status`] diz qual, e cada TRANSIÇÃO é um [`Event`] |
//!
//! ⚠️ **Os eventos são de TRANSIÇÃO, nunca de estado** (a lei *«um evento lido como estado acerta
//! enquanto ninguém o apagar»*): `Arrived` sai no tique em que chegou, não em todos os tiques em que
//! está lá. Quem pinta o estado lê [`AgentRuntime::status`].

use crate::geom::{EPS, V2, dist, dist_to_segment, scale, sub};
use crate::link::{Hop, Query, plan_with_links};
use crate::mesh::NavMesh;
use crate::polyanya::{NoPath, Polyanya};

/// Os números de um agente — em METROS e segundos (o `Transform` já é metros).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AgentConfig {
    /// A esta distância do alvo o agente pára e conta como chegado.
    pub arrive_distance: f64,
    /// O alvo tem de andar MAIS que isto desde o último plano para valer um recálculo (Q6).
    pub repath_distance: f64,
    /// Sem progresso durante isto, o agente está PRESO (e recalcula).
    pub stuck_after_s: f64,
    /// A velocidade do executor (m/s) — mede o «alcançado» de um canto (Q5).
    pub speed: f64,
}

/// O que o agente está a fazer — o que o painel pinta.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Status {
    /// Sem alvo, ou desligado.
    #[default]
    Idle,
    /// A seguir um caminho até ao alvo.
    Moving,
    /// A seguir o caminho até ao ponto ALCANÇÁVEL mais perto de um alvo que não se alcança.
    MovingPartial,
    /// No alvo (a menos de [`AgentConfig::arrive_distance`]).
    Arrived,
    /// Não há caminho nenhum (nem parcial) — a malha está vazia ou o agente fora dela.
    NoPath,
}

/// Uma TRANSIÇÃO que vale um sinal (o nome é autorado no componente; vazio = calado).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    Arrived,
    /// Sem caminho — ou só um caminho PARCIAL (o alvo está noutra ilha).
    NoPath,
    /// O progresso parou durante [`AgentConfig::stuck_after_s`] — o caminho é recalculado no mesmo
    /// tique. ⚠️ «Preso» é um ACONTECIMENTO e não um estado: o agente recalcula e volta a `Moving`, e
    /// um estado `Stuck` que ninguém pode ver seria uma variante inalcançável.
    Stuck,
    /// (W7) Atravessou o atalho com este `id` (o da ponte). A lei devolve-o em
    /// [`Steer::crossed`]; a ponte fá-lo facto com os outros.
    Crossed(u32),
}

/// A memória de um agente entre tiques. ⚠️ Vive na ponte e entra no anel de checkpoints: um *scrub*
/// devolve o caminho, o ponto actual e os relógios EXACTOS daquele tique (S7).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AgentRuntime {
    /// O caminho em curso (o 1.º ponto é onde ele estava ao planear; o último, o destino).
    pub path: Vec<V2>,
    /// O índice do ponto para onde ele vai agora.
    pub next: usize,
    /// Onde o alvo estava quando o caminho foi planeado.
    pub planned_for: Option<V2>,
    pub status: Status,
    /// O caminho acaba no ponto alcançável mais perto de um alvo que está noutra ilha.
    pub partial: bool,
    /// A menor distância que falta já vista (o detector de «preso»).
    best_remaining: f64,
    stuck_clock: f64,
    /// Quantas vezes a procura correu (os gates de Q6 contam isto).
    pub searches: u64,
    /// (W7) Os ATALHOS do caminho em curso (o troço `path[at] → path[at + 1]` de cada um).
    pub hops: Vec<Hop>,
}

/// A resposta de um tique.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Steer {
    /// Para onde ir: unitária, ou zero (parado).
    pub dir: V2,
    pub event: Option<Event>,
    /// (W7) Um TELEPORTE: a ponte põe o corpo AQUI neste tique.
    pub teleport: Option<V2>,
    /// (W7) O atalho que o agente ATRAVESSOU neste tique (o `id` da ponte). ⚠️ Um campo à parte do
    /// `event`: atravessar e chegar podem acontecer no mesmo tique, e nenhum cala o outro.
    pub crossed: Option<u32>,
}

impl AgentRuntime {
    /// Esquece o caminho (a geometria mudou, ou o agente foi desligado): o próximo tique recalcula.
    pub fn forget_path(&mut self) {
        self.path.clear();
        self.hops.clear();
        self.next = 0;
        self.planned_for = None;
        self.partial = false;
    }

    /// O que falta andar pelo caminho, a partir de `pos` — o que o Inspector lê (*«3,2 m to go»*).
    #[must_use]
    pub fn remaining(&self, pos: V2) -> f64 {
        if self.next >= self.path.len() {
            return 0.0;
        }
        let mut d = dist(pos, self.path[self.next]);
        for (i, w) in self.path[self.next..].windows(2).enumerate() {
            // Um teleporte não se anda.
            if !self.hop_at(self.next + i).is_some_and(|h| h.teleport) {
                d += dist(w[0], w[1]);
            }
        }
        d
    }

    /// O atalho que começa no ponto `i` do caminho, se há.
    #[must_use]
    pub fn hop_at(&self, i: usize) -> Option<Hop> {
        self.hops.iter().copied().find(|h| h.at == i)
    }
}

/// **Um tique da condução.** `mesh = None` ⇒ o agente não está em região nenhuma; `target = None` ⇒
/// sem alvo.
pub fn step(
    rt: &mut AgentRuntime,
    mesh: Option<&NavMesh>,
    search: &mut Polyanya,
    pos: V2,
    target: Option<V2>,
    cfg: &AgentConfig,
    dt: f64,
) -> Steer {
    step_with(rt, mesh, search, &Query::default(), pos, target, cfg, dt)
}

/// ⭐ (W7) [`step`] com os custos das áreas e os atalhos ([`Query`]) — a MESMA condução; sem eles é
/// `step`, ao bit.
#[allow(clippy::too_many_arguments)]
pub fn step_with(
    rt: &mut AgentRuntime,
    mesh: Option<&NavMesh>,
    search: &mut Polyanya,
    q: &Query<'_>,
    pos: V2,
    target: Option<V2>,
    cfg: &AgentConfig,
    dt: f64,
) -> Steer {
    let prev = rt.status;
    let Some(t) = target else {
        rt.forget_path();
        return halt(rt, prev, Status::Idle, None);
    };
    let Some(mesh) = mesh.filter(|m| !m.polys().is_empty()) else {
        rt.forget_path();
        return halt(rt, prev, Status::NoPath, None);
    };
    if dist(pos, t) <= cfg.arrive_distance {
        return halt(rt, prev, Status::Arrived, None);
    }

    // ── Recalcular? (Q6: só por um dos quatro motivos) ────────────────────────
    // ⚠️ (W7) A meio de uma porta de um sentido o agente está FORA da malha (a porta é proibida para
    // quem não a atravessa): replanear ali levava-o para trás. Só o «preso» replaneia lá dentro.
    let numa_porta = rt.next >= 1 && rt.hop_at(rt.next - 1).is_some_and(|h| !h.teleport);
    let off_corridor = !numa_porta
        && rt.next >= 1
        && rt.next < rt.path.len()
        && dist_to_segment(rt.path[rt.next - 1], rt.path[rt.next], pos) > cfg.repath_distance;
    let target_moved = rt.planned_for.is_none()
        || (!numa_porta
            && rt
                .planned_for
                .is_some_and(|p| dist(p, t) > cfg.repath_distance));
    let stuck = cfg.stuck_after_s > 0.0 && rt.stuck_clock >= cfg.stuck_after_s;
    let mut stuck_event = None;
    if rt.path.is_empty() || target_moved || off_corridor || stuck {
        if stuck {
            stuck_event = Some(Event::Stuck);
        }
        rt.stuck_clock = 0.0;
        rt.best_remaining = f64::INFINITY;
        rt.searches += 1;
        rt.planned_for = Some(t);
        match plan(mesh, search, q, pos, t) {
            Some((path, hops, partial)) => {
                rt.path = path;
                rt.hops = hops;
                rt.partial = partial;
                rt.next = 1.min(rt.path.len().saturating_sub(1));
            }
            None => {
                rt.path.clear();
                rt.hops.clear();
                rt.next = 0;
                return halt(rt, prev, Status::NoPath, stuck_event);
            }
        }
    }

    // ── Avançar os pontos alcançados (Q5) — e os atalhos (W7) ─────────────────
    let accept = (cfg.speed * dt).max(EPS);
    let mut crossed = None;
    while rt.next + 1 < rt.path.len() && dist(pos, rt.path[rt.next]) <= accept {
        // A entrada de um TELEPORTE: o corpo salta para a saída neste tique.
        if let Some(h) = rt.hop_at(rt.next)
            && h.teleport
        {
            let saida = rt.path[rt.next + 1];
            rt.next += 1;
            rt.stuck_clock = 0.0;
            rt.best_remaining = f64::INFINITY;
            return Steer {
                dir: [0.0; 2],
                event: stuck_event,
                teleport: Some(saida),
                crossed: Some(h.link),
            };
        }
        // A saída de uma porta de um sentido: atravessou.
        if rt.next >= 1
            && let Some(h) = rt.hop_at(rt.next - 1)
            && !h.teleport
        {
            crossed = Some(h.link);
        }
        rt.next += 1;
    }
    let Some(&goal) = rt.path.get(rt.next) else {
        return halt(rt, prev, Status::NoPath, stuck_event);
    };
    let at_end = rt.next + 1 == rt.path.len() && dist(pos, goal) <= accept;
    if at_end {
        // O fim do caminho: o alvo, ou o ponto mais perto dele que esta ilha tem. ⚠️ Um alvo encostado
        // a uma parede (dentro do raio DESTE agente) não é «inalcançável»: o fim do caminho é o mais
        // perto que um corpo deste tamanho chega, e isso é CHEGAR.
        rt.stuck_clock = 0.0;
        let s = if rt.partial {
            Status::MovingPartial
        } else {
            Status::Arrived
        };
        let mut st = halt(rt, prev, s, stuck_event);
        st.crossed = crossed;
        return st;
    }

    // ── «Preso»: o que falta tem de DESCER ────────────────────────────────────
    let rem = rt.remaining(pos);
    // Um progresso conta quando desce mais que meio passo do executor — o ruído de um deslize na
    // parede não reinicia o relógio, e um agente que anda reinicia-o todo tique.
    if rem < rt.best_remaining - 0.5 * accept {
        rt.best_remaining = rem;
        rt.stuck_clock = 0.0;
    } else {
        rt.stuck_clock += dt;
    }

    let status = if rt.partial {
        Status::MovingPartial
    } else {
        Status::Moving
    };
    let event = stuck_event.or_else(|| transition(prev, status));
    rt.status = status;
    let l = dist(goal, pos);
    let dir = if l <= EPS {
        [0.0; 2]
    } else {
        scale(sub(goal, pos), 1.0 / l)
    };
    Steer {
        dir,
        event,
        teleport: None,
        crossed,
    }
}

/// Parado, num estado: a resposta e o evento da transição (se houve).
fn halt(rt: &mut AgentRuntime, prev: Status, s: Status, forced: Option<Event>) -> Steer {
    rt.status = s;
    Steer {
        dir: [0.0; 2],
        event: forced.or_else(|| transition(prev, s)),
        teleport: None,
        crossed: None,
    }
}

/// O evento que a passagem `prev → s` vale. ⚠️ `Moving ↔ MovingPartial ↔ NoPath` falam UMA vez à
/// entrada da família «sem caminho completo», não a cada troca dentro dela.
fn transition(prev: Status, s: Status) -> Option<Event> {
    if prev == s {
        return None;
    }
    let sem_caminho = |x: Status| matches!(x, Status::NoPath | Status::MovingPartial);
    match s {
        Status::Arrived => Some(Event::Arrived),
        _ if sem_caminho(s) && !sem_caminho(prev) => Some(Event::NoPath),
        _ => None,
    }
}

/// O caminho de `pos` até `t` (ou até ao ponto alcançável mais perto dele, `partial = true`), e os
/// atalhos dele. `None` só quando não há caminho nenhum.
fn plan(
    mesh: &NavMesh,
    search: &mut Polyanya,
    q: &Query<'_>,
    pos: V2,
    t: V2,
) -> Option<(Vec<V2>, Vec<Hop>, bool)> {
    // Um agente empurrado para fora da malha volta pelo ponto mais perto dela.
    let (s, sp) = mesh.nearest_point(pos, None)?;
    // (W7) Com atalhos, o alvo pode estar noutra ilha e ser alcançável (um teleporte liga-as): o
    // grafo dos atalhos tenta o ponto mais perto dele em QUALQUER ilha; sem caminho, o de sempre.
    if !q.links.is_empty()
        && let Some((t_any, _)) = mesh.nearest_point(t, None)
        && let Some((mut pts, mut hops, _)) = plan_with_links(mesh, search, q, s, t_any)
    {
        if dist(pts[0], pos) > EPS {
            pts.insert(0, pos);
            hops.iter_mut().for_each(|h| h.at += 1);
        }
        return Some((pts, hops, false));
    }
    let island = mesh.island(sp);
    // ⚠️ «Parcial» quer dizer OUTRA ilha — o alvo fora da malha por estar encostado a uma parede está
    // na mesma ilha (o ponto mais perto dele, em qualquer ilha, é desta).
    let (_, tp_any) = mesh.nearest_point(t, None)?;
    let partial = mesh.island(tp_any) != island;
    let (t_in, _) = mesh.nearest_point(t, Some(island))?;
    match search.find_path_costs(mesh, q.costs, s, t_in) {
        Ok(p) => {
            let mut pts = p.points;
            if dist(pts[0], pos) > EPS {
                pts.insert(0, pos);
            }
            Some((pts, Vec::new(), partial))
        }
        Err(NoPath::Unreachable | NoPath::StartOff | NoPath::TargetOff) => None,
    }
}

#[cfg(test)]
#[path = "agent_tests.rs"]
mod tests;
