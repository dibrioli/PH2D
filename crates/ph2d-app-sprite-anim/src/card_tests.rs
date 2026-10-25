//! **Os gates do cartão «Animation»** (plano GameEngineFuncional 07): cada edição nova, do clique à
//! cena, e o que o snapshot devolve.

use crate::inspector::{apply_anim_edit, build_anim_info};
use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue, apply_editor_commands};
use ph2d_ecs::{
    AnimDirection, AnimationTag, Entity, SimWorld, SpriteAnimCursor, SpriteAnimFrames,
    SpriteAnimations, SpriteAnimator, SpriteGrid, Transform,
};
use ph2d_editor_core::{AnimChooser, AnimCycleMode, AnimFieldEdit, InspectorAnimInfo};
use ph2d_physics_ecs::{CharacterAnimations, PlatformPlayer};

fn registry() -> ComponentRegistry {
    let mut r = ComponentRegistry::new();
    ph2d_ecs::scene::register_ecs_components(&mut r);
    ph2d_render::register_render_components(&mut r);
    ph2d_physics_ecs::register_physics_components(&mut r);
    r
}

/// Uma sprite 8 × 4 com `idle` (0..=5) e `run` (8..=15) e um tocador em `idle`.
fn hero(sim: &mut SimWorld) -> Entity {
    let mut lib = SpriteAnimations::new();
    lib.insert(AnimationTag::new("idle", 0, 5)).unwrap();
    lib.insert(AnimationTag::new("run", 8, 15)).unwrap();
    sim.world_mut()
        .spawn((
            Transform::default(),
            ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
            SpriteGrid {
                hframes: 8,
                vframes: 4,
                frame: 0,
            },
            lib,
            SpriteAnimator::new("idle"),
        ))
        .id()
}

fn edit(sim: &mut SimWorld, e: Entity, reg: &ComponentRegistry, ed: AnimFieldEdit) {
    let queue = EditorCommandQueue::new();
    let _ = apply_anim_edit(sim, e.to_bits(), &ed, &queue, reg);
    apply_editor_commands(sim.world_mut(), &queue, reg).expect("o commit aplica");
}

fn info(sim: &SimWorld, e: Entity) -> InspectorAnimInfo {
    build_anim_info(sim.world(), e.to_bits(), 1).expect("é uma sprite animada")
}

#[test]
fn a_lista_clicada_chega_a_cena_e_volta_ao_snapshot() {
    let (mut sim, reg) = (SimWorld::new(), registry());
    let e = hero(&mut sim);
    edit(
        &mut sim,
        e,
        &reg,
        AnimFieldEdit::Frames(1, vec![9, 8, 9, 31]),
    );
    let w = sim.world();
    assert_eq!(
        w.get::<SpriteAnimFrames>(e).and_then(|f| f.get("run")),
        Some(&[9, 8, 9, 31][..])
    );
    assert!(
        w.get::<SpriteAnimCursor>(e).is_some(),
        "a lista traz o seu passo"
    );
    assert_eq!(info(&sim, e).rows[1].frames, [9, 8, 9, 31]);
    edit(&mut sim, e, &reg, AnimFieldEdit::Frames(1, vec![]));
    let w = sim.world();
    assert!(w.get::<SpriteAnimFrames>(e).is_none() && w.get::<SpriteAnimCursor>(e).is_none());
    assert!(
        info(&sim, e).rows[1].frames.is_empty(),
        "vazia = o intervalo"
    );
}

#[test]
fn o_modo_do_ciclo_escreve_repeat_e_direcao_juntos() {
    let (mut sim, reg) = (SimWorld::new(), registry());
    let e = hero(&mut sim);
    for (mode, repeat, dir) in [
        (AnimCycleMode::Once, Some(1), AnimDirection::Forward),
        (AnimCycleMode::PingPong, None, AnimDirection::PingPong),
        (AnimCycleMode::Repeat, None, AnimDirection::Forward),
    ] {
        edit(&mut sim, e, &reg, AnimFieldEdit::CycleMode(0, mode.tag()));
        let t = sim.world().get::<SpriteAnimations>(e).unwrap().0[0].clone();
        assert_eq!((t.repeat, t.direction), (repeat, dir), "{mode:?}");
        let row = &info(&sim, e).rows[0];
        assert_eq!(AnimCycleMode::of_row(row.direction_tag, row.repeat), mode);
    }
}

