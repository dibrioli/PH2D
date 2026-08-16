//! **O DISCO TEM O TAMANHO QUE SE VÊ** (`PH2D_GPU_COOK_DEMO=48`) — a cena do
//! **grupo H** da conferência (doc 89, folha 03, linhas 60 e 62).
//!
//! ## A leitura é por PARES, como nos grupos F e G
//!
//! Cada par é **a mesma fileira apertada com uma coluna de diferença**, e a
//! pergunta nunca é *"apareceu alguma coisa?"* — é *"as duas fazem coisas
//! DIFERENTES, e a diferença é a que a wave promete?"*.
//!
//! 1. **TAMANHO** — as duas fileiras nascem no MESMO amontoado e são relaxadas
//!    pelo MESMO `motion.collide`. Em cima toda peça tem o tamanho de fábrica e o
//!    passo do empacotamento é UNIFORME; em baixo o tamanho cresce da esquerda
//!    para a direita e **o vão cresce com ele** — as peças grandes reclamam mais
//!    espaço, e é isso que `r_i + r_j` significa. ⚠️ Em cima as peças grandes
//!    ficam **sobrepostas**: é literalmente o que o nó fazia antes desta wave, com
//!    o próprio doc dele a prometer que dois discos apenas se tocam.
//! 2. **FALLOFF** — o mesmo rig com um `field.box` no meio. Em cima o campo não
//!    existe e a fileira empacota inteira; em baixo as peças **dentro da caixa se
//!    ATRAVESSAM** (o nó não age ali) e as de fora seguem afastadas.
//! 3. **MUTAR ≠ PINAR** — a distinção que a folha nomeia, e ela precisa das duas
//!    metades. Em cima a peça do meio tem `falloff = 0`: ela é **transparente**, e
//!    as vizinhas empacotam **através** dela. Em baixo a MESMA peça tem
//!    `inv_mass = 0` (o `motion.pin_constraint`): ela é **obstáculo** — não se
//!    move e as vizinhas se amontoam em volta.
//!
//! ⚠️ **Esta cena julga-se PARADA.** Nada aqui depende do relógio: o
//! `motion.collide` é `Effect::Pure` e relaxa a entrada a cada cook.
//!
//! ⚠️ **As peças são REDONDAS de propósito.** O nó empacota DISCOS, e um quadrado
//! faria a mesma folga parecer errada nas diagonais — o olho leria a geometria da
//! peça em vez da lei.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// Quantas peças por fileira. Ímpar de propósito: as bandas 5-6 precisam de um
/// meio EXATO para pôr a peça mutada/pinada nele.
pub(crate) const COLS: usize = 9;
/// Quantas fileiras a cena empilha (três PARES).
pub(crate) const BANDS: usize = 6;
/// A distância vertical entre fileiras.
const BAND_GAP: f32 = 1.05;

/// O passo do amontoado de partida. **Muito mais apertado que `2·RADIUS`**, senão
/// não há sobreposição para desfazer e as seis fileiras saem idênticas — mas **não
/// apertado demais**: a relaxação de Jacobi propaga uma peça por varredura ao
/// longo de uma corrente, então um amontoado extremo não CONVERGE em 64 passos e
/// os vãos saem irregulares, que se lê como a lei a falhar. Medido: a `0,12` a
/// fileira parava com vãos entre `0,229` e `0,397` contra um alvo de `0,42`.
///
/// ⚠️ **O raio efetivo é `RADIUS · DOT`, e não `RADIUS`** — é a wave inteira: o
/// disco mede o que a peça mede. A primeira versão desta cena escolheu os três
/// números como se fossem independentes (`PITCH 0,16` contra um `RADIUS 0,24` que
/// a escala de `0,2` levava a **0,048**), e as seis fileiras saíram byte a byte
/// iguais — a fixture não continha o fenômeno, e o gate do amontoado a pegou.
const PITCH: f32 = 0.32;
/// O raio de fábrica de um disco, ANTES da escala da peça.
const RADIUS: f32 = 0.6;
/// Varreduras. Alto o bastante para a fileira CONVERGIR — um empacotamento a meio
/// caminho tem folgas irregulares que se leem como a lei a falhar.
const SWEEPS: f32 = 64.0;

/// O tamanho de repouso de uma peça (bandas 1, 3-6). Com o `RADIUS` acima isto dá
/// um raio efetivo de **0,21**, logo um diâmetro de `0,42` contra um passo de
/// partida de `0,32`: cada peça nasce a **76%** do diâmetro da vizinha.
const DOT: f32 = 0.35;
/// Quanto o tamanho cresce ao longo da fileira variada (banda 2).
///
/// ⚠️ **É uma régua, não um gosto:** o `value.instance_field` em Ramp devolve
/// `i/(N−1) ∈ [0, 1]`, então com este ganho os tamanhos vão de `DOT` a `2,2 · DOT`
/// e o maior disco é **2,2× o menor**. Menos que isso e o leque do empacotamento
/// desaparece no ruído do olho; muito mais e a fileira sai do quadro.
const SIZE_RAMP: f32 = 1.2;

