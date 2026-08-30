//! ⭐⭐⭐ **A CENA `=109` — a tabela vira desenho, das DUAS maneiras que a indústria distingue.**
//!
//! O padrão-ouro parte a pergunta em duas, e a Adobe pagou um formato inteiro para descobrir
//! isso: *"JSON, CSV e TSV só podem conter valores estáticos"* — foi por isso que o `.mgjson`
//! existe. Esta cena mostra as duas lado a lado:
//!
//! 1. **`source.table`** — cada linha é um ELEMENTO. Doze meses, doze pontos, e a altura de
//!    cada um sai da coluna `vendas`. É o *Import CSV* do Blender / *Table Import* do Houdini.
//! 2. **`value.table`** — cada linha é um MOMENTO. Um quadrado só, cujo tamanho segue a coluna
//!    `nivel` ao longo da coluna `tempo` **enquanto o playhead anda**. É o `dataValue()` do AE.
//!
//! ⚠️ **A cena ESCREVE o próprio ficheiro** (`temp_dir()`), como o smoke do Aseprite: provar a
//! feature não pode depender de o Enio ter um CSV à mão, e um caminho que não existe faria as
//! duas colunas desenharem nada — o que se leria como *"a feature não funciona"*.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, NodeId, Pos};

/// Doze meses. ⚠️ A coluna `mes` é TEXTO de propósito — ela prova que o leitor a salta e
/// **nomeia**, em vez de a converter em zeros (a divergência medida contra a regra do Blender).
const CSV: &str = "\
mes,vendas,tempo,nivel
jan,12,0.0,0.2
fev,19,0.5,0.9
mar,7,1.0,0.3
abr,25,1.5,1.0
mai,31,2.0,0.5
jun,18,2.5,0.8
jul,9,3.0,0.2
ago,22,3.5,0.6
set,28,4.0,1.0
out,35,4.5,0.4
nov,15,5.0,0.7
dez,21,5.5,0.3
";

/// O ficheiro que a cena escreve, e que os dois nós apontam.
pub(crate) fn fixture_path() -> std::path::PathBuf {
    std::env::temp_dir().join("ph2d_table_demo.csv")
}

/// ⚠️ Reescreve **sempre**: um ficheiro deixado por uma corrida antiga com outro conteúdo faria
/// a cena ensinar outra coisa, e o Enio leria o defeito como sendo do nó.
///
/// ⚠️⚠️ **E uma escrita falhada é DITA, alto.** A 1.ª redacção era `let _ = fs::write(..)`, e o
/// próprio doc deste módulo diz que um ficheiro ausente *"leria-se como «a feature não
/// funciona»"* — que é exactamente o que aconteceria com o `TMPDIR` só-leitura, o disco cheio,
/// ou o ficheiro já lá de outro dono.
///
/// ⚠️ O caminho é FIXO e três testes do mesmo binário escrevem-no em paralelo. Medido: escrevem
/// os **mesmos bytes**, então a corrida é benigna — e um caminho por-processo custaria ao
/// artista o gesto que a cena ensina (*editar o ficheiro e voltar a escolhê-lo*), porque ele
/// mudaria a cada arranque.
fn write_fixture() -> std::path::PathBuf {
    let p = fixture_path();
    if let Err(e) = std::fs::write(&p, CSV) {
        eprintln!(
            "[cena 109] ⚠️ NAO consegui escrever o ficheiro de exemplo em {}: {e}\n             As duas colunas vao desenhar NADA — o defeito e' este, nao o no'.",
            p.display()
        );
    }
    p
}

fn wire(g: &mut ph2d_nodegraph::graph::Graph, a: NodeId, b: NodeId, port: u16) -> Option<()> {
    g.connect(Edge {
        from: (a, 0),
        to: (b, port),
        delayed: false,
    })
    .ok()
}

