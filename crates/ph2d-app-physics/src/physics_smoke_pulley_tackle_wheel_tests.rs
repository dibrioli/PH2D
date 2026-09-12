//! **A metade da fronteira**: os gates da cena `physics_smoke_pulley_tackle` que exercitam um
//! GESTO DA SHELL (`crate::physics::…`) sobre uma cena que vive na crate da família.
//!
//! ⚠️ **Eles estão aqui porque um `#[cfg(test)]` é INVISÍVEL do outro lado da
//! fronteira de crate** (HOWTO §2): o sujeito é da crate, o gesto é da shell, e um
//! teste não pode ter um pé em cada. Os outros gates da mesma cena ficaram **com o
//! sujeito**, na crate — *o corte é por quem o teste EXERCITA, não por quem ele nomeia.*

use crate::physics_smoke_pulley_tackle::*;
use ph2d_ecs::{Entity, Name, SimWorld};
use ph2d_physics_ecs::PhysicsBridge;

fn entity_of(sim: &mut SimWorld, name: &str) -> Entity {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .expect("entidade viva")
}

/// **A montagem é autorável com dois cliques** — o conta-gotas arma, o clique no
/// corpo monta, e a lixeira desmonta.
///
/// ⚠️ Este gate é da CATEGORIA que a política de UI do plano chama de quarta: as
/// outras três (o componente existe · é pintado e registrado · o clique chega ao
/// barramento) podem estar todas verdes com a **sequência não levando a lugar
/// nenhum**. Aqui ela é dirigida de ponta a ponta pelas portas do produto.
#[test]
fn the_mount_gesture_leads_somewhere() {
    let mut sim = SimWorld::new();
    build_tackle(sim.world_mut());
    ph2d_physics_ecs::resolve_body_names(sim.world_mut());
    let wheel = entity_of(&mut sim, "Plain Rope Wheel 1");
    let block = entity_of(&mut sim, "Plain Block");
    let read = |sim: &mut SimWorld| {
        *sim.world()
            .get::<ph2d_physics_ecs::PulleyWheel>(wheel)
            .expect("a roldana existe")
    };
    assert_eq!(read(&mut sim).body, 0, "ela nasce no CENÁRIO");

    // O clique no corpo, pela porta que o pick de canvas termina.
    crate::joint_wheel::set_wheel_mount(&mut sim, wheel.to_bits(), block);
    let mounted = read(&mut sim);
    assert_eq!(
        mounted.body,
        ph2d_ecs::stable_id_for_name(sim.world_mut(), "Plain Block"),
        "o conta-gotas tinha de montar a roldana no bloco clicado"
    );
    assert!(
        !mounted.mounted,
        "o local é semeado pela PONTE, contra a pose de repouso — derivá-lo aqui \
         seria a segunda porta"
    );

    // Um dispatch semeia o local e marca a montagem.
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    let seeded = read(&mut sim);
    assert!(seeded.mounted, "a ponte tinha de semear o eixo local");

    // E a lixeira desfaz — pela porta pura, que é a que o painel alcança.
    let unmounted =
        crate::joint_wheel::wheel_with_edit(seeded, ph2d_editor_core::WheelFieldEdit::Unmount)
            .expect("desmontar é uma escrita de componente");
    assert_eq!(unmounted.body, 0, "a lixeira tinha de voltar ao cenário");
    assert!(
        !unmounted.mounted,
        "o sentinela tem de voltar junto: um local semeado descreve um frame que \
         não é mais o de ninguém, e a próxima montagem o herdaria em silêncio"
    );
}

/// **O eyedropper ARMA, ele não escreve** — a metade que separa este gesto de uma
/// edição de número.
#[test]
fn arming_the_mount_pick_writes_nothing() {
    let wheel = ph2d_physics_ecs::PulleyWheel::default();
    assert!(
        crate::joint_wheel::wheel_with_edit(wheel, ph2d_editor_core::WheelFieldEdit::PickMountBody,)
            .is_none(),
        "armar o pick não pode ser uma escrita de componente: o alvo vem do \
         próximo clique no canvas"
    );
}
