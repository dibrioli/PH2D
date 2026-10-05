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
use crate::link::{Hop, Query};
use crate::mesh::NavMesh;
use crate::plano::{Planeado, Plano, plan};
use crate::polyanya::Polyanya;

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
    /// O raio do CORPO (m) — a entrada de um atalho alcança-se quando fica DENTRO dele (ver
    /// [`alcance_de_atalho`]); `0` = um ponto (só a régua de um canto).
    pub radius: f64,
}

/// ⭐ (report do dono, 05/10) **A entrada de um ATALHO alcança-se quando fica DENTRO do corpo** (a um
/// raio), e não no centro: dois agentes iguais que vão ao mesmo portal encostam-se um de cada lado do
/// ponto, e para lá chegar cada um teria de se sobrepor ao outro — o desvio não deixa, e ficavam os dois
/// parados para sempre. Medido na cena `=4` (o 2.º inimigo a nascer em `135` sítios): no centro, `44`
/// presos; a `0,5` raio, `1`; a `0,75`, `0`; a um raio e a dois, `0` — fica UM (o limiar com folga, e o
/// que a palavra diz: o corpo está em cima do portal). Um canto continua a alcançar-se no passo do
/// executor (Q5): só o atalho é uma porta que se ATRAVESSA, não um sítio por onde se passa.
#[must_use]
pub fn alcance_de_atalho(cfg: &AgentConfig, passo: f64) -> f64 {
    cfg.radius.max(passo)
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
    /// (W7) «Chegou» já foi anunciado para esta aproximação — ver [`step_with`] (a chegada que se
    /// anuncia UMA vez).
    pub arrival_told: bool,
    /// (W9) Há quantos tiques a malha mudou debaixo de um caminho que este agente continua a andar
    /// (`0` = em dia) — a fila do replaneio ([`crate::refresh`]).
    pub owed: u32,
    /// (W9) O caminho em curso já não se anda na malha nova: passa à frente na fila.
    pub broken: bool,
    /// (W9) O trabalho da última procura ([`crate::Stats::work`], W14) — a estimativa do que a próxima
    /// custa.
    pub last_work: u64,
    /// ⭐ (W15) A procura que este agente tem A MEIO (o estado dela fica na ponte, fora do anel).
    pub a_meio: Option<AMeio>,
    /// (W15) Quantas vezes SEGUIDAS a procura a meio recomeçou (com trabalho feito) porque as entradas
    /// mudaram: cada uma DOBRA a fatia da seguinte ([`fatia_depois_de`]) — uma malha que nunca pára não
    /// a deixa sem acabar, e o excesso cresce aos poucos.
    pub recomecos: u32,
}

/// (W15) **A fatia de uma procura que já recomeçou `k` vezes seguidas:** `pode · 2^k`. ⚠️ Medido e
/// recusado, nesta ordem (plano 30 §23.5): recomeçar INTEIRA ao 1.º recomeço (`47` procuras e `5,1 M` de
/// trabalho num tique, a `50` agentes na lama — uma porta recomeça TODAS as procuras a meio daquela
/// malha); e ao 3.º (picos de `150 000`: na cena de stress há procuras de `200–360 000`, que precisam de
/// mais tiques do que a porta da sonda lhes dá).
#[must_use]
pub fn fatia_depois_de(pode: u64, recomecos: u32) -> u64 {
    pode.saturating_mul(1u64.checked_shl(recomecos.min(63)).unwrap_or(u64::MAX))
}

/// ⭐ (W15) **Uma procura A MEIO, descrita** — o que entra no anel. Refazê-la desde o começo até ao
/// mesmo `trabalho` dá o MESMO estado (plano 30 §23.2), com as mesmas entradas.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AMeio {
    /// Onde o agente estava e onde estava o alvo quando ela começou.
    pub pos: V2,
    pub alvo: V2,
    /// O trabalho já feito ([`Plano::trabalho`]).
    pub trabalho: u64,
    /// A assinatura das entradas com que começou ([`Vez::entradas`]).
    pub entradas: u64,
}

/// ⭐ (W15) **A vez de procurar deste agente, NESTE tique** — o que a ponte lhe dá e o que ele gasta.
pub struct Vez<'a> {
    /// O trabalho que as procuras deste agente podem gastar agora ([`crate::Stats::work`]); `0` = não é a
    /// vez dele (a fila deve-lha, e ele anda o caminho que tem); `u64::MAX` = sem tecto.
    pub pode: u64,
    /// O caminho em curso tem de ser refeito (a malha mudou debaixo dele, e a fila serviu-o).
    pub refazer: bool,
    /// A assinatura das entradas da procura (a malha, os custos, os atalhos): uma procura a meio de
    /// outras entradas recomeça.
    pub entradas: u64,
    /// A procura a meio deste agente; `None` com [`AgentRuntime::a_meio`] posto (depois de um scrub)
    /// refaz-se até ao mesmo ponto.
    pub plano: &'a mut Option<Plano>,
    /// A resposta da procura a meio, já acabada FORA da condução (a ponte avança-as em paralelo,
    /// [`advance_mid`]).
    pub pronto: Option<Planeado>,
    /// (report do dono, 05/10) A saída de um teletransporte está livre para o corpo deste agente?
    /// `None` = sempre (a lei sem mundo); ocupada, ele espera na entrada.
    pub saida_livre: Option<&'a dyn Fn(V2) -> bool>,
    /// Saída: o trabalho que as procuras gastaram nesta condução.
    pub gasto: u64,
}

