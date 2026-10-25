//! ⭐ **O Stop devolve o documento ao BIT** (R0a): a sessão + a porta real do undo (captura e
//! restauro), sobre o que um jogo faz ao mundo — mover, renomear, criar, apagar.
//!
//! ⚠️ A cola no quadro (`App::jogo_quadro`) exige `gfx`; a ordem dela no quadro é provada em
//! `fase_sequences_tests::o_play_de_cima_chega_a_timeline_antes_do_relogio_da_vista`, e o app real
//! imprime os dois resumos no smoke (`PH2D_GAME_SMOKE`).

use super::super::tests::{capture, registry, scene};
use ph2d_app_game::{Passo, Sessao};
use ph2d_ecs::{Name, Transform};

/// O que um jogo faz ao mundo num punhado de quadros.
fn joga(sim: &mut ph2d_ecs::SimWorld) {
    let w = sim.world_mut();
    let mut q = w.query::<(&mut Transform, &mut Name)>();
    for (mut t, mut n) in q.iter_mut(w) {
        if n.as_str() == "Spr" {
            t.translation.x += 40.0;
            *n = Name::new("Spr (hit)");
        }
    }
    w.spawn((
        Transform::from_translation(ph2d_core::Vec2::new(9.0, 9.0)),
        Name::new("Bullet"),
    ));
}

#[test]
fn o_stop_devolve_o_documento_ao_bit() {
    let reg = registry();
    let (mut sim, vec) = scene();
    let antes = capture(&mut sim, &vec, &reg);
    let mut s = Sessao::default();

    assert_eq!(s.quadro(true), Passo::Fotografa);
    s.guarda(capture(&mut sim, &vec, &reg));
    joga(&mut sim);
    // Controlo: o jogo mudou de facto o documento — sem isto o gate passaria no vazio.
    let a_jogar = capture(&mut sim, &vec, &reg);
    assert_ne!(
        a_jogar.digest(),
        antes.digest(),
        "o jogo não mexeu no mundo"
    );
    assert_eq!(s.quadro(true), Passo::Nada, "a pausa não repõe");

    let Passo::Repoe(foto) = s.quadro(false) else {
        panic!("a saída da corrida tem de repor a fotografia");
    };
    let (rvec, _, _, _) = foto.restore(&mut sim, &reg);
    let depois = capture(&mut sim, &rvec, &reg);
    assert_eq!(depois, antes, "o Stop deixou o documento diferente");
    assert_eq!(depois.digest(), antes.digest());
}

/// O resumo é o do CONTEÚDO: a mesma cena capturada duas vezes dá o mesmo número, e uma só mudança
/// de nome dá outro.
#[test]
fn o_resumo_e_o_do_conteudo() {
    let reg = registry();
    let (mut sim, vec) = scene();
    let a = capture(&mut sim, &vec, &reg).digest();
    assert_eq!(capture(&mut sim, &vec, &reg).digest(), a);
    let w = sim.world_mut();
    let mut q = w.query::<&mut Name>();
    for mut n in q.iter_mut(w) {
        *n = Name::new("outro");
    }
    assert_ne!(capture(&mut sim, &vec, &reg).digest(), a);
}

/// ⭐ **O Play liga o «tocar sozinho» das animações** (N1): só o abrir do projecto o fazia, e um
/// *Animated Sprite* ou um *Character* acabado de pôr pelo menu ficava parado no 1.º quadro a jogar.
/// A ligação vem DEPOIS da fotografia — o documento guarda o animador parado e o Stop devolve-o assim.
///
/// *Mutação: tirar a chamada do `jogo_entra` ⇒ vermelho (a ordem: pô-la antes do `capture_project`).*
#[test]
fn o_play_liga_o_autoplay_depois_da_fotografia() {
    let fonte = include_str!("undo_jogo.rs");
    let corpo = fonte
        .split_once("fn jogo_entra(&mut self) {")
        .and_then(|(_, r)| r.split_once("fn jogo_sai("))
        .expect("o jogo_entra")
        .0;
    let foto = corpo.find("self.capture_project()").expect("a fotografia");
    let liga = corpo
        .find("start_autoplay_animations(")
        .expect("o Play não liga o autoplay");
    assert!(
        foto < liga,
        "o autoplay liga-se antes da fotografia: o documento guardaria o animador a tocar"
    );
    let mut sim = ph2d_ecs::SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            {
                let mut a = ph2d_ecs::SpriteAnimator::new("idle");
                a.autoplay = true;
                a
            },
            {
                let mut t = ph2d_ecs::SpriteAnimations::new();
                t.insert(ph2d_ecs::AnimationTag::new("idle", 0, 5))
                    .expect("tag");
                t
            },
        ))
        .id();
    crate::render_loop::start_autoplay_animations(&mut sim);
    assert!(
        sim.world()
            .get::<ph2d_ecs::SpriteAnimator>(e)
            .expect("animador")
            .playing
    );
}
