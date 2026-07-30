//! **O que a §13 escreve na roldana** (W-Pulley W1).
//!
//! A metade de shell da seção: o snapshot que ela lê e a edição que ela aplica.
//! Os gates de PAINEL (o clique chega, a caixa é semeada, o valor é espelhado)
//! moram em `ph2d-panel-inspector/tests/seam_wheel.rs`; aqui mora o que
//! acontece com o COMPONENTE.

use ph2d_ecs::scene::{EditorCommandQueue, apply_editor_commands};
use ph2d_ecs::{Name, SimWorld, Transform, stable_name_id};
use ph2d_editor::WheelFieldEdit;
use ph2d_physics_ecs::{JointKind, PhysicsJoint, PulleyWheel, WestonAxle, WrapSide};

use super::inspector_joint_tests::registry;
use super::inspector_joint_wheel::{apply_wheel_edit, build_wheel_info, wheel_with_edit};

fn wheel(order: u16) -> PulleyWheel {
    PulleyWheel {
        rope: stable_name_id("Rope"),
        order,
        radius: 0.45,
        wrap: WrapSide::Auto,
        motor_speed: 0.0,
        break_enabled: false,
        break_force: PulleyWheel::DEFAULT_BREAK_FORCE,
        ..Default::default()
    }
}

/// **A ordem é 1-based na tela e 0-based no componente**, e a conversão é uma
/// de cada lado.
///
/// ⚠️ Mutação: escrever `v as u16` sem o `-1` faz o 1º nó virar o 2º — a corda
/// re-roteia sozinha na primeira vez que alguém abre a seção e não toca em nada
/// (o `sync` semeia 1, o commit escreve 1, e o componente passa de 0 para 1).
#[test]
fn the_order_row_counts_from_one_and_the_component_from_zero() {
    let w = wheel(0);
    assert_eq!(
        wheel_with_edit(w, WheelFieldEdit::Order(1)).map(|n| n.order),
        Some(0),
        "o 1º nó da row é o índice 0 do componente"
    );
    assert_eq!(
        wheel_with_edit(w, WheelFieldEdit::Order(4)).map(|n| n.order),
        Some(3)
    );
    // E o degenerado que a fronteira do painel já barra, barrado de novo aqui:
    // `0 - 1` num `u16` seria `u16::MAX`, ou seja a roldana no FIM da corda.
    assert_eq!(
        wheel_with_edit(w, WheelFieldEdit::Order(0)).map(|n| n.order),
        Some(0),
        "um zero que escapasse não pode virar u16::MAX"
    );
}

/// **Um tag de Wrap que não nomeia variante nenhum é RECUSADO**, nunca dobrado
/// em `Auto`.
///
/// A lição do `BodyKind` do W4: com dois variants dobrar o desconhecido é
/// redundante, com o terceiro vira um chip que seleciona outra coisa.
#[test]
fn an_unknown_wrap_tag_is_refused_not_folded() {
    for (tag, want) in [
        (0, WrapSide::Auto),
        (1, WrapSide::Over),
        (2, WrapSide::Under),
    ] {
        assert_eq!(
            wheel_with_edit(wheel(0), WheelFieldEdit::Wrap(tag)).map(|n| n.wrap),
            Some(want)
        );
    }
    assert_eq!(
        wheel_with_edit(wheel(0), WheelFieldEdit::Wrap(3)),
        None,
        "o tag 3 não é um lado; dobrá-lo em Auto seria o chip escolhendo outra coisa"
    );
}

/// **O raio passa pela MESMA porta de carga do load** — negativo inverteria a
/// tangente, `NaN` envenenaria a pose e o hash C9.
#[test]
fn the_radius_goes_through_the_same_clamp_the_load_uses() {
    assert_eq!(
        wheel_with_edit(wheel(0), WheelFieldEdit::Radius(-2.0)).map(|n| n.radius),
        Some(PulleyWheel::MIN_RADIUS)
    );
    assert_eq!(
        wheel_with_edit(wheel(0), WheelFieldEdit::Radius(f32::NAN)).map(|n| n.radius),
        Some(PulleyWheel::DEFAULT_RADIUS)
    );
    // E o caminho normal não é tocado.
    assert_eq!(
        wheel_with_edit(wheel(0), WheelFieldEdit::Radius(0.8)).map(|n| n.radius),
        Some(0.8)
    );
}

