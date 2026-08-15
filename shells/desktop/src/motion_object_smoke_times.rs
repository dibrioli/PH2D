//! **A cena das DUAS FASES** (`PH2D_MOTION_OBJ_SMOKE=7`) — o mesmo objeto Flip
//! trazido para o grafo duas vezes, em tempos diferentes (doc 89, folha 14).
//!
//! Irmão de `motion_object_smoke` pelo teto de LOC (HR-18), cortado por ASSUNTO: os
//! outros modos perguntam *o objeto chega ao grafo?*, este pergunta *QUANDO*.
//!
//! ⚠️ E ele traz a própria fixture — um Flip **animado**. Os outros modos montam um
//! objeto de UM desenho, e sobre um desenho só um offset é indistinguível de nenhum
//! offset: a cena dizia "funciona" sobre um param inerte.

use super::flip_rect;
use ph2d_core::Vec2;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// **Um Flip ANIMADO** — quatro desenhos, o quadrado laranja num lugar diferente em
/// cada um. É a fixture que o `time_offset` precisa e que os outros modos não têm:
/// com UM desenho só, um offset é indistinguível de nenhum offset, e a cena diria
/// "funciona" sobre um param inerte.
///
/// A 12 fps as chaves em `0/3/6/9` dão um desenho a cada `0,25 s` — então o offset da
/// cena (`0,25`) pousa **exatamente** no desenho seguinte, e o que o artista vê é uma
/// grade adiantada em um passo, não um borrão a meio caminho.
pub(crate) fn spawn_flip_walk_named(flip: &mut ph2d_flip::FlipDoc, name: &str) {
    use ph2d_flip::{Hold, KeyKind, Rgba};
    let oid = flip.push_object(name);
    let obj = flip.object_mut(oid).expect("objeto Flip recém-criado");
    obj.fps = 12.0;
    // BG: o campo azul, o MESMO em todo quadro — é ele que torna o movimento do
    // quadrado legível (sem um fundo fixo, duas grades deslocadas leem como duas
    // grades em lugares diferentes).
    let bg = obj.add_layer("BG");
    if let Some(d) = obj.insert_frame(bg, 0, Hold::Implicit, KeyKind::Keyframe) {
        obj.drawing_mut(d)
            .expect("desenho BG")
            .strokes
            .push(flip_rect(
                Vec2::new(-0.9, -0.6),
                Vec2::new(0.9, 0.6),
                Rgba::new(0.2, 0.5, 0.95, 1.0),
            ));
    }
    // FG: quatro chaves, o quadrado varrendo da esquerda para a direita.
    let fg = obj.add_layer("FG");
    for (i, key) in [0, 3, 6, 9].into_iter().enumerate() {
        let x = -0.55 + (i as f32) * 0.37;
        if let Some(d) = obj.insert_frame(fg, key, Hold::Implicit, KeyKind::Keyframe) {
            obj.drawing_mut(d)
                .expect("desenho FG")
                .strokes
                .push(flip_rect(
                    Vec2::new(x - 0.16, -0.3),
                    Vec2::new(x + 0.16, 0.3),
                    Rgba::new(0.98, 0.7, 0.15, 1.0),
                ));
        }
    }
}

