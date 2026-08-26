//! **A cena do ESTILO DO SINK** (`PH2D_MOTION_OBJ_SMOKE=9`, doc 89 folha 17) — quatro
//! pares, e cada par muda **um** param do `motion.output`.
//!
//! | fileira | esquerda | direita |
//! |---|---|---|
//! | pivô | `Pivot X = 0` — cada cópia gira no PRÓPRIO centro | `Pivot X = 0,5` — o ponto de giro salta para a aresta |
//! | sub-UV | a arte inteira, quatro vezes | **um QUARTO dela** em cada cópia |
//! | filtro | `Linear` — o pedaço ampliado sai borrado | **`Nearest`** — sai em blocos duros |
//! | ordem | `Texture` — os dois materiais REAGRUPAM | **`Stream`** — a ordem das linhas ganha |
//!
//! ⚠️ **Esta cena precisa de DOIS objectos com texturas diferentes**, e é por isso que ela
//! vive aqui e não no roteador de `PH2D_GPU_COOK_DEMO`: os demos daquele roteador
//! amostram **um ladrilho BRANCO opaco** (`init.rs`, o `motion_default_uv`), e sobre um
//! ladrilho chapado o filtro, o sub-UV e a mídia mista são todos **invisíveis** — três
//! fileiras que passariam verdes e mudas.
//!
//! ⚠️⚠️ **O OBJECTO É UM FLIP, e a 1.ª versão desta cena usava uma ESTRELA VECTORIAL —
//! errado, com os quatro pares a sairem idênticos** (smoke do Enio, 2026-08-25).
//!
//! O mecanismo: **desde o [ADR-0154] um `source.object` de VECTOR não assa mais uma tile
//! raster** — ele emite `geometry_id`, e o `lower_to_vector_instances_onto` manda a linha
//! para o passe vectorial. Uma `VectorInstance` carrega `geometry_id / P / size / basis /
//! tint` e **mais nada**: não há `anchor`, não há `sampling`, não há `uv_xform`, não há
//! `sub_order`. Os quatro pares não estavam «não a funcionar» — eles nunca chegavam ao
//! lowering que os lê. (A rota de tile só volta acima de `LOD_COUNT = 16 000` cópias, e
//! esta cena tem cinco.)
//!
//! ⚠️ **E o que me levou a escolher a estrela foi um COMENTÁRIO VELHO**: a mensagem do
//! modo `=2` deste mesmo ficheiro dizia *"a estrela foi ASSADA numa tile pela membrana"*,
//! que era verdade antes do ADR-0154 e deixou de ser — enquanto o modo `=5`, três braços
//! abaixo, já dizia o contrário. *Dois modos do mesmo smoke descreviam comportamentos
//! opostos para a mesma entrada, e o mais novo era o certo.* Corrigido junto com esta wave.
//!
//! ⇒ O objecto é um **Flip**, que a membrana ASSA numa tile a 256 px/unidade
//! (`resolve_drawing_leaf`: *"A Flip child stamps as its baked tile (`texture_id`) — no
//! live path"*). Ele tem **texels a sério**, que é o que faz `Nearest` e `Linear`
//! diferirem, e a arte dele é desenhada AQUI para servir as quatro fileiras: quatro
//! quadrantes de cores distintas (⇒ os quartos do sub-UV são inconfundíveis) e um
//! xadrez fino ao centro (⇒ o pedaço ampliado tem estrutura e uma aresta dura).
//! Os ladrilhos do átlas de demo são cores CHAPADAS: ampliá-los 10× não mostra filtro
//! nenhum, e é por isso que o `Chip` só entra na fileira da ordem.
//!
//! [ADR-0154]: ../../../docs/architecture/decisions/
//!
//! ⚠️ **A ordem só se vê com o stream a ALTERNAR de textura.** Duas cadeias concatenadas
//! dariam `A,A,A,B,B,B` — e aí a ordem das linhas e o agrupamento por textura são a MESMA
//! coisa, e o par sairia igual dos dois lados. Por isso as duas grelhas nascem
//! **entrelaçadas em `x`** e passam por um `motion.sort(key = X)`: a fileira fica
//! `A,B,A,B,A,B`, as cópias sobrepõem-se, e aí `Texture` empilha um material inteiro por
//! cima do outro enquanto `Stream` faz a escada da esquerda para a direita.

