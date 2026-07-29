//! **As alças de uma RODLANA chegam ao barramento** (W-Pulley W6) — arch-gates
//! sobre as costuras window-gated que um unit test não dirige.
//!
//! O que É testável por comportamento já está: `ph2d_editor::gizmo::point` prova
//! que as três alças desenham e registram hit, `render_loop::point_gizmo` prova a
//! regra de publicação (o aro de saída só existe quando há um segundo raio), e
//! `ph2d-physics-ecs::pulley_mount` prova o que uma re-colocação de eixo FAZ. O
//! que nenhum deles alcança é a GLUE — o `advance_joint_anchor_drag` precisa de um
//! `AppGfx` vivo (janela + GPU), e o commit de Position mora dentro do
//! `render_loop`. Estas são as duas metades restantes da lista de UI, como
//! arch-gates sobre o fonte.

use std::fs;

/// A metade do gesto que é da RODLANA — módulo filho desde o split do W6, e é
/// nele que estas afirmações moram agora.
fn wheel_module() -> String {
    fs::read_to_string("src/joint_anchor_drag_wheel.rs").expect("joint_anchor_drag_wheel.rs")
}

/// **Mover o eixo de uma roldana MONTADA passa pela porta de re-colocação.**
///
/// Sem isso o gesto é **morto e silencioso**: o centro de uma montada é derivado
/// (`corpo · local`) e o `sync_mounted_wheels` o reescreve no frame seguinte — a
/// alça anda com o dedo e volta ao soltar, sem erro e sem aviso. O `Transform`
/// sozinho não basta, e é exatamente essa a metade que faltava.
#[test]
fn dragging_a_wheel_centre_reseats_a_mounted_axle() {
    let arm = wheel_module();
    assert!(
        arm.contains("t.translation = ph2d_core::Vec2::new(to[0], to[1])"),
        "o arrasto tem de escrever o Transform da roldana"
    );
    assert!(
        arm.contains("ph2d_physics_ecs::reseat_mounted_axle(sim.world_mut(), wheel)"),
        "…e tem de desarmar o sentinela pela porta compartilhada, senão numa \
         roldana MONTADA o próximo reconcile reescreve o número e o gesto some"
    );
}

/// **A row Position é o SEGUNDO gesto que re-coloca o mesmo eixo**, e passa pela
/// MESMA porta.
///
/// Dois gestos que dizem a mesma coisa não podem discordar sobre o que ela
/// significa — e uma regra escrita uma vez por chamador é a regra que o terceiro
/// chamador nasce sem. O sítio do joint (`set_joint_anchor_world`) está
/// imediatamente acima, e o da roldana tem de estar no mesmo bloco.
#[test]
fn committing_a_wheel_position_reseats_the_same_axle() {
    let src = fs::read_to_string("src/render_loop/mod.rs").expect("render_loop/mod.rs");
    let start = src
        .find("if let Some((bits, world)) = joint_pivot_commit {")
        .expect("o bloco de commit de pivô sumiu");
    let block = &src[start..start + 1400];
    assert!(
        block.contains("reseat_mounted_axle"),
        "o commit de Position numa roldana montada tem de re-colocar o eixo pela \
         mesma porta do dot de canvas"
    );
}

/// **As duas alças de raio perguntam a MESMA porta** — no agarre e na escrita.
///
/// O modo de falha que isto barra não é um crash: é agarrar o aro de SAÍDA,
/// medir o deslocamento contra o raio de ENTRADA e escrever o resultado no de
/// saída — a alça salta ao pegar e o número sai errado por uma diferença de
/// raios. Enumerar o par à mão nos dois lados é como o terceiro raio nasceria
/// mexendo em metade dos sítios.
#[test]
fn both_radius_handles_ask_one_door() {
    let parent = fs::read_to_string("src/joint_anchor_drag.rs").expect("joint_anchor_drag.rs");
    let wheel = wheel_module();
    assert!(
        parent.contains("wheel::open_grab(sim, joint, kind, cursor)")
            && parent.contains("wheel::resize_wheel(&mut gfx.sim, entity, drag.kind, cursor, off)"),
        "o agarre e o apply do raio têm de ir para a MESMA metade-roldana"
    );
    assert!(
        wheel.contains("if kind.is_wheel_radius() {"),
        "o agarre tem de perguntar a porta, não enumerar os dois kinds"
    );
    assert!(
        wheel.contains("wheel_radius_of(&w, kind)"),
        "o agarre tem de medir contra o raio DESTA alça"
    );
    // E o piso é UM: os dois raios passam pelo mesmo `max`, então o de saída não
    // pode chegar a zero — zero é o sentinela de *roldana comum*, e o arrasto
    // apagaria a própria alça sob o dedo.
    assert!(
        wheel.contains("let r = (distance(centre, cursor) + off).max(ph2d_physics_ecs::PulleyWheel::MIN_RADIUS);"),
        "o aro de saída tem de ter o mesmo piso do de entrada"
    );
}
