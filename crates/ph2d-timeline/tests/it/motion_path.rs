//! **The Position channel end to end** (ADR-0141): a scalar track whose value is a
//! distance, a trajectory on the binding, and an object that follows it.
//!
//! These drive `apply_from_doc` — the product's own pass — rather than the geometry
//! in isolation, because the thing worth pinning is that a *distance* reaches the
//! Transform as a *place*.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Transform, World};
use ph2d_timeline::{MotionPath, PathAnchor, PropKind, TimelineDoc, apply_from_doc};

/// An L-shaped path: right 10, then up 10, with square corners. Its total length is
/// 20 and every point on it is trivially checkable by hand, which is what makes it a
/// good oracle for "did the object go where the distance says".
fn ell() -> MotionPath {
    MotionPath::new(vec![
        PathAnchor::corner([0.0, 0.0]),
        PathAnchor::corner([10.0, 0.0]),
        PathAnchor::corner([10.0, 10.0]),
    ])
}

/// A world with one entity, and a document with a Position binding on it carrying
/// `path` plus the keys `(t, s)`.
fn rig(path: MotionPath, keys: &[(f64, f32)]) -> (World, Entity, TimelineDoc) {
    let mut w = World::new();
    let e = w.spawn(Transform::from_translation(Vec2::ZERO)).id();
    let mut doc = TimelineDoc::new();
    // Bind explicitly: a rig may want the binding (and its path) BEFORE any key —
    // which is also the real order of the authoring gesture.
    doc.bind(e.to_bits(), PropKind::Position);
    for &(t, s) in keys {
        doc.insert_key(
            e.to_bits(),
            PropKind::Position,
            RationalTime::from_seconds(t),
            AnimValue::Float(s),
            Interp::Linear,
        );
    }
    let i = doc
        .bindings()
        .iter()
        .position(|b| b.prop == PropKind::Position)
        .expect("the key bound one");
    let target = doc.bindings()[i].target;
    doc.set_clip_path(doc.active_index(), target, path);
    (w, e, doc)
}

fn pos(w: &World, e: Entity) -> [f32; 2] {
    let xf = w.get::<Transform>(e).unwrap();
    [xf.translation.x, xf.translation.y]
}

fn near(a: [f32; 2], b: [f32; 2]) -> bool {
    (a[0] - b[0]).abs() < 1e-3 && (a[1] - b[1]).abs() < 1e-3
}

/// **The channel, whole.** The track holds distances; the object lands on the
/// trajectory at those distances. Mid-track it is around the CORNER — which is the
/// point of the whole design: a straight line in the graph editor is not a straight
/// line on the canvas.
#[test]
fn a_position_track_walks_the_object_along_its_path() {
    // 0 s at the start, 2 s at the far end (20 units of travel).
    let (mut w, e, mut doc) = rig(ell(), &[(0.0, 0.0), (2.0, 20.0)]);

    apply_from_doc(&mut w, &mut doc, 0.0);
    assert!(near(pos(&w, e), [0.0, 0.0]), "start: {:?}", pos(&w, e));

    // A quarter of the time, a quarter of the DISTANCE: 2.5 units along the first
    // leg. ⚠️ This sample is NOT at a symmetric fraction on purpose — at halves and
    // at segment boundaries a curve walked by its PARAMETER agrees with one walked
    // by arc length, so a gate that only checked those would pass on an engine that
    // never measured anything (it did: the mutation that swaps `inv_arclen` for
    // `s / span` survived this file until this line existed).
    apply_from_doc(&mut w, &mut doc, 0.25);
    assert!(near(pos(&w, e), [2.5, 0.0]), "quarter: {:?}", pos(&w, e));

    // Half the time, half the DISTANCE: 10 units along an L is the corner.
    apply_from_doc(&mut w, &mut doc, 1.0);
    let mid = pos(&w, e);
    assert!(near(mid, [10.0, 0.0]), "halfway is the corner, got {mid:?}");

    // Three quarters: 15 units — 10 across, then 5 up the second leg.
    apply_from_doc(&mut w, &mut doc, 1.5);
    assert!(near(pos(&w, e), [10.0, 5.0]), "{:?}", pos(&w, e));

    apply_from_doc(&mut w, &mut doc, 2.0);
    assert!(near(pos(&w, e), [10.0, 10.0]), "{:?}", pos(&w, e));
}

/// **The trajectory is not a straight line, and a linear track proves it.** Between
/// the two keys the value interpolates linearly, so a channel that had quietly
/// become "lerp the two endpoints" would put the object on the DIAGONAL. It is the
/// same failure the FLIP tween documented one module over: a lerp cuts the chord.
#[test]
fn the_object_follows_the_curve_and_not_the_chord_between_its_ends() {
    let (mut w, e, mut doc) = rig(ell(), &[(0.0, 0.0), (2.0, 20.0)]);
    apply_from_doc(&mut w, &mut doc, 1.0);
    let mid = pos(&w, e);
    let chord = [5.0, 5.0]; // where a lerp of the two endpoints would land
    let off_chord = ((mid[0] - chord[0]).powi(2) + (mid[1] - chord[1]).powi(2)).sqrt();
    assert!(
        off_chord > 5.0,
        "the object sat {off_chord:.3} from the straight-line answer — that is the \
         straight-line answer, and the path was ignored"
    );
}

