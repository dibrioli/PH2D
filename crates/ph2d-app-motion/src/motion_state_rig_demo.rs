//! ⭐⭐⭐ **COISAS QUE SE SEGURAM** (`PH2D_GPU_COOK_DEMO=120`) — a cena do **ciclo 9**
//! ([doc 114](../../docs/Motion%20Nodes/114_ciclo_9_rig_e_corpos_moles.md) §10).
//!
//! ## O que ela ensina, e por que são TRÊS fileiras
//!
//! O ciclo 8 respondeu *de onde vem a peça*. Este responde à pergunta seguinte: **o que faz um
//! monte de peças ficar junto**. O grupo parte-se em duas categorias na paleta (§1 do doc), e a
//! cena é essa partição mais a lição que cada wave deixou:
//!
//! ```text
//!   CIMA    O QUE SE SEGURA SOZINHO   Verlet Rope (a corda)   |  Wave (o campo)
//!   MEIO    QUEM SEGURA               FK: os pais decidem     |  IK: a mão vai ao ALVO
//!   BAIXO   A PELE                    Skin: os ossos todos    |  ...cada osso com o SEU envelope
//! ```
//!
//! ⭐ **A fileira de CIMA é a categoria «Source»** — cinco nós que EMITEM uma coisa que se
//! segura — e a de BAIXO/MEIO é a categoria «Transform», cinco nós que AGEM sobre ela. *A
//! partição não é arrumação: é a premissa do tutorial.*
//!
//! ⚠️⚠️ **A fileira do MEIO é o par que separa as duas cinemáticas, e as duas ESTÃO CERTAS:** à
//! esquerda cada junta recebe o próprio ângulo e a cadeia resolve-se dos pais para os filhos (é
//! isso que o cartão `FK` faz); à direita o artista diz **onde a mão tem de estar** e o solver
//! acha os ângulos. ⛔ Não é «uma melhor que a outra» — é *quem escreve o quê*.
//!
//! ⭐⭐ **A fileira de BAIXO é a wave W1′ deste ciclo** (§6 do doc): a pele da direita passa por
//! uma caneta que escreve a coluna `bone_weight`, e com ela **cada osso puxa o seu quinhão**. O
//! da esquerda é o CONTROLO — a mesma pele com os ossos todos a puxar por igual —, e sem ele
//! *«mexeu»* não separa o envelope de um grafo diferente.
//!
//! ⚠️ **A caneta do envelope são TRÊS cartões e isso é a lição, não um preço:** um CAMPO
//! (`field.index_range`) decide *quanto*, o `value.attribute` lê esse número da coluna `falloff` e
//! o `motion.drive(Custom…)` escreve-o na coluna que se quiser. ⭐ **É o mesmo trio do cartão
//! `Strength` do pano do IK, e é de propósito:** dois panos, uma lição — *um campo decide o
//! quanto; o par atributo+caneta leva-o ao sítio*. A W1 deste ciclo foi **refutada por medição**
//! exactamente aqui (§3.1 do doc): o escritor genérico de coluna já existia, e construir um
//! segundo seria reconstruir o que a composição já exprime.
//!
//! ⛔ **QUATRO dos dez nós do grupo não estão aqui, e cada ausência tem motivo medido:**
//! - `motion.soft_body` e `motion.boids` — os dois **produzem** como a corda e o campo, e os dois
//!   já têm cena própria (o corpo mole na `=87`; o bando na conferência). *Uma cena de ciclo
//!   mostra a LEI; o catálogo tem as cenas da conferência.*
//! - `rig.fabrik` e `rig.rubber_hose` — são **a mesma lei do `rig.ik_2bone`** com outra
//!   contagem de juntas e outro carácter. Pô-los lado a lado ensinaria *«há três nomes»*, que é
//!   o oposto de *«a mão vai ao alvo»*. ⭐ E a força deles é a MESMA coluna `falloff` que a §7-W2
//!   fechou, logo o passo do tutorial que a exercita vale para os três.
//!
//! ⚠️⚠️ **A cena inteira coze na CPU, e é PROPRIEDADE medida, não defeito:** nove dos dez nós do
//! grupo não têm kernel (§2 do doc, com dois instrumentos independentes a concordarem), e o
//! décimo — o `motion.boids` — não está aqui. Com seis panos pequenos não se vê; quem mede o
//! preço é a §7, e é por isso que a medição do grupo **não sai desta cena**.

use crate::motion_demo_legend::Caption;
use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, NodeId, Pos};

