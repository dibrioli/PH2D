//! **Os gates da costura da §11** — irmão de [`crate::inspector`] por CAP de LOC.
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
            sprite_default(),
            // ⭐ A grelha é um componente (ADR-0164 F1 passo 6).
            ph2d_ecs::SpriteGrid {
                hframes: cells,
                vframes: 1,
                frame: 0,
                sheet: 0,
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
/// ⚠️ A animação nova nasce **sem quadros** (smoke do dono 09/11: a folha inteira entrava sem ele pedir): o
/// artista clica as células no editor. Escolhida e a tocar, não toca nada e não é alarme («fora da grelha»).
#[test]
fn the_whole_authoring_path_lands_in_the_scene() {
    let mut sim = SimWorld::new();
    let reg = registry();
    let e = sprite(&mut sim, 8);

    // ⚠️ **A §11 não existe sobre uma sprite SEM biblioteca nem tocador** (ADR-0166 / F3): a seção
    // segue os componentes dela, e a rota para o primeiro é o `+` do cabeçalho.
    assert!(
        build_anim_info(sim.world(), e.to_bits(), 1).is_none(),
        "uma sprite pelada nao tem §11 Animation"
    );

    edit(&mut sim, e, &reg, AnimFieldEdit::AddPlayer);
    let before = info(&sim, e);
    assert!(before.player_present);
    assert_eq!(before.cells, 8, "o pool e' a grelha");

    edit(&mut sim, e, &reg, AnimFieldEdit::Add);
    let i = info(&sim, e);
    assert_eq!(i.rows.len(), 1);
    assert_eq!(
        i.rows[0].frames,
        Some(Vec::new()),
        "a animacao nova nasce vazia"
    );
    assert_eq!(i.rows[0].steps, 0);

    let name = i.rows[0].name.clone();
    edit(&mut sim, e, &reg, AnimFieldEdit::SetCurrent(name.clone()));
    edit(&mut sim, e, &reg, AnimFieldEdit::Playing(true));
    let i = info(&sim, e);
    assert_eq!(i.current, name);
    assert!(i.playing);
    assert_eq!(i.current_index(), Some(0));
    assert!(!i.current_dangling());
    assert_eq!(i.progress(), None, "sem quadros nao ha barra");
}

/// ⭐ **O «+» escolhe a nova** (handoff GEF 09/11, ABERTO 4): pela porta única das duas superfícies, o cartão
/// Animation e o editor; o cartão segue o `current`. E o jogo continua a começar na de antes, a lei do `SetCurrent`.
/// *Mutação: `commit_anim_edit` sem o `SetCurrent` da nova ⇒ o `current` fica na primeira.*
#[test]
fn o_mais_escolhe_a_animacao_nova() {
    let mut sim = SimWorld::new();
    let reg = registry();
    let e = sprite(&mut sim, 8);
    edit(&mut sim, e, &reg, AnimFieldEdit::AddPlayer);
    let commit = |sim: &mut SimWorld| {
        let queue = EditorCommandQueue::new();
        commit_anim_edit(sim, e.to_bits(), &AnimFieldEdit::Add, &queue, &reg)
    };
    assert!(matches!(commit(&mut sim), Commit::Feito));
    let primeira = info(&sim, e).rows[0].name.clone();
    assert_eq!(info(&sim, e).current, primeira, "a primeira fica escolhida");
    assert!(matches!(commit(&mut sim), Commit::Feito));
    let i = info(&sim, e);
    assert_eq!(i.rows.len(), 2);
    assert_eq!(i.current, i.rows[1].name, "a nova fica escolhida");
    assert_eq!(i.current_index(), Some(1));
    let inicio = sim
        .world()
        .get::<SpriteAnimator>(e)
        .and_then(|p| p.start.clone());
    assert_eq!(
        inicio,
        Some(primeira),
        "o jogo continua a começar na de antes"
    );
    let pedidos = sim
        .world()
        .get_resource::<crate::editor::AnimEditorAlignRequests>();
    assert_eq!(
        pedidos.map(|p| p.0.clone()),
        Some(vec![(e.to_bits(), false); 2]),
        "e sai do Align"
    );
    // Sem tocador, o Add não inventa um para escolher a nova.
    let so = sprite(&mut sim, 8);
    let queue = EditorCommandQueue::new();
    assert!(matches!(
        commit_anim_edit(&mut sim, so.to_bits(), &AnimFieldEdit::Add, &queue, &reg),
        Commit::Feito
    ));
    assert!(sim.world().get::<SpriteAnimator>(so).is_none());
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
    edit(&mut sim, e, &reg, AnimFieldEdit::Frames(0, celulas(0..=3)));
    let name = info(&sim, e).rows[0].name.clone();
    edit(&mut sim, e, &reg, AnimFieldEdit::SetCurrent(name));
    edit(&mut sim, e, &reg, AnimFieldEdit::FrameMs(0, 100));
    edit(&mut sim, e, &reg, AnimFieldEdit::Playing(true));

    // 60 Hz: seis tiques ≈ 100 ms ⇒ um frame.
    let dt = 1.0 / 60.0;
    crate::tick::tick_sprite_animations(
        &mut sim,
        6,
        dt,
        &[],
        &mut ph2d_preview_drive::PreviewDrive::default(),
    );
    assert_eq!(info(&sim, e).frame.cell, 1);
    crate::tick::tick_sprite_animations(
        &mut sim,
        6,
        dt,
        &[],
        &mut ph2d_preview_drive::PreviewDrive::default(),
    );
    assert_eq!(info(&sim, e).frame.cell, 2);

    // Pausado: o frame fica.
    edit(&mut sim, e, &reg, AnimFieldEdit::Playing(false));
    crate::tick::tick_sprite_animations(
        &mut sim,
        600,
        dt,
        &[],
        &mut ph2d_preview_drive::PreviewDrive::default(),
    );
    assert_eq!(info(&sim, e).frame.cell, 2, "pausado nao anda");
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

    crate::tick::start_autoplay_animations(&mut sim);
    assert!(info(&sim, e).playing, "o load tem de comecar");
}

/// As células `r` da folha `0`, por ordem (o «+» cria a animação sem quadros; o artista clica-as).
fn celulas(r: std::ops::RangeInclusive<u32>) -> Vec<ph2d_editor_core::AnimFrameRef> {
    r.map(ph2d_editor_core::AnimFrameRef::cell).collect()
}

/// `idle` e `run` numa sprite «Auto», como o Add ▸ Animated Sprite deixa um projeto antigo.
fn auto(sim: &mut SimWorld, reg: &ComponentRegistry) -> Entity {
    let e = sprite(sim, 8);
    edit(sim, e, reg, AnimFieldEdit::AddPlayer);
    for (nome, de, ate) in [("idle", 0, 3), ("run", 4, 7)] {
        edit(sim, e, reg, AnimFieldEdit::Add);
        let i = (info(sim, e).rows.len() - 1) as u8;
        edit(sim, e, reg, AnimFieldEdit::Rename(i, nome.into()));
        edit(sim, e, reg, AnimFieldEdit::Frames(i, celulas(de..=ate)));
    }
    let mut a = sim.world().get::<SpriteAnimator>(e).unwrap().clone();
    a.current = "idle".into();
    a.autoplay = true;
    sim.world_mut().entity_mut(e).insert(a);
    e
}

fn tocando(sim: &SimWorld, e: Entity) -> (String, bool, u32) {
    let a = sim.world().get::<SpriteAnimator>(e).unwrap();
    let g = sim.world().get::<ph2d_ecs::SpriteGrid>(e).unwrap();
    (a.current.clone(), a.playing, g.frame)
}

/// ⭐ **O ▶ é prévia** (ficha 02, decisão 2): num projeto que nunca escolheu, o primeiro ▶ grava antes o `current` antigo
/// como início, e o jogo começa nele, na 1.ª célula dele, por cima do que o ▶ deixou no ecrã.
/// *Mutação: tirar o `vai_para_o_inicio` do `start_autoplay_animations` ⇒ o jogo começa em `run`.*
#[test]
fn o_play_da_prevista_nao_muda_com_qual_o_jogo_comeca() {
    let mut sim = SimWorld::new();
    let reg = registry();
    let e = auto(&mut sim, &reg);
    edit(&mut sim, e, &reg, AnimFieldEdit::SetCurrent("run".into()));
    edit(&mut sim, e, &reg, AnimFieldEdit::Playing(true));
    assert_eq!(tocando(&sim, e), ("run".into(), true, 4), "a prévia toca");
    assert_eq!(info(&sim, e).card.starts_with, "idle");
    crate::tick::start_autoplay_animations(&mut sim);
    assert_eq!(tocando(&sim, e), ("idle".into(), true, 0));
}

/// Escolher no menu vale, e o `current` não se mexe até o jogo começar.
/// *Mutação: o `StartsWith` escrever também o `current` ⇒ a 1.ª afirmação cai.*
#[test]
fn escolher_o_inicio_vale_no_jogo_e_nao_mexe_na_previa() {
    let mut sim = SimWorld::new();
    let reg = registry();
    let e = auto(&mut sim, &reg);
    edit(&mut sim, e, &reg, AnimFieldEdit::StartsWith("run".into()));
    assert_eq!(tocando(&sim, e).0, "idle");
    assert_eq!(info(&sim, e).card.starts_with, "run");
    crate::tick::start_autoplay_animations(&mut sim);
    assert_eq!(tocando(&sim, e), ("run".into(), true, 4));
}

/// Renomear segue; apagar cai na primeira; um nome que a lista não tem não se aceita.
/// *Mutação: tirar o braço do renomear ⇒ o início fica com o nome velho e o menu cai no `current`.*
#[test]
fn o_inicio_segue_o_renomear_e_cai_na_primeira_ao_apagar() {
    let mut sim = SimWorld::new();
    let reg = registry();
    let e = auto(&mut sim, &reg);
    edit(&mut sim, e, &reg, AnimFieldEdit::StartsWith("run".into()));
    edit(&mut sim, e, &reg, AnimFieldEdit::Rename(1, "corre".into()));
    assert_eq!(
        sim.world()
            .get::<SpriteAnimator>(e)
            .unwrap()
            .start
            .clone()
            .unwrap(),
        "corre"
    );
    edit(
        &mut sim,
        e,
        &reg,
        AnimFieldEdit::StartsWith("nao existe".into()),
    );
    assert_eq!(
        sim.world()
            .get::<SpriteAnimator>(e)
            .unwrap()
            .start
            .clone()
            .unwrap(),
        "corre"
    );
    edit(&mut sim, e, &reg, AnimFieldEdit::Remove(1));
    assert_eq!(
        sim.world()
            .get::<SpriteAnimator>(e)
            .unwrap()
            .start
            .clone()
            .unwrap(),
        "idle"
    );
}

/// Projeto antigo (sem o componente) começa no `current`; Script não arranca e Rules não muda de animação.
/// *Mutação: tirar a pergunta `regras` ⇒ o objecto com regras salta para o início.*
#[test]
fn projeto_antigo_script_e_regras_ficam_como_estao() {
    let mut sim = SimWorld::new();
    let reg = registry();
    let e = auto(&mut sim, &reg);
    let mut a = sim.world().get::<SpriteAnimator>(e).unwrap().clone();
    a.current = "run".into();
    sim.world_mut().entity_mut(e).insert(a);
    crate::tick::start_autoplay_animations(&mut sim);
    assert_eq!(
        tocando(&sim, e).0,
        "run",
        "sem início: o `current` de sempre"
    );

    let e = auto(&mut sim, &reg);
    edit(&mut sim, e, &reg, AnimFieldEdit::StartsWith("run".into()));
    edit(&mut sim, e, &reg, AnimFieldEdit::Autoplay(false));
    crate::tick::start_autoplay_animations(&mut sim);
    assert_eq!(tocando(&sim, e), ("idle".into(), false, 0), "Script");

    let e = auto(&mut sim, &reg);
    edit(&mut sim, e, &reg, AnimFieldEdit::StartsWith("run".into()));
    sim.world_mut()
        .entity_mut(e)
        .insert(ph2d_physics_ecs::AnimRules::default());
    crate::tick::start_autoplay_animations(&mut sim);
    assert_eq!(tocando(&sim, e).0, "idle", "Rules");
}

/// **Recuperar um engasgo numa chamada dá o MESMO que recuperar passo a passo.**
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

/// **O MOTOR e o PAINEL dão a mesma resposta a «esta reprodução está pendurada?»**
/// ⚠️ A lei existe **duas vezes**, e de propósito: o painel é chrome e não vê o motor
/// ([`ph2d_ecs::animator_state`] vive no `ph2d-ecs`; [`InspectorAnimInfo::current_dangling`] no
/// modelo do editor).
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
        if let Some(mut g) = sim.world_mut().get_mut::<ph2d_ecs::SpriteGrid>(e) {
            g.hframes = cells;
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

// ── Os marcadores em quadro (SFX Lab W1). O resto do caminho: `shells/game-scenes/tests/it/sprite_marcadores.rs` ──

mod marcadores {
    use super::*;

    #[test]
    fn removing_the_last_marker_removes_the_component() {
        let (mut sim, reg) = (SimWorld::default(), registry());
        let e = sprite(&mut sim, 4);
        assert!(edit(&mut sim, e, &reg, AnimFieldEdit::Add).is_none());
        edit(&mut sim, e, &reg, AnimFieldEdit::MarkerAdd(0));
        assert!(sim.world().get::<ph2d_ecs::SpriteAnimMarkers>(e).is_some());
        edit(&mut sim, e, &reg, AnimFieldEdit::MarkerRemove(0, 0));
        assert!(sim.world().get::<ph2d_ecs::SpriteAnimMarkers>(e).is_none());
    }
}
