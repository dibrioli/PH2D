//! Os gates do [`super::STRENGTH`] — a força com sinal (doc 89, folha 10).
//!
//! ⚠️ **O harness é local de propósito.** O `lib.rs` deste nó mede **690** linhas contra o teto
//! de 700 do HR-18, e o `mod tests` que já existe lá dentro conta para esse número — pôr mais
//! quatro casos ali reprovaria o gate de LOC no fecho, e a cura de um teto é um split.

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

static SRC_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("field.box.strength.src"),
    name: "field.box.strength.src",
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

struct Src(Vec<[f32; 2]>);
impl NodeOp for Src {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(Stream::new(self.0.len()).with("P", Column::Vec2(self.0.clone())));
    }
}

struct Ops(Src);
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC_MAN.id => Some(&self.0),
            t if t == MANIFEST.id => Some(&FieldBox),
            _ => None,
        }
    }
}

/// Uma caixa 4×4 no centro, arestas duras, com a força pedida. Devolve o `falloff` de cada
/// ponto: um DENTRO da caixa e um FORA.
fn falloff(strength: f32) -> Vec<f32> {
    let ops = Ops(Src(vec![[0.0, 0.0], [8.0, 0.0]]));
    let mut g = Graph::new();
    let src = g.add_node("field.box.strength.src");
    let bx = g.add_node("field.box");
    g.set_param(bx, "width", 4.0);
    g.set_param(bx, "height", 4.0);
    g.set_param(bx, "soft", 0.0);
    g.set_param(bx, STRENGTH, strength);
    g.connect(Edge {
        from: (src, 0),
        to: (bx, 0),
        delayed: false,
    })
    .expect("liga");
    let mut cook = Cook::new();
    let out: NodeId = bx;
    match cook.cook(&g, &ops, out, 0.0).expect("coze")[0]
        .as_stream()
        .get("falloff")
    {
        Some(Column::Scalar(v)) => v.clone(),
        other => panic!("falloff: {other:?}"),
    }
}

/// **`1` É A MÁSCARA DE SEMPRE E `0` DESLIGA O CAMPO — os dois AO BIT.**
///
/// ⚠️ É a forma de dois termos que compra isto. `1 + (f − 1)·s` daria a mesma álgebra e
/// **outro número**: para `f < 0,5` a subtração `f − 1` já perde bits, e o literal deixaria de
/// ser literal exactamente nas máscaras suaves, que são a maioria.
#[test]
fn one_is_the_mask_that_shipped_and_zero_is_the_identity() {
    assert_eq!(falloff(1.0), vec![1.0, 0.0], "força 1 = a caixa de sempre");
    assert_eq!(
        falloff(0.0),
        vec![1.0, 1.0],
        "força 0 = o campo não existe, e a coluna sai na identidade"
    );
}

/// **`1` É BYTE-IDÊNTICO SOBRE UMA MÁSCARA SUAVE** — e é aqui que a forma de dois termos se
/// distingue da outra.
///
/// ⚠️ O gate acima usa arestas duras, onde `f ∈ {0, 1}` e as **duas** formas dão o mesmo
/// número — ele passaria sobre a errada. Este usa uma rampa, onde `f` cai em `[0,1]` inteiro,
/// e compara com o [`box_mask`] cru, sem tolerância: `assert_eq!` sobre `f32`.
#[test]
fn one_is_bit_identical_over_a_soft_mask_too() {
    let pts: Vec<[f32; 2]> = (0..24).map(|i| [i as f32 * 0.37 - 4.0, 0.0]).collect();
    let (w, h, soft, curve_kind) = (4.0f32, 4.0f32, 1.5f32, 2);
    let ops = Ops(Src(pts.clone()));
    let mut g = Graph::new();
    let src = g.add_node("field.box.strength.src");
    let bx = g.add_node("field.box");
    for (k, v) in [("width", w), ("height", h), ("soft", soft)] {
        g.set_param(bx, k, v);
    }
    g.set_param(bx, "curve", curve_kind as f32);
    g.set_param(bx, STRENGTH, 1.0);
    g.connect(Edge {
        from: (src, 0),
        to: (bx, 0),
        delayed: false,
    })
    .expect("liga");
    let mut cook = Cook::new();
    let got = match cook.cook(&g, &ops, bx, 0.0).expect("coze")[0]
        .as_stream()
        .get("falloff")
    {
        Some(Column::Scalar(v)) => v.clone(),
        other => panic!("falloff: {other:?}"),
    };
    let mut fractional = 0;
    for (p, g_) in pts.iter().zip(&got) {
        let want = box_mask(p[0], p[1], w, h, soft, curve_kind);
        assert_eq!(*g_, want, "em {p:?}: a força 1 tem de ser a máscara crua");
        if want > 1e-6 && want < 1.0 - 1e-6 {
            fractional += 1;
        }
    }
    // ⚠️ O controle da FIXTURE: se nenhuma amostra caísse na rampa, o gate acima seria o de
    // arestas duras outra vez, com outro nome.
    assert!(
        fractional >= 4,
        "a fixture tem de conter a rampa, e só {fractional} amostras lá caíram"
    );
}

/// **A FORÇA INTERPOLA** — meio caminho entre a máscara e a identidade.
#[test]
fn a_half_strength_is_halfway_to_the_identity() {
    let f = falloff(0.5);
    assert!((f[0] - 1.0).abs() < 1e-6, "dentro continua cheio: {f:?}");
    assert!(
        (f[1] - 0.5).abs() < 1e-6,
        "fora sobe do zero para o meio: {f:?}"
    );
}

/// **O SINAL PASSA DE UM, e isso NÃO é o `invert`.**
///
/// ⚠️ As duas metades separam os dois controles: em `s = −1` o que estava cheio fica em `1`
/// (a máscara deixa de morder) e o que estava vazio vai a `2` (o campo empurra). O `invert`
/// trocaria os dois DENTRO de `[0,1]` — daria `0` e `1`, que é outro par de números.
#[test]
fn a_negative_strength_pushes_past_one_and_is_not_the_invert() {
    let f = falloff(-1.0);
    assert!((f[0] - 1.0).abs() < 1e-6, "dentro: {f:?}");
    assert!((f[1] - 2.0).abs() < 1e-6, "fora empurra para 2: {f:?}");
    assert_ne!(f, vec![0.0, 1.0], "isto seria o invert, e é outra coisa");
}

/// **O KNOB ESTÁ NO PAINEL E O DEVICE SABE DELE.**
#[test]
fn the_strength_is_reachable_and_uploaded() {
    let hint = PARAM_HINTS
        .iter()
        .find(|h| h.param == STRENGTH)
        .expect("o Strength tem de estar pintado");
    assert!(
        hint.min < 0.0,
        "o curso tem de alcançar o negativo, senão o sinal é inexprimível pelo painel"
    );
    assert!(hint.max > 1.0, "…e passar de 1, senão só sabe apagar");
    assert!(
        GPU_KERNEL.params.contains(&STRENGTH),
        "o device tem de receber a força: {:?}",
        GPU_KERNEL.params
    );
}
