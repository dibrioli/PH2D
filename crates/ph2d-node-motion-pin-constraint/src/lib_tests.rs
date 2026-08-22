//! Gates do `motion.pin_constraint` — a lei do pino, os modos e a mira.
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 700 para `crates/`), e o
//! corte é o que a casa já usa nos irmãos: o arquivo de produção fica com a LEI e
//! este com as PROVAS. Os caminhos não mudam — `#[path]` mantém o módulo a
//! chamar-se `tests` e `use super::*` resolve como sempre resolveu.

use super::*;

/// A stream of `n` elements at the origin, with the given optional fields.
fn stream(n: usize, falloff: Option<Vec<f32>>, inv_mass: Option<Vec<f32>>) -> Stream {
    let mut s = Stream::new(n).with("P", Column::Vec2(vec![[0.0, 0.0]; n]));
    if let Some(f) = falloff {
        s.set(FALLOFF, Column::Scalar(f));
    }
    if let Some(w) = inv_mass {
        s.set(INV_MASS, Column::Scalar(w));
    }
    s
}

fn weights(s: &Stream) -> Vec<f32> {
    match s.get(INV_MASS) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => panic!("no inv_mass column"),
    }
}

/// The range is what gets pinned: inside it the inverse mass is 0 (infinite
/// mass), outside it stays 1 (free). FALSIFIED if the node pinned the whole
/// stream (the bug that would freeze every sim downstream).
#[test]
fn the_index_range_is_what_gets_pinned() {
    let out = pin(
        &stream(5, None, None),
        &Stream::new(0),
        &Stream::new(0),
        1,
        2,
        1.0,
        0.0,
    );
    assert_eq!(weights(&out), vec![1.0, 0.0, 0.0, 1.0, 1.0]);
}

/// `strength` is a PARTIAL pin (Blender's pin weight): half strength leaves
/// half the inverse mass, i.e. an element twice as heavy as its neighbours.
#[test]
fn strength_is_a_partial_pin() {
    let out = pin(
        &stream(2, None, None),
        &Stream::new(0),
        &Stream::new(0),
        0,
        1,
        0.25,
        0.0,
    );
    assert_eq!(weights(&out), vec![0.75, 1.0]);
}

/// The `falloff` field scales the pin, so an upstream falloff pins a REGION:
/// full field = nailed, half = heavy, zero = untouched.
#[test]
fn the_falloff_field_scales_the_pin() {
    let out = pin(
        &stream(3, Some(vec![1.0, 0.5, 0.0]), None),
        &Stream::new(0),
        &Stream::new(0),
        0,
        3,
        1.0,
        0.0,
    );
    assert_eq!(weights(&out), vec![0.0, 0.5, 1.0]);
}

/// Two pins COMPOSE (multiply) instead of the second erasing the first —
/// the falloff family's rule. Two half-pins on the same element leave a
/// quarter of the inverse mass.
#[test]
fn pins_compose_multiplicatively() {
    let once = pin(
        &stream(1, None, None),
        &Stream::new(0),
        &Stream::new(0),
        0,
        1,
        0.5,
        0.0,
    );
    let twice = pin(&once, &Stream::new(0), &Stream::new(0), 0, 1, 0.5, 0.0);
    assert_eq!(weights(&twice), vec![0.25]);
}

/// `count = 0` (or a zero strength) selects nothing: every element stays
/// free, and an upstream weight rides through untouched.
#[test]
fn an_empty_selection_is_the_identity() {
    assert_eq!(
        weights(&pin(
            &stream(3, None, None),
            &Stream::new(0),
            &Stream::new(0),
            0,
            0,
            1.0,
            0.0
        )),
        vec![1.0; 3]
    );
    assert_eq!(
        weights(&pin(
            &stream(3, None, None),
            &Stream::new(0),
            &Stream::new(0),
            0,
            3,
            0.0,
            0.0
        )),
        vec![1.0; 3]
    );
    let carried = stream(2, None, Some(vec![0.0, 0.5]));
    assert_eq!(
        weights(&pin(
            &carried,
            &Stream::new(0),
            &Stream::new(0),
            0,
            0,
            1.0,
            0.0
        )),
        vec![0.0, 0.5]
    );
}

/// A non-finite param never poisons the weights (a hand-edited document can
/// carry any `f32`): the element stays free rather than going NaN.
#[test]
fn a_non_finite_strength_reads_as_free() {
    let out = pin(
        &stream(1, None, None),
        &Stream::new(0),
        &Stream::new(0),
        0,
        1,
        f32::NAN,
        0.0,
    );
    assert_eq!(weights(&out), vec![1.0]);
}