/// A que distância do centro cada metade vive — o mesmo `2,9` da `=118` e da `=119`.
const COL_X: f32 = 2.9;
/// A distância vertical entre fileiras.
///
/// ⛔⛔⛔ **A premissa que aqui estava — *«a câmara abre com 10 unidades de altura»* — nunca foi
/// medida contra a superfície REAL, e estava errada por `0,47`.** O chrome desta cena não desenha
/// na janela: desenha numa **BANDA** entre a barra do topo e a timeline, e é a banda que decide o
/// enquadramento. Fotografada a cena (`fotografa_cena.sh`, janela `1930×1040`, arrumação de
/// fábrica) e lida a escala pelas próprias fichas (`3,2` de mundo = `178 px` ⇒ `55,6 px/unidade`),
/// a banda mostra **`8,665`** unidades e está centrada em **`−0,93`** — *ela não é centrada na
/// origem*. Com `ROW_GAP = 3,2` a cena media `9,133` centrada em `+0,13`: a fileira de CIMA saía
/// pelo topo e **as duas fichas dela eram invisíveis**.
///
/// ⚠️ *«Três fileiras mais as fichas têm de caber sem zoom — senão o primeiro passo do smoke é
/// procurar a cena»* continua a ser a lei; o que estava errado era o número contra o qual ela foi
/// verificada. ⭐ Os números de hoje são derivados da banda medida: altura `2·g + f + 1,233` (o
/// `1,233` é o quanto a manga desce abaixo do ponto em que é pousada, **medido**), e a folga que
/// sobra é `~0,24` de cada lado.
const ROW_GAP: f32 = 2.9;
/// A que altura acima do centro do pano pousa a ficha. ⚠️ Ela desceu com o `ROW_GAP` e pela mesma
/// medição — e o que a impede de descer mais é a fileira DE BAIXO: a ficha dela tem de ficar acima
/// do topo da manga (folga medida `0,86`).
const FICHA_Y: f32 = 1.15;
/// **O DESLOCAMENTO DA CENA INTEIRA**, para o centro dela cair no centro da BANDA.
///
/// ⭐ Encolher não chega: a banda está centrada em `−0,93` e a cena, pela forma como as três
/// fileiras se dispõem, centra-se perto da origem. ⛔ **E isto não se cura pela `FICHA_Y`** — a
/// conta pede `f = −0,62`, ou seja a ficha **por baixo** da fileira que ela nomeia.
///
/// ⚠️ Ele entra numa porta só ([`fileira_y`]), que é por onde os seis panos **e** as seis fichas
/// passam: escrito nos dois sítios, um deles ficaria para trás no dia seguinte.
const CENA_Y: f32 = -0.9;

// ── A CORDA ────────────────────────────────────────────────────────────────────────────────
/// Pontos da corda. ⚠️ **Não é um tecto — é o tamanho de uma corda que alguém faz**, e a §7 mediu
/// o preço dela: `272,7`–`275,5 ns/ponto`, PLANO sobre uma faixa de `14×`. A `20` isto custa
/// `0,005 ms`, `0,03 %` de um quadro.
const CORDA_PONTOS: f32 = 20.0;
/// O comprimento em unidades de mundo — a corda pendurada tem de caber no pano.
const CORDA_COMPRIMENTO: f32 = 1.9;
/// O semi-eixo de uma CONTA da corda — **derivado do espaçamento, não escolhido**.
///
/// ⛔⛔ **Ele valia `0,13` enquanto uma posição se desenhava como uma marca, e a forma mudou a
/// unidade:** a receita do `source.shape` corta a peça de uma caixa de largura `2 × size`, logo o
/// mesmo número passou a pintar um disco de `0,26` sobre um espaçamento de
/// `CORDA_COMPRIMENTO / (CORDA_PONTOS − 1) = 0,1` — **`2,6×` de sobreposição**, e a foto mostrava
/// uma BARRA BRANCA CONTÍNUA onde a legenda promete vinte pontos presos uns aos outros.
///
/// ⭐ Com `0,05` o disco mede exactamente o vão: as contas encostam em repouso, **separam-se onde
/// a corda estica e apertam onde ela encolhe** — que é a coisa que este pano existe para mostrar.
/// ⚠️ *Nenhum gate desta cena via isto: eles leem `P` e `size`, e os dois estavam certos.*
const CORDA_PECA: f32 = 0.05;
/// A porta `state` da corda. ⚠️ **Contada no manifesto** (`anchor_x` · `anchor_y` · `state`),
/// nunca adivinhada: um índice errado liga o laço a um ANCORADOURO e a corda voa.
const CORDA_PORTA_ESTADO: u16 = 2;

// ── O CAMPO ────────────────────────────────────────────────────────────────────────────────
/// O lado da grelha do campo. ⚠️⚠️ **O tecto dele é `512` desde a W4-bis** (§9 do doc) e este `11`
/// é o que cabe no pano — *o passo do tutorial é digitar um número maior e ver que ele obedece*,
/// que é a única forma de um tecto medido chegar ao artista.
const CAMPO_LADO: f32 = 11.0;
const CAMPO_VAO: f32 = 0.17;
const CAMPO_PECA: f32 = 0.1;
/// O período do pulso que alimenta a fonte do centro. Sem ele o campo é uma grelha PARADA, e a
/// cena ensinaria que uma onda não ondula.
const CAMPO_PULSO: f32 = 0.5;
/// A porta `state` do campo — contada no manifesto (`drive` · `state`).
const CAMPO_PORTA_ESTADO: u16 = 1;

