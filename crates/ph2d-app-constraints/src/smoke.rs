//! **As cenas de smoke das constraints** — `PH2D_CONSTRAINT_SMOKE=<n>` (plano §6).
//!
//! | n | o que ensina |
//! |---|---|
//! | 1 | **Mão e espada** — a MÃO passa (curva da timeline); no 24 pega a ESPADA do chão (Child Of, força 0→1 com o Auto Key), no 48 larga-a noutro sítio; nem pegar nem largar salta, e o antes do 24 continua no chão (as chaves de compensação, `compensa`) |
//! | 2 | **Olho** — arraste a BOLA vermelha: a SETA aponta para ela (Look At) e a PUPILA corre atrás dela (Copy Location) sem sair do olho (Limit Distance, por baixo na pilha) |
//! | 3 | **Ponteiros** — os dois autorados a 170°: o de cima tem limite −45°…45° e pára na borda MAIS PERTO (45°); o de baixo copia a rotação da GUIA (−170°) a 50 % e fica a 180° (o arco curto, não 0°) |

use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Resource;
use bevy_ecs::world::World;
use ph2d_constraints::{
    Canais, ChildOf, CopyTransform, Distance, DistanceMode, Ik, Kind, Limit, LookAt, Space,
};
use ph2d_constraints_ecs::Constraints;
use ph2d_ecs::{GlobalTransform, Name, Transform, assign_missing_stable_ids, stable_id_of};
use ph2d_render::{DISC_TILE_KEY, Sprite, WHITE_TILE_KEY};