/// Cooks through the registry: the weights land on the stream and every
/// other column (the positions the node must NOT touch) passes through.
#[test]
fn registers_and_cooks_the_weight_column() {
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    static SRC: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.pin.test.src"),
        name: "motion.pin.test.src",
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
                Stream::new(3).with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]])),
            );
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC.id => Some(&Src),
                t if t == MANIFEST.id => Some(&MotionPinConstraint),
                _ => None,
            }
        }
    }
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    assert!(reg.resolve(MANIFEST.id).is_some());

    let mut g = Graph::new();
    let src = g.add_node("motion.pin.test.src");
    let p = g.add_node("motion.pin_constraint");
    g.set_param(p, "count", 2.0);
    g.connect(Edge {
        from: (src, 0),
        to: (p, 0),
        delayed: false,
    })
    .unwrap();
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, p, 0.0).unwrap();
    let s = out[0].as_stream();
    assert_eq!(s.count(), 3, "count preserved");
    assert_eq!(weights(s), vec![0.0, 0.0, 1.0], "the first two are pinned");
    match s.get("P").unwrap() {
        Column::Vec2(v) => assert_eq!(v[1], [1.0, 0.0], "positions ride through"),
        _ => panic!("P"),
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Doc 89 folha 03 — o LIMIAR DE RUPTURA. Ver [`BREAK_ABOVE`] para o mecanismo.
// ─────────────────────────────────────────────────────────────────────────

/// Um stream de `n` elementos com a carga (`accel`) escrita à mão.
fn loaded(n: usize, accel: &[[f32; 2]]) -> Stream {
    Stream::new(n)
        .with("P", Column::Vec2(vec![[0.0, 0.0]; n]))
        .with(ACCEL, Column::Vec2(accel.to_vec()))
}

fn inv_mass_of(s: &Stream) -> Vec<f32> {
    match s.get(INV_MASS) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => panic!("inv_mass"),
    }
}

/// **`0` NÃO RASGA NADA — e é a identidade, com carga ou sem ela.**
///
/// ⚠️ O braço COM carga é o que importa: um nó que comparasse `carga > 0` soltaria
/// todo pin de toda cena que tenha uma força, em silêncio.
#[test]
fn a_zero_threshold_never_tears_even_under_load() {
    let sem = pin(
        &loaded(2, &[[0.0, 0.0]; 2]),
        &Stream::new(0),
        &loaded(2, &[[0.0, 0.0]; 2]),
        0,
        2,
        1.0,
        0.0,
    );
    let com = pin(
        &loaded(2, &[[99.0, 0.0]; 2]),
        &Stream::new(0),
        &loaded(2, &[[99.0, 0.0]; 2]),
        0,
        2,
        1.0,
        0.0,
    );
    assert_eq!(inv_mass_of(&sem), vec![0.0, 0.0], "pinado");
    assert_eq!(
        inv_mass_of(&com),
        vec![0.0, 0.0],
        "e continua pinado sob carga"
    );
}

/// **ACIMA DO LIMIAR O PIN SOLTA; ABAIXO, SEGURA.**
///
/// Carga `3` e `6` contra um limiar de `5`: a primeira segura, a segunda rasga.
/// FALSIFICADO por uma comparação invertida, ou por um limiar que agisse sobre o
/// stream todo em vez de por elemento.
#[test]
fn the_pin_tears_only_where_the_load_exceeds_the_threshold() {
    let out = pin(
        &loaded(2, &[[3.0, 0.0], [6.0, 0.0]]),
        &Stream::new(0),
        &loaded(2, &[[3.0, 0.0], [6.0, 0.0]]),
        0,
        2,
        1.0,
        5.0,
    );
    assert_eq!(inv_mass_of(&out), vec![0.0, 1.0], "só o segundo rasgou");
}

/// **A CARGA É O MÓDULO, não uma componente** — uma força só em Y rasga tanto como
/// uma só em X.
#[test]
fn the_load_is_the_magnitude_of_the_accumulated_force() {
    let out = pin(
        &loaded(3, &[[6.0, 0.0], [0.0, 6.0], [3.0, 4.0]]),
        &Stream::new(0),
        &loaded(3, &[[6.0, 0.0], [0.0, 6.0], [3.0, 4.0]]),
        0,
        3,
        1.0,
        4.9,
    );
    // 6, 6 e 5 — os três passam de 4,9.
    assert_eq!(inv_mass_of(&out), vec![1.0, 1.0, 1.0]);
}

/// **SEM `accel` NO STREAM A CARGA É ZERO E NADA RASGA** — a resposta certa (não há
/// força a puxar), não um erro silencioso.
///
/// ⚠️ É também o que documenta a ORDEM de fiação: um pin posto ANTES das forças não
/// vê carga nenhuma, porque a coluna ainda não existe.
#[test]
fn a_stream_without_accel_carries_no_load() {
    let out = pin(
        &stream(2, None, None),
        &Stream::new(0),
        &Stream::new(0),
        0,
        2,
        1.0,
        0.01,
    );
    assert_eq!(inv_mass_of(&out), vec![0.0, 0.0], "sem carga, nada rasga");
}

