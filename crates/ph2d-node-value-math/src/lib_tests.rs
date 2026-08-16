//! Os gates do `value.math` — a suíte que mora ao lado do motor.
//!
//! ⚠️ FILHO por `#[path]`, nunca irmão: `use super::*` tem de alcançar `combine`, `Op`,
//! `scalar_col` e `field_at`, que são privados de propósito. Saiu do `lib.rs` porque ele
//! bateu **938 > 700** com os seis modos de comparação do grupo E — e ⚠️ **isso shipou
//! VERMELHO-LATENTE**: o gate mora na `ph2d-editor-core`, então um fechamento por
//! `cargo test -p ph2d-node-value-math` **não o alcança**. É a mesma causa estrutural que
//! esta linha já documentou quatro vezes.

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};

// Direct unit tests of the core (no cook needed for the arithmetic).

/// Each op folds a pair the way its name says — the reference-convergent
/// combine vocabulary (TD Math CHOP).
#[test]
fn every_op_combines_the_pair_as_named() {
    assert_eq!(Op::Add.apply(2.0, 3.0, 0.0), 5.0);
    assert_eq!(Op::Subtract.apply(2.0, 3.0, 0.0), -1.0);
    assert_eq!(Op::Multiply.apply(2.0, 3.0, 0.0), 6.0);
    assert_eq!(Op::Divide.apply(6.0, 3.0, 0.0), 2.0);
    assert_eq!(Op::Min.apply(2.0, 3.0, 0.0), 2.0);
    assert_eq!(Op::Max.apply(2.0, 3.0, 0.0), 3.0);
}

/// **Cada comparação dobra o par numa MÁSCARA 0/1** — as seis, na ordem da
/// referência, e o resultado é EXATAMENTE `0.0` ou `1.0`.
///
/// ⚠️ A metade do `is_comparison` não é cerimônia: uma máscara é o que cinco
/// famílias deste grafo consomem (a §5 do CLAUDE.md nomeia-as), e um `0.999`
/// no lugar de `1.0` não falha em lugar nenhum — ele **dilui** o que quer que
/// a leia, em silêncio.
#[test]
fn every_comparison_folds_the_pair_into_a_zero_or_one_mask() {
    assert_eq!(Op::Less.apply(1.0, 2.0, 0.0), 1.0);
    assert_eq!(Op::Less.apply(2.0, 2.0, 0.0), 0.0);
    assert_eq!(Op::LessOrEqual.apply(2.0, 2.0, 0.0), 1.0);
    assert_eq!(Op::Greater.apply(3.0, 2.0, 0.0), 1.0);
    assert_eq!(Op::Greater.apply(2.0, 2.0, 0.0), 0.0);
    assert_eq!(Op::GreaterOrEqual.apply(2.0, 2.0, 0.0), 1.0);
    assert_eq!(Op::Equal.apply(2.0, 2.0, 0.0), 1.0);
    assert_eq!(Op::NotEqual.apply(2.0, 3.0, 0.0), 1.0);
    // E TODA saída de comparação é um dos dois literais, nunca um número perto
    // deles: varrido sobre pares que cruzam a fronteira nas duas direções.
    for op in [
        Op::Less,
        Op::LessOrEqual,
        Op::Greater,
        Op::GreaterOrEqual,
        Op::Equal,
        Op::NotEqual,
    ] {
        assert!(op.is_comparison(), "a porta única concorda com a lista");
        for (a, b) in [(-3.0, 2.0), (2.0, 2.0), (7.5, 2.0), (0.0, 0.0)] {
            let m = op.apply(a, b, 0.25);
            assert!(m == 0.0 || m == 1.0, "máscara {m} para ({a}, {b})");
        }
    }
    // E os oito aritméticos NÃO são comparações — a porta separa as famílias.
    for op in [Op::Add, Op::Divide, Op::Max, Op::FlooredModulo] {
        assert!(!op.is_comparison());
    }
}

