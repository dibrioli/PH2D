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
    let table = table::parse(text);
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

/// **A row da tabela existe e é um campo de TEXTO** — sem ela o canal seria
/// alcançável só por um grafo montado em código.
#[test]
fn the_table_has_a_text_row_on_the_panel() {
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    let hints = reg.param_ui(MANIFEST.id).expect("hints");
    let row = hints
        .iter()
        .find(|h| h.param == TABLE_KEY)
        .expect("a row da tabela");
    assert!(matches!(row.widget, ParamWidget::Text), "campo de texto");
}

/// **O kernel LÊ o buffer, nunca o `_sample`** — a decisão que mantém um passo
/// autorado exato (o acessor gerado LERPA entre vizinhos).
///
/// ⚠️ **A lei sobreviveu à chegada do `interp = Linear`, e a distinção é o ponto:**
/// o `Linear` interpola entre dois **SLOTS**, porque o artista pediu; o
/// `vp_table_sample` interpolaria entre duas **AMOSTRAS DA LUT**, que é uma
/// reamostragem que ninguém pediu e que tornaria inexato até o modo `Step`. Os
/// dois são lerps e não são a mesma coisa — é por isso que este gate continua a
/// proibir só um deles.
///
/// ⚠️ E ele varre `wgsl` **e** `wgsl_lib`: a indexação mudou de lugar quando o
/// `vp_at` nasceu, e um gate que só olhasse o corpo teria ficado verde sobre um
/// kernel que passasse a lerpar a LUT dentro da biblioteca.
#[test]
fn the_kernel_reads_the_lut_buffer_and_never_lerps_it() {
    let src = format!("{}\n{}", GPU_KERNEL.wgsl, GPU_KERNEL.wgsl_lib);
    assert!(
        src.contains("lut_vp_table[1u + u32(idx)]"),
        "indexa o buffer direto"
    );
    assert!(
        !src.contains("vp_table_sample"),
        "e nunca passa pelo acessor que interpola"
    );
    // O canal esta registrado com a chave e a resolucao que a lei declara.
    assert_eq!(LUTS.len(), 1);
    assert_eq!(LUTS[0].text_key, TABLE_KEY);
    assert_eq!(LUTS[0].resolution, table::LUT_LEN);
}

/// **`offset = 0` É O NÓ QUE SEMPRE SHIPOU — BIT-A-BIT, E NOS DOIS MODOS.**
///
/// ⚠️ A metade que importa é o `Linear`: `round(i)` ser `i` é óbvio, mas em
/// `Linear` a resposta passa pelo ramo da fração, e é o `t == 0 ⇒ verbatim` que
/// impede `a + 0·(b − a)` de arredondar o default do nó.
#[test]
fn a_zero_offset_is_the_node_that_shipped_in_both_modes() {
    let vals = SLOT_VALS;
    for &authored in &[&[][..], &[3.5, -1.0, 7.25][..]] {
        for steps in 1..=SLOTS {
            for i in 0..20 {
                let want =
                    table::value_at(i, authored).unwrap_or_else(|| pattern_value(i, steps, &vals));
                for interp in [Interp::Step, Interp::Linear] {
                    assert_eq!(
                        phased_value(i, 0.0, interp, steps, &vals, authored),
                        want,
                        "i={i} steps={steps} {interp:?}"
                    );
                }
            }
        }
    }
}

