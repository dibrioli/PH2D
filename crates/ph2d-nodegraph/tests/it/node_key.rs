//! **A identidade que um nó vê de si mesmo** — `EvalCtx::node_key`.
//!
//! Ela existe para um nó estocástico se decorrelacionar de um irmão de mesma
//! semente. O que estes gates pinam é o que os consumidores dependem: que ela é
//! o `NodeId`, que difere entre nós, e que **não se move** entre cooks — senão
//! um campo aleatório re-sorteava sozinho a cada quadro.

use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::{Cook, EvalCtx, OpResolver};
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::graph::Graph;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// Um nó que EMITE a própria identidade, para o teste poder lê-la de fora.
static TELL: NodeManifest = NodeManifest {
    id: NodeTypeId::of("test.tell_node_key"),
    name: "test.tell_node_key",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};
struct Tell;
impl NodeOp for Tell {
    fn manifest(&self) -> &'static NodeManifest {
        &TELL
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        #[expect(
            clippy::cast_precision_loss,
            reason = "a fixture usa ids pequenos; o produto le o u32 cru"
        )]
        let k = ctx.node_key() as f32;
        ctx.emit(Stream::new(1).with("v", Column::Scalar(vec![k])));
    }
}
struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        (ty == TELL.id).then_some(&Tell as &dyn NodeOp)
    }
}

fn told(cook: &mut Cook, g: &Graph, n: ph2d_nodegraph::graph::NodeId) -> f32 {
    let out = cook.cook(g, &Ops, n, 0.0).unwrap();
    match out[0].as_stream().get("v").unwrap() {
        Column::Scalar(v) => v[0],
        _ => panic!("v"),
    }
}

/// **A chave É o `NodeId`** — não um contador do cook, não uma posição de
/// iteração. É o que a torna estável no arquivo salvo.
#[test]
fn the_key_a_node_sees_is_its_own_node_id() {
    let mut g = Graph::new();
    let a = g.add_node("test.tell_node_key");
    let b = g.add_node("test.tell_node_key");
    let mut cook = Cook::new();
    #[expect(clippy::cast_precision_loss, reason = "ids pequenos na fixture")]
    {
        assert_eq!(told(&mut cook, &g, a), a.0 as f32);
        assert_eq!(told(&mut cook, &g, b), b.0 as f32);
    }
}

/// **Dois nós vêem chaves DIFERENTES** — a propriedade inteira, sem a qual o
/// `unique_per_node` não decorrelacionaria coisa nenhuma.
#[test]
fn two_nodes_see_different_keys() {
    let mut g = Graph::new();
    let a = g.add_node("test.tell_node_key");
    let b = g.add_node("test.tell_node_key");
    let mut cook = Cook::new();
    assert_ne!(told(&mut cook, &g, a), told(&mut cook, &g, b));
}

/// **A chave não se move entre cooks** — um campo aleatório que a lesse
/// re-sortearia sozinho a cada quadro, e o scrub deixaria de ser bit-exato.
#[test]
fn the_key_does_not_move_between_cooks() {
    let mut g = Graph::new();
    let a = g.add_node("test.tell_node_key");
    let mut cook = Cook::new();
    let first = told(&mut cook, &g, a);
    // Um nó novo no meio não pode mexer com quem já existe.
    let _ = g.add_node("test.tell_node_key");
    for t in 1..5 {
        let mut c2 = Cook::new();
        let out = c2.cook(&g, &Ops, a, f64::from(t)).unwrap();
        let v = match out[0].as_stream().get("v").unwrap() {
            Column::Scalar(v) => v[0],
            _ => panic!("v"),
        };
        assert_eq!(v, first, "tick {t}");
    }
}

/// **Um id nunca é reusado** — apagar um nó e criar outro não devolve a chave do
/// morto, senão o campo do nó novo nasceria idêntico ao de um que o artista
/// acabou de tirar da tela.
#[test]
fn a_removed_nodes_key_is_never_handed_out_again() {
    let mut g = Graph::new();
    let a = g.add_node("test.tell_node_key");
    g.remove_node(a);
    let b = g.add_node("test.tell_node_key");
    assert_ne!(a.0, b.0, "o id do morto nao volta");
}
