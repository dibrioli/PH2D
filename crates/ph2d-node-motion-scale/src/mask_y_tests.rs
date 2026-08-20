//! Os gates do [`super::USE_FALLOFF_Y`] — a máscara do eixo Y (doc 89, folha 05).

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};

/// **A fixture que contém o fenómeno**: DOIS elementos cujas duas máscaras são
/// *trocadas* — o primeiro tem X cheio e Y nulo, o segundo o inverso.
///
/// ⚠️ Uma fixture com as duas colunas IGUAIS passaria por todos estes gates com o
/// bug intacto (partilhar o peso e ler o peso certo dão o mesmo número quando os
/// pesos são o mesmo). É a diferença entre eles que é o teste.
static SRC_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.scale.test.two_masks"),
    name: "motion.scale.test.two_masks",
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

struct TwoMasks;
impl NodeOp for TwoMasks {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(
            Stream::new(2)
                .with("size", Column::Vec2(vec![[1.0, 1.0], [1.0, 1.0]]))
                .with("falloff", Column::Scalar(vec![1.0, 0.0]))
                .with(FALLOFF_Y, Column::Scalar(vec![0.0, 1.0])),
        );
    }
}

/// A mesma lista SEM o segundo canal — o controle da identidade.
static BARE_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.scale.test.one_mask"),
    name: "motion.scale.test.one_mask",
    ..SRC_MAN
};

struct OneMask;
impl NodeOp for OneMask {
    fn manifest(&self) -> &'static NodeManifest {
        &BARE_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(
            Stream::new(2)
                .with("size", Column::Vec2(vec![[1.0, 1.0], [1.0, 1.0]]))
                .with("falloff", Column::Scalar(vec![1.0, 0.0])),
        );
    }
}

struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC_MAN.id => Some(&TwoMasks),
            t if t == BARE_MAN.id => Some(&OneMask),
            t if t == MANIFEST.id => Some(&MotionScale),
            _ => None,
        }
    }
}

/// Coze `src → motion.scale(…)` e devolve a coluna `size`.
fn sizes(src: &str, uniform: f32, split: f32) -> Vec<[f32; 2]> {
    let mut g = Graph::new();
    let s = g.add_node(src);
    let sc = g.add_node("motion.scale");
    g.connect(Edge {
        from: (s, 0),
        to: (sc, 0),
        delayed: false,
    })
    .unwrap();
    g.set_param(sc, "amount", 3.0);
    g.set_param(sc, "amount_y", 5.0);
    g.set_param(sc, "uniform", uniform);
    g.set_param(sc, USE_FALLOFF_Y, split);
    let mut cook = Cook::new();
    match cook.cook(&g, &Ops, sc, 0.0).unwrap()[0]
        .as_stream()
        .get("size")
    {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("size"),
    }
}

/// **DESLIGADO, A COLUNA NOVA NÃO EXISTE PARA O NÓ — ao bit.**
///
/// ⚠️ Este é o gate da inércia, e ele mede a coisa certa: a lista TEM um
/// `falloff_y` que discorda do `falloff` em todos os elementos, então se o nó o
/// tivesse lido por presença (o desenho que este param recusa) os dois lados
/// divergiriam. Comparar com o próprio nó sobre uma lista sem a coluna é o que
/// torna a igualdade uma afirmação sobre o CÓDIGO, e não sobre números que
/// escrevi à mão.
#[test]
fn with_the_toggle_off_the_second_column_may_as_well_not_exist() {
    let with = sizes("motion.scale.test.two_masks", 0.0, 0.0);
    let without = sizes("motion.scale.test.one_mask", 0.0, 0.0);
    assert_eq!(with, without);
}

/// **LIGADO, CADA EIXO ANDA COM A SUA MÁSCARA** — squash de um, stretch do vizinho.
#[test]
fn the_toggle_lets_each_axis_ride_its_own_mask() {
    let s = sizes("motion.scale.test.two_masks", 0.0, 1.0);
    // i0: falloff 1 (X cheio → 3), falloff_y 0 (Y intacto → 1).
    assert_eq!(s[0], [3.0, 1.0], "o primeiro estica só na largura");
    // i1: falloff 0 (X intacto → 1), falloff_y 1 (Y cheio → 5).
    assert_eq!(s[1], [1.0, 5.0], "e o vizinho só na altura");
}

/// **LIGADO SEM A COLUNA, O EIXO Y LEVA O EFEITO CHEIO** — a identidade `1.0`, não
/// uma cópia do `falloff` (ver [`FALLOFF_Y`]).
///
/// ⚠️ É a metade que faz o kernel do device poder existir: a resposta da CPU aqui
/// é a mesma que o `identity: [1.0; 4]` da binding dá, sem o device precisar de
/// saber se a coluna estava lá.
#[test]
fn an_absent_second_mask_is_full_effect_and_not_a_copy_of_the_first() {
    let s = sizes("motion.scale.test.one_mask", 0.0, 1.0);
    assert_eq!(s[0], [3.0, 5.0]);
    // i1 tem `falloff = 0`: o X fica na identidade e o Y vai cheio na mesma.
    assert_eq!(s[1], [1.0, 5.0], "o Y não pode herdar o peso do X");
}

/// **O LINK VENCE** — com `uniform` ligado o toggle não morde.
///
/// ⚠️ O `ParamGate` esconde o controle nesse estado, e este gate prova que
/// esconder e não-fazer são a MESMA coisa. Um param escondido que ainda mordesse
/// seria um efeito sem porta na tela.
#[test]
fn the_uniform_link_wins_over_the_second_mask() {
    let linked = sizes("motion.scale.test.two_masks", 1.0, 1.0);
    let plain = sizes("motion.scale.test.two_masks", 1.0, 0.0);
    assert_eq!(linked, plain);
    // E o que ele faz é o de sempre: os dois eixos com o `amount` e o `falloff`.
    assert_eq!(linked, vec![[3.0, 3.0], [1.0, 1.0]]);
}

/// **O TOGGLE ESTÁ PINTADO, GATEADO PELO LINK, e o device lê o canal.**
#[test]
fn the_toggle_is_painted_gated_and_uploaded() {
    let h = PARAM_HINTS
        .iter()
        .find(|h| h.param == USE_FALLOFF_Y)
        .expect("o Separate Y Mask tem de estar pintado");
    assert!(matches!(h.widget, ParamWidget::Toggle));
    assert!(
        PARAM_GATES
            .iter()
            .any(|g| g.param == USE_FALLOFF_Y && g.when == "uniform"),
        "com o link ligado o controle não é pintado"
    );
    assert!(
        GPU_KERNEL.bindings.iter().any(|b| b.column == FALLOFF_Y),
        "o device tem de ler a segunda máscara: {:?}",
        GPU_KERNEL
            .bindings
            .iter()
            .map(|b| b.column)
            .collect::<Vec<_>>()
    );
    assert!(
        GPU_KERNEL.params.contains(&USE_FALLOFF_Y),
        "…e tem de saber se a lê: {:?}",
        GPU_KERNEL.params
    );
}