/// **O snapshot resolve a corda pelo NOME, e só uma CORDA conta.**
///
/// ⚠️ Um sprite homônimo não põe a roldana em rota nenhuma — o `harvest` casa
/// `PulleyWheel::rope` com o nome de uma entidade-JOINT —, então dizer `bound`
/// por um nome que bate seria a seção afirmando uma ligação que o solver não
/// faz.
#[test]
fn the_snapshot_names_the_rope_and_only_a_rope_counts() {
    let mut sim = SimWorld::default();
    let rope = sim.world_mut().spawn((
        Name::new("Rope"),
        PhysicsJoint::of_kind(JointKind::Pulley),
        Transform::default(),
    ));
    let _ = rope;
    let w = sim
        .world_mut()
        .spawn((Name::new("Rope Wheel 1"), wheel(2), Transform::default()))
        .id();
    let info = build_wheel_info(&mut sim, w.to_bits(), false, false).expect("a roldana tem seção");
    assert!(info.bound);
    assert_eq!(info.rope_name, "Rope");
    assert_eq!(info.order_ui, 3, "o componente conta de zero, a row de um");
    assert_eq!(info.wrap_tag, 0);

    // A MESMA roldana com um homônimo que NÃO é corda.
    let mut sim = SimWorld::default();
    sim.world_mut()
        .spawn((Name::new("Rope"), Transform::default()));
    let w = sim
        .world_mut()
        .spawn((Name::new("Rope Wheel 1"), wheel(0), Transform::default()))
        .id();
    let info = build_wheel_info(&mut sim, w.to_bits(), false, false).expect("a roldana tem seção");
    assert!(
        !info.bound && info.rope_name.is_empty(),
        "um sprite chamado 'Rope' não é uma corda"
    );
}

/// **E a seção não existe para quem não é roldana** — a metade de ausência.
#[test]
fn a_body_has_no_wheel_section() {
    let mut sim = SimWorld::default();
    let e = sim
        .world_mut()
        .spawn((Name::new("Crate"), Transform::default()))
        .id();
    assert!(build_wheel_info(&mut sim, e.to_bits(), false, false).is_none());
}

/// **Graus na row, radianos no componente** — a fronteira do motor do Pin, e a
/// conversão acontece uma vez de cada lado.
///
/// ⚠️ Os dois sentidos num gate só de propósito: converter só na ida deixaria a
/// row mostrando o número certo e escrevendo o errado (ou o contrário), e cada
/// metade sozinha parece funcionar até alguém re-selecionar a roldana.
#[test]
fn the_motor_speaks_degrees_at_the_boundary_and_radians_inside() {
    let w = wheel(0);
    let next = wheel_with_edit(w, WheelFieldEdit::MotorDegPerS(180.0)).expect("aceita");
    assert!(
        (next.motor_speed - std::f32::consts::PI).abs() < 1.0e-5,
        "180 graus/s viraram {} rad/s",
        next.motor_speed
    );
    // E a volta, pela porta que o snapshot usa.
    let mut sim = SimWorld::default();
    sim.world_mut().spawn((
        Name::new("Rope"),
        PhysicsJoint::of_kind(JointKind::Pulley),
        Transform::default(),
    ));
    let e = sim
        .world_mut()
        .spawn((Name::new("Rope Wheel 1"), next, Transform::default()))
        .id();
    let info = build_wheel_info(&mut sim, e.to_bits(), false, false).expect("a roldana tem seção");
    assert!(
        (info.motor_deg_per_s - 180.0).abs() < 1.0e-3,
        "o snapshot devolveu {} graus/s",
        info.motor_deg_per_s
    );
    // O sinal atravessa: negativo paga corda.
    let paying = wheel_with_edit(w, WheelFieldEdit::MotorDegPerS(-90.0)).expect("aceita");
    assert!(paying.motor_speed < 0.0, "o sinal foi engolido");
}

/// **Um motor `NaN` é recusado pela MESMA porta de carga do raio** — sem ela um
/// `NaN` viraria um comprimento de corda `NaN` no primeiro sub-passo, e daí a
/// pose e o hash C9.
#[test]
fn a_non_finite_motor_goes_through_the_same_clamp() {
    let w = wheel(0);
    assert_eq!(
        wheel_with_edit(w, WheelFieldEdit::MotorDegPerS(f32::NAN)).map(|n| n.motor_speed),
        Some(0.0)
    );
}

/// **O par switch/limiar do eixo atravessa a fronteira intacto**, e o limiar
/// passa pela MESMA porta de carga que o raio e o motor.
///
/// ⚠️ Um limiar NEGATIVO partiria a roldana antes de a corda tocá-la, e um `NaN`
/// **nunca compara verdadeiro** — ou seja, seria um eixo indestrutível com a
/// caixa marcada, que é o pior dos dois erros porque não parece um erro.
#[test]
fn the_axle_threshold_goes_through_the_same_clamp() {
    let w = wheel(0);
    assert_eq!(
        wheel_with_edit(w, WheelFieldEdit::BreakEnabled(true)).map(|n| n.break_enabled),
        Some(true)
    );
    assert_eq!(
        wheel_with_edit(w, WheelFieldEdit::BreakForce(250.0)).map(|n| n.break_force),
        Some(250.0)
    );
    assert_eq!(
        wheel_with_edit(w, WheelFieldEdit::BreakForce(-5.0)).map(|n| n.break_force),
        Some(0.0),
        "um limiar negativo é clampado, não guardado"
    );
    assert_eq!(
        wheel_with_edit(w, WheelFieldEdit::BreakForce(f32::NAN)).map(|n| n.break_force),
        Some(PulleyWheel::DEFAULT_BREAK_FORCE),
        "NaN nunca compara verdadeiro: seria um eixo indestrutível com a caixa marcada"
    );
}