// ── O ESQUELETO ────────────────────────────────────────────────────────────────────────────
/// Juntas da corrente: a raiz mais **quatro** ossos.
pub(crate) const OSSOS_JUNTAS: f32 = 5.0;
/// O comprimento de cada osso. ⚠️ Ele decide o ALCANCE do solver de duas juntas — `2 × 0,45` —, e
/// o alvo da direita é posto DENTRO desse alcance de propósito: um alvo inalcançável faz o braço
/// esticar-se e parar, que é a lei certa a parecer um defeito.
const OSSO_LEN: f32 = 0.45;
/// Para onde a raiz aponta: `90°` é a cadeia a subir do encaixe.
const OSSO_RAIZ: f32 = 90.0;
/// Quanto cada junta dobra, em graus, no pano do FK. A rampa multiplica-o, logo a dobra CRESCE ao
/// longo da cadeia — é isso que mostra que o ângulo é **por junta**.
const FK_DOBRA: f32 = 40.0;
/// O semi-eixo da peça de OSSO, e ele é **derivado e não escolhido**.
///
/// ⛔⛔ A receita do `source.shape` corta toda forma de uma caixa de **largura `2 × size`**, logo
/// a peça mede `2 × OSSO_PECA` de comprimento. Com `OSSO_LEN / 2` ela vai **exactamente** de uma
/// junta à seguinte: a cadeia ladrilha. Com o vão inteiro cada osso mediria o DOBRO do vão e a
/// corrente saía como uma massa contínua em que não se distingue peça nenhuma — o defeito que a
/// foto da cena irmã `=125` já pagou.
const OSSO_PECA: f32 = OSSO_LEN / 2.0;
/// ⭐⭐ **A ESBELTEZA do osso, DERIVADA da própria silhueta:** o `aspect` multiplica o semi-eixo
/// `y`, logo a altura é `2 × aspect × size` e o comprimento é `2 × size` ⇒ **`1/3` é, à letra,
/// «três vezes mais comprido do que largo»**. ⛔ Sem ele o valor de fábrica (`1`) faz da peça um
/// QUADRADO, e a fileira do meio mostra blocos onde devia mostrar ossos.
const OSSO_ESBELTEZA: f32 = 1.0 / 3.0;

// ── O ALVO DO IK ───────────────────────────────────────────────────────────────────────────
/// A altura do alvo acima da raiz, e o quanto ele varre para cada lado.
const ALVO_Y: f32 = 0.6;
const ALVO_VARRE: f32 = 0.5;
/// Voltas por segundo do varrimento.
const ALVO_RITMO: f32 = 0.35;
/// O raio do campo que dá a FORÇA da restrição. ⚠️ **`20` não é um tecto — é «grande ao pé da
/// corrente»:** com `4 × 0,45 = 1,8` de comprimento, a junta mais distante lê `falloff ≈ 0,99`, e o
/// pano abre com o solver à força cheia. O passo do tutorial é apertá-lo.
const FORCA_RAIO: f32 = 20.0;

// ── A PELE ─────────────────────────────────────────────────────────────────────────────────
/// A pele é uma **MANGA** e não um quadrado: `3` de largura por `7` de altura.
///
/// ⛔⛔ **A 1.ª redacção era `5 × 5` com vão `0,4`, e a FIGURA do tutorial refutou-a:** uma grelha
/// mais LARGA do que a corrente é comprida fica com metade das peças longe de qualquer osso, cada
/// uma a seguir o osso mais próximo por si — e o que se desenha são vinte e cinco pontos
/// dispersos, que se leem como RUÍDO e não como pele. ⚠️ *A suíte estava verde: os gates mediam
/// que os dois panos diferem, e dois ruídos diferentes também diferem.* Quem o apanhou foi olhar
/// para a imagem.
///
/// ⭐ Uma manga `0,44 × 1,68` embrulha a corrente (`4 × 0,45 = 1,8`), logo cada peça tem um osso
/// por perto e o conjunto deforma-se como uma coisa só.
const PELE_COLS: f32 = 3.0;
const PELE_ROWS: f32 = 7.0;
const PELE_VAO_X: f32 = 0.22;
const PELE_VAO_Y: f32 = 0.28;
const PELE_PECA: f32 = 0.11;
/// Quanto a pele sobe para ficar POR CIMA da corrente. ⚠️ Uma grelha centrada na origem só cobria
/// a metade de baixo dos ossos, e metade da pele não teria osso nenhum a puxá-la.
const PELE_SOBE: f32 = 0.9;

// ── As escadas dos enums, lidas com nome ───────────────────────────────────────────────────
/// `motion.drive`: o canal `Custom…`, o que escreve uma coluna PELO NOME.
const CANAL_CUSTOM: f32 = 9.0;
/// `motion.drive`: o modo `Set`.
const MODO_SET: f32 = 1.0;
/// `value.instance_field`: o modo `Ramp` (`i/(N−1)` em `[0,1]`).
const CAMPO_RAMPA: f32 = 1.0;
/// `motion.oscillator`: o canal `X`.
const OSC_X: f32 = 0.0;

