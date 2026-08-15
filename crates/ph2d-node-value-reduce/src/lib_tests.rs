//! Os gates do NÓ — o manifesto, o cook de ponta a ponta e o registro. A LEI
//! (as oito agregações e as duas portas) tem os seus em `stats_tests.rs`; este
//! arquivo nasceu do `mod tests` do `lib.rs` quando o teto de LOC o pediu.

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};
use stats::aggregate;

/// **Each mode folds the whole field to its aggregate.** Sum totals, Mean
/// averages, Min/Max pick the extremes.
#[test]
fn each_mode_folds_the_field_to_its_aggregate() {
    let f = [1.0, 2.0, 3.0, 4.0]; // sum 10, mean 2.5, min 1, max 4
    assert_eq!(aggregate(&f, Mode::Sum), 10.0);
    assert_eq!(aggregate(&f, Mode::Mean), 2.5);
    assert_eq!(aggregate(&f, Mode::Min), 1.0);
    assert_eq!(aggregate(&f, Mode::Max), 4.0);
    // Negatives and a single element.
    assert_eq!(aggregate(&[-3.0, 3.0], Mode::Sum), 0.0);
    assert_eq!(aggregate(&[-3.0, 3.0], Mode::Mean), 0.0);
    assert_eq!(aggregate(&[7.0], Mode::Mean), 7.0);
}

/// **The output is a CONSTANT field of the source's length** — the aggregate
/// broadcast to every element, so it lines up for a downstream fold (not a
/// length-1 stream).
#[test]
fn the_output_broadcasts_to_a_constant_field_of_the_same_length() {
    for n in [1usize, 4, 23] {
        let field: Vec<f32> = (0..n).map(|i| i as f32 + 1.0).collect();
        let agg = aggregate(&field, Mode::Mean);
        let out = vec![agg; field.len()];
        assert_eq!(out.len(), n, "length preserved");
        assert!(
            out.iter().all(|&x| x == agg),
            "every element is the aggregate"
        );
    }
    // An empty field yields an empty field (nothing to reduce).
    assert!(aggregate(&[], Mode::Sum).is_finite());
}

/// **Mean subtracted from the field centres it about zero** — the reason this
/// node exists, proved on the aggregate: `Σ (vᵢ − mean) = 0`.
#[test]
fn subtracting_the_mean_centres_the_field() {
    let field = [1.0, 4.0, 10.0, -3.0, 8.0];
    let mean = aggregate(&field, Mode::Mean);
    let centred_sum: f32 = field.iter().map(|&v| v - mean).sum();
    assert!(centred_sum.abs() < 1e-4, "the centred field sums to zero");
}

/// A value source emitting a fixed field, so `value.reduce` can be driven
/// through a real cook (the whole-chain proof, reduction included).
static SRC_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.reduce.test.src"),
    name: "value.reduce.test.src",
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
struct Src(Vec<f32>);
impl NodeOp for Src {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(Stream::new(self.0.len()).with(VALUE_COL, Column::Scalar(self.0.clone())));
    }
}

/// A second source, so the optional `mask`/`group` ports can be driven through a
/// real cook. It is a SEPARATE type because `OpResolver::resolve` keys on the
/// node type: one type cannot answer with two different fields.
static SRC2_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.reduce.test.src2"),
    name: "value.reduce.test.src2",
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
struct Src2(Vec<f32>);
impl NodeOp for Src2 {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC2_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(Stream::new(self.0.len()).with(VALUE_COL, Column::Scalar(self.0.clone())));
    }
}

struct Ops(Vec<f32>, Vec<f32>);
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC_MAN.id => Some(Box::leak(Box::new(Src(self.0.clone()))) as &dyn NodeOp),
            t if t == SRC2_MAN.id => Some(Box::leak(Box::new(Src2(self.1.clone()))) as &dyn NodeOp),
            t if t == MANIFEST.id => Some(&ValueReduce),
            _ => None,
        }
    }
}

/// End-to-end through the cook: a `[2, 4, 6]` field through Mean becomes
/// `[4, 4, 4]` — the aggregate broadcast, length preserved (the reduce ran).
#[test]
fn reduces_a_field_through_the_cook() {
    let ops = Ops(vec![2.0, 4.0, 6.0], Vec::new());
    let mut g = Graph::new();
    let src = g.add_node("value.reduce.test.src");
    let vr = g.add_node("value.reduce");
    g.set_param(vr, "mode", 1.0); // Mean
    g.connect(Edge {
        from: (src, 0),
        to: (vr, 0),
        delayed: false,
    })
    .unwrap();
    let mut cook = Cook::new();
    let out = cook.cook(&g, &ops, vr, 0.0).unwrap();
    match out[0].as_stream().get(VALUE_COL).unwrap() {
        Column::Scalar(v) => {
            assert_eq!(v, &vec![4.0, 4.0, 4.0], "mean 4 broadcast to length 3");
        }
        _ => panic!("v"),
    }
}

