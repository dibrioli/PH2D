//! A marcha restrita ao cone (ciclo 12, doc 120 §8.5) — o gate conta AVALIAÇÕES, porque a pergunta
//! é *«este laço foi simulado?»* e não *«a resposta está certa?»*: um laço simulado a mais dá a
//! mesma resposta e custa `4 ms`.

use super::cone_a_montante;
use crate::cook::{Cook, EvalCtx, OpResolver, TimeFans, TimeScopes};
use crate::effect::Effect;
use crate::graph::{Edge, Graph, NodeId};
use crate::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
use crate::port::{Clock, Dim, Domain, PortType};
use std::sync::atomic::{AtomicU64, Ordering};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

macro_rules! no_que_conta {
    ($man:ident, $id:literal) => {
        static $man: NodeManifest = NodeManifest {
            id: NodeTypeId::of($id),
            name: $id,
            inputs: &[PortSpec {
                name: "in",
                ty: INST_VEC2,
            }],
            outputs: &[PortSpec {
                name: "out",
                ty: INST_VEC2,
            }],
            effect: Effect::Pure,
            clock: Clock::Frame,
            params: &[],
            lowerings: &[LoweringKind::Cpu],
        };
    };
}
no_que_conta!(MAN_A, "test.laco_a");
no_que_conta!(MAN_B, "test.laco_b");
no_que_conta!(MAN_C, "test.fronteira");

/// Um nó que passa a entrada e conta quantas vezes foi avaliado.
struct Conta(&'static NodeManifest, AtomicU64);
impl NodeOp for Conta {
    fn manifest(&self) -> &'static NodeManifest {
        self.0
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        self.1.fetch_add(1, Ordering::Relaxed);
        let s = ctx.input(0).clone();
        ctx.emit(s);
    }
}
struct Ops([Conta; 3]);
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        self.0
            .iter()
            .find(|c| c.0.id == ty)
            .map(|c| c as &dyn NodeOp)
    }
}
impl Ops {
    fn novos() -> Self {
        Self([
            Conta(&MAN_A, AtomicU64::new(0)),
            Conta(&MAN_B, AtomicU64::new(0)),
            Conta(&MAN_C, AtomicU64::new(0)),
        ])
    }
    fn evals(&self, i: usize) -> u64 {
        self.0[i].1.load(Ordering::Relaxed)
    }
}

/// `A ⟲pre` → `C` (a fronteira) · `B ⟲pre` sozinho (o laço que a placa reclamou).
fn grafo() -> (Graph, NodeId, NodeId, NodeId) {
    let mut g = Graph::new();
    let a = g.add_node("test.laco_a");
    let b = g.add_node("test.laco_b");
    let c = g.add_node("test.fronteira");
    for n in [a, b] {
        g.connect(Edge {
            from: (n, 0),
            to: (n, 0),
            delayed: true,
        })
        .expect("auto-laco pre");
    }
    g.connect(Edge {
        from: (a, 0),
        to: (c, 0),
        delayed: false,
    })
    .expect("a -> c");
    (g, a, b, c)
}

/// ⭐⭐⭐ **Marchar DENTRO do cone da fronteira simula só o laço que ela lê** — e o CONTROLO é a
/// marcha de sempre, que simula os dois (é a lei certa quando a CPU coze o grafo inteiro).
#[test]
fn a_marcha_restrita_so_avanca_o_laco_que_a_fronteira_le() {
    let (g, a, _laco_fora, c) = grafo();
    assert_eq!(
        cone_a_montante(&g, &[c]),
        [a, c].into_iter().collect(),
        "o cone de C e' C e A (B e' um laco que ninguem a montante de C le)"
    );
    let ops = Ops::novos();
    let mut cook = Cook::new();
    cook.advance_tick_fanned_within(&g, &ops, 0.0, &TimeScopes::new(), &TimeFans::new(), &[c])
        .expect("avanca");
    assert_eq!(ops.evals(0), 1, "o laco A (no cone) e' simulado");
    assert_eq!(ops.evals(1), 0, "o laco B (fora do cone) NAO e' simulado");

    // O CONTROLO: a marcha de sempre simula os dois laços.
    let ops = Ops::novos();
    let mut cook = Cook::new();
    cook.advance_tick_fanned(&g, &ops, 0.0, &TimeScopes::new(), &TimeFans::new())
        .expect("avanca");
    assert_eq!(
        (ops.evals(0), ops.evals(1)),
        (1, 1),
        "CONTROLO: sem alvo, toda fonte de pre avanca"
    );
}

/// ⚠️ **O cone segue as arestas de `pre` também** — um nó que lê o tique ANTERIOR de uma fonte
/// precisa que ela avance, mesmo sem aresta directa.
#[test]
fn o_cone_segue_as_arestas_de_pre() {
    let mut g = Graph::new();
    let fonte = g.add_node("test.laco_a");
    let leitor = g.add_node("test.fronteira");
    g.connect(Edge {
        from: (fonte, 0),
        to: (leitor, 0),
        delayed: true,
    })
    .expect("fonte --pre--> leitor");
    assert!(
        cone_a_montante(&g, &[leitor]).contains(&fonte),
        "a fonte do pre esta no cone de quem a le"
    );
}

/// ⛔⛔ **Um laço que só CONDUZ UM PARAM da fronteira está no cone dela** (auditoria do fecho,
/// 2026-09-24): o fio de param não vive nas `edges()`, e a 1.ª redacção do cone esquecia-o — o laço
/// ficava de fora, era re-semeado a cada tique e o param lia sempre o 1.º valor. ⚠️ O CONTROLO é o
/// mesmo grafo sem o fio: aí o laço está FORA, que é o que prova que é o fio que o põe dentro.
#[test]
fn o_cone_segue_os_fios_que_conduzem_um_param() {
    let (mut g, a, b, c) = grafo();
    assert!(
        !cone_a_montante(&g, &[c]).contains(&b),
        "CONTROLO: sem o fio, o laco B nao alimenta a fronteira C"
    );
    g.drive_param(c, "mark", (b, 0))
        .expect("B conduz um param de C");
    let cone = cone_a_montante(&g, &[c]);
    assert!(
        cone.contains(&b),
        "o laco que conduz um param da fronteira tem de avancar com ela"
    );
    assert!(cone.contains(&a), "e a aresta directa continua la");
}
