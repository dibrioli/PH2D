use super::*;
use crate::{AnimDirection, AnimationTag, SpriteAnimations, SpriteAnimator};

const TICK: u64 = 10_000; // 10 ms

fn tag(from: u32, to: u32, dir: AnimDirection, repeat: Option<u32>) -> AnimationTag {
    let mut t = AnimationTag::new("walk", from, to);
    t.frame_ms = 30;
    t.direction = dir;
    t.repeat = repeat;
    t
}

fn playing(t: &AnimationTag) -> SpriteAnimator {
    let mut a = SpriteAnimator::new(t.name.clone());
    a.playing = true;
    a.rewind(Some(t));
    a
}

/// O que se vê ao longo de `n` tiques: a célula e se ainda toca.
fn film_range(t: &AnimationTag, n: usize) -> Vec<(u32, bool)> {
    let mut a = playing(t);
    let mut frame = 99;
    (0..n)
        .map(|_| {
            crate::advance(&mut a, t, &mut frame, 8, TICK);
            (frame, a.playing)
        })
        .collect()
}

fn film_list(t: &AnimationTag, list: &[u32], n: usize) -> Vec<(u32, bool)> {
    let mut a = playing(t);
    let (mut frame, mut step) = (99, 0);
    (0..n)
        .map(|_| {
            advance_frames(&mut a, t, Some(list), &mut frame, &mut step, 8, TICK);
            (frame, a.playing)
        })
        .collect()
}

#[test]
fn uma_lista_igual_ao_intervalo_toca_igual_em_toda_direcao_e_laco() {
    for dir in AnimDirection::ALL {
        for repeat in [None, Some(1), Some(3)] {
            let t = tag(2, 5, dir, repeat);
            assert_eq!(
                film_list(&t, &[2, 3, 4, 5], 80),
                film_range(&t, 80),
                "{dir:?} {repeat:?}"
            );
        }
    }
}

#[test]
fn sem_lista_e_o_intervalo_de_sempre_e_o_passo_acompanha() {
    let t = tag(2, 5, AnimDirection::Forward, None);
    let mut a = playing(&t);
    let mut b = a.clone();
    let (mut fa, mut fb, mut step) = (99, 99, 0);
    for _ in 0..40 {
        crate::advance(&mut a, &t, &mut fa, 8, TICK);
        advance_frames(&mut b, &t, None, &mut fb, &mut step, 8, TICK);
        assert_eq!((fa, &a), (fb, &b));
        assert_eq!(step, fb - 2);
    }
}

#[test]
fn a_lista_toca_na_ordem_clicada_com_repeticoes() {
    let t = tag(0, 0, AnimDirection::Forward, None);
    let seen: Vec<u32> = film_list(&t, &[1, 6, 1, 3], 12)
        .into_iter()
        .map(|(f, _)| f)
        .collect();
    // 30 ms por quadro sobre tiques de 10 ms: cada célula fica três tiques.
    assert_eq!(seen, [1, 1, 6, 6, 6, 1, 1, 1, 3, 3, 3, 1]);
}

#[test]
fn uma_celula_fora_da_grelha_cala_a_lista() {
    let t = tag(0, 0, AnimDirection::Forward, None);
    let mut a = playing(&t);
    let (mut frame, mut step) = (0, 0);
    let out = advance_frames(
        &mut a,
        &t,
        Some(&[1, 9]),
        &mut frame,
        &mut step,
        8,
        1_000_000,
    );
    assert_eq!((frame, out), (0, AnimOutcome::default()));
    assert_eq!(tag_cells(&t, Some(&[1, 9]), 8), None);
    assert_eq!(tag_cells(&t, Some(&[1, 7]), 8), Some(vec![1, 7]));
    assert_eq!(
        tag_cells(&tag(2, 5, AnimDirection::Forward, None), None, 4),
        Some(vec![2, 3])
    );
}

#[test]
fn quadros_por_segundo_e_ms_vao_e_voltam() {
    for ms in crate::FRAME_MS_MIN..=crate::FRAME_MS_MAX {
        assert_eq!(ms_of_fps(fps_of_ms(ms)), ms, "{ms} ms");
    }
    assert_eq!(ms_of_fps(12.0), 83);
    assert_eq!(ms_of_fps(0.0), crate::FRAME_MS_MAX);
    assert_eq!(ms_of_fps(f32::NAN), crate::FRAME_MS_MAX);
    assert_eq!(ms_of_fps(1e9), crate::FRAME_MS_MIN);
}

#[test]
fn renomear_leva_a_lista_e_o_tocador() {
    let mut lib = SpriteAnimations::new();
    lib.insert(AnimationTag::new("run", 0, 3)).unwrap();
    lib.insert(AnimationTag::new("idle", 4, 5)).unwrap();
    let mut frames = SpriteAnimFrames::default();
    frames.set("run", vec![3, 2, 1]);
    let mut anim = SpriteAnimator::new("run");
    assert!(rename_tag(
        &mut lib,
        Some(&mut frames),
        Some(&mut anim),
        "run",
        "corre"
    ));
    assert!(lib.get("corre").is_some() && lib.get("run").is_none());
    assert_eq!(frames.get("corre"), Some(&[3, 2, 1][..]));
    assert_eq!(anim.current, "corre");
    assert!(
        !rename_tag(&mut lib, None, None, "corre", "idle"),
        "nome ocupado"
    );
    assert!(!rename_tag(&mut lib, None, None, "corre", ""), "nome vazio");
    assert!(
        !rename_tag(&mut lib, None, None, "nada", "x"),
        "tag inexistente"
    );
}

#[test]
fn por_uma_lista_vazia_apaga_a_entrada() {
    let mut f = SpriteAnimFrames::default();
    f.set("a", vec![1]);
    f.set("a", vec![]);
    assert!(f.is_empty());
}
