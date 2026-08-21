//! Os gates do [`super::DENSITY_COL`] — a densidade por-instância (doc 89, folha 02).
//!
//! ⚠️ **A 1ª versão destes gates repetia a aritmética do `eval` num `run()` local, e
//! a mutação que IGNORA a coluna sobreviveu.** Um gate que reimplementa a lei mede a
//! cópia dele. Agora eles cozinham o nó.

use super::*;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::{Cook, EvalCtx, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};
use ph2d_nodegraph::node::{NodeManifest, NodeOp, NodeTypeId, PortSpec};

/// Duas peças no MESMO ponto submerso e paradas — a rolha e a pedra.
///
/// ⚠️ **No mesmo sítio de propósito:** o empuxo depende da profundidade, então peças
/// em alturas diferentes teriam acelerações diferentes sem knob nenhum, e o gate
/// mediria a fixture em vez do nó.
static SRC_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("force.buoyancy.test.pair"),
    name: "force.buoyancy.test.pair",
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

/// A MESMA fonte sem a coluna — o controle da ausência.
static BARE_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("force.buoyancy.test.bare"),
    name: "force.buoyancy.test.bare",
    ..SRC_MAN
};

fn base() -> Stream {
    Stream::new(2)
        .with("P", Column::Vec2(vec![[0.0, -1.0], [0.0, -1.0]]))
        .with("vel", Column::Vec2(vec![[0.0, 0.0], [0.0, 0.0]]))
}

struct WithCol;
impl NodeOp for WithCol {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(base().with(DENSITY_COL, Column::Scalar(vec![2.0, 0.25])));
    }
}

struct Ones;
impl NodeOp for Ones {
    fn manifest(&self) -> &'static NodeManifest {
        &ONES_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(base().with(DENSITY_COL, Column::Scalar(vec![1.0, 1.0])));
    }
}

/// A fonte cuja coluna é toda `1` — o gêmeo da ausência.
static ONES_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("force.buoyancy.test.ones"),
    name: "force.buoyancy.test.ones",
    ..SRC_MAN
};

struct Bare;
impl NodeOp for Bare {
    fn manifest(&self) -> &'static NodeManifest {
        &BARE_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(base());
    }
}

struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC_MAN.id => Some(&WithCol),
            t if t == BARE_MAN.id => Some(&Bare),
            t if t == ONES_MAN.id => Some(&Ones),
            t if t == MANIFEST.id => Some(&ForceBuoyancy),
            _ => None,
        }
    }
}

/// Coze `src → force.buoyancy(mar parado, sem arrasto)` e devolve os `accel`.
fn floats(src: &str) -> Vec<[f32; 2]> {
    let mut g = Graph::new();
    let s = g.add_node(src);
    let b = g.add_node("force.buoyancy");
    g.connect(Edge {
        from: (s, 0),
        to: (b, 0),
        delayed: false,
    })
    .unwrap();
    // Mar liso e sem arrasto: sobra o empuxo puro, que é o que o knob escala.
    g.set_param(b, "wave_amplitude", 0.0);
    g.set_param(b, "drag", 0.0);
    let mut cook = Cook::new();
    match cook.cook(&g, &Ops, b, 0.0).unwrap()[0]
        .as_stream()
        .get("accel")
    {
        Some(Column::Vec2(v)) => v.clone(),
        other => panic!("accel: {other:?}"),
    }
}

/// **SEM A COLUNA, AS DUAS PEÇAS FLUTUAM IGUAL** — o nó de sempre.
#[test]
fn without_the_column_both_pieces_get_the_global_density() {
    let a = floats("force.buoyancy.test.bare");
    assert_eq!(a[0], a[1], "sem coluna não há como diferir");
    assert!(a[0][1] > 0.0, "e o empuxo empurra para cima: {:?}", a[0]);
}

/// **A ROLHA E A PEDRA NA MESMA ÁGUA** — o caso de uso que a célula nomeia.
///
/// ⚠️ O oráculo é a RAZÃO e não os valores: `2` contra `0,25` tem de dar oito vezes,
/// seja qual for a profundidade da fixture. Um gate sobre números absolutos mediria
/// o `depth` junto.
#[test]
fn a_cork_and_a_stone_float_differently_in_the_same_water() {
    let a = floats("force.buoyancy.test.pair");
    assert!(
        (a[0][1] / a[1][1] - 8.0).abs() < 1e-4,
        "a razão tem de ser 8: {:?} contra {:?}",
        a[0],
        a[1]
    );
}

/// **UMA COLUNA DE UNS É EXACTAMENTE A AUSÊNCIA** — é isso que faz o neutro existir.
///
/// ⚠️ Uma densidade por-instância ABSOLUTA teria de valer `0` quando a coluna falta,
/// e aí a ausência afundaria tudo. Multiplicativa, a ausência é `1`.
#[test]
fn a_column_of_ones_is_exactly_no_column_at_all() {
    assert_eq!(
        floats("force.buoyancy.test.ones"),
        floats("force.buoyancy.test.bare")
    );
}

/// **O ÚNICO FALLBACK QUE PODE ACONTECER É A AUSÊNCIA** — e o gate diz por quê.
///
/// ⚠️ **Escrevi primeiro um gate de coluna CURTA, e ele reprovou — sobre o
/// substrato, não sobre o nó:** `Stream::set` assere `column length must equal
/// stream element count`. Uma coluna curta é **impossível por construção**, então o
/// `unwrap_or(1.0)` do [`scale_at`] é defesa de borda e não um caminho de produto.
/// *Um gate que encena o que o substrato proíbe está a medir a fixture.*
///
/// O que fica gateado é a defesa em si, chamada direto — ela existe para o índice
/// fora de alcance nunca virar pânico, e o valor dela tem de ser o MESMO neutro.
#[test]
fn the_only_reachable_fallback_is_an_absent_column() {
    let s = base().with(DENSITY_COL, Column::Scalar(vec![2.0, 3.0]));
    assert_eq!(scale_at(&s, 0), 2.0);
    assert_eq!(scale_at(&s, 1), 3.0);
    // Fora de alcance: o neutro, nunca um pânico.
    assert_eq!(scale_at(&s, 99), 1.0);
    // E a ausência, que é o caso real.
    assert_eq!(scale_at(&base(), 0), 1.0);
}

/// **O DEVICE LÊ A MESMA COLUNA, com a MESMA identidade.**
#[test]
fn the_device_reads_the_same_column_with_the_same_identity() {
    let b = GPU_KERNEL
        .bindings
        .iter()
        .find(|b| b.column == DENSITY_COL)
        .expect("o device tem de ler a densidade");
    assert_eq!(b.identity[0], 1.0, "ausente vale 1 nos dois lados");
    assert!(matches!(b.access, ColumnAccess::Read), "ele não a escreve");
}
