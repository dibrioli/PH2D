//! **A PREGUIÇA DO ROTEADOR** (`PH2D_GPU_COOK_DEMO=107`) — a última célula P2 da conferência
//! (doc 89, folha 15): o *"only the input that is passed through the node is computed"* que o
//! Blender documenta duas vezes.
//!
//! ⚠️ **Esta cena não se julga pela IMAGEM, julga-se pelo MOVIMENTO.** A saída é a mesma com o
//! modo ligado e desligado — é essa a promessa —, e o que muda é o custo: quatro ramos de ruído
//! fractal de oito oitavas sobre as [`SIDE`]² peças, dos quais o roteador usa **um**.
//!
//! ⚠️ **Os números saem da sonda `measure_lazy_switch_cost`, não de prosa** — este cabeçalho já
//! disse `4096 peças` e `~10,8 / ~2,8 ms` enquanto o código dizia `50 176` e a tabela do [`SIDE`]
//! dizia outra coisa ainda: **três respostas para a mesma pergunta no mesmo ficheiro**, achadas
//! pela auditoria de 2026-08-27. Rode a sonda antes de citar qualquer um deles.
//!
//! ⚠️ **O `select` fica DESLIGADO de propósito.** Uma porta sem aresta lê o campo vazio, que é
//! `0` em todo índice — uniforme por construção, que é a primeira das três condições da
//! preguiça. Ligar-lhe um `value.instance_field` faria dele um campo POR ELEMENTO, e aí cada
//! elemento escolhe o seu ramo, nenhum é dispensável e o modo recua para o caminho de sempre.
//! *A sonda que precificou esta feature usava exactamente esse select, e por isso media um ganho
//! que o mecanismo nunca poderia entregar naquele grafo.*

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// Quantas peças por lado — **MEDIDO, e o recurso é tempo de CPU no COZIMENTO**.
///
/// Sonda `measure_lazy_switch_cost` (release, `load 2,74` de 32 núcleos, mediana de 7 com
/// aquecimento fora), `SIDE = 224` ⇒ **50 176** peças:
///
/// ```text
///   modo LIGADO      4,12 ms/cook    25% de um quadro de 16,7
///   modo DESLIGADO  13,81 ms/cook    83% de um quadro, ANTES de desenhar as 50 176 pecas
/// ```
///
/// `224` é onde o cozimento sozinho decide o quadro: ligado sobra folga para o resto do quadro,
/// desligado ele já come 83% do orçamento antes de uma peça ser desenhada.
///
/// ⚠️ **A tabela anterior tinha TRÊS linhas e nenhum instrumento**, e a auditoria de 2026-08-27
/// mostrou dois defeitos nela: as duas colunas carregavam o custo fixo do 2.º sink — que era
/// então o campo inteiro em repouso, e que esta mesma jornada reduziu a uma peça — e a coluna
/// OFF era **super-linear sem recurso nomeado** (`224 → 256` dava `1,31×` em peças e `4,36×` em
/// milissegundos), o que é a assinatura de leitura sob carga. ⇒ *ficam as duas linhas que a
/// sonda produz, e quem quiser uma terceira roda a sonda.*
///
/// ⛔ **O que está aqui é o COZIMENTO, não o quadro.** O quadro soma o desenho das peças, que
/// esta sonda não mede — e afirmar um número de quadro sem o medir foi exactamente o que a
/// tabela velha fez.
pub(super) const SIDE: f32 = 224.0;
/// O custo do COZIMENTO nos dois modos, em ms — o que a sonda `measure_lazy_switch_cost`
/// imprimiu (release, máquina calma, mediana de 7).
///
/// ⚠️ **Eles são `const` para que o anúncio os CITE em vez de os repetir.** A 1.ª versão deste
/// demo passava `on = 9.59, off = 33.63` como literais inline no `motion_state_demo_announce.rs`
/// — sozinha entre as seis cenas anunciadas, e por isso a única fora do gate
/// `the_announcement_cites_the_numbers_the_scene_uses`. Quando esta jornada mudou a cena, os dois
/// números do anúncio ficaram errados e **nada** podia dizê-lo.
pub(super) const COOK_ON_MS: f32 = 4.12;
/// Ver [`COOK_ON_MS`].
pub(super) const COOK_OFF_MS: f32 = 13.81;
/// Quantas oitavas tornam um ramo CARO.
const OCTAVES: f32 = 8.0;
/// Quantos ramos o roteador tem (o manifesto do nó).
const BRANCHES: usize = 4;

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .ok()
}

