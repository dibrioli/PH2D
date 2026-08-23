//! **Os gates da costura da §11** — irmão de [`super::inspector_anim`] por CAP de LOC.
//!
//! ⚠️ **Só a shell vê as duas metades**: o painel é chrome e não depende do motor, e a lei da
//! animação vive no `ph2d-ecs`. É aqui que se prova que o snapshot diz o que a cena tem e que o
//! commit escreve o que o clique pediu.

use super::*;
use ph2d_ecs::scene::{EditorCommandQueue, apply_editor_commands, register_ecs_components};
use ph2d_ecs::{AnimDirection, SpriteAnimations, Transform};

fn registry() -> ComponentRegistry {
    let mut r = ComponentRegistry::new();
    register_ecs_components(&mut r);
    ph2d_render::register_render_components(&mut r);
    r
}

/// Uma sprite com uma grelha de `cells` células.
fn sprite(sim: &mut SimWorld, cells: u32) -> Entity {
    sim.world_mut()
        .spawn((
            Transform::default(),
            ph2d_render::Sprite {
                hframes: cells,
                vframes: 1,
                ..sprite_default()
            },
        ))
        .id()
}

/// A porta canónica de uma sprite de atlas — a mesma que o resto do app usa.
fn sprite_default() -> ph2d_render::Sprite {
    ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4])
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

fn info(sim: &SimWorld, e: Entity) -> InspectorAnimInfo {
    build_anim_info(sim.world(), e.to_bits(), &[e.to_bits()], 1).expect("e' uma sprite")
}

/// **⛔ Uma entidade sem `Sprite` não tem §11** — o pool é a grelha dela.
#[test]
fn only_a_sprite_gets_the_animation_section() {
    let mut sim = SimWorld::new();
    let plain = sim.world_mut().spawn(Transform::default()).id();
    assert!(build_anim_info(sim.world(), plain.to_bits(), &[], 1).is_none());
}

/// **O caminho inteiro: anexar o tocador, criar uma animação, escolhê-la, tocá-la.**
///
/// ⚠️ A animação nova cobre a **grelha inteira** — um intervalo de uma célula não mostraria nada,
/// e o artista teria de adivinhar que o botão funcionou.
#[test]
fn the_whole_authoring_path_lands_in_the_scene() {
    let mut sim = SimWorld::new();
    let reg = registry();
    let e = sprite(&mut sim, 8);

    let before = info(&sim, e);
    assert!(!before.player_present, "sem tocador de partida");
    assert_eq!(before.cells, 8, "o pool e' a grelha");

    edit(&mut sim, e, &reg, AnimFieldEdit::AddPlayer);
    assert!(info(&sim, e).player_present);

    edit(&mut sim, e, &reg, AnimFieldEdit::Add);
    let i = info(&sim, e);
    assert_eq!(i.rows.len(), 1);
    assert_eq!(
        (i.rows[0].from, i.rows[0].to),
        (0, 7),
        "a animacao nova cobre a grelha inteira"
    );

    let name = i.rows[0].name.clone();
    edit(&mut sim, e, &reg, AnimFieldEdit::SetCurrent(name.clone()));
    edit(&mut sim, e, &reg, AnimFieldEdit::Playing(true));
    let i = info(&sim, e);
    assert_eq!(i.current, name);
    assert!(i.playing);
    assert_eq!(i.current_index(), Some(0));
    assert!(!i.current_dangling());
}

/// **A velocidade fecha o círculo: o que o artista escreve é o que ele volta a ler.**
///
/// ⚠️ E o motor guarda `Q16.16` — a conversão mora numa ponta só do ficheiro.
#[test]
fn the_speed_round_trips_through_the_fixed_point() {
    let mut sim = SimWorld::new();
    let reg = registry();
    let e = sprite(&mut sim, 4);
    edit(&mut sim, e, &reg, AnimFieldEdit::AddPlayer);

    edit(&mut sim, e, &reg, AnimFieldEdit::Speed(1.5));
    assert!((info(&sim, e).speed - 1.5).abs() < 1e-4);
    edit(&mut sim, e, &reg, AnimFieldEdit::Speed(-2.0));
    assert!((info(&sim, e).speed + 2.0).abs() < 1e-4);
    // ⚠️ E a faixa do motor é imposta na ENTRADA: um scrub além dela não pode gravar um valor
    // que a lei depois trunca — o knob prometeria uma excursão que a cena não tem.
    edit(&mut sim, e, &reg, AnimFieldEdit::Speed(1e6));
    assert!((info(&sim, e).speed - 100.0).abs() < 1e-3);
}

