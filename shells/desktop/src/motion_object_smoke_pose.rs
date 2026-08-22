//! **A cena da POSE DO OBJETO** (`PH2D_MOTION_OBJ_SMOKE=8`, doc 89 folha 14) — duas
//! grelhas de carimbos do MESMO objeto girado, uma sem herdar a pose e outra a herdá-la.
//!
//! Arquivo próprio pelo mesmo corte que o `=7` já fez: uma cena que traz fiação própria
//! sai do despachante, que estava no teto de LOC do HR-18.

use super::{DEMO_TILE_KEY, OBJECT, build_stamp_graph};
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};
use ph2d_render::Sprite;

/// **Modo `=8`: o objeto nasce com POSE** — girado e achatado, para que herdá-la ou não
/// seja uma diferença de UMA olhada (doc 89 folha 14).
///
/// ⚠️ Os dois números não são estéticos: um objeto girado `0` ou de escala `1` faria as
/// duas metades sairem iguais, e o par ficaria verde e mudo.
const POSE_ROT: f32 = 0.6;
const POSE_SCALE: [f32; 2] = [1.9, 0.55];

/// Como o [`spawn_sprite`], mas com a pose acima — o objeto do modo `=8`.
pub(super) fn spawn_posed_sprite(sim: &mut ph2d_ecs::SimWorld) {
    sim.world_mut().spawn((
        Transform {
            translation: Vec2::new(0.0, 0.0),
            rotation: POSE_ROT,
            scale: Vec2::new(POSE_SCALE[0], POSE_SCALE[1]),
            ..Transform::IDENTITY
        },
        Sprite::atlas(DEMO_TILE_KEY, [0.8, 0.8], [1.0, 1.0, 1.0, 1.0]),
        Name::new(OBJECT),
    ));
}

/// Põe a fileira de carimbos do modo `=8` no seu lado da tela e escolhe se ela herda a
/// pose do objeto.
///
/// ⚠️ **A colocação entra ANTES do `motion.output`, e a fonte dela é o `duplicator`** —
/// a lei que a cena `=73` pagou: um deslocamento posto depois de um campo seria
/// multiplicado por ele. Aqui não há campo, mas a ordem é a mesma de propósito, para que
/// a próxima pessoa não aprenda o hábito errado deste arquivo.
pub(super) fn place_stamp_row(graph: &mut Graph, out: NodeId, side: u8, pose: bool) {
    // O `source.object` desta cadeia é o único nó desse tipo sem `space` escrito ainda.
    if pose
        && let Some(src) = graph
            .nodes()
            .iter()
            .rev()
            .find(|n| n.type_name == "source.object")
            .map(|n| n.id)
    {
        graph.set_param(
            src,
            ph2d_node_source_object::SPACE_PARAM,
            ph2d_node_source_object::SPACE_OBJECT_POSE,
        );
    }
    // Um `motion.move` entre o duplicator e o output.
    let Some(edge) = graph
        .edges()
        .iter()
        .find(|e| e.to == (out, 0))
        .map(|e| e.from)
    else {
        return;
    };
    let mv = graph.add_node("motion.move");
    graph.set_pos(
        mv,
        Pos {
            x: 320.0,
            y: -200.0 - f32::from(side) * 60.0,
        },
    );
    graph.set_param(mv, "dx", if side == 0 { -3.6 } else { 3.6 });
    graph.disconnect(out, 0);
    let _ = graph.connect(Edge {
        from: edge,
        to: (mv, 0),
        delayed: false,
    });
    let _ = graph.connect(Edge {
        from: (mv, 0),
        to: (out, 0),
        delayed: false,
    });
}

/// A cena do modo `=8`, montada de uma vez no frame 3.
pub(super) fn run(gfx: &mut crate::AppGfx) {
    spawn_posed_sprite(&mut gfx.sim);
    for (k, pose) in [(0u8, false), (1, true)] {
        let out = build_stamp_graph(&mut gfx.motion.doc.graph, OBJECT);
        // A grelha deste par vai para o seu lado da tela.
        place_stamp_row(&mut gfx.motion.doc.graph, out, k, pose);
        gfx.motion.sinks.push(out);
    }
    let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
    eprintln!(
        "[motion.obj smoke =8] O objeto 'Object' esta' GIRADO e ACHATADO na cena.
  As duas grades carimbam o MESMO objeto.
  ESQUERDA (Transform = Position Only): as copias saem DIREITAS -- a pose do
    objeto nao viaja com o carimbo. E' o que este no' sempre fez.
  DIREITA  (Transform = Object Pose): cada copia nasce GIRADA e ACHATADA como o
    objeto. Gire o sprite na cena e as copias da direita giram com ele.
  > clique num no' Object e troque o `Transform` entre os dois.
  (!) DEU ERRADO se as duas grades sairem iguais."
    );
}