/// **A igualdade é COM tolerância, e a tolerância é LIDA.**
///
/// Falsificável nas duas direções: um kernel que ignorasse `eps` daria a mesma
/// máscara para 0,001 e para 0,5, e um que o lesse com o sinal trocado
/// inverteria a banda.
#[test]
fn equality_is_within_a_tolerance_and_the_tolerance_is_read() {
    // Duas amostras que a igualdade EXATA separa e uma tolerância junta.
    assert_eq!(
        Op::Equal.apply(1.0, 1.05, 0.0),
        0.0,
        "eps 0 = igualdade exata"
    );
    assert_eq!(Op::Equal.apply(1.0, 1.05, 0.1), 1.0, "dentro da banda");
    assert_eq!(Op::Equal.apply(1.0, 1.2, 0.1), 0.0, "fora da banda");
    // A fronteira é FECHADA (`<=`), o que torna `eps = 0` a igualdade exata em
    // vez de "nunca igual" — um `<` ali faria `Equal` com eps 0 ser sempre 0.
    //
    // ⚠️ A fixture usa 1,25 e 0,25 — **potências de dois, exatas em binário** —
    // e não 1,1 e 0,1. Este gate nasceu VERMELHO com aqueles: `(1.0f32 −
    // 1.1).abs()` vale `0.100000024`, que é MAIOR que `0.1`, então o teste
    // media a representação do decimal e não o predicado. *Um gate de
    // FRONTEIRA precisa de operandos cuja diferença seja exatamente o número
    // que ele afirma.*
    assert_eq!(
        Op::Equal.apply(1.0, 1.25, 0.25),
        1.0,
        "a fronteira pertence"
    );
    assert_eq!(
        Op::Equal.apply(1.0, 1.0, 0.0),
        1.0,
        "eps 0 é igualdade exata"
    );
    // E `Not Equal` é o COMPLEMENTO exato nos finitos.
    for (a, b, e) in [(1.0, 1.05, 0.1), (1.0, 1.2, 0.1), (0.0, 0.0, 0.0)] {
        assert_eq!(
            Op::Equal.apply(a, b, e) + Op::NotEqual.apply(a, b, e),
            1.0,
            "({a}, {b}, {e}): as duas máscaras particionam"
        );
    }
}

/// **`Not Equal` é a comparação DIRETA, não a negação de `Equal`** — as duas
/// formas só divergem no NaN, e é lá que o gate mede.
///
/// ⚠️ O ponto não é o NaN em si (o guard do divisor existe justamente para
/// nenhum chegar aqui): é que a forma escrita tem de ser a MESMA dos dois lados
/// da fronteira CPU/WGSL. Uma negação em Rust contra um `>` no device
/// discordaria exatamente aqui, e nada mais no grafo mudaria.
#[test]
fn not_equal_is_the_direct_comparison_not_the_negation() {
    let nan = f32::NAN;
    assert_eq!(Op::Equal.apply(nan, 0.0, 0.1), 0.0);
    assert_eq!(
        Op::NotEqual.apply(nan, 0.0, 0.1),
        0.0,
        "toda comparação com NaN é falsa — a negação daria 1.0 aqui"
    );
    // Idem para a ordem: nenhuma das quatro é verdadeira sobre um NaN.
    for op in [Op::Less, Op::LessOrEqual, Op::Greater, Op::GreaterOrEqual] {
        assert_eq!(op.apply(nan, 0.0, 0.0), 0.0);
        assert_eq!(op.apply(0.0, nan, 0.0), 0.0);
    }
}

