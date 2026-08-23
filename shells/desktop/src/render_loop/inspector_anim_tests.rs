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
    build_anim_info(sim.world(), e.to_bits(), 1).expect("e' uma sprite")
}

/// **⛔ Uma entidade sem `Sprite` não tem §11** — o pool é a grelha dela.
#[test]
fn only_a_sprite_gets_the_animation_section() {
    let mut sim = SimWorld::new();
    let plain = sim.world_mut().spawn(Transform::default()).id();
    assert!(build_anim_info(sim.world(), plain.to_bits(), 1).is_none());
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

/// **Recuperar um engasgo numa chamada dá o MESMO que recuperar passo a passo.**
///
/// ⚠️ **Este gate existe porque uma mutação derrubou uma afirmação minha.** O tique corria um laço
/// de `ticks` chamadas, com a justificação de que um passo grande «atravessaria o fim de um ciclo
/// sem o fechar» — e a mutação que trocou o laço por uma chamada única **passou**. A lei tem o seu
/// próprio laço de recuperação, e é ele que fecha cada ciclo.
///
/// A equivalência é o que autoriza a forma simples; sem gate, ela seria uma suposição.
#[test]
fn catching_up_in_one_call_is_the_same_as_catching_up_step_by_step() {
    use ph2d_ecs::{AnimationTag, SpriteAnimator, advance};

    // Uma tag com repetição FINITA e ping-pong: os dois casos onde um passo grande poderia
    // atropelar o fecho de um ciclo.
    for tag in [
        AnimationTag {
            repeat: Some(3),
            frame_ms: 20,
            ..AnimationTag::new("x", 0, 3)
        },
        AnimationTag {
            direction: AnimDirection::PingPong,
            repeat: Some(2),
            frame_ms: 20,
            hold_ms: 30,
            repeat_delay_ms: 50,
            ..AnimationTag::new("y", 0, 4)
        },
    ] {
        let dt = 16_667_u64;
        let ticks = 8_u32; // o teto de sub-passos do passo fixo

        let mut a1 = SpriteAnimator::new(&tag.name);
        a1.playing = true;
        a1.speed_q16 = ph2d_ecs::SPEED_ONE_Q16;
        a1.rewind(Some(&tag));
        let mut f1 = u32::MAX;
        for _ in 0..ticks {
            advance(&mut a1, &tag, &mut f1, 8, dt);
            if !a1.playing {
                break;
            }
        }

        let mut a2 = a1.clone();
        a2.playing = true;
        a2.speed_q16 = ph2d_ecs::SPEED_ONE_Q16;
        a2.rewind(Some(&tag));
        let mut f2 = u32::MAX;
        advance(&mut a2, &tag, &mut f2, 8, dt * u64::from(ticks));

        assert_eq!(
            (f1, a1.repeat_count, a1.playing),
            (f2, a2.repeat_count, a2.playing),
            "'{}' divergiu entre as duas formas de recuperar",
            tag.name
        );
    }
}

/// **Os gates do TRANSPORTE da §11** — irmão deste ficheiro pelo teto de LOC do shell (HR-18).
///
/// ⚠️ **O corte é por LEI, e não por tamanho:** aqui mora o que responde *«o que está a tocar, e
/// a partir de onde»* — rebobinar, escolher, ligar, retomar. O que fica no ficheiro-pai é a
/// AUTORIA (a biblioteca, os nomes, os intervalos) e o snapshot. São as duas metades que a própria
/// seção separa com dois rótulos.
///
/// As ferramentas (`registry`, `sprite`, `edit`, `info`) vêm do pai por `use super::*`.
#[path = "inspector_anim_transport_tests.rs"]
mod transport;

/// **O MOTOR e o PAINEL dão a mesma resposta a «esta reprodução está pendurada?»**
///
/// ⚠️ A lei existe **duas vezes**, e de propósito: o painel é chrome e não vê o motor
/// ([`ph2d_ecs::animator_state`] vive no `ph2d-ecs`; [`InspectorAnimInfo::current_dangling`] no
/// modelo do editor). É o mesmo padrão do `ph2d_ecs_dir_label` a espelhar
/// `AnimDirection::label` — e, como ali, quem impede as duas de divergirem é um gate da SHELL, o
/// único sítio que vê os dois lados.
///
/// ⚠️ **E foi a auditoria que o encontrou** (2026-08-23): a `animator_state` estava exportada,
/// tinha gate próprio no `ph2d-ecs` e **nenhum consumidor** — enquanto o painel reimplementava a
/// lei sem nada a prendê-los. *Uma segunda resposta sem oráculo é uma divergência com data
/// marcada.*
///
/// **Mutação que deve sangrar:** tirar o `fits` de uma das duas.
#[test]
fn the_panel_and_the_engine_agree_on_a_dangling_playback() {
    let reg = registry();
    let mut sim = SimWorld::new();
    let e = sprite(&mut sim, 8);
    let mut lib = SpriteAnimations::new();
    lib.insert(AnimationTag::new("walk", 0, 3)).unwrap();
    lib.insert(AnimationTag::new("far", 6, 7)).unwrap();
    sim.world_mut().entity_mut(e).insert(lib);
    edit(&mut sim, e, &reg, AnimFieldEdit::AddPlayer);

    // ⚠️ Os QUATRO casos, e o terceiro e o quarto sao os que se leem igual no ecra'.
    for (name, cells, what) in [
        ("", 8, "sem escolha"),
        ("walk", 8, "a tocar uma que cabe"),
        ("ghost", 8, "o nome nao esta' na biblioteca"),
        ("far", 4, "a grelha encolheu debaixo dela"),
    ] {
        edit(&mut sim, e, &reg, AnimFieldEdit::SetCurrent(name.into()));
        if let Some(mut s) = sim.world_mut().get_mut::<ph2d_render::Sprite>(e) {
            s.hframes = cells;
        }
        let animator = sim
            .world()
            .get::<ph2d_ecs::SpriteAnimator>(e)
            .cloned()
            .expect("o tocador foi anexado");
        let tags = sim.world().get::<SpriteAnimations>(e);
        let engine = ph2d_ecs::animator_state(&animator, tags, cells);
        let panel = info(&sim, e).current_dangling();
        assert_eq!(
            matches!(engine, ph2d_ecs::AnimatorState::Dangling),
            panel,
            "o motor diz {engine:?} e o painel diz dangling={panel} para «{what}»"
        );
    }
}