use super::{DEMO_TILE_KEY, OBJECT, flip_rect};
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};
use ph2d_render::Sprite;

/// O nome do SEGUNDO objecto — o que dá a fileira da ordem a sua outra textura.
pub(crate) const CHIP: &str = "Chip";

/// O `channel` do `motion.oscillator` que escreve `rot` (a escada de `channel_column`).
const OSC_CHANNEL_ROT: f32 = 2.0;

/// O centro em `x` de cada coluna, e o `y` de cada fileira.
const COL_X: f32 = 3.5;
const ROW_Y: [f32; 4] = [3.3, 1.3, -1.0, -3.4];

/// ⚠️ **As medidas abaixo são DERIVADAS do tamanho do carimbo, não escolhidas.** A bbox do
/// Flip desta cena é `2 · HALF = 1,6` unidades de mundo, e é ela que o `motion.duplicator`
/// carimba — bem maior que a estrela da 1.ª versão. Sem as reduzir, duas colunas a `±3,5`
/// (7 de distância) sobrepor-se-iam e a cena leria como uma papa.
///
/// `STAMP` põe uma cópia a `1,6 · 0,45 = 0,72`; com `STEP = 0,95`, cinco cópias medem
/// `4 · 0,95 + 0,72 = 4,52` — folga de `2,5` entre colunas.
const STAMP: f32 = 0.45;
const STEP: f32 = 0.95;
/// A ampliação da fileira do filtro: `1,6 · 1,5 = 2,4` de altura, que cabe entre as
/// fileiras vizinhas (`ROW_Y` separa-as por `2,0`–`2,4`) e ainda é **3,3×** um carimbo
/// normal — o bastante para o bloco do `Nearest` se ver.
const ZOOM: f32 = 1.5;

fn wire(g: &mut Graph, a: NodeId, ap: u16, b: NodeId, bp: u16) {
    g.connect(Edge {
        from: (a, ap),
        to: (b, bp),
        delayed: false,
    })
    .expect("connect");
}

fn node(g: &mut Graph, kind: &str, ps: &[(&str, f32)], y: f32, x: f32) -> NodeId {
    let n = g.add_node(kind);
    g.set_pos(n, Pos { x, y });
    for (k, v) in ps {
        g.set_param(n, *k, *v);
    }
    n
}

/// `source.object(name) → duplicator ← grid` — a metade comum de toda fileira.
/// Devolve o `duplicator` (a cabeça da cadeia).
fn stamped(g: &mut Graph, name: &str, grid: &[(&str, f32)], y: f32) -> NodeId {
    let src = node(g, "source.object", &[], y, 0.0);
    g.set_text_param(src, "object", name);
    let gr = node(g, "motion.grid", grid, y + 70.0, 0.0);
    let dup = node(g, "motion.duplicator", &[], y, 210.0);
    wire(g, src, 0, dup, 0);
    wire(g, gr, 0, dup, 1);
    dup
}

/// Põe a cadeia no seu quadrante e termina-a num `motion.output` com o estilo dado.
///
/// ⚠️ **A colocação entra ANTES do sink e DEPOIS de tudo o que é campo** — a lei que a
/// cena `=73` pagou: um deslocamento multiplicado por uma máscara estica a fileira por
/// cima das vizinhas. Aqui não há máscara, e a ordem é a mesma de propósito.
fn sink(g: &mut Graph, head: NodeId, row: usize, right: bool, style: &[(&str, f32)]) -> NodeId {
    let y = row as f32 * 240.0;
    let mv = node(
        g,
        "motion.move",
        &[
            ("dx", if right { COL_X } else { -COL_X }),
            ("dy", ROW_Y[row]),
        ],
        y,
        420.0,
    );
    wire(g, head, 0, mv, 0);
    let out = node(g, "motion.output", style, y, 600.0);
    wire(g, mv, 0, out, 0);
    out
}

