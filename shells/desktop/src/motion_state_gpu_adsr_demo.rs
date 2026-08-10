//! A cena do **COMPASSO** (`PH2D_GPU_COOK_DEMO=25`) — o `carry` do `pulse.counter` e o
//! `pulse.adsr`, os dois últimos itens da folha 12 do doc 89, montados como um documento
//! pronto para smoke.
//!
//! **A cadeia** é um Y a partir do metrônomo — os DOIS relógios saem do mesmo `pulse.beat`:
//!
//! - o **PULO** (o relógio cru): `pulse.beat(0,4 s) → pulse.adsr(curto) → motion.drive(Y)`
//! - o **CRESCIMENTO** (o dividido): `… → pulse.counter(4, Wrap) →[carry]→ pulse.adsr(longo)
//!   → motion.drive(Size)`
//!
//! Ela existe porque **as duas features só se veem juntas**. O `carry` é a segunda saída de
//! um nó — a porta que faz contadores COMPOREM — e sozinho ele é invisível: um pulso a cada
//! quatro continua sendo um pulso de um tique. O envelope é o que o torna **visível**, e por
//! sua vez ele precisa de um gatilho esparso para que a subida-e-queda caiba entre dois
//! disparos. Uma cena por feature mostraria duas piscadas de 1/60 s.
//!
//! ⚠️ **O ramo do PULO é o que torna a cena FALSIFICÁVEL, e a primeira versão não o tinha.**
//! Ela desenhava só o relógio dividido, e um divisor 4:1 que esconde o relógio de entrada é
//! **indistinguível de um metrônomo lento sem contador nenhum**: a única cadência na tela
//! vira "a batida", por definição, e o olho não tem contra o que contar. Reportado no smoke
//! de 2026-08-10 (*"parece que incha a cada batida e não a cada 4"*) — a leitura estava
//! certa sobre o que estava desenhado. **Um divisor se mostra desenhando os DOIS relógios.**
//!
//! ⚠️ **O que o olho tem de separar são DOIS fatos, e a cena falha se só um aparecer:**
//! *(a)* a grade **PULA a cada batida** e só **CRESCE a cada quatro** — é o divisor, e com
//! o `count_max` em 1 todo pulo viria com um crescimento (medido); *(b)* o
//! crescimento **sobe, segura e desce** em vez de piscar — é o envelope, e sem ele o `carry`
//! daria um quadro só. Os dois eixos são ortogonais de propósito (posição × tamanho): num
//! canal só eles se somariam e nenhum dos dois seria contável.
//!
//! ⚠️ **Os `pre` self-loops são escritos à MÃO.** O editor os plumba ao SOLTAR um nó; um
//! documento montado por `add_node` não os ganha, e **quatro** nós desta cena dependem disso
//! (`pulse.beat`, `pulse.counter` e os DOIS `pulse.adsr`) — sem eles a memória de borda não
//! existe e a cena fica parada, sem erro nenhum.
//!
//! ⚠️ **A cadeia é CPU-only por natureza**, como a `=23`: os `pulse.*` não têm kernel porque
//! um pulso é um evento por LINHA com memória de borda no `pre`, não um mapa por texel. O
//! censo de cobertura diz isso em vez de contar como omissão.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::NodeId;

/// Lado da grade — `SIDE²` pontos, a densidade das cenas de pulso vizinhas.
pub const SIDE: f32 = 24.0;
/// Espaçamento entre pontos, em unidades de mundo.
const GAP: f32 = 0.42;
/// O tamanho de repouso de um ponto.
const DOT: f32 = 0.16;
/// Quanto o envelope acrescenta ao Size no pico. Grande de propósito: o que a cena
/// mede é a FORMA da subida, e um inchaço tímido não deixa ver se ele rampa ou pisca.
pub const SWELL: f32 = 0.9;

/// O período do metrônomo. Rápido para a razão 4:1 ser contável a olho.
pub const BEAT: f32 = 0.4;
/// De quantas em quantas batidas o `carry` dispara — o divisor de relógio.
pub const DIVIDE_BY: f32 = 4.0;

/// O envelope do CRESCIMENTO, em segundos. A soma (0,05 + 0,25 + 0,35 + 0,35 = **1,0 s**)
/// cabe confortavelmente no compasso de `BEAT × DIVIDE_BY` = 1,6 s, então cada crescimento
/// **termina antes** do seguinte — se não coubesse, o `retrigger` cortaria a queda e a
/// cena mediria outra coisa.
const ATTACK: f32 = 0.05;
const DECAY: f32 = 0.25;
const SUSTAIN: f32 = 0.6;
const HOLD: f32 = 0.35;
const RELEASE: f32 = 0.35;

