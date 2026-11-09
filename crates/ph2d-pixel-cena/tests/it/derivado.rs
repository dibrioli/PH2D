//! P14a-2: o derivado volta depois do respawn, não entra no undo nem no ficheiro, e o jogo toca o
//! mesmo quadro que o editor.

use ph2d_ecs::scene::incremental::{CaptureCache, capture_incremental};
use ph2d_ecs::scene::registry::{ComponentRegistry, register_ecs_components};
use ph2d_ecs::scene::save::WorldSnapshot;
use ph2d_ecs::scene::{snapshot_to_world, world_to_snapshot};
use ph2d_ecs::{
    Entity, FrameRef, Name, PixelSprite, SimWorld, SpriteAnimations, SpriteAnimator, SpriteGrid,
    Transform, TransformPropagationState, WorklistBuf, World,
};
use ph2d_pixel_cena::fixture::heroi_p14;
use ph2d_pixel_cena::{CLIP_SEM_TAGS, animador_de_nascenca, clips_das_tags, deriva, registar};
use ph2d_pixel_core::play::{Play, PlayOptions};
use ph2d_pixel_core::{FrameSet, Op, Sprite, apply};

fn reg() -> ComponentRegistry {
    let mut r = ComponentRegistry::new();
    register_ecs_components(&mut r);
    registar(&mut r);
    r
}

fn pixel_de(w: &mut World) -> Entity {
    let mut q = w.query_filtered::<Entity, bevy_ecs::query::With<PixelSprite>>();
    q.single(w).expect("um Pixel Sprite")
}

/// O mundo do jogo: um Pixel Sprite com o animador gravado, como nasce no `object_add`.
fn mundo(s: &Sprite) -> SimWorld {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Transform::IDENTITY,
        Name::new("Heroi"),
        PixelSprite(1),
        animador_de_nascenca(),
    ));
    sim
}

/// O snapshot pelo caminho gravado (o do ficheiro e da foto do Play).
fn foto(sim: &mut SimWorld, r: &ComponentRegistry) -> WorldSnapshot {
    let mut prop = TransformPropagationState::new(sim.world_mut());
    let mut wl = WorklistBuf::default();
    let mut snap = WorldSnapshot::default();
    world_to_snapshot(sim.world_mut(), &mut prop, &mut wl, r, &mut snap).expect("foto");
    snap
}

fn com_tag_run_renomeada(s: &mut Sprite) {
    let mut tags = s.tags().to_vec();
    tags[1].name = "corre".into();
    let (frames, slices) = (s.frames().to_vec(), s.slices().to_vec());
    apply(
        s,
        Op::Frames(Box::new(FrameSet {
            frames,
            cels: Vec::new(),
            tags,
            slices,
        })),
    )
    .expect("tags");
}

#[test]
fn os_clips_voltam_depois_do_undo_e_do_stop() {
    let r = reg();
    let s = heroi_p14();
    let mut sim = mundo(&s);
    deriva(sim.world_mut(), |_| Some((&s, 1)));
    let e = pixel_de(sim.world_mut());
    sim.world_mut()
        .get_mut::<SpriteGrid>(e)
        .expect("grelha")
        .frame = 3;
    let snap = foto(&mut sim, &r);

    // O Stop e o undo: despawn de tudo e respawn do snapshot (`undo.rs` `restore`).
    let todos: Vec<Entity> = {
        let w = sim.world_mut();
        let mut q = w.query_filtered::<Entity, bevy_ecs::query::With<Transform>>();
        q.iter(w).collect()
    };
    for e in todos {
        sim.world_mut().despawn(e);
    }
    snapshot_to_world(sim.world_mut(), &snap, &r).expect("respawn");
    let e = pixel_de(sim.world_mut());
    assert!(
        sim.world().get::<SpriteAnimations>(e).is_none(),
        "o derivado não pode voltar pelo snapshot"
    );
    assert!(
        sim.world().get::<SpriteAnimator>(e).is_some(),
        "o animador é gravado"
    );

    deriva(sim.world_mut(), |_| Some((&s, 1)));
    assert_eq!(
        sim.world().get::<SpriteAnimations>(e),
        Some(&clips_das_tags(&s, CLIP_SEM_TAGS)),
        "os clips voltam iguais às tags"
    );
    assert_eq!(
        sim.world().get::<SpriteGrid>(e).copied(),
        Some(SpriteGrid::new(6, 1)),
        "a grelha volta com um quadro por coluna"
    );
}