pub(super) fn build_lazy_switch_demo_document(
    doc: &mut MotionDoc,
    _reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let grid = g.add_node("motion.grid");
    g.set_pos(grid, Pos { x: 0.0, y: 0.0 });
    g.set_param(grid, "rows", SIDE);
    g.set_param(grid, "cols", SIDE);
    g.set_param(grid, "gap_x", 3.8 / SIDE);
    g.set_param(grid, "gap_y", 3.8 / SIDE);

    let size = g.add_node("motion.scale");
    g.set_pos(size, Pos { x: 150.0, y: 0.0 });
    g.set_param(size, "amount", 0.045);
    wire(g, grid, 0, size, 0)?;

    let sw = g.add_node("value.switch");
    g.set_pos(sw, Pos { x: 620.0, y: 120.0 });
    // ⭐ **Nasce LIGADO nesta cena** — o smoke abre no caminho bom e o artista DESLIGA para
    // sentir o que ele custa. Abrir a arrastar-se pareceria uma cena partida.
    g.set_param(sw, ph2d_node_value_switch::LAZY, 1.0);
    g.set_label(sw, "Switch (Skip Unused Inputs)");

    // Os quatro ramos CAROS — ruído fractal de oito oitavas sobre as 4096 peças, cada um com a
    // sua semente e o seu ritmo.
    //
    // ⚠️ **`value.noise` e não `motion.noise` + `value.attribute`.** A 1.ª versão desta cena
    // usava o par, copiado da sonda — e o `value.attribute` no modo de omissão procura uma
    // coluna ESCALAR chamada `P`, que não existe (o `P` é `Vec2`). Os quatro ramos emitiam
    // VAZIO, o roteador não conduzia nada e o campo ficava parado: *a cena montava, cozinhava
    // 4096 peças, media o custo certo — e não se mexia.* ⚠️ A sonda tem o mesmo par e continua
    // válida, porque o que ela mede é o CUSTO do cozimento e o nó de ruído é cozido na mesma;
    // o que ela nunca precisou foi que o valor chegasse ao fim.
    //
    // ⚠️ **E o `value.noise` é `Effect::Temporal`** — o que faz desta cena a prova, em produto,
    // de que a cerca do estado nomeia o mecanismo certo: ler o relógio não impede o salto, a
    // realimentação é que impede.
    for k in 0..BRANCHES {
        #[expect(clippy::cast_precision_loss, reason = "0..4")]
        let row = 260.0 + k as f32 * 110.0;
        let ns = g.add_node("value.noise");
        g.set_pos(ns, Pos { x: 380.0, y: row });
        g.set_param(ns, "octaves", OCTAVES);
        g.set_param(ns, "amplitude", 0.55);
        g.set_param(ns, "speed", 0.6);
        #[expect(clippy::cast_precision_loss, reason = "0..4")]
        g.set_param(ns, "frequency", 0.5 + k as f32 * 0.35);
        #[expect(clippy::cast_precision_loss, reason = "0..4")]
        g.set_param(ns, "seed", k as f32 + 1.0);
        wire(g, size, 0, ns, 0)?;
        #[expect(clippy::cast_possible_truncation, reason = "0..4")]
        wire(g, ns, 0, sw, k as u16 + 1)?;
    }

    // A saída do roteador CONDUZ o deslocamento vertical das peças — sem um consumidor
    // visível, um smoke de custo não teria o que se julgar a olho.
    let drive = g.add_node("motion.drive");
    g.set_pos(drive, Pos { x: 800.0, y: 0.0 });
    g.set_param(drive, "channel", 1.0);
    g.set_param(drive, "scale", 1.0);
    wire(g, size, 0, drive, 0)?;
    wire(g, sw, 0, drive, 1)?;

    let out = g.add_node("motion.output");
    g.set_pos(out, Pos { x: 980.0, y: 0.0 });
    wire(g, drive, 0, out, 0)?;

    // ⚠️ **UMA SEGUNDA SAÍDA, e ela é a razão de a cena existir como cena.** O cozimento é
    // **GPU-residente por omissão**, e no device o grafo inteiro vira UM dispatch — não há
    // ramo para saltar. Este modo é uma propriedade do cozimento de **CPU**, e um documento vai
    // para a CPU quando o plano de GPU não o cobre: vector vivo, escopos de tempo, nós de
    // CPU-only… ou **mais de um sink** (`motion_bridge_gpu`: `motion.sinks.len() != 1`).
    //
    // ⚠️ **MEDIDO, e é o número que decide o desenho:** neste mesmo grafo com um sink só, a rota
    // de GPU faz o quadro em **3,75 ms** com os quatro ramos, contra **13,10 ms** da CPU com a
    // preguiça ligada. ⇒ *forçar a CPU quando o artista liga o modo tornaria o botão uma
    // armadilha*, e é por isso que a recusa NÃO existe: o modo vale onde a CPU já é o caminho.
    // Uma segunda saída é a forma mais honesta de pôr esta cena lá — é autoria legítima, não um
    // truque, e o texto do smoke di-lo.
    // ⚠️⚠️ **E ela lê uma grelha PRÓPRIA de UMA peça — não o campo.** A 1.ª versão ligava o `peek`
    // ao mesmo `size`, e o pump **ACUMULA** todos os sinks (`lower_to_instances_onto` sobre um
    // `Vec` só, `ph2d-eval-motion`): a 2.ª saída lowerava outras `SIDE²` instâncias na posição de
    // REPOUSO e, por ser a última, desenhava-as **por cima**. Medido pela auditoria de 2026-08-27:
    // a banda parada cobria `3,78` dos `4,59` da onda ⇒ **só 17% da altura ondulava**, e o campo é
    // opaco (cada peça cobre `2,65×` o passo). *A cena escondia exactamente aquilo que pedia ao
    // artista para julgar* — e o defeito nasceu da cura de um smoke anterior, o que o torna a
    // segunda vez que este demo foi entregue sem se olhar para ele a correr.
    //
    // ⚠️ **Uma peça basta, e um sink SEM aresta não serve:** o que a rota de GPU conta é
    // `sinks.len()`, não o que eles cozem — mas o `diagnose` do sweep das 107 cenas acusa
    // `MissingInput` numa saída solta, e com razão. ⇒ a âncora é autoria a sério, minúscula.
    let anchor = g.add_node("motion.grid");
    g.set_pos(anchor, Pos { x: 620.0, y: 700.0 });
    g.set_param(anchor, "rows", 1.0);
    g.set_param(anchor, "cols", 1.0);
    let anchor_size = g.add_node("motion.scale");
    g.set_pos(anchor_size, Pos { x: 800.0, y: 700.0 });
    g.set_param(anchor_size, "amount", 0.002);
    wire(g, anchor, 0, anchor_size, 0)?;

    let peek = g.add_node("motion.output");
    g.set_pos(peek, Pos { x: 980.0, y: 700.0 });
    g.set_label(peek, "(segunda saida: poe a cena no cozimento de CPU)");
    wire(g, anchor_size, 0, peek, 0)?;
    Some(vec![out, peek])
}

#[cfg(test)]
#[path = "motion_state_lazy_switch_demo_tests.rs"]
mod tests;
