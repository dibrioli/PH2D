//! **A geometria e as keys andam juntas, ou não andam** (ADR-0141 §2).

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{Transform, World};

use crate::{MotionPath, PathAnchor, PropKind, TangentKind, TimelineDoc, apply_from_doc};

/// Um L de 10 + 10, com uma key por âncora, e o objeto que a percorre.
fn rig() -> (World, ph2d_ecs::Entity, TimelineDoc) {
    let mut w = World::new();
    let e = w.spawn(Transform::default()).id();
    let mut doc = TimelineDoc::new();
    let path = MotionPath::new(vec![
        PathAnchor::corner([0.0, 0.0]),
        PathAnchor::corner([10.0, 0.0]),
        PathAnchor::corner([10.0, 10.0]),
    ]);
    doc.bind(e.to_bits(), PropKind::Position);
    for i in 0..3 {
        doc.insert_key(
            e.to_bits(),
            PropKind::Position,
            RationalTime::from_seconds(f64::from(i)),
            AnimValue::Float(path.arclen_at(i as usize).unwrap() as f32),
            Interp::Linear,
        );
    }
    doc.bindings_mut()[0].path = Some(path);
    (w, e, doc)
}

fn pos(w: &World, e: ph2d_ecs::Entity) -> [f32; 2] {
    let xf = w.get::<Transform>(e).unwrap();
    [xf.translation.x, xf.translation.y]
}

/// **A metade que faltava.** Mover uma âncora reescreve as distâncias que as KEYS
/// guardam, não só a tabela do caminho. Fechar apenas a tabela deixa o sistema
/// estável e errado: a curva nova na tela, o objeto ainda andando os números da velha.
///
/// O oráculo é onde o objeto ESTÁ nos instantes das keys — o único lugar onde as duas
/// metades têm de concordar, e o único que o artista olha.
#[test]
fn moving_an_anchor_rewrites_the_keys_too_so_the_object_still_lands_on_them() {
    let (mut w, e, mut doc) = rig();
    let target = doc.bindings()[0].target;

    // A ponta do L vai de (10, 10) para (10, 30): a perna de cima triplica.
    assert!(doc.move_path_anchor(target, 2, PathAnchor::corner([10.0, 30.0])));

    for (t, expect) in [
        (0.0, [0.0_f32, 0.0]),
        (1.0, [10.0, 0.0]),
        (2.0, [10.0, 30.0]),
    ] {
        apply_from_doc(&mut w, &mut doc, t);
        let p = pos(&w, e);
        assert!(
            (p[0] - expect[0]).abs() < 1e-3 && (p[1] - expect[1]).abs() < 1e-3,
            "em t={t} o objeto está em {p:?}, não na âncora {expect:?} — a key ainda \
             guarda a distância da curva ANTIGA"
        );
    }

    // E a key final de fato carrega o número novo (30 de perna + 10 de base).
    let AnimValue::Float(s) = doc
        .active_clip()
        .track(target)
        .unwrap()
        .keys()
        .last()
        .unwrap()
        .value
    else {
        panic!()
    };
    assert!(
        (s - 40.0).abs() < 1e-3,
        "a key guarda {s}, o percurso mede 40"
    );
}

/// ⚠️ **Uma edição espacial não move o TEMPO.** Puxar a curva muda por onde o objeto
/// passa, no mesmo compasso — quem muda o *quando* arrasta a key no dope-sheet, e é
/// outro gesto.
#[test]
fn dragging_an_anchor_leaves_the_timing_alone() {
    let (_, _, mut doc) = rig();
    let target = doc.bindings()[0].target;
    let times = |d: &TimelineDoc| -> Vec<f64> {
        d.active_clip()
            .track(target)
            .unwrap()
            .keys()
            .iter()
            .map(|k| k.t.to_seconds())
            .collect()
    };
    let before = times(&doc);
    assert!(doc.move_path_anchor(target, 1, PathAnchor::corner([4.0, 7.0])));
    assert_eq!(before, times(&doc), "os tempos das keys se moveram");
}

