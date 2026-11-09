//! ⭐ **Delete sheet no undo** (plano GEF 26 F10): apagar uma folha é UM passo (o diff do mundo) e o `Ctrl+Z` devolve a
//! folha e os quadros. Irmão por assunto do [`super`].

use super::*;
use ph2d_ecs::{ExtraSheet, FrameRef, ShownSheet, SpriteAnimFrames, SpriteSheets};

/// Mata: o Delete sheet não tirar a `ExtraSheet` (o mundo não muda: zero passos) · uma folha/lista fora do registo
/// (o restauro não a devolve) · o `ShownSheet` registado (outra parte do diff mudaria).
#[test]
fn delete_sheet_deixa_quadros_vazios_no_lugar_e_um_ctrl_z_devolve_tudo() {
    let reg = registry();
    let (mut sim, vec) = scene();
    let x = ExtraSheet {
        asset: ph2d_asset::AssetId::from_bytes(b"run.png"),
        size: [2.0, 1.0],
        anchor: [0.0, 0.5],
        hframes: 2,
        vframes: 1,
    };
    let mut frames = SpriteAnimFrames::default();
    frames.set(
        "mix",
        vec![
            FrameRef::Cell { sheet: 0, cell: 0 },
            FrameRef::Cell { sheet: 1, cell: 1 },
        ],
    );
    let mut lib = ph2d_ecs::SpriteAnimations::new();
    lib.insert(ph2d_ecs::AnimationTag::new("mix", 0, 0))
        .unwrap();
    let e = sim
        .world_mut()
        .spawn((
            Name::new("Hero"),
            ph2d_ecs::StableId(7),
            ph2d_render::Sprite::atlas(1, [1.0, 1.0], [1.0; 4]),
            ph2d_ecs::SpriteGrid::new(2, 1),
            SpriteSheets([x].into_iter().collect()),
            frames.clone(),
            lib,
            ShownSheet(1),
        ))
        .id();
    let antes = capture(&mut sim, &vec, &reg);
    // Pela porta do painel (`EditorAction::ImageUseEdit`), o que o shell aplica no quadro.
    let db = ph2d_asset::AssetDb::default();
    let edit = ph2d_editor_core::ImageUseEdit::DeleteSheet;
    let mapa = Default::default();
    assert!(ph2d_app_sprite_anim::image_use::apply_edit(
        sim.world_mut(),
        e,
        &db,
        &mapa,
        32.0,
        edit
    ));
    let w = sim.world();
    assert!(w.get::<SpriteSheets>(e).is_none());
    assert_eq!(
        w.get::<SpriteAnimFrames>(e).unwrap().get("mix").unwrap(),
        &[FrameRef::Cell { sheet: 0, cell: 0 }, FrameRef::Empty]
    );
    let depois = capture(&mut sim, &vec, &reg);
    assert_eq!(
        antes.parts_that_differ(&depois),
        vec!["world"],
        "um passo, só o mundo"
    );
    let _ = antes.restore(&mut sim, &reg);
    let mut q = sim
        .world_mut()
        .query::<(&Name, &SpriteSheets, &SpriteAnimFrames)>();
    let (_, s, f) = q
        .iter(sim.world())
        .find(|(n, ..)| n.as_str() == "Hero")
        .expect("a folha voltou");
    assert_eq!((s.0.as_slice(), f), (&[x][..], &frames));
}

/// ⭐ Auditoria do fecho 08/11 (E): o Equalize sizes numa folha extra grava a folha E o `Transform` (escala, arranjo)
/// no mesmo quadro, sem `ImageEditSnapshot`; o `Ctrl+Z` é o do documento (`undo_owner` dá `Global` com passo), e
/// UM passo devolve as duas coisas. *Mata: o `Transform` ou as `SpriteSheets` fora do registo.*
#[test]
fn equalize_numa_folha_extra_e_um_passo_que_devolve_folha_e_transform() {
    let reg = registry();
    let (mut sim, vec) = scene();
    let x = ExtraSheet {
        asset: ph2d_asset::AssetId::from_bytes(b"run.png"),
        size: [2.0, 1.0],
        anchor: [0.0, 0.5],
        hframes: 2,
        vframes: 1,
    };
    let e = sim
        .world_mut()
        .spawn((
            Name::new("Hero"),
            ph2d_ecs::StableId(7),
            ph2d_ecs::Transform::default(),
            ph2d_render::Sprite::atlas(1, [1.0, 1.0], [1.0; 4]),
            SpriteSheets([x].into_iter().collect()),
            ShownSheet(1),
        ))
        .id();
    let antes = capture(&mut sim, &vec, &reg);
    let novo = ph2d_asset::AssetId::from_bytes(b"run-eq.png");
    assert!(ph2d_anim_sheets::shown_edit::set_image(
        sim.world_mut(),
        e,
        1,
        novo,
        [3.0, 1.0]
    ));
    {
        let mut t = sim.world_mut().get_mut::<ph2d_ecs::Transform>(e).unwrap();
        (t.scale.x, t.scale.y, t.translation.x) = (0.5, 0.5, 4.0);
    }
    let depois = capture(&mut sim, &vec, &reg);
    assert_eq!(
        antes.parts_that_differ(&depois),
        vec!["world"],
        "um passo, só o mundo"
    );
    let _ = antes.restore(&mut sim, &reg);
    let mut q = sim
        .world_mut()
        .query::<(&Name, &SpriteSheets, &ph2d_ecs::Transform)>();
    let (_, s, t) = q
        .iter(sim.world())
        .find(|(n, ..)| n.as_str() == "Hero")
        .expect("o objeto voltou");
    assert_eq!(s.0.as_slice(), &[x][..]);
    assert_eq!((t.scale.x, t.translation.x), (1.0, 0.0));
}
