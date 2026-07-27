//! **A sonda da cena 56** — os números que a mensagem afirma, medidos sobre as
//! MESMAS peças que o artista abre.
//!
//! `#[ignore]` como as irmãs: ela imprime uma tabela, não afirma uma. Roda com
//! `cargo test -p ph2d-host-desktop --bin ph2d-host-desktop probe_smoke_56 -- --ignored --nocapture`.
//!
//! Mais dois gates NÃO-ignorados, porque a cena faz em letras grandes duas
//! afirmações que só ela pode falsificar: **a barra é a única das três que não
//! encurta**, e **a treliça segura o ápice**.

use super::{
    MEASURED_ROD_MIN, MEASURED_ROPE_MIN, MEASURED_SPRING_MIN, MEASURED_TRUSS_DRIFT, SPAN,
    spawn_props,
};
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::PhysicsBridge;

fn scene() -> (SimWorld, PhysicsBridge) {
    let mut sim = SimWorld::new();
    spawn_props(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    (sim, bridge)
}

fn by_name(sim: &SimWorld, want: &str) -> Entity {
    let mut q = sim.world().try_query::<(Entity, &Name)>().unwrap();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == want)
        .map(|(e, _)| e)
        .unwrap_or_else(|| panic!("a cena 56 não tem `{want}`"))
}

fn pos(sim: &SimWorld, e: Entity) -> [f32; 2] {
    let t = sim.world().get::<Transform>(e).unwrap();
    [t.translation.x, t.translation.y]
}

fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

/// A **menor** distância gancho→carga da coluna `label` ao longo de 3 s.
///
/// ⚠️ O mínimo da TRAJETÓRIA e não o valor final, porque um peso que oscila
/// deixa até uma corda TESA no fim: os três tipos terminam a 2 m, e o que os
/// separa acontece no caminho.
fn min_span(sim: &mut SimWorld, bridge: &mut PhysicsBridge, label: &str) -> f32 {
    let hook = by_name(sim, &format!("{label} Hook"));
    let load = by_name(sim, &format!("{label} Load"));
    let mut min = dist(pos(sim, hook), pos(sim, load));
    for tick in 1..=180 {
        bridge.dispatch(sim, true, tick);
        min = min.min(dist(pos(sim, hook), pos(sim, load)));
    }
    min
}

#[test]
#[ignore = "sonda de medição: imprime os números da cena 56"]
fn probe_smoke_56() {
    for label in ["Rope", "Spring", "Rod"] {
        let (mut sim, mut bridge) = scene();
        println!(
            "  {label:<7}: menor distancia {:.4} m (autorada {SPAN:.2})",
            min_span(&mut sim, &mut bridge, label)
        );
    }

    let (mut sim, mut bridge) = scene();
    let apex = by_name(&sim, "Truss Apex");
    let start = pos(&sim, apex);
    let mut drift = 0.0_f32;
    for tick in 1..=180 {
        bridge.dispatch(&mut sim, true, tick);
        drift = drift.max(dist(start, pos(&sim, apex)));
    }
    println!("  Trelica: o apice desvia no maximo {drift:.4} m em 3 s");
}

/// **A barra é a ÚNICA das três que não encurta** — a afirmação inteira do
/// passo 1 da mensagem, e a razão de as três colunas existirem lado a lado.
///
/// As três metades num gate só porque cada uma sozinha passa sobre a cena
/// errada: *"a barra mantém 2 m"* também vale numa cena sem gravidade, e
/// *"a corda encurta"* também vale se nada estiver preso a nada.
#[test]
fn only_the_rod_keeps_its_length_while_the_rope_and_the_spring_give() {
    let (mut sim, mut bridge) = scene();
    let rod = min_span(&mut sim, &mut bridge, "Rod");
    let (mut sim, mut bridge) = scene();
    let rope = min_span(&mut sim, &mut bridge, "Rope");
    let (mut sim, mut bridge) = scene();
    let spring = min_span(&mut sim, &mut bridge, "Spring");

    assert!(
        (rod - SPAN).abs() < 0.02,
        "a coluna VERDE tinha de manter {SPAN:.2} m em todo instante; encolheu para {rod:.4}"
    );
    assert!(
        rope < SPAN * 0.75,
        "o controle falhou: a coluna CIANO (corda) tinha de afrouxar, mas o \
         mínimo dela foi {rope:.4} m"
    );
    assert!(
        spring < rod - 0.01,
        "o controle falhou: a coluna AMARELA (mola) tinha de ceder mais que a \
         barra, mas mediu {spring:.4} contra {rod:.4}"
    );
}

/// **A treliça segura**, e é o que só duas barras fazem: uma corda só puxa, e o
/// ápice está exatamente na linha entre as âncoras.
#[test]
fn the_truss_holds_its_apex_in_place() {
    let (mut sim, mut bridge) = scene();
    let apex = by_name(&sim, "Truss Apex");
    let start = pos(&sim, apex);
    let mut drift = 0.0_f32;
    for tick in 1..=180 {
        bridge.dispatch(&mut sim, true, tick);
        drift = drift.max(dist(start, pos(&sim, apex)));
    }
    // ⚠️ **0,01 e não 0,05**, e o número veio de a fixture ser consertada: com as
    // âncoras a 4 m o ápice sentava na linha entre elas (a configuração
    // degenerada) e cedia 4,7 cm, o que passava raspando num limiar de 5 cm —
    // um gate verde sobre uma treliça que de fato afundava. Num triângulo de
    // verdade o desvio medido é 0,0000.
    assert!(
        drift < 0.01,
        "o ápice preso por duas barras nao pode viajar; desviou {drift:.4} m"
    );
}

/// **A MENSAGEM NÃO MENTE** — os quatro números que a cena imprime são os que a
/// simulação de fato produz.
///
/// Esta linha já shipou duas cenas afirmando coisas que a medição desmentiu, e
/// a resposta então foi *"rode a sonda antes de escrever a mensagem"*. Isto é a
/// versão executável dessa regra: a prosa e o produto passam a divergir com o
/// gate VERMELHO em vez de em silêncio, que é a única forma de a regra
/// sobreviver a quem não leu o handoff.
#[test]
fn the_scene_message_states_the_numbers_the_sim_produces() {
    for (label, claimed) in [
        ("Rope", MEASURED_ROPE_MIN),
        ("Spring", MEASURED_SPRING_MIN),
        ("Rod", MEASURED_ROD_MIN),
    ] {
        let (mut sim, mut bridge) = scene();
        let live = min_span(&mut sim, &mut bridge, label);
        assert!(
            (live - claimed).abs() < 0.01,
            "a mensagem da cena 56 diz {claimed:.2} m para `{label}` e a sim produz {live:.4}"
        );
    }

    let (mut sim, mut bridge) = scene();
    let apex = by_name(&sim, "Truss Apex");
    let start = pos(&sim, apex);
    let mut drift = 0.0_f32;
    for tick in 1..=180 {
        bridge.dispatch(&mut sim, true, tick);
        drift = drift.max(dist(start, pos(&sim, apex)));
    }
    assert!(
        (drift - MEASURED_TRUSS_DRIFT).abs() < 0.005,
        "a mensagem diz que o ápice desvia {MEASURED_TRUSS_DRIFT:.3} m e a sim produz {drift:.4}"
    );
}