/// A coluna do QUINHÃO de cada osso — o contrato entre a caneta e o `rig.skin_deformer`.
///
/// ⚠️⚠️ **É um literal e NÃO o `ph2d_node_rig_skin_deformer::BONE_WEIGHT`, e a razão é de
/// arquitectura:** esta crate monta grafos por NOME DE TIPO (`"rig.skin_deformer"`) e não depende
/// de nenhuma crate de nó — depender de uma por uma constante abriria a porta a depender das
/// trinta. ⛔ O preço é uma segunda cópia de um contrato, e o que o paga é um GATE e não uma
/// promessa: `o_quinhao_por_osso_muda_a_pele` compara os dois panos de baixo, e com o nome errado
/// a caneta escreve uma coluna que ninguém lê ⇒ os dois ficam idênticos e ele reprova.
const COLUNA_QUINHAO: &str = "bone_weight";
/// A coluna do ÂNGULO de cada junta — o mesmo caso, e o gate que a paga é o
/// `a_cinematica_directa_e_a_inversa_dao_panos_diferentes` (sem a dobra, o pano do FK é uma
/// corrente recta e a diferença para o IK deixa de ser a que a cena promete).
const COLUNA_ROT: &str = "rot";
/// A coluna do catálogo que TODO campo escreve — o `motion.falloff` e a família `field.*`.
const COLUNA_FALLOFF: &str = "falloff";
/// Até que fracção da corrente o quinhão vale `1`. ⚠️ **`0,5` sobre CINCO juntas dá `s = i/4` ⇒
/// `0 · 0,25 · 0,5` dentro e `0,75 · 1` fora**: as duas últimas deixam de puxar. *O número lê-se
/// na aritmética da banda, não se escolhe por gosto.*
const BANDA_DO_QUINHAO: f32 = 0.5;

fn no(doc: &mut MotionDoc, tipo: &str, x: f32, y: f32) -> NodeId {
    let n = doc.graph.add_node(tipo);
    doc.graph.set_pos(n, Pos { x, y });
    n
}

fn liga(doc: &mut MotionDoc, de: NodeId, para: (NodeId, u16)) -> Option<()> {
    doc.graph
        .connect(Edge {
            from: (de, 0),
            to: para,
            delayed: false,
        })
        .ok()
}

/// **O LAÇO DE ESTADO de um solver** — a saída dele a voltar à porta `state`, ATRASADA.
///
/// ⛔⛔ **Sem ele a corda não se mexe, e o modo de falha é MUDO:** o solver recebe um estado vazio
/// a cada quadro, recomeça do repouso e entrega sempre o mesmo pano. Nada estoura, nada avisa — o
/// dono vê uma corda pendurada e imóvel e conclui que a ferramenta está partida. *Foi o gate
/// `a_corda_balanca_e_o_campo_ondula` que o apanhou, e ele existe por isto.*
///
/// ⚠️ **`delayed` não é uma afinação: é o que torna o ciclo legal.** Uma aresta normal de um nó
/// para si próprio é um ciclo e o cozedor **recusa o grafo**; atrasada, ela diz *«o estado do
/// tique passado»*, que é o que uma simulação de facto lê.
fn laco_de_estado(doc: &mut MotionDoc, n: NodeId, porta: u16) -> Option<()> {
    doc.graph
        .connect(Edge {
            from: (n, 0),
            to: (n, porta),
            delayed: true,
        })
        .ok()
}

/// **A FORMA que um pano veste** — o que o [`pousa`] carimba em cada posição da corrente.
///
/// ⛔⛔⛔ **Ela não é enfeite: sem ela o pano não existe.** Desde 2026-09-19 (ordem do dono, que
/// retirou os gizmos dos nós que só passam posições) vale a lei
/// [`ph2d_eval_motion::tem_aparencia`] — *uma corrente que não veio de uma forma não vira pixel* —,
/// e ela ship LIGADA (só `PH2D_MOTION_SO_COM_FORMA=0` a desliga). Esta cena é de 17/09 e os seis
/// panos dela acabavam num `motion.output` sem forma nenhuma pelo caminho.
#[derive(Clone, Copy)]
struct Veste {
    /// O índice da forma no catálogo do `source.shape`.
    ///
    /// ⚠️ **Resolvido pelo NOME e nunca escrito como literal** ([`super::sim_demo::indice_de`]):
    /// a lista de formas é *append-only* e já foi a `45`, logo um número aqui passa a apontar
    /// para outra forma no dia em que alguém insira uma — **em silêncio**, porque um índice
    /// válido desenha sempre alguma coisa.
    kind: f32,
    /// O multiplicador do semi-eixo Y. `1` é uma peça tão alta quanto longa.
    aspect: f32,
}

impl Veste {
    const fn redonda(kind: f32) -> Self {
        Self { kind, aspect: 1.0 }
    }
}

