//! Os gates do `motion.tint`.

use super::*;
use ph2d_nodegraph::cook::{Cook, EvalCtx, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};

// Source: 2 white instances with falloff [1, 0.5].
static SRC_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.tint.test.src"),
    name: "motion.tint.test.src",
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
        &SRC_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(
            Stream::new(2)
                .with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 1.0]]))
                .with("falloff", Column::Scalar(vec![1.0, 0.5])),
        );
    }
}
struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC_MAN.id => Some(&Src),
            t if t == MANIFEST.id => Some(&MotionTint),
            _ => None,
        }
    }
}

#[test]
fn tint_sets_target_masked_by_falloff() {
    let mut g = Graph::new();
    let src = g.add_node("motion.tint.test.src");
    let tn = g.add_node("motion.tint");
    g.connect(Edge {
        from: (src, 0),
        to: (tn, 0),
        delayed: false,
    })
    .unwrap();
    g.set_param(tn, "r", 1.0);
    g.set_param(tn, "g", 0.0);
    g.set_param(tn, "b", 0.0);
    g.set_param(tn, "a", 0.4);
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, tn, 0.0).unwrap();
    match out[0].as_stream().get("tint").unwrap() {
        // existing = white; target = (1,0,0,0.4).
        // i0 f=1: exactly the target ; i1 f=0.5: lerp(white,target,0.5).
        Column::Vec4(v) => {
            assert_eq!(v, &vec![[1.0, 0.0, 0.0, 0.4], [1.0, 0.5, 0.5, 0.7]]);
        }
        _ => panic!("tint"),
    }
}

fn default_of(name: &str) -> f32 {
    MANIFEST
        .params
        .iter()
        .find(|p| p.name == name)
        .unwrap()
        .default
}

#[test]
fn default_params_are_opaque_white_and_no_op_on_white() {
    // The colour modifier's identity: Solid mode by default, Start opaque
    // white → a white stream stays white at every falloff (no red/warm
    // dominance from merely dropping in a Tint — the reported-cast fix).
    assert_eq!(default_of("mode"), 0.0, "default mode is Solid");
    let start = [
        default_of("r"),
        default_of("g"),
        default_of("b"),
        default_of("a"),
    ];
    assert_eq!(start, [1.0, 1.0, 1.0, 1.0]);
    let white = [1.0, 1.0, 1.0, 1.0];
    assert_eq!(mixed_tint(white, white, 1.0), white);
    assert_eq!(mixed_tint(white, white, 0.5), white);
    assert_eq!(mixed_tint(white, white, 0.0), white);
}

#[test]
fn lerp4_is_endpoint_exact() {
    let a = [1.0, 1.0, 1.0, 1.0];
    let b = [0.2, 0.4, 0.6, 0.3];
    assert_eq!(lerp4(a, b, 0.0), a);
    assert_eq!(lerp4(a, b, 1.0), b);
    assert_eq!(lerp4(a, b, 0.5), [0.6, 0.7, 0.8, 0.65]);
}

#[test]
fn gradient_mode_ramps_start_to_end_by_normalized_index() {
    // A 3-instance stream carrying Index[0,1,2]+Count[3,3,3]; Gradient mode,
    // default Start=white / End=black → a grayscale ramp keyed by index.
    static GSRC: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.tint.test.grid"),
        name: "motion.tint.test.grid",
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
    struct GSrc;
    impl NodeOp for GSrc {
        fn manifest(&self) -> &'static NodeManifest {
            &GSRC
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(
                Stream::new(3)
                    .with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]]))
                    .with("Index", Column::Scalar(vec![0.0, 1.0, 2.0]))
                    .with("Count", Column::Scalar(vec![3.0, 3.0, 3.0])),
            );
        }
    }
    struct GOps;
    impl OpResolver for GOps {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == GSRC.id => Some(&GSrc),
                t if t == MANIFEST.id => Some(&MotionTint),
                _ => None,
            }
        }
    }
    let mut g = Graph::new();
    let src = g.add_node("motion.tint.test.grid");
    let tn = g.add_node("motion.tint");
    g.connect(Edge {
        from: (src, 0),
        to: (tn, 0),
        delayed: false,
    })
    .unwrap();
    g.set_param(tn, "mode", 1.0); // Gradient (Start=white, End=black defaults)
    let mut cook = Cook::new();
    let out = cook.cook(&g, &GOps, tn, 0.0).unwrap();
    match out[0].as_stream().get("tint").unwrap() {
        // falloff absent → 1, so tint == target. t = 0, 0.5, 1 across the ramp.
        Column::Vec4(v) => {
            assert_eq!(v[0], [1.0, 1.0, 1.0, 1.0]); // start (white)
            assert_eq!(v[1], [0.5, 0.5, 0.5, 1.0]); // mid grey
            assert_eq!(v[2], [0.0, 0.0, 0.0, 1.0]); // end (black)
        }
        _ => panic!("tint"),
    }
}

