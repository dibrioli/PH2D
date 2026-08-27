//! **AS PROVAS DO NÓ** — irmão pelo teto de LOC (700), pelo idioma que a casa já usa
//! (`transform_tests.rs`, `children_order_tests.rs`): o pai fica com a lei, este com o que a
//! afirma.

use super::*;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};

/// A source that emits a fixed two-instance stream — the "before" the glow
/// passthrough must reproduce exactly.
struct Source;
const SOURCE: NodeManifest = NodeManifest {
    id: NodeTypeId::of("test.source"),
    name: "test.source",
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
fn fixture() -> Stream {
    Stream::new(2)
        .with("P", Column::Vec2(vec![[1.0, 2.0], [3.0, 4.0]]))
        .with(
            "tint",
            Column::Vec4(vec![[6.0, 4.0, 2.0, 1.0], [1.0, 1.0, 1.0, 1.0]]),
        )
}
impl NodeOp for Source {
    fn manifest(&self) -> &'static NodeManifest {
        &SOURCE
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(fixture());
    }
}
struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        if ty == MANIFEST.id {
            Some(&FxGlow as &dyn NodeOp)
        } else if ty == SOURCE.id {
            Some(&Source as &dyn NodeOp)
        } else {
            None
        }
    }
}

/// **The invariant that lets `fx.glow` live anywhere: it is byte-identical in
/// the cook.** `source → fx.glow` produces exactly what `source` alone does —
/// so dropping the node in a chain never changes the stream, and the glow it
/// configures is a pure addition on the render side.
#[test]
fn glow_is_a_byte_identical_passthrough() {
    let mut g = Graph::new();
    let src = g.add_node("test.source");
    let glow = g.add_node(TYPE_NAME);
    g.connect(Edge {
        from: (src, 0),
        to: (glow, 0),
        delayed: false,
    })
    .unwrap();
    let mut cook = Cook::new();
    let through = cook.cook(&g, &Ops, glow, 0.0).unwrap()[0]
        .as_stream()
        .clone();
    assert_eq!(
        through.get("P"),
        fixture().get("P"),
        "the glow node must not move a point"
    );
    assert_eq!(
        through.get("tint"),
        fixture().get("tint"),
        "the glow node must not touch a colour (the HDR tint survives verbatim)"
    );
}

#[test]
fn no_glow_node_reads_none() {
    // The neutral point: a graph without the node never runs the pass.
    let g = Graph::new();
    assert_eq!(from_graph(&g), None);
}

#[test]
fn reads_defaults_then_overrides() {
    let mut g = Graph::new();
    let n = g.add_node(TYPE_NAME);
    // Untouched → manifest defaults (white, full-saturation glow).
    assert_eq!(
        from_graph(&g),
        Some(Glow {
            threshold: 1.0,
            knee: 0.6,
            intensity: 0.8,
            radius: 1.0,
            saturation: 1.0,
            tint: [1.0, 1.0, 1.0, 1.0],
            // Os três da folha 11, todos no neutro: halo redondo, sem teto.
            stretch: 1.0,
            angle: 0.0,
            clamp: 0.0,
            // …e os três da wave dos modos + rampa, idem: `ramp_len = 0` é *sem rampa*,
            // e o passe usa o `tint` constante de sempre.
            operation: 0.0,
            source: 0.0,
            // …e a máscara de sujidade apagada: sem imagem escolhida (canal de texto,
            // vazio) e com o knob no zero.
            dirt_intensity: 0.0,
        })
    );
    // A dragged slider overrides just that knob; the rest stay at default.
    g.set_param(n, "intensity", 2.5);
    g.set_param(n, "threshold", 1.5);
    g.set_param(n, "saturation", 0.0);
    g.set_param(n, "tint_r", 0.5);
    let glow = from_graph(&g).unwrap();
    assert_eq!(glow.intensity, 2.5);
    assert_eq!(glow.threshold, 1.5);
    assert_eq!(glow.saturation, 0.0, "a white bloom");
    assert_eq!(
        glow.tint,
        [0.5, 1.0, 1.0, 1.0],
        "just the red channel moved"
    );
    assert_eq!(glow.radius, 1.0, "untouched knob keeps its default");
}

/// **TODO PARAM DO MANIFESTO CHEGA AO `Glow`** — e o oráculo é DERIVADO.
///
/// ⚠️ Este gate dizia `params.len() == 9`, e a folha 11 partiu-o ao apendar três
/// knobs sobre código correcto. Uma CONTAGEM escrita à mão não afirma nada sobre
/// o nó: ela não sabe se o param novo é lido, e envelhece na primeira adição. O
/// que a célula queria dizer é *"nenhum knob nasce mudo"*, e isso mede-se
/// mexendo em cada um e exigindo que a struct MUDE.
///
/// ⚠️ **Sem controle positivo isto passaria por vácuo** se `MANIFEST.params`
/// ficasse vazio, então o gate também exige que a varredura tenha achado gente.
#[test]
fn every_manifest_param_reaches_the_glow_struct() {
    let mut checked = 0usize;
    for spec in MANIFEST.params {
        let mut g = Graph::new();
        let n = g.add_node(TYPE_NAME);
        let before = from_graph(&g).expect("o nó existe");
        // Um valor que difere do default seja ele qual for.
        g.set_param(n, spec.name, spec.default + 1.25);
        let after = from_graph(&g).expect("o nó existe");
        assert_ne!(
            before, after,
            "`{}` está no manifesto e o leitor não o vê — um knob mudo",
            spec.name
        );
        checked += 1;
    }
    assert!(checked >= 9, "controle: a varredura achou {checked} params");
    assert_eq!(default_of("knee"), 0.6);
    assert_eq!(default_of("stretch"), 1.0, "o halo redondo é o neutro");
}