pub(crate) fn build_table_demo_document(
    doc: &mut MotionDoc,
    _reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let file = write_fixture();
    let file = file.to_string_lossy().into_owned();
    let g = &mut doc.graph;
    let mut sinks = Vec::new();

    // ---- 1. Uma linha é um ELEMENTO: doze pontos, a altura vinda da coluna `vendas`.
    let src = g.add_node("source.table");
    g.set_pos(src, Pos { x: 80.0, y: 120.0 });
    g.set_text_param(src, ph2d_node_source_table::FILE_KEY, &file);
    g.set_param(src, ph2d_node_source_table::param::SPACING, 0.34);

    let attr = g.add_node("value.attribute");
    g.set_pos(attr, Pos { x: 80.0, y: 320.0 });
    g.set_text_param(attr, ph2d_node_value_attribute::ATTR_KEY, "vendas");

    let drive = g.add_node("motion.drive");
    g.set_pos(drive, Pos { x: 420.0, y: 120.0 });
    // ⚠️ **`1` é Y** — a lista é `X · Y · Rotation · Size · …`, e o índice errado aqui deslocaria
    // os pontos de lado em vez de os levantar, com a fileira ainda a "responder aos dados".
    g.set_param(drive, "channel", 1.0);
    g.set_param(drive, "mode", 0.0);
    // As vendas vão de 7 a 35; este ganho põe a coluna dentro da tela.
    g.set_param(drive, "scale", 0.06);
    wire(g, src, drive, 0)?;
    wire(g, attr, drive, 1)?;
    // O `value.attribute` lê a geometria que vem do `source.table`.
    wire(g, src, attr, 0)?;

    let mv1 = g.add_node("motion.move");
    g.set_pos(mv1, Pos { x: 720.0, y: 120.0 });
    g.set_param(mv1, "dx", -2.2);
    g.set_param(mv1, "dy", -1.0);
    wire(g, drive, mv1, 0)?;
    let out1 = g.add_node("motion.output");
    g.set_pos(
        out1,
        Pos {
            x: 1000.0,
            y: 120.0,
        },
    );
    wire(g, mv1, out1, 0)?;
    sinks.push(out1);

    // ---- 2. Uma linha é um MOMENTO: UM quadrado, o tamanho a seguir a tabela no tempo.
    let grid = g.add_node("motion.grid");
    g.set_pos(grid, Pos { x: 80.0, y: 620.0 });
    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", 1.0);

    let vt = g.add_node("value.table");
    g.set_pos(vt, Pos { x: 420.0, y: 820.0 });
    g.set_text_param(vt, ph2d_node_value_table::FILE_KEY, &file);
    g.set_text_param(vt, ph2d_node_value_table::TIME_KEY, "tempo");
    g.set_text_param(vt, ph2d_node_value_table::VALUE_KEY, "nivel");
    // Repete, senão a demo pára de se mexer aos 5,5 segundos e lê-se como partida.
    g.set_param(vt, ph2d_node_value_table::param::OUTSIDE, 1.0);
    wire(g, grid, vt, 0)?;

    let drive2 = g.add_node("motion.drive");
    g.set_pos(drive2, Pos { x: 720.0, y: 620.0 });
    g.set_param(drive2, "channel", 3.0); // Size
    g.set_param(drive2, "mode", 0.0);
    g.set_param(drive2, "scale", 1.4);
    wire(g, grid, drive2, 0)?;
    wire(g, vt, drive2, 1)?;

    let mv2 = g.add_node("motion.move");
    g.set_pos(
        mv2,
        Pos {
            x: 1000.0,
            y: 620.0,
        },
    );
    g.set_param(mv2, "dx", 2.0);
    g.set_param(mv2, "dy", 0.0);
    wire(g, drive2, mv2, 0)?;
    let out2 = g.add_node("motion.output");
    g.set_pos(
        out2,
        Pos {
            x: 1280.0,
            y: 620.0,
        },
    );
    wire(g, mv2, out2, 0)?;
    sinks.push(out2);

    Some(sinks)
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_table_tests.rs"]
mod tests;