/// A porta **recusa** o que não é uma trajetória, em vez de escrever em silêncio.
#[test]
fn the_door_refuses_what_is_not_a_trajectory() {
    let (_, e, mut doc) = rig();
    let target = doc.bindings()[0].target;
    assert!(
        !doc.move_path_anchor(target, 99, PathAnchor::corner([0.0, 0.0])),
        "não há âncora 99"
    );

    // Um binding de OUTRO tipo, com o mesmo gesto.
    let scalar = doc.bind(e.to_bits(), PropKind::TranslationX);
    assert!(
        !doc.move_path_anchor(scalar, 0, PathAnchor::corner([1.0, 1.0])),
        "TranslationX não tem trajetória"
    );
    assert_eq!(doc.path_anchor(scalar, 0), None);
    assert_eq!(
        doc.path_anchor(target, 1).map(|a| a.anchor),
        Some([10.0, 0.0])
    );
}

/// Depois de mover uma âncora a track continua **monotónica** nas distâncias — é o que
/// a mantém legível no graph editor, e o que impede o objeto de andar para trás num
/// segmento que o artista só esticou.
#[test]
fn the_rewritten_distances_still_only_go_forward() {
    let (_, _, mut doc) = rig();
    let target = doc.bindings()[0].target;
    assert!(doc.move_path_anchor(target, 1, PathAnchor::corner([3.0, -8.0])));
    let vs: Vec<f32> = doc
        .active_clip()
        .track(target)
        .unwrap()
        .keys()
        .iter()
        .map(|k| match k.value {
            AnimValue::Float(v) => v,
            _ => panic!(),
        })
        .collect();
    assert!(
        vs.windows(2).all(|w| w[1] > w[0]),
        "as distâncias deixaram de crescer: {vs:?}"
    );
}

/// **O K do modo Path**: capturar a pose é acrescentar uma ÂNCORA, e o percurso
/// cresce. Três K's num objeto sem trajetória nenhuma constroem a curva inteira.
#[test]
fn keying_a_position_track_grows_the_trajectory() {
    let mut w = World::new();
    let e = w.spawn(Transform::default()).id();
    let mut doc = TimelineDoc::new();
    let target = doc.bind(e.to_bits(), PropKind::Position);

    for (t, p) in [
        (0.0, [0.0_f32, 0.0]),
        (1.0, [10.0, 0.0]),
        (2.0, [10.0, 10.0]),
    ] {
        assert!(doc.add_path_key(target, RationalTime::from_seconds(t), p));
    }
    let path = doc.bindings()[0].path.as_ref().unwrap();
    assert_eq!(path.len(), 3, "uma âncora por key");
    // ⚠️ Auto Bezier num ângulo reto **abaúla para FORA**, não corta por dentro: a alça
    // do meio segue a corda dos vizinhos (a diagonal), então a curva passa por fora da
    // quina. Medido 20,7205 contra os 20,0 exatos do L de quina viva — e é esse excesso
    // que prova que a âncora nasceu SUAVE. (Eu esperava um percurso menor; a medição
    // disse o contrário, e o comportamento é o do AE.)
    println!("MEDIDO  percurso das 3 keys = {:.4}", path.length());
    assert!(
        (20.05..21.5).contains(&path.length()),
        "percurso {:.4}: 20,0 exato é o L de quina viva (ninguém suavizou)",
        path.length()
    );

    // E o objeto de fato passa por onde o artista apertou K.
    for (t, p) in [
        (0.0, [0.0_f32, 0.0]),
        (1.0, [10.0, 0.0]),
        (2.0, [10.0, 10.0]),
    ] {
        apply_from_doc(&mut w, &mut doc, t);
        let got = pos(&w, e);
        let d = ((got[0] - p[0]).powi(2) + (got[1] - p[1]).powi(2)).sqrt();
        assert!(
            d < 1e-3,
            "em t={t} o objeto está a {d:.4} de onde o K foi dado"
        );
    }
}