/// **A tolerância não vaza para as comparações de ORDEM** — um `>` não tem do
/// que ser tolerante, e um epsilon enorme não pode mover a fronteira dele.
///
/// ⚠️ **A fixture tem de conter pares dos DOIS lados de cada predicado**, e esta
/// nasceu sem: os três primeiros pares têm todos `a <= b`, e sobre eles a
/// mutação `a > b + eps` devolve *falso* com qualquer epsilon — **verde sobre o
/// defeito exato que este gate persegue**. Um epsilon somado só muda a resposta
/// onde a comparação era VERDADEIRA; sem um par com `a > b` não há o que mudar.
#[test]
fn the_order_comparisons_ignore_the_tolerance() {
    for op in [Op::Less, Op::LessOrEqual, Op::Greater, Op::GreaterOrEqual] {
        for (a, b) in [
            (1.0, 1.05),
            (2.0, 2.0),
            (-3.0, 7.0),
            // …e os dois que faltavam: `a > b`, onde um epsilon somado morde.
            (7.0, 2.0),
            (-3.0, -7.0),
        ] {
            assert_eq!(
                op.apply(a, b, 0.0),
                op.apply(a, b, 1e6),
                "({a}, {b}): a ordem não lê a tolerância"
            );
        }
    }
}

/// **Os oito ops ARITMÉTICOS não são tocados pelo param novo** — o campo
/// `epsilon` entrou no manifesto e nenhum documento já autorado muda um bit.
///
/// ⚠️ É a metade que torna a wave segura: apender um param a um `NodeManifest`
/// é aditivo por construção, mas *ser aditivo* e *não ser lido* são coisas
/// diferentes, e só a segunda é o que um grafo salvo precisa.
#[test]
fn the_arithmetic_ops_are_untouched_by_the_new_epsilon() {
    for op in [
        Op::Add,
        Op::Subtract,
        Op::Multiply,
        Op::Divide,
        Op::Min,
        Op::Max,
        Op::Modulo,
        Op::FlooredModulo,
    ] {
        for (a, b) in [(7.0, 3.0), (-7.0, 3.0), (2.5, -0.75), (5.0, 0.0)] {
            assert_eq!(
                op.apply(a, b, 0.0).to_bits(),
                op.apply(a, b, 999.0).to_bits(),
                "({a}, {b}): a aritmética é surda ao epsilon, BIT A BIT"
            );
        }
    }
}

/// **Os índices 0..7 continuam a significar o que significavam** — as seis
/// comparações são APENDADAS, e um documento salvo com `op = 5` ainda é `Max`.
#[test]
fn the_comparisons_are_appended_so_every_authored_op_still_means_what_it_meant() {
    let core = [
        Op::Add,
        Op::Subtract,
        Op::Multiply,
        Op::Divide,
        Op::Min,
        Op::Max,
        Op::Modulo,
        Op::FlooredModulo,
    ];
    for (i, want) in core.into_iter().enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "i <= 7")]
        let got = Op::from_param(i as f32);
        assert!(got == want, "o índice {i} mudou de significado");
    }
    // E os seis novos ocupam 8..13, na ordem da referência.
    let new = [
        Op::Less,
        Op::LessOrEqual,
        Op::Greater,
        Op::GreaterOrEqual,
        Op::Equal,
        Op::NotEqual,
    ];
    for (k, want) in new.into_iter().enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "k <= 5")]
        let got = Op::from_param((8 + k) as f32);
        assert!(got == want, "o índice {} não é o esperado", 8 + k);
    }
}

/// **O `epsilon` é pintado sob as duas igualdades e sob mais nada**, e a
/// expectativa é DERIVADA do enum em vez de escrita à mão: uma sétima
/// comparação que esquecesse o gate sangra aqui.
#[test]
fn epsilon_is_painted_only_under_the_two_equality_ops() {
    let gate = PARAM_GATES
        .iter()
        .find(|g| g.param == "epsilon")
        .expect("o epsilon é gateado");
    assert_eq!(gate.when, "op");
    for i in 0..=13i32 {
        #[expect(clippy::cast_precision_loss, reason = "i <= 13")]
        let op = Op::from_param(i as f32);
        let reads = matches!(op, Op::Equal | Op::NotEqual);
        assert_eq!(
            gate.values.contains(&i),
            reads,
            "o índice {i} {} o epsilon, e a tabela de gate discorda",
            if reads { "LÊ" } else { "não lê" }
        );
    }
}