/// `escala → move(centro) → FORMA carimbada → output`, o final comum dos seis panos.
/// ⚠️ **Ele é o mesmo em toda a cena de propósito:** o que muda de pano para pano é só o que
/// SEGURA — e, desde a lei da aparência, a peça com que ele se veste.
///
/// ⭐⭐ **O `motion.scale` continua a ser quem decide o tamanho da peça**, e é o `point_scale = 1`
/// que o faz chegar ao carimbo: o valor de fábrica do duplicador é `0` — *«a forma vence e a
/// escala do ponto é deitada fora»* —, e com ele os seis `_PECA` desta cena não teriam consumidor.
/// *É a ordem que o [doc 115 §32.5](../../docs/Motion%20Nodes/115_o_colisor_sai_do_grafo.md)
/// prescreve, à letra.*
///
/// ⚠️ A forma entra na porta `0` e os pontos na `1` — a ordem do manifesto do duplicador, e as
/// **duas são obrigatórias** (ADR-0155): sem uma delas o nó não tem o que copiar.
fn pousa(
    doc: &mut MotionDoc,
    de: NodeId,
    veste: Veste,
    tamanho: f32,
    centro: [f32; 2],
    y: f32,
) -> Option<NodeId> {
    let s = no(doc, "motion.scale", 240.0, y);
    doc.graph.set_param(s, "amount", tamanho);
    let mv = no(doc, "motion.move", 420.0, y);
    doc.graph.set_param(mv, "dx", centro[0]);
    doc.graph.set_param(mv, "dy", centro[1]);
    let forma = no(doc, "source.shape", 420.0, y + 150.0);
    doc.graph
        .set_param(forma, ph2d_node_motion_shape::param::KIND, veste.kind);
    doc.graph
        .set_param(forma, ph2d_node_motion_shape::param::ASPECT, veste.aspect);
    let dup = no(doc, "motion.duplicator", 560.0, y);
    doc.graph.set_param(dup, "point_scale", 1.0);
    let o = no(doc, "motion.output", 700.0, y);
    liga(doc, de, (s, 0))?;
    liga(doc, s, (mv, 0))?;
    liga(doc, forma, (dup, 0))?;
    liga(doc, mv, (dup, 1))?;
    liga(doc, dup, (o, 0))?;
    Some(o)
}

/// **A CANETA GENÉRICA** — `value.instance_field(Ramp) → motion.drive(Custom…, coluna)`.
///
/// ⭐⭐ Ela é a razão de a W1 deste ciclo ter sido **refutada antes de ser construída** (§3.1 do
/// doc): o escritor de coluna por nome já existia, e a folha 16 dizia por escrito que não. Dois
/// cartões, e escrevem `parent`, `len`, `rot`, `bone_weight` — qualquer coluna.
///
/// ⚠️ **A fonte entra nos DOIS cartões:** no campo (que só precisa da CONTAGEM para fazer a
/// rampa) e no `drive` (que é quem leva a corrente adiante). Ligar só um deles dá uma rampa sem
/// sujeito ou um sujeito sem rampa — e o `drive` **recusa escrever sem valor** desde o ciclo 8.
fn caneta(doc: &mut MotionDoc, fonte: NodeId, coluna: &str, escala: f32, y: f32) -> Option<NodeId> {
    let campo = no(doc, "value.instance_field", -220.0, y - 90.0);
    doc.graph.set_param(campo, "mode", CAMPO_RAMPA);
    let d = no(doc, "motion.drive", -60.0, y);
    doc.graph.set_param(d, "channel", CANAL_CUSTOM);
    doc.graph.set_param(d, "mode", MODO_SET);
    doc.graph.set_param(d, "scale", escala);
    doc.graph.set_text_param(d, "column", coluna);
    liga(doc, fonte, (campo, 0))?;
    liga(doc, fonte, (d, 0))?;
    liga(doc, campo, (d, 1))?;
    Some(d)
}

/// **A CANETA DA BANDA** — `field.index_range → value.attribute("falloff") → motion.drive(coluna)`.
///
/// ⛔⛔ **A 1.ª redacção do envelope era a caneta da RAMPA, e a FIGURA do tutorial refutou-a.** Uma
/// rampa dá `0` na raiz e `1` na ponta — e a ponta é justamente onde a corrente MAIS se dobra,
/// logo o quinhão pequeno caía sobre a parte da pele que já não se mexia: as duas figuras de baixo
/// diferiam `0,39` do lado de uma peça (mediana `0,20`), *duas imagens que o leitor lê como
/// iguais*. ⚠️ **A suíte estava verde** — os gates exigiam que os dois panos diferissem, e dois
/// panos que diferem por um quinto de peça diferem.
///
/// ⭐⭐ A banda faz o contrário e é o que se vê: as juntas do **fim** ficam com quinhão ZERO, logo a
/// ponta da manga deixa de seguir a dobra e fica para trás. ⭐ E ela reusa o mesmo conceito do
/// cartão `Strength` do pano do IK — *um campo decide QUANTO, e o par `value.attribute` +
/// `motion.drive` leva esse número para a coluna que se quiser*. Dois panos, uma lição.
fn caneta_da_banda(doc: &mut MotionDoc, fonte: NodeId, coluna: &str, y: f32) -> Option<NodeId> {
    let banda = no(doc, "field.index_range", -300.0, y - 90.0);
    doc.graph.set_param(banda, "start", 0.0);
    doc.graph.set_param(banda, "end", BANDA_DO_QUINHAO);
    // ⚠️ `soft = 0` é o DEGRAU: com a borda macia de fábrica (`0,10`) as cinco juntas caem todas
    // dentro da rampa e o quinhao vira outra rampa — a lei que esta função existe para não repetir.
    doc.graph.set_param(banda, "soft", 0.0);
    doc.graph.set_label(banda, "Range: que ossos puxam");
    let a = no(doc, "value.attribute", -160.0, y - 90.0);
    doc.graph.set_text_param(a, "attr", COLUNA_FALLOFF);
    let d = no(doc, "motion.drive", -20.0, y);
    doc.graph.set_param(d, "channel", CANAL_CUSTOM);
    doc.graph.set_param(d, "mode", MODO_SET);
    doc.graph.set_param(d, "scale", 1.0);
    doc.graph.set_text_param(d, "column", coluna);
    doc.graph.set_label(d, "Drive: o quinhao de cada osso");
    liga(doc, fonte, (banda, 0))?;
    liga(doc, banda, (a, 0))?;
    liga(doc, fonte, (d, 0))?;
    liga(doc, a, (d, 1))?;
    Some(d)
}