/// ⚠️ **Uma âncora inserida no MEIO re-suaviza os vizinhos `auto`, e SÓ os `auto`.**
///
/// Uma alça Auto Bezier é função dos dois vizinhos, então inserir entre dois pontos
/// invalida as alças dos dois. Re-suavizar quem o artista moldou jogaria o trabalho
/// dele fora; nunca re-suavizar deixa uma dobra visível numa curva que era lisa.
#[test]
fn a_new_anchor_resmooths_the_auto_neighbours_and_spares_the_shaped_one() {
    let mut doc = TimelineDoc::new();
    let target = doc.bind(1, PropKind::Position);
    for (t, p) in [(0.0, [0.0_f32, 0.0]), (2.0, [20.0, 0.0])] {
        doc.add_path_key(target, RationalTime::from_seconds(t), p);
    }
    // O artista MOLDA a primeira âncora (auto = false a partir daqui).
    let mut shaped = doc.path_anchor(target, 0).unwrap();
    shaped.out_handle = [1.0, 9.0];
    shaped.auto = false;
    assert!(doc.move_path_anchor(target, 0, shaped));
    // A alça de ENTRADA do último — a de saída dele é zero por construção (não há
    // vizinho adiante), então compará-la seria comparar dois zeros.
    let before_auto = doc.path_anchor(target, 1).unwrap().in_handle;

    // Um K no meio, bem fora da linha.
    assert!(doc.add_path_key(target, RationalTime::from_seconds(1.0), [10.0, 12.0]));

    assert_eq!(
        doc.path_anchor(target, 0).unwrap().out_handle,
        [1.0, 9.0],
        "a âncora MOLDADA foi re-suavizada — o trabalho do artista foi jogado fora"
    );
    assert_ne!(
        doc.path_anchor(target, 2).unwrap().in_handle,
        before_auto,
        "o vizinho AUTO não re-suavizou — a curva fica com uma dobra onde era lisa"
    );
    assert!(
        doc.path_anchor(target, 1).unwrap().auto,
        "a nova nasce auto"
    );
}

/// Re-keyar um instante que já tem key **move** aquela âncora, não empilha uma segunda
/// — o contrato do `upsert_key`, honrado também aqui.
#[test]
fn keying_the_same_instant_moves_that_anchor_instead_of_stacking_one() {
    let mut doc = TimelineDoc::new();
    let target = doc.bind(1, PropKind::Position);
    for (t, p) in [(0.0, [0.0_f32, 0.0]), (1.0, [5.0, 0.0])] {
        doc.add_path_key(target, RationalTime::from_seconds(t), p);
    }
    assert!(doc.add_path_key(target, RationalTime::from_seconds(1.0), [5.0, 9.0]));
    assert_eq!(doc.bindings()[0].path.as_ref().unwrap().len(), 2);
    assert_eq!(doc.path_anchor(target, 1).unwrap().anchor, [5.0, 9.0]);
    assert_eq!(doc.active_clip().track(target).unwrap().keys().len(), 2);
}

/// A porta recusa o que não é Position, e a primeira âncora de um caminho não tem
/// percurso — o que é o comportamento de qualquer canal com uma key só.
#[test]
fn the_first_key_has_no_journey_and_a_scalar_target_is_refused() {
    let mut doc = TimelineDoc::new();
    let scalar = doc.bind(1, PropKind::TranslationX);
    assert!(!doc.add_path_key(scalar, RationalTime::from_seconds(0.0), [1.0, 1.0]));

    let target = doc.bind(1, PropKind::Position);
    assert!(doc.add_path_key(target, RationalTime::from_seconds(0.0), [3.0, 4.0]));
    let path = doc.bindings()[1].path.as_ref().unwrap();
    assert_eq!(path.len(), 1);
    assert_eq!(path.length(), 0.0);
}