/// O envelope do PULO — um pico que sobe e cai em **0,13 s**, um terço da batida, para que
/// quatro pulos sejam quatro eventos SEPARADOS e não um tremor contínuo. `sustain = 0` é o
/// que o torna um pico: depois do decay não sobra nada para segurar.
const HOP_ATTACK: f32 = 0.03;
const HOP_DECAY: f32 = 0.10;
/// Quanto o pulo levanta a grade, em unidades de mundo. **Menor que `GAP`** de propósito:
/// uma fileira que sobe mais que o espaçamento invade a de cima e o pulo deixa de se ler
/// como pulo.
pub const HOP: f32 = 0.30;

/// Monta o documento e devolve os sinks.
pub fn build_gpu_adsr_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;

    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", SIDE);
    g.set_param(grid, "cols", SIDE);
    g.set_param(grid, "gap_x", GAP);
    g.set_param(grid, "gap_y", GAP);
    let scale = g.add_node("motion.scale");
    g.set_param(scale, "amount", DOT);

    let beat = g.add_node("pulse.beat");
    g.set_param(beat, "period", BEAT);

    // O DIVISOR: conta as batidas e só o `carry` (a saída 1) sai a cada volta.
    let count = g.add_node("pulse.counter");
    g.set_param(count, "count_max", DIVIDE_BY);
    g.set_param(count, "mode", 0.0); // Wrap — a única que completa voltas.

    let env = g.add_node("pulse.adsr");
    g.set_param(env, "attack", ATTACK);
    g.set_param(env, "decay", DECAY);
    g.set_param(env, "sustain", SUSTAIN);
    g.set_param(env, "hold", HOLD);
    g.set_param(env, "release", RELEASE);

    // O ramo do PULO: o MESMO metrônomo, sem passar pelo contador. É ele que dá ao olho o
    // relógio de entrada contra o qual contar o dividido.
    let hop = g.add_node("pulse.adsr");
    g.set_param(hop, "attack", HOP_ATTACK);
    g.set_param(hop, "decay", HOP_DECAY);
    g.set_param(hop, "sustain", 0.0);
    g.set_param(hop, "hold", 0.0);
    g.set_param(hop, "release", 0.0);

    let drive_y = g.add_node("motion.drive");
    g.set_param(drive_y, "channel", 1.0); // Y
    g.set_param(drive_y, "mode", 0.0); // Add
    g.set_param(drive_y, "scale", HOP);

    let drive = g.add_node("motion.drive");
    g.set_param(drive, "channel", 3.0); // Size
    g.set_param(drive, "mode", 0.0); // Add
    g.set_param(drive, "scale", SWELL);
    let out = g.add_node("motion.output");

    for (i, n) in [grid, scale, drive_y, drive, out].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: 80.0 + i as f32 * 190.0,
                y: 260.0,
            },
        );
    }
    for (i, n) in [beat, count, env].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: 270.0 + i as f32 * 190.0,
                y: 40.0,
            },
        );
    }
    g.set_pos(hop, Pos { x: 460.0, y: 150.0 });

    for (a, ap, b, bp) in [
        (grid, 0u16, scale, 0u16),
        // O metrônomo lê o mesmo stream, só pela contagem de linhas.
        (scale, 0, beat, 0),
        (beat, 0, count, 0),
        // O ramo CRU: o pulo sai do metrônomo, antes do divisor.
        (beat, 0, hop, 0),
        // ⚠️ **A porta de saída 1**: o `carry`, não a contagem — e ligar a `0` é
        // **INEXPRIMÍVEL**, medido: a contagem é um VALOR (relógio `Frame`) e este porto
        // pede um PULSO (`Event`), então o tipo recusa e o documento nem se monta. O erro
        // que este fio PODE cometer é outro — um `count_max` errado —, e é esse que o gate
        // da razão pega.
        (count, 1, env, 0),
        (scale, 0, drive_y, 0),
        (hop, 0, drive_y, 1),
        (drive_y, 0, drive, 0),
        (env, 0, drive, 1),
        (drive, 0, out, 0),
    ] {
        g.connect(Edge {
            from: (a, ap),
            to: (b, bp),
            delayed: false,
        })
        .ok()?;
    }
    // Os quatro `pre` self-loops (memória de borda / idade dos envelopes).
    for (n, port) in [(beat, 1u16), (count, 1), (env, 1), (hop, 1)] {
        g.connect(Edge {
            from: (n, 0),
            to: (n, port),
            delayed: true,
        })
        .ok()?;
    }
    g.validate(reg).ok()?;
    Some(vec![out])
}

#[cfg(test)]
#[path = "motion_state_gpu_adsr_demo_tests.rs"]
mod tests;