/// **Um animador acabado de anexar toca** — ele não nasce com velocidade zero.
///
/// ⚠️ O `Default` de um `i32` é `0`, que na lei é PAUSA. Sem esta cura, `+ Add Animator` seguido
/// de `Playing` não moveria nada, e o artista culparia a animação.
#[test]
fn a_freshly_attached_player_actually_moves() {
    let mut sim = SimWorld::new();
    let reg = registry();
    let e = sprite(&mut sim, 4);
    edit(&mut sim, e, &reg, AnimFieldEdit::AddPlayer);
    assert!(
        (info(&sim, e).speed - 1.0).abs() < 1e-4,
        "um animador novo tem de nascer a` velocidade normal"
    );
}

/// **Renomear a animação que TOCA leva o tocador junto.**
///
/// ⚠️ Sem isto, corrigir uma letra no nome deixaria o `current` a apontar para uma animação que
/// já não existe: a sprite pararia, e nada ligaria as duas coisas aos olhos do artista.
#[test]
fn renaming_the_current_animation_carries_the_player_with_it() {
    let mut sim = SimWorld::new();
    let reg = registry();
    let e = sprite(&mut sim, 4);
    edit(&mut sim, e, &reg, AnimFieldEdit::AddPlayer);
    edit(&mut sim, e, &reg, AnimFieldEdit::Add);
    let old = info(&sim, e).rows[0].name.clone();
    edit(&mut sim, e, &reg, AnimFieldEdit::SetCurrent(old));
    edit(
        &mut sim,
        e,
        &reg,
        AnimFieldEdit::Rename(0, "walk_cycle".into()),
    );
    let i = info(&sim, e);
    assert_eq!(i.current, "walk_cycle");
    assert!(
        !i.current_dangling(),
        "o tocador ficou a apontar para o vazio"
    );
}

/// **Um nome repetido é RECUSADO com aviso**, nunca em silêncio.
#[test]
fn a_duplicate_name_is_refused_out_loud() {
    let mut sim = SimWorld::new();
    let reg = registry();
    let e = sprite(&mut sim, 4);
    edit(&mut sim, e, &reg, AnimFieldEdit::Add);
    edit(&mut sim, e, &reg, AnimFieldEdit::Add);
    let first = info(&sim, e).rows[0].name.clone();
    let toast = edit(&mut sim, e, &reg, AnimFieldEdit::Rename(1, first.clone()));
    assert!(toast.is_some(), "recusar em silencio e' pior que aceitar");
    assert_ne!(info(&sim, e).rows[1].name, first, "e nao escreveu");
}

/// **`0` no campo Repeat significa «para sempre»** — e volta a ler-se como `0`.
#[test]
fn zero_repeat_means_forever_in_both_directions() {
    let mut sim = SimWorld::new();
    let reg = registry();
    let e = sprite(&mut sim, 4);
    edit(&mut sim, e, &reg, AnimFieldEdit::Add);
    edit(&mut sim, e, &reg, AnimFieldEdit::Repeat(0, 3));
    assert_eq!(info(&sim, e).rows[0].repeat, 3);
    assert_eq!(
        sim.world()
            .get::<SpriteAnimations>(e)
            .unwrap()
            .iter()
            .next()
            .unwrap()
            .repeat,
        Some(3)
    );
    edit(&mut sim, e, &reg, AnimFieldEdit::Repeat(0, 0));
    assert_eq!(info(&sim, e).rows[0].repeat, 0);
    assert_eq!(
        sim.world()
            .get::<SpriteAnimations>(e)
            .unwrap()
            .iter()
            .next()
            .unwrap()
            .repeat,
        None,
        "o zero do campo tem de virar `None` no motor"
    );
}

/// **As duas substituições viajam pelos MESMOS tags que o painel pinta.**
///
/// ⚠️ `0` é «herdar» nas duas, e é o deslocamento de um que as separa da direção da tag. Trocar o
/// `+1`/`-1` compila e faz o artista escolher `Forward` e obter `Reverse`.
#[test]
fn the_two_overrides_travel_by_the_tags_the_panel_paints() {
    let mut sim = SimWorld::new();
    let reg = registry();
    let e = sprite(&mut sim, 4);
    edit(&mut sim, e, &reg, AnimFieldEdit::AddPlayer);

    for (tag, want) in [
        (0u8, None),
        (1, Some(AnimDirection::Forward)),
        (2, Some(AnimDirection::Reverse)),
        (3, Some(AnimDirection::PingPong)),
        (4, Some(AnimDirection::PingPongReverse)),
    ] {
        edit(&mut sim, e, &reg, AnimFieldEdit::DirectionOverride(tag));
        assert_eq!(
            sim.world()
                .get::<ph2d_ecs::SpriteAnimator>(e)
                .unwrap()
                .direction_override,
            want,
            "o tag {tag} escreveu outra direcao"
        );
        assert_eq!(
            info(&sim, e).direction_override_tag,
            tag,
            "e nao volta igual"
        );
    }
    for (tag, want) in [(0u8, None), (1, Some(true)), (2, Some(false))] {
        edit(&mut sim, e, &reg, AnimFieldEdit::LoopOverride(tag));
        assert_eq!(
            sim.world()
                .get::<ph2d_ecs::SpriteAnimator>(e)
                .unwrap()
                .loop_override,
            want
        );
        assert_eq!(info(&sim, e).loop_override_tag, tag);
    }
}

