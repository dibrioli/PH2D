//! Gates da **janela de PREVIEW/RETUNE** do Offset (Corner/Side pós-release) — módulo
//! FILHO de `vec_expand_tests.rs` pelo teto de 600 LOC (HR-18): os fixtures
//! (`posed_scene`/`drag_frame`/`square`…) são do pai, e uma cópia divergiria.

use super::*;

/// **Clicar Round DEPOIS de soltar o slider re-offseta na hora — sem mover a forma.** O
/// pedido do Enio (2026-07-20): a quina se escolhe VENDO o resultado. A janela re-roda o
/// preview ao `d` comitado; o oráculo do lugar é o mesmo do arrasto (bbox × xform vivo), e
/// o de FORMA é a área: Round come as quatro quinas do Miter (`(4−π)·d²`).
///
/// ⚠️ O reset de identidade dentro do `apply` é load-bearing: o release ASSENTOU as
/// entidades do resultado (`Transform` = centro), e o preview re-insere geometria de MUNDO
/// sob os MESMOS ids — sem o reset, o retune desenharia a forma com a pose DOBRADA (a
/// doença exata do "pula pro canto direito", `9c0446df`).
#[test]
fn changing_the_join_after_release_retunes_the_committed_offset() {
    let (mut scene, mut pen, mut sim, mut map, xf) = posed_scene();
    // O estado dos chips é thread-local — pina Miter/Both e restaura no fim (outros
    // gates deste arquivo medem áreas computadas para Miter).
    ph2d_panel_vector::set_expand_join(0);
    ph2d_panel_vector::set_expand_side(2);

    let mut sess = OffsetSession::begin(&scene, &pen, &xf).expect("há seleção");
    drag_frame(&mut sess, &mut scene, &mut pen, &mut sim, &mut map, 0.5);
    // O release: preview final; a sessão morre e o settle ASSENTA o resultado.
    sess.preview(&mut scene, &mut pen, 0.5);
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    crate::vec_transform::settle_origins(&mut sim, &mut scene, &map, &[]);
    let miter_area = ph2d_vec_boolean::area(&scene.paths()[0]);

    let mut win = OffsetRetune::after_release(sess, 0.5).expect("churnou — há janela");
    assert_eq!(
        win.step(7, expand_knobs()),
        RetuneStep::Keep,
        "frame 1 aprende"
    );

    // O artista clica Round:
    ph2d_panel_vector::set_expand_join(1);
    let knobs = expand_knobs();
    assert_eq!(
        win.step(7, knobs),
        RetuneStep::Retune,
        "knob mudou = retune"
    );
    win.apply(&mut scene, &mut pen, &mut sim, &map, knobs);
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    crate::vec_transform::settle_origins(&mut sim, &mut scene, &map, &[]);

    let round_area = ph2d_vec_boolean::area(&scene.paths()[0]);
    assert!(
        round_area < miter_area - 0.05,
        "a quina não mudou (miter {miter_area}, round {round_area})"
    );
    let live = crate::vec_transform::build(&sim, &map);
    let pid = scene.paths()[0].id;
    let (lo, hi) = scene.path_bbox(pid).expect("bbox");
    let c = xform_of(&live, pid).apply([(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5]);
    assert!(
        (c[0] - 4.0).abs() < 1e-3 && c[1].abs() < 1e-3,
        "o retune moveu a forma (pose dobrada?): {c:?}"
    );
    ph2d_panel_vector::set_expand_join(0);
}

/// **Uma CADEIA de retunes muda a forma em CADA passo** (Round → Bevel → Miter), não só no
/// primeiro. O memo do preview (`OffsetSession::last`) pula frames de `(d, knobs)` iguais;
/// se o `apply` não invalidasse o memo, o 2º retune em diante casaria a chave antiga e
/// PULARIA — a forma travaria no 1º join (o report de 2026-07-20, "Round para Bevel ou
/// Miter não muda mais"). O oráculo é VERTS (Round=arcos, Bevel=chanfro, Miter=quina): os
/// três têm de ser DISTINTOS. `depth` é passado estável (a morte da janela é gateada à
/// parte); aqui prova-se só que o retune re-deriva a cada troca.
#[test]
fn a_chain_of_retunes_changes_the_shape_at_every_step() {
    let (mut scene, mut pen, mut sim, mut map, xf) = posed_scene();
    // Estado inicial dos chips: Miter/Both. Restaura no fim (outros gates deste arquivo
    // computam áreas para Miter).
    ph2d_panel_vector::set_expand_join(0);
    ph2d_panel_vector::set_expand_side(2);

    let mut sess = OffsetSession::begin(&scene, &pen, &xf).expect("há seleção");
    drag_frame(&mut sess, &mut scene, &mut pen, &mut sim, &mut map, 0.5);
    sess.preview(&mut scene, &mut pen, 0.5);
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    crate::vec_transform::settle_origins(&mut sim, &mut scene, &map, &[]);
    let mut win = OffsetRetune::after_release(sess, 0.5).expect("churnou");
    assert_eq!(
        win.step(7, expand_knobs()),
        RetuneStep::Keep,
        "frame 1 aprende"
    );

    let verts = |scene: &VecScene| -> usize { scene.paths().iter().map(|p| p.verts.len()).sum() };
    // Aplica um join e devolve os verts do resultado.
    let retune_to = |join: u8,
                     win: &mut OffsetRetune,
                     scene: &mut VecScene,
                     pen: &mut PenTool,
                     sim: &mut ph2d_ecs::SimWorld,
                     map: &mut crate::vec_entities::VecEntityMap|
     -> usize {
        // ⚠️ O frame "aprende" que o app real SEMPRE tem: o `apply` do retune anterior
        // re-armou `depth = None`, e a próxima `step` (frame ocioso, knobs ainda os
        // antigos) o re-aprende (Keep) ANTES de qualquer troca. Sem espelhar esse frame o
        // teste bate no `None => Keep` e não representa a sequência do produto.
        assert_eq!(
            win.step(7, expand_knobs()),
            RetuneStep::Keep,
            "o frame de aprender-depth pós-apply"
        );
        ph2d_panel_vector::set_expand_join(join);
        let k = expand_knobs();
        assert_eq!(
            win.step(7, k),
            RetuneStep::Retune,
            "join {join}: knob mudou = retune"
        );
        win.apply(scene, pen, sim, map, k);
        crate::vec_entities::sync(sim, scene, map);
        verts(scene)
    };

    let round = retune_to(1, &mut win, &mut scene, &mut pen, &mut sim, &mut map);
    let bevel = retune_to(2, &mut win, &mut scene, &mut pen, &mut sim, &mut map);
    let miter = retune_to(0, &mut win, &mut scene, &mut pen, &mut sim, &mut map);

    assert!(
        round > 12,
        "Round devia produzir ARCOS (muitos verts), deu {round} — o retune pulou?"
    );
    assert_ne!(round, bevel, "Round→Bevel não mudou os verts ({round})");
    assert_ne!(bevel, miter, "Bevel→Miter não mudou os verts ({bevel})");
    assert!(
        miter < bevel && bevel < round,
        "a ordem de verts esperada é Miter < Bevel < Round (deu {miter} < {bevel} < {round})"
    );
    ph2d_panel_vector::set_expand_join(0);
}

/// **A janela morre quando o undo ANDA — pra qualquer lado.** Depois do nosso passo,
/// qualquer profundidade diferente significa outra edição (mover, apagar) ou um Ctrl+Z:
/// retunar por cima restauraria a cena do grab e engoliria o que o artista fez.
#[test]
fn the_retune_window_dies_when_the_undo_moves() {
    let make = || {
        let (mut scene, _h, mut pen, xf) = scene_with(vec![square(2.0)]);
        let mut sess = OffsetSession::begin(&scene, &pen, &xf).expect("há seleção");
        sess.preview(&mut scene, &mut pen, 0.5);
        OffsetRetune::after_release(sess, 0.5).expect("churnou")
    };
    let k = expand_knobs();

    let mut win = make();
    assert_eq!(
        win.step(7, k),
        RetuneStep::Keep,
        "frame 1 aprende a profundidade"
    );
    assert_eq!(
        win.step(7, k),
        RetuneStep::Keep,
        "nada mudou = nada a fazer"
    );
    assert_eq!(
        win.step(8, k),
        RetuneStep::Dead,
        "edição alheia fecha a janela"
    );

    let mut win = make();
    assert_eq!(win.step(7, k), RetuneStep::Keep);
    assert_eq!(win.step(6, k), RetuneStep::Dead, "Ctrl+Z também fecha");
}

/// **Um arrasto que nunca churnou não abre janela** — não há resultado a retunar, e uma
/// janela sobre nada faria um clique perdido num chip re-rodar a cena inteira.
#[test]
fn a_drag_that_never_churned_opens_no_retune_window() {
    let (mut scene, _h, mut pen, xf) = scene_with(vec![square(2.0)]);
    let mut sess = OffsetSession::begin(&scene, &pen, &xf).expect("há seleção");
    sess.preview(&mut scene, &mut pen, 0.0);
    assert!(
        OffsetRetune::after_release(sess, 0.0).is_none(),
        "grab-e-solta sem mover não deixa janela armada"
    );
}