/// A meia-largura da caixa que muda o meio da fileira (banda 4), em unidades de
/// mundo. ⚠️ **Tem de cobrir so o MIOLO:** com meia-largura de `1,0` ela engolia
/// a fileira inteira, todo par ficava mudo e NADA se movia -- a banda saia identica
/// ao amontoado de partida, que e a feature a funcionar e a cena a nao a mostrar.
/// A `0,5` ela cobre tres pecas, e as de fora seguem a empacotar contra o miolo.
const BOX_HALF: f32 = 0.5;

/// Os rótulos que o roteador imprime. ⚠️ A contagem é DERIVADA daqui.
pub(crate) const BAND_LABELS: [&str; BANDS] = [
    "1  tamanho UNIFORME  (o passo e' o mesmo em toda a fileira -- e as pecas grandes SOBREPOEM)",
    "2  tamanho VARIADO   (o vao cresce com o disco: r_i + r_j)",
    "3  sem falloff       (a fileira inteira empacota)",
    "4  falloff de CAIXA  (as pecas do MEIO se ATRAVESSAM; as de fora seguem afastadas)",
    "5  a do meio MUTADA  (transparente: as vizinhas empacotam ATRAVES dela)",
    "6  a do meio PINADA  (obstaculo: ninguem a atravessa, e a fileira empacota em volta)",
];

fn wire(g: &mut Graph, from: NodeId, out_port: u16, to: NodeId, in_port: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, out_port),
        to: (to, in_port),
        delayed: false,
    })
    .ok()
}

/// O que cada fileira acrescenta ao rig comum.
#[derive(Clone, Copy, PartialEq)]
enum Kind {
    /// A fileira nua — o controle das bandas 1 e 3.
    Plain,
    /// O tamanho sobe da esquerda para a direita (banda 2).
    SizeRamp,
    /// Um `field.box` muda o meio (banda 4).
    BoxFalloff,
    /// A peça do meio é MUTADA — `falloff = 0` só nela (banda 5).
    MuteMiddle,
    /// A peça do meio é PINADA — `inv_mass = 0` só nela (banda 6).
    PinMiddle,
}