#[test]
fn quem_escolhe_eu_script_e_o_personagem() {
    let (mut sim, reg) = (SimWorld::new(), registry());
    let e = hero(&mut sim);
    edit(
        &mut sim,
        e,
        &reg,
        AnimFieldEdit::Chooser(AnimChooser::Me.tag()),
    );
    assert!(sim.world().get::<SpriteAnimator>(e).unwrap().autoplay);
    assert_eq!(info(&sim, e).card.chooser, AnimChooser::Me);
    edit(
        &mut sim,
        e,
        &reg,
        AnimFieldEdit::Chooser(AnimChooser::Script.tag()),
    );
    assert!(!sim.world().get::<SpriteAnimator>(e).unwrap().autoplay);
    assert_eq!(info(&sim, e).card.chooser, AnimChooser::Script);

    // Sem Platform Player não há estados: «O personagem» não pega.
    edit(
        &mut sim,
        e,
        &reg,
        AnimFieldEdit::Chooser(AnimChooser::Character.tag()),
    );
    assert!(sim.world().get::<CharacterAnimations>(e).is_none());
    assert!(!info(&sim, e).card.has_character_body);

    sim.world_mut()
        .entity_mut(e)
        .insert(PlatformPlayer::default());
    edit(
        &mut sim,
        e,
        &reg,
        AnimFieldEdit::Chooser(AnimChooser::Character.tag()),
    );
    let card = info(&sim, e).card;
    assert_eq!(card.chooser, AnimChooser::Character);
    assert!(card.has_character_body && card.character.is_some());
    edit(
        &mut sim,
        e,
        &reg,
        AnimFieldEdit::Chooser(AnimChooser::Me.tag()),
    );
    assert!(
        sim.world().get::<CharacterAnimations>(e).is_none(),
        "sair desliga o automático"
    );
}

#[test]
fn os_estados_do_personagem_escolhem_se_e_acompanham_o_renomear() {
    let (mut sim, reg) = (SimWorld::new(), registry());
    let e = hero(&mut sim);
    sim.world_mut()
        .entity_mut(e)
        .insert((PlatformPlayer::default(), CharacterAnimations::default()));
    edit(
        &mut sim,
        e,
        &reg,
        AnimFieldEdit::CharacterState(1, "run".into()),
    );
    edit(&mut sim, e, &reg, AnimFieldEdit::CharacterRunFrom(1.25));
    edit(&mut sim, e, &reg, AnimFieldEdit::CharacterFlip(false));
    edit(&mut sim, e, &reg, AnimFieldEdit::Frames(1, vec![8, 9]));
    edit(&mut sim, e, &reg, AnimFieldEdit::SetCurrent("run".into()));
    edit(&mut sim, e, &reg, AnimFieldEdit::Rename(1, "corre".into()));
    let w = sim.world();
    let a = w.get::<CharacterAnimations>(e).unwrap();
    assert_eq!((a.run.as_str(), a.run_from, a.flip), ("corre", 1.25, false));
    assert_eq!(w.get::<SpriteAnimator>(e).unwrap().current, "corre");
    assert_eq!(
        w.get::<SpriteAnimFrames>(e).and_then(|f| f.get("corre")),
        Some(&[8, 9][..])
    );
    assert_eq!(info(&sim, e).card.character.unwrap().names[1], "corre");
}

#[test]
fn apagar_uma_animacao_leva_a_sua_lista() {
    let (mut sim, reg) = (SimWorld::new(), registry());
    let e = hero(&mut sim);
    edit(&mut sim, e, &reg, AnimFieldEdit::Frames(1, vec![8]));
    edit(&mut sim, e, &reg, AnimFieldEdit::Remove(1));
    assert!(sim.world().get::<SpriteAnimFrames>(e).is_none());
}

#[test]
fn a_grelha_confirmada_e_o_passo_escolhido() {
    let (mut sim, reg) = (SimWorld::new(), registry());
    let e = hero(&mut sim);
    edit(&mut sim, e, &reg, AnimFieldEdit::Grid(4, 2));
    let card = info(&sim, e).card;
    assert_eq!((card.hframes, card.vframes), (4, 2));
    edit(&mut sim, e, &reg, AnimFieldEdit::Grid(0, 9999));
    let g = *sim.world().get::<SpriteGrid>(e).unwrap();
    assert_eq!((g.hframes, g.vframes), (1, crate::card::GRID_SIDE_MAX));

    edit(&mut sim, e, &reg, AnimFieldEdit::Grid(8, 4));
    edit(&mut sim, e, &reg, AnimFieldEdit::Frames(0, vec![3, 1, 3]));
    edit(&mut sim, e, &reg, AnimFieldEdit::SetStep(2));
    let w = sim.world();
    assert_eq!(w.get::<SpriteGrid>(e).unwrap().frame, 3);
    assert_eq!(
        w.get::<SpriteAnimCursor>(e).unwrap().step,
        2,
        "o 2.º 3, não o 1.º"
    );
    assert_eq!(info(&sim, e).card.step, 2);
}

#[test]
fn o_tique_toca_a_lista_na_ordem_clicada() {
    let (mut sim, reg) = (SimWorld::new(), registry());
    let e = hero(&mut sim);
    edit(&mut sim, e, &reg, AnimFieldEdit::Frames(0, vec![5, 2, 7]));
    edit(&mut sim, e, &reg, AnimFieldEdit::SetCurrent("idle".into()));
    edit(&mut sim, e, &reg, AnimFieldEdit::Playing(true));
    let mut drive = ph2d_preview_drive::PreviewDrive::default();
    let mut seen = vec![sim.world().get::<SpriteGrid>(e).unwrap().frame];
    for _ in 0..3 {
        // `frame_ms` 100 = seis tiques de 1/60 s.
        crate::tick::tick_sprite_animations(&mut sim, 6, 1.0 / 60.0, &[], &mut drive);
        seen.push(sim.world().get::<SpriteGrid>(e).unwrap().frame);
    }
    assert_eq!(seen, [5, 2, 7, 5]);
}
