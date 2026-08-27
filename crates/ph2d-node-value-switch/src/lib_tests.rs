//! **AS PROVAS DO ROTEADOR** — irmão pelo teto de LOC (700), pelo idioma que a casa já usa
//! (`transform_tests.rs`, `lib_tests.rs` do `fx.glow`): o pai fica com a lei, este com o que a
//! afirma.

use super::*;
use ph2d_nodegraph::cook::OpResolver;

/// A length-1 `select` broadcasts: the WHOLE field switches together. `0`
/// picks `in0`, `1` picks `in1` — the common global-switch case.
#[test]
fn a_global_select_routes_the_whole_field() {
    let ins = vec![vec![10.0, 11.0], vec![20.0, 21.0], vec![], vec![]];
    assert_eq!(
        switch(&[0.0], &ins, false),
        vec![10.0, 11.0],
        "select 0 → in0"
    );
    assert_eq!(
        switch(&[1.0], &ins, false),
        vec![20.0, 21.0],
        "select 1 → in1"
    );
}

/// `select` rounds to the nearest input index (0.4 → 0, 0.6 → 1) and clamps
/// past the connected range (a huge/negative select never indexes out).
#[test]
fn select_rounds_to_nearest_and_clamps() {
    let ins = vec![vec![10.0], vec![20.0], vec![], vec![]];
    assert_eq!(
        switch(&[0.4], &ins, false),
        vec![10.0],
        "0.4 rounds down to in0"
    );
    assert_eq!(
        switch(&[0.6], &ins, false),
        vec![20.0],
        "0.6 rounds up to in1"
    );
    // clamp: N_INPUTS is 4, so index 3 is the top; 9.0 clamps to in3.
    let four = vec![vec![1.0], vec![2.0], vec![3.0], vec![4.0]];
    assert_eq!(
        switch(&[9.0], &four, false),
        vec![4.0],
        "9 clamps to the top input"
    );
    assert_eq!(
        switch(&[-5.0], &four, false),
        vec![1.0],
        "negative clamps to in0"
    );
}

/// FALSIFICATION of per-element routing: a length-N `select` picks a possibly
/// DIFFERENT input for each element — the Houdini per-point mux. Element 0
/// reads in0, element 1 reads in1.
#[test]
fn a_per_element_select_routes_each_element_independently() {
    let ins = vec![vec![10.0, 11.0], vec![20.0, 21.0], vec![], vec![]];
    // select [0, 1] → element 0 from in0 (10), element 1 from in1 (21).
    assert_eq!(
        switch(&[0.0, 1.0], &ins, false),
        vec![10.0, 21.0],
        "each element its own input"
    );
}

/// A length-1 source is HELD (broadcast) across a longer field — the `1→N`
/// rule reaches the routed inputs too.
#[test]
fn a_length_one_source_broadcasts_through_the_switch() {
    // in1 is a single global constant; select 1 over a 3-long select → held.
    let ins = vec![vec![], vec![7.0], vec![], vec![]];
    assert_eq!(switch(&[1.0, 1.0, 1.0], &ins, false), vec![7.0, 7.0, 7.0]);
}

