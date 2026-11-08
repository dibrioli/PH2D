//! Os marcadores em quadro pelo mundo: o clique no cartão vira commit, e o snapshot lê de volta. Saiu de
//! `ph2d-app-sprite-anim` pela catraca da crate; as edições puras estão em `ph2d_ecs::SpriteAnimMarkers`.

use ph2d_app_sprite_anim::inspector::{apply_anim_edit, build_anim_info};
use ph2d_ecs::scene::{
    ComponentRegistry, EditorCommandQueue, apply_editor_commands, register_ecs_components,
};
use ph2d_ecs::{Entity, SimWorld, SpriteAnimMarkers, SpriteAnimations, SpriteGrid, Transform};
use ph2d_editor_core::{AnimFieldEdit, Toast};

fn registry() -> ComponentRegistry {
    let mut r = ComponentRegistry::new();
    register_ecs_components(&mut r);
    ph2d_render::register_render_components(&mut r);
    r
}

/// Uma sprite de `cells` células.
fn sprite(sim: &mut SimWorld, cells: u32) -> Entity {
    let grade = SpriteGrid {
        hframes: cells,
        vframes: 1,
        frame: 0,
        sheet: 0,
    };
    let arte = ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]);
    sim.world_mut()
        .spawn((Transform::default(), arte, grade))
        .id()
}

fn edit(
    sim: &mut SimWorld,
    e: Entity,
    reg: &ComponentRegistry,
    ed: AnimFieldEdit,
) -> Option<Toast> {
    let queue = EditorCommandQueue::new();
    let t = apply_anim_edit(sim, e.to_bits(), &ed, &queue, reg);
    apply_editor_commands(sim.world_mut(), &queue, reg).expect("o commit aplica");
    t
}

/// Uma sprite de 4 células com `walk` e `run`.
fn palco(sim: &mut SimWorld, reg: &ComponentRegistry) -> Entity {
    let e = sprite(sim, 4);
    assert!(edit(sim, e, reg, AnimFieldEdit::Add).is_none());
    assert!(edit(sim, e, reg, AnimFieldEdit::Add).is_none());
    e
}

fn marcas(sim: &SimWorld, e: Entity) -> Option<SpriteAnimMarkers> {
    sim.world().get::<SpriteAnimMarkers>(e).cloned()
}

#[test]
fn plus_creates_and_frame_and_signal_write_only_their_marker() {
    let (mut sim, reg) = (SimWorld::default(), registry());
    let e = palco(&mut sim, &reg);
    assert!(marcas(&sim, e).is_none());
    edit(&mut sim, e, &reg, AnimFieldEdit::MarkerAdd(0));
    edit(&mut sim, e, &reg, AnimFieldEdit::MarkerAdd(0));
    edit(&mut sim, e, &reg, AnimFieldEdit::MarkerAdd(1));
    let m = marcas(&sim, e).expect("o primeiro + cria o componente");
    let nomes: Vec<String> = sim
        .world()
        .get::<SpriteAnimations>(e)
        .expect("biblioteca")
        .iter()
        .map(|t| t.name.clone())
        .collect();
    let nome = |i: usize| nomes[i].clone();
    assert_eq!(
        m.of_tag(&nome(0)).map(|x| x.step).collect::<Vec<_>>(),
        [0, 1]
    );
    assert_eq!(m.of_tag(&nome(1)).count(), 1);

    edit(&mut sim, e, &reg, AnimFieldEdit::MarkerStep(0, 1, 3));
    edit(
        &mut sim,
        e,
        &reg,
        AnimFieldEdit::MarkerSignal(0, 1, "  footstep ".into()),
    );
    // O quadro tem teto no commit: a animação tem 4 quadros.
    edit(&mut sim, e, &reg, AnimFieldEdit::MarkerStep(1, 0, 99));
    let m = marcas(&sim, e).expect("marcadores");
    let a: Vec<_> = m.of_tag(&nome(0)).cloned().collect();
    assert_eq!((a[0].step, a[0].signal.as_str()), (0, ""));
    assert_eq!((a[1].step, a[1].signal.as_str()), (3, "footstep"));
    assert_eq!(m.of_tag(&nome(1)).next().map(|x| x.step), Some(3));
    // E o snapshot lê o que se escreveu.
    let info = build_anim_info(sim.world(), e.to_bits(), 1).expect("info");
    assert_eq!(info.rows[0].markers[1].signal, "footstep");
    assert_eq!(info.rows[0].steps, 4);
}

/// Provado pelo gesto: o «Rename» da lista leva o marcador junto.
#[test]
fn renaming_the_tag_carries_its_marker() {
    let (mut sim, reg) = (SimWorld::default(), registry());
    let e = palco(&mut sim, &reg);
    edit(&mut sim, e, &reg, AnimFieldEdit::MarkerAdd(0));
    edit(
        &mut sim,
        e,
        &reg,
        AnimFieldEdit::MarkerSignal(0, 0, "step".into()),
    );
    assert!(edit(&mut sim, e, &reg, AnimFieldEdit::Rename(0, "hop".into())).is_none());
    let m = marcas(&sim, e).expect("marcadores");
    assert_eq!(m.of_tag("hop").count(), 1);
    let info = build_anim_info(sim.world(), e.to_bits(), 1).expect("info");
    assert_eq!(info.rows[0].markers[0].signal, "step");
}

/// Um marcador cujo quadro passou do fim fica no dado; o painel é que avisa. E remover a animação o leva.
#[test]
fn a_marker_past_the_end_stays_and_goes_only_with_its_animation() {
    let (mut sim, reg) = (SimWorld::default(), registry());
    let e = palco(&mut sim, &reg);
    edit(&mut sim, e, &reg, AnimFieldEdit::MarkerAdd(0));
    edit(&mut sim, e, &reg, AnimFieldEdit::MarkerStep(0, 0, 3));
    // Encolhe a animação para 2 quadros.
    edit(&mut sim, e, &reg, AnimFieldEdit::To(0, 1));
    let info = build_anim_info(sim.world(), e.to_bits(), 1).expect("info");
    assert_eq!(info.rows[0].steps, 2);
    assert_eq!(info.rows[0].markers[0].step, 3, "nunca some calado");
    edit(&mut sim, e, &reg, AnimFieldEdit::Remove(0));
    assert!(marcas(&sim, e).is_none(), "a animação levou o marcador");
}
