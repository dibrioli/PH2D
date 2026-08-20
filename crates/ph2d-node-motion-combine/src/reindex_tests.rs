//! **A RENUMERAÇÃO** — os gates da célula do `reindex` (doc 89, folha 08).
//!
//! Irmão por `#[path]`, então `super` é a raiz da crate e o `use super::*` alcança o
//! `combine`/`reindex` privados.

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};

/// Um stream de `n` linhas com as colunas de identidade que uma FONTE escreve
/// (`motion.grid` escreve exactamente estas duas).
fn sourced(n: usize, tag: f32) -> Stream {
    #[expect(clippy::cast_precision_loss, reason = "uma contagem de elementos")]
    let idx: Vec<f32> = (0..n).map(|i| i as f32).collect();
    #[expect(clippy::cast_precision_loss, reason = "uma contagem de elementos")]
    let total = n as f32;
    Stream::new(n)
        .with("P", Column::Vec2(vec![[tag, 0.0]; n]))
        .with("Index", Column::Scalar(idx))
        .with("Count", Column::Scalar(vec![total; n]))
}

fn scalars(s: &Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// **DESLIGADO É O MUNDO DE SEMPRE, AO BIT** — e é o mundo em que as duas colunas mentem.
///
/// ⚠️ O gate afirma a MENTIRA de propósito: ela é o comportamento que o default preserva, e
/// pinar-a é o que torna visível o dia em que alguém mudar o default sem dizer.
#[test]
fn with_reindex_off_the_identity_columns_travel_verbatim() {
    let mut out = combine(&[snapshot(&sourced(9, 1.0)), snapshot(&sourced(4, 2.0))]);
    assert_eq!(out.count(), 13);
    assert_eq!(
        scalars(&out, "Index"),
        vec![0., 1., 2., 3., 4., 5., 6., 7., 8., 0., 1., 2., 3.],
        "o `Index` da junção repete — é o defeito que o default preserva"
    );
    assert_eq!(
        scalars(&out, "Count"),
        vec![9.0; 9]
            .into_iter()
            .chain(vec![4.0; 4])
            .collect::<Vec<_>>()
    );

    // E o CONTROLE: ligado, as duas passam a dizer a verdade sobre a lista JUNTA.
    reindex(&mut out);
    assert_eq!(
        scalars(&out, "Index"),
        (0..13).map(|i| i as f32).collect::<Vec<_>>()
    );
    assert_eq!(scalars(&out, "Count"), vec![13.0; 13]);
}

/// **A RENUMERAÇÃO ESCREVE AS DUAS COLUNAS MESMO QUANDO NENHUMA ENTRADA AS TROUXE.**
///
/// ⚠️ Sem isto o `reindex` seria um remendo condicional: uma junção de fontes sem `Index`
/// sairia sem ele, e o nó a jusante inventaria o dele a partir da POSIÇÃO — a mesma resposta
/// por acidente, até alguém pôr um `motion.sort` no meio.
#[test]
fn reindex_mints_the_columns_even_when_no_input_had_them() {
    let bare = |n: usize| Stream::new(n).with("P", Column::Vec2(vec![[0.0, 0.0]; n]));
    let mut out = combine(&[snapshot(&bare(2)), snapshot(&bare(3))]);
    assert!(
        out.get("Index").is_none(),
        "o CONTROLE: nem uma entrada trouxe"
    );
    reindex(&mut out);
    assert_eq!(scalars(&out, "Index"), vec![0., 1., 2., 3., 4.]);
    assert_eq!(scalars(&out, "Count"), vec![5.0; 5]);
}

/// **O `eval` só renumera quando o param o pede** — a costura entre o knob e a lei.
///
/// ⚠️ É o gate que os dois de cima NÃO dão: eles chamam `reindex` à mão. Uma lei correcta que
/// o `eval` nunca invoca é o modo de falha clássico desta casa.
#[test]
fn the_param_is_what_turns_the_renumbering_on() {
    struct Src(&'static NodeManifest, usize, f32);
    impl NodeOp for Src {
        fn manifest(&self) -> &'static NodeManifest {
            self.0
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(sourced(self.1, self.2));
        }
    }
    static A_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.combine.test.a"),
        name: "motion.combine.test.a",
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
    static B_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.combine.test.b"),
        name: "motion.combine.test.b",
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
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            static A: Src = Src(&A_MAN, 9, 1.0);
            static B: Src = Src(&B_MAN, 4, 2.0);
            static C: MotionCombine = MotionCombine;
            match ty {
                t if t == A_MAN.id => Some(&A),
                t if t == B_MAN.id => Some(&B),
                t if t == MANIFEST.id => Some(&C),
                _ => None,
            }
        }
    }
    let run = |on: bool| {
        let mut g = Graph::new();
        let a = g.add_node("motion.combine.test.a");
        let b = g.add_node("motion.combine.test.b");
        let c = g.add_node("motion.combine");
        if on {
            g.set_param(c, "reindex", 1.0);
        }
        for (from, port) in [(a, 0u16), (b, 1)] {
            g.connect(Edge {
                from: (from, 0),
                to: (c, port),
                delayed: false,
            })
            .unwrap();
        }
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, c, 0.0).unwrap();
        scalars(out[0].as_stream(), "Index")
    };
    assert_eq!(run(false)[9], 0.0, "desligado, a 10ª linha reinicia em 0");
    assert_eq!(run(true)[9], 9.0, "ligado, ela é a décima da lista junta");
}

/// **LIGADO, O NÓ RECUSA O DEVICE** — a concatenação no device é um `StreamOp::Concat` que o
/// sequenciador faz com `copy_buffer_to_buffer`, sem shader; renumerar ali seria uma segunda
/// implementação da mesma lei.
///
/// ⚠️ **As duas metades**: desligado ele tem de CLAIMAR (senão a recusa custaria o device a
/// todo documento, não só aos que pedem a renumeração).
#[test]
fn the_kernel_recedes_when_the_renumbering_is_on() {
    let applicable = GPU_KERNEL.applicable.expect("o kernel declara a recusa");
    assert!(
        applicable(&|_| 0.0),
        "desligado: o device continua a claimar"
    );
    assert!(!applicable(&|n| if n == REINDEX { 1.0 } else { 0.0 }));
}