/// **Trocar de animação REBOBINA o ciclo.**
#[test]
fn choosing_another_animation_rewinds_the_cycle() {
    let mut sim = SimWorld::new();
    let reg = registry();
    let e = sprite(&mut sim, 8);
    edit(&mut sim, e, &reg, AnimFieldEdit::AddPlayer);
    edit(&mut sim, e, &reg, AnimFieldEdit::Add);
    edit(&mut sim, e, &reg, AnimFieldEdit::Add);
    let (a, b) = {
        let i = info(&sim, e);
        (i.rows[0].name.clone(), i.rows[1].name.clone())
    };
    edit(&mut sim, e, &reg, AnimFieldEdit::SetCurrent(a));
    // Suja o estado à mão, como um ciclo a meio faria.
    if let Some(mut p) = sim.world_mut().get_mut::<ph2d_ecs::SpriteAnimator>(e) {
        p.repeat_count = 5;
        p.elapsed_ticks = 999;
    }
    edit(&mut sim, e, &reg, AnimFieldEdit::SetCurrent(b));
    let p = sim.world().get::<ph2d_ecs::SpriteAnimator>(e).unwrap();
    assert_eq!((p.repeat_count, p.elapsed_ticks), (0, 0));
}

/// **O TIQUE escreve `Sprite::frame`, e só quando ele MUDA.**
///
/// ⚠️ A segunda metade não é zelo: o `Sprite` é `SimComponent` e o undo regista por DIFF. Tocar
/// no componente todo o quadro faria uma sprite pausada produzir um passo de undo por quadro.
#[test]
fn the_tick_writes_the_frame_and_only_when_it_changes() {
    let mut sim = SimWorld::new();
    let reg = registry();
    let e = sprite(&mut sim, 4);
    edit(&mut sim, e, &reg, AnimFieldEdit::AddPlayer);
    edit(&mut sim, e, &reg, AnimFieldEdit::Add);
    let name = info(&sim, e).rows[0].name.clone();
    edit(&mut sim, e, &reg, AnimFieldEdit::SetCurrent(name));
    edit(&mut sim, e, &reg, AnimFieldEdit::FrameMs(0, 100));
    edit(&mut sim, e, &reg, AnimFieldEdit::Playing(true));

    // 60 Hz: seis tiques ≈ 100 ms ⇒ um frame.
    let dt = 1.0 / 60.0;
    crate::render_loop::sprite_anim_tick::tick_sprite_animations(&mut sim, 6, dt);
    assert_eq!(info(&sim, e).frame, 1);
    crate::render_loop::sprite_anim_tick::tick_sprite_animations(&mut sim, 6, dt);
    assert_eq!(info(&sim, e).frame, 2);

    // Pausado: o frame fica.
    edit(&mut sim, e, &reg, AnimFieldEdit::Playing(false));
    crate::render_loop::sprite_anim_tick::tick_sprite_animations(&mut sim, 600, dt);
    assert_eq!(info(&sim, e).frame, 2, "pausado nao anda");
}

/// **O `autoplay` arranca ao carregar o projeto, e mais em lado nenhum.**
#[test]
fn autoplay_starts_only_at_load_time() {
    let mut sim = SimWorld::new();
    let reg = registry();
    let e = sprite(&mut sim, 4);
    edit(&mut sim, e, &reg, AnimFieldEdit::AddPlayer);
    edit(&mut sim, e, &reg, AnimFieldEdit::Add);
    let name = info(&sim, e).rows[0].name.clone();
    edit(&mut sim, e, &reg, AnimFieldEdit::SetCurrent(name));
    edit(&mut sim, e, &reg, AnimFieldEdit::Autoplay(true));
    assert!(!info(&sim, e).playing, "marcar a caixa NAO comeca a tocar");

    crate::render_loop::sprite_anim_tick::start_autoplay_animations(&mut sim);
    assert!(info(&sim, e).playing, "o load tem de comecar");
}

/// **A conversão do passo fixo ARREDONDA.**
///
/// ⚠️ A 60 Hz o passo é `16666,66…` µs; truncar perderia 0,66 µs por tique — 2,4 ms por minuto.
/// Invisível num quadro, e uma animação longa dessincronizaria de um relógio que não trunca.
#[test]
fn the_fixed_step_conversion_rounds_instead_of_truncating() {
    assert_eq!(
        crate::render_loop::sprite_anim_tick::step_ticks(1.0 / 60.0),
        16_667
    );
    assert_eq!(
        crate::render_loop::sprite_anim_tick::step_ticks(1.0 / 120.0),
        8_333
    );
    assert_eq!(crate::render_loop::sprite_anim_tick::step_ticks(0.0), 0);
}
