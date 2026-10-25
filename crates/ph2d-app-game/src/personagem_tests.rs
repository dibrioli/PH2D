use super::*;
use ph2d_ecs::ChildOf;

/// O animador é o do próprio personagem; sem ele, o do 1.º filho que tem um (a imagem dentro do corpo).
#[test]
fn o_alvo_e_o_proprio_ou_o_filho_com_animador() {
    let mut w = World::new();
    let so = w.spawn(SpriteAnimator::new("idle")).id();
    assert_eq!(alvo(&w, so), Some(so));
    let corpo = w.spawn_empty().id();
    let _vazio = w.spawn(ChildOf(corpo)).id();
    let imagem = w.spawn((SpriteAnimator::new("idle"), ChildOf(corpo))).id();
    assert_eq!(alvo(&w, corpo), Some(imagem));
    let nada = w.spawn_empty().id();
    assert_eq!(alvo(&w, nada), None);
}

/// ⭐ Report do dono 25/10 («ao pular não volta mais a correr»): o salto toca UMA vez e, ao acabar, o
/// animador pára; a troca para correr tem de o pôr a tocar outra vez, do início.
#[test]
fn depois_do_salto_que_acabou_a_corrida_toca() {
    let mut a = SpriteAnimator::new("jump");
    a.playing = false;
    a.repeat_count = 1;
    a.elapsed_ticks = 9_999;
    troca(&mut a, "run".into());
    assert_eq!(
        (
            a.current.as_str(),
            a.playing,
            a.elapsed_ticks,
            a.repeat_count
        ),
        ("run", true, 0, 0)
    );
}

/// Sem vista do player (a física desarmada), a ponte não escreve nada.
#[test]
fn sem_vista_nao_escreve() {
    let mut w = World::new();
    let e = w
        .spawn((CharacterAnimations::default(), SpriteAnimator::new("idle")))
        .id();
    anima_personagens(&mut w, &PhysicsBridge::new());
    assert_eq!(
        w.get::<SpriteAnimator>(e).map(|a| a.current.as_str()),
        Some("idle")
    );
}