#[test]
fn os_clips_derivados_nao_entram_no_undo_nem_no_ficheiro() {
    let r = reg();
    let mut s = heroi_p14();
    let mut sim = mundo(&s);
    deriva(sim.world_mut(), |_| Some((&s, 1)));
    let e = pixel_de(sim.world_mut());
    let mut cache = CaptureCache::new();
    let mut out = WorldSnapshot::default();
    capture_incremental(sim.world_mut(), &mut cache, &r, &mut out).expect("1.ª");

    // Editar uma tag no Pixel (stamp novo) reescreve o derivado…
    com_tag_run_renomeada(&mut s);
    deriva(sim.world_mut(), |_| Some((&s, 2)));
    assert!(
        sim.world()
            .get::<SpriteAnimations>(e)
            .is_some_and(|l| l.get("corre").is_some()),
        "o stamp novo reescreveu os clips"
    );
    // … e nada disso é um passo de undo do mundo.
    let rep = capture_incremental(sim.world_mut(), &mut cache, &r, &mut out).expect("2.ª");
    assert_eq!(rep.reserialized, 0, "o derivado marcou diferença no undo");
    assert!(!rep.changed());

    // O ficheiro: sem clips nem grelha; o animador vai.
    let snap = foto(&mut sim, &r);
    let id = |n: &str| r.get_by_name(n).expect(n).type_id;
    let tipos: Vec<_> = snap
        .entities
        .iter()
        .flat_map(|row| row.components.iter().map(|c| c.type_id))
        .collect();
    assert!(!tipos.contains(&id("ph2d::ecs::SpriteAnimations")));
    assert!(!tipos.contains(&id("ph2d::ecs::SpriteGrid")));
    assert!(tipos.contains(&id("ph2d::ecs::SpriteAnimator")));

    // Controlo: mudar o animador (escolha do artista) É um passo.
    sim.world_mut()
        .get_mut::<SpriteAnimator>(e)
        .expect("animador")
        .autoplay = false;
    let rep = capture_incremental(sim.world_mut(), &mut cache, &r, &mut out).expect("3.ª");
    assert_eq!(rep.reserialized, 1, "o controlo: o animador grava-se");
}

/// O tique de `ph2d_app_sprite_anim::tick` sem os enablers e a pré-visualização (a crate puxa render
/// e editor; a lei é a mesma `advance_frames_marked`).
fn tique(w: &mut World, e: Entity, dt_us: u64) {
    let tags = w.get::<SpriteAnimations>(e).expect("clips").clone();
    let mut grid = *w.get::<SpriteGrid>(e).expect("grelha");
    let mut anim = w.get::<SpriteAnimator>(e).expect("animador").clone();
    if !anim.playing {
        return;
    }
    let tag = tags.get(&anim.current).expect("tag");
    let cells = ph2d_ecs::cells_per_sheet(Some(&grid), None);
    let mut frame = grid.shown();
    let mut step = 0;
    ph2d_ecs::advance_frames_marked(
        &mut anim,
        tag,
        None,
        &mut frame,
        &mut step,
        &cells,
        dt_us,
        None,
        &mut Vec::new(),
    );
    grid.show(frame);
    w.entity_mut(e).insert((grid, anim));
}

