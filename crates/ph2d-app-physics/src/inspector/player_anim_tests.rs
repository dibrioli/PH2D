use super::*;
use ph2d_ecs::AnimationTag;

fn tags(names: &[&str]) -> SpriteAnimations {
    let mut t = SpriteAnimations::new();
    for (i, n) in names.iter().enumerate() {
        t.insert(AnimationTag::new(*n, i as u32, i as u32)).expect("tag");
    }
    t
}

fn personagem(world: &mut World, com_anim: bool) -> Entity {
    let e = world
        .spawn((tags(&["idle", "run", "jump", "fall", "wave"]), SpriteAnimator::new("idle")))
        .id();
    if com_anim {
        world.entity_mut(e).insert(CharacterAnimations::default());
    }
    e
}

/// O interruptor anexa o componente de partida e tira-o — nunca um componente presente a dizer «off».
#[test]
fn o_interruptor_anexa_e_remove() {
    let mut w = World::new();
    let e = personagem(&mut w, false);
    assert!(anim_info(&w, e).is_none());
    assert!(apply_anim_edit(&mut w, e, PlayerFieldEdit::AnimateByState(true)));
    assert_eq!(w.get::<CharacterAnimations>(e), Some(&CharacterAnimations::default()));
    apply_anim_edit(&mut w, e, PlayerFieldEdit::AnimateByState(false));
    assert!(w.get::<CharacterAnimations>(e).is_none());
}

/// Cada clique num estado passa à animação seguinte da lista, e dá a volta.
#[test]
fn o_clique_passa_a_seguinte_e_da_a_volta() {
    let mut w = World::new();
    let e = personagem(&mut w, true);
    apply_anim_edit(&mut w, e, PlayerFieldEdit::AnimNext(1));
    assert_eq!(w.get::<CharacterAnimations>(e).unwrap().run, "jump");
    for _ in 0..3 {
        apply_anim_edit(&mut w, e, PlayerFieldEdit::AnimNext(1));
    }
    assert_eq!(w.get::<CharacterAnimations>(e).unwrap().run, "idle", "depois de `wave` volta ao início");
    assert_eq!(w.get::<CharacterAnimations>(e).unwrap().idle, "idle", "só o estado clicado muda");
}

/// Um nome que a lista não tem: o snapshot di-lo, e o clique leva-o à primeira animação.
#[test]
fn um_nome_fora_da_lista_e_dito_e_o_clique_o_corrige() {
    let mut w = World::new();
    let e = personagem(&mut w, true);
    w.get_mut::<CharacterAnimations>(e).unwrap().fall = "tumble".into();
    let info = anim_info(&w, e).unwrap();
    assert_eq!(info.known, [true, true, true, false]);
    assert!(info.has_tags);
    apply_anim_edit(&mut w, e, PlayerFieldEdit::AnimNext(3));
    assert_eq!(w.get::<CharacterAnimations>(e).unwrap().fall, "idle");
}

/// As animações podem morar no 1.º filho com animador (a imagem como peça do corpo).
#[test]
fn as_animacoes_do_filho_contam() {
    let mut w = World::new();
    let e = w.spawn(CharacterAnimations::default()).id();
    let filho = w.spawn((tags(&["idle", "run"]), SpriteAnimator::new("idle"))).id();
    w.entity_mut(e).add_child(filho);
    let info = anim_info(&w, e).unwrap();
    assert_eq!(info.known, [true, true, false, false]);
    apply_anim_edit(&mut w, e, PlayerFieldEdit::AnimNext(0));
    assert_eq!(w.get::<CharacterAnimations>(e).unwrap().idle, "run");
}

/// Sem animações em lado nenhum: o snapshot diz que não há para onde trocar, e o clique não inventa.
#[test]
fn sem_animacoes_nao_ha_troca() {
    let mut w = World::new();
    let e = w.spawn(CharacterAnimations::default()).id();
    assert!(!anim_info(&w, e).unwrap().has_tags);
    apply_anim_edit(&mut w, e, PlayerFieldEdit::AnimNext(0));
    assert_eq!(w.get::<CharacterAnimations>(e).unwrap().idle, "idle");
}

#[test]
fn o_limiar_nao_desce_de_zero_e_o_espelho_escreve() {
    let mut w = World::new();
    let e = personagem(&mut w, true);
    apply_anim_edit(&mut w, e, PlayerFieldEdit::AnimRunFrom(-3.0));
    apply_anim_edit(&mut w, e, PlayerFieldEdit::AnimFlip(false));
    let a = w.get::<CharacterAnimations>(e).unwrap();
    assert_eq!((a.run_from, a.flip), (0.0, false));
    assert!(!apply_anim_edit(&mut w, e, PlayerFieldEdit::Speed(1.0)), "edição de outro bloco");
}

/// ⭐ A PORTA que a shell chama com o `InspectorPlayerEdit` (`apply_player_edit`) chega ao componente.
#[test]
fn a_porta_da_seccao_escreve_o_componente() {
    use ph2d_physics_ecs::{BodyKind, PlatformPlayer, RigidBody};
    let mut sim = ph2d_ecs::SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            RigidBody { kind: BodyKind::Dynamic },
            PlatformPlayer::default(),
            tags(&["idle", "run", "jump", "fall"]),
            SpriteAnimator::new("idle"),
        ))
        .id();
    let bits = e.to_bits();
    super::super::player::apply_player_edit(&mut sim, bits, PlayerFieldEdit::AnimateByState(true));
    super::super::player::apply_player_edit(&mut sim, bits, PlayerFieldEdit::AnimNext(3));
    super::super::player::apply_player_edit(&mut sim, bits, PlayerFieldEdit::AnimRunFrom(1.5));
    let a = sim.world().get::<CharacterAnimations>(e).expect("o interruptor anexou");
    assert_eq!((a.fall.as_str(), a.run_from), ("idle", 1.5));
}