/// Constrói uma fileira e devolve o nó de saída dela.
fn build_band(g: &mut Graph, row: usize, kind: Kind) -> Option<NodeId> {
    #[expect(clippy::cast_precision_loss, reason = "seis fileiras")]
    let offset_y = (row as f32) * -BAND_GAP;
    let x = 40.0f32;
    #[expect(clippy::cast_precision_loss, reason = "seis fileiras")]
    let y = (row as f32) * 130.0;

    // O amontoado de partida: UMA linha de peças a um passo muito menor que o
    // diâmetro, então quase todo par nasce sobreposto.
    let grid = g.add_node("motion.grid");
    g.set_pos(grid, Pos { x, y });
    g.set_param(grid, "rows", 1.0);
    #[expect(clippy::cast_precision_loss, reason = "quinze colunas")]
    g.set_param(grid, "cols", COLS as f32);
    g.set_param(grid, "gap_x", PITCH);
    g.set_param(grid, "gap_y", PITCH);

    // O tamanho de repouso, para as peças serem visíveis e a banda 2 ter uma base
    // sobre a qual a rampa soma.
    let base_size = g.add_node("motion.scale");
    g.set_pos(base_size, Pos { x: x + 150.0, y });
    g.set_param(base_size, "amount", DOT);
    g.set_param(base_size, "uniform", 1.0);
    wire(g, grid, 0, base_size, 0)?;

    // A fileira desce para a própria faixa.
    let place = g.add_node("motion.move");
    g.set_pos(place, Pos { x: x + 300.0, y });
    g.set_param(place, "dx", 0.0);
    g.set_param(place, "dy", offset_y);
    wire(g, base_size, 0, place, 0)?;

    let mut head = place;

    if kind == Kind::SizeRamp {
        // `i/(N−1)` → o canal Size, somado sobre o tamanho de repouso.
        let field = g.add_node("value.instance_field");
        g.set_pos(
            field,
            Pos {
                x: x + 300.0,
                y: y + 60.0,
            },
        );
        g.set_param(field, "mode", 1.0); // Ramp
        wire(g, head, 0, field, 0)?;
        let drive = g.add_node("motion.drive");
        g.set_pos(drive, Pos { x: x + 450.0, y });
        // O vocabulário de canais do `motion.drive`: 0 X · 1 Y · 2 Rotation ·
        // 3 Size · 4 Opacity. ⚠️ Não há const pública para citar (elas são
        // `pub(crate)` no crate do nó), então o gate da cena mede a PROPRIEDADE —
        // que a fileira 2 tem tamanhos que VARIAM — em vez de repetir o número.
        g.set_param(drive, "channel", 3.0);
        g.set_param(drive, "mode", 0.0); // Add, sobre o tamanho que já está lá
        g.set_param(drive, "scale", SIZE_RAMP * DOT);
        wire(g, head, 0, drive, 0)?;
        wire(g, field, 0, drive, 1)?;
        head = drive;
    }

    if kind == Kind::BoxFalloff {
        // O campo espacial que zera o peso do nó no MEIO da fileira. `invert` a 1
        // porque uma caixa nasce a valer 1 DENTRO — e o que a cena quer é o nó
        // agir FORA dela.
        let boxf = g.add_node("field.box");
        g.set_pos(boxf, Pos { x: x + 450.0, y });
        g.set_param(boxf, "width", BOX_HALF * 2.0);
        g.set_param(boxf, "height", 1.0);
        g.set_param(boxf, "center_x", 0.0);
        g.set_param(boxf, "center_y", offset_y);
        g.set_param(boxf, "soft", 0.0);
        g.set_param(boxf, "invert", 1.0);
        wire(g, head, 0, boxf, 0)?;
        head = boxf;
    }

    if kind == Kind::MuteMiddle || kind == Kind::PinMiddle {
        // As duas metades do par 5-6 saem do MESMO nó de constraint, que escreve
        // `inv_mass`; o que difere é para onde o número vai.
        let pin = g.add_node("motion.pin_constraint");
        g.set_pos(pin, Pos { x: x + 450.0, y });
        #[expect(clippy::cast_precision_loss, reason = "quinze colunas")]
        g.set_param(pin, "first", (COLS / 2) as f32);
        g.set_param(pin, "count", 1.0);
        g.set_param(pin, "strength", 1.0);
        wire(g, head, 0, pin, 0)?;
        head = pin;

        if kind == Kind::MuteMiddle {
            // MUTAR: o mesmo elemento, mas o número entra pelo `falloff`. O
            // `value.attribute` lê a coluna que o pin escreveu e o `motion.drive`
            // a deposita no canal Falloff — `inv_mass = 0` vira `falloff = 0`.
            let attr = g.add_node("value.attribute");
            g.set_pos(
                attr,
                Pos {
                    x: x + 600.0,
                    y: y + 60.0,
                },
            );
            g.set_text_param(attr, ph2d_node_value_attribute::ATTR_KEY, "inv_mass");
            wire(g, head, 0, attr, 0)?;
            let drive = g.add_node("motion.drive");
            g.set_pos(drive, Pos { x: x + 750.0, y });
            g.set_param(drive, "channel", 5.0); // Falloff
            g.set_param(drive, "mode", 1.0); // Set
            g.set_param(drive, "scale", 1.0);
            wire(g, head, 0, drive, 0)?;
            wire(g, attr, 0, drive, 1)?;
            head = drive;

            // ⚠️ E o `inv_mass` MORRE aqui, senão a banda 5 seria as duas coisas ao
            // mesmo tempo e o par não isolaria uma variável. Um segundo pin de
            // contagem zero devolve a coluna a todos livres.
            //
            // ⚠️ **É DEFESA EM CAMADAS e a mutação prova isso:** deixá-lo ligado
            // (`count = 1`) mantém os seis gates VERDES, porque o `falloff = 0` já
            // tira a peça de TODO par — o `inv_mass` que ela ainda carregaria nunca
            // chega a ser consultado. O nó fica porque é ele que torna o rótulo
            // *"MUTADA"* verdadeiro, não porque a imagem mude.
            let unpin = g.add_node("motion.pin_constraint");
            g.set_pos(unpin, Pos { x: x + 900.0, y });
            g.set_param(unpin, "first", 0.0);
            g.set_param(unpin, "count", 0.0);
            wire(g, head, 0, unpin, 0)?;
            head = unpin;
        }
    }

    let collide = g.add_node("motion.collide");
    g.set_pos(collide, Pos { x: x + 1050.0, y });
    g.set_param(collide, "radius", RADIUS);
    g.set_param(collide, "iterations", SWEEPS);
    g.set_param(collide, "strength", 1.0);
    wire(g, head, 0, collide, 0)?;

    // ⚠️ **A CADEIA TEM DE ACABAR NUM `motion.output`, e não é decoração.** O
    // laço de render **re-resolve os sinks a cada quadro** a partir dos nós de
    // saída (`motion_bridge.rs`: `motion.sinks = output_nodes(&doc.graph)`), então
    // o que esta função devolve serve aos GATES e é descartado pelo app. Uma cena
    // sem Output cozinha certo, passa em toda a suíte e **desenha NADA na tela** —
    // o modo de falha que uma cena de smoke não pode ter, porque ele é
    // indistinguível da feature quebrada.
    let out = g.add_node("motion.output");
    g.set_pos(out, Pos { x: x + 1200.0, y });
    wire(g, collide, 0, out, 0)?;
    Some(out)
}

/// Monta as seis fileiras e devolve os nós de saída, de cima para baixo.
pub(crate) fn build_collide_demo_document(
    doc: &mut MotionDoc,
    registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let kinds = [
        Kind::Plain,
        Kind::SizeRamp,
        Kind::Plain,
        Kind::BoxFalloff,
        Kind::MuteMiddle,
        Kind::PinMiddle,
    ];
    let mut sinks = Vec::with_capacity(BANDS);
    for (row, kind) in kinds.into_iter().enumerate() {
        sinks.push(build_band(g, row, kind)?);
    }
    g.validate(registry).ok()?;
    Some(sinks)
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_collide_tests.rs"]
mod tests;
