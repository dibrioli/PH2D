//! Os gates do [`super::CARRY_ROTATION`] — a órbita que leva a orientação
//! (doc 89, folha 05).

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};

/// Dois elementos no aro de um círculo de raio 1, com orientação autorada e um
/// `falloff` que zera o segundo.
///
/// ⚠️ **A máscara é a metade que separa** *"o ângulo é levado"* de *"o ângulo é
/// somado"*: um elemento com `falloff = 0` não pode mover-se NEM virar, e uma
/// implementação que escrevesse `rot + θ` sem o peso passaria por todos os outros
/// oráculos.
static SRC_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.orbit.test.ring"),
    name: "motion.orbit.test.ring",
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

struct Ring;
impl NodeOp for Ring {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(
            Stream::new(2)
                .with("P", Column::Vec2(vec![[1.0, 0.0], [0.0, 1.0]]))
                .with("rot", Column::Scalar(vec![10.0, 20.0]))
                .with("falloff", Column::Scalar(vec![1.0, 0.0])),
        );
    }
}

/// A mesma lista SEM orientação — o caso em que a coluna é cunhada.
static BARE_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.orbit.test.bare"),
    name: "motion.orbit.test.bare",
    ..SRC_MAN
};

struct BareRing;
impl NodeOp for BareRing {
    fn manifest(&self) -> &'static NodeManifest {
        &BARE_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(Stream::new(2).with("P", Column::Vec2(vec![[1.0, 0.0], [0.0, 1.0]])));
    }
}

struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC_MAN.id => Some(&Ring),
            t if t == BARE_MAN.id => Some(&BareRing),
            t if t == MANIFEST.id => Some(&MotionOrbit),
            _ => None,
        }
    }
}

/// Coze `src → motion.orbit(angle, carry)` com `speed = 0` (uma reposição estática,
/// para o oráculo não depender do playhead).
fn run(src: &str, angle: f32, carry: f32) -> Stream {
    let mut g = Graph::new();
    let s = g.add_node(src);
    let o = g.add_node("motion.orbit");
    g.connect(Edge {
        from: (s, 0),
        to: (o, 0),
        delayed: false,
    })
    .unwrap();
    g.set_param(o, "angle", angle);
    g.set_param(o, "speed", 0.0);
    g.set_param(o, CARRY_ROTATION, carry);
    let mut cook = Cook::new();
    cook.cook(&g, &Ops, o, 0.0).unwrap()[0].as_stream().clone()
}

fn rot_of(st: &Stream) -> Option<Vec<f32>> {
    match st.get("rot") {
        Some(Column::Scalar(v)) => Some(v.clone()),
        _ => None,
    }
}

fn p_of(st: &Stream) -> Vec<[f32; 2]> {
    match st.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("P"),
    }
}

/// **DESLIGADO, A ORIENTAÇÃO ATRAVESSA INTACTA** — e uma lista sem ela continua sem
/// ela (o nó não cunha coluna nenhuma).
#[test]
fn with_the_knob_off_the_rotation_column_is_untouched_and_never_minted() {
    let out = run("motion.orbit.test.ring", 90.0, 0.0);
    assert_eq!(rot_of(&out), Some(vec![10.0, 20.0]));
    let bare = run("motion.orbit.test.bare", 90.0, 0.0);
    assert_eq!(
        rot_of(&bare),
        None,
        "a forma do stream de saída é contrato com quem está a jusante"
    );
}

/// **LIGADO, A ORIENTAÇÃO SEGUE O MESMO `θ` DA POSIÇÃO.**
#[test]
fn the_carried_rotation_is_the_same_angle_the_position_travelled() {
    let out = run("motion.orbit.test.ring", 90.0, 1.0);
    let rot = rot_of(&out).expect("rot");
    assert!(
        (rot[0] - 100.0).abs() < 1e-4,
        "10° + a volta de 90° = 100°, e deu {}",
        rot[0]
    );
    // E a posição foi a mesma que sempre foi: (1,0) a 90° é (0,1).
    let p = p_of(&out);
    assert!(
        p[0][0].abs() < 1e-4 && (p[0][1] - 1.0).abs() < 1e-4,
        "{p:?}"
    );
}

/// **A MÁSCARA PESA OS DOIS** — `falloff = 0` não move e não vira.
///
/// ⚠️ É o gate que impede a máscara de significar duas coisas no mesmo nó.
#[test]
fn the_falloff_weights_the_turn_exactly_as_it_weights_the_move() {
    let out = run("motion.orbit.test.ring", 90.0, 1.0);
    let rot = rot_of(&out).expect("rot");
    assert_eq!(rot[1], 20.0, "o elemento mascarado não pode virar");
    let p = p_of(&out);
    assert!(
        (p[1][0]).abs() < 1e-6 && (p[1][1] - 1.0).abs() < 1e-6,
        "…nem mover-se: {p:?}"
    );
}

/// **UMA LISTA SEM `rot` COMEÇA EM ZERO E RECEBE A COLUNA.**
///
/// ⚠️ E o número é o mesmo que o `identity: 0.0` da binding do device dá — é isso
/// que faz a variante `ORBIT_CARRY` poder cunhar com `ReadWrite` sem divergir.
#[test]
fn an_absent_rotation_column_starts_at_zero_and_is_minted() {
    let out = run("motion.orbit.test.bare", 45.0, 1.0);
    let rot = rot_of(&out).expect("a coluna tem de nascer neste modo");
    assert!((rot[0] - 45.0).abs() < 1e-4, "{rot:?}");
    assert!((rot[1] - 45.0).abs() < 1e-4, "{rot:?}");
}

/// **UMA VOLTA INTEIRA MOVE A ORIENTAÇÃO E NÃO A POSIÇÃO** — o par que prova que
/// os dois canais são o mesmo `θ` e não a mesma FUNÇÃO.
///
/// ⚠️ A `360°` a posição volta ao sítio (a órbita é cíclica) mas a orientação
/// **acumulou** uma volta. Uma implementação que derivasse `rot` da posição (o
/// atalho tentador: `atan2` do vetor ao pivô) daria `+0` aqui, e passaria por
/// todos os gates acima.
#[test]
fn a_full_turn_moves_the_heading_and_not_the_position() {
    let out = run("motion.orbit.test.ring", 360.0, 1.0);
    let p = p_of(&out);
    assert!(
        (p[0][0] - 1.0).abs() < 1e-3 && p[0][1].abs() < 1e-3,
        "a peça voltou ao sítio: {p:?}"
    );
    let rot = rot_of(&out).expect("rot");
    assert!(
        (rot[0] - 370.0).abs() < 1e-4,
        "e a orientação acumulou: {rot:?}"
    );
}

/// **O KNOB ESTÁ PINTADO e o device tem uma VARIANTE que escreve `rot`.**
#[test]
fn the_knob_is_painted_and_the_device_has_a_variant_that_writes_rot() {
    let h = PARAM_HINTS
        .iter()
        .find(|h| h.param == CARRY_ROTATION)
        .expect("o Carry Rotation tem de estar pintado");
    assert!(matches!(h.widget, ParamWidget::Toggle));
    let plain = GPU_KERNEL.resolve(&|_| 0.0);
    let carry = GPU_KERNEL.resolve(&|n| if n == CARRY_ROTATION { 1.0 } else { 0.0 });
    assert!(
        !plain.bindings.iter().any(|b| b.column == "rot"),
        "desligado o device não pode materializar `rot`"
    );
    assert!(
        carry.bindings.iter().any(|b| b.column == "rot"),
        "ligado tem de o escrever"
    );
    assert!(carry.wgsl.contains("write_rot"));
}
