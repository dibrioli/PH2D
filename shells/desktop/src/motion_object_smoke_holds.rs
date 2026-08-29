//! **A cena do RITMO** (`PH2D_MOTION_OBJ_SMOKE=11`) — dois flipbooks lado a lado, a mesma
//! velocidade, e só um deles com uma pose SEGURA.
//!
//! # Por que ela existe: a cena `=9` não podia mostrar isto
//!
//! ⚠️ **Report do Enio, 2026-08-28: *"não há nenhuma animação ou movimento na cena de
//! smoke"*** — e ele estava certo. Eu mandei-o procurar uma mudança de RITMO na fileira
//! sub-UV da cena `=9`, que **nunca põe o `speed`** (o default é `0`): ela mostra quatro
//! cópias congeladas, cada uma num quarto da arte, e é parada de propósito. Uma
//! redistribuição do tempo numa cena **sem tempo nenhum** não muda um pixel.
//!
//! *Uma cura medida numa fixtura que não contém o fenómeno lê-se como inútil* — e aqui nem
//! era a cura, era o smoke.
//!
//! # O que se olha, e o CONTROLE de cada lado
//!
//! | | `Frame Holds` | o que se vê |
//! |---|---|---|
//! | **esquerda** | vazio | os quatro quadrantes trocam ao mesmo ritmo — o metrónomo |
//! | **direita** | `1 1 3 1` | o **terceiro** quadrante fica o triplo do tempo |
//!
//! ⭐ **E as duas voltam ao mesmo quadrante no MESMO instante.** É a leitura que separa
//! *redistribuir* de *abrandar*: os pesos mudam quanto cada quadro dura, nunca quanto a
//! volta inteira demora. Se a direita ficar para trás da esquerda, a lista virou uma segunda
//! resposta a *«quão rápido»*, que é o defeito que o desenho existe para não ter.
//!
//! ⚠️ **A arte é a mesma da cena `=9`** (`sink::spawn_flip_art`): quatro quadrantes de cores
//! distintas. Não é conveniência — é a única fixtura desta casa em que uma célula de sub-UV
//! é **inconfundível**. Os ladrilhos do átlas de demo são cores chapadas, e sobre eles um
//! flipbook inteiro seria invisível (a mesma razão pela qual a `=9` não vive no roteador do
//! `PH2D_GPU_COOK_DEMO`).

use super::sink;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// Quantas células por segundo os DOIS lados percorrem. Com quatro quadrantes, uma volta
/// leva `4 / 1,6 = 2,5 s` — devagar o bastante para o olho seguir qual quadrante está no ecrã.
const SPEED: f32 = 1.6;

/// O ritmo autorado do lado direito: a terceira pose fica o triplo.
pub(crate) const HOLDS: &str = "1 1 3 1";

/// O centro em `x` de cada lado, e o tamanho do carimbo.
const COL_X: f32 = 2.2;
/// A bbox do Flip é `1,6` de mundo; `1,9` põe cada lado com `3,04` de altura e deixa
/// `~1,4` de folga entre as duas colunas (`2 · 2,2 − 3,04`).
const STAMP: f32 = 1.9;