/// **A queixa do smoke: arrastar uma âncora não pode QUEBRAR a curva.**
///
/// Numa trajetória toda-`auto`, mover uma âncora tem de re-derivar as alças Auto Bezier
/// dela e dos vizinhos — senão elas continuam apontando para onde a âncora ESTAVA e a
/// curva entorta. O oráculo é a alça de SAÍDA da âncora movida: numa Auto Bezier ela é
/// paralela à corda dos vizinhos (`next − prev`); depois do arrasto tem de seguir a
/// corda NOVA, não a velha.
#[test]
fn dragging_an_anchor_re_smooths_so_the_curve_does_not_break() {
    let mut doc = TimelineDoc::new();
    let target = doc.bind(1, PropKind::Position);
    for (t, p) in [
        (0.0, [0.0_f32, 0.0]),
        (1.0, [10.0, 0.0]),
        (2.0, [20.0, 0.0]),
    ] {
        doc.add_path_key(target, RationalTime::from_seconds(t), p);
    }
    // A do meio sobe muito. A corda dos vizinhos continua horizontal (0→20 em y=0), mas
    // a POSIÇÃO da âncora mudou, e é a corda que manda na alça auto — então o teste real
    // é a âncora do meio contra os vizinhos MOVIDOS, não ela.
    let prev = doc.path_anchor(target, 0).unwrap().anchor;
    let next = doc.path_anchor(target, 2).unwrap().anchor;
    let mut mid = doc.path_anchor(target, 1).unwrap();
    mid.anchor = [10.0, 12.0];
    assert!(doc.move_path_anchor(target, 1, mid));

    let a = doc.path_anchor(target, 1).unwrap();
    assert!(a.auto, "a âncora movida continua auto (posição, não forma)");
    // A alça de saída paralela à corda (next − prev): produto vetorial ≈ 0.
    let chord = [next[0] - prev[0], next[1] - prev[1]];
    let cross = a.out_handle[0] * chord[1] - a.out_handle[1] * chord[0];
    let scale = a.out_handle[0].hypot(a.out_handle[1]) * chord[0].hypot(chord[1]);
    println!("MEDIDO  cross/scale = {:.4}", cross / scale.max(1e-6));
    assert!(
        (cross / scale.max(1e-6)).abs() < 1e-3,
        "a alça auto não seguiu a corda dos vizinhos — a curva quebrou"
    );

    // E um vizinho AUTO re-derivou: a alça de entrada do último aponta para a âncora
    // do meio, que acabou de subir.
    let last_in = doc.path_anchor(target, 2).unwrap().in_handle;
    assert!(
        last_in[1] > 0.1,
        "o vizinho não re-suavizou: a alça dele ainda ignora a âncora que subiu ({last_in:?})"
    );
}

