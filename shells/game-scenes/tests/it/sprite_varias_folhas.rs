//! **As várias folhas do Animated Sprite** (plano GEF 26 §6) sobre o tique REAL e a lei do ecrã ([`sheet_view`]):
//! uma animação que mistura `idle` (4 × 1) e `run` (6 × 2) toca as duas, e cada quadro pinta a imagem da sua folha.

use ph2d_app_sprite_anim::tick::tick_sprite_animations;
use ph2d_asset::AssetId;
use ph2d_ecs::{
    AnimationTag, Entity, ExtraSheet, FrameRef, SimWorld, SpriteAnimCursor, SpriteAnimFrames,
    SpriteAnimations, SpriteAnimator, SpriteGrid, SpriteSheets, Transform,
};
use ph2d_preview_drive::PreviewDrive;
use ph2d_render::{SheetView, Sprite, SpriteSource, resolve_sheet_textures, sheet_view};
use std::collections::BTreeMap;

const DT: f64 = 1.0 / 60.0;
const RUN_TEXTURE: u32 = 77;

fn run_sheet() -> ExtraSheet {
    ExtraSheet {
        asset: AssetId::from_bytes(b"run.png"),
        size: [1.5, 1.5],
        anchor: [0.0, -0.25],
        hframes: 6,
        vframes: 2,
    }
}

/// `mix` = idle 0, idle 1, run 5, run 6; 100 ms por quadro (seis tiques de 60 Hz).
fn scene() -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    let mut lib = SpriteAnimations::new();
    lib.insert(AnimationTag {
        frame_ms: 100,
        ..AnimationTag::new("mix", 0, 0)
    })
    .unwrap();
    let mut frames = SpriteAnimFrames::default();
    frames.set(
        "mix",
        vec![
            FrameRef::cell(0),
            FrameRef::cell(1),
            FrameRef::Cell { sheet: 1, cell: 5 },
            FrameRef::Cell { sheet: 1, cell: 6 },
        ],
    );
    let mut player = SpriteAnimator::new("mix");
    player.playing = true;
    let e = sim
        .world_mut()
        .spawn((
            Transform::default(),
            Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
            SpriteGrid::new(4, 1),
            SpriteSheets([run_sheet()].into_iter().collect()),
            player,
            lib,
            frames,
            SpriteAnimCursor::default(),
        ))
        .id();
    resolve_sheet_textures(sim.world_mut(), &BTreeMap::new(), &mut |_| {
        Some(RUN_TEXTURE)
    });
    (sim, e)
}

fn tick(sim: &mut SimWorld, n: u32) {
    tick_sprite_animations(sim, n, DT, &[], &mut PreviewDrive::default());
}

fn shown(sim: &SimWorld, e: Entity) -> FrameRef {
    sim.world().get::<SpriteGrid>(e).unwrap().shown()
}

/// O tique escreve a folha junto com a célula; o ecrã troca de imagem, tamanho, Pivot e grelha no quadro da `run`.
///
/// *Mutação: o tique escrever só `grid.frame` ⇒ o 3.º quadro fica na folha 0, célula 5, e reprova.*
#[test]
fn um_quadro_de_outra_folha_pinta_a_imagem_dela() {
    let (mut sim, e) = scene();
    let sprite = *sim.world().get::<Sprite>(e).unwrap();
    let mut vistos = vec![shown(&sim, e)];
    for _ in 0..4 {
        tick(&mut sim, 6);
        vistos.push(shown(&sim, e));
    }
    assert_eq!(
        vistos,
        [
            FrameRef::cell(0),
            FrameRef::cell(1),
            FrameRef::Cell { sheet: 1, cell: 5 },
            FrameRef::Cell { sheet: 1, cell: 6 },
            FrameRef::cell(0),
        ],
        "a ordem da tira, folha incluída"
    );
    tick(&mut sim, 6);
    tick(&mut sim, 6);
    assert_eq!(shown(&sim, e), FrameRef::Cell { sheet: 1, cell: 5 });
    let SheetView::Other(s, g) = sheet_view(sim.world(), e, &sprite) else {
        panic!("o quadro da folha 1 tem de pôr a imagem dela");
    };
    assert_eq!(
        s.source,
        SpriteSource::Individual {
            texture_id: RUN_TEXTURE
        }
    );
    assert_eq!((s.size, s.anchor), (run_sheet().size, run_sheet().anchor));
    assert_eq!((g.hframes, g.vframes, g.frame), (6, 2, 5));
    tick(&mut sim, 12);
    assert_eq!(
        sheet_view(sim.world(), e, &sprite),
        SheetView::Own,
        "de volta à idle"
    );
}

/// Um quadro vazio (folha apagada) a tocar não pinta, e a animação passa por ele no seu tempo.
#[test]
fn o_vazio_a_tocar_nao_pinta_e_dura_o_seu_tempo() {
    let (mut sim, e) = scene();
    let sprite = *sim.world().get::<Sprite>(e).unwrap();
    sim.world_mut().get_mut::<SpriteAnimFrames>(e).unwrap().set(
        "mix",
        vec![FrameRef::cell(0), FrameRef::Empty, FrameRef::cell(2)],
    );
    tick(&mut sim, 6);
    assert_eq!(shown(&sim, e), FrameRef::Empty);
    assert_eq!(sheet_view(sim.world(), e, &sprite), SheetView::Nothing);
    tick(&mut sim, 5);
    assert_eq!(shown(&sim, e), FrameRef::Empty, "ainda no tempo dele");
    tick(&mut sim, 1);
    assert_eq!(shown(&sim, e), FrameRef::cell(2));
}