/// **Fileira 1 — o PIVÔ.** Cinco cópias, cada uma com a sua rotação (estática: o
/// `frequency = 0` faz a fase ser só o escalonamento por índice, então o relógio não
/// mexe nada). Com o pivô ao centro elas giram no lugar; com ele na aresta, o ponto de
/// giro salta e a fileira abre-se em leque.
fn row_pivot(g: &mut Graph, sinks: &mut Vec<NodeId>) {
    for right in [false, true] {
        let dup = stamped(
            g,
            OBJECT,
            &[("rows", 1.0), ("cols", 5.0), ("gap_x", STEP)],
            0.0,
        );
        let small = node(g, "motion.scale", &[("amount", STAMP)], 0.0, 280.0);
        wire(g, dup, 0, small, 0);
        let dup = small;
        let osc = node(
            g,
            "motion.oscillator",
            &[
                ("channel", OSC_CHANNEL_ROT),
                ("amplitude", 70.0),
                ("frequency", 0.0),
                ("phase_stagger", 0.21),
            ],
            0.0,
            320.0,
        );
        wire(g, dup, 0, osc, 0);
        let style: &[(&str, f32)] = if right { &[("pivot_x", 0.5)] } else { &[] };
        sinks.push(sink(g, osc, 0, right, style));
    }
}

/// **Fileira 2 — o SUB-UV.** Quatro cópias; à direita cada uma mostra um quarto da arte
/// (`stagger = 1` ⇒ a célula anda uma por elemento).
fn row_sub_uv(g: &mut Graph, sinks: &mut Vec<NodeId>) {
    for right in [false, true] {
        let dup = stamped(
            g,
            OBJECT,
            &[("rows", 1.0), ("cols", 4.0), ("gap_x", STEP)],
            240.0,
        );
        let small = node(g, "motion.scale", &[("amount", STAMP)], 240.0, 280.0);
        wire(g, dup, 0, small, 0);
        let dup = small;
        let head = if right {
            let uv = node(
                g,
                "motion.sub_uv",
                &[("cols", 2.0), ("rows", 2.0), ("stagger", 1.0)],
                240.0,
                320.0,
            );
            wire(g, dup, 0, uv, 0);
            uv
        } else {
            dup
        };
        sinks.push(sink(g, head, 1, right, &[]));
    }
}

/// **Fileira 3 — o FILTRO.** UMA cópia grande de um pedaço pequeno da arte: a ampliação
/// é o que torna o filtro visível, e o sub-UV é o que fabrica a ampliação.
///
/// ⚠️ **A célula é a mesma dos dois lados** — o que muda é só o sampler. Uma célula
/// diferente faria o par mostrar duas artes e ninguém saberia a que atribuir a diferença.
fn row_filter(g: &mut Graph, sinks: &mut Vec<NodeId>) {
    for right in [false, true] {
        let dup = stamped(g, OBJECT, &[("rows", 1.0), ("cols", 1.0)], 480.0);
        let uv = node(
            g,
            "motion.sub_uv",
            &[("cols", 10.0), ("rows", 10.0), ("cell", 44.0)],
            480.0,
            320.0,
        );
        wire(g, dup, 0, uv, 0);
        let big = node(g, "motion.scale", &[("amount", ZOOM)], 480.0, 380.0);
        wire(g, uv, 0, big, 0);
        // 1 = Nearest · 2 = Linear (a escada dos tags de `FilterMode`).
        sinks.push(sink(
            g,
            big,
            2,
            right,
            &[("filter", if right { 1.0 } else { 2.0 })],
        ));
    }
}

