//! ⭐ **As entradas do jogo de plataforma no menu Add** (`05_N1_plano.md` §1, plano 24): rótulo, grupo,
//! posição, dica e o ensino da busca que não acha. Só DADOS; quem as cria é `ph2d_app_game::object_add`.

use ph2d_editor_core::object_add::{AddEntry, AddGroup, AddTeach};

/// Uma imagem.
pub const SPRITE: AddEntry = AddEntry::new("object_add.game.sprite", AddGroup::Pictures)
    .at(1)
    .with_tip("object_add.game.sprite.tip", super::SPRITE);
/// Uma imagem que se anima por intervalos da folha.
pub const ANIMATED_SPRITE: AddEntry =
    AddEntry::new("object_add.game.animated_sprite", AddGroup::Pictures)
        .at(2)
        .with_tip(
            "object_add.game.animated_sprite.tip",
            super::ANIMATED_SPRITE,
        );
/// Um bloco fixo em que as coisas batem (chão, plataforma, parede).
pub const STATIC_BODY: AddEntry = AddEntry::new("object_add.game.static_body", AddGroup::Game)
    .at(1)
    .with_tip("object_add.game.static_body.tip", super::STATIC_BODY);
/// O personagem que o jogador controla.
pub const CHARACTER: AddEntry = AddEntry::new("object_add.game.character", AddGroup::Game)
    .at(0)
    .with_tip("object_add.game.character.tip", super::CHARACTER);
/// Uma forma de colisão invisível — dentro de um corpo, é mais uma forma dele.
pub const COLLISION_SHAPE: AddEntry =
    AddEntry::new("object_add.game.collision_shape", AddGroup::Game)
        .at(2)
        .with_tip(
            "object_add.game.collision_shape.tip",
            super::COLLISION_SHAPE,
        );
/// Uma camada de fundo que fica para trás quando a câmara anda (parallax) — plano 11.
pub const BACKGROUND_LAYER: AddEntry =
    AddEntry::new("object_add.game.background_layer", AddGroup::Pictures)
        .at(4)
        .with_tip(
            "object_add.game.background_layer.tip",
            super::BACKGROUND_LAYER,
        );
/// O que esta família põe no menu, na ordem do modal.
pub const ENTRIES: &[AddEntry] = &[
    SPRITE,
    ANIMATED_SPRITE,
    STATIC_BODY,
    CHARACTER,
    COLLISION_SHAPE,
    BACKGROUND_LAYER,
];

/// ⭐ **O que a busca que não acha ensina** (plano 24 §2.5): a peça não existe como objecto, e a frase diz
/// de que peças se monta. Só cita componentes do catálogo (gate `cada_ensino_so_cita_o_que_existe`).
pub const TEACH: &[AddTeach] = &[
    AddTeach::new(
        "object_add.teach.coin",
        "object_add.teach.coin.words",
        SPRITE,
    ),
    AddTeach::new(
        "object_add.teach.enemy",
        "object_add.teach.enemy.words",
        ANIMATED_SPRITE,
    ),
    AddTeach::new(
        "object_add.teach.door",
        "object_add.teach.door.words",
        SPRITE,
    ),
    AddTeach::new(
        "object_add.teach.trap",
        "object_add.teach.trap.words",
        STATIC_BODY,
    ),
    AddTeach::new(
        "object_add.teach.checkpoint",
        "object_add.teach.checkpoint.words",
        SPRITE,
    ),
    AddTeach::new(
        "object_add.teach.bullet",
        "object_add.teach.bullet.words",
        SPRITE,
    ),
    AddTeach::new(
        "object_add.teach.moving_platform",
        "object_add.teach.moving_platform.words",
        STATIC_BODY,
    ),
    AddTeach::new(
        "object_add.teach.health",
        "object_add.teach.health.words",
        CHARACTER,
    ),
];
