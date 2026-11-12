//! **A cena do Top-Down Player** (`PH2D_GAME_SMOKE=24`, ficha GEF 05): um jogo visto de cima — o chão, três
//! paredes para ver o deslize e o herói de fábrica com o mover de vista de cima e Facing = Move. Montada SÓ com
//! peças: o «Static Body» e a «Sprite» do menu Add e o `+` do Inspector (as portas vêm da composição).

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, ZIndexOverride};
use ph2d_physics_ecs::{Collider, ColliderShape, TopDownPlayer};
use ph2d_render::{Sprite, WHITE_TILE_KEY};
use ph2d_topdown::rotation::{self, RotationMode};

/// O número da cena no `PH2D_GAME_SMOKE`.
pub const CENA: u32 = 24;

/// O nome do herói (nome de objecto da cena, não texto da interface).
pub const HEROI: &str = "Hero";

/// Onde o herói nasce (m): no meio da arena — o «Frame Selected» do smoke enquadra-a inteira à volta dele.
pub const INICIO: [f32; 2] = [-3.0, -1.0];

/// As paredes: nome, centro, meia-extensão (m). A diagonal ↗ do início bate de raspão na de cima e desliza
/// até ao canto da da direita (que fica à esquerda do Inspector na foto de 1930 px).
pub const PAREDES: [(&str, [f32; 2], [f32; 2]); 3] = [
    ("Wall Top", [-3.5, 2.2], [4.5, 0.3]),
    ("Wall Right", [0.7, -1.0], [0.3, 3.5]),
    ("Pillar", [-5.5, -3.0], [0.5, 0.5]),
];

/// O chão (m): o centro e o tamanho da arena.
pub const CHAO: ([f32; 2], [f32; 2]) = ([-3.5, -1.0], [9.0, 7.0]);
const CHAO_RGBA: [f32; 4] = [0.22, 0.30, 0.24, 1.0];
const PAREDE_RGBA: [f32; 4] = [0.45, 0.42, 0.38, 1.0];

/// As peças que a cena pede à composição.
#[derive(Clone, Copy)]
pub struct Pecas<'a> {
    /// O «Static Body» do menu Add (nasce com a forma em filha).
    pub estatico: &'a dyn Fn(&mut SimWorld) -> Entity,
    /// A «Sprite» do menu Add.
    pub imagem: &'a dyn Fn(&mut SimWorld) -> Entity,
    /// O `+` do Inspector: anexa o componente pelo nome canónico, com a cascata e as sementes.
    pub anexa: &'a dyn Fn(&mut SimWorld, u64, &str) -> Result<(), String>,
}

fn arruma(sim: &mut SimWorld, e: Entity, nome: &str, at: [f32; 2]) {
    let mut em = sim.world_mut().entity_mut(e);
    em.insert(Name::new(nome));
    if let Some(mut t) = em.get_mut::<Transform>() {
        t.translation = Vec2::new(at[0], at[1]);
    }
}

/// Monta a cena e devolve o herói (o objecto que o Inspector abre).
pub fn montar(sim: &mut SimWorld, p: Pecas<'_>) -> Entity {
    let chao = (p.imagem)(sim);
    arruma(sim, chao, "Floor", CHAO.0);
    sim.world_mut().entity_mut(chao).insert((
        Sprite::atlas(WHITE_TILE_KEY, CHAO.1, CHAO_RGBA),
        ZIndexOverride(-1),
    ));
    for (nome, c, h) in PAREDES {
        let e = (p.estatico)(sim);
        arruma(sim, e, nome, c);
        let w = sim.world_mut();
        w.entity_mut(e).insert(Sprite::atlas(
            WHITE_TILE_KEY,
            [2.0 * h[0], 2.0 * h[1]],
            PAREDE_RGBA,
        ));
        let f = ph2d_physics_ecs::forma_para_editar(w, e).unwrap_or(e);
        if let Some(mut col) = w.get_mut::<Collider>(f) {
            col.shape = ColliderShape::Cuboid {
                half_x: h[0],
                half_y: h[1],
            };
        }
    }
    let heroi = (p.imagem)(sim);
    arruma(sim, heroi, HEROI, INICIO);
    if let Err(e) = ph2d_render::veste_o_heroi(sim.world_mut(), heroi) {
        eprintln!("[game-smoke] =24 a folha do herói: {e}");
    }
    if let Err(e) = (p.anexa)(sim, heroi.to_bits(), "ph2d::physics::TopDownPlayer") {
        eprintln!("[game-smoke] =24 o Top-Down Player: {e}");
    }
    if let Some(mut td) = sim.world_mut().get_mut::<TopDownPlayer>(heroi) {
        td.rotation_mode = rotation::to_wire(RotationMode::ToMovement);
    }
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_missing_root_order(sim.world_mut());
    println!(
        "[game-smoke] =24 um jogo visto de cima: o herói (Hero) tem o Top-Down Player com Facing = Move e está \
         selecionado. Play (F5): as setas andam em 8 direções e a imagem vira para onde anda; encoste na diagonal \
         numa parede e ele desliza ao longo dela."
    );
    heroi
}
