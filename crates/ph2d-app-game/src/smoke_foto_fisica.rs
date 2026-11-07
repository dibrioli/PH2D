//! **As fotos dos cartões de física** (`PH2D_GAME_SMOKE=4`, `PH2D_GAME_SMOKE_FOTO=fisica_*`, GEF02): um objecto por
//! face do cartão, selecionado e com a secção aberta. Irmão de [`super`] para duas linhas não colidirem na lista.

use ph2d_core::Vec2;
use ph2d_ecs::{
    ChildOf, Name, SimWorld, Transform, assign_missing_root_order, assign_missing_stable_ids,
};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

use super::por_nome;

/// Que face do cartão a foto mostra.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CasoFisica {
    Dynamic,
    Kinematic,
    /// A plataforma do nível, feita pelo «Static Body» do menu Add.
    Static,
    /// A plataforma baixa antes e depois de Width 4: o Half Size do cartão tem de acompanhar.
    Baixa,
    Larga,
    Sensor,
    Bola,
    /// O Player do nível: cápsula, com o cartão do jogador ao lado.
    Player,
    /// Uma forma sem corpo acima: a «Collision Shape» acabada de nascer do menu Add.
    Solta,
    /// Uma forma dentro de um corpo.
    Peca,
    /// Uma imagem sem física: a face que oferece o corpo.
    Vazia,
}

/// O caso do nome pedido em `PH2D_GAME_SMOKE_FOTO`.
#[must_use]
pub fn caso(nome: &str) -> Option<CasoFisica> {
    Some(match nome.strip_prefix("fisica_")? {
        "dynamic" => CasoFisica::Dynamic,
        "kinematic" => CasoFisica::Kinematic,
        "static" => CasoFisica::Static,
        "baixa" => CasoFisica::Baixa,
        "larga" => CasoFisica::Larga,
        "sensor" => CasoFisica::Sensor,
        "bola" => CasoFisica::Bola,
        "player" => CasoFisica::Player,
        "solta" => CasoFisica::Solta,
        "peca" => CasoFisica::Peca,
        "vazia" => CasoFisica::Vazia,
        _ => return None,
    })
}

const CAIXA: ColliderShape = ColliderShape::Cuboid {
    half_x: 0.5,
    half_y: 0.5,
};

/// Monta o caso no mundo e devolve quem fica selecionado.
pub fn prepara(sim: &mut SimWorld, caso: CasoFisica) -> Option<u64> {
    let onde = Transform::from_translation(Vec2::new(super::ONDE_LARGA[0], super::ONDE_LARGA[1]));
    let imagem = Sprite::atlas(WHITE_TILE_KEY, [1.0, 1.0], [1.0; 4]);
    let corpo = |kind| RigidBody { kind };
    let pai = por_nome(sim, "Platform High")?;
    match caso {
        CasoFisica::Static => return Some(pai.to_bits()),
        CasoFisica::Player => return por_nome(sim, "Player").map(|e| e.to_bits()),
        CasoFisica::Baixa | CasoFisica::Larga => {
            return por_nome(sim, "Platform Low").map(|e| e.to_bits());
        }
        _ => {}
    }
    let w = sim.world_mut();
    let alvo = match caso {
        CasoFisica::Static | CasoFisica::Player | CasoFisica::Baixa | CasoFisica::Larga => {
            unreachable!("voltam acima")
        }
        CasoFisica::Dynamic | CasoFisica::Kinematic | CasoFisica::Bola => {
            let kind = if caso == CasoFisica::Kinematic {
                BodyKind::Kinematic
            } else {
                BodyKind::Dynamic
            };
            let shape = if caso == CasoFisica::Bola {
                ColliderShape::Ball { radius: 0.5 }
            } else {
                CAIXA
            };
            let col = Collider {
                shape,
                ..Collider::default()
            };
            let nome = Name::new("Crate");
            w.spawn((onde, nome, imagem, corpo(kind), col)).id()
        }
        CasoFisica::Sensor => {
            let col = Collider {
                shape: CAIXA,
                is_sensor: true,
                ..Collider::default()
            };
            let nome = Name::new("Zone");
            w.spawn((onde, nome, corpo(BodyKind::Static), col)).id()
        }
        CasoFisica::Solta => {
            let col = Collider {
                shape: CAIXA,
                ..Collider::default()
            };
            w.spawn((onde, Name::new("Collision Shape"), col)).id()
        }
        CasoFisica::Peca => {
            let col = Collider {
                shape: CAIXA,
                ..Collider::default()
            };
            let junto = Transform::from_translation(Vec2::new(0.0, 1.0));
            let nome = Name::new("Collision Shape");
            w.spawn((junto, nome, col, ChildOf(pai))).id()
        }
        CasoFisica::Vazia => w.spawn((onde, Name::new("Sprite"), imagem)).id(),
    };
    assign_missing_stable_ids(w);
    assign_missing_root_order(w);
    Some(alvo.to_bits())
}
