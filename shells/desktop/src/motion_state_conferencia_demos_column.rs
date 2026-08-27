//! **A VOLTA COMPLETA DE UMA COLUNA NOMEADA** — a cena `=56`, o grupo P da
//! conferência (folha 06 linha 38, e a metade que faltava da §10.0 do plano).
//!
//! ⚠️ **A cena é um CIRCUITO, não um efeito.** O `motion.drive` escreve um número
//! numa coluna que o artista batiza, o `value.attribute` a lê de volta pelo mesmo
//! nome, e um segundo `motion.drive` a põe no tamanho. Se qualquer elo faltar, a
//! fileira sai toda do mesmo tamanho — e é exactamente isso que o CONTROLE é.
//!
//! **Dois pares:**
//! - **1-2** a volta completa: a de cima é o controle (a cadeia **sem** o
//!   escritor), a de baixo escreve `heat` e lê `heat`.
//! - **3-4** o NOME importa: a de baixo lê um nome que ninguém escreveu, e por
//!   isso volta a ser plana — *uma coluna é uma palavra, e a palavra errada não
//!   é um erro, é o silêncio*.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O vão vertical entre bandas.
const BAND_DY: f32 = 7.0;
/// Peças por fileira.
const COUNT: f32 = 24.0;
/// O nome que o artista escolheu.
const COLUMN: &str = "heat";
/// O nome que ninguém escreveu — a banda 4.
const WRONG: &str = "cold";

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .ok()
}

fn place(g: &mut Graph, head: NodeId, dy: f32, x: f32, y: f32) -> Option<NodeId> {
    let mv = g.add_node("motion.move");
    g.set_pos(mv, Pos { x, y });
    g.set_param(mv, "dx", -6.0);
    g.set_param(mv, "dy", dy);
    wire(g, head, 0, mv, 0)?;
    let out = g.add_node("motion.output");
    g.set_pos(out, Pos { x: x + 220.0, y });
    wire(g, mv, 0, out, 0)?;
    Some(out)
}

/// Uma fileira: grade → [escritor nomeado] → leitor nomeado → tamanho.
///
/// `write` diz se o escritor entra na cadeia; `read` é o nome que o leitor pede.
fn row(g: &mut Graph, write: bool, read: &str, x: f32, y: f32) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    g.set_pos(grid, Pos { x, y });
    g.set_param(grid, "cols", COUNT);
    g.set_param(grid, "rows", 1.0);
    // ⚠️ Era `spacing`, que o `motion.grid` NAO declara — um `set_param` NO-OP
    // silencioso, e a grade shipava com o `gap` de omissao (1,0), DOBRO do autorado.
    // Achado pela `validate` no varredor das 107 (auditoria de 2026-08-27).
    g.set_param(grid, "gap_x", 0.5);
    g.set_param(grid, "gap_y", 0.5);

    // A rampa ao longo do índice — o número que vai viajar na coluna.
    //
    // ⚠️ **Ele tem PORTA DE ENTRADA e o `mode` é `Ramp`, não `Index`.** A minha
    // primeira versão desta cena esqueceu as duas coisas: sem a entrada o nó não
    // sabe quantos elementos há e o campo sai VAZIO, e `mode = 0` é o índice CRU
    // (`0, 1, 2 …`), não a rampa `0..1`. As quatro bandas saíam planas — a
    // assinatura exacta da feature quebrada, sobre um nó cujos gates de unidade
    // passavam todos.
    let ramp = g.add_node("value.instance_field");
    g.set_pos(ramp, Pos { x, y: y + 110.0 });
    g.set_param(ramp, "mode", 1.0); // Ramp → 0..1
    wire(g, grid, 0, ramp, 0)?;

    let mut head = grid;
    if write {
        let w = g.add_node("motion.drive");
        g.set_pos(w, Pos { x: x + 220.0, y });
        g.set_param(w, "channel", 9.0); // Custom
        g.set_param(w, "mode", 1.0); // Set
        g.set_text_param(w, "column", COLUMN);
        wire(g, head, 0, w, 0)?;
        wire(g, ramp, 0, w, 1)?;
        head = w;
    }

    // O LEITOR: a mesma palavra, pelo escape "Custom…" do `value.attribute`.
    let rd = g.add_node("value.attribute");
    g.set_pos(rd, Pos { x: x + 440.0, y });
    g.set_param(rd, "mode", 0.0); // a coluna escalar, crua
    g.set_text_param(rd, "attr", read);
    wire(g, head, 0, rd, 0)?;

    let size = g.add_node("motion.drive");
    g.set_pos(size, Pos { x: x + 660.0, y });
    g.set_param(size, "channel", 3.0); // Size
    g.set_param(size, "mode", 0.0); // Add
    g.set_param(size, "scale", 0.7);
    wire(g, head, 0, size, 0)?;
    wire(g, rd, 0, size, 1)?;
    Some(size)
}

/// Monta a cena. Devolve os sinks, um por banda.
pub(crate) fn build_column_demo_document(
    doc: &mut MotionDoc,
    _registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::with_capacity(4);
    let bands = [
        (false, COLUMN),
        (true, COLUMN),
        (true, COLUMN),
        (true, WRONG),
    ];
    for (row_i, (write, read)) in bands.into_iter().enumerate() {
        let gy = row_i as f32 * 300.0;
        let head = row(g, write, read, 0.0, gy)?;
        sinks.push(place(g, head, (1.5 - row_i as f32) * BAND_DY, 880.0, gy)?);
    }
    Some(sinks)
}

/// Os rótulos das quatro bandas, na ordem em que a cena as monta.
pub(crate) fn band_labels() -> impl Iterator<Item = (usize, &'static str)> {
    [
        "CONTROLE -- ninguem escreve `heat`, e o leitor nao acha nada: fileira PLANA",
        "A VOLTA -- drive(Custom `heat`) escreve, attribute(`heat`) le': a fileira CRESCE",
        "A VOLTA, de novo -- o par de baixo compara o NOME, nao a cadeia",
        "O NOME ERRADO -- le' `cold`, que ninguem escreveu: PLANA outra vez",
    ]
    .into_iter()
    .enumerate()
}

/// Os dois nomes que a cena autora.
pub(crate) fn names() -> (&'static str, &'static str) {
    (COLUMN, WRONG)
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_column_tests.rs"]
mod tests;