/// O jogo e o editor tocam `tag` desde o início: o MESMO quadro em cada ms enquanto a tag toca no
/// editor; depois dela o editor segue a linha do tempo (lei do Aseprite) e o jogo fica na pose final
/// (lei do motor), que é o último quadro que o editor mostrou dentro da tag.
fn confere(s: &Sprite, tag: &str) {
    let mut sim = mundo(s);
    deriva(sim.world_mut(), |_| Some((s, 1)));
    let e = pixel_de(sim.world_mut());
    let clip = sim
        .world()
        .get::<SpriteAnimations>(e)
        .and_then(|l| l.get(tag))
        .cloned()
        .expect("clip");
    let mut a = SpriteAnimator::new(tag);
    a.playing = true;
    a.rewind(Some(&clip));
    sim.world_mut().entity_mut(e).insert(a);
    let mut editor = Play::start(s, clip.from as usize, PlayOptions::default());
    let celula = |w: &World| match w.get::<SpriteGrid>(e).expect("grelha").shown() {
        FrameRef::Cell { cell, .. } => cell as usize,
        FrameRef::Empty => usize::MAX,
    };
    let dentro = |i: usize| (clip.from as usize..=clip.to as usize).contains(&i);
    tique(sim.world_mut(), e, 0);
    let (mut pose, mut saiu, mut diverge) = (editor.index, None, Vec::new());
    for t in 0..=2000u32 {
        if t > 0 {
            editor.advance(s, 1);
            tique(sim.world_mut(), e, 1000);
        }
        let (jogo, ed) = (celula(sim.world()), editor.index);
        if saiu.is_none() && !dentro(ed) {
            saiu = Some(t);
        }
        let quer = if saiu.is_some() { pose } else { ed };
        pose = if saiu.is_some() { pose } else { ed };
        if jogo != quer && diverge.len() < 8 {
            diverge.push((t, jogo, quer));
        }
    }
    assert!(saiu.is_some(), "{tag}: a tag do editor não acabou em 2 s");
    assert!(
        diverge.is_empty(),
        "{tag} (t ms, quadro do jogo, o que devia): {diverge:?}"
    );
    let a = sim.world().get::<SpriteAnimator>(e).expect("animador");
    assert!(!a.playing, "{tag}: o jogo parou na pose final");
}

#[test]
fn o_jogo_e_o_editor_tocam_o_mesmo_quadro() {
    let mut s = heroi_p14();
    confere(&s, "run"); // PingPong, 2 passadas: 2 3 4 3 2
    let mut tags = s.tags().to_vec();
    tags[1].repeat = 3; // ímpar: 2 3 4 3 2 3 4, pára em 4
    let (frames, slices) = (s.frames().to_vec(), s.slices().to_vec());
    apply(
        &mut s,
        Op::Frames(Box::new(FrameSet {
            frames,
            cels: Vec::new(),
            tags,
            slices,
        })),
    )
    .expect("tags");
    confere(&s, "run");
}

/// P14d: o quadro que o JOGO mostra em cada ms de `t = 0..=2000` a tocar `tag` desde o início — o
/// lado nosso de `a_sequencia_e_a_do_godot`.
pub(crate) fn sequencia_do_jogo(s: &Sprite, tag: &str) -> Vec<usize> {
    let mut sim = mundo(s);
    deriva(sim.world_mut(), |_| Some((s, 1)));
    let e = pixel_de(sim.world_mut());
    let clip = sim
        .world()
        .get::<SpriteAnimations>(e)
        .and_then(|l| l.get(tag))
        .cloned()
        .expect("clip");
    let mut a = SpriteAnimator::new(tag);
    a.playing = true;
    a.rewind(Some(&clip));
    sim.world_mut().entity_mut(e).insert(a);
    tique(sim.world_mut(), e, 0);
    (0..=2000u32)
        .map(|t| {
            if t > 0 {
                tique(sim.world_mut(), e, 1000);
            }
            match sim.world().get::<SpriteGrid>(e).expect("grelha").shown() {
                FrameRef::Cell { cell, .. } => cell as usize,
                FrameRef::Empty => usize::MAX,
            }
        })
        .collect()
}