/// End-to-end through the cook: two source fields and an animated select
/// (its own value node) route through the registry.
#[test]
fn routes_two_sources_through_the_cook() {
    use ph2d_nodegraph::cook::Cook;
    use ph2d_nodegraph::graph::{Edge, Graph};

    static SRC: NodeManifest = NodeManifest {
        id: NodeTypeId::of("value.switch.test.src"),
        name: "value.switch.test.src",
        inputs: &[],
        outputs: &[PortSpec {
            name: "out",
            ty: VALUE,
        }],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[ph2d_nodegraph::node::ParamSpec {
            name: "v",
            default: 0.0,
        }],
        lowerings: &[LoweringKind::Cpu],
    };
    struct Src;
    impl NodeOp for Src {
        fn manifest(&self) -> &'static NodeManifest {
            &SRC
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            let v = ctx.param("v");
            ctx.emit(Stream::new(1).with(VALUE_COL, Column::Scalar(vec![v])));
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC.id => Some(&Src),
                t if t == MANIFEST.id => Some(&ValueSwitch),
                _ => None,
            }
        }
    }
    let mut g = Graph::new();
    let sel = g.add_node("value.switch.test.src");
    let a = g.add_node("value.switch.test.src");
    let b = g.add_node("value.switch.test.src");
    let sw = g.add_node("value.switch");
    g.set_param(sel, "v", 1.0); // select in1
    g.set_param(a, "v", 100.0);
    g.set_param(b, "v", 200.0);
    for (from, port) in [(sel, 0), (a, 1), (b, 2)] {
        g.connect(Edge {
            from: (from, 0),
            to: (sw, port),
            delayed: false,
        })
        .unwrap();
    }
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, sw, 0.0).unwrap();
    match out[0].as_stream().get(VALUE_COL).unwrap() {
        Column::Scalar(v) => assert_eq!(v, &vec![200.0], "select 1 routed in1 (b)"),
        _ => panic!("v"),
    }
}

#[test]
fn registers_and_resolves() {
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    assert!(reg.resolve(MANIFEST.id).is_some());
}

/// **NUM SELECT INTEIRO, O CROSSFADER E' O ROTEADOR -- BIT-A-BIT.** O gate de
/// neutralidade: e' assim que um documento ja' autorado sobrevive ao knob novo.
#[test]
fn on_an_integer_select_blend_is_the_router_bit_for_bit() {
    let ins = vec![
        vec![10.0, 11.0, 12.0],
        vec![-20.0, 21.5, 22.0],
        vec![30.0],
        vec![40.0, 41.0, 42.0],
    ];
    for k in -2..=6 {
        let s = vec![k as f32];
        assert_eq!(
            switch(&s, &ins, true),
            switch(&s, &ins, false),
            "select={k}"
        );
    }
}

/// **DESLIGADO, O NO' SALTA; LIGADO, ELE DISSOLVE** -- e o gate mede a
/// diferenca no ponto onde as duas leis discordam ao maximo.
///
/// ⚠️ **A fixture escolhe o meio-inteiro de proposito.** Num select inteiro as
/// duas leis coincidem (o gate acima), entao um teste que so' amostrasse `0` e
/// `1` ficaria verde com o `blend` desligado por engano. Em `0.5` o roteador
/// arredonda PARA CIMA (metade para longe do zero) e le' `in1` inteiro; o
/// crossfader le' o ponto medio. A distancia entre as duas respostas e' meia
/// entrada, que e' precisamente o "pop" que o knob existe para apagar.
#[test]
fn a_half_integer_select_is_where_the_two_laws_disagree_most() {
    let ins = vec![vec![0.0], vec![100.0], vec![], vec![]];
    assert_eq!(switch(&[0.5], &ins, false), vec![100.0], "salta para in1");
    assert_eq!(switch(&[0.5], &ins, true), vec![50.0], "dissolve no meio");
    // E a dissolucao e' MONOTONA e cobre a distancia inteira -- a propriedade
    // que um pop nao tem.
    let mut prev = f32::NEG_INFINITY;
    for k in 0..=20 {
        let s = k as f32 / 20.0;
        let o = switch(&[s], &ins, true)[0];
        assert!(o >= prev, "select={s}: {o} < {prev}");
        prev = o;
    }
    assert_eq!(prev, 100.0, "chega inteiro na outra ponta");
}

/// **AS PONTAS SATURAM, nao dao a volta** -- a mesma lei de borda do roteador.
/// Um `select` fora da faixa le' a entrada da ponta, e nunca mistura `in3` com
/// `in0` (que e' o que um `%` faria).
#[test]
fn the_ends_saturate_they_do_not_wrap() {
    let four = vec![vec![1.0], vec![2.0], vec![3.0], vec![4.0]];
    assert_eq!(
        switch(&[3.7], &four, true),
        vec![4.0],
        "acima satura em in3"
    );
    assert_eq!(switch(&[-2.5], &four, true), vec![1.0], "abaixo em in0");
    assert_eq!(switch(&[9.0], &four, true), vec![4.0], "bem acima");
}

