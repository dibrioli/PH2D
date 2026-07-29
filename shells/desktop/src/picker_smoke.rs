//! **A cena mínima para JULGAR o picker de colunas do `value.attribute`**
//! (`PH2D_PICKER_SMOKE=1`).
//!
//! Um teste que se auto-verifica: uma grade ESTÁTICA de pontos (sem movimento, para
//! o tamanho de cada um ser legível), o `value.attribute` já SELECIONADO e já em
//! **Custom** (o `attr` nasce vazio, então nenhum canal curado casa e o picker abre
//! na aba Custom), mostrando os chips **"From stream"** das colunas que a grade de
//! fato carrega: **Count** e **Index**.
//!
//! ```text
//! grid → scale → tint → value.attribute → drive(Size, Add) → output
//! ```
//!
//! O `drive` está em modo **Add** de propósito: com o `attr` vazio o campo é zero,
//! então os pontos abrem TODOS do mesmo tamanho (a grade é visível). Clicar um chip
//! escreve a coluna real e o tamanho passa a variar por ela — e o contraste entre os
//! dois chips é o que torna o teste inequívoco:
//!
//! - **Index** vale `0, 1, 2, …` por ponto ⇒ os pontos crescem em RAMPA (pequenos no
//!   começo, grandes no fim). Se você vê uma rampa, o picker leu a coluna certa.
//! - **Count** vale o MESMO número (o total) em todo ponto ⇒ todos os pontos saltam
//!   para o MESMO tamanho maior, uniforme. Coluna diferente, resposta diferente.
//!
//! Se os chips estivessem "mortos" (o bug que você pegou), clicar não faria nada e a
//! grade ficaria uniforme e pequena para sempre.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// Ligado? Lido UMA vez (o prólogo do frame não paga um `getenv` por quadro).
fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("PH2D_PICKER_SMOKE").is_ok_and(|v| v != "0"))
}

static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// Build the grid → attribute → drive(Size, Add) scene. Returns the `Attribute`
/// node so the caller can pre-select it (the picker only shows for a selected node).
fn build_picker_scene(g: &mut Graph) -> NodeId {
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 6.0);
    g.set_param(grid, "cols", 6.0);
    g.set_param(grid, "gap_x", 1.4);
    g.set_param(grid, "gap_y", 1.4);

    // The grid carries no `size` column, so give the dots a small base size.
    let scale = g.add_node("motion.scale");
    g.set_param(scale, "amount", 0.16);

    let tint = g.add_node("motion.tint");
    g.set_param(tint, "mode", 0.0); // Solid
    g.set_param(tint, "r", 1.0);
    g.set_param(tint, "g", 0.72);
    g.set_param(tint, "b", 0.30);
    g.set_param(tint, "a", 1.0);

    // The node in FOCUS: `attr` empty on purpose -> the picker opens on Custom, so
    // the "From stream" chips are on screen from the first frame.
    let attr = g.add_node("value.attribute");
    g.set_param(attr, "mode", 0.0);

    let drive = g.add_node("motion.drive");
    g.set_param(drive, "channel", 3.0); // Size
    g.set_param(drive, "scale", 0.014); // size = base + 0.014 * value
    g.set_param(drive, "mode", 0.0); // Add (empty attr -> +0 -> uniform base)

    let out = g.add_node("motion.output");

    for (i, n) in [grid, scale, tint, attr, drive, out]
        .into_iter()
        .enumerate()
    {
        g.set_pos(
            n,
            Pos {
                x: 60.0 + i as f32 * 190.0,
                y: 120.0,
            },
        );
    }
    for (from, fp, to, tp) in [
        (grid, 0, scale, 0),
        (scale, 0, tint, 0),
        // Fan-out: the tinted stream feeds the attribute (which reads a column) AND
        // the drive (whose size the value modulates).
        (tint, 0, attr, 0),
        (tint, 0, drive, 0),
        (attr, 0, drive, 1),
        (drive, 0, out, 0),
    ] {
        g.connect(Edge {
            from: (from, fp),
            to: (to, tp),
            delayed: false,
        })
        .expect("picker-scene edge");
    }
    attr
}

/// The whole document + the `Attribute` node to select. Validated.
fn build_picker_doc(reg: &NodeRegistry) -> (MotionDoc, NodeId) {
    let mut doc = MotionDoc::new();
    let attr = build_picker_scene(&mut doc.graph);
    doc.graph
        .validate(reg)
        .expect("picker document is well-typed");
    (doc, attr)
}

impl crate::App {
    /// Roda no prólogo do frame, ao lado dos outros smokes. No-op sem a env.
    pub(crate) fn picker_smoke(&mut self) {
        use std::sync::atomic::Ordering;
        if !on() || self.gfx.is_none() || FRAME.fetch_add(1, Ordering::Relaxed) != 4 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let (doc, attr) = build_picker_doc(&gfx.motion.registry);
        gfx.motion.doc = doc;
        gfx.motion.sinks.clear();
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
        // Select the Attribute so its picker is on screen immediately.
        ph2d_panel_motion_graph::request_graph_selection(vec![attr.0]);
        eprintln!(
            "[picker smoke] Uma grade 6x6 de pontos IGUAIS. O no 'Attribute' ja esta \
             selecionado e o painel de params (a direita) mostra a aba 'Custom' com os \
             chips 'From stream': Count e Index.\n  \
             TESTE: clique o chip 'Index' -> os pontos passam a CRESCER em rampa \
             (pequenos no comeco, grandes no fim). Depois clique 'Count' -> todos saltam \
             para o MESMO tamanho maior (uniforme). Se os dois chips mudam a grade assim, \
             o picker le colunas VIVAS e funciona. (Se nada muda, os chips estao mortos.)"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::attr::Column;
    use ph2d_nodegraph::cook::Cook;

    fn registry() -> NodeRegistry {
        let mut reg = NodeRegistry::new();
        ph2d_node_registry_init::register_all_nodes(&mut reg).expect("node registry builds");
        reg
    }

    /// The scene is well-typed and the stream feeding the Attribute really carries
    /// the scalar columns the picker will offer as chips (`Index`, `Count`) — cooked
    /// through the real registry, so a broken upstream would show empty chips (which
    /// is exactly what the artist would report as "nothing to click").
    #[test]
    fn the_upstream_carries_the_columns_the_picker_will_offer() {
        let reg = registry();
        let mut g = Graph::new();
        let attr = build_picker_scene(&mut g);
        g.validate(&reg).expect("well-typed");

        // Cook the node feeding the Attribute's input (its single upstream edge).
        let src = g
            .edges()
            .iter()
            .find(|e| e.to == (attr, 0))
            .map(|e| e.from.0)
            .expect("the Attribute has an input");
        let mut cook = Cook::new();
        let out = cook.cook(&g, &reg, src, 0.0).expect("upstream cooks");
        let stream = out[0].as_stream();
        let scalars: Vec<&str> = stream
            .columns()
            .filter(|(_, c)| matches!(c, Column::Scalar(_)))
            .map(|(n, _)| n.as_str())
            .collect();
        assert!(
            scalars.contains(&"Index"),
            "picker offers Index: {scalars:?}"
        );
        assert!(
            scalars.contains(&"Count"),
            "picker offers Count: {scalars:?}"
        );
    }
}