/// **OS OSSOS de uma corrente resolvida** — o nó que tem de existir entre um solver e a peça.
///
/// ⛔⛔⛔ **Carimbar a `Shape: Bone` na corrente do solver desenha-a UMA JUNTA À FRENTE.** A lei
/// do `fk::resolve` é `P[i] = P[pai] + len[i] · (cos wrot[i], sin wrot[i])`, logo o `len`/`rot`
/// que o elemento `i` carrega são os do osso que **CHEGA** a ele e o `P[i]` dele é a **PONTA**
/// desse osso: *um elemento é uma JUNTA, e o osso que ele carrega vive ATRÁS dele*. Sem este nó
/// a cadeia não ladrilha — cada peça sai pelo lado de fora do arco e a última fica pendurada
/// para lá da corrente (medido no cabeçalho do [`ph2d_node_rig_bones`], com a tabela).
///
/// ⚠️ **Ele vem DEPOIS do solver**, sempre: ele lê a corrente **resolvida**, e pô-lo antes da
/// pose entregaria os ossos da pose anterior.
///
/// ⚠️ A corrente tem `OSSOS_JUNTAS` juntas e sai com `OSSOS_JUNTAS − 1` ossos — **a raiz é a
/// única junta sem osso a chegar**, e emitir um osso por junta obrigaria sempre uma peça a
/// mentir.
fn ossos_de(doc: &mut MotionDoc, solver: NodeId, y: f32) -> Option<NodeId> {
    let b = no(doc, "rig.bones", 300.0, y);
    liga(doc, solver, (b, 0))?;
    Some(b)
}

/// A corrente de juntas — a fonte das três cadeias de rig desta cena.
fn esqueleto(doc: &mut MotionDoc, y: f32) -> NodeId {
    let e = no(doc, "rig.skeleton", -400.0, y);
    doc.graph.set_param(e, "joints", OSSOS_JUNTAS);
    doc.graph.set_param(e, "length", OSSO_LEN);
    doc.graph.set_param(e, "root_angle", OSSO_RAIZ);
    e
}

/// A pele de um dos dois panos de baixo. `envelope = true` mete a caneta no REPOUSO.
///
/// ⚠️⚠️ **O envelope escreve-se no REPOUSO e não na POSE, e a ordem não é arbitrária:** o
/// `rig.skin_deformer` lê os pesos do osso na porta `rest` (é ali que ele monta os ossos), e a
/// `posed` é só onde eles foram parar. Escrito na pose, o número chega ao solver e ele
/// **descarta-o** — o defeito que o `CLAUDE.md` §5.0 chama *«o consumidor que PROJECTA o valor
/// fora»*, que nenhuma sonda de fiação vê porque o campo **é** lido.
fn pele(doc: &mut MotionDoc, envelope: bool, y: f32) -> Option<NodeId> {
    let e = esqueleto(doc, y);

    // O REPOUSO — a corrente como foi autorada, com ou sem o envelope por cima.
    let fonte_repouso = if envelope {
        caneta_da_banda(doc, e, COLUNA_QUINHAO, y)?
    } else {
        e
    };
    let fk_repouso = no(doc, "rig.fk", 60.0, y - 180.0);
    liga(doc, fonte_repouso, (fk_repouso, 0))?;

    // A POSE — a MESMA corrente com o `rot` conduzido por uma rampa: ela curva-se.
    let dobra = caneta(doc, e, COLUNA_ROT, FK_DOBRA, y + 200.0)?;
    let fk_pose = no(doc, "rig.fk", 60.0, y + 200.0);
    liga(doc, dobra, (fk_pose, 0))?;

    // A PELE — uma grelha pousada POR CIMA da corrente.
    let g = no(doc, "motion.grid", -400.0, y - 360.0);
    doc.graph.set_param(g, "rows", PELE_ROWS);
    doc.graph.set_param(g, "cols", PELE_COLS);
    doc.graph.set_param(g, "gap_x", PELE_VAO_X);
    doc.graph.set_param(g, "gap_y", PELE_VAO_Y);
    let sobe = no(doc, "motion.move", -220.0, y - 360.0);
    doc.graph.set_param(sobe, "dy", PELE_SOBE);
    liga(doc, g, (sobe, 0))?;

    let p = no(doc, "rig.skin_deformer", 120.0, y);
    doc.graph.set_label(
        p,
        if envelope {
            "Skin: cada osso o seu quinhao"
        } else {
            "Skin: os ossos todos"
        },
    );
    liga(doc, sobe, (p, 0))?;
    liga(doc, fk_repouso, (p, 1))?;
    liga(doc, fk_pose, (p, 2))?;
    Some(p)
}

