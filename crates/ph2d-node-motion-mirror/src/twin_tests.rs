//! Os gates do [`super::FLIP_ROT`] e do [`super::REINDEX`] — o gêmeo por inteiro
//! e as colunas de identidade da lista que CRESCE (doc 89, folha 05).

use super::*;
use ph2d_nodegraph::cook::{Cook, EvalCtx, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};

/// Uma fonte de TRÊS elementos orientados, com velocidade, tamanho e as duas
/// colunas de identidade — a lista que o defeito medido descrevia.
///
/// ⚠️ **O `size` está aqui de propósito**: ele é `Vec2` como o `vel`, e é o
/// controle negativo da regra por-tipo (ver [`twin`]).
static SRC_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.mirror.test.school"),
    name: "motion.mirror.test.school",
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

struct School;
impl NodeOp for School {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(
            Stream::new(3)
                .with("P", Column::Vec2(vec![[1.0, 0.0], [2.0, 1.0], [3.0, -1.0]]))
                .with("rot", Column::Scalar(vec![0.0, 30.0, 90.0]))
                .with(
                    "vel",
                    Column::Vec2(vec![[2.0, 1.0], [0.0, 3.0], [-1.0, 0.0]]),
                )
                .with("size", Column::Vec2(vec![[0.5, 0.5]; 3]))
                .with(INDEX, Column::Scalar(vec![0.0, 1.0, 2.0]))
                .with(COUNT, Column::Scalar(vec![3.0; 3])),
        );
    }
}

/// A mesma lista sem coluna de identidade nenhuma.
static BARE_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.mirror.test.bare"),
    name: "motion.mirror.test.bare",
    ..SRC_MAN
};

struct BareSchool;
impl NodeOp for BareSchool {
    fn manifest(&self) -> &'static NodeManifest {
        &BARE_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(Stream::new(2).with("P", Column::Vec2(vec![[1.0, 0.0], [2.0, 0.0]])));
    }
}

struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC_MAN.id => Some(&School),
            t if t == BARE_MAN.id => Some(&BareSchool),
            t if t == MANIFEST.id => Some(&MotionMirror),
            _ => None,
        }
    }
}

/// Coze `src → motion.mirror(axis, flip_rot, reindex)`.
fn run(src: &str, axis: f32, flip: f32, reindex: f32) -> Stream {
    let mut g = Graph::new();
    let s = g.add_node(src);
    let m = g.add_node("motion.mirror");
    g.connect(Edge {
        from: (s, 0),
        to: (m, 0),
        delayed: false,
    })
    .unwrap();
    g.set_param(m, "axis", axis);
    g.set_param(m, FLIP_ROT, flip);
    g.set_param(m, REINDEX, reindex);
    let mut cook = Cook::new();
    cook.cook(&g, &Ops, m, 0.0).unwrap()[0].as_stream().clone()
}

fn scalar(st: &Stream, name: &str) -> Option<Vec<f32>> {
    match st.get(name) {
        Some(Column::Scalar(v)) => Some(v.clone()),
        _ => None,
    }
}

fn vec2(st: &Stream, name: &str) -> Option<Vec<[f32; 2]>> {
    match st.get(name) {
        Some(Column::Vec2(v)) => Some(v.clone()),
        _ => None,
    }
}

/// **DESLIGADO, O GÊMEO É UMA CÓPIA — as duas colunas orientadas incluídas.**
#[test]
fn by_default_the_twin_copies_orientation_and_velocity() {
    let out = run("motion.mirror.test.school", 0.0, 0.0, 0.0);
    assert_eq!(
        scalar(&out, "rot"),
        Some(vec![0.0, 30.0, 90.0, 0.0, 30.0, 90.0])
    );
    assert_eq!(
        vec2(&out, "vel"),
        Some(vec![
            [2.0, 1.0],
            [0.0, 3.0],
            [-1.0, 0.0],
            [2.0, 1.0],
            [0.0, 3.0],
            [-1.0, 0.0]
        ])
    );
}

/// **UM ESPELHO VERTICAL LEVA `θ` A `180 − θ`, e não a `−θ`.**
///
/// ⚠️ Este é o gate que separa as duas fórmulas, e o oráculo é escolhido para as
/// distinguir: a `30°` a resposta certa é `150` e a errada é `−30`. Num teste com
/// `θ = 0` ou `θ = 90` as duas coincidiriam (`180−0 = 180 ≡ −0` não, mas
/// `180−90 = 90 = −90 + 180`) — é por isso que a fixture tem um ângulo oblíquo.
#[test]
fn a_vertical_mirror_reflects_the_heading_instead_of_negating_it() {
    let out = run("motion.mirror.test.school", 0.0, 1.0, 0.0);
    let rot = scalar(&out, "rot").expect("rot");
    assert_eq!(&rot[..3], &[0.0, 30.0, 90.0], "os originais não se mexem");
    assert_eq!(&rot[3..], &[180.0, 150.0, 90.0], "e o gêmeo é o reflexo");
    // A velocidade troca só o componente NORMAL à reta.
    let vel = vec2(&out, "vel").expect("vel");
    assert_eq!(&vel[3..], &[[-2.0, 1.0], [0.0, 3.0], [1.0, 0.0]]);
}