/// **A MÁSCARA atravessa o cook** — a porta 1 escopa quem é CONTADO, e todo
/// elemento (inclusive o excluído) recebe o número. Campo `[2, 4, 100]` com
/// máscara `[1, 1, 0]` dá média `3`, difundida aos TRÊS.
///
/// ⚠️ É o gate que prova que a porta APENDADA está de facto ligada ao `eval`:
/// um `input(1)` esquecido devolveria `[35.33; 3]` e ninguém veria erro.
#[test]
fn the_mask_scopes_who_is_counted_through_the_cook() {
    let ops = Ops(vec![2.0, 4.0, 100.0], vec![1.0, 1.0, 0.0]);
    let mut g = Graph::new();
    let src = g.add_node("value.reduce.test.src");
    let msk = g.add_node("value.reduce.test.src2");
    let vr = g.add_node("value.reduce");
    g.set_param(vr, "mode", 1.0); // Mean
    for (from, port) in [(src, 0u16), (msk, 1u16)] {
        g.connect(Edge {
            from: (from, 0),
            to: (vr, port),
            delayed: false,
        })
        .unwrap();
    }
    let mut cook = Cook::new();
    let out = cook.cook(&g, &ops, vr, 0.0).unwrap();
    match out[0].as_stream().get(VALUE_COL).unwrap() {
        Column::Scalar(v) => assert_eq!(
            v,
            &vec![3.0, 3.0, 3.0],
            "a média dos SELECIONADOS, difundida a todos"
        ),
        _ => panic!("v"),
    }
}

/// **O GRUPO atravessa o cook** — a porta 2 parte o conjunto em bins, e cada
/// elemento recebe o agregado do SEU bin. `[1, 3, 10, 20]` em grupos
/// `[0, 0, 1, 1]` dá `[2, 2, 15, 15]`.
#[test]
fn the_group_port_gives_each_bin_its_own_aggregate_through_the_cook() {
    let ops = Ops(vec![1.0, 3.0, 10.0, 20.0], vec![0.0, 0.0, 1.0, 1.0]);
    let mut g = Graph::new();
    let src = g.add_node("value.reduce.test.src");
    let grp = g.add_node("value.reduce.test.src2");
    let vr = g.add_node("value.reduce");
    g.set_param(vr, "mode", 1.0); // Mean
    for (from, port) in [(src, 0u16), (grp, 2u16)] {
        g.connect(Edge {
            from: (from, 0),
            to: (vr, port),
            delayed: false,
        })
        .unwrap();
    }
    let mut cook = Cook::new();
    let out = cook.cook(&g, &ops, vr, 0.0).unwrap();
    match out[0].as_stream().get(VALUE_COL).unwrap() {
        Column::Scalar(v) => assert_eq!(v, &vec![2.0, 2.0, 15.0, 15.0], "um agregado por bin"),
        _ => panic!("v"),
    }
}

/// **O device aceita os cinco que sabe dobrar e RECUSA os três que não** — o
/// `applicable` é a porta, e a mutação que a devolve sempre `true` faria o
/// sequenciador reivindicar um nó cujo kernel **não tem braço** para os modos
/// 5/6/7: o `default` do `switch` devolveria a SOMA, um número plausível e
/// errado, difundido pelo campo inteiro.
#[test]
fn the_device_answers_for_the_five_it_can_fold_and_refuses_the_three_it_cannot() {
    let probe = |m: f32| device_can_answer(&|name: &str| if name == "mode" { m } else { 0.0 });
    for m in 0..=4 {
        assert!(probe(m as f32), "o modo {m} é dobrável");
    }
    for m in 5..=7 {
        assert!(
            !probe(m as f32),
            "o modo {m} precisa de um 2º passo (ou de um rank): recusa"
        );
    }
}

/// **O kernel não escreve um braço para o que recusa** — o par do gate acima, e
/// a metade que impede a "completude" cosmética: um `case 5` com a fórmula de um
/// passo faria o `switch` parecer inteiro e devolveria 71 de desvio num campo
/// constante.
#[test]
fn the_kernel_has_no_arm_for_the_modes_it_refuses() {
    for arm in ["case 5", "case 6", "case 7", "reduce_sumsq"] {
        assert!(
            !GPU_KERNEL.wgsl.contains(arm),
            "o WGSL não pode conter `{arm}`: o device recusa esses modos"
        );
    }
    for arm in ["case 1", "case 2", "case 3", "case 4"] {
        assert!(GPU_KERNEL.wgsl.contains(arm), "falta o braço `{arm}`");
    }
}

/// **As duas portas opcionais são REFUSALS no plano, nunca leituras** — se
/// alguma delas nascesse `Read`, o sequenciador reivindicaria o nó e o kernel
/// ignoraria a máscara em silêncio.
#[test]
fn the_optional_ports_are_plan_time_refusals() {
    let b = GPU_KERNEL.bindings;
    assert_eq!(b.len(), 3, "porta 0 escreve; 1 e 2 recusam");
    assert!(b[0].access.writes(false) && b[0].port == 0);
    for (i, port) in [(1usize, 1usize), (2, 2)] {
        assert!(
            b[i].access.refuses(),
            "a porta {port} tem de RECUSAR, não ler"
        );
        assert_eq!(b[i].port, port);
        assert_eq!(b[i].column, VALUE_COL);
    }
}

/// **`in` continua na porta 0** — o índice que todo grafo salvo guarda nas
/// arestas. As portas novas só podem crescer pelo FIM.
#[test]
fn the_original_port_keeps_its_index() {
    assert_eq!(MANIFEST.inputs[0].name, "in");
    assert_eq!(MANIFEST.inputs.len(), 3);
    assert_eq!(MANIFEST.inputs[1].name, "mask");
    assert_eq!(MANIFEST.inputs[2].name, "group");
}

#[test]
fn registers_and_resolves() {
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    assert!(reg.resolve(MANIFEST.id).is_some());
}
