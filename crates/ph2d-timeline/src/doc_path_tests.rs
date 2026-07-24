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
