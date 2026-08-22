//! **A ESQUIVA A OBSTÁCULOS** — as três colunas que a nomeiam e a aceleração que
//! elas produzem.
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 700 para `crates/`), e o corte
//! é por RESPONSABILIDADE: as três regras de Reynolds (coesão, alinhamento, separação)
//! ficam no `lib.rs`; a quarta — a que lê um SEGUNDO stream como geometria — mora aqui,
//! porque é a única que olha para fora do bando.

/// Resolved simulation weights (arithmetic-ready).
/// **O PESO DO DESVIO** (doc 89 folha 03 — o *Obstacle Avoidance* do Reynolds, o POP
/// Steer Avoid Obstacle do Houdini).
///
/// `0` desliga — e desliga os três params da família, porque um raio e uma antecipação
/// sem peso não fazem nada. Todo bando autorado antes disto lê zero e voa como voava.
///
/// ⚠️ **A célula dizia *"a ausência de qualquer geometria de obstáculo alcançável de
/// dentro"*, e isso deixou de ser verdade nesta janela:** a porta-template — um segundo
/// stream lido como geometria — é o padrão que o `field.shape` estreou e o
/// `force.attractor` repetiu. O obstáculo é a lista de pontos da porta `obstacle`.
pub(super) const AVOID: &str = "avoid";
/// A que distância um obstáculo começa a incomodar.
pub(super) const AVOID_RADIUS: &str = "avoid_radius";

/// **A ANTECIPAÇÃO, em segundos** — o *lookahead* do Reynolds.
///
/// O agente não pergunta *"há pedra onde estou?"* e sim *"há pedra onde vou estar?"*: a
/// sonda é `p + v · lookahead`. `0` sonda a própria posição, que é o comportamento de um
/// campo de repulsão comum — útil, e é o default porque é o mais previsível.
///
/// ⚠️ **É por isto que o desvio precisa da VELOCIDADE, e é o que o separa de uma
/// simples repulsão:** um agente rápido tem de virar mais cedo, e o `lookahead` faz o
/// raio efectivo crescer com a velocidade **dele**, sem um segundo knob.
pub(super) const LOOKAHEAD: &str = "lookahead";

/// A ACELERAÇÃO DE DESVIO de um agente — para longe do obstáculo mais próximo da sonda.
///
/// `None` quando não há obstáculo dentro do raio (ou não há obstáculos de todo): quem
/// chama não soma nada, nem sequer um zero.
///
/// ⚠️ **A força cresce para dentro, linearmente** (`1 − d/raio`), e não com o inverso do
/// quadrado: um inverso do quadrado é infinito no contacto, e um agente que atravessa o
/// centro de uma pedra receberia um impulso arbitrário. A rampa linear é a mesma forma
/// que a `separation` deste nó já usa — uma lei, dois sítios.
///
/// ⚠️ **Empate de distância desempata pelo ÍNDICE MAIS BAIXO** (a ordem total dos
/// irmãos), e o obstáculo EXACTAMENTE sob a sonda não produz direcção nenhuma — `d = 0`
/// não tem para onde empurrar, então a resposta é zero em vez de um `NaN`.
pub(super) fn avoid_accel(
    pos: [f32; 2],
    vel: [f32; 2],
    obstacles: &[[f32; 2]],
    radius: f32,
    lookahead: f32,
) -> Option<[f32; 2]> {
    if obstacles.is_empty() || radius <= 0.0 {
        return None;
    }
    let probe = [pos[0] + vel[0] * lookahead, pos[1] + vel[1] * lookahead];
    let mut best = [0.0f32; 2];
    let mut best_d2 = f32::INFINITY;
    for q in obstacles {
        let (dx, dy) = (probe[0] - q[0], probe[1] - q[1]);
        let d2 = dx * dx + dy * dy;
        if d2 < best_d2 {
            best_d2 = d2;
            best = [dx, dy];
        }
    }
    let d = best_d2.sqrt();
    if d >= radius || d <= 0.0 {
        return None;
    }
    let k = 1.0 - d / radius;
    Some([best[0] / d * k, best[1] / d * k])
}
