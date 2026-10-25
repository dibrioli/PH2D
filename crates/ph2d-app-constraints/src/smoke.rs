//! **As cenas de smoke das constraints** — `PH2D_CONSTRAINT_SMOKE=<n>` (plano §6).
//!
//! | n | o que ensina |
//! |---|---|
//! | 2 | **Olho** — arraste a BOLA vermelha: a SETA aponta para ela (Look At) e a PUPILA corre atrás dela (Copy Location) sem sair do olho (Limit Distance, por baixo na pilha) |
//! | 3 | **Ponteiros** — os dois autorados a 170°: o de cima tem limite −45°…45° e pára na borda MAIS PERTO (45°); o de baixo copia a rotação da GUIA (−170°) a 50 % e fica a 180° (o arco curto, não 0°) |

use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use ph2d_constraints::{
    Canais, CopyTransform, Distance, DistanceMode, Kind, Limit, LookAt, Space,
};
use ph2d_constraints_ecs::Constraints;
use ph2d_ecs::{GlobalTransform, Name, Transform, assign_missing_stable_ids, stable_id_of};
use ph2d_render::{DISC_TILE_KEY, Sprite, WHITE_TILE_KEY};

/// O roteador desta família — o maior nível que [`encenar`] responde.
pub const ROUTERS: &[ph2d_app_host::SmokeRouter] = &[ph2d_app_host::SmokeRouter {
    env: "PH2D_CONSTRAINT_SMOKE",
    max_level: 3,
}];

/// O raio do olho da cena 2 e a folga da pupila dentro dele (o número que a sonda mede).
pub const RAIO_DO_OLHO: f32 = 1.0;
pub const FOLGA_DA_PUPILA: f64 = 0.6;

/// O nível armado por variável de ambiente (`None` = inerte).
#[must_use]
pub fn nivel_armado() -> Option<u32> {
    std::env::var("PH2D_CONSTRAINT_SMOKE")
        .ok()
        .and_then(|v| v.trim().parse::<u32>().ok())
}

/// Povoa `world` com a cena `nivel`. Devolve se havia cena para esse nível.
pub fn encenar(world: &mut World, nivel: u32) -> bool {
    match nivel {
        2 => olho(world),
        3 => ponteiros(world),
        _ => return false,
    }
    true
}

fn peca(
    world: &mut World,
    nome: &str,
    tile: u32,
    centro: [f32; 2],
    rot_deg: f32,
    lado: [f32; 2],
    cor: [f32; 4],
) -> Entity {
    let mut t = Transform::IDENTITY;
    t.translation.x = centro[0];
    t.translation.y = centro[1];
    t.rotation = rot_deg.to_radians();
    world
        .spawn((
            Name::new(nome),
            Sprite::atlas(tile, lado, cor),
            t,
            GlobalTransform::from_transform(t),
        ))
        .id()
}

/// Empilha `kinds` em `dono`, cada um com o alvo `alvo` (por identidade durável).
fn pilha(world: &mut World, dono: Entity, itens: &[(Kind, Option<Entity>, f64)]) {
    assign_missing_stable_ids(world);
    let mut c = Constraints::default();
    for &(kind, alvo, forca) in itens {
        let id = c.push(kind, alvo.and_then(|a| stable_id_of(world, a)));
        if let Some(it) = c.item_mut(id) {
            it.influence = forca;
        }
    }
    world.entity_mut(dono).insert(c);
}

/// Cena 2 — **Olho.**
fn olho(world: &mut World) {
    let d = 2.0 * RAIO_DO_OLHO;
    let olho = peca(world, "Olho", DISC_TILE_KEY, [0.0, 0.0], 0.0, [d, d], [0.95, 0.95, 0.92, 1.0]);
    let bola = peca(world, "Bola", DISC_TILE_KEY, [3.0, 1.2], 0.0, [0.5, 0.5], [0.9, 0.15, 0.1, 1.0]);
    let pupila = peca(world, "Pupila", DISC_TILE_KEY, [0.0, 0.0], 0.0, [0.7, 0.7], [0.08, 0.08, 0.1, 1.0]);
    let seta = peca(world, "Seta", WHITE_TILE_KEY, [-3.0, -1.5], 0.0, [1.4, 0.18], [0.2, 0.45, 0.9, 1.0]);
    let copia = Kind::CopyTransform(CopyTransform {
        canais: Canais::LOCATION,
        space: Space::World,
        ..CopyTransform::default()
    });
    let dentro = Kind::Distance(Distance {
        distance: FOLGA_DA_PUPILA,
        mode: DistanceMode::Inside,
    });
    pilha(world, pupila, &[(copia, Some(bola), 1.0), (dentro, Some(olho), 1.0)]);
    pilha(world, seta, &[(Kind::LookAt(LookAt::default()), Some(bola), 1.0)]);
}

/// Cena 3 — **Ponteiros.**
fn ponteiros(world: &mut World) {
    let lado = [2.0, 0.2];
    let limitado = peca(world, "Ponteiro com limite", WHITE_TILE_KEY, [0.0, 1.5], 170.0, lado, [0.9, 0.55, 0.1, 1.0]);
    let meio = peca(world, "Ponteiro a meio", WHITE_TILE_KEY, [0.0, -1.5], 170.0, lado, [0.2, 0.7, 0.35, 1.0]);
    let guia = peca(world, "Guia", WHITE_TILE_KEY, [3.5, -1.5], -170.0, lado, [0.6, 0.6, 0.6, 1.0]);
    let q = std::f64::consts::FRAC_PI_4;
    let limite = Kind::Limit(Limit {
        rot: Some([-q, q]),
        ..Limit::default()
    });
    pilha(world, limitado, &[(limite, None, 1.0)]);
    let copia = Kind::CopyTransform(CopyTransform {
        canais: Canais::ROTATION,
        space: Space::World,
        ..CopyTransform::default()
    });
    pilha(world, meio, &[(copia, Some(guia), 0.5)]);
}