/// **O chip de WESTON anexa e desanexa o MARCADOR — e re-abre o `L0`.**
///
/// ⚠️ **É o único toggle desta seção que devolve `true` sempre**, e a razão é
/// estrutural: armar a Weston ACRESCENTA um nó à rota (o contato de retorno) e re-pesa
/// a corda, então o comprimento derivado tem de ser re-semeado — e o `route_differs`,
/// que compara campos da `PulleyWheel`, é **cego** a um componente ao lado dela.
///
/// ⚠️ Mutação: rotear a Weston pelo funil de campo (o `wheel_with_edit`) não escreve
/// componente nenhum e devolve `false` — o chip fica marcado na tela e a máquina não
/// muda, que é a forma exata de um controle morto.
#[test]
fn the_weston_chip_attaches_the_marker_and_reopens_the_length() {
    let mut sim = SimWorld::default();
    let reg = registry();
    let e = sim
        .world_mut()
        .spawn((
            Name::new("Sheave"),
            PulleyWheel {
                rope: stable_name_id("Rope"),
                radius: 0.5,
                radius_out: 0.375,
                ..Default::default()
            },
            Transform::default(),
        ))
        .id();
    let bits = e.to_bits();
    let apply = |sim: &mut SimWorld, on: bool| -> bool {
        let queue = EditorCommandQueue::default();
        let changed = apply_wheel_edit(sim, bits, WheelFieldEdit::Weston(on), &queue, &reg);
        apply_editor_commands(sim.world_mut(), &queue, &reg).expect("commands apply");
        changed
    };

    assert!(
        sim.world().get::<WestonAxle>(e).is_none(),
        "uma roldana nasce TAMBOR — a talha é um gesto"
    );
    assert!(apply(&mut sim, true), "armar a Weston muda a ROTA");
    assert!(
        sim.world().get::<WestonAxle>(e).is_some(),
        "o marcador anexou"
    );
    assert!(apply(&mut sim, false), "desarmar também muda a rota");
    assert!(
        sim.world().get::<WestonAxle>(e).is_none(),
        "e o arquivo não carrega um marcador desligado"
    );
}

/// **O funil de campo RECUSA a Weston** — ela não mora em campo nenhum.
///
/// Devolver `Some(next)` inalterado ali faria a edição parecer aplicada e o
/// `apply_wheel_edit` enfileirar uma escrita que não muda nada.
#[test]
fn the_field_funnel_refuses_the_weston_because_it_is_a_marker() {
    let w = PulleyWheel {
        radius: 0.5,
        radius_out: 0.375,
        ..Default::default()
    };
    assert!(wheel_with_edit(w, WheelFieldEdit::Weston(true)).is_none());
    assert!(wheel_with_edit(w, WheelFieldEdit::Weston(false)).is_none());
}

/// **O readout `Gear` sai da porta do MOTOR**, e ele muda de lei com o chip.
///
/// `R/r` num tambor (0,5/0,375 = 1,333) e `R/(R−r)` numa Weston (0,5/0,125 = 4).
/// ⚠️ Uma conta escrita no painel mostraria um número e o solver usaria outro — o
/// defeito exato do `ratio` DIGITADO que o W4 aposentou.
#[test]
fn the_gear_readout_changes_law_with_the_chip() {
    let mut sim = SimWorld::default();
    let e = sim
        .world_mut()
        .spawn((
            Name::new("Sheave"),
            PulleyWheel {
                rope: stable_name_id("Rope"),
                radius: 0.5,
                radius_out: 0.375,
                ..Default::default()
            },
            Transform::default(),
        ))
        .id();
    let bits = e.to_bits();
    let drum = build_wheel_info(&mut sim, bits, false, false).expect("tem seção");
    assert!(!drum.weston);
    assert!(
        (drum.gear - 0.5 / 0.375).abs() < 1.0e-6,
        "o tambor compra R/r; deu {}",
        drum.gear
    );
    sim.world_mut().entity_mut(e).insert(WestonAxle);
    let weston = build_wheel_info(&mut sim, bits, false, false).expect("tem seção");
    assert!(weston.weston);
    assert!(
        (weston.gear - 4.0).abs() < 1.0e-6,
        "a Weston compra R/(R−r); deu {}",
        weston.gear
    );
    // E sem segundo diâmetro não há engrenagem nenhuma a mostrar.
    if let Some(mut pw) = sim.world_mut().get_mut::<PulleyWheel>(e) {
        pw.radius_out = 0.0;
    }
    let plain = build_wheel_info(&mut sim, bits, false, false).expect("tem seção");
    assert_eq!(plain.gear, 1.0);
}