/// **A Position binding's `rest` is a DISTANCE**, captured by projecting the authored
/// pose onto the path — never a coordinate, and never zero.
///
/// Zero is the start of the trajectory. `rest` exists precisely so a lane that only
/// partially covers has something honest to fade in *from* (ADR-0115 R5), and the
/// failure it was invented to prevent — the sprite flying to an origin nobody chose —
/// is exactly what a zero here would reintroduce.
#[test]
fn a_position_bindings_rest_is_where_the_pose_sits_on_the_path_not_zero() {
    let (mut w, e, mut doc) = rig(ell(), &[(0.0, 0.0), (2.0, 20.0)]);
    // Park the object beside the second leg, level with 7 units up it — i.e. 17
    // units along the path — before the timeline has ever written to it.
    w.get_mut::<Transform>(e).unwrap().translation = Vec2::new(10.6, 7.0);

    apply_from_doc(&mut w, &mut doc, 0.0);

    let rest = doc.bindings()[0]
        .rest
        .expect("captured on the first live frame");
    assert!(
        (rest - 17.0).abs() < 1e-2,
        "rest is {rest:.4}; the pose sits 17 units along this path"
    );
    assert!(
        rest > 1.0,
        "a rest of ~0 means the base is the START of the trajectory, which is the \
         'fly to the origin' bug this field exists to prevent"
    );
}

/// A Position binding with no trajectory yet writes NOTHING — the same silence an
/// empty track gets. A default would be a place nobody authored.
#[test]
fn a_position_binding_without_a_path_leaves_the_object_alone() {
    let (mut w, e, mut doc) = rig(ell(), &[(0.0, 0.0), (2.0, 20.0)]);
    {
        let t = doc.bindings()[0].target;
        doc.clear_clip_path(doc.active_index(), t);
    }
    w.get_mut::<Transform>(e).unwrap().translation = Vec2::new(3.0, 4.0);

    apply_from_doc(&mut w, &mut doc, 1.0);

    assert_eq!(pos(&w, e), [3.0, 4.0], "the object must not have moved");
    assert_eq!(
        doc.bindings()[0].rest,
        None,
        "and no rest was invented for a path that does not exist"
    );
}

/// A distance past the end of the trajectory means "at the end" — a key can outlive
/// the anchor that justified it (the artist deletes a leg), and the honest answer is
/// the last point on the path, not a panic and not the origin.
#[test]
fn a_distance_past_the_end_of_the_path_holds_at_the_end() {
    let (mut w, e, mut doc) = rig(ell(), &[(0.0, 0.0), (2.0, 500.0)]);
    apply_from_doc(&mut w, &mut doc, 2.0);
    assert!(near(pos(&w, e), [10.0, 10.0]), "{:?}", pos(&w, e));
}

/// The Position track survives the file, and so does its trajectory.
#[test]
fn a_position_binding_and_its_path_round_trip_through_the_document() {
    let (_, _, doc) = rig(ell(), &[(0.0, 0.0), (2.0, 20.0)]);
    let bytes = doc.to_bytes().unwrap();
    let back = TimelineDoc::from_bytes(&bytes).unwrap();
    let path = back
        .clip_path(back.active_index(), back.bindings()[0].target)
        .expect("the trajectory came back");
    assert_eq!(path.anchors(), ell().anchors());
    assert!(
        (path.length() - 20.0).abs() < 1e-9,
        "the derived table was rebuilt on load: {}",
        path.length()
    );
}

