//! **AS CINCO FONTES** (`PH2D_GPU_COOK_DEMO=24`) — a P0 da folha 12 do doc 89 montada como
//! documento pronto para smoke: *nada era DISPARADO por um pulso*.
//!
//! A cena `=23` mostra um campo decidindo **quem escuta** um evento; esta mostra um evento
//! decidindo **o que passa a existir**. É o `Event Handler` do Niagara — *quando isto acontece,
//! faça aquilo* — com uma porta e um número.
//!
//! **A cadeia:**
//! `motion.grid(1×5) → sim.spawn(pulse) → motion.combine → force.wind → sim.step →
//! sim.lifetime → motion.cull → sim.zone`.
//!
//! ⚠️ **`rate = 0`, e é isso que torna a cena um TESTE em vez de uma ilustração.** O nascimento
//! por taxa e o nascimento por pulso são ADITIVOS (a porta desconectada é o mundo de antes, byte
//! a byte), então zerar a taxa deixa **o pulso como único autor** de tudo o que aparece: se a
//! porta não estivesse ligada a tela ficaria VAZIA para sempre, e não meio cheia.
//!
//! ⚠️ **As cinco bocas são o oráculo de OLHO da lei "nasce na linha que FIROU".** Se o mapa
//! linha→nascimento estivesse errado (todo recém-nascido colhendo a linha 0, o modo de falha
//! natural), as cinco fontes colapsariam numa só — e isso se vê sem sonda nenhuma.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::NodeId;

/// Quantas bocas. Cinco porque três já leem como "alguns" e sete não acrescentam informação —
/// o que a contagem tem de comprar é a distinção entre *cada boca* e *uma boca só*.
const NOZZLES: f32 = 5.0;
/// O intervalo entre as batidas. Meio segundo é o compasso que se conta olhando (o mesmo da
/// `=23`, e pela mesma razão).
const PERIOD: f32 = 0.5;
/// Quantos elementos uma boca dá à luz por batida. **MEDIDO, não escolhido:** a `sim.spawn`
/// tem teto de [`MAX_PER_TICK`](ph2d_node_sim_spawn) por tique somando TODAS as linhas, então
/// cinco bocas × este número tem de caber nele — e cabe com folga de duas ordens.
const BURST: f32 = 8.0;

/// **AS CINCO FONTES** (`PH2D_GPU_COOK_DEMO=24`): um metrônomo, cinco bocas, e uma população
/// que só existe porque um evento a fez existir.
///
/// O que o artista vê: nada, e a cada meio segundo **cinco baforadas simultâneas**, uma em cada
/// boca, caindo e desaparecendo de velhice. Entre as batidas **nada nasce** — e isso não é uma
/// pausa cosmética: é a taxa em zero, com o pulso como único autor.
pub(super) fn build_gpu_spawn_pulse_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;

    // As BOCAS: uma fileira, bem espaçada, para uma baforada não se confundir com a vizinha.
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", NOZZLES);
    g.set_param(grid, "gap_x", 2.2);
    let scale = g.add_node("motion.scale");
    g.set_param(scale, "amount", 0.09);

    // O metrônomo — conta as MESMAS linhas do template, então o pulso dele chega alinhado boca
    // a boca (a porta `pulse` lê a linha `i` para a linha `i`).
    let beat = g.add_node("pulse.beat");
    g.set_param(beat, "period", PERIOD);

    // A ZONA e o seu interior.
    let zone = g.add_node("sim.zone");
    let spawn = g.add_node("sim.spawn");
    g.set_param(spawn, "rate", 0.0); // SÓ o pulso dá à luz
    g.set_param(spawn, "burst", BURST);
    let combine = g.add_node("motion.combine");
    let wind = g.add_node("force.wind");
    g.set_param(wind, "angle", -90.0); // para baixo: a gravidade da cena
    g.set_param(wind, "strength", 3.0);
    g.set_param(wind, "gust", 0.0); // uma queda limpa; a rajada é outra demonstração
    let step = g.add_node("sim.step");
    let life = g.add_node("sim.lifetime");
    g.set_param(life, "life", 1.6);
    g.set_param(life, "variance", 0.25);
    let cull = g.add_node("motion.cull");
    g.set_param(cull, "mode", 1.0); // Falloff: o `sim.lifetime` já marcou quem morreu
    let out = g.add_node("motion.output");

    for (i, n) in [grid, scale, zone, out].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: 80.0 + i as f32 * 190.0,
                y: 200.0,
            },
        );
    }
    // O metrônomo acima da fileira: ele é a outra entrada do nascimento.
    g.set_pos(beat, Pos { x: 270.0, y: 40.0 });
    // O INTERIOR lê da direita para a esquerda, a forma que um laço de estado tem numa tela
    // que se lê da esquerda para a direita (a convenção da cena da chuva).
    for (i, n) in [spawn, combine, wind, step, life, cull]
        .into_iter()
        .enumerate()
    {
        g.set_pos(
            n,
            Pos {
                x: 80.0 + i as f32 * 170.0,
                y: 360.0,
            },
        );
    }

    for (a, ap, b, bp) in [
        (grid, 0u16, scale, 0u16),
        // O template das bocas vai para o nascimento…
        (scale, 0, spawn, 0),
        // …e o metrônomo, com as mesmas linhas, para a porta `pulse`.
        (scale, 0, beat, 0),
        (beat, 0, spawn, 1),
        // O laço: o estado do tique anterior + os recém-nascidos deste.
        (zone, 0, combine, 0),
        (spawn, 0, combine, 1),
        (combine, 0, wind, 0),
        (wind, 0, step, 0),
        (step, 0, life, 0),
        (life, 0, cull, 0),
        (cull, 0, zone, 1),
        (zone, 0, out, 0),
    ] {
        let delayed = (a, b) == (zone, combine);
        g.connect(Edge {
            from: (a, ap),
            to: (b, bp),
            delayed,
        })
        .ok()?;
    }
    // O `pre` self-loop da memória de borda do metrônomo. O editor o plumba ao SOLTAR um nó;
    // um documento montado por `add_node` o escreve à mão.
    g.connect(Edge {
        from: (beat, 0),
        to: (beat, 1),
        delayed: true,
    })
    .ok()?;
    g.validate(reg).ok()?;
    Some(vec![out])
}

#[cfg(test)]
#[path = "motion_state_gpu_spawn_pulse_demo_tests.rs"]
mod tests;