/// **A alça de tangente MOLDA a curva** (o *"ainda não temos alças de manipulação"*).
///
/// Arrastar a ponta de uma alça: a âncora vira MOLDADA (`auto = false`), a alça
/// arrastada aponta para a ponta, a OPOSTA espelha (fica collinear) e o objeto passa a
/// fazer um caminho diferente — o percurso cresce.
#[test]
fn dragging_a_tangent_shapes_the_curve_and_marks_the_anchor() {
    let mut doc = TimelineDoc::new();
    let target = doc.bind(1, PropKind::Position);
    for (t, p) in [
        (0.0, [0.0_f32, 0.0]),
        (1.0, [10.0, 0.0]),
        (2.0, [20.0, 0.0]),
    ] {
        doc.add_path_key(target, RationalTime::from_seconds(t), p);
    }
    let before = doc.position_path(1).unwrap().length();

    // Puxa a alça de SAÍDA da âncora do meio para bem longe, para cima.
    let mid = doc.path_anchor(target, 1).unwrap().anchor;
    assert!(doc.move_path_tangent(target, 1, true, [mid[0] + 3.0, mid[1] + 9.0]));

    let a = doc.path_anchor(target, 1).unwrap();
    assert!(!a.auto, "a âncora moldada deixa de ser auto");
    assert_eq!(a.out_handle, [3.0, 9.0], "a alça de saída vai para a ponta");
    // A oposta é collinear (produto vetorial ≈ 0) e aponta para o LADO OPOSTO.
    let cross = a.in_handle[0] * a.out_handle[1] - a.in_handle[1] * a.out_handle[0];
    assert!(
        cross.abs() < 1e-4,
        "a alça oposta não é collinear: {:?}",
        a.in_handle
    );
    assert!(
        a.in_handle[1] < 0.0,
        "a oposta aponta para o lado contrário"
    );

    let after = doc.position_path(1).unwrap().length();
    println!("MEDIDO  percurso {before:.4} -> {after:.4}");
    assert!(after > before + 1.0, "moldar a curva não mudou o percurso");

    // E uma edição de alça NÃO re-suaviza a âncora de volta (ela é moldada agora).
    let m2 = doc.path_anchor(target, 1).unwrap().anchor;
    doc.move_path_tangent(target, 1, false, [m2[0] - 1.0, m2[1] - 5.0]);
    assert!(!doc.path_anchor(target, 1).unwrap().auto);
}

/// Numa PONTA do caminho a alça oposta é zero e o espelho não a acorda — puxar a alça
/// de saída da primeira âncora não inventa uma alça de entrada onde não há vizinho.
#[test]
fn shaping_an_end_anchor_does_not_wake_the_zero_opposite_handle() {
    let mut doc = TimelineDoc::new();
    let target = doc.bind(1, PropKind::Position);
    for (t, p) in [(0.0, [0.0_f32, 0.0]), (2.0, [10.0, 0.0])] {
        doc.add_path_key(target, RationalTime::from_seconds(t), p);
    }
    assert!(doc.move_path_tangent(target, 0, true, [4.0, 6.0]));
    let a = doc.path_anchor(target, 0).unwrap();
    assert_eq!(a.out_handle, [4.0, 6.0]);
    assert_eq!(
        a.in_handle,
        [0.0, 0.0],
        "a ponta não ganhou alça de entrada"
    );
}

/// A porta de tangente recusa o que não é uma trajetória Position.
#[test]
fn move_path_tangent_refuses_a_scalar_target() {
    let mut doc = TimelineDoc::new();
    let scalar = doc.bind(1, PropKind::TranslationX);
    assert!(!doc.move_path_tangent(scalar, 0, true, [1.0, 1.0]));
}

// ── the three handle types on the drag door (TangentKind) ────────────────────────
//
// `move_path_tangent`'s Smooth branch is the byte-identical old behaviour (covered by
// `dragging_a_tangent_shapes_the_curve_and_marks_the_anchor`). These pin the other two
// arms so a mutation of either bleeds, and the convert door that the menu drives.

/// Build a straight 3-key horizontal path and return `(doc, target)`.
fn straight3() -> (TimelineDoc, ph2d_anim::AnimTarget) {
    let mut doc = TimelineDoc::new();
    let target = doc.bind(1, PropKind::Position);
    for (t, p) in [
        (0.0, [0.0_f32, 0.0]),
        (1.0, [10.0, 0.0]),
        (2.0, [20.0, 0.0]),
    ] {
        doc.add_path_key(target, RationalTime::from_seconds(t), p);
    }
    (doc, target)
}

fn key_value(doc: &TimelineDoc, target: ph2d_anim::AnimTarget, i: usize) -> f32 {
    match doc.active_clip().track(target).unwrap().keys()[i].value {
        AnimValue::Float(v) => v,
        _ => panic!(),
    }
}

