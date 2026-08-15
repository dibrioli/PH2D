//! Gates do `value.pattern` — o NÓ (manifesto, kernel, registro e o cook).
//! A LEI da tabela mora no irmão `table_tests.rs`.

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};

/// **The pattern is assigned by index and cycles.** `steps = 3` over 7
/// instances repeats `v0,v1,v2` — `[a,b,c,a,b,c,a]`.
#[test]
fn the_pattern_cycles_by_index() {
    let vals = [10.0, 20.0, 30.0, 40.0, 0.0, 0.0, 0.0, 0.0];
    let got: Vec<f32> = (0..7).map(|i| pattern_value(i, 3, &vals)).collect();
    assert_eq!(got, vec![10.0, 20.0, 30.0, 10.0, 20.0, 30.0, 10.0]);
}

/// **`steps` is clamped to `[1, SLOTS]`** — `0` collapses to a constant `v0`,
/// and a value past the slots never indexes out of bounds.
#[test]
fn steps_is_clamped_to_the_slots() {
    let vals = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
    // steps 0 -> clamps to 1 -> every element is v0.
    assert!(
        (0..5).all(|i| pattern_value(i, 0, &vals) == 1.0),
        "0 steps is constant v0"
    );
    // steps beyond SLOTS clamps to SLOTS (uses all eight, never out of bounds).
    assert_eq!(pattern_value(8, 20, &vals), 1.0, "index 8 wraps to slot 0");
    assert_eq!(pattern_value(7, 20, &vals), 8.0, "the last slot is used");
}

/// **Every one of the eight slots is reachable** — `steps = 8` reads `v0…v7`
/// in order, so no slot is a dead param.
#[test]
fn all_eight_slots_are_reachable() {
    let vals = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8];
    let got: Vec<f32> = (0..SLOTS).map(|i| pattern_value(i, SLOTS, &vals)).collect();
    assert_eq!(got, vals.to_vec(), "all eight slots read in order");
}

/// The grid-like source: a count-only input, so `value.pattern` can produce a
/// field of length N through a real cook (it reads the count, never a `v`).
static SRC_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.pattern.test.src"),
    name: "value.pattern.test.src",
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
struct Src(usize);
impl NodeOp for Src {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        // A field of N zeros — only its LENGTH matters to `value.pattern`.
        ctx.emit(Stream::new(self.0).with(VALUE_COL, Column::Scalar(vec![0.0; self.0])));
    }
}

/// End-to-end through the cook: a length-5 input with `steps = 2`,
/// `v0 = 0, v1 = 1` produces `[0, 1, 0, 1, 0]` — the pattern authored by
/// param, keyed on the input's count.
#[test]
fn produces_the_pattern_through_the_cook() {
    struct Ops(usize);
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(Box::leak(Box::new(Src(self.0))) as &dyn NodeOp),
                t if t == MANIFEST.id => Some(&ValuePattern),
                _ => None,
            }
        }
    }
    let ops = Ops(5);
    let mut g = Graph::new();
    let src = g.add_node("value.pattern.test.src");
    let vp = g.add_node("value.pattern");
    g.set_param(vp, "steps", 2.0);
    g.set_param(vp, "v0", 0.0);
    g.set_param(vp, "v1", 1.0);
    g.connect(Edge {
        from: (src, 0),
        to: (vp, 0),
        delayed: false,
    })
    .unwrap();
    let mut cook = Cook::new();
    let out = cook.cook(&g, &ops, vp, 0.0).unwrap();
    match out[0].as_stream().get(VALUE_COL).unwrap() {
        Column::Scalar(v) => {
            assert_eq!(
                v,
                &vec![0.0, 1.0, 0.0, 1.0, 0.0],
                "alternating pattern, length 5"
            );
        }
        _ => panic!("v"),
    }
}

#[test]
fn registers_and_resolves() {
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    assert!(reg.resolve(MANIFEST.id).is_some());
}

/// Coze `n` elementos com os params/tabela dados, pelo caminho REAL.
fn cook_field(n: usize, table: Option<&str>, steps: f32, vals: &[f32; SLOTS]) -> Vec<f32> {
    struct Ops(usize);
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(Box::leak(Box::new(Src(self.0))) as &dyn NodeOp),
                t if t == MANIFEST.id => Some(&ValuePattern),
                _ => None,
            }
        }
    }
    let ops = Ops(n);
    let mut g = Graph::new();
    let src = g.add_node("value.pattern.test.src");
    let vp = g.add_node("value.pattern");
    g.set_param(vp, "steps", steps);
    for (k, v) in vals.iter().enumerate() {
        g.set_param(vp, format!("v{k}"), *v);
    }
    if let Some(t) = table {
        g.set_text_param(vp, TABLE_KEY, t);
    }
    g.connect(Edge {
        from: (src, 0),
        to: (vp, 0),
        delayed: false,
    })
    .unwrap();
    let mut cook = Cook::new();
    let out = cook.cook(&g, &ops, vp, 0.0).unwrap();
    match out[0].as_stream().get(VALUE_COL).unwrap() {
        Column::Scalar(v) => v.clone(),
        _ => panic!("v"),
    }
}

/// Oito valores que NÃO são a tabela — para a tabela ter o que vencer.
const SLOT_VALS: [f32; SLOTS] = [9.0, 8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0];

