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

/// Quantas peças por lado — ele **SEGUE O TECTO DO DONO**, e é por isso que os dois números
/// abaixo têm uma cerca de compilação.
///
/// ⛔⛔ **Ele já foi `224`, e desceu com o tecto** (2026-09-21): a grelha clampa cada LADO em
/// [`LADO_MAX_DE_GRELHA`](ph2d_nodegraph::node::LADO_MAX_DE_GRELHA), logo um `224` escrito
/// entregaria o tecto e a cena anunciaria um campo que ela não produz.
#[expect(clippy::cast_precision_loss, reason = "um lado de grelha")]
pub(super) const SIDE: f32 = ph2d_nodegraph::node::LADO_MAX_DE_GRELHA as f32;

/// **A POPULAÇÃO EM QUE OS DOIS NÚMEROS ABAIXO FORAM MEDIDOS** — e a cerca que os impede de
/// envelhecer em silêncio.
///
/// ⛔⛔⛔ **ELA NASCEU DE UM DEFEITO REAL, E ELE ACONTECEU DUAS VEZES EM DOIS DIAS.** O [`SIDE`]
/// segue o tecto de instâncias do dono; os `COOK_*_MS` são medições. Quando o tecto se mexe, o
/// primeiro muda sozinho e os segundos **não** — e o anúncio, que os CITA, passa a dizer ao dono
/// dois números medidos numa cena que já não existe:
///
/// | dia | tecto | `SIDE` | peças | o que o anúncio dizia | o que a sonda mede |
/// |---|---|---|---|---|---|
/// | até 20/09 | — | `224` | `50 176` | `4,12` / `13,81` | `4,12` / `13,81` ✅ |
/// | 21/09 | `16 384` | `128` | `16 384` | `4,12` / `13,81` | **por medir** ⛔ |
/// | 22/09 | `32 768` | `181` | `32 761` | `4,12` / `13,81` | **`2,44` / `8,74`** ⛔ |
///
/// ⚠️⚠️ **O gate que existia não podia ver isto:** o
/// `the_announcement_cites_the_numbers_the_scene_uses` afirma que o anúncio **CITA as consts** em
/// vez de repetir literais — ele é sobre a PROVENIÊNCIA do número, nunca sobre a VERDADE dele.
/// *Uma const citada continua a ser um literal quando o mundo que a produziu mudou.*
///
/// ⇒ a cerca é de COMPILAÇÃO e o sujeito dela é a POPULAÇÃO: mover o tecto **parte a build**
/// aqui, e quem o mover é obrigado a correr a sonda em vez de se lembrar de o fazer. É a mesma
/// forma do `181.0` escrito à mão no WGSL do `motion.grid`, e pela mesma razão — *uma constante
/// que atravessa uma fronteira (ali de LINGUAGEM, aqui de MEDIÇÃO) não pode depender de alguém
/// se lembrar.*
const MEDIDO_EM_PECAS: usize =
    ph2d_nodegraph::node::LADO_MAX_DE_GRELHA * ph2d_nodegraph::node::LADO_MAX_DE_GRELHA;
const _: () = assert!(
    MEDIDO_EM_PECAS == 32_761,
    "o tecto de instancias mudou, logo a cena =107 mudou de populacao e os COOK_*_MS descrevem \
     outra cena: corra `cargo test -p ph2d-app-motion --release --lib measure_lazy_switch_cost \
     -- --ignored --nocapture` com a maquina calma e traga os dois numeros para ca"
);

/// O custo do COZIMENTO nos dois modos, em ms — o que a sonda `measure_lazy_switch_cost`
/// imprimiu (release, `load 3,48` de 32 núcleos, mediana de 7 com aquecimento fora), a
/// [`MEDIDO_EM_PECAS`] peças:
///
/// ```text
///   modo LIGADO      2,44 ms/cook    15% de um quadro de 16,7
///   modo DESLIGADO   8,74 ms/cook    52% de um quadro, ANTES de desenhar as 32 761 pecas
/// ```
///
/// É esta a lição da cena: **o cozimento sozinho decide o quadro**. Ligado sobra folga para o
/// resto; desligado ele come metade do orçamento antes de uma peça ser desenhada, e a razão
/// entre os dois é `3,6×`.
///
/// ⚠️ **A tabela anterior tinha TRÊS linhas e nenhum instrumento**, e a auditoria de 2026-08-27
/// mostrou dois defeitos nela: as duas colunas carregavam o custo fixo do 2.º sink — que era
/// então o campo inteiro em repouso, e que aquela jornada reduziu a uma peça — e a coluna
/// OFF era **super-linear sem recurso nomeado** (`224 → 256` dava `1,31×` em peças e `4,36×` em
/// milissegundos), o que é a assinatura de leitura sob carga. ⇒ *ficam as duas linhas que a
/// sonda produz, e quem quiser uma terceira roda a sonda.*
///
/// ⛔ **O que está aqui é o COZIMENTO, não o quadro.** O quadro soma o desenho das peças, que
/// esta sonda não mede — e afirmar um número de quadro sem o medir foi exactamente o que a
/// tabela velha fez.
///
/// ⚠️ **Eles são `const` para que o anúncio os CITE em vez de os repetir.** A 1.ª versão deste
/// demo passava `on = 9.59, off = 33.63` como literais inline no `motion_state_demo_announce.rs`
/// — sozinha entre as seis cenas anunciadas, e por isso a única fora do gate
/// `the_announcement_cites_the_numbers_the_scene_uses`. ⛔ Mas citar não é ser verdade: ver
/// [`MEDIDO_EM_PECAS`], que é a metade que faltava.
pub(super) const COOK_ON_MS: f32 = 2.44;
/// Ver [`COOK_ON_MS`].
pub(super) const COOK_OFF_MS: f32 = 8.74;
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

    // ⭐ **UMA SAÍDA SÓ, e a cena PEDE a CPU** (doc 119 W3 — ver [`PEDE_A_CPU`]). Até ao ciclo 11 ela
    // tinha uma segunda saída minúscula, que era a cerca do multi-sink a pô-la na CPU; levantada a
    // cerca, a cena iria à placa e o botão ficava inerte. A segunda saída também já custara uma
    // auditoria (2026-08-27: desenhada por cima, escondia `83 %` da onda).
    Some(vec![out])
}

/// ⭐ **Porque é que esta cena corre na CPU** — a frase que a leitura da rota imprime.
///
/// ⚠️ **MEDIDO, e é o número que decide o desenho:** neste grafo, a rota da placa faz o quadro em
/// **3,75 ms** com os quatro ramos, contra **13,10 ms** da CPU com a preguiça ligada. ⇒ *forçar a
/// CPU quando o ARTISTA liga o modo tornaria o botão uma armadilha*, e por isso a recusa não vive
/// no nó: o modo vale onde a CPU já é o caminho, e esta cena — que existe para o ensinar — pede-a.
pub(super) const PEDE_A_CPU: &str =
    "CPU: a cena =107 ensina um modo do cozimento da CPU (Skip Unused Inputs) -- ela pede-a";

#[cfg(test)]
#[path = "motion_state_lazy_switch_demo_tests.rs"]
mod tests;
