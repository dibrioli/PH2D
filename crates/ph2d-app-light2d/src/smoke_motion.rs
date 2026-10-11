//! ⭐ Cena 8 — **As formas do Motion** (W5e): a lâmpada e o chão sobre os objetos Motion do smoke do
//! Motion (`PH2D_MOTION_OBJETO_SMOKE=1`: a «Pile», formas que caem e se empilham — a simulação —, e o
//! «Swarm», cópias puxadas por campos). Cada objeto Motion ganha o «bloqueia a luz»: as formas dele
//! fazem sombra enquanto caem.
//!
//! O grafo do Motion não está no `World` (é do documento do Motion): a cena só põe a luz e marca os
//! objetos ([`marcar_motion`]) quando o smoke do Motion os cria — os dois roteadores juntos.

use bevy_ecs::world::World;
use ph2d_ecs::{MotionGenerator, Name, RootOrder};
use ph2d_light2d_ecs::{BloqueiaLuz, Luz, TipoDeLuz};

/// A lâmpada da cena 8 — acima das taças da «Pile» (a cena 114 do Motion, na origem).
pub const LAMPADA_DAS_FORMAS: [f32; 2] = [-1.5, 4.0];

/// Cena 8 — o chão no fundo, a luz ambiente e a lâmpada.
pub(crate) fn formas_do_motion(world: &mut World) {
    let chao = super::quadrado(
        world,
        "Chão",
        [1.5, 0.0],
        [16.0, 10.0],
        [0.6, 0.6, 0.62, 1.0],
    );
    world.entity_mut(chao).insert(RootOrder(0));
    super::ambiente(world, 0.03, [0.6, 0.7, 1.0]);
    let (t, g) = super::pose(LAMPADA_DAS_FORMAS[0], LAMPADA_DAS_FORMAS[1]);
    world.spawn((
        Name::new("Lâmpada"),
        Luz {
            cor: [1.0, 0.92, 0.8],
            intensidade: std::f32::consts::TAU * 1.5,
            tamanho_m: 0.25,
            alcance_m: f32::INFINITY,
            tipo: TipoDeLuz::Ponto,
        },
        t,
        g,
    ));
}

/// Os objetos Motion sem «bloqueia a luz» recebem-no; devolve quantos marcou (zero = o smoke do Motion
/// ainda não os criou).
pub fn marcar_motion(world: &mut World) -> usize {
    let novos: Vec<_> = world
        .query_filtered::<bevy_ecs::entity::Entity, (
            bevy_ecs::query::With<MotionGenerator>,
            bevy_ecs::query::Without<BloqueiaLuz>,
        )>()
        .iter(world)
        .collect();
    for &e in &novos {
        world.entity_mut(e).insert(BloqueiaLuz { opacidade: 1.0 });
    }
    novos.len()
}
