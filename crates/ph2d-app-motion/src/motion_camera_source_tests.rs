//! ⭐⭐⭐ **A VISTA CHEGA AO TAMANHO** — o gate que prova que o `source.camera` SERVE (ciclo 8, W3 —
//! [doc 113](../../../docs/Motion%20Nodes/113_ciclo_8_fontes_e_dados.md) §5).
//!
//! ⚠️ **Um nó que publica e ninguém consegue usar é um controlo morto com cara de feature**
//! (`CLAUDE.md` §5.0: *o painel escreve onde · quem lê · o leitor DECIDE?*). A folha 14 da
//! conferência nomeou quatro coisas que a ausência da câmara tornava inexprimíveis, e a primeira
//! delas — **escalar com o zoom** — é uma CADEIA, não um param:
//!
//! ```text
//!   source.camera → value.attribute(zoom) → motion.drive(Size, Divide, Scale = 1/px)
//!                                      grid ↗                                ↘ output
//! ```
//!
//! ⚠️ **Sem nó de matemática, e a razão é uma LEI e não elegância:** o `motion.drive` já sabe
//! dividir, e com ele o caso «ainda não há vista» resolve-se sozinho — *um fio sem valor não
//! escreve* (a cura da W3, kernel e CPU). A cadeia com um `value.math` no meio faz a mesma
//! imagem, mas ali um operando AUSENTE lê a identidade `0` (lei declarada do nó), e `px ÷ 0`
//! entrega um zero que o `Set` escreve: as peças SOMEM. *A composição mais curta é a que não tem
//! esse estado.*
//!
//! ⚠️ **A régua é o produto `tamanho × zoom`**, que é o tamanho em PIXELS de ecrã: ele tem de ficar
//! o mesmo quando a câmara se afasta. Medir só «o tamanho mudou» passaria com qualquer fio ligado.

use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::graph::{Edge, NodeId};

/// Quantos pixels de ecrã a peça tem de medir, esteja a câmara onde estiver.
const PIXELS: f32 = 40.0;

fn liga(m: &mut MotionState, de: NodeId, dp: u16, para: NodeId, pp: u16) {
    m.doc
        .graph
        .connect(Edge {
            from: (de, dp),
            to: (para, pp),
            delayed: false,
        })
        .expect("as portas encaixam");
}

/// Monta a cadeia e devolve `(estado, sink)`.
fn cena() -> (MotionState, NodeId) {
    let mut m = MotionState::new();
    let cam = m.doc.graph.add_node("source.camera".to_string());
    let attr = m.doc.graph.add_node("value.attribute".to_string());
    m.doc.graph.set_text_param(
        attr,
        ph2d_node_value_attribute::ATTR_KEY,
        ph2d_node_source_camera::ZOOM_COLUMN,
    );
    let grid = m.doc.graph.add_node("motion.grid".to_string());
    m.doc.graph.set_param(grid, "rows", 2.0);
    m.doc.graph.set_param(grid, "cols", 2.0);
    let drive = m.doc.graph.add_node("motion.drive".to_string());
    // `Size` é o índice `3` da escada do `motion.drive` (X · Y · Rotation · Size …) e `Divide` o
    // `4` da do modo (Add · Set · Multiply · Subtract · Divide …). O `Scale` multiplica o valor
    // ANTES da operação, então `size = 1 ÷ (zoom · 1/px)` = `px ÷ zoom`.
    m.doc.graph.set_param(drive, "channel", 3.0);
    m.doc.graph.set_param(drive, "mode", 4.0);
    m.doc.graph.set_param(drive, "scale", 1.0 / PIXELS);
    let out = m.doc.graph.add_node("motion.output".to_string());
    liga(&mut m, cam, 0, attr, 0);
    liga(&mut m, grid, 0, drive, 0);
    liga(&mut m, attr, 0, drive, 1);
    liga(&mut m, drive, 0, out, 0);
    (m, out)
}

/// Publica uma vista com este `zoom` (px por unidade) e devolve o `size.x` da primeira peça.
fn tamanho_com_zoom(m: &mut MotionState, sink: NodeId, zoom: f32) -> f32 {
    m.pump.cook.set_external(
        ph2d_nodegraph::external::CAMERA.to_string(),
        Stream::new(1)
            .with("P", Column::Vec2(vec![[0.0, 0.0]]))
            .with("size", Column::Vec2(vec![[16.0, 9.0]]))
            .with(
                ph2d_node_source_camera::ZOOM_COLUMN,
                Column::Scalar(vec![zoom]),
            ),
    );
    let out = m
        .pump
        .cook
        .cook(&m.doc.graph, &m.registry, sink, 0.0)
        .expect("coze");
    match out[0].as_stream().get("size") {
        Some(Column::Vec2(v)) => v[0][0],
        outra => panic!("a cadeia escreve `size`: {outra:?}"),
    }
}

/// ⭐⭐⭐ **A PEÇA MEDE OS MESMOS PIXELS EM QUALQUER ZOOM.**
#[test]
fn a_piece_keeps_its_screen_size_when_the_camera_zooms() {
    let (mut m, sink) = cena();
    let perto = tamanho_com_zoom(&mut m, sink, 100.0);
    let longe = tamanho_com_zoom(&mut m, sink, 25.0);
    assert!(
        (perto * 100.0 - PIXELS).abs() < 1e-3 && (longe * 25.0 - PIXELS).abs() < 1e-3,
        "o tamanho em PIXELS tem de ser {PIXELS} nos dois: {} e {}",
        perto * 100.0,
        longe * 25.0
    );
    // ⚠️ E o CONTROLO: o tamanho em MUNDO mudou mesmo — senão o gate acima passaria com a cadeia
    // desligada e uma peça de tamanho constante.
    assert!(
        (longe / perto - 4.0).abs() < 1e-2,
        "afastar a camara 4x tem de quadruplicar a peca no mundo: {perto} -> {longe}"
    );
}

/// ⭐⭐⭐ **SEM VISTA PUBLICADA a cadeia não escreve tamanho nenhum** — a peça fica com o dela, em
/// vez de desaparecer.
///
/// ⛔⛔ **Esta metade REPROVOU quando foi escrita, e o defeito era do `motion.drive`:** um campo de
/// valor vazio resolvia-se pela identidade `0`, e o modo de escrita punha `Size = 0` — a arte a
/// sumir sem erro nenhum. A cura (*um fio sem valor não escreve*) vale nos dois motores e está
/// provada por mutação dos dois lados (doc 113 §5).
#[test]
fn without_a_published_view_the_chain_leaves_the_size_alone() {
    let (mut m, sink) = cena();
    let out = m
        .pump
        .cook
        .cook(&m.doc.graph, &m.registry, sink, 0.0)
        .expect("coze");
    let s = out[0].as_stream();
    assert_eq!(s.count(), 4, "a grade continua a emitir as quatro pecas");
    match s.get("size") {
        None => {}
        Some(Column::Vec2(v)) => assert!(
            v.iter().all(|z| (z[0] - 1.0).abs() < 1e-6),
            "sem vista o `size` fica na identidade: {v:?}"
        ),
        outra => panic!("`size` devia ser Vec2: {outra:?}"),
    }
}