/// O roteador desta família — o maior nível que [`encenar`] responde.
pub const ROUTERS: &[ph2d_app_host::SmokeRouter] = &[ph2d_app_host::SmokeRouter {
    env: "PH2D_CONSTRAINT_SMOKE",
    max_level: 4,
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

/// O objecto com que a cena abre ESCOLHIDO (o cartão e o gizmo dele à vista); a shell consome-o
/// uma vez ([`escolhido`]).
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Escolher(pub u64);

/// Povoa `world` com a cena `nivel`. Devolve se havia cena para esse nível.
pub fn encenar(world: &mut World, nivel: u32) -> bool {
    let sujeito = match nivel {
        1 => mao_e_espada(world),
        2 => olho(world),
        3 => ponteiros(world),
        4 => braco(world),
        _ => return false,
    };
    world.insert_resource(Escolher(sujeito.to_bits()));
    true
}

/// O objecto que a cena pediu escolhido — uma vez só.
pub fn escolhido(world: &mut World) -> Option<u64> {
    world.remove_resource::<Escolher>().map(|e| e.0)
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

/// Cena 2 — **Olho.** Abre com a pupila escolhida.
fn olho(world: &mut World) -> Entity {
    let d = 2.0 * RAIO_DO_OLHO;
    let olho = peca(
        world,
        "Olho",
        DISC_TILE_KEY,
        [0.0, 0.0],
        0.0,
        [d, d],
        [0.95, 0.95, 0.92, 1.0],
    );
    let bola = peca(
        world,
        "Bola",
        DISC_TILE_KEY,
        [3.0, 1.2],
        0.0,
        [0.5, 0.5],
        [0.9, 0.15, 0.1, 1.0],
    );
    let pupila = peca(
        world,
        "Pupila",
        DISC_TILE_KEY,
        [0.0, 0.0],
        0.0,
        [0.7, 0.7],
        [0.08, 0.08, 0.1, 1.0],
    );
    let seta = peca(
        world,
        "Seta",
        WHITE_TILE_KEY,
        [-3.0, -1.5],
        0.0,
        [1.4, 0.18],
        [0.2, 0.45, 0.9, 1.0],
    );
    let copia = Kind::CopyTransform(CopyTransform {
        canais: Canais::LOCATION,
        space: Space::World,
        ..CopyTransform::default()
    });
    let dentro = Kind::Distance(Distance {
        distance: FOLGA_DA_PUPILA,
        mode: DistanceMode::Inside,
    });
    pilha(
        world,
        pupila,
        &[(copia, Some(bola), 1.0), (dentro, Some(olho), 1.0)],
    );
    pilha(
        world,
        seta,
        &[(Kind::LookAt(LookAt::default()), Some(bola), 1.0)],
    );
    pupila
}

/// Cena 3 — **Ponteiros.** Abre com o ponteiro do limite escolhido.
fn ponteiros(world: &mut World) -> Entity {
    let lado = [2.0, 0.2];
    let limitado = peca(
        world,
        "Ponteiro com limite",
        WHITE_TILE_KEY,
        [0.0, 1.5],
        170.0,
        lado,
        [0.9, 0.55, 0.1, 1.0],
    );
    let meio = peca(
        world,
        "Ponteiro a meio",
        WHITE_TILE_KEY,
        [0.0, -1.5],
        170.0,
        lado,
        [0.2, 0.7, 0.35, 1.0],
    );
    let guia = peca(
        world,
        "Guia",
        WHITE_TILE_KEY,
        [3.5, -1.5],
        -170.0,
        lado,
        [0.6, 0.6, 0.6, 1.0],
    );
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
    limitado
}

/// Cena 1: as poses da MÃO por quadro — `(quadro, x, y, graus)`. Desce à espada no 24, levanta-a no
/// 36, pousa-a no 48 e vai-se embora.
pub const MAO: [(i64, f32, f32, f32); 5] = [
    (0, -4.0, 1.5, 0.0),
    (QUADRO_PEGA, -1.0, -0.9, 0.0),
    (36, 0.75, 1.4, 25.0),
    (QUADRO_LARGA, 2.5, -0.9, 0.0),
    (72, 4.5, 1.5, 0.0),
];
/// O quadro em que a mão pega a espada e o em que a larga.
pub const QUADRO_PEGA: i64 = 24;
pub const QUADRO_LARGA: i64 = 48;
/// Onde a espada está no chão.
pub const ESPADA_NO_CHAO: [f32; 2] = [-1.0, -1.25];

/// Cena 1 — **Mão e espada.** A espada tem um Child Of na mão a força 0; abre escolhida. A curva da
/// mão é da timeline: a cena conduzida ([`crate::smoke_pega`]) grava-a.
fn mao_e_espada(world: &mut World) -> Entity {
    let _ = peca(
        world,
        "Chão",
        WHITE_TILE_KEY,
        [0.5, -1.4],
        0.0,
        [10.0, 0.12],
        [0.35, 0.3, 0.25, 1.0],
    );
    let (_, x, y, r) = MAO[0];
    let mao = peca(
        world,
        "Mão",
        DISC_TILE_KEY,
        [x, y],
        r,
        [0.5, 0.5],
        [0.95, 0.75, 0.6, 1.0],
    );
    let espada = peca(
        world,
        "Espada",
        WHITE_TILE_KEY,
        ESPADA_NO_CHAO,
        0.0,
        [1.6, 0.16],
        [0.75, 0.8, 0.9, 1.0],
    );
    pilha(
        world,
        espada,
        &[(Kind::ChildOf(ChildOf::default()), Some(mao), 0.0)],
    );
    espada
}

/// Cena 4: o comprimento de cada osso do braço (metros).
pub const OSSO: f32 = 2.0;
/// Cena 4: onde a bola (o alvo da IK) e o ponteiro (o Rodador) nascem.
pub const BOLA: [f32; 2] = [0.8, 1.6];
pub const RODADOR: ([f32; 2], f32) = ([-3.0, -2.0], -60.0);

fn osso(world: &mut World, nome: &str, cabeca: [f32; 2], rot_deg: f32, pai: Option<Entity>) -> Entity {
    let mut t = Transform::IDENTITY;
    t.translation.x = cabeca[0];
    t.translation.y = cabeca[1];
    t.rotation = rot_deg.to_radians();
    let e = world
        .spawn((
            Name::new(nome),
            t,
            GlobalTransform::from_transform(t),
            ph2d_skeleton_ecs::Bone {
                length: f64::from(OSSO),
                strength: 1.0,
                ..Default::default()
            },
        ))
        .id();
    if let Some(p) = pai {
        world.entity_mut(e).insert(ph2d_ecs::ChildOf(p));
    }
    e
}

/// A arte presa a um osso (filha dele, no espaço dele).
fn arte(world: &mut World, osso: Entity, nome: &str, tile: u32, centro: [f32; 2], lado: [f32; 2], cor: [f32; 4]) {
    let e = peca(world, nome, tile, centro, 0.0, lado, cor);
    world.entity_mut(e).insert(ph2d_ecs::ChildOf(osso));
}

/// Cena 4 (plano §6): o braço de 2 ossos com a IK na mão e um Copy Rotation ABAIXO dela — a mão roda
/// com o Rodador (no Blender o Copy Rotation some, `ik_com_copy_rotation_abaixo`). O sujeito é a mão.
fn braco(world: &mut World) -> Entity {
    let ombro = osso(world, "Ombro", [-2.0, -1.0], 20.0, None);
    let antebraco = osso(world, "Antebraço", [OSSO, 0.0], 40.0, Some(ombro));
    let pele = [0.95, 0.75, 0.6, 1.0];
    arte(world, ombro, "Braço", WHITE_TILE_KEY, [OSSO / 2.0, 0.0], [OSSO, 0.35], pele);
    arte(world, antebraco, "Antebraço (arte)", WHITE_TILE_KEY, [OSSO / 2.0, 0.0], [OSSO, 0.3], pele);
    arte(world, antebraco, "Dedo", WHITE_TILE_KEY, [OSSO + 0.3, 0.0], [0.6, 0.12], [0.85, 0.6, 0.45, 1.0]);
    let bola = peca(world, "Bola", DISC_TILE_KEY, BOLA, 0.0, [0.45, 0.45], [0.9, 0.15, 0.1, 1.0]);
    let rodador = peca(
        world,
        "Rodador",
        WHITE_TILE_KEY,
        RODADOR.0,
        RODADOR.1,
        [1.4, 0.18],
        [0.6, 0.6, 0.6, 1.0],
    );
    let copia = Kind::CopyTransform(CopyTransform {
        canais: Canais::ROTATION,
        space: Space::World,
        ..CopyTransform::default()
    });
    pilha(
        world,
        antebraco,
        &[(Kind::Ik(Ik::default()), Some(bola), 1.0), (copia, Some(rodador), 1.0)],
    );
    antebraco
}