/// **Fileira 4 — a ORDEM.** Duas grelhas ENTRELAÇADAS em `x`, juntadas e ordenadas por
/// `x`, de modo que o stream alterne de textura cópia a cópia.
fn row_sort(g: &mut Graph, sinks: &mut Vec<NodeId>) {
    for right in [false, true] {
        // As duas grelhas partilham o passo e diferem por meio passo — é isso que as
        // entrelaça depois do `sort`.
        // ⚠️ O passo é MENOR que o carimbo (`0,55` contra `0,72`), de propósito: sem
        // sobreposição não há «quem fica à frente», e as duas ordens desenhariam igual.
        let a = stamped(
            g,
            OBJECT,
            &[("rows", 1.0), ("cols", 3.0), ("gap_x", 1.1)],
            720.0,
        );
        let sa = node(g, "motion.scale", &[("amount", STAMP)], 720.0, 280.0);
        wire(g, a, 0, sa, 0);
        let a = sa;
        let b = stamped(
            g,
            CHIP,
            &[("rows", 1.0), ("cols", 3.0), ("gap_x", 1.1)],
            790.0,
        );
        // O `Chip` nasce a `0,85` e o Flip a `1,6`: os dois passam por uma escala que os
        // deixa do MESMO tamanho, senão a fileira leria como *dois tamanhos* em vez de
        // *dois materiais*, e o olho atribuiria a diferença à coisa errada.
        let sb = node(g, "motion.scale", &[("amount", 0.85)], 790.0, 280.0);
        wire(g, b, 0, sb, 0);
        let off = node(g, "motion.move", &[("dx", 0.55)], 790.0, 320.0);
        wire(g, sb, 0, off, 0);
        let mix = node(g, "motion.combine", &[], 720.0, 360.0);
        wire(g, a, 0, mix, 0);
        wire(g, off, 0, mix, 1);
        // `key = 1` é o X (a escada do `motion.sort`).
        let srt = node(g, "motion.sort", &[("key", 1.0)], 720.0, 400.0);
        wire(g, mix, 0, srt, 0);
        sinks.push(sink(
            g,
            srt,
            3,
            right,
            &[("sort", if right { 1.0 } else { 0.0 })],
        ));
    }
}

/// **A ARTE do objecto desta cena** — um Flip de duas camadas, desenhado para servir as
/// quatro fileiras (e não reaproveitado do `=3`, cujo retângulo azul com um quadrado
/// laranja ao centro é quase simétrico: os quatro quartos sairiam parecidos).
///
/// - **BG**: quatro quadrantes de cores distintas ⇒ cada quarto do sub-UV é outra cor.
/// - **FG**: um xadrez fino ao centro ⇒ o pedaço que a fileira do filtro amplia tem
///   estrutura e uma aresta dura, que é o que separa `Nearest` de `Linear`.
///
/// ⚠️ **A bbox é QUADRADA** (`±HALF`): o sub-UV corta em fracções do rectângulo, e uma
/// bbox oblonga daria quartos oblongos — legível, mas o par pareceria mudar de forma além
/// de mudar de conteúdo, e o olho atribuiria a diferença à coisa errada.
const HALF: f32 = 0.8;
/// O lado de uma casa do xadrez, em unidades de mundo. A 256 px/unidade isto dá ~26 px
/// na tile assada — grosso o bastante para a ampliação da fileira 3 mostrar o bloco.
const CHECK: f32 = 0.1;