/// **O RASGO É PERMANENTE — mas SÓ com o laço de estado, e o gate mede as duas
/// metades.**
///
/// ⚠️ Sem o `pre` a marca não sobrevive ao quadro e o pin volta a segurar quando a
/// rajada passa: um **cedimento elástico**, não um rasgo. Não é um modo escondido; é
/// a consequência de faltar o fio, e está escrito no doc do param.
#[test]
fn a_torn_pin_stays_torn_only_when_the_state_loop_exists() {
    let rajada = loaded(1, &[[9.0, 0.0]]);
    let calmo = loaded(1, &[[0.0, 0.0]]);
    // Tique 1: rasga.
    let t1 = pin(&rajada, &Stream::new(0), &rajada, 0, 1, 1.0, 5.0);
    assert_eq!(inv_mass_of(&t1), vec![1.0], "rasgou");
    // Tique 2 COM o laço: a marca chega, e ele continua solto mesmo sem carga.
    let com = pin(&calmo, &t1, &calmo, 0, 1, 1.0, 5.0);
    assert_eq!(inv_mass_of(&com), vec![1.0], "rasgado é rasgado");
    // Tique 2 SEM o laço: ele volta a pinar — o cedimento elástico.
    let sem = pin(&calmo, &Stream::new(0), &calmo, 0, 1, 1.0, 5.0);
    assert_eq!(inv_mass_of(&sem), vec![0.0], "sem memória, ele re-pina");
    assert_ne!(
        inv_mass_of(&com),
        inv_mass_of(&sem),
        "e as duas leis diferem"
    );
}

/// **A MEMÓRIA SAI SEMPRE, mesmo a zeros** — uma coluna que aparece e desaparece
/// conforme o param faria o `pre` do tique seguinte ler um stream de outra forma.
#[test]
fn the_tear_memory_is_always_written() {
    for limiar in [0.0_f32, 5.0] {
        let out = pin(
            &loaded(2, &[[0.0, 0.0]; 2]),
            &Stream::new(0),
            &loaded(2, &[[0.0, 0.0]; 2]),
            0,
            2,
            1.0,
            limiar,
        );
        match out.get(TORN) {
            Some(Column::Scalar(v)) => assert_eq!(v.len(), 2, "limiar {limiar}"),
            _ => panic!("a coluna do rasgo tem de sair sempre (limiar {limiar})"),
        }
    }
}

/// **O LIMIAR LIGADO RECUSA O DEVICE, E O DESLIGADO NÃO.**
#[test]
fn the_tear_refuses_the_device_and_the_default_does_not() {
    let f = GPU_KERNEL.applicable.expect("o kernel declara a recusa");
    assert!(f(&|_: &str| 0.0), "sem limiar: o device continua a valer");
    assert!(
        !f(&|n: &str| if n == BREAK_ABOVE { 5.0 } else { 0.0 }),
        "com limiar: o rasgo é estado, e o kernel é um mapa sem memória"
    );
}

/// **A CARGA VEM DA PORTA `load`; SEM ELA, DO PRÓPRIO `in`.**
///
/// ⚠️ Os dois idiomas: com o `motion.integrate` o pin tem de estar no caminho da
/// arte (é de lá que o `inv_mass` é lido) e a carga chega-lhe pela porta; com um
/// GERADOR (`motion.soft_body`) ele cabe dentro da cadeia de estado e o `in` já a
/// traz. Uma precedência, não duas fontes.
#[test]
fn the_load_falls_back_to_the_nodes_own_input() {
    let carga = loaded(1, &[[9.0, 0.0]]);
    let limpo = stream(1, None, None);
    // Pela PORTA: o `in` não tem carga nenhuma.
    let pela_porta = pin(&limpo, &Stream::new(0), &carga, 0, 1, 1.0, 5.0);
    assert_eq!(inv_mass_of(&pela_porta), vec![1.0], "a porta manda");
    // Pelo `in`: a porta está vazia.
    let pelo_in = pin(&carga, &Stream::new(0), &Stream::new(0), 0, 1, 1.0, 5.0);
    assert_eq!(inv_mass_of(&pelo_in), vec![1.0], "o recuo funciona");
    // E a PORTA VENCE o `in` quando as DUAS trazem carga — uma precedência, não uma
    // soma. ⚠️ A precedência é sobre a COLUNA e não sobre o fio: uma porta ligada a
    // um stream SEM `accel` não é fonte de carga nenhuma, e é indistinguível de uma
    // porta desligada — que é o que o braço `limpo` acima já prova.
    let leve = loaded(1, &[[1.0, 0.0]]);
    let manda = pin(&carga, &Stream::new(0), &leve, 0, 1, 1.0, 5.0);
    assert_eq!(
        inv_mass_of(&manda),
        vec![0.0],
        "a carga da PORTA (1) está abaixo do limiar; a do `in` (9) não pode falar"
    );
}