/// **Duas cópias do MESMO objeto, em tempos diferentes** — a cena do `time_offset`.
///
/// Duas cadeias `source.object → duplicator ← grid → move → output`, idênticas em
/// tudo menos num número: a da esquerda em `time_offset = 0` (o quadro atual, o mundo
/// de sempre) e a da direita em `0,25 s`. A 12 fps isso é **três desenhos à frente**.
///
/// ⚠️ O `motion.move` existe só para as duas grades não se sobreporem — ele desloca no
/// ESPAÇO, que é justamente a coisa que o artista já sabia fazer; o que a cena está
/// mostrando é o deslocamento no TEMPO, e é por isso que as duas grades têm de exibir
/// a MESMA arte em fases diferentes.
pub(super) fn build_two_times_graph(graph: &mut Graph, name: &str) -> Vec<NodeId> {
    let wire = |g: &mut Graph, a: NodeId, ap: u16, b: NodeId, bp: u16| {
        g.connect(Edge {
            from: (a, ap),
            to: (b, bp),
            delayed: false,
        })
        .expect("connect");
    };
    let mut sinks = Vec::new();
    for (k, offset) in [(0usize, 0.0_f32), (1, 0.25)] {
        let src = graph.add_node("source.object");
        let grid = graph.add_node("motion.grid");
        let dup = graph.add_node("motion.duplicator");
        let mv = graph.add_node("motion.move");
        let out = graph.add_node("motion.output");
        let col = (k as f32) * 520.0;
        let row = -260.0;
        graph.set_pos(src, Pos { x: col, y: row });
        graph.set_pos(
            grid,
            Pos {
                x: col,
                y: row + 120.0,
            },
        );
        graph.set_pos(
            dup,
            Pos {
                x: col + 150.0,
                y: row + 60.0,
            },
        );
        graph.set_pos(
            mv,
            Pos {
                x: col + 290.0,
                y: row + 60.0,
            },
        );
        graph.set_pos(
            out,
            Pos {
                x: col + 400.0,
                y: row + 60.0,
            },
        );
        wire(graph, src, 0, dup, 0);
        wire(graph, grid, 0, dup, 1);
        wire(graph, dup, 0, mv, 0);
        wire(graph, mv, 0, out, 0);

        graph.set_text_param(src, "object", name);
        graph.set_param(src, ph2d_node_source_object::TIME_OFFSET_PARAM, offset);
        graph.set_param(grid, "rows", 2.0);
        graph.set_param(grid, "cols", 2.0);
        graph.set_param(grid, "gap_x", 2.2);
        graph.set_param(grid, "gap_y", 2.2);
        graph.set_param(mv, "dx", (k as f32) * 6.0 - 3.0);
        graph.set_label(src, if k == 0 { "Now" } else { "+0.25s" });
        sinks.push(out);
    }
    sinks
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A cena AUTORA duas fases do MESMO objeto** — e é a metade declarativa que
    /// impede a cena de mentir: se o laço parasse de escrever o `time_offset`, as duas
    /// grades desenhariam o mesmo desenho e o smoke leria isso como *"o offset não
    /// funciona"* quando o defeito estaria na CENA.
    #[test]
    fn the_two_times_scene_authors_two_phases_of_one_object() {
        let mut g = Graph::new();
        let sinks = build_two_times_graph(&mut g, "Walk");
        assert_eq!(sinks.len(), 2, "duas cadeias, dois sinks");

        let ty = ph2d_node_source_object::MANIFEST.id;
        let mut offs: Vec<f32> = g
            .nodes()
            .iter()
            .filter(|n| n.type_id() == ty)
            .map(|n| {
                g.node_param_overrides(n.id)
                    .and_then(|p| p.get(ph2d_node_source_object::TIME_OFFSET_PARAM))
                    .copied()
                    .unwrap_or(0.0)
            })
            .collect();
        offs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(
            offs,
            vec![0.0, 0.25],
            "uma em agora, a outra tres desenhos a frente"
        );

        // E as duas nomeiam o MESMO objeto — sem isso a cena compararia duas coisas
        // diferentes e a diferenca de fase nao provaria nada.
        for n in g.nodes().iter().filter(|n| n.type_id() == ty) {
            assert_eq!(
                g.node_text_params()
                    .get(&n.id)
                    .and_then(|p| p.get("object")),
                Some(&"Walk".to_string()),
                "as duas cadeias tem de olhar para o mesmo objeto"
            );
        }
    }

    /// **A fixture CONTÉM o fenômeno:** o objeto tem desenhos DIFERENTES em quadros
    /// diferentes. Sobre um Flip de um desenho só, um offset é indistinguível de
    /// nenhum offset — e a cena passaria verde sobre um param inerte.
    #[test]
    fn the_walk_fixture_actually_has_more_than_one_drawing() {
        let mut flip = ph2d_flip::FlipDoc::default();
        spawn_flip_walk_named(&mut flip, "Walk");
        let obj = flip.objects().first().expect("o objeto foi criado");
        let fg = obj.layers().last().expect("a camada FG");
        let drawings: std::collections::BTreeSet<_> = [0, 3, 6, 9]
            .into_iter()
            .filter_map(|f| fg.drawing_at_cycled(f))
            .collect();
        assert_eq!(
            drawings.len(),
            4,
            "quatro quadros, quatro desenhos DISTINTOS — senao o offset nao tem o que mostrar"
        );
    }
}