#[test]
fn mixed_tint_reaches_any_rgba_at_full_falloff() {
    // f=0 → exactly existing (identity); f=1 → exactly the target (all RGBA).
    assert_eq!(
        mixed_tint([1.0, 1.0, 1.0, 1.0], [0.2, 0.4, 0.6, 0.3], 0.0),
        [1.0, 1.0, 1.0, 1.0]
    );
    assert_eq!(
        mixed_tint([1.0, 1.0, 1.0, 1.0], [0.2, 0.4, 0.6, 0.3], 1.0),
        [0.2, 0.4, 0.6, 0.3]
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// `blend` — doc 89 folha 09, a célula do C4D *Color group → Blending Mode*.
//
// ⚠️ **Todos estes gates COZINHAM pelo `Graph`/`Cook`/`OpResolver`.** Um gate que
// chamasse `blended()` direto prova a aritmética e mais nada: a folha 02 desta
// mesma linha viu QUATRO mutações sobreviverem exactamente assim, porque o teste
// reimplementava a lei em vez de a exercer pelo caminho que o app corre.
// ─────────────────────────────────────────────────────────────────────────────

/// A cor EXISTENTE da fonte do blend.
///
/// ⚠️ **Nenhum canal é 0 nem 1**, e é isso que faz os gates discriminarem: com um
/// `existing` branco, `Multiply` e `Mix` devolvem o mesmo número, `Divide`
/// devolve o mesmo que `Multiply` invertido, e uma mutação que trocasse dois
/// braços passaria verde.
const EXISTING: [f32; 4] = [0.8, 0.5, 0.25, 0.5];

/// A máscara do SEGUNDO elemento da fonte — o que prova a ORDEM (blend antes do
/// lerp). O primeiro fica em `1.0`, a máscara neutra.
const HALF_MASK: f32 = 0.5;

static BSRC_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.tint.test.blendsrc"),
    name: "motion.tint.test.blendsrc",
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
struct BSrc;
impl NodeOp for BSrc {
    fn manifest(&self) -> &'static NodeManifest {
        &BSRC_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(
            Stream::new(2)
                .with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0]]))
                .with("tint", Column::Vec4(vec![EXISTING; 2]))
                .with("falloff", Column::Scalar(vec![1.0, HALF_MASK])),
        );
    }
}
struct BOps;
impl OpResolver for BOps {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == BSRC_MAN.id => Some(&BSrc),
            t if t == MANIFEST.id => Some(&MotionTint),
            _ => None,
        }
    }
}

/// Cozinha um `motion.tint` sobre [`EXISTING`] com a cor-alvo `target` e o
/// `blend` pedido. Devolve os DOIS elementos: `[0]` sem máscara (a lei do blend
/// nua) e `[1]` a meia máscara (a ordem).
fn cook_blend_pair(target: [f32; 4], blend: Option<f32>) -> [[f32; 4]; 2] {
    let mut g = Graph::new();
    let src = g.add_node("motion.tint.test.blendsrc");
    let tn = g.add_node("motion.tint");
    g.connect(Edge {
        from: (src, 0),
        to: (tn, 0),
        delayed: false,
    })
    .unwrap();
    for (k, v) in [
        ("r", target[0]),
        ("g", target[1]),
        ("b", target[2]),
        ("a", target[3]),
    ] {
        g.set_param(tn, k, v);
    }
    // ⚠️ Só escreve o param quando o caso o pede: o braço `None` é o que prova
    // que o DEFAULT é Mix, e não uma escrita explícita de `0.0` a fingi-lo.
    if let Some(b) = blend {
        g.set_param(tn, "blend", b);
    }
    let mut cook = Cook::new();
    let out = cook.cook(&g, &BOps, tn, 0.0).unwrap();
    match out[0].as_stream().get("tint").unwrap() {
        Column::Vec4(v) => [v[0], v[1]],
        _ => panic!("tint"),
    }
}

/// A cor do elemento SEM máscara — a lei do blend, isolada.
fn cook_blend(target: [f32; 4], blend: Option<f32>) -> [f32; 4] {
    cook_blend_pair(target, blend)[0]
}