/// **E' PER-ELEMENTO tambem quando dissolve** -- a propriedade que o no' anuncia
/// no doc e que a mistura poderia ter perdido se lesse `select` uma vez so'.
#[test]
fn the_crossfade_is_per_element() {
    let ins = vec![vec![0.0, 0.0, 0.0], vec![10.0, 10.0, 10.0], vec![], vec![]];
    let out = switch(&[0.0, 0.25, 1.0], &ins, true);
    assert_eq!(out, vec![0.0, 2.5, 10.0]);
}

/// **A CONTAGEM DA SAÍDA DEPENDE DOS RAMOS NÃO ESCOLHIDOS — e é isto que impede a
/// avaliação preguiçosa de ser transparente** (doc 89, folha 15).
///
/// ⚠️ **Este é o terceiro perigo da célula da preguiça, e o desenho dela não o nomeava.**
/// Os dois que ele nomeava são sobre *quando* é legal saltar um ramo (o `select` pode ser um
/// campo por elemento; uma sub-árvore com estado congela se um tique não a cozinhar). Este é
/// sobre o que se perde ao saltar mesmo quando é legal: `n` é o **máximo** dos comprimentos
/// de TODAS as entradas mais o do `select` (ver [`switch`]), então um ramo comprido que
/// ninguém escolheu ainda decide **quantos** elementos saem — e o escolhido enche-os pela
/// regra 1→N do [`field_at`].
///
/// ⚠️ **E não há como saber isso sem cozinhar.** O `count_law` vive na maquinaria de GPU
/// (`ph2d_nodegraph::gpu`), que dimensiona *dispatches*; no caminho de CPU o comprimento de
/// um stream só existe depois de ele ser avaliado. ⇒ *saltar um ramo é, no caso geral, mudar
/// o que o nó computa* — e é por isso que a preguiça tem de ser um **MODO** declarado, com o
/// caminho de omissão byte-idêntico, e nunca uma optimização silenciosa do escalonador.
#[test]
fn the_output_count_is_decided_by_branches_nobody_chose() {
    // `select = 0` ⇒ o ramo 0 é o escolhido, e ele tem UM valor (a regra 1→N).
    // O ramo 3 tem oito, e ninguém o escolheu.
    let select = vec![0.0];
    let ins = vec![vec![7.0], Vec::new(), Vec::new(), vec![0.0; 8]];
    let out = switch(&select, &ins, false);
    assert_eq!(
        out.len(),
        8,
        "hoje o ramo 3 decide a contagem mesmo sem ser escolhido"
    );
    assert!(
        out.iter().all(|v| *v == 7.0),
        "e o valor e' o do ramo 0: {out:?}"
    );
    // A prova do contrário: sem aquele ramo, a saída tem UM elemento. Uma preguiça
    // transparente teria de devolver este, e não o de cima.
    let lean = vec![vec![7.0], Vec::new(), Vec::new(), Vec::new()];
    assert_eq!(switch(&select, &lean, false).len(), 1);
}

/// **O MODO DE MISTURA LÊ DOIS RAMOS** — o quarto facto que a preguiça tem de honrar.
///
/// Com `blend` ligado e um `select` fraccionário, o nó dissolve entre `floor` e `floor+1`:
/// saltar «tudo menos o escolhido» apagaria metade do resultado. ⚠️ E em `t == 0` o par
/// colapsa num só **por ramo** (o valor sai verbatim), então o número de ramos vivos depende
/// do VALOR do select, não só do modo.
#[test]
fn the_blend_mode_needs_the_pair_not_the_one() {
    let ins = vec![vec![0.0], vec![10.0], vec![20.0], vec![30.0]];
    // select 1,5 ⇒ metade do ramo 1 e metade do 2.
    assert_eq!(switch(&[1.5], &ins, true), vec![15.0]);
    // select 1,0 ⇒ so' o ramo 1, verbatim (o `t == 0` devolve `a` sem tocar em `b`).
    assert_eq!(switch(&[1.0], &ins, true), vec![10.0]);
}