/// Onde a fileira `k` vive, **já deslocada** para o centro da banda ([`CENA_Y`]).
///
/// ⚠️ **Os seis panos e as seis fichas passam TODOS por aqui**, e é isso que torna o deslocamento
/// uma lei e não dois números que têm de concordar.
fn fileira_y(k: usize) -> f32 {
    CENA_Y
        + match k {
            0 => ROW_GAP,
            1 => 0.0,
            _ => -ROW_GAP,
        }
}

fn grafo_y(k: usize, lado: usize) -> f32 {
    #[expect(clippy::cast_precision_loss, reason = "seis cadeias")]
    let i = (k * 2 + lado) as f32;
    -1400.0 + i * 560.0
}

pub(super) fn build(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    let mut sinks = Vec::new();

    // ── AS TRÊS VESTES, resolvidas pelo NOME uma vez só ────────────────────────────────
    //
    // ⛔⛔ **A corda leva um CÍRCULO e não o `Shape: Rope Segment`, e a razão é MEDIDA:** o
    // `motion.verlet_rope` publica `P`, `rope_prev` e `sim_t` — e **nenhuma coluna `rot`**. O
    // segmento de corda é uma forma ORIENTADA (um cordão com um nó em cada ponta, cortado de `a`
    // a `b`), logo sem ângulo os vinte sairiam todos deitados na horizontal: uma corda pendurada
    // desenhada como uma pilha de traços. *É a espécie que o `CLAUDE.md` §5.0 chama de pior que
    // uma cena ausente.* ⭐ O círculo é a conta honesta — uma corda de Verlet É uma fila de
    // partículas —, e não precisa de direcção nenhuma para o ser.
    //
    // ⏳ E fica NOMEADO: o `Shape: Rope Segment` entrou no catálogo em 2026-09-19 e **não tem um
    // único consumidor no repo**. Quem o quiser usar precisa de um nó que escreva `rot` a partir
    // da direcção ao vizinho — o `rig.bones` não serve (exige `parent`/`len`/`rot`, que é uma
    // corrente de rig) e o `motion.look_at` também não (ele aponta a um ALVO, não ao seguinte).
    let circulo = Veste::redonda(super::sim_demo::indice_de(
        reg,
        "source.shape",
        "kind",
        "Circle",
    )?);
    let quadrado = Veste::redonda(super::sim_demo::indice_de(
        reg,
        "source.shape",
        "kind",
        "Square",
    )?);
    let osso = Veste {
        kind: super::sim_demo::indice_de(reg, "source.shape", "kind", "Bone")?,
        aspect: OSSO_ESBELTEZA,
    };

    // ── CIMA: O QUE SE SEGURA SOZINHO — a corda · o campo ───────────────────────────────
    {
        let y = grafo_y(0, 0);
        let c = no(doc, "motion.verlet_rope", -400.0, y);
        doc.graph.set_param(c, "count", CORDA_PONTOS);
        doc.graph.set_param(c, "length", CORDA_COMPRIMENTO);
        doc.graph.set_label(c, "Verlet Rope: a corda");
        laco_de_estado(doc, c, CORDA_PORTA_ESTADO)?;
        // ⚠️ A corda PENDURA-SE, logo o pano dela sobe: centrada, metade dela sairia por baixo.
        sinks.push(pousa(
            doc,
            c,
            circulo,
            CORDA_PECA,
            [-COL_X, fileira_y(0) + 0.9],
            y,
        )?);
    }
    {
        let y = grafo_y(0, 1);
        let w = no(doc, "motion.wave", -400.0, y);
        doc.graph.set_param(w, "rows", CAMPO_LADO);
        doc.graph.set_param(w, "cols", CAMPO_LADO);
        doc.graph.set_param(w, "spacing", CAMPO_VAO);
        doc.graph.set_label(w, "Wave: o campo");
        laco_de_estado(doc, w, CAMPO_PORTA_ESTADO)?;
        // A fonte do centro precisa de um pulso — sem ele a grelha fica parada.
        let lfo = no(doc, "value.lfo", -600.0, y);
        doc.graph.set_param(lfo, "period", CAMPO_PULSO);
        doc.graph.set_param(lfo, "amplitude", 1.0);
        liga(doc, lfo, (w, 0))?;
        sinks.push(pousa(
            doc,
            w,
            circulo,
            CAMPO_PECA,
            [COL_X, fileira_y(0)],
            y,
        )?);
    }

    // ── MEIO: QUEM SEGURA — os pais decidem · a mão vai ao alvo ─────────────────────────
    {
        let y = grafo_y(1, 0);
        let e = esqueleto(doc, y);
        let dobra = caneta(doc, e, COLUNA_ROT, FK_DOBRA, y)?;
        // ⚠️ **O rótulo não é enfeite: o passo 7 do tutorial manda clicar NESTE cartão**, e a cena
        // tem quatro `motion.drive`. *Um passo que diz «o Drive» num grafo com quatro é um passo
        // que o dono não consegue executar.*
        doc.graph.set_label(dobra, "Drive: o angulo de cada junta");
        let f = no(doc, "rig.fk", 120.0, y);
        doc.graph.set_label(f, "FK: os pais decidem");
        liga(doc, dobra, (f, 0))?;
        let b = ossos_de(doc, f, y)?;
        sinks.push(pousa(
            doc,
            b,
            osso,
            OSSO_PECA,
            [-COL_X, fileira_y(1) - 0.9],
            y,
        )?);
    }
    {
        let y = grafo_y(1, 1);
        let e = esqueleto(doc, y);
        // O ALVO: um ponto só, a varrer de um lado ao outro DENTRO do alcance.
        let alvo = no(doc, "motion.grid", -400.0, y + 200.0);
        doc.graph.set_param(alvo, "rows", 1.0);
        doc.graph.set_param(alvo, "cols", 1.0);
        let sobe = no(doc, "motion.move", -240.0, y + 200.0);
        doc.graph.set_param(sobe, "dy", ALVO_Y);
        let osc = no(doc, "motion.oscillator", -80.0, y + 200.0);
        doc.graph.set_param(osc, "channel", OSC_X);
        doc.graph.set_param(osc, "amplitude", ALVO_VARRE);
        doc.graph.set_param(osc, "frequency", ALVO_RITMO);
        doc.graph.set_label(osc, "Target: onde a mao tem de estar");
        liga(doc, alvo, (sobe, 0))?;
        liga(doc, sobe, (osc, 0))?;

        // ⭐⭐ **A FORÇA da restrição, e ela é a coluna que já existia** (§7-W2 do doc): o
        // `motion.falloff` escreve `falloff`, e os três solvers do grupo misturam a pose resolvida
        // com a de repouso por esse número. ⚠️ **Ele nasce com um raio GRANDE de propósito** —
        // a corrente inteira fica dentro dele, a força lê `≈ 1` e o pano abre à força cheia; o
        // passo do tutorial é APERTAR o raio e ver a mão ficar a meio caminho.
        let forca = no(doc, "motion.falloff", -80.0, y);
        doc.graph.set_param(forca, "radius", FORCA_RAIO);
        doc.graph
            .set_label(forca, "Strength: quanto a restricao puxa");
        liga(doc, e, (forca, 0))?;

        let ik = no(doc, "rig.ik_2bone", 120.0, y);
        doc.graph.set_label(ik, "IK: a mao vai ao alvo");
        liga(doc, forca, (ik, 0))?;
        liga(doc, osc, (ik, 1))?;
        let b = ossos_de(doc, ik, y)?;
        sinks.push(pousa(
            doc,
            b,
            osso,
            OSSO_PECA,
            [COL_X, fileira_y(1) - 0.9],
            y,
        )?);
    }

    // ── BAIXO: A PELE — todos por igual · cada osso o seu quinhão ───────────────────────
    for lado in 0..2 {
        let y = grafo_y(2, lado);
        let x = if lado == 0 { -COL_X } else { COL_X };
        let p = pele(doc, lado == 1, y)?;
        // ⚠️ A pele veste QUADRADOS e não círculos: uma manga é feita de retalhos, e são eles que
        // fazem a deformação ler-se como tecido em vez de uma nuvem de pontos.
        sinks.push(pousa(
            doc,
            p,
            quadrado,
            PELE_PECA,
            [x, fileira_y(2) - 0.9],
            y,
        )?);
    }

    doc.graph.validate(reg).ok()?;
    Some(sinks)
}

/// As fichas que a cena pousa no canvas — uma por pano, com o título de cada fileira à esquerda.
pub(super) fn captions() -> Vec<Caption> {
    let ficha = |x: f32, k: usize, t: &str| Caption {
        text: t.to_string(),
        world: [x, fileira_y(k) + FICHA_Y],
    };
    vec![
        ficha(-COL_X, 0, "SEGURA-SE SOZINHO: a corda"),
        ficha(COL_X, 0, "...e o campo"),
        ficha(-COL_X, 1, "QUEM SEGURA: os pais decidem (FK)"),
        ficha(COL_X, 1, "...ou a MAO decide (IK)"),
        ficha(-COL_X, 2, "A PELE: os ossos todos puxam"),
        ficha(COL_X, 2, "...cada osso o SEU quinhao"),
    ]
}

#[cfg(test)]
#[path = "motion_state_rig_demo_tests.rs"]
mod tests;
/// Os gates do TUTORIAL deste ciclo — irmãos dos da cena pelo teto de LOC; ver o cabeçalho deles.
#[cfg(all(test, feature = "panel-motion-graph"))]
#[path = "motion_state_rig_tutorial_tests.rs"]
mod tutorial_tests;