/// **UM OFFSET INTEIRO ROLA O PADRÃO, e dá a volta nos DOIS sentidos.**
///
/// ⚠️ **A mutação que este gate mata é escrever `%` em vez de `rem_euclid`.** O
/// `%` de Rust devolve negativo para um índice negativo, e um índice negativo é o
/// caso *normal* deste knob (o artista arrasta o slider para a esquerda). O
/// resultado seria um pânico de indexação ou — pior, com um `as usize` — uma
/// leitura de lixo. O gate afirma o valor exato dos dois lados.
#[test]
fn an_integer_offset_rolls_the_pattern_both_ways() {
    let vals = SLOT_VALS;
    let steps = 3; // ciclo = [v0, v1, v2]
    let at = |off: f32, i: usize| phased_value(i, off, Interp::Step, steps, &vals, &[]);
    for i in 0..9 {
        assert_eq!(at(1.0, i), at(0.0, i + 1), "i={i}: +1 rola uma posicao");
        assert_eq!(
            at(3.0, i),
            at(0.0, i),
            "i={i}: um ciclo inteiro e' identidade"
        );
    }
    // Para trás: `i = 0` com offset −1 tem de ler o ÚLTIMO slot do ciclo.
    assert_eq!(
        at(-1.0, 0),
        at(0.0, 2),
        "-1 no elemento 0 le' o fim do ciclo"
    );
    assert_eq!(at(-4.0, 0), at(0.0, 2), "e da' a volta mais de uma vez");
}

/// **`Step` SALTA, `Linear` DESLIZA — e o gate mede onde as duas leis discordam.**
///
/// ⚠️ A fixture põe a fase em **meio slot**, que é o único sítio onde a diferença
/// existe: numa fase inteira as duas coincidem (o gate acima), então um teste que
/// só amostrasse offsets inteiros ficaria verde com o `interp` ignorado.
#[test]
fn step_jumps_and_linear_slides() {
    let vals = [0.0, 10.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let steps = 2; // ciclo = [0, 10]
    let step_half = phased_value(0, 0.5, Interp::Step, steps, &vals, &[]);
    let lin_half = phased_value(0, 0.5, Interp::Linear, steps, &vals, &[]);
    assert_eq!(
        step_half, 10.0,
        "Step encosta no slot 1 (metade p/ longe do 0)"
    );
    assert_eq!(lin_half, 5.0, "Linear fica no meio");
    // E o deslize é MONÓTONO ao longo de um slot inteiro — a propriedade que um
    // salto não tem.
    let mut prev = f32::NEG_INFINITY;
    for k in 0..=10 {
        let o = phased_value(0, k as f32 / 10.0, Interp::Linear, steps, &vals, &[]);
        assert!(o >= prev, "offset={}: {o} < {prev}", k as f32 / 10.0);
        prev = o;
    }
    assert_eq!(prev, 10.0, "chega inteiro no slot seguinte");
}

/// **O ENCAIXE DO CICLO MISTURA O ÚLTIMO COM O PRIMEIRO** — a leitura periódica,
/// que é a única coerente com um padrão que já se repetia. Um `Linear` que
/// saturasse na ponta faria o último degrau de cada volta comportar-se diferente
/// dos outros, e o artista veria uma costura a cada `steps` elementos.
#[test]
fn linear_wraps_around_the_seam() {
    let vals = [0.0, 10.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let steps = 2; // ciclo = [0, 10]
    // Elemento 1 (slot 1 = 10) com meia fase: o vizinho é o slot 0 da volta
    // seguinte, valor 0 ⇒ o meio é 5.
    assert_eq!(phased_value(1, 0.5, Interp::Linear, steps, &vals, &[]), 5.0);
}

/// **A TABELA AUTORADA OBEDECE À MESMA FASE** — ela não é um segundo nó.
#[test]
fn the_authored_table_obeys_the_same_phase() {
    let vals = SLOT_VALS;
    let authored = [1.0, 2.0, 3.0, 4.0];
    let at = |off: f32, i: usize| phased_value(i, off, Interp::Step, 3, &vals, &authored);
    assert_eq!(at(0.0, 0), 1.0);
    assert_eq!(at(1.0, 0), 2.0, "a fase rola a TABELA, nao os slots");
    assert_eq!(
        at(-1.0, 0),
        4.0,
        "e da' a volta no comprimento DELA (4, nao 3)"
    );
    // E o Linear mistura entradas da tabela, não dos slots.
    assert_eq!(
        phased_value(0, 0.5, Interp::Linear, 3, &vals, &authored),
        1.5
    );
}
