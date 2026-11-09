//! A porta única da tag (P14 §4): o Pixel Sprite e o `.ase` importado dão os mesmos clips.

use ph2d_ecs::{AnimDirection, SpriteAnimations, tag_do_aseprite};
use ph2d_pixel_cena::fixture::{DURACOES_MS, heroi_p14};
use ph2d_pixel_cena::{clips_das_tags, registar};
use ph2d_pixel_core::{FrameSet, Op, apply};

/// O que o import do Sprite Inspector faz de um `.ase`: lê-o com `ph2d_aseprite::parse` e passa
/// cada tag pela porta (o `tag_from_ase` do shell é este laço mais a nota i18n).
fn clips_do_ase(bytes: &[u8]) -> SpriteAnimations {
    let doc = ph2d_aseprite::parse(bytes).expect("o .ase do Pixel lê-se");
    let dur: Vec<u32> = doc
        .frames
        .iter()
        .map(|f| u32::from(f.duration_ms))
        .collect();
    let mut lib = SpriteAnimations::new();
    for t in &doc.tags {
        lib.insert(tag_do_aseprite(
            &t.name,
            u32::from(t.from),
            u32::from(t.to),
            t.direction,
            t.repeat,
            &dur,
        ))
        .expect("entra");
    }
    lib
}

#[test]
fn a_tag_do_pixel_e_a_do_aseprite_pela_mesma_porta() {
    let s = heroi_p14();
    let (bytes, _) = ph2d_pixel_io::ase::escrever(&s).expect("grava .ase");
    let pixel = clips_das_tags(&s, "heroi");
    let ase = clips_do_ase(&bytes);
    let p: Vec<_> = pixel.iter().collect();
    let a: Vec<_> = ase.iter().collect();
    assert_eq!(p.len(), 3);
    for (p, a) in p.iter().zip(&a) {
        assert_eq!(p, a, "a tag {} difere entre o Pixel e o .ase", p.name);
    }
    assert_eq!(pixel, ase);
    // O fenómeno está na fixture: uniforme, por quadro, PingPong com repeat, Reverse.
    let run = pixel.get("run").expect("run");
    assert_eq!(
        (run.from, run.to, run.direction, run.repeat),
        (2, 4, AnimDirection::PingPong, Some(1)),
        "2 passadas do Aseprite = 1 ciclo do motor"
    );
    assert_eq!(
        (run.frame_ms, run.per_frame_ms.as_slice()),
        (120, &[120, 120, 60][..])
    );
    let idle = pixel.get("idle").expect("idle");
    assert_eq!(
        (idle.frame_ms, idle.per_frame_ms.len(), idle.repeat),
        (80, 0, None)
    );
    let hit = pixel.get("hit").expect("hit");
    assert_eq!((hit.direction, hit.frame_ms), (AnimDirection::Reverse, 200));
}

#[test]
fn sem_tags_nasce_uma_que_cobre_tudo() {
    let mut s = heroi_p14();
    let frames = s.frames().to_vec();
    let slices = s.slices().to_vec();
    apply(
        &mut s,
        Op::Frames(Box::new(FrameSet {
            frames,
            cels: Vec::new(),
            tags: Vec::new(),
            slices,
        })),
    )
    .expect("sem tags");
    let lib = clips_das_tags(&s, "heroi");
    let t: Vec<_> = lib.iter().collect();
    assert_eq!(t.len(), 1, "sem tags nasce UMA");
    assert_eq!(t[0].name, "heroi");
    assert_eq!((t[0].from, t[0].to), (0, 5), "cobre TODOS os quadros");
    assert_eq!(
        t[0].per_frame_ms,
        DURACOES_MS.to_vec(),
        "o ritmo passa inteiro"
    );
    assert_eq!(t[0].frame_ms, 80, "empate 80×120 ⇒ a menor");
}

#[test]
fn os_clips_do_pixel_sprite_nao_se_gravam() {
    use ph2d_ecs::scene::registry::{ComponentRegistry, register_ecs_components};
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    registar(&mut reg);
    let clips = clips_das_tags(&heroi_p14(), "heroi");
    let mut w = ph2d_ecs::World::new();
    let pixel = w.spawn((ph2d_ecs::PixelSprite(1), clips.clone())).id();
    let comum = w.spawn(clips).id();
    let e = reg
        .get_by_name("ph2d::ecs::SpriteAnimations")
        .expect("registado");
    assert!(
        matches!((e.grava)(&w, pixel), Ok(None)),
        "no Pixel Sprite é derivado"
    );
    assert!(
        matches!((e.grava)(&w, comum), Ok(Some(_))),
        "num sprite comum grava-se"
    );
}