/// **Corner: dragging one handle leaves the OPPOSITE exactly where it was** — the cusp
/// the artist reaches for to kink the trajectory.
#[test]
fn dragging_a_corner_tangent_leaves_the_opposite_handle_untouched() {
    let (mut doc, target) = straight3();
    assert!(doc.set_path_tangent_kind(target, 1, TangentKind::Corner));
    let before_in = doc.path_anchor(target, 1).unwrap().in_handle;

    let mid = doc.path_anchor(target, 1).unwrap().anchor;
    assert!(doc.move_path_tangent(target, 1, true, [mid[0] + 3.0, mid[1] + 9.0]));

    let a = doc.path_anchor(target, 1).unwrap();
    assert_eq!(
        a.out_handle,
        [3.0, 9.0],
        "the dragged handle went to the tip"
    );
    assert_eq!(a.in_handle, before_in, "Corner: the opposite handle moved");
    assert_eq!(a.kind, TangentKind::Corner, "the drag kept the kind");
}

/// **Symmetric: dragging one handle mirrors the opposite EXACTLY** (same length, opposite
/// direction) — the most constrained of the three.
#[test]
fn dragging_a_symmetric_tangent_mirrors_the_opposite_exactly() {
    let (mut doc, target) = straight3();
    assert!(doc.set_path_tangent_kind(target, 1, TangentKind::Symmetric));

    let mid = doc.path_anchor(target, 1).unwrap().anchor;
    assert!(doc.move_path_tangent(target, 1, true, [mid[0] + 3.0, mid[1] + 9.0]));

    let a = doc.path_anchor(target, 1).unwrap();
    assert_eq!(a.out_handle, [3.0, 9.0]);
    assert_eq!(
        a.in_handle,
        [-3.0, -9.0],
        "Symmetric: the opposite is not the exact mirror of the dragged handle"
    );
}

/// **Setting the kind rewrites the key distances**: `Symmetric` averages the two handle
/// lengths, which changes a segment's arc length, so the number every following key stores
/// changes with it (the single-door invariant of `rewrite_path_key_values`). And the door
/// refuses a non-Position target and a missing anchor.
#[test]
fn setting_the_tangent_kind_rewrites_the_key_distances_and_refuses_bad_targets() {
    let (mut doc, target) = straight3();
    // Shape the middle asymmetrically: a long out handle; the Smooth mirror keeps the in
    // handle's own (short auto) length, so in and out lengths now differ.
    assert!(doc.move_path_tangent(target, 1, true, [12.0, 8.0]));
    let last_before = key_value(&doc, target, 2);

    // Convert to Symmetric → both handles take the average length → the arc changes.
    assert!(doc.set_path_tangent_kind(target, 1, TangentKind::Symmetric));
    let last_after = key_value(&doc, target, 2);
    assert!(
        (last_before - last_after).abs() > 1e-3,
        "Symmetric averaged the handles but the key distance did not follow: \
         {last_before} -> {last_after}"
    );

    // Refusals: a scalar target and an out-of-range anchor touch nothing.
    let scalar = doc.bind(2, PropKind::TranslationX);
    assert!(!doc.set_path_tangent_kind(scalar, 0, TangentKind::Corner));
    assert!(!doc.set_path_tangent_kind(target, 99, TangentKind::Corner));
}

