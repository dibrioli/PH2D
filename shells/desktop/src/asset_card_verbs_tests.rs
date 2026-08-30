//! Os gates dos verbos do menu de um cartão ([`super`]).

use super::users_of;
use ph2d_ecs::{ChildOf, MasterRoot, Name, SimWorld, SpritePixels, StableId, Transform};
use ph2d_editor::interaction::drag_payload::DragPayload;

/// ⭐⭐⭐ **Uma PEÇA DE RECEITA não é um utilizador** (auditoria de 2026-08-30, achado nº 1).
///
/// ⛔ A varredura crua contava-a — as peças de uma receita carregam `SpritePixels` como qualquer
/// outra — e elas **não estão na cena**: o extract salta-as e a Hierarquia não lhes dá linha.
/// Consequência medida: com uma imagem usada só por uma receita, o *Select users* dizia
/// *«Selected 1 object(s)»* e **nada acendia**. É a espécie *«o consumidor que PROJECTA o valor
/// fora»* do §5.0 — o fio está completo e quem o lê descarta.
///
/// ⚠️ **A fixtura tem o fenómeno de propósito:** a MESMA textura usada por uma peça de receita e
/// por um objecto da cena. Sem o objecto da cena, um filtro que devolvesse sempre vazio passaria.
///
/// **Mutação que deve sangrar:** apagar o `.filter(|e| !is_unedited_recipe(..))`.
#[test]
fn a_recipe_piece_is_not_a_user_because_the_artist_cannot_reach_it() {
    let mut sim = SimWorld::new();
    let db = ph2d_asset::AssetDb::new();
    let tex = db.insert_image_rgba8(2, 2, vec![5u8; 16]);

    // A receita, com uma peça que usa a textura.
    let master = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Recipe"), MasterRoot))
        .id();
    let hidden_piece = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("RecipePiece"),
            SpritePixels(tex),
            ChildOf(master),
        ))
        .id();
    // E um objecto da CENA que usa a mesma textura.
    let on_canvas = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("OnCanvas"),
            SpritePixels(tex),
        ))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    // ⚠️ A marca é DERIVADA — sem este passe a peça da receita não é peça de nada, e o gate mediria
    // um mundo que o produto nunca tem.
    ph2d_ecs::assign_master_pieces(sim.world_mut());

    let users = users_of(
        &mut sim,
        DragPayload::Image {
            asset: *tex.as_bytes(),
        },
    );
    assert!(
        users.contains(&on_canvas.to_bits()),
        "o objecto da cena TEM de contar"
    );
    assert!(
        !users.contains(&hidden_piece.to_bits()),
        "a peça da receita não pode contar — o artista não a vê nem a alcança"
    );
    assert_eq!(users.len(), 1, "e mais ninguém: {users:?}");
}

/// ⚠️ **Uma entidade nascida no mesmo quadro ainda não tem `StableId`** — o
/// `assign_missing_stable_ids` corre uma vez por quadro. Exigi-lo na consulta fá-la-ia desaparecer
/// da resposta **em silêncio**, que é o defeito que o `map_or(u64::MAX, …)` evita.
///
/// **Mutação que deve sangrar:** voltar a `query::<(Entity, &SpritePixels, &StableId)>()`.
#[test]
fn an_object_born_this_frame_still_counts_as_a_user() {
    let mut sim = SimWorld::new();
    let db = ph2d_asset::AssetDb::new();
    let tex = db.insert_image_rgba8(2, 2, vec![5u8; 16]);
    let with_id = sim
        .world_mut()
        .spawn((Transform::IDENTITY, SpritePixels(tex)))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    assert!(sim.world().get::<StableId>(with_id).is_some());
    // Este nasce DEPOIS do passe do quadro — é o caso que morde.
    let newborn = sim
        .world_mut()
        .spawn((Transform::IDENTITY, SpritePixels(tex)))
        .id();
    assert!(sim.world().get::<StableId>(newborn).is_none());

    let users = users_of(
        &mut sim,
        DragPayload::Image {
            asset: *tex.as_bytes(),
        },
    );
    assert!(
        users.contains(&newborn.to_bits()),
        "o recém-nascido tem de contar: {users:?}"
    );
    assert_eq!(users.len(), 2);
}
