//! **O cartão Animation sobre o Pixel Sprite** (P14a-4, `docs/PixelArt/P14_plano.md` §6): os clips são
//! as tags do documento (derivados); o cartão só escolhe o que toca, pelo mesmo handler do Animated Sprite.

use super::*;
use ph2d_ecs::scene::{EditorCommandQueue, apply_editor_commands, register_ecs_components};
use ph2d_ecs::{Name, PixelSprite, Transform};
use ph2d_editor_core::AnimChooser;
use ph2d_pixel_cena::fixture::heroi_p14;
use ph2d_pixel_cena::{animador_de_nascenca, deriva, quadro_mostrado};

fn registry() -> ComponentRegistry {
    let mut r = ComponentRegistry::new();
    register_ecs_components(&mut r);
    ph2d_pixel_cena::registar(&mut r);
    r
}

/// O Pixel Sprite como o `object_add` o deixa, com os derivados escritos pela deriva.
fn pixel(sim: &mut SimWorld, s: &ph2d_pixel_core::Sprite) -> Entity {
    let e = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Heroi"),
            PixelSprite(1),
            animador_de_nascenca(),
        ))
        .id();
    deriva(sim.world_mut(), |_| Some((s, 1)));
    e
}

fn edit(sim: &mut SimWorld, e: Entity, reg: &ComponentRegistry, ed: AnimFieldEdit) {
    let queue = EditorCommandQueue::new();
    let _ = apply_anim_edit(sim, e.to_bits(), &ed, &queue, reg);
    apply_editor_commands(sim.world_mut(), &queue, reg).expect("o commit aplica");
}

#[test]
fn o_pixel_sprite_tem_o_cartao_animation_com_clips_do_documento() {
    let s = heroi_p14();
    let mut sim = SimWorld::new();
    let e = pixel(&mut sim, &s);
    let info = build_anim_info(sim.world(), e.to_bits(), 1).expect("o Pixel Sprite tem §11");
    assert!(info.clips_do_documento, "os clips são do documento");
    assert!(info.player_present);
    let nomes: Vec<&str> = info.rows.iter().map(|r| r.name.as_str()).collect();
    assert_eq!(nomes, ["idle", "run", "hit"], "a lista são as tags");
    assert_eq!(
        info.cells, 6,
        "o pool é a grelha derivada, um quadro por célula"
    );
    assert!(info.autoplay, "o animador nasce em Autoplay");

    // Controlo: quem não tem nada continua sem cartão, com ou sem marca de Pixel.
    let nada = sim.world_mut().spawn(Transform::IDENTITY).id();
    assert!(build_anim_info(sim.world(), nada.to_bits(), 1).is_none());
    let vazio = sim
        .world_mut()
        .spawn((Transform::IDENTITY, PixelSprite(9)))
        .id();
    assert!(build_anim_info(sim.world(), vazio.to_bits(), 1).is_none());
}

/// O clique (Who Picks ▸ Script/Me e o menu Starts with) grava no `SpriteAnimator` do Pixel Sprite.
#[test]
fn o_clique_do_cartao_grava_no_animador_do_pixel_sprite() {
    let s = heroi_p14();
    let mut sim = SimWorld::new();
    let reg = registry();
    let e = pixel(&mut sim, &s);
    edit(
        &mut sim,
        e,
        &reg,
        AnimFieldEdit::Chooser(AnimChooser::Script.tag()),
    );
    assert!(
        !sim.world().get::<SpriteAnimator>(e).unwrap().autoplay,
        "Script"
    );
    edit(
        &mut sim,
        e,
        &reg,
        AnimFieldEdit::Chooser(AnimChooser::Me.tag()),
    );
    assert!(sim.world().get::<SpriteAnimator>(e).unwrap().autoplay, "Me");
    edit(&mut sim, e, &reg, AnimFieldEdit::StartsWith("run".into()));
    assert_eq!(
        sim.world()
            .get::<SpriteAnimator>(e)
            .unwrap()
            .start
            .as_deref(),
        Some("run")
    );
    assert_eq!(
        build_anim_info(sim.world(), e.to_bits(), 1)
            .expect("§11")
            .card
            .starts_with,
        "run"
    );
}

/// Starts with = `run` + o Play (o arranque do Autoplay e o tique do motor) ⇒ o quadro mostrado é o de `run`.
#[test]
fn comecar_com_run_mostra_o_quadro_de_run_no_play() {
    let s = heroi_p14();
    let mut sim = SimWorld::new();
    let reg = registry();
    let e = pixel(&mut sim, &s);
    edit(&mut sim, e, &reg, AnimFieldEdit::StartsWith("run".into()));
    let editor = s.frames()[0].id;
    crate::tick::start_autoplay_animations(&mut sim);
    let id = |i: usize| s.frames()[i].id;
    assert_eq!(
        quadro_mostrado(sim.world(), e, &s, editor),
        id(2),
        "o Play começa no 1.º quadro de run"
    );
    // 125 ms a 1 kHz: o quadro 2 dura 120 ms ⇒ o 3, ainda dentro de run.
    let mut drive = ph2d_preview_drive::PreviewDrive::default();
    crate::tick::tick_sprite_animations(&mut sim, 125, 0.001, &[], &mut drive);
    assert_eq!(quadro_mostrado(sim.world(), e, &s, editor), id(3));
}