fn dd(a: [f32; 2], b: [f32; 2]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

// ── inserting a point on the path (ADR-0141): shape + motion preserved ───────────
//
// Double-clicking the curve adds a waypoint by de Casteljau split — the "add point" of a
// vector editor. The promise is that the object passes exactly the same place at the same
// time; the new anchor is an editable handle, not a bump.

/// **The money gate.** Adding a point on the path must change NEITHER the curve's shape NOR
/// the object's motion — it only grows an editable anchor. A curved path with linear keys,
/// sampled before and after: every sample lands in the same place.
#[test]
fn inserting_a_point_preserves_the_shape_and_the_motion() {
    let mut w = World::new();
    let e = w.spawn(Transform::default()).id();
    let mut doc = TimelineDoc::new();
    // Three K's -> a CURVED auto-smooth path with a LINEAR distance track.
    for (t, p) in [
        (0.0, [0.0_f32, 0.0]),
        (1.0, [10.0, 4.0]),
        (2.0, [16.0, -4.0]),
    ] {
        doc.key_the_path(e.to_bits(), RationalTime::from_seconds(t), p);
    }
    let target = doc.bindings()[0].target;

    // Where the object is at a handful of times, BEFORE the insert.
    let times = [0.3_f64, 0.7, 1.2, 1.6];
    let before: Vec<[f32; 2]> = times
        .iter()
        .map(|&t| {
            apply_from_doc(&mut w, &mut doc, t);
            pos(&w, e)
        })
        .collect();

    // Insert a point 40% along the FIRST segment (mid-curve, not on an anchor).
    let d = doc
        .position_path(e.to_bits())
        .unwrap()
        .arclen_at(1)
        .unwrap()
        * 0.4;
    assert!(
        doc.insert_path_anchor_at(target, d),
        "the point was not inserted"
    );

    // 1:1 grew by one, on both sides.
    let path = doc.position_path(e.to_bits()).unwrap();
    assert_eq!(path.len(), 4, "no fourth anchor");
    assert_eq!(
        doc.active_clip().track(target).unwrap().keys().len(),
        4,
        "the keys did not follow the anchors 1:1"
    );

    // And the motion is UNTOUCHED: every sample lands where it did.
    let mut worst = 0.0f32;
    for (k, &t) in times.iter().enumerate() {
        apply_from_doc(&mut w, &mut doc, t);
        let now = pos(&w, e);
        worst = worst.max(dd(now, before[k]));
    }
    println!("MEASURED worst drift after inserting a point = {worst:e}");
    assert!(
        worst < 1e-2,
        "the object moved {worst:e} after inserting a point — the insert warped the path or \
         shifted the timing (it must do neither)"
    );
}

/// The inserted anchor sits at the CLICKED distance, at the point the path already had
/// there — the de Casteljau split point, not a re-invented one.
#[test]
fn the_inserted_anchor_sits_at_the_clicked_point() {
    let mut doc = TimelineDoc::new();
    for (t, p) in [
        (0.0, [0.0_f32, 0.0]),
        (1.0, [10.0, 4.0]),
        (2.0, [16.0, -4.0]),
    ] {
        doc.key_the_path(1, RationalTime::from_seconds(t), p);
    }
    let target = doc.bindings()[0].target;
    let d = doc.position_path(1).unwrap().length() * 0.55;
    let want = doc.position_path(1).unwrap().at(d).unwrap().point;

    assert!(doc.insert_path_anchor_at(target, d));

    // Which anchor is new? The one whose distance is ~d.
    let path = doc.position_path(1).unwrap();
    let i = (0..path.len())
        .find(|&i| (path.arclen_at(i).unwrap() - d).abs() < 1e-2)
        .expect("an anchor landed at the clicked distance");
    let got = path.anchors()[i].anchor;
    assert!(
        dd(got, want) < 1e-2,
        "the new anchor sits at {got:?}, the clicked point was {want:?}"
    );
    assert!(
        !path.anchors()[i].auto,
        "the inserted anchor is shaped, not auto"
    );
}

/// The door refuses a non-Position target and a path too short to have a segment.
#[test]
fn insert_refuses_a_scalar_target_and_a_bare_path() {
    let mut doc = TimelineDoc::new();
    let scalar = doc.bind(1, PropKind::TranslationX);
    assert!(
        !doc.insert_path_anchor_at(scalar, 5.0),
        "TranslationX has no path"
    );

    // A one-anchor path has no segment to split.
    let target = doc.bind(2, PropKind::Position);
    doc.add_path_key(target, RationalTime::from_seconds(0.0), [0.0, 0.0]);
    assert!(
        !doc.insert_path_anchor_at(target, 0.0),
        "a one-point path cannot be split"
    );
}