/// **O DEFAULT É MIX, E MIX É A LEI DE SEMPRE — bit a bit.**
///
/// O braço sem `set_param` é o controlo: um grafo autorado antes deste param
/// existir não guarda `blend` nenhum, lê o default do manifesto, e tem de sair
/// EXACTAMENTE onde saía. `assert_eq!` sobre os bits, não um epsilon.
#[test]
fn the_default_blend_is_mix_and_mix_is_the_law_this_node_always_had() {
    assert_eq!(
        default_of("blend"),
        0.0,
        "um grafo velho lê este número, e ele tem de ser Mix"
    );
    let target = [0.2, 0.9, 0.4, 1.0];
    let unwritten = cook_blend(target, None);
    assert_eq!(unwritten, target, "falloff ausente ⇒ o alvo, exactamente");
    assert_eq!(
        unwritten,
        cook_blend(target, Some(0.0)),
        "escrever 0 tem de ser o mesmo que não escrever nada"
    );
}

/// **AS CINCO LEIS SÃO CINCO NÚMEROS DIFERENTES, e cada uma é a sua aritmética.**
///
/// ⚠️ O valor esperado está escrito à mão, não derivado de `blend_channel` — um
/// oráculo que chamasse a função sob teste passaria por qualquer mutação dela.
#[test]
fn each_blend_mode_computes_its_own_arithmetic_through_the_cook() {
    let t = [0.5, 0.25, 0.5, 0.5];
    // e = (0.8, 0.5, 0.25, 0.5); todos os operandos são potências de dois
    // somadas, portanto exactos em f32 — daí o `assert_eq!`.
    let cases: [(f32, [f32; 4], &str); 5] = [
        (0.0, [0.5, 0.25, 0.5, 0.5], "Mix = o alvo"),
        (1.0, [1.3, 0.75, 0.75, 1.0], "Add = e + t"),
        (2.0, [0.3, 0.25, -0.25, 0.0], "Subtract = e - t"),
        (3.0, [0.4, 0.125, 0.125, 0.25], "Multiply = e * t"),
        (4.0, [1.6, 2.0, 0.5, 1.0], "Divide = e / t"),
    ];
    let mut seen: Vec<[f32; 4]> = Vec::new();
    for (v, want, why) in cases {
        let got = cook_blend(t, Some(v));
        assert_eq!(got, want, "blend {v}: {why}");
        assert!(
            !seen.contains(&got),
            "blend {v} devolveu o mesmo que um modo anterior — as leis colapsaram"
        );
        seen.push(got);
    }
}

/// **`Subtract` PODE SAIR NEGATIVO e `Add`/`Divide` PODEM PASSAR DE 1** — e isso
/// é produto, não descuido (ver os docs do módulo: o RGB é HDR de propósito, e a
/// escolha é do dispositivo, não do caminho lento).
///
/// FALSIFICADO se alguém acrescentar um `clamp` "por segurança": o `2.0` que um
/// `Add` de dois brancos produz é a fonte de brilho que o `fx.glow` enxerga.
#[test]
fn nothing_is_clamped_because_the_column_is_hdr() {
    let white = [1.0, 1.0, 1.0, 1.0];
    let added = cook_blend(white, Some(1.0));
    assert_eq!(added[0], 1.8, "0.8 + 1.0, sem tecto");
    let subtracted = cook_blend(white, Some(2.0));
    assert!(subtracted[2] < 0.0, "0.25 - 1.0 é negativo: {subtracted:?}");
}

/// **DIVIDIR POR NADA NÃO MUDA NADA — e `-0.0` é nada.**
///
/// ⚠️ O braço do `-0.0` é o gate que a folha 11 desta mesma linha pagou, e a
/// mutação que o prova está MEDIDA: trocar `t == 0.0` por `t.to_bits() == 0`
/// (que só apanha o `+0.0`) deixa o `-0.0` cair no `e / t`, que é `-inf`, e um
/// `inf` nesta coluna viaja por todo consumidor a jusante. Nenhum epsilon aqui:
/// a saída tem de ser FINITA e exactamente `e`.
///
/// ⛔ **Não escreva o guarda com um teste de sinal.** `-0.0` e `+0.0` comparam
/// IGUAIS em IEEE e têm bits diferentes; qualquer forma que olhe os bits separa
/// dois valores que a aritmética não separa, e o segundo sai infinito.
#[test]
fn divide_by_a_zero_channel_returns_the_channel_and_never_an_infinity() {
    for zero in [0.0_f32, -0.0_f32] {
        let got = cook_blend([zero, zero, zero, zero], Some(4.0));
        assert_eq!(
            got, EXISTING,
            "dividir por {zero} tem de devolver o que lá estava"
        );
        assert!(
            got.iter().all(|c| c.is_finite()),
            "nada de inf/NaN: {got:?}"
        );
    }
}