/// **UM ESPELHO HORIZONTAL LEVA `θ` A `−θ`** — a outra fórmula, e ela é outra.
#[test]
fn a_horizontal_mirror_negates_the_heading() {
    let out = run("motion.mirror.test.school", 1.0, 1.0, 0.0);
    let rot = scalar(&out, "rot").expect("rot");
    assert_eq!(&rot[3..], &[0.0, -30.0, -90.0]);
    let vel = vec2(&out, "vel").expect("vel");
    assert_eq!(&vel[3..], &[[2.0, -1.0], [0.0, -3.0], [-1.0, 0.0]]);
}

/// **REFLETIR DUAS VEZES É NÃO REFLETIR** — a propriedade que define um espelho.
///
/// ⚠️ O oráculo é uma INVOLUÇÃO e não um número: ele vale nos dois eixos e em
/// qualquer ângulo, então uma fórmula que acertasse a fixture por acaso (um
/// `θ + 180`, digamos) morre aqui, porque `θ + 360 ≠ θ` como número.
#[test]
fn reflecting_twice_is_the_identity_on_both_axes() {
    for vertical in [true, false] {
        for deg in [0.0f32, 30.0, -47.5, 90.0, 179.0] {
            assert_eq!(mirror_angle(mirror_angle(deg, vertical), vertical), deg);
            let q = [1.5f32, -2.5];
            assert_eq!(mirror_vec(mirror_vec(q, vertical), vertical), q);
        }
    }
}

/// **O `size` NUNCA É REFLETIDO** — o controle negativo da regra por-tipo.
///
/// ⚠️ `vel` e `size` são as duas colunas `Vec2` desta lista. Uma cura escrita como
/// *"todo `Vec2` espelha"* passaria por todos os gates acima e faria metade do
/// cardume nascer com largura negativa.
#[test]
fn the_size_column_is_copied_and_never_reflected() {
    let out = run("motion.mirror.test.school", 0.0, 1.0, 0.0);
    assert_eq!(vec2(&out, "size"), Some(vec![[0.5, 0.5]; 6]));
}

/// **DESLIGADO, AS COLUNAS DE IDENTIDADE DESCREVEM A LISTA DE ANTES** — o defeito
/// MEDIDO, fixado como o comportamento de hoje (ver [`REINDEX`] para o porquê do
/// default).
#[test]
fn without_the_renumbering_the_identity_columns_describe_the_old_list() {
    let out = run("motion.mirror.test.school", 0.0, 0.0, 0.0);
    assert_eq!(out.count(), 6);
    assert_eq!(
        scalar(&out, INDEX),
        Some(vec![0.0, 1.0, 2.0, 0.0, 1.0, 2.0]),
        "o Index repete-se — cada metade lê a rampa inteira"
    );
    assert_eq!(
        scalar(&out, COUNT),
        Some(vec![3.0; 6]),
        "e o Count diz 3 sobre uma lista de 6"
    );
}

/// **LIGADO, AS DUAS PASSAM A DESCREVER A LISTA DOBRADA** — e são as DUAS.
///
/// ⚠️ Meia cura faz mal: com só o `Count` corrigido a rampa do `motion.tint`
/// alcançaria metade, duas vezes (`Index` até 2, dividido por 5).
#[test]
fn the_renumbering_fixes_both_columns_at_once() {
    let out = run("motion.mirror.test.school", 0.0, 0.0, 1.0);
    assert_eq!(
        scalar(&out, INDEX),
        Some(vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0])
    );
    assert_eq!(scalar(&out, COUNT), Some(vec![6.0; 6]));
}

/// **CUNHA AS DUAS MESMO EM BRANCO** — porque a CONTAGEM mudou.
///
/// ⚠️ É aqui que este nó difere do `motion.sort`, que preserva a contagem e por
/// isso deliberadamente NÃO cunha. Um `Count` ausente numa lista dobrada faz o
/// consumidor a jusante inventar o dele a partir da posição — a resposta certa por
/// acidente, até alguém pôr um `motion.combine` no meio.
#[test]
fn it_mints_both_columns_even_when_the_input_had_none() {
    let plain = run("motion.mirror.test.bare", 0.0, 0.0, 0.0);
    assert_eq!(scalar(&plain, INDEX), None, "sem o knob, não cunha nada");
    let out = run("motion.mirror.test.bare", 0.0, 0.0, 1.0);
    assert_eq!(scalar(&out, INDEX), Some(vec![0.0, 1.0, 2.0, 3.0]));
    assert_eq!(scalar(&out, COUNT), Some(vec![4.0; 4]));
}

/// **OS DOIS KNOBS ESTÃO PINTADOS, e são Toggles.**
#[test]
fn both_knobs_are_painted_as_toggles() {
    for p in [FLIP_ROT, REINDEX] {
        let h = PARAM_HINTS
            .iter()
            .find(|h| h.param == p)
            .unwrap_or_else(|| panic!("`{p}` tem de estar pintado"));
        assert!(matches!(h.widget, ParamWidget::Toggle), "`{p}` é um Toggle");
    }
}
