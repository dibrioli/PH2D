//! Os gates do [`super::REINDEX`] — as colunas de identidade depois de replicar
//! (doc 89, folha 05).

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};

/// Três elementos com as duas colunas de identidade honestas.
static SRC_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.kaleidoscope.test.trio"),
    name: "motion.kaleidoscope.test.trio",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};

struct Trio;
impl NodeOp for Trio {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(
            Stream::new(3)
                .with("P", Column::Vec2(vec![[1.0, 0.0], [2.0, 0.0], [3.0, 0.0]]))
                .with(INDEX, Column::Scalar(vec![0.0, 1.0, 2.0]))
                .with(COUNT, Column::Scalar(vec![3.0; 3])),
        );
    }
}

/// A mesma lista sem identidade nenhuma.
static BARE_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.kaleidoscope.test.bare"),
    name: "motion.kaleidoscope.test.bare",
    ..SRC_MAN
};

struct Bare;
impl NodeOp for Bare {
    fn manifest(&self) -> &'static NodeManifest {
        &BARE_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(Stream::new(2).with("P", Column::Vec2(vec![[1.0, 0.0], [2.0, 0.0]])));
    }
}

struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC_MAN.id => Some(&Trio),
            t if t == BARE_MAN.id => Some(&Bare),
            t if t == MANIFEST.id => Some(&MotionKaleidoscope),
            _ => None,
        }
    }
}

fn run(src: &str, segments: f32, reindex: f32) -> Stream {
    let mut g = Graph::new();
    let s = g.add_node(src);
    let k = g.add_node("motion.kaleidoscope");
    g.connect(Edge {
        from: (s, 0),
        to: (k, 0),
        delayed: false,
    })
    .unwrap();
    g.set_param(k, "segments", segments);
    g.set_param(k, REINDEX, reindex);
    let mut cook = Cook::new();
    cook.cook(&g, &Ops, k, 0.0).unwrap()[0].as_stream().clone()
}

fn scalar(st: &Stream, name: &str) -> Option<Vec<f32>> {
    match st.get(name) {
        Some(Column::Scalar(v)) => Some(v.clone()),
        _ => None,
    }
}

/// **DESLIGADO, O `Index` REPETE-SE UMA VEZ POR FATIA** — o defeito MEDIDO, fixado
/// como o comportamento de hoje (ver [`REINDEX`] para o porquê do default `0`).
#[test]
fn without_the_renumbering_the_index_repeats_once_per_slice() {
    let out = run("motion.kaleidoscope.test.trio", 4.0, 0.0);
    assert_eq!(out.count(), 12, "quatro fatias de três");
    assert_eq!(
        scalar(&out, INDEX),
        Some(vec![
            0.0, 1.0, 2.0, 0.0, 1.0, 2.0, 0.0, 1.0, 2.0, 0.0, 1.0, 2.0
        ])
    );
    assert_eq!(
        scalar(&out, COUNT),
        Some(vec![3.0; 12]),
        "e o Count diz 3 sobre uma lista de 12"
    );
}

/// **LIGADO, AS DUAS DESCREVEM A LISTA REPLICADA.**
#[test]
fn the_renumbering_describes_the_replicated_list() {
    let out = run("motion.kaleidoscope.test.trio", 4.0, 1.0);
    let idx: Vec<f32> = (0..12).map(|i| i as f32).collect();
    assert_eq!(scalar(&out, INDEX), Some(idx));
    assert_eq!(scalar(&out, COUNT), Some(vec![12.0; 12]));
}

/// **CUNHA AS DUAS MESMO EM BRANCO** — a contagem mudou (× `segments`).
#[test]
fn it_mints_both_columns_even_when_the_input_had_none() {
    let plain = run("motion.kaleidoscope.test.bare", 3.0, 0.0);
    assert_eq!(scalar(&plain, INDEX), None, "sem o knob, não cunha nada");
    let out = run("motion.kaleidoscope.test.bare", 3.0, 1.0);
    assert_eq!(
        scalar(&out, INDEX),
        Some(vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0])
    );
    assert_eq!(scalar(&out, COUNT), Some(vec![6.0; 6]));
}

/// **A RENUMERAÇÃO NÃO MOVE UMA PEÇA** — ela é sobre a lista, não sobre a forma.
///
/// ⚠️ Sem este gate, uma renumeração que também tocasse `P` (um `set` fora de
/// ordem, um `Stream::new` com a contagem errada) passaria pelos dois gates acima.
#[test]
fn the_renumbering_moves_no_element() {
    let a = run("motion.kaleidoscope.test.trio", 4.0, 0.0);
    let b = run("motion.kaleidoscope.test.trio", 4.0, 1.0);
    match (a.get("P"), b.get("P")) {
        (Some(Column::Vec2(x)), Some(Column::Vec2(y))) => assert_eq!(x, y),
        _ => panic!("P"),
    }
}

/// **O KNOB ESTÁ PINTADO, e o device recua só quando ele morde.**
///
/// ⚠️ A recusa é estrutural e nomeada: este é um kernel `SourceRows`, cujas colunas
/// não-`P` chegam à saída por um GATHER do template. Escrever `Index`/`Count`
/// **novos** é outra operação — a mesma razão pela qual o `motion.combine` já
/// declara este `applicable`.
#[test]
fn the_knob_is_painted_and_the_device_steps_back_only_when_it_bites() {
    let h = PARAM_HINTS
        .iter()
        .find(|h| h.param == REINDEX)
        .expect("o Reindex tem de estar pintado");
    assert!(matches!(h.widget, ParamWidget::Toggle));
    let app = GPU_KERNEL.applicable.expect("a recusa existe");
    assert!(app(&|_| 0.0), "desligado, o device faz o de sempre");
    assert!(
        !app(&|n| if n == REINDEX { 1.0 } else { 0.0 }),
        "ligado, o CPU `eval` é quem responde"
    );
}