fn wire(g: &mut Graph, a: NodeId, b: NodeId) {
    g.connect(Edge {
        from: (a, 0),
        to: (b, 0),
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

/// Um lado: UMA cópia grande da arte, a percorrer os quatro quadrantes.
///
/// ⚠️ **Uma cópia e não uma fileira**, ao contrário da `=9`: ali o assunto era *qual* célula
/// cada cópia mostra, e a fileira era a resposta. Aqui o assunto é **quando** a célula muda,
/// e várias cópias em fases diferentes (o `stagger`) esconderiam exactamente isso.
fn side(g: &mut Graph, holds: Option<&str>) -> NodeId {
    let y = if holds.is_some() { 140.0 } else { 0.0 };
    let src = node(g, "source.object", &[], y, 0.0);
    g.set_text_param(src, "object", super::OBJECT);
    let gr = node(
        g,
        "motion.grid",
        &[("rows", 1.0), ("cols", 1.0)],
        y + 60.0,
        0.0,
    );
    let dup = node(g, "motion.duplicator", &[], y, 210.0);
    wire(g, src, dup);
    g.connect(Edge {
        from: (gr, 0),
        to: (dup, 1),
        delayed: false,
    })
    .expect("connect");
    flipbook(g, dup, holds, y)
}

/// **A metade que decide o RITMO** — tudo o que vem depois da fonte.
///
/// ⚠️ **Separada da fonte de propósito, e a separação é o gate.** A fonte desta cena é um
/// `source.object`, que só devolve alguma coisa quando o app publicou o objecto — logo um gate
/// que cozinhasse a cena inteira sem o app leria um stream VAZIO e não mediria nada. Assim o
/// gate alimenta esta mesma metade com uma grelha e mede a lei; e a metade que só a cena tem
/// (a fonte, e sobretudo o `speed` **não ser zero**) é afirmada sobre o grafo REAL.
///
/// *Era exactamente esse o defeito da instrução anterior: a cena que eu indiquei tinha o
/// `speed` por omissão em `0`.*
pub(crate) fn flipbook(g: &mut Graph, src: NodeId, holds: Option<&str>, y: f32) -> NodeId {
    let big = node(g, "motion.scale", &[("amount", STAMP)], y, 340.0);
    wire(g, src, big);

    let uv = node(
        g,
        "motion.sub_uv",
        &[("cols", 2.0), ("rows", 2.0), ("speed", SPEED)],
        y,
        460.0,
    );
    if let Some(h) = holds {
        g.set_text_param(uv, ph2d_node_motion_sub_uv::HOLDS_KEY, h);
    }
    wire(g, big, uv);

    let mv = node(
        g,
        "motion.move",
        &[("dx", if holds.is_some() { COL_X } else { -COL_X })],
        y,
        580.0,
    );
    wire(g, uv, mv);
    let out = node(g, "motion.output", &[], y, 720.0);
    wire(g, mv, out);
    out
}

/// O documento da cena — um sink por lado (o da esquerda é o controle).
pub(crate) fn build_holds_graph(g: &mut Graph) -> Vec<NodeId> {
    vec![side(g, None), side(g, Some(HOLDS))]
}

/// Monta a cena e imprime o que olhar.
pub(crate) fn run(gfx: &mut crate::AppGfx) {
    let sinks = build_holds_graph(&mut gfx.motion.doc.graph);
    gfx.motion.sinks.extend(sinks);
    let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
    eprintln!(
        "[cena 11] O RITMO. ⚠️ ESTA CENA MEXE -- de' Play.

  Dois quadrados grandes, lado a lado. Cada um percorre os QUATRO quadrantes coloridos
  da mesma arte, na mesma velocidade.

    ESQUERDA  = o metronomo: os quatro trocam todos ao mesmo ritmo.
    DIREITA   = com «Frame Holds» = {HOLDS}: o TERCEIRO quadrante fica o triplo do tempo.

  A LEITURA QUE IMPORTA: as duas voltam ao mesmo quadrante no MESMO instante. Os pesos
  mudam quanto cada quadro dura -- nunca quanto a volta inteira demora.

  QUER MEXER?
    · Clique no quadrado da direita e edite «Frame Holds»: experimente `3 1 1 1`
      (a primeira pose e' que segura) ou `1 1 1 1` (volta a ser igual a' esquerda).
    · Apague a caixa: a direita tem de ficar IDENTICA a' esquerda.
    · «Cells / Second» muda a velocidade dos dois -- e nos dois por igual.

  DEU ERRADO se:
    · a direita ficar para TRAS da esquerda (os pesos viraram velocidade);
    · nenhum dos dois trocar de quadrante (a cena esta' parada -- de' Play);
    · a direita nao segurar em quadrante nenhum;
    · apagar a caixa nao devolver a direita ao ritmo da esquerda."
    );
}

/// A arte — a MESMA da cena `=9`, e é ela que torna cada célula inconfundível.
pub(crate) fn spawn_art(flip: &mut ph2d_flip::FlipDoc) {
    sink::spawn_flip_art(flip);
}

#[cfg(test)]
#[path = "motion_object_smoke_holds_tests.rs"]
mod tests;
