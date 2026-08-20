//! Os gates do [`super::SPACE`] — o espaço do deslocamento (doc 89, folha 05).

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};

/// Uma fonte de DOIS elementos na origem, orientados a `0°` e a `90°`.
///
/// ⚠️ **Os dois estão no MESMO ponto de propósito.** O que o modo Local muda é a
/// direção do passo, e pôr as peças em sítios diferentes deixaria a diferença de
/// posições explicável por onde elas começaram. Partindo do mesmo ponto, a posição
/// final **é** o offset.
static SRC_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.move.test.turned"),
    name: "motion.move.test.turned",
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

struct Turned;
impl NodeOp for Turned {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(
            Stream::new(2)
                .with("P", Column::Vec2(vec![[0.0, 0.0], [0.0, 0.0]]))
                .with("rot", Column::Scalar(vec![0.0, 90.0])),
        );
    }
}

/// A mesma fonte SEM a coluna de orientação.
static BARE_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.move.test.bare"),
    name: "motion.move.test.bare",
    ..SRC_MAN
};

struct Bare;
impl NodeOp for Bare {
    fn manifest(&self) -> &'static NodeManifest {
        &BARE_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(Stream::new(2).with("P", Column::Vec2(vec![[0.0, 0.0], [0.0, 0.0]])));
    }
}

struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC_MAN.id => Some(&Turned),
            t if t == BARE_MAN.id => Some(&Bare),
            t if t == MANIFEST.id => Some(&MotionMove),
            _ => None,
        }
    }
}

/// Coze `src → motion.move(dx, dy, space)` e devolve as posições e o `rot` de saída.
fn run(src: &str, dx: f32, dy: f32, space: f32) -> (Vec<[f32; 2]>, Option<Vec<f32>>) {
    let mut g = Graph::new();
    let s = g.add_node(src);
    let mv = g.add_node("motion.move");
    g.connect(Edge {
        from: (s, 0),
        to: (mv, 0),
        delayed: false,
    })
    .unwrap();
    g.set_param(mv, "dx", dx);
    g.set_param(mv, "dy", dy);
    g.set_param(mv, SPACE, space);
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, mv, 0.0).unwrap();
    let st = out[0].as_stream();
    let p = match st.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("P"),
    };
    let r = match st.get("rot") {
        Some(Column::Scalar(v)) => Some(v.clone()),
        _ => None,
    };
    (p, r)
}

/// **O MUNDO É O NÓ QUE SEMPRE SHIPOU, AO BIT — mesmo sobre uma lista orientada.**
///
/// ⚠️ Este é o gate que separa *"o default está certo"* de *"o default não foi
/// tocado"*: a fonte TEM `rot = 90°` no segundo elemento, então se o modo Mundo
/// tivesse caído no caminho novo por engano o segundo ponto iria para `(0, 3)`.
#[test]
fn world_ignores_the_rotation_column_entirely() {
    let (p, _) = run("motion.move.test.turned", 3.0, 0.0, 0.0);
    assert_eq!(p, vec![[3.0, 0.0], [3.0, 0.0]]);
}

/// **LOCAL ANDA PARA A FRENTE DE CADA UM** — a mesma caixa, dois sentidos.
#[test]
fn local_turns_the_offset_by_each_elements_own_rotation() {
    let (p, _) = run("motion.move.test.turned", 3.0, 0.0, SPACE_LOCAL);
    assert!(
        (p[0][0] - 3.0).abs() < 1e-5 && p[0][1].abs() < 1e-5,
        "a `0°` o passo continua a ser o de mundo: {:?}",
        p[0]
    );
    // A `90°` o mesmo `(3, 0)` aponta para +Y. A barra é a da senoide parabólica
    // (HR-5) — e neste ângulo ela é EXACTA, ver `trig::anchors_match_true_trig`.
    assert!(
        p[1][0].abs() < 1e-5 && (p[1][1] - 3.0).abs() < 1e-5,
        "a `90°` o passo tem de correr em Y: {:?}",
        p[1]
    );
}

/// **UMA LISTA SEM ORIENTAÇÃO EM LOCAL É O MOVIMENTO DE MUNDO, AO BIT.**
///
/// ⚠️ E o gate compara com o próprio nó, não com números escritos à mão: a
/// igualdade que ele defende é *"as duas portas dão a MESMA resposta"*, que é a
/// premissa de que o kernel do device depende (a binding `rot` tem `identity: 0`).
#[test]
fn a_local_move_without_a_rot_column_is_the_world_move() {
    let (world, _) = run("motion.move.test.bare", 2.0, -1.0, 0.0);
    let (local, _) = run("motion.move.test.bare", 2.0, -1.0, SPACE_LOCAL);
    assert_eq!(world, local);
}