impl Vez<'_> {
    /// Sem tecto: a procura inteira, no tique (a condução de antes da W15).
    pub fn sem_tecto(plano: &mut Option<Plano>) -> Vez<'_> {
        Vez {
            pode: u64::MAX,
            refazer: false,
            entradas: 0,
            plano,
            pronto: None,
            saida_livre: None,
            gasto: 0,
        }
    }
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
    let mut plano = None;
    let mut vez = Vez::sem_tecto(&mut plano);
    step_in_turn(rt, mesh, search, q, &mut vez, pos, target, cfg, dt)
}

/// ⭐⭐ (W15) [`step_with`] com a VEZ de procurar ([`Vez`]): a procura paga do orçamento do tique, pára
/// a meio quando ele acaba e continua no tique seguinte; enquanto espera, o agente anda o caminho que
/// tem (sem caminho, fica parado). Sem tecto é `step_with`, ao bit.
#[allow(clippy::too_many_arguments)]
pub fn step_in_turn(
    rt: &mut AgentRuntime,
    mesh: Option<&NavMesh>,
    search: &mut Polyanya,
    q: &Query<'_>,
    vez: &mut Vez<'_>,
    pos: V2,
    target: Option<V2>,
    cfg: &AgentConfig,
    dt: f64,
) -> Steer {
    let prev = rt.status;
    let Some(t) = target else {
        rt.forget_path();
        larga(rt, vez);
        return halt(rt, prev, Status::Idle, None);
    };
    let Some(mesh) = mesh.filter(|m| m.poly_count() != 0) else {
        rt.forget_path();
        larga(rt, vez);
        return halt(rt, prev, Status::NoPath, None);
    };
    // (W7) Longe do alvo mais que chegada + recálculo (a régua de «mudou o bastante» do Q6), a
    // próxima chegada é outra e volta a anunciar-se.
    if dist(pos, t) > cfg.arrive_distance + cfg.repath_distance {
        rt.arrival_told = false;
    }
    if dist(pos, t) <= cfg.arrive_distance {
        larga(rt, vez);
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
    // (W15) Uma procura a meio de outras entradas recomeça — e a seguinte corre inteira.
    if let Some(a) = rt.a_meio
        && a.entradas != vez.entradas
    {
        larga(rt, vez);
        // Só conta a que já tinha trabalho feito (a que ainda não começou não perdeu nada).
        if a.trabalho > 0 {
            rt.recomecos = rt.recomecos.saturating_add(1);
        }
    }
    let quer = rt.path.is_empty() || target_moved || off_corridor || stuck || vez.refazer;
    // ⚠️ Com uma procura a meio os motivos esperam por ela: o alvo que anda não a recomeça (acabaria
    // nunca), e depois dela o recálculo do Q6 decide outra vez.
    let mut pronto: Option<Planeado> = vez.pronto.take();
    if pronto.is_none() && rt.a_meio.is_none() && quer {
        if stuck {
            stuck_event = Some(Event::Stuck);
        }
        rt.stuck_clock = 0.0;
        rt.best_remaining = f64::INFINITY;
        rt.searches += 1;
        rt.planned_for = Some(t);
        if vez.pode == u64::MAX {
            let (r, w) = plan(mesh, search, q, pos, t);
            (pronto, rt.last_work) = (Some(r), w);
            vez.gasto = vez.gasto.saturating_add(w);
        } else {
            match Plano::begin(mesh, search, q, pos, t) {
                Err(r) => (pronto, rt.last_work) = (Some(r), 0),
                Ok(p) => {
                    // A procura abre-se já (o que não custa nada acaba aqui); sem a vez dele, a fila
                    // deve-lha, e ele anda o que tem.
                    *vez.plano = Some(p);
                    rt.a_meio = Some(AMeio {
                        pos,
                        alvo: t,
                        trabalho: 0,
                        entradas: vez.entradas,
                    });
                    rt.owed = rt.owed.max(1);
                }
            }
        }
    }
    if pronto.is_none() && rt.a_meio.is_some() && vez.pode > 0 {
        let (r, w) = advance_mid(rt, mesh, q, vez.plano, vez.pode);
        vez.gasto = vez.gasto.saturating_add(w);
        if let Some((r, buffers)) = r {
            *search = buffers;
            pronto = Some(r);
        }
    }
    if let Some(planeado) = pronto {
        // (W9) Um caminho novo, por qualquer motivo, salda a dívida da fila.
        (rt.owed, rt.broken, rt.recomecos) = (0, false, 0);
        match planeado {
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
    } else if rt.path.is_empty() {
        // À espera do 1.º caminho: parado, no estado em que estava.
        return Steer {
            event: stuck_event,
            ..Steer::default()
        };
    }

    // ── Avançar os pontos alcançados (Q5) — e os atalhos (W7) ─────────────────
    let accept = (cfg.speed * dt).max(EPS);
    let mut crossed = None;
    let alcance = |rt: &AgentRuntime| {
        if rt.hop_at(rt.next).is_some() {
            alcance_de_atalho(cfg, accept)
        } else {
            accept
        }
    };
    while rt.next + 1 < rt.path.len() && dist(pos, rt.path[rt.next]) <= alcance(rt) {
        // A entrada de um TELEPORTE: o corpo salta para a saída neste tique.
        if let Some(h) = rt.hop_at(rt.next)
            && h.teleport
        {
            let saida = rt.path[rt.next + 1];
            // ⭐ (report do dono, 05/10) Com a saída OCUPADA espera em cima da entrada — sair dentro de
            // outro corpo prendia-o (o herói). Esperar não é estar preso: o relógio do «preso» só anda
            // mais abaixo, e daqui volta-se antes dele.
            if vez.saida_livre.is_some_and(|livre| !livre(saida)) {
                return Steer {
                    event: stuck_event,
                    ..Steer::default()
                };
            }
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
    let mut event = forced.or_else(|| transition(prev, s));
    // ⭐ (W7) **A chegada anuncia-se UMA vez por aproximação.** Medido na cena `=4`: dois inimigos
    // encostados ao mesmo herói empurram-se, e o empurrado saía e voltava a «chegou» — 6 sinais em
    // 15 s. Ele continua a fechar o espaço (o movimento não muda); o anúncio volta só depois de ele
    // ter ido MESMO embora (`step_with`).
    if event == Some(Event::Arrived) {
        if rt.arrival_told {
            event = None;
        }
        rt.arrival_told = true;
    }
    Steer {
        dir: [0.0; 2],
        event,
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

/// (W15) Esquece a procura a meio (o agente chegou, ficou sem alvo ou sem malha, ou as entradas mudaram).
fn larga(rt: &mut AgentRuntime, vez: &mut Vez<'_>) {
    rt.a_meio = None;
    *vez.plano = None;
}

/// ⭐ (W15) **Avança a procura a meio de um agente** com `pode` de trabalho — refeita até ao mesmo ponto
/// se `plano` está vazio (um scrub; o que ela refaz não conta). Devolve a resposta e os buffers, se ela
/// acabou, e o trabalho gasto. Só toca o agente e o plano dele: a ponte corre várias em PARALELO.
pub fn advance_mid(
    rt: &mut AgentRuntime,
    mesh: &NavMesh,
    q: &Query<'_>,
    plano: &mut Option<Plano>,
    pode: u64,
) -> (Option<(Planeado, Polyanya)>, u64) {
    let Some(a) = rt.a_meio else {
        return (None, 0);
    };
    if plano.is_none() {
        let mut novos = Polyanya::new();
        match Plano::begin(mesh, &mut novos, q, a.pos, a.alvo) {
            Err(r) => {
                rt.a_meio = None;
                return (Some((r, novos)), 0);
            }
            Ok(mut p) => {
                // Até ao MESMO `pop`: o primeiro em que o trabalho passou `trabalho − 1`.
                if a.trabalho > 0
                    && let Some(r) = p.run(mesh, q, a.trabalho - 1)
                {
                    rt.a_meio = None;
                    rt.last_work = p.trabalho();
                    return (Some((r, p.into_search())), 0);
                }
                *plano = Some(p);
            }
        }
    }
    let Some(p) = plano.as_mut() else {
        return (None, 0);
    };
    let antes = p.trabalho();
    let tecto = antes.saturating_add(fatia_depois_de(pode, rt.recomecos));
    let r = p.run(mesh, q, tecto);
    let gasto = p.trabalho() - antes;
    match r {
        Some(r) => {
            rt.last_work = p.trabalho();
            rt.a_meio = None;
            let buffers = plano.take().map(Plano::into_search).unwrap_or_default();
            (Some((r, buffers)), gasto)
        }
        None => {
            if let Some(a) = rt.a_meio.as_mut() {
                a.trabalho = p.trabalho();
            }
            rt.owed = rt.owed.max(1);
            (None, gasto)
        }
    }
}

#[cfg(test)]
#[path = "agent_tests.rs"]
mod tests;
