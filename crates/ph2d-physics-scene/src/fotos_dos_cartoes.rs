//! **As fotos dos cartões de física** (`PH2D_GAME_SMOKE=4`, `PH2D_GAME_SMOKE_FOTO=fisica_*`, ficha f1): um objecto por
//! face do cartão, sobre o nível do jogo. Folha: `ph2d-app-game` (pela catraca da crate) só as pede.

use ph2d_core::Vec2;
use ph2d_ecs::{
    ChildOf, Entity, Name, SimWorld, Transform, assign_missing_root_order,
    assign_missing_stable_ids,
};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, LayerNames, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

fn por_nome(sim: &mut SimWorld, nome: &str) -> Option<Entity> {
    let w = sim.world_mut();
    let mut q = w.query::<(Entity, &Name)>();
    q.iter(w).find(|(_, n)| n.0 == nome).map(|(e, _)| e)
}

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
    /// A bola com Scale 2 × 1: o aviso e as duas saídas.
    Esticada,
    /// O Player do nível: cápsula, com o cartão do jogador ao lado.
    Player,
    /// O Player com os cartões de cima dobrados, para Body e Shape caberem na foto.
    PlayerCartoes,
    /// Uma forma sem corpo acima: a «Collision Shape» acabada de nascer do menu Add.
    Solta,
    /// Uma forma dentro de um corpo.
    Peca,
    /// Uma imagem sem física: a face que oferece o corpo.
    Vazia,
    /// A caixa com camadas com nome, a matriz a separar duas e a lista aberta (onda 2).
    Camadas,
    /// O mesmo, com a camada sem par: «Hits: nothing».
    CamadaSozinha,
    /// A camada Player toca Ground e Enemy: «Hits» com os nomes e o Rename aberto.
    CamadaComPares,
    /// O mesmo, com o painel Physics à vista: a matriz e os campos dos nomes.
    CamadasNoPainel,
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
        "esticada" => CasoFisica::Esticada,
        "player" => CasoFisica::Player,
        "player_cartoes" => CasoFisica::PlayerCartoes,
        "solta" => CasoFisica::Solta,
        "peca" => CasoFisica::Peca,
        "vazia" => CasoFisica::Vazia,
        "camadas" => CasoFisica::Camadas,
        "camada_sozinha" => CasoFisica::CamadaSozinha,
        "camada_com_pares" => CasoFisica::CamadaComPares,
        "camadas_no_painel" => CasoFisica::CamadasNoPainel,
        _ => return None,
    })
}

/// Os nomes da foto das camadas; as outras ficam com o número.
const NOMES_DA_FOTO: [&str; 3] = ["Ground", "Player", "Enemy"];

const CAIXA: ColliderShape = ColliderShape::Cuboid {
    half_x: 0.5,
    half_y: 0.5,
};

/// Monta o caso no mundo e devolve quem fica selecionado.
pub fn prepara(sim: &mut SimWorld, caso: CasoFisica, onde: [f32; 2]) -> Option<u64> {
    let onde = Transform::from_translation(Vec2::new(onde[0], onde[1]));
    let imagem = Sprite::atlas(WHITE_TILE_KEY, [1.0, 1.0], [1.0; 4]);
    let corpo = |kind| RigidBody { kind };
    let pai = por_nome(sim, "Platform High")?;
    match caso {
        CasoFisica::Static => return Some(pai.to_bits()),
        CasoFisica::Player | CasoFisica::PlayerCartoes => {
            return por_nome(sim, "Player").map(|e| e.to_bits());
        }
        CasoFisica::Baixa | CasoFisica::Larga => {
            return por_nome(sim, "Platform Low").map(|e| e.to_bits());
        }
        _ => {}
    }
    let w = sim.world_mut();
    let alvo = match caso {
        CasoFisica::Static
        | CasoFisica::Player
        | CasoFisica::PlayerCartoes
        | CasoFisica::Baixa
        | CasoFisica::Larga => {
            unreachable!("voltam acima")
        }
        CasoFisica::Camadas
        | CasoFisica::CamadaSozinha
        | CasoFisica::CamadaComPares
        | CasoFisica::CamadasNoPainel => {
            for (i, n) in NOMES_DA_FOTO.iter().enumerate() {
                LayerNames::rename(w, i as u8, n);
            }
            let col = Collider {
                shape: CAIXA,
                layer: 1,
                ..Collider::default()
            };
            w.spawn((
                onde,
                Name::new("Crate"),
                imagem,
                corpo(BodyKind::Dynamic),
                col,
            ))
            .id()
        }
        CasoFisica::Dynamic | CasoFisica::Kinematic | CasoFisica::Bola | CasoFisica::Esticada => {
            let kind = if caso == CasoFisica::Kinematic {
                BodyKind::Kinematic
            } else {
                BodyKind::Dynamic
            };
            let shape = if matches!(caso, CasoFisica::Bola | CasoFisica::Esticada) {
                ColliderShape::Ball { radius: 0.5 }
            } else {
                CAIXA
            };
            let col = Collider {
                shape,
                ..Collider::default()
            };
            let nome = Name::new("Crate");
            let mut pose = onde;
            if caso == CasoFisica::Esticada {
                pose.scale = Vec2::new(2.0, 1.0);
            }
            w.spawn((pose, nome, imagem, corpo(kind), col)).id()
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

/// O que o Player tem agora, numa linha (o tamanho, o ladrilho, o corte e a memória por uso).
#[must_use]
pub fn estado_do_player(sim: &mut SimWorld) -> String {
    let Some(e) = por_nome(sim, "Player") else {
        return "sem Player".into();
    };
    let w = sim.world();
    format!(
        "size={:?} tiled={:?} grid={:?} area={:?} anims={} memoria={:?}",
        w.get::<ph2d_render::Sprite>(e).map(|s| s.size),
        w.get::<ph2d_ecs::TiledDraw>(e).map(|t| t.tile),
        w.get::<ph2d_ecs::SpriteGrid>(e)
            .map(|g| (g.hframes, g.vframes)),
        w.get::<ph2d_ecs::SpriteRegion>(e).map(|r| r.rect),
        w.get::<ph2d_ecs::SpriteAnimations>(e).is_some(),
        w.get::<ph2d_ecs::ImageUseMemory>(e)
            .map(|m| [&m.still, &m.repeated, &m.animated].map(|s| s
                .as_ref()
                .and_then(|s| s.look.map(|l| (l.size, l.tiled.map(|t| t.tile)))))),
    )
}