/// **A TABELA vence, e sem teto de oito** — onze valores ciclam através do cook.
///
/// ⚠️ **Os valores da tabela são FRACIONÁRIOS e os dos slots INTEIROS, de
/// propósito.** O controle abaixo pergunta *"o caminho legado contribuiu?"*, e
/// ele só sabe responder se os dois conjuntos forem **disjuntos** — a primeira
/// versão deste gate usou `"0 1 2 … 10"`, que CONTÉM os oito valores dos slots,
/// e o controle nasceu vácuo (reprovou sobre produto correto).
#[test]
fn the_authored_table_wins_and_is_not_capped_at_eight() {
    let text = "0.05 0.15 0.25 0.35 0.45 0.55 0.65 0.75 0.85 0.95 1.05";
    let table = ph2d_steps::parse(text);
    assert_eq!(table.len(), 11, "a fixture contem mais de oito passos");
    let got = cook_field(23, Some(text), 3.0, &SLOT_VALS);
    let want: Vec<f32> = (0..23).map(|i| table[i % 11]).collect();
    assert_eq!(got, want, "onze passos, ciclados — nao oito");
    // ⚠️ O CONTROLE: nenhum dos oito slots aparece na saida.
    assert!(
        !got.iter().any(|v| SLOT_VALS.contains(v)),
        "o caminho legado nao contribuiu"
    );
}

/// **Sem tabela o nó é o que shipava, elemento a elemento** — o ORÁCULO é o
/// `pattern_value`, que esta wave não toca.
///
/// ⚠️ Este é o gate que torna a wave uma ADIÇÃO e não uma mudança: o mundo de
/// antes é alcançado por AUSÊNCIA, não por um valor a digitar.
#[test]
fn with_no_table_the_node_is_the_one_that_shipped() {
    for steps in [1.0f32, 2.0, 3.0, 8.0] {
        let got = cook_field(19, None, steps, &SLOT_VALS);
        let want: Vec<f32> = (0..19)
            .map(|i| pattern_value(i, steps as usize, &SLOT_VALS))
            .collect();
        assert_eq!(got, want, "steps {steps}");
    }
}

/// **Uma tabela malformada cai no caminho legado** — e o painel volta a mostrar
/// os nove controles, então a queda é VISÍVEL em vez de silenciosa.
#[test]
fn a_malformed_table_falls_back_to_the_slots() {
    let legacy = cook_field(9, None, 3.0, &SLOT_VALS);
    for bad in ["", "   ", "0.1 oops 0.9", "nan"] {
        assert_eq!(cook_field(9, Some(bad), 3.0, &SLOT_VALS), legacy, "{bad:?}");
    }
}

/// **Autorar a tabela ESCONDE os nove controles legados** — eles ficam inertes
/// (o `eval` nem os lê), e um controle que não faz nada não é pintado.
#[test]
fn authoring_a_table_hides_the_nine_legacy_controls() {
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    let gates = reg
        .param_gates_text(MANIFEST.id)
        .expect("o no declara gates de texto");
    let gated: Vec<&str> = gates.iter().map(|g| g.param).collect();
    for p in ["steps", "v0", "v1", "v2", "v3", "v4", "v5", "v6", "v7"] {
        assert!(gated.contains(&p), "`{p}` tem de sumir com a tabela");
    }
    assert!(
        gates
            .iter()
            .all(|g| g.when_text == TABLE_KEY && !g.when_present),
        "os nove aparecem so na AUSENCIA da tabela"
    );
    // ⚠️ E o CONTROLE: a propria row da tabela NAO e gateada por si mesma, senao
    // ela desapareceria no instante em que fosse preenchida.
    assert!(!gated.contains(&TABLE_KEY));
}

/// **A row da tabela existe e é uma FAIXA DE PASSOS** — sem ela o canal seria
/// alcançável só por um grafo montado em código.
///
/// ⚠️ **E a faixa dela é a MESMA dos slots legados**, que é a metade que o gate existe
/// para pinar: os dois widgets desenham o mesmo número, então uma barra que lesse outra
/// faixa mostraria o padrão numa altura que o slider ao lado contradiz — o par de réguas
/// discordando sobre a mesma grandeza, e sem nada na tela dizendo qual está certa.
#[test]
fn the_table_row_is_a_step_strip_on_the_slots_own_range() {
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    let hints = reg.param_ui(MANIFEST.id).expect("hints");
    let row = hints
        .iter()
        .find(|h| h.param == TABLE_KEY)
        .expect("a row da tabela");
    assert!(matches!(row.widget, ParamWidget::Steps), "faixa de passos");
    let slot = hints
        .iter()
        .find(|h| h.param == "v0")
        .expect("o slider do slot 0");
    assert_eq!(
        (row.min, row.max),
        (slot.min, slot.max),
        "a faixa da barra é a do slider ao lado"
    );
}

/// **O kernel LÊ o buffer, nunca o `_sample`** — a decisão que mantém um passo
/// autorado exato (o acessor gerado LERPA entre vizinhos).
#[test]
fn the_kernel_reads_the_lut_buffer_and_never_lerps_it() {
    assert!(
        GPU_KERNEL.wgsl.contains("lut_vp_table[1u + (i % vp_n)]"),
        "indexa o buffer direto"
    );
    assert!(
        !GPU_KERNEL.wgsl.contains("vp_table_sample"),
        "e nunca passa pelo acessor que interpola"
    );
    // O canal esta registrado com a chave e a resolucao que a lei declara.
    assert_eq!(LUTS.len(), 1);
    assert_eq!(LUTS[0].text_key, TABLE_KEY);
    assert_eq!(LUTS[0].resolution, ph2d_steps::LUT_LEN);
}
