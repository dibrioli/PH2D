//! **O EIXO** — a fila ou o espaço, e a queda quando não há posição.
//!
//! Saíram do `lib.rs` no teto de LOC, por assunto: o pai fica com **o que o nó É**
//! (o manifesto, a amostra, o kernel) e este irmão com **onde ele amostra**.
//! Seguem FILHO por `#[path]`, então `use super::*` alcança os privados.

use super::*;

fn spec() -> Sample {
    Sample {
        frequency: 0.7,
        speed: 0.0,
        octaves: 3,
        roughness: 0.5,
        amplitude: 1.0,
        offset: 0.0,
        seed: 4.0,
    }
}

/// **O campo ESPACIAL varia com a posição, e é a propriedade inteira.**
///
/// Dois pontos vizinhos no espaço leem valores próximos (o campo é contínuo) e
/// dois pontos distantes leem valores diferentes — que é o que um ruído
/// espacial é, e o que a leitura por índice não pode dar: ali a resposta
/// depende de em que ORDEM o elemento está na lista, não de onde ele está.
#[test]
fn the_world_sample_varies_with_position_not_with_ordinal() {
    let s = spec();
    let near = (s.at_world(0.0, 0.0, 0.0) - s.at_world(0.02, 0.0, 0.0)).abs();
    let far = (s.at_world(0.0, 0.0, 0.0) - s.at_world(9.3, 4.1, 0.0)).abs();
    assert!(
        near < 0.05,
        "o campo e continuo: vizinhos proximos, deu {near}"
    );
    assert!(far > 0.05, "e pontos distantes DIFEREM, deu {far}");
    // O mesmo ponto, sempre o mesmo valor — determinismo (HR-5).
    assert_eq!(
        s.at_world(1.7, -0.4, 0.0).to_bits(),
        s.at_world(1.7, -0.4, 0.0).to_bits()
    );
}

/// **O TEMPO continua a andar no campo espacial** — a referência (MOPs Noise
/// Falloff) pede *"noise→falloff, **animável**"* no mesmo fôlego, e um campo
/// que congelasse ao ganhar `P` perderia metade do pedido.
#[test]
fn the_world_field_still_moves_with_the_playhead() {
    let s = Sample {
        speed: 1.0,
        ..spec()
    };
    let a = s.at_world(1.0, 2.0, 0.0);
    let b = s.at_world(1.0, 2.0, 3.7);
    assert!(
        (a - b).abs() > 0.01,
        "o campo espacial anima: {a} contra {b}"
    );
}

/// **O eixo INDEX é o nó que sempre shipou, ao bit** — as duas leituras são
/// funções diferentes, e é o `space` que escolhe, não uma delas que mudou.
#[test]
fn the_index_axis_is_untouched() {
    let s = spec();
    for i in 0..8u32 {
        let x = 0.0f32 * s.speed;
        let y = i as f32 * s.frequency + s.seed;
        let want = fbm_2d(x, y, s.octaves, s.roughness) * s.amplitude + s.offset;
        assert_eq!(s.at(i, 0.0).to_bits(), want.to_bits(), "i = {i}");
    }
}

// --- a queda sem `P` ---

use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};

/// Uma fonte SEM coluna `P` — o `Count` sozinho, que é o que muitos produtores
/// de valor entregam.
static BARE: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.noise.space.bare"),
    name: "value.noise.space.bare",
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
struct Bare;
impl NodeOp for Bare {
    fn manifest(&self) -> &'static NodeManifest {
        &BARE
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(Stream::new(6).with("Count", Column::Scalar(vec![6.0; 6])));
    }
}
struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == BARE.id => Some(&Bare),
            t if t == MANIFEST.id => Some(&ValueNoise),
            _ => None,
        }
    }
}

fn values(space: f32) -> Vec<f32> {
    let mut g = Graph::new();
    let src = g.add_node("value.noise.space.bare");
    let vn = g.add_node("value.noise");
    g.set_param(vn, "space", space);
    g.set_param(vn, "frequency", 0.7);
    g.connect(Edge {
        from: (src, 0),
        to: (vn, 0),
        delayed: false,
    })
    .expect("in");
    let mut cook = Cook::new();
    match cook.cook(&g, &Ops, vn, 0.0).expect("coze")[0]
        .as_stream()
        .get(VALUE_COL)
    {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// **Sem coluna `P`, o modo World CAI no ÍNDICE — e o campo continua a VARIAR.**
///
/// ⚠️ É a armadilha do device escrita como gate: a `identity` de um binding
/// ausente é zero, e zero como POSIÇÃO daria a todo elemento o mesmo ponto ⇒
/// um campo procedural que devolve **um valor só**, que lê como *"o ruído
/// morreu"*. Um stream sem posição não tem espaço a amostrar, e a resposta
/// honesta é a fila.
#[test]
fn with_no_position_the_world_axis_falls_back_to_the_index() {
    let world = values(1.0);
    assert_eq!(world.len(), 6);
    assert_eq!(world, values(0.0), "sem `P` os dois eixos coincidem");
    let flat = world.windows(2).all(|w| (w[0] - w[1]).abs() < 1e-9);
    assert!(!flat, "e o campo VARIA em vez de colapsar: {world:?}");
}
