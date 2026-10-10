//! ⭐ O crossfade contra o ORÁCULO CORRIDO (doc 128 §5.2): o `AnimationNodeTransition` do Godot
//! 4.7.2, gravado por `docs/Motion Nodes/ferramentas/godot_xfade_oracle/corre.sh` em
//! `tests/fixtures/godot_xfade.txt`. A cadeia é a do produto — `value.input` (o pedido publicado
//! pelo jogo) → `value.follow` (Linear, `duration` = `xfade_time`) — cozida tique a tique.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::external::{INPUT_VALUE, input_of};
use ph2d_nodegraph::graph::{Edge, Graph};

const FIXTURE: &str = include_str!("fixtures/godot_xfade.txt");

#[test]
fn the_linear_follow_is_the_godot_crossfade_frame_by_frame() {
    let mut reg = NodeRegistry::new();
    ph2d_node_value_input::register(&mut reg).unwrap();
    ph2d_node_value_follow::register(&mut reg).unwrap();
    let mut casos = 0;
    for linha in FIXTURE.lines().filter(|l| l.starts_with("XFADE")) {
        let (cab, valores) = linha.split_once('|').unwrap();
        let cab: Vec<&str> = cab.split_whitespace().collect();
        let xfade: f32 = cab[1].parse().unwrap();
        let pedido: usize = cab[3].parse().unwrap();
        let godot: Vec<f32> = valores
            .split_whitespace()
            .map(|v| v.parse().unwrap())
            .collect();

        let mut g = Graph::new();
        let entrada = g.add_node("value.input");
        g.set_text_param(entrada, "name", "estado");
        let seguir = g.add_node("value.follow");
        g.set_param(seguir, "duration", xfade);
        g.connect(Edge {
            from: (entrada, 0),
            to: (seguir, 0),
            delayed: false,
        })
        .unwrap();
        g.connect(Edge {
            from: (seguir, 0),
            to: (seguir, 1),
            delayed: true,
        })
        .unwrap();
        let mut cook = Cook::new();
        let mut pior = 0.0f32;
        for (k, &esperado) in godot.iter().enumerate() {
            let estado = if k >= pedido { 1.0 } else { 0.0 };
            cook.set_external(
                input_of("estado"),
                Stream::new(1).with(INPUT_VALUE, Column::Scalar(vec![estado])),
            );
            let t = k as f64 / 60.0;
            let v = match cook.cook(&g, &reg, seguir, t).unwrap()[0]
                .as_stream()
                .get("v")
            {
                Some(Column::Scalar(v)) => v[0],
                _ => f32::NAN,
            };
            cook.advance_tick(&g, &reg, t).unwrap();
            pior = pior.max((v - esperado).abs());
        }
        // A régua: o Godot põe `1e-5` (o CMP_EPSILON dele) no quadro do pedido em vez de 0.
        assert!(pior <= 2e-5, "xfade {xfade}: pior diferença {pior}");
        casos += 1;
    }
    assert_eq!(casos, 3, "as três durações da fixture");
}