/// **ESTE NÓ MOVE E NÃO VIRA** — o `rot` atravessa intacto nos dois modos.
///
/// ⚠️ A cerca executável da lei do doc-comment. Um `motion.move` que escrevesse
/// `rot` deixaria de compor com o `motion.rotate` em qualquer ordem.
#[test]
fn the_rotation_column_passes_through_untouched() {
    for space in [0.0, SPACE_LOCAL] {
        let (_, r) = run("motion.move.test.turned", 5.0, 5.0, space);
        assert_eq!(
            r,
            Some(vec![0.0, 90.0]),
            "space {space}: o `rot` não é deste nó"
        );
    }
}

/// **A ROTAÇÃO GIRA NO SENTIDO ANTI-HORÁRIO, e não é a sua transposta.**
///
/// ⚠️ **Este gate nasceu de uma MUTAÇÃO SOBREVIVENTE.** Trocar `dx·c − dy·s` por
/// `dx·c + dy·s` (a rotação por `−θ`) passava por todos os outros oráculos deste
/// arquivo, porque todos usavam `dy = 0` — e com `dy = 0` as duas expressões são
/// **a mesma**. Um vetor sobre um eixo não distingue o sentido de uma rotação;
/// só um oblíquo distingue. O comprimento também não: as duas formas são
/// isometrias.
///
/// Os quatro quartos de volta são exactos na senoide parabólica (ver [`trig`]),
/// então aqui a barra é de `f32` e não de aproximação.
#[test]
fn the_local_offset_turns_counter_clockwise_and_not_its_transpose() {
    // A `90°`: `(1, 0) → (0, 1)` e `(0, 1) → (−1, 0)`. A transposta trocaria os dois.
    let (ax, ay) = offset_for(1.0, 0.0, Some(90.0));
    assert!(ax.abs() < 1e-6 && (ay - 1.0).abs() < 1e-6, "({ax}, {ay})");
    let (bx, by) = offset_for(0.0, 1.0, Some(90.0));
    assert!(
        (bx + 1.0).abs() < 1e-6 && by.abs() < 1e-6,
        "o eixo Y local tem de ir para −X, e foi para ({bx}, {by})"
    );
    // E um ângulo OBLÍQUO sobre um vetor OBLÍQUO — o caso geral que a mutação
    // atravessava. `(1, 1)` a `45°` sobe para o eixo Y; a transposta desceria ao X.
    let (cx, cy) = offset_for(1.0, 1.0, Some(45.0));
    assert!(
        cx.abs() < 5e-3 && cy > 1.0,
        "`(1, 1)` a 45° tem de virar vertical, e deu ({cx:.4}, {cy:.4})"
    );
}

/// **O OFFSET É UMA ROTAÇÃO DE VETOR, e a prova é o comprimento.**
///
/// ⚠️ O oráculo não é *"o resultado muda"* — é que o passo conserva o módulo em
/// qualquer ângulo. Se a direção fosse (por exemplo) um `dx` escalado pelo cosseno,
/// o comprimento encolheria; a barra de 0,5% é a da senoide parabólica (HR-5), que
/// não é norm-preserving ao bit.
#[test]
fn the_local_offset_is_a_rotation_and_keeps_its_length() {
    for deg in [0.0f32, 17.0, 90.0, 200.0, 359.0] {
        let (ox, oy) = offset_for(3.0, 4.0, Some(deg));
        let len = ox.hypot(oy);
        assert!(
            (len - 5.0).abs() < 5.0 * 5e-3,
            "a {deg}° o passo mede {len:.4} e tinha de medir 5"
        );
    }
}

/// **O KNOB ESTÁ PINTADO, e o device tem uma VARIANTE por modo.**
#[test]
fn the_knob_is_painted_and_the_device_has_one_variant_per_mode() {
    let h = PARAM_HINTS
        .iter()
        .find(|h| h.param == SPACE)
        .expect("o Space tem de estar pintado");
    match h.widget {
        ParamWidget::Enum { labels } => assert_eq!(labels, &["World", "Local"]),
        _ => panic!("o Space é um Enum"),
    }
    let world = GPU_KERNEL.resolve(&|_| 0.0);
    let local = GPU_KERNEL.resolve(&|n| if n == SPACE { SPACE_LOCAL } else { 0.0 });
    assert_eq!(
        world.bindings.len(),
        2,
        "o modo Mundo NÃO pode pedir a coluna `rot` ao plano"
    );
    assert!(
        local.bindings.iter().any(|b| b.column == "rot"),
        "o modo Local tem de ler a orientação no device"
    );
    assert_ne!(world.wgsl, local.wgsl, "e os corpos têm de diferir");
}
