//! Os gates do Draw Mode Tiled (plano GEF 14): pelo nível do jogo e pela porta do Inspector (`apply_edit`).

use super::*;
use crate::image_choice::choose_library_image;
use crate::image_use::{ImageUse, apply_edit, image_use};
use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::{Name, SimWorld};
use ph2d_editor_core::ImageUseEdit;
use ph2d_render::{GROUND_TILE, GROUND_TILE_KEY, PLATFORM_TILE, PLATFORM_TILE_KEY, WHITE_TILE_KEY};
use std::collections::BTreeMap;

const PPM: f32 = 100.0;

fn biblioteca() -> (AssetDb, BTreeMap<u32, AssetId>) {
    let db = AssetDb::new();
    let mut atlas = BTreeMap::new();
    ph2d_app_components::factory_images::register(&db, &mut atlas);
    (db, atlas)
}

fn sem_placa(_: u32, _: u32, _: &[u8]) -> Result<u32, String> {
    Err("uma imagem da biblioteca não sobe cópia".into())
}

/// O nível 2 e a sua Platform Low (Tiled 2,4 × 0,5, ladrilho 1 × 0,5, collider automático).
fn platform_low() -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    ph2d_app_game::smoke::montar(&mut sim, &ph2d_app_components::test_support::registo(), 2);
    let w = sim.world_mut();
    let mut q = w.query::<(Entity, &Name)>();
    let e = q
        .iter(w)
        .find(|(_, n)| n.as_str() == "Platform Low")
        .map(|(e, _)| e)
        .expect("Platform Low");
    (sim, e)
}

fn edita(sim: &mut SimWorld, e: Entity, edit: ImageUseEdit) -> bool {
    let (db, atlas) = biblioteca();
    apply_edit(sim.world_mut(), e, &db, &atlas, PPM, edit)
}

fn caixa(sim: &SimWorld, e: Entity) -> ColliderShape {
    sim.world().get::<Collider>(e).expect("collider").shape
}

/// ⭐ D3: a Largura muda o desenho e leva o collider automático; a escala fica 1.
#[test]
fn largura_leva_o_collider_automatico() {
    let (mut sim, e) = platform_low();
    assert_eq!(
        draw_of(sim.world(), e),
        InspectorDraw::Tiled { size: [2.4, 0.5] }
    );
    assert!(edita(&mut sim, e, ImageUseEdit::Width(4.0)));
    assert_eq!(
        sim.world().get::<Sprite>(e).map(|s| s.size),
        Some([4.0, 0.5])
    );
    assert_eq!(
        caixa(&sim, e),
        ColliderShape::Cuboid {
            half_x: 2.0,
            half_y: 0.25
        }
    );
    // As repetições saem do tamanho: 4 ladrilhos de 1 m.
    assert_eq!(
        ph2d_ecs::resolve_uv_xform(sim.world(), e, [4.0, 0.5]),
        [4.0, 1.0, 0.0, 0.0]
    );
}

/// ⭐ D3 (controlo): um collider que o artista mudou à mão fica onde ele o pôs.
#[test]
fn collider_a_mao_fica() {
    let (mut sim, e) = platform_low();
    let mao = ColliderShape::Cuboid {
        half_x: 0.7,
        half_y: 0.1,
    };
    sim.world_mut()
        .get_mut::<Collider>(e)
        .expect("collider")
        .shape = mao;
    assert!(edita(&mut sim, e, ImageUseEdit::Height(1.0)));
    assert_eq!(
        sim.world().get::<Sprite>(e).map(|s| s.size),
        Some([2.4, 1.0])
    );
    assert_eq!(caixa(&sim, e), mao);
}

/// ⭐ D4: trocar a imagem de um Tiled guarda o tamanho; o ladrilho passa a ser a célula da imagem nova.
#[test]
fn trocar_a_imagem_de_um_tiled_guarda_o_tamanho() {
    let (mut sim, e) = platform_low();
    let (db, atlas) = biblioteca();
    choose_library_image(
        &mut sim,
        &mut sem_placa,
        &db,
        &atlas,
        PPM,
        e,
        atlas[&GROUND_TILE_KEY],
    )
    .expect("troca");
    let w = sim.world();
    assert_eq!(w.get::<Sprite>(e).map(|s| s.size), Some([2.4, 0.5]));
    assert_eq!(
        w.get::<TiledDraw>(e).map(|t| t.tile),
        Some(GROUND_TILE.native_m())
    );
    assert_eq!(image_use(w, e), Some(ImageUse::Repeated));
}

/// ⭐ Plano 15 D3 (a caixa Tiled da 14 é agora o uso «Repeated»): passar a repetida uma sprite parada põe o
/// ladrilho de uma célula nativa e guarda o tamanho; voltar a parada estica. O UV à mão sai.
#[test]
fn ligar_e_desligar_o_tiled() {
    let mut sim = SimWorld::new();
    let (db, atlas) = biblioteca();
    let e = sim
        .world_mut()
        .spawn((
            ph2d_ecs::Transform::IDENTITY,
            Sprite::atlas(WHITE_TILE_KEY, [1.0, 1.0], [1.0; 4]),
        ))
        .id();
    choose_library_image(
        &mut sim,
        &mut sem_placa,
        &db,
        &atlas,
        PPM,
        e,
        atlas[&PLATFORM_TILE_KEY],
    )
    .expect("escolha");
    sim.world_mut().entity_mut(e).insert(UvTransform {
        scale: [3.0, 1.0],
        offset: [0.0, 0.0],
    });
    let antes = sim.world().get::<Sprite>(e).map(|s| s.size);
    assert_eq!(image_use(sim.world(), e), Some(ImageUse::Still));
    assert!(edita(&mut sim, e, repetida()));
    let w = sim.world();
    assert_eq!(w.get::<Sprite>(e).map(|s| s.size), antes);
    assert_eq!(
        w.get::<TiledDraw>(e).map(|t| t.tile),
        Some(PLATFORM_TILE.native_m())
    );
    assert!(w.get::<UvTransform>(e).is_none() && w.get::<TextureRepeat>(e).is_none());
    assert!(!edita(&mut sim, e, repetida()), "já é repetida");
    assert!(edita(
        &mut sim,
        e,
        ImageUseEdit::Use(ph2d_editor_core::InspectorImageUse::Still)
    ));
    assert_eq!(draw_of(sim.world(), e), InspectorDraw::Simple);
    assert_eq!(image_use(sim.world(), e), Some(ImageUse::Still));
    // Uma camada de fundo não oferece Draw Mode.
    sim.world_mut()
        .entity_mut(e)
        .insert(ph2d_ecs::ScrollRepeat { tile: [1.0, 0.0] });
    assert!(!edita(
        &mut sim,
        e,
        ImageUseEdit::Use(ph2d_editor_core::InspectorImageUse::Animated)
    ));
}

fn repetida() -> ImageUseEdit {
    ImageUseEdit::Use(ph2d_editor_core::InspectorImageUse::Repeated)
}