/// **O que o modo Path ganha de graça, provado em vez de prometido.**
///
/// A track mede DISTÂNCIA, então a inclinação que o graph editor desenha e que o
/// speed graph plota é, literalmente, a velocidade do objeto na tela. Isto é a
/// definição de parametrização por comprimento de arco enunciada como o fato que o
/// artista vê — e é o argumento inteiro do ADR-0141 §2 para não inventar um canal 2D.
///
/// Oráculo por diferenças finitas nos DOIS lados (o número da track, e o ponto que o
/// apply de fato escreve), nunca chamando `sample_speed`: uma função checada por ela
/// mesma é um espelho, não um oráculo.
#[test]
fn the_slope_of_the_track_is_the_speed_on_the_canvas() {
    // Uma curva de verdade, com quina: numa reta todo mundo acerta.
    let path = MotionPath::new(vec![
        PathAnchor::corner([0.0, 0.0]),
        MotionPath::auto_smooth(Some([0.0, 0.0]), [10.0, 4.0], Some([16.0, -6.0])),
        PathAnchor::corner([16.0, -6.0]),
    ]);
    let total = path.length() as f32;
    // Ease-in-out: a velocidade VARIA, então uma identidade que só valesse em
    // velocidade constante não passaria aqui.
    let (mut w, e, mut doc) = rig(path, &[]);
    for (t, s, interp) in [
        (
            0.0_f64,
            0.0_f32,
            Interp::Bezier {
                x1: 0.6,
                y1: 0.0,
                x2: 0.4,
                y2: 1.0,
            },
        ),
        (2.0, total, Interp::Linear),
    ] {
        doc.insert_key(
            e.to_bits(),
            PropKind::Position,
            RationalTime::from_seconds(t),
            AnimValue::Float(s),
            interp,
        );
    }

    let h = 1e-3;
    let mut worst = 0.0f64;
    for step in 1..20 {
        let t = 2.0 * f64::from(step) / 20.0;
        let track = |t: f64| {
            doc.active_clip()
                .track(doc.bindings()[0].target)
                .map(|tr| {
                    use ph2d_anim::AttributeEvaluator;
                    match tr.sample(t) {
                        AnimValue::Float(v) => f64::from(v),
                        _ => unreachable!(),
                    }
                })
                .unwrap()
        };
        let ds = track(t + h) - track(t - h);

        apply_from_doc(&mut w, &mut doc, t - h);
        let a = pos(&w, e);
        apply_from_doc(&mut w, &mut doc, t + h);
        let b = pos(&w, e);
        let dp = f64::from(((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt());

        // O quociente é 1 quando "andar 1 na track" é "andar 1 na tela".
        worst = worst.max((dp / ds - 1.0).abs());
    }
    // Medido: 5,2e-5 — o resíduo é a corda contra o arco na janela de diferença
    // finita, não erro do motor.
    println!("MEDIDO  pior desvio de |dp/ds - 1| = {worst:.2e}");
    assert!(
        worst < 0.01,
        "a inclinação da track errou a velocidade na tela por {worst:.2e}: o valor que \
         o speed graph plota não é a velocidade do objeto"
    );
}

/// **Roving em modo Path é velocidade constante NA TELA.**
///
/// `Track::resolve_roving` deriva o tempo de uma key roving para velocidade constante
/// em VALOR — e como o valor aqui é distância percorrida, isso é velocidade constante
/// no canvas, sobre uma curva, sem nada novo. É o segundo item que o ADR-0141
/// prometeu de graça.
#[test]
fn a_roving_key_gives_constant_speed_along_the_curve() {
    let path = MotionPath::new(vec![
        PathAnchor::corner([0.0, 0.0]),
        MotionPath::auto_smooth(Some([0.0, 0.0]), [10.0, 6.0], Some([20.0, 0.0])),
        PathAnchor::corner([20.0, 0.0]),
    ]);
    let (l0, l1) = (path.arclen_at(1).unwrap() as f32, path.length() as f32);
    let (mut w, e, mut doc) = rig(path, &[]);

    // Três keys, e a do MEIO no tempo ERRADO de propósito (0,2 s de 2 s, quando a
    // metade do caminho já foi percorrida) — sem o rove o objeto dispara e depois
    // rasteja.
    let target = doc.bindings()[0].target;
    let mut ids = Vec::new();
    for (t, s) in [(0.0_f64, 0.0_f32), (0.2, l0), (2.0, l1)] {
        let (_, id) = doc.insert_key(
            e.to_bits(),
            PropKind::Position,
            RationalTime::from_seconds(t),
            AnimValue::Float(s),
            Interp::Linear,
        );
        ids.push(id);
    }

    let speeds = |w: &mut World, doc: &mut TimelineDoc| -> Vec<f64> {
        (1..12)
            .map(|k| {
                let t = 2.0 * f64::from(k) / 12.0;
                let h = 5e-3;
                apply_from_doc(w, doc, t - h);
                let a = pos(w, e);
                apply_from_doc(w, doc, t + h);
                let b = pos(w, e);
                f64::from(((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt()) / (2.0 * h)
            })
            .collect()
    };
    let spread = |v: &[f64]| {
        v.iter().copied().fold(0.0, f64::max) / v.iter().copied().fold(f64::INFINITY, f64::min)
    };

    // O CONTROLE: sem rove, a velocidade é tudo menos constante.
    let before = spread(&speeds(&mut w, &mut doc));

    let tr = doc.active_clip_mut().track_mut(target).unwrap();
    assert!(tr.set_roving(ids[1], true), "a key do meio passa a rovar");
    tr.resolve_roving();
    let after = spread(&speeds(&mut w, &mut doc));

    println!("MEDIDO  espalhamento de velocidade: {before:.2}x sem rove -> {after:.2}x com");
    assert!(
        before > 3.0,
        "a fixture não contém o fenômeno: sem rove a velocidade já variava só \
         {before:.2}x, então o gate passaria com o rove desligado"
    );
    assert!(
        after < 1.15,
        "com rove a velocidade ainda variou {after:.2}x — não é velocidade constante \
         ao longo da curva"
    );
}

// ── A TRAJETÓRIA É DO CLIP — um clip novo nasce em BRANCO ────────────────────

/// **Um clip criado depois não tem trajetória nenhuma** (report do Enio, 2026-07-30, com
/// foto: *"Ao criar Clip 2 tudo buga, alças em path fantasma aparecem, não consigo criar
/// keys onde quero. Cada clip novo deve ser um branco e criar do zero seu próprio PATH"*).
///
/// A trajetória morava no BINDING, que é do DOCUMENTO, sob o argumento de que dois clips
/// que animam o objeto são duas CRONOMETRAGENS da mesma jornada. A lei irmã —
/// *"âncora `i` pareia com a key `i`"* — só pode valer para UMA delas, porque a track é do
/// CLIP; e o preço visível eram as alças do clip 1 agarráveis dentro do clip 2, sobre uma
/// curva que ele não autorou.
///
/// **Mutação que deve sangrar:** `clip_path` ler o primeiro clip que tenha caminho em vez
/// do clip pedido (o fallback de RESOLUÇÃO não pode vazar para a LEITURA CRUA — é ela que
/// o desenho e o hit-test do canvas fazem).
#[test]
fn a_clip_created_later_starts_with_no_path_at_all() {
    let (_w, e, mut doc) = rig(MotionPath::new(Vec::new()), &[]);
    let bits = e.to_bits();
    let target = doc
        .binding_for(bits, PropKind::Position)
        .expect("bound")
        .target;

    // CLIP A autora um trilho de dois pontos.
    doc.key_the_path(bits, RationalTime::from_seconds(0.0), [0.0, 0.0]);
    doc.key_the_path(bits, RationalTime::from_seconds(1.0), [10.0, 0.0]);
    assert_eq!(doc.clip_path(0, target).map(MotionPath::len), Some(2));

    // CLIP B, criado depois: BRANCO.
    let b = doc.add_clip("B".into());
    doc.set_active(b);
    assert!(
        doc.clip_path(b, target).is_none(),
        "um clip novo não herda trajetória — não há alça a desenhar nem curva a agarrar"
    );
}

/// **E a primeira key desse clip cria a trajetória DELE, do zero.**
///
/// A metade que torna o branco útil em vez de um beco: dentro do clip 2 o K põe a âncora
/// onde o artista clicou (era o *"não consigo criar keys onde quero"* — o modelo anterior
/// projetava a pose no trilho alheio e keyava PROGRESSO ao longo dele).
///
/// **Mutação que deve sangrar:** `add_path_key` instalar a trajetória em qualquer clip que
/// não o ATIVO.
#[test]
fn the_first_key_of_a_new_clip_builds_that_clips_own_path() {
    let (mut w, e, mut doc) = rig(MotionPath::new(Vec::new()), &[]);
    let bits = e.to_bits();
    let target = doc
        .binding_for(bits, PropKind::Position)
        .expect("bound")
        .target;

    doc.key_the_path(bits, RationalTime::from_seconds(0.0), [0.0, 0.0]);
    doc.key_the_path(bits, RationalTime::from_seconds(1.0), [10.0, 0.0]);
    let a = doc.clip_path(0, target).cloned().expect("o trilho de A");

    let b = doc.add_clip("B".into());
    doc.set_active(b);
    // O artista desenha OUTRO percurso, para CIMA, num clip que nasceu vazio.
    doc.key_the_path(bits, RationalTime::from_seconds(0.0), [0.0, 0.0]);
    doc.key_the_path(bits, RationalTime::from_seconds(1.0), [0.0, 30.0]);

    let bp = doc.clip_path(b, target).cloned().expect("o trilho de B");
    assert_eq!(bp.len(), 2, "as âncoras nasceram onde o K clicou");
    assert!(
        (bp.length() - 30.0).abs() < 1e-6,
        "o percurso de B é o de B ({}), não o de A ({})",
        bp.length(),
        a.length()
    );
    assert_eq!(
        doc.clip_path(0, target).map(MotionPath::anchors),
        Some(a.anchors()),
        "e A ficou exatamente como estava"
    );

    // E o objeto percorre o trilho do clip ATIVO.
    apply_from_doc(&mut w, &mut doc, 1.0);
    assert!(near(pos(&w, e), [0.0, 30.0]), "{:?}", pos(&w, e));
}

/// **Reformar o trilho de um clip não toca o do outro** — a terceira camada, e ela precisa
/// de gate próprio: com a trajetória por-clip, `move_path_anchor` alcança só a do clip
/// ATIVO, e é essa restrição que o gate pina.
///
/// **Mutação que deve sangrar:** `active_path_mut` procurar a trajetória em qualquer clip.
#[test]
fn reshaping_one_clips_path_leaves_the_others_untouched() {
    let (_w, e, mut doc) = rig(MotionPath::new(Vec::new()), &[]);
    let bits = e.to_bits();
    let target = doc
        .binding_for(bits, PropKind::Position)
        .expect("bound")
        .target;

    doc.key_the_path(bits, RationalTime::from_seconds(0.0), [0.0, 0.0]);
    doc.key_the_path(bits, RationalTime::from_seconds(1.0), [10.0, 0.0]);

    let b = doc.add_clip("B".into());
    doc.set_active(b);
    doc.key_the_path(bits, RationalTime::from_seconds(0.0), [0.0, 0.0]);
    doc.key_the_path(bits, RationalTime::from_seconds(1.0), [0.0, 30.0]);
    let a_before = doc.clip_path(0, target).cloned().expect("A");

    // Arrasta a âncora de chegada de B para longe.
    let mut last = doc.path_anchor(target, 1).expect("a âncora de B");
    last.anchor = [0.0, 60.0];
    assert!(doc.move_path_anchor(target, 1, last));

    assert!(
        (doc.clip_path(b, target).unwrap().length() - 60.0).abs() < 1e-6,
        "B seguiu o dedo"
    );
    assert_eq!(
        doc.clip_path(0, target).map(MotionPath::anchors),
        Some(a_before.anchors()),
        "e A não se mexeu um texel"
    );
}

/// **Duplicar um clip leva a trajetória junto** — uma cópia da animação percorre a MESMA
/// jornada, e é a única forma de partir de um trilho pronto agora que um clip NOVO nasce
/// em branco.
///
/// **Mutação que deve sangrar:** `duplicate_clip` deixar de clonar `paths`.
#[test]
fn duplicating_a_clip_copies_its_trajectory() {
    let (_w, e, mut doc) = rig(MotionPath::new(Vec::new()), &[]);
    let bits = e.to_bits();
    let target = doc
        .binding_for(bits, PropKind::Position)
        .expect("bound")
        .target;
    doc.key_the_path(bits, RationalTime::from_seconds(0.0), [0.0, 0.0]);
    doc.key_the_path(bits, RationalTime::from_seconds(1.0), [10.0, 0.0]);

    let copy = doc.duplicate_clip(0).expect("a cópia");
    assert_eq!(
        doc.clip_path(copy, target).map(MotionPath::anchors),
        doc.clip_path(0, target).map(MotionPath::anchors),
        "a cópia percorre a mesma jornada"
    );

    // E é uma cópia INDEPENDENTE: reformar a do original não move a da cópia.
    doc.set_active(0);
    let mut a0 = doc.path_anchor(target, 1).expect("âncora");
    a0.anchor = [40.0, 0.0];
    assert!(doc.move_path_anchor(target, 1, a0));
    assert!(
        (doc.clip_path(copy, target).unwrap().length() - 10.0).abs() < 1e-6,
        "a cópia ficou onde estava"
    );
}

/// **O avaliador não perde a trajetória sob a composição** — o RECUO do `path_for`.
///
/// Sob o Arrange o clip ATIVO é o que o dropdown selecionou, e pode ser um que nunca
/// autorou trajetória; sem o recuo, um documento inteiro autorado no clip 1 e composto no
/// Arrange deixaria de mapear distância→ponto e o objeto pararia na origem.
///
/// ⚠️ Este gate e o `a_clip_created_later_starts_with_no_path_at_all` são as DUAS metades
/// da mesma decisão e por isso são dois: a leitura CRUA (`clip_path`, o desenho) não pode
/// ter recuo, e a de RESOLUÇÃO (`path_for`, o avaliador) tem de ter.
///
/// **Mutação que deve sangrar:** `path_for` largar o `or_else`.
#[test]
fn the_evaluator_still_finds_the_trajectory_from_a_clip_that_has_none() {
    let (mut w, e, mut doc) = rig(MotionPath::new(Vec::new()), &[]);
    let bits = e.to_bits();
    doc.key_the_path(bits, RationalTime::from_seconds(0.0), [0.0, 0.0]);
    doc.key_the_path(bits, RationalTime::from_seconds(1.0), [10.0, 0.0]);

    // Um clip criado depois é o ativo, e não tem trajetória própria.
    let b = doc.add_clip("B".into());
    doc.set_active(b);
    doc.upsert_key(
        bits,
        PropKind::Position,
        RationalTime::from_seconds(0.0),
        AnimValue::Float(10.0),
        Interp::Linear,
    );

    apply_from_doc(&mut w, &mut doc, 0.0);
    assert!(
        near(pos(&w, e), [10.0, 0.0]),
        "a distância keyada em B foi resolvida sobre a jornada que existe: {:?}",
        pos(&w, e)
    );
}

/// **Esquecer a track de um clip esquece a trajetória DELE** — senão a geometria fica no
/// arquivo para sempre, pendurada num alvo que já não é animado, e o próximo bind do mesmo
/// objeto herdaria a curva de uma animação que não existe mais.
///
/// **Mutação que deve sangrar:** `unbind` não chamar o `take_active_path`.
#[test]
fn unbinding_a_position_track_forgets_that_clips_trajectory() {
    let (_w, e, mut doc) = rig(MotionPath::new(Vec::new()), &[]);
    let bits = e.to_bits();
    let target = doc
        .binding_for(bits, PropKind::Position)
        .expect("bound")
        .target;
    doc.key_the_path(bits, RationalTime::from_seconds(0.0), [0.0, 0.0]);
    doc.key_the_path(bits, RationalTime::from_seconds(1.0), [10.0, 0.0]);
    assert!(doc.clip_path(0, target).is_some(), "o trilho existe");

    doc.unbind(bits, PropKind::Position);
    assert!(
        doc.clip_path(0, target).is_none(),
        "a geometria foi junto com a track que a percorria"
    );
}

/// **No Arrange, cada strip percorre a trajetória do SEU clip** (report do Enio,
/// 2026-07-30: *"só o path do clip selecionado na aba keys toca em arrange que possui
/// outros clips e outros paths em outras strips — o arrange toca apenas um clip"*).
///
/// Com a trajetória no clip, o `path_for` (que responde pelo clip ATIVO) é exato no Keys e
/// ERRADO no Arrange: as distâncias que o strip de B keya iam parar na curva de A. A cura é
/// perguntar ao STRIP que dirige (`driving_path`), e numa SEQUÊNCIA — strips que não se
/// sobrepõem, o Arrange normal — a resposta é exata em todo instante.
///
/// A cena: A vai 10 para a DIREITA, B vai 10 para CIMA, cada um no seu strip. O clip aberto
/// no Keys é o A o tempo todo, de propósito — é a variável que o report acusa.
///
/// **Mutação que deve sangrar:** o `apply` voltar a usar `path_for` sob a composição.
#[test]
fn each_strip_in_arrange_walks_its_own_clips_trajectory() {
    let (mut w, e, mut doc) = rig(MotionPath::new(Vec::new()), &[]);
    let bits = e.to_bits();

    // CLIP A: para a direita.
    doc.key_the_path(bits, RationalTime::from_seconds(0.0), [0.0, 0.0]);
    doc.key_the_path(bits, RationalTime::from_seconds(1.0), [10.0, 0.0]);

    // CLIP B: para cima, e com trajetória PRÓPRIA.
    let b = doc.add_clip("B".into());
    doc.set_active(b);
    doc.key_the_path(bits, RationalTime::from_seconds(0.0), [0.0, 0.0]);
    doc.key_the_path(bits, RationalTime::from_seconds(1.0), [0.0, 10.0]);

    // O Keys fica aberto em A — a variável que o report acusa.
    doc.set_active(0);

    // Uma faixa, dois strips em SEQUÊNCIA (sem sobreposição).
    let lane = doc.add_lane("L".into()).expect("faixa");
    doc.add_strip(lane, 0, 0.0, 1.0);
    doc.add_strip(lane, b, 2.0, 3.0);

    apply_from_doc(&mut w, &mut doc, 1.0);
    assert!(
        near(pos(&w, e), [10.0, 0.0]),
        "no strip de A o objeto percorre a curva de A: {:?}",
        pos(&w, e)
    );

    apply_from_doc(&mut w, &mut doc, 3.0);
    assert!(
        near(pos(&w, e), [0.0, 10.0]),
        "e no strip de B, a de B — nao a do clip aberto no Keys: {:?}",
        pos(&w, e)
    );
}

/// **Num crossfade entre duas curvas DIFERENTES o objeto VIAJA entre elas, sem salto**
/// (Enio, 2026-07-30: *"o Fade gera Path de transição entre um path de uma strip e outro
/// path de outra strip. Isso acaba deformando os paths de ambas as strips. O Fade precisa
/// ser similar ao modo sem Path"*).
///
/// A track de Position guarda uma DISTÂNCIA, e distância só significa algo sobre UMA curva.
/// Cruzar duas misturava números de réguas diferentes e avaliava o resultado numa curva só:
/// o objeto corria pela trajetória ERRADA durante o fade e depois saltava para a outra — o
/// *"path de transição que deforma os dois"* do report (medido: `[5.47, 0]` → `[5.0, 0]` →
/// `[0, 4.53]`, cinco unidades de salto).
///
/// Agora cada strip converte a própria distância na PRÓPRIA curva e o blend compõe
/// COORDENADAS, que é exatamente o que o modo Separate (sem Path) faz.
///
/// ⚠️ **O oráculo é a CONTINUIDADE, não um ponto** — um endpoint sozinho fica verde sobre
/// um salto. O passo máximo ao longo do cruzamento tem de ficar da ordem do passo de um
/// trecho normal; e o gate ainda exige o essencial que a geometria promete: **fora da
/// sobreposição cada strip está EXATAMENTE na sua curva**, e o meio do fade está FORA das
/// duas (é a viagem entre elas — é isso que o modo sem Path também faz).
///
/// **Mutação que deve sangrar:** `Query::axis` ser ignorado (o blend volta a compor
/// distâncias).
#[test]
fn a_crossfade_between_two_curves_travels_instead_of_jumping() {
    let (mut w, e, mut doc) = rig(MotionPath::new(Vec::new()), &[]);
    let bits = e.to_bits();
    doc.key_the_path(bits, RationalTime::from_seconds(0.0), [0.0, 0.0]);
    doc.key_the_path(bits, RationalTime::from_seconds(2.0), [10.0, 0.0]);
    let b = doc.add_clip("B".into());
    doc.set_active(b);
    doc.key_the_path(bits, RationalTime::from_seconds(0.0), [0.0, 0.0]);
    doc.key_the_path(bits, RationalTime::from_seconds(2.0), [0.0, 10.0]);
    doc.set_active(0);
    let lane = doc.add_lane("L".into()).expect("faixa");
    doc.add_strip(lane, 0, 0.0, 2.0);
    doc.add_strip(lane, b, 1.0, 3.0); // sobreposição em [1, 2]

    let mut prev: Option<[f32; 2]> = None;
    let mut worst = 0.0_f32;
    for k in 0..=48 {
        let t = f64::from(k) / 16.0;
        apply_from_doc(&mut w, &mut doc, t);
        let p = pos(&w, e);
        if let Some(q) = prev {
            let step = ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2)).sqrt();
            worst = worst.max(step);
        }
        prev = Some(p);
    }
    // Um trecho normal anda 10 unidades em 2 s a 16 amostras/s => ~0,31 por passo. O salto
    // do modelo antigo media ~5. A barra é generosa de proposito: o que ela recusa e o
    // SALTO, nao um passo de fade um pouco mais rapido.
    assert!(
        worst < 1.0,
        "o percurso tem de ser contínuo ao longo do fade; maior passo = {worst}"
    );

    // Fora da sobreposição, cada strip está EXATAMENTE na sua curva.
    apply_from_doc(&mut w, &mut doc, 0.5);
    assert!(near(pos(&w, e), [2.5, 0.0]), "{:?}", pos(&w, e));
    apply_from_doc(&mut w, &mut doc, 2.5);
    assert!(near(pos(&w, e), [0.0, 7.5]), "{:?}", pos(&w, e));

    // E o meio do fade está FORA das duas — é a viagem entre elas.
    apply_from_doc(&mut w, &mut doc, 1.5);
    let p = pos(&w, e);
    assert!(
        p[0] > 0.1 && p[1] > 0.1,
        "no meio do cruzamento o objeto não está em nenhuma das duas curvas: {p:?}"
    );
}

/// Sonda: o percurso ao longo de um crossfade entre duas curvas diferentes.
#[test]
#[ignore]
fn probe_crossfade_between_two_curves() {
    let (mut w, e, mut doc) = rig(MotionPath::new(Vec::new()), &[]);
    let bits = e.to_bits();
    doc.key_the_path(bits, RationalTime::from_seconds(0.0), [0.0, 0.0]);
    doc.key_the_path(bits, RationalTime::from_seconds(2.0), [10.0, 0.0]);
    let b = doc.add_clip("B".into());
    doc.set_active(b);
    doc.key_the_path(bits, RationalTime::from_seconds(0.0), [0.0, 0.0]);
    doc.key_the_path(bits, RationalTime::from_seconds(2.0), [0.0, 10.0]);
    doc.set_active(0);
    let lane = doc.add_lane("L".into()).expect("faixa");
    doc.add_strip(lane, 0, 0.0, 2.0);
    doc.add_strip(lane, b, 1.0, 3.0);
    for k in 0..=12 {
        let t = f64::from(k) * 0.25;
        apply_from_doc(&mut w, &mut doc, t);
        println!("t={t:.2} pos={:?}", pos(&w, e));
    }
}

/// SONDA: a GEOMETRIA muda quando o Arrange toca sobre um fade?
#[test]
#[ignore]
fn probe_does_the_fade_write_geometry() {
    let (mut w, e, mut doc) = rig(MotionPath::new(Vec::new()), &[]);
    let bits = e.to_bits();
    doc.key_the_path(bits, RationalTime::from_seconds(0.0), [0.0, 0.0]);
    doc.key_the_path(bits, RationalTime::from_seconds(2.0), [10.0, 0.0]);
    let b = doc.add_clip("B".into());
    doc.set_active(b);
    doc.key_the_path(bits, RationalTime::from_seconds(0.0), [0.0, 0.0]);
    doc.key_the_path(bits, RationalTime::from_seconds(2.0), [0.0, 10.0]);
    doc.set_active(0);
    let lane = doc.add_lane("L".into()).expect("faixa");
    doc.add_strip(lane, 0, 0.0, 2.0);
    doc.add_strip(lane, b, 1.0, 3.0);
    let target = doc.binding_for(bits, PropKind::Position).unwrap().target;

    let snap = |d: &TimelineDoc| -> Vec<Vec<[f32; 2]>> {
        (0..d.clips().len())
            .map(|c| {
                d.clip_path(c, target)
                    .map(|p| p.anchors().iter().map(|a| a.anchor).collect())
                    .unwrap_or_default()
            })
            .collect()
    };
    let before = snap(&doc);
    println!("ANTES  {before:?}");
    for k in 0..=48 {
        apply_from_doc(&mut w, &mut doc, f64::from(k) / 16.0);
    }
    let after = snap(&doc);
    println!("DEPOIS {after:?}");
    println!(
        "GEOMETRIA MUDOU? {}",
        if before == after { "NAO" } else { "SIM" }
    );
}

/// **O AUTOKEY NÃO PODE PLANTAR ÂNCORA SOBRE A POSE QUE O PRÓPRIO APPLY ESCREVEU**
/// (report do Enio, 2026-07-30, terceira rodada: *"o Fade gera Path de transição entre um
/// path de uma strip e outro path de outra strip. Isso acaba deformando os paths de ambas
/// as strips"*).
///
/// O `autokey_props` compara a pose do MUNDO com a pose que o documento diz — e keya
/// quando elas diferem, porque diferença significa *"o artista moveu o objeto"*. Num canal
/// de trajetória isso planta uma ÂNCORA (`AutokeyPlan::path_key` → `key_the_path`), ou
/// seja **geometria nova**. Durante um fade o apply põe o objeto entre as duas curvas; se o
/// lado que LÊ reconstruir a pose de outra maneira, todo frame do cruzamento vira uma
/// âncora — a *"curva de transição"* do report, e ela deforma o caminho que a recebe.
///
/// É a lição [[feedback_derived_coordinate_seed_must_match_sample]], e o doc do
/// `autokey_props` já a cita: *"whatever writes a derived coordinate and whatever reads it
/// must be the SAME function"*. O que quebrou a igualdade foi a composição de PONTOS: o
/// apply parou de escrever `path.at(distância)` sob a pilha, e o leitor não soube.
///
/// **Mutação que deve sangrar:** `position_shown` voltar a reconstruir a pose como
/// `path_for(...).at(distância composta)`.
#[test]
fn the_autokey_plants_no_anchor_on_the_pose_the_apply_itself_wrote() {
    let (mut w, e, mut doc) = rig(MotionPath::new(Vec::new()), &[]);
    let bits = e.to_bits();
    doc.key_the_path(bits, RationalTime::from_seconds(0.0), [0.0, 0.0]);
    doc.key_the_path(bits, RationalTime::from_seconds(2.0), [10.0, 0.0]);
    let b = doc.add_clip("B".into());
    doc.set_active(b);
    doc.key_the_path(bits, RationalTime::from_seconds(0.0), [0.0, 0.0]);
    doc.key_the_path(bits, RationalTime::from_seconds(2.0), [0.0, 10.0]);
    doc.set_active(0);
    let lane = doc.add_lane("L".into()).expect("faixa");
    doc.add_strip(lane, 0, 0.0, 2.0);
    doc.add_strip(lane, b, 1.0, 3.0); // sobreposição em [1, 2] = o fade

    // Varre o cruzamento inteiro. Em CADA instante: o apply escreve a pose, e o autokey é
    // perguntado sobre EXATAMENTE essa pose — o artista não tocou em nada.
    let mut planted = Vec::new();
    for k in 0..=32 {
        let t = 1.0 + f64::from(k) / 32.0;
        apply_from_doc(&mut w, &mut doc, t);
        let p = pos(&w, e);
        let pose: ph2d_timeline::PoseSample =
            [Some(p[0]), Some(p[1]), None, None, None, None, None];
        let plan = ph2d_timeline::autokey_props(&doc, bits, t, &pose, &pose, false, false);
        if let Some(at) = plan.path_key {
            planted.push((t, at));
        }
    }
    assert!(
        planted.is_empty(),
        "o autokey plantou {} âncora(s) sobre a pose que o apply acabou de escrever — \
         é a CURVA DE TRANSIÇÃO do report. Primeiras: {:?}",
        planted.len(),
        &planted[..planted.len().min(4)]
    );
}