pub(super) fn spawn_flip_art(flip: &mut ph2d_flip::FlipDoc) {
    use ph2d_flip::{Hold, KeyKind, Rgba};
    let oid = flip.push_object(OBJECT);
    let obj = flip.object_mut(oid).expect("objeto Flip recem-criado");
    obj.fps = 12.0;
    let bg = obj.add_layer("BG");
    if let Some(d) = obj.insert_frame(bg, 0, Hold::Implicit, KeyKind::Keyframe) {
        let quads = [
            ([-HALF, 0.0], [0.0, HALF], Rgba::new(0.90, 0.35, 0.30, 1.0)),
            ([0.0, 0.0], [HALF, HALF], Rgba::new(0.35, 0.70, 0.95, 1.0)),
            ([-HALF, -HALF], [0.0, 0.0], Rgba::new(0.45, 0.80, 0.45, 1.0)),
            ([0.0, -HALF], [HALF, 0.0], Rgba::new(0.95, 0.80, 0.25, 1.0)),
        ];
        let dr = obj.drawing_mut(d).expect("desenho BG");
        for (min, max, c) in quads {
            dr.strokes.push(flip_rect(
                Vec2::new(min[0], min[1]),
                Vec2::new(max[0], max[1]),
                c,
            ));
        }
    }
    let fg = obj.add_layer("FG");
    if let Some(d) = obj.insert_frame(fg, 0, Hold::Implicit, KeyKind::Keyframe) {
        let dr = obj.drawing_mut(d).expect("desenho FG");
        // Um xadrez 6x6 de lado `CHECK` centrado na origem — só as casas escuras.
        for r in 0..6 {
            for c in 0..6 {
                if (r + c) % 2 != 0 {
                    continue;
                }
                let x = (c as f32 - 3.0) * CHECK;
                let y = (r as f32 - 3.0) * CHECK;
                dr.strokes.push(flip_rect(
                    Vec2::new(x, y),
                    Vec2::new(x + CHECK, y + CHECK),
                    ph2d_flip::Rgba::new(0.12, 0.10, 0.14, 1.0),
                ));
            }
        }
    }
}

/// O SEGUNDO objecto da cena — um sprite de ladrilho chapado, que é a OUTRA textura.
///
/// ⚠️ **Ele é chapado de propósito e só entra na fileira da ORDEM**: ali o que se mede é
/// *que material está à frente*, e uma cor sólida é a leitura mais rápida disso.
pub(super) fn spawn_chip(sim: &mut ph2d_ecs::SimWorld) {
    sim.world_mut().spawn((
        Transform::from_translation(Vec2::new(0.0, 0.0)),
        Sprite::atlas(DEMO_TILE_KEY, [0.85, 0.85], [1.0, 1.0, 1.0, 1.0]),
        Name::new(CHIP),
    ));
}

/// Monta as oito cadeias e devolve os oito sinks — **função PURA sobre o grafo**, para o
/// gate a poder montar sem `AppGfx` nenhum.
pub(crate) fn build_sink_style_graph(g: &mut Graph) -> Vec<NodeId> {
    let mut sinks = Vec::with_capacity(8);
    row_pivot(g, &mut sinks);
    row_sub_uv(g, &mut sinks);
    row_filter(g, &mut sinks);
    row_sort(g, &mut sinks);
    sinks
}

/// A cena do modo `=9`, montada no frame 6 (a entidade da estrela nasce no `sync`).
pub(super) fn run(gfx: &mut crate::AppGfx) {
    let sinks = build_sink_style_graph(&mut gfx.motion.doc.graph);
    gfx.motion.sinks.extend(sinks);
    let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
    eprintln!(
        "[motion.obj smoke =9] O SINK ganhou o ESTILO DE DESENHO (doc 89 folha 17).
  Quatro fileiras, cada uma com um par -- ESQUERDA = como era, DIREITA = o param novo.
  1 PIVO   : a esquerda cada estrela gira no proprio centro; a direita o ponto de
             giro esta' na aresta, e a fileira abre-se em leque.
  2 SUB-UV : a esquerda quatro estrelas inteiras; a direita cada copia mostra um
             QUARTO da arte (2x2).
  3 FILTRO : o MESMO pedacinho ampliado -- a esquerda `Linear` (borrado), a direita
             `Nearest` (blocos duros). E' o modo de pixel-art.
  4 ORDEM  : estrelas e quadrados alternados e sobrepostos -- a esquerda `Texture`
             (um material inteiro por cima do outro), a direita `Stream` (a escada
             da esquerda para a direita, que e' a ordem das linhas).
  > clique num no' Output e mexa em Pivot X / Filter / Sort.
  (i) No MEIO da tela ficam os dois objectos que a cena carimba (a arte de quatro
      cores e o quadrado laranja). Eles sao a FONTE, nao uma quinta fileira.
  (!) DEU ERRADO se algum par sair igual dos dois lados, ou se alguma fileira sumir."
    );
}

#[cfg(test)]
#[path = "motion_object_smoke_sink_tests.rs"]
mod tests;