/// FALSIFICATION of the divide guard: dividing by a (near-)zero divisor
/// collapses to `0.0` — a downstream field never sees `inf`/`NaN`. An
/// unguarded `a / 0.0` would be non-finite and poison the whole graph.
#[test]
fn divide_by_zero_collapses_to_zero_not_infinity() {
    let q = Op::Divide.apply(5.0, 0.0, 0.0);
    assert!(q.is_finite(), "guarded: finite, not inf/NaN");
    assert_eq!(q, 0.0, "collapses to 0");
    // A divisor below the epsilon is treated as zero too.
    assert_eq!(Op::Divide.apply(5.0, 1e-12, 0.0), 0.0);
    // A real divisor still divides.
    assert_eq!(Op::Divide.apply(5.0, 2.0, 0.0), 2.5);
}

/// The broadcast rule (doc 12): a length-1 field is HELD at every index of a
/// length-N field — the whole point of the combiner, and what makes
/// `gradient(N) × global(1)` one wire. Falsifiable: an element-wise-only
/// implementation would read `b` past its single entry as 0 and multiply the
/// tail to 0.
#[test]
fn a_length_one_field_broadcasts_across_a_length_n_field() {
    // gradient [0, 0.5, 1] × global 2.0 → [0, 1, 2] (b broadcast to all 3).
    let out = combine(&[0.0, 0.5, 1.0], &[2.0], Op::Multiply, 0.0);
    assert_eq!(out, vec![0.0, 1.0, 2.0], "the single b held at every index");
    // Symmetric: a length-1 `a` broadcasts against a length-N `b`.
    let out = combine(&[10.0], &[1.0, 2.0, 3.0], Op::Add, 0.0);
    assert_eq!(
        out,
        vec![11.0, 12.0, 13.0],
        "the single a held at every index"
    );
}

/// Two equal-length fields combine element-wise, length preserved.
#[test]
fn two_length_n_fields_combine_element_wise() {
    let out = combine(&[1.0, 2.0, 3.0], &[10.0, 20.0, 30.0], Op::Add, 0.0);
    assert_eq!(out, vec![11.0, 22.0, 33.0]);
}

/// A disconnected (empty) input reads as the zero field: `a + {} = a`
/// (additive identity passthrough), while `a × {} = 0` (the documented
/// consequence of the zero degenerate field). The output still tracks the
/// connected input's length.
#[test]
fn a_disconnected_input_is_the_zero_field() {
    assert_eq!(
        combine(&[1.0, 2.0], &[], Op::Add, 0.0),
        vec![1.0, 2.0],
        "add: passthrough of the connected field"
    );
    assert_eq!(
        combine(&[1.0, 2.0], &[], Op::Multiply, 0.0),
        vec![0.0, 0.0],
        "multiply: the zero field collapses it"
    );
    // Both empty → empty (no field at all).
    assert!(combine(&[], &[], Op::Add, 0.0).is_empty());
}

// Two value sources with distinct type ids (the `motion.drive` two-source
// harness): field `a` is length-3, field `b` is length-1, so the cook sees
// the broadcast max.
static SRC_A_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.math.test.a"),
    name: "value.math.test.a",
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
static SRC_B_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.math.test.b"),
    name: "value.math.test.b",
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
struct SrcA;
impl NodeOp for SrcA {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC_A_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(Stream::new(3).with(VALUE_COL, Column::Scalar(vec![0.0, 0.5, 1.0])));
    }
}
struct SrcB;
impl NodeOp for SrcB {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC_B_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(Stream::new(1).with(VALUE_COL, Column::Scalar(vec![2.0])));
    }
}
struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC_A_MAN.id => Some(&SrcA),
            t if t == SRC_B_MAN.id => Some(&SrcB),
            t if t == MANIFEST.id => Some(&ValueMath),
            _ => None,
        }
    }
}

