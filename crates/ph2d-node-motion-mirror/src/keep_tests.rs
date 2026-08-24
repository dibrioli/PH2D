//! Os gates do [`super::KEEP`] — **espelhar sem duplicar** (doc 89 folha 05).
//!
//! ⚠️ **A fixtura é ASSIMÉTRICA de propósito.** Um layout já simétrico em torno da linha de
//! espelho devolve a mesma figura nos dois modos (as duas metades coincidem), então ele
//! serviria de controle e nunca de prova.

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// Três pontos claramente de um lado só, com `size` e `Index` próprios — as colunas que o
/// corte tem de cortar junto.
static SRC: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.mirror.keep.src"),
    name: "motion.mirror.keep.src",
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
struct Src;
impl NodeOp for Src {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(
            Stream::new(3)
                .with("P", Column::Vec2(vec![[1.0, 0.0], [2.0, 0.5], [4.0, -0.5]]))
                .with(
                    "size",
                    Column::Vec2(vec![[1.0, 1.0], [2.0, 2.0], [3.0, 3.0]]),
                )
                .with("Index", Column::Scalar(vec![0.0, 1.0, 2.0])),
        );
    }
}
struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC.id => Some(&Src),
            t if t == MANIFEST.id => Some(&MotionMirror),
            _ => None,
        }
    }
}

fn run(setup: impl FnOnce(&mut Graph, NodeId)) -> Stream {
    let mut g = Graph::new();
    let src = g.add_node("motion.mirror.keep.src");
    let mi = g.add_node("motion.mirror");
    g.connect(Edge {
        from: (src, 0),
        to: (mi, 0),
        delayed: false,
    })
    .expect("in");
    setup(&mut g, mi);
    let mut cook = Cook::new();
    cook.cook(&g, &Ops, mi, 0.0).expect("cozinha")[0]
        .as_stream()
        .clone()
}

fn pos(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// ⭐ **`Both` é o nó que sempre shipou, AO BIT** — e ele é o default.
#[test]
fn keeping_both_is_the_node_that_shipped_bit_for_bit() {
    let implicit = pos(&run(|_, _| {}));
    let explicit = pos(&run(|g, n| g.set_param(n, KEEP, 0.0)));
    assert_eq!(implicit.len(), 6, "tres originais e tres gemeos");
    for (i, (a, b)) in implicit.iter().zip(&explicit).enumerate() {
        assert_eq!(
            (a[0].to_bits(), a[1].to_bits()),
            (b[0].to_bits(), b[1].to_bits()),
            "elemento {i}"
        );
    }
}

/// ⭐ **`Reflection Only` fica com a METADE ESPELHADA, e é ela mesma** — os `n` de trás, com os
/// bits que o modo `Both` já lhes dava.
#[test]
fn keeping_only_the_reflection_leaves_the_twins_untouched() {
    let both = pos(&run(|_, _| {}));
    let only = pos(&run(|g, n| g.set_param(n, KEEP, 1.0)));
    assert_eq!(only.len(), 3, "so' os tres gemeos");
    for (k, q) in only.iter().enumerate() {
        let twin = both[3 + k];
        assert_eq!(
            (q[0].to_bits(), q[1].to_bits()),
            (twin[0].to_bits(), twin[1].to_bits()),
            "o gemeo {k} tinha de sair identico ao do modo Both"
        );
    }
    // E ele NÃO é o original: a fixtura está toda de um lado, então a reflexão muda de sítio.
    for (k, q) in only.iter().enumerate() {
        let orig = both[k];
        assert!(
            (q[0] - orig[0]).abs() > 0.5,
            "o gemeo {k} ficou em cima do original ({q:?} contra {orig:?}) -- \
             a fixtura tem de ser assimetrica"
        );
    }
}

/// ⚠️ **TODA coluna é cortada junto** — um `size` de `2n` sobre um `P` de `n` é um stream
/// mal-formado, e o modo de falha é a coluna a ler o elemento errado em silêncio.
#[test]
fn every_column_is_cut_with_the_positions() {
    let s = run(|g, n| g.set_param(n, KEEP, 1.0));
    assert_eq!(s.count(), 3, "a CONTAGEM do stream acompanha");
    for name in ["P", "size", "Index"] {
        let len = match s.get(name) {
            Some(Column::Vec2(v)) => v.len(),
            Some(Column::Scalar(v)) => v.len(),
            _ => panic!("a coluna `{name}` sumiu"),
        };
        assert_eq!(len, 3, "a coluna `{name}` ficou com {len} linhas");
    }
    // E são os valores dos GÊMEOS: o `size` do terceiro elemento é o `3.0` da fonte.
    match s.get("size") {
        Some(Column::Vec2(v)) => assert!(
            (v[2][0] - 3.0).abs() < 1e-6,
            "o size do ultimo gemeo e' o do ultimo original: {:?}",
            v[2]
        ),
        _ => panic!("size"),
    }
}

/// ⚠️ **O `reindex` renumera O QUE SOBROU** — uma lista de `n` que se diz `0..2n` mente para
/// todo nó a jusante. É por isso que o corte vem ANTES dele.
#[test]
fn the_reindex_renumbers_what_survived_the_cut() {
    let s = run(|g, n| {
        g.set_param(n, KEEP, 1.0);
        g.set_param(n, REINDEX, 1.0);
    });
    match s.get("Index") {
        Some(Column::Scalar(v)) => assert_eq!(v, &vec![0.0, 1.0, 2.0], "0..n, nao 3..2n"),
        _ => panic!("Index"),
    }
    match s.get("Count") {
        Some(Column::Scalar(v)) => assert!(
            v.iter().all(|c| (c - 3.0).abs() < 1e-6),
            "a contagem publicada tem de ser a que sobrou: {v:?}"
        ),
        None => {}
        _ => panic!("Count"),
    }
}

/// ⚠️ **A linha de espelho continua a ser a do que ENTROU** — recalculá-la sobre o que sai
/// faria o `offset` significar coisas diferentes nos dois modos.
#[test]
fn the_mirror_line_is_still_the_one_of_the_input() {
    for off in [0.0_f32, 1.5] {
        let both = pos(&run(|g, n| g.set_param(n, "offset", off)));
        let only = pos(&run(|g, n| {
            g.set_param(n, "offset", off);
            g.set_param(n, KEEP, 1.0);
        }));
        for (k, q) in only.iter().enumerate() {
            assert_eq!(
                (q[0].to_bits(), q[1].to_bits()),
                (both[3 + k][0].to_bits(), both[3 + k][1].to_bits()),
                "offset {off}, gemeo {k}: o corte mexeu na reflexao"
            );
        }
    }
}

/// **O modo tem painel e as duas palavras.**
#[test]
fn the_mode_is_reachable_and_named() {
    assert_eq!(KEEP_LABELS.len(), 2, "dois modos, dois rotulos");
    assert!(
        MANIFEST.params.iter().any(|p| p.name == KEEP),
        "`keep` no manifesto"
    );
    let hint = PARAM_HINTS
        .iter()
        .find(|h| h.param == KEEP)
        .expect("`keep` sem hint de painel");
    assert!(
        matches!(hint.widget, ParamWidget::Enum { .. }),
        "um enum, nao um toggle -- as palavras sao o que o artista le^"
    );
}
