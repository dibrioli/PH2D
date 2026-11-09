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
    let list = FrameRef::run(0, list.iter().copied());
    let mut a = playing(t);
    let (mut frame, mut step) = (FrameRef::cell(99), 0);
    (0..n)
        .map(|_| {
            advance_frames(&mut a, t, Some(&list), &mut frame, &mut step, &[8], TICK);
            (frame.cell_on(0).unwrap_or(u32::MAX), a.playing)
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
    let (mut fa, mut fb, mut step) = (99, FrameRef::cell(99), 0);
    for _ in 0..40 {
        crate::advance(&mut a, &t, &mut fa, 8, TICK);
        advance_frames(&mut b, &t, None, &mut fb, &mut step, &[8], TICK);
        assert_eq!((FrameRef::cell(fa), &a), (fb, &b));
        assert_eq!(step, fa - 2);
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
    let (mut frame, mut step) = (FrameRef::cell(0), 0);
    let fora = FrameRef::run(0, [1, 9]);
    let out = advance_frames(
        &mut a,
        &t,
        Some(&fora),
        &mut frame,
        &mut step,
        &[8],
        1_000_000,
    );
    assert_eq!((frame, out), (FrameRef::cell(0), AnimOutcome::default()));
    assert_eq!(tag_cells(&t, Some(&fora), &[8]), None);
    let dentro = FrameRef::run(0, [1, 7]);
    assert_eq!(tag_cells(&t, Some(&dentro), &[8]), Some(dentro.clone()));
    assert_eq!(
        tag_cells(&tag(2, 5, AnimDirection::Forward, None), None, &[4]),
        Some(FrameRef::run(0, [2, 3]))
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
    frames.set("run", FrameRef::run(0, [3, 2, 1]));
    let mut anim = SpriteAnimator::new("run");
    assert!(rename_tag(
        &mut lib,
        Some(&mut frames),
        Some(&mut anim),
        "run",
        "corre"
    ));
    assert!(lib.get("corre").is_some() && lib.get("run").is_none());
    assert_eq!(frames.get("corre"), Some(&FrameRef::run(0, [3, 2, 1])[..]));
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

/// ⭐ Uma lista VAZIA é uma animação sem quadros (o «+», smoke do dono 09/11), não o intervalo: não toca, a imagem
/// fica, nada entra em pânico; só o `remove` a devolve ao intervalo. *Mutação: `set` com vazio apagar a entrada ⇒
/// `get` dá `None` e a tag toca a folha inteira.*
#[test]
fn uma_animacao_sem_quadros_nao_toca_e_segura_a_imagem() {
    let mut f = SpriteAnimFrames::default();
    f.set("walk", vec![FrameRef::cell(1)]);
    f.set("walk", vec![]);
    assert_eq!(f.get("walk"), Some(&[][..]), "a entrada vazia fica");
    let t = tag(0, 7, AnimDirection::PingPong, None);
    let list = f.get("walk");
    assert_eq!(tag_cells(&t, list, &[8]), None);
    let mut a = playing(&t);
    assert_eq!(entry_cell(&a, &t, list, &[8]), None);
    let (mut frame, mut step) = (FrameRef::cell(5), 2);
    for _ in 0..20 {
        let out = advance_frames(&mut a, &t, list, &mut frame, &mut step, &[8], TICK);
        assert!(!out.frame_changed);
    }
    assert_eq!(
        (frame, step),
        (FrameRef::cell(5), 2),
        "a imagem fica a que estava"
    );
    f.remove("walk");
    assert!(f.is_empty());
    assert_eq!(tag_cells(&t, f.get("walk"), &[8]).map(|s| s.len()), Some(8));
}

#[test]
fn um_tempo_igual_ao_ritmo_nao_e_tempo_proprio() {
    let mut t = AnimationTag::new("idle", 0, 5);
    t.frame_ms = 167;
    t.per_frame_ms = vec![167; 6];
    assert!(
        !t.has_per_frame_timing(),
        "o herói de fábrica escreve o ritmo por extenso"
    );
    t.per_frame_ms[2] = 300;
    assert!(t.has_per_frame_timing());
}

// ── Várias folhas (plano GEF 26) ──

/// O que se vê, quadro a quadro, de uma lista sobre as folhas `sheets`.
fn film_refs(t: &AnimationTag, list: &[FrameRef], sheets: &[u32], n: usize) -> Vec<FrameRef> {
    let mut a = playing(t);
    let (mut frame, mut step) = (FrameRef::cell(99), 0);
    (0..n)
        .map(|_| {
            advance_frames(&mut a, t, Some(list), &mut frame, &mut step, sheets, TICK);
            frame
        })
        .collect()
}

#[test]
fn uma_animacao_mistura_duas_folhas_na_ordem_da_tira() {
    let t = tag(0, 0, AnimDirection::Forward, None);
    let idle = |c| FrameRef::Cell { sheet: 0, cell: c };
    let run = |c| FrameRef::Cell { sheet: 1, cell: c };
    let list = [idle(0), idle(1), run(5), idle(2), run(0)];
    let seen = film_refs(&t, &list, &[4, 12], 14);
    // 30 ms por quadro sobre tiques de 10 ms: três tiques cada (o 1.º já conta um), e a folha viaja com a célula.
    let esperado: Vec<FrameRef> = list.iter().flat_map(|f| [*f; 3]).skip(1).collect();
    assert_eq!(seen, esperado);
}

#[test]
fn um_quadro_fora_da_folha_dele_cala_a_lista_mas_a_outra_folha_cabe() {
    let t = tag(0, 0, AnimDirection::Forward, None);
    let fora = [FrameRef::cell(1), FrameRef::Cell { sheet: 1, cell: 12 }];
    assert_eq!(
        tag_cells(&t, Some(&fora), &[4, 12]),
        None,
        "célula 12 de 12"
    );
    let sem_folha = [FrameRef::Cell { sheet: 2, cell: 0 }];
    assert_eq!(
        tag_cells(&t, Some(&sem_folha), &[4, 12]),
        None,
        "folha 2 não existe"
    );
    let dentro = [FrameRef::cell(3), FrameRef::Cell { sheet: 1, cell: 11 }];
    assert_eq!(
        tag_cells(&t, Some(&dentro), &[4, 12]),
        Some(dentro.to_vec())
    );
}

#[test]
fn o_vazio_a_tocar_dura_o_seu_tempo() {
    let t = tag(0, 0, AnimDirection::Forward, None);
    let list = [FrameRef::cell(0), FrameRef::Empty, FrameRef::cell(1)];
    let seen = film_refs(&t, &list, &[4], 8);
    assert_eq!(
        seen,
        [
            FrameRef::cell(0),
            FrameRef::cell(0),
            FrameRef::Empty,
            FrameRef::Empty,
            FrameRef::Empty,
            FrameRef::cell(1),
            FrameRef::cell(1),
            FrameRef::cell(1),
        ],
        "o buraco não se salta: o ritmo da animação fica"
    );
}

#[test]
fn o_intervalo_de_outra_folha_entra_pela_ponta_e_toca_nela() {
    let mut t = tag(2, 4, AnimDirection::Forward, None);
    t.sheet = 1;
    let mut a = playing(&t);
    // A tela está num quadro da folha 0 (outra animação): o intervalo da folha 1 entra pelo início dele.
    let (mut frame, mut step) = (FrameRef::cell(3), 0);
    let mut seen = Vec::new();
    for _ in 0..9 {
        advance_frames(&mut a, &t, None, &mut frame, &mut step, &[4, 12], TICK);
        seen.push(frame);
    }
    let run = |c| FrameRef::Cell { sheet: 1, cell: c };
    assert_eq!(seen[0], run(2));
    assert!(
        seen.iter()
            .all(|f| matches!(f, FrameRef::Cell { sheet: 1, .. }))
    );
    assert_eq!(seen[7], run(4));
    assert_eq!(
        entry_cell(&playing(&t), &t, None, &[4, 12]),
        Some((run(2), 0))
    );
}