/// End-to-end through the cook: a length-3 gradient `a` and a length-1 global
/// `b` are multiplied, and the output is the broadcast max (length 3) —
/// exactly the `instance_field × lfo` shape the boot scene wires.
#[test]
fn combines_two_value_sources_through_the_cook() {
    let mut g = Graph::new();
    let a = g.add_node("value.math.test.a");
    let b = g.add_node("value.math.test.b");
    let m = g.add_node("value.math");
    g.connect(Edge {
        from: (a, 0),
        to: (m, 0),
        delayed: false,
    })
    .unwrap();
    g.connect(Edge {
        from: (b, 0),
        to: (m, 1),
        delayed: false,
    })
    .unwrap();
    g.set_param(m, "op", 2.0); // Multiply
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, m, 0.0).unwrap();
    match out[0].as_stream().get(VALUE_COL).unwrap() {
        Column::Scalar(v) => assert_eq!(v, &vec![0.0, 1.0, 2.0], "length-3 × broadcast 2"),
        _ => panic!("v"),
    }
}

#[test]
fn registers_and_resolves() {
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    assert!(reg.resolve(MANIFEST.id).is_some());
}

/// **Os dois modulos diferem no SINAL que seguem, e e' essa a razao de os
/// dois existirem.** O truncado segue o DIVIDENDO (`-7 mod 3 = -1`, o `%` do
/// C/Houdini), o aterrado segue o DIVISOR (`= 2`, o `%` do Python / o `mod`
/// do GLSL). Um modulo so' obrigaria metade dos usos a uma cadeia de
/// correcao de sinal -- e acima de zero eles COINCIDEM, que e' por que a
/// fixture tem de descer abaixo dele.
#[test]
fn the_two_moduli_differ_by_the_sign_they_follow() {
    assert_eq!(Op::Modulo.apply(7.0, 3.0, 0.0), 1.0);
    assert_eq!(
        Op::FlooredModulo.apply(7.0, 3.0, 0.0),
        1.0,
        "acima de zero os dois coincidem"
    );
    assert_eq!(Op::Modulo.apply(-7.0, 3.0, 0.0), -1.0, "sinal do DIVIDENDO");
    assert_eq!(
        Op::FlooredModulo.apply(-7.0, 3.0, 0.0),
        2.0,
        "sinal do DIVISOR"
    );
    // Divisor negativo: o aterrado o segue, o truncado nao.
    assert_eq!(Op::Modulo.apply(7.0, -3.0, 0.0), 1.0);
    assert_eq!(Op::FlooredModulo.apply(7.0, -3.0, 0.0), -2.0);
}

/// **O aterrado aterra em `[0, b)` para todo `b > 0`** -- a propriedade que
/// faz dele o modulo que alguem quer ao escrever *"repita a cada N"*, e que
/// um ponto isolado nao afirma.
///
/// A tolerancia de 1e-6 e' honesta e nao folga: o resultado e' `a - b·k`, e
/// para um `a/b` que roda para logo abaixo de um inteiro a subtracao pode
/// devolver um negativo do tamanho de um ulp de `a`.
#[test]
fn the_floored_modulo_wraps_into_the_half_open_range() {
    let b = 0.75_f32;
    for k in -40..40 {
        let a = k as f32 * 0.13;
        let m = Op::FlooredModulo.apply(a, b, 0.0);
        assert!(m > -1e-6 && m < b + 1e-6, "a={a} -> {m}, fora de [0,{b})");
    }
}

/// FALSIFICACAO da guarda: um divisor (quase) nulo colapsa em `0.0` nos DOIS
/// modulos -- eles dividem, entao herdam a guarda do Divide, e um campo a
/// jusante nunca ve' `inf`/`NaN`.
#[test]
fn a_zero_divisor_collapses_both_moduli() {
    for op in [Op::Modulo, Op::FlooredModulo] {
        assert_eq!(op.apply(5.0, 0.0, 0.0), 0.0);
        assert_eq!(op.apply(5.0, 1e-12, 0.0), 0.0);
        assert!(op.apply(5.0, 0.0, 0.0).is_finite());
    }
}
