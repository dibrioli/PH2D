//! ⭐ **O «Starts with» pela porta do desfazer e do abrir** ([ficha](../../../../docs/GameEngineFuncional/componentes/02_animated_sprite.md)):
//! o componente viaja no instantâneo, repor o de antes desfaz, e o projeto antigo abre igual.

use crate::comp::registo;
use ph2d_app_sprite_anim::inspector::apply_anim_edit;
use ph2d_app_sprite_anim::tick::start_autoplay_animations;
use ph2d_ecs::scene::{
    EditorCommandQueue, WorldSnapshot, apply_editor_commands, snapshot_to_world, world_to_snapshot,
};
use ph2d_ecs::{
    AnimationTag, Entity, SimWorld, SpriteAnimStart, SpriteAnimations, SpriteAnimator, SpriteGrid,
};
use ph2d_editor_core::AnimFieldEdit;

fn cena() -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    let mut lib = SpriteAnimations::new();
    lib.insert(AnimationTag::new("idle", 0, 3)).expect("idle");
    lib.insert(AnimationTag::new("run", 4, 7)).expect("run");
    let mut a = SpriteAnimator::new("idle");
    a.autoplay = true;
    let e = sim
        .world_mut()
        .spawn((
            ph2d_ecs::Transform::default(),
            ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
            SpriteGrid {
                hframes: 8,
                vframes: 1,
                frame: 0,
            },
            lib,
            a,
        ))
        .id();
    (sim, e)
}

fn edita(sim: &mut SimWorld, e: Entity, edit: AnimFieldEdit) {
    let (q, r) = (EditorCommandQueue::new(), registo());
    let _ = apply_anim_edit(sim, e.to_bits(), &edit, &q, &r);
    apply_editor_commands(sim.world_mut(), &q, &r).expect("o commit aplica");
}

fn foto(sim: &mut SimWorld) -> WorldSnapshot {
    let r = registo();
    let mut prop = ph2d_ecs::TransformPropagationState::new(sim.world_mut());
    let mut work = ph2d_ecs::WorklistBuf::default();
    let mut snap = WorldSnapshot::default();
    world_to_snapshot(sim.world_mut(), &mut prop, &mut work, &r, &mut snap).expect("captura");
    snap
}

fn aberto(snap: &WorldSnapshot) -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    snapshot_to_world(sim.world_mut(), snap, &registo()).expect("repõe");
    let w = sim.world_mut();
    let e = w
        .query::<(Entity, &SpriteAnimator)>()
        .iter(w)
        .next()
        .expect("o tocador")
        .0;
    (sim, e)
}

fn atual(sim: &SimWorld, e: Entity) -> String {
    sim.world()
        .get::<SpriteAnimator>(e)
        .expect("tocador")
        .current
        .clone()
}

/// ⭐ O gesto inteiro: ▶ em `run`, gravar e reabrir (a porta do undo e do save) começa em `idle`; desfazer tira o início.
/// *Mutação: deixar de registar o `SpriteAnimStart` ⇒ a foto perde-o e o reabrir fica em `run`.*
#[test]
fn o_starts_with_viaja_na_foto_e_o_desfazer_repoe() {
    let (mut sim, e) = cena();
    let antes = foto(&mut sim);
    edita(&mut sim, e, AnimFieldEdit::SetCurrent("run".into()));
    edita(&mut sim, e, AnimFieldEdit::Playing(true));
    assert_eq!(atual(&sim, e), "run");

    let (mut reaberto, r) = aberto(&foto(&mut sim));
    assert_eq!(
        reaberto
            .world()
            .get::<SpriteAnimStart>(r)
            .map(|s| s.0.as_str()),
        Some("idle")
    );
    start_autoplay_animations(&mut reaberto);
    assert_eq!(atual(&reaberto, r), "idle");

    let (desfeito, d) = aberto(&antes);
    assert!(desfeito.world().get::<SpriteAnimStart>(d).is_none());
    assert_eq!(atual(&desfeito, d), "idle");
}

/// O Play joga sobre a cópia que a fotografia guardou: começar no início não escreve no documento fotografado.
#[test]
fn a_foto_tirada_antes_de_comecar_guarda_a_previa() {
    let (mut sim, e) = cena();
    edita(&mut sim, e, AnimFieldEdit::SetCurrent("run".into()));
    let documento = foto(&mut sim);
    start_autoplay_animations(&mut sim);
    assert_eq!(atual(&sim, e), "idle", "o jogo");
    let (volta, d) = aberto(&documento);
    assert_eq!(atual(&volta, d), "run", "o Stop devolve o documento");
}

/// Sem o blob (projeto de antes): abre, e começa no `current` gravado.
#[test]
fn o_projeto_antigo_abre_igual() {
    let (mut sim, e) = cena();
    edita(&mut sim, e, AnimFieldEdit::SetCurrent("run".into()));
    sim.world_mut().entity_mut(e).remove::<SpriteAnimStart>();
    let gravado = foto(&mut sim);
    let (mut velho, v) = aberto(&gravado);
    // Abrir não inventa os componentes novos, e gravar de novo dá os mesmos bytes.
    assert!(velho.world().get::<SpriteAnimStart>(v).is_none());
    assert!(velho.world().get::<ph2d_ecs::SpriteCutNotice>(v).is_none());
    assert_eq!(foto(&mut velho), gravado);
    start_autoplay_animations(&mut velho);
    assert_eq!(atual(&velho, v), "run");
}