/// **A ORDEM: o blend encontra a cor ANTES da máscara, nunca depois.**
///
/// `e = 0.8`, alvo `t = 0.5`, `Multiply`, máscara `0.5`:
/// - a lei certa é `lerp(e, e·t, ½)` = `lerp(0.8, 0.4, ½)` = **0.6**;
/// - mascarar primeiro e blendar depois daria `e · lerp(e, t, ½)` = `0.8 · 0.65`
///   = **0.52**.
///
/// ⚠️ **Este gate precisou de um fixture NOVO, e vale a pena dizer porquê:** a
/// primeira versão usava a fonte branca (`existing = 1.0`), e com `e = 1` as
/// duas ordens dão o MESMO número — `1 · lerp(1, t, f)` é literalmente
/// `lerp(1, 1·t, f)`. Era um controlo que não controlava nada. *Um fixture só
/// prova o que contém* (memória da casa), e a identidade multiplicativa apaga
/// exactamente a diferença que este teste existe para ver.
#[test]
fn the_blend_meets_the_colour_before_the_mask_lerps() {
    let pair = cook_blend_pair([0.5, 0.5, 0.5, 0.5], Some(3.0)); // Multiply
    assert_eq!(pair[0][0], 0.4, "máscara cheia: o produto, exacto");
    assert_eq!(
        pair[1][0], 0.6,
        "meia máscara: lerp(e, e·t, ½). A ordem trocada daria 0.52"
    );
}

/// **A FAIXA DO SLIDER E A LISTA DE NOMES SÃO A MESMA LISTA**, e todo índice dela
/// nomeia um modo distinto.
///
/// ⚠️ O laço sobre os índices é o que faz este gate valer: um `match` exaustivo
/// em `from_param` NÃO guarda a lista que o painel itera (memória da casa) — um
/// rótulo a mais com o `BLEND_MAX` parado deixaria uma linha inalcançável.
#[test]
fn the_blend_slider_reaches_every_named_mode_and_no_more() {
    let hint = PARAM_HINTS
        .iter()
        .find(|h| h.param == "blend")
        .expect("o painel pinta a linha");
    let ParamWidget::Enum { labels } = hint.widget else {
        panic!("o blend é um Enum nomeado, não um slider cru")
    };
    assert_eq!(labels, &BlendMode::LABELS, "uma lista só");
    assert_eq!(hint.min, 0.0);
    assert_eq!(
        hint.max as usize,
        labels.len() - 1,
        "o topo é o último rótulo"
    );
    let mut modes = Vec::new();
    for k in 0..labels.len() {
        #[expect(clippy::cast_precision_loss, reason = "5 índices")]
        let m = BlendMode::from_param(k as f32);
        assert!(
            !modes.contains(&m),
            "o índice {k} repete um modo já nomeado"
        );
        modes.push(m);
    }
    assert_eq!(modes.len(), BlendMode::LABELS.len());
}

/// **UM NÚMERO FORA DA LISTA CAI EM MIX** — a identidade, nunca um vizinho
/// arbitrário. É o que faz um grafo gravado por um build FUTURO (com um sexto
/// modo) abrir sem repintar a cena com uma lei que ninguém escolheu.
#[test]
fn an_unknown_blend_value_degrades_to_mix() {
    let target = [0.2, 0.9, 0.4, 1.0];
    for v in [5.0, 99.0, -3.0] {
        assert_eq!(
            cook_blend(target, Some(v)),
            target,
            "blend {v} tem de ser Mix"
        );
    }
}

/// **O KERNEL DE GPU DECLARA O PARAM** — senão o device corre a lei antiga e a
/// cor diverge do caminho de referência sem nada na tela dizer porquê.
#[test]
fn the_gpu_kernel_carries_the_blend_param_and_its_law() {
    assert!(
        GPU_KERNEL.params.contains(&"blend"),
        "o uniforme tem de carregar o blend: {:?}",
        GPU_KERNEL.params
    );
    assert!(
        GPU_KERNEL.wgsl.contains("tn_blend("),
        "o corpo tem de chamar a lei"
    );
    for arm in [
        "return e + t;",
        "return e - t;",
        "return e * t;",
        "return e / t;",
    ] {
        assert!(
            GPU_KERNEL.wgsl_lib.contains(arm),
            "falta o braço `{arm}` no WGSL"
        );
    }
    assert!(
        GPU_KERNEL.wgsl_lib.contains("if (t == 0.0) { return e; }"),
        "o divisor zero tem de ser tratado no device como na CPU"
    );
}
