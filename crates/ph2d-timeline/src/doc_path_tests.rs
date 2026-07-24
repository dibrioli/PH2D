//! **A geometria e as keys andam juntas, ou não andam** (ADR-0141 §2).

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{Transform, World};

use crate::{MotionPath, PathAnchor, PropKind, TimelineDoc, apply_from_doc};

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
