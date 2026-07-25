//! **Auto-orient** (ADR-0141 §6): o objeto encara a tangente do caminho — opt-in, com
//! a recusa NOMEADA e o ângulo que se segura onde não há direção.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Transform, World};
use ph2d_timeline::{
    AutoOrient, MotionPath, PathAnchor, PropKind, TangentKind, TimelineDoc, apply_from_doc,
};

/// Um quarto de volta: sai andando para +X e termina a subir em +Y, então o ângulo
/// tem de varrer de 0 a π/2. Uma reta não distinguiria "girou" de "não girou".
fn quarter_turn() -> MotionPath {
    MotionPath::new(vec![
        PathAnchor {
            anchor: [0.0, 0.0],
            in_handle: [0.0, 0.0],
            out_handle: [6.0, 0.0],
            auto: false,
            kind: TangentKind::Smooth,
        },
        PathAnchor {
            anchor: [10.0, 10.0],
            in_handle: [0.0, -6.0],
            out_handle: [0.0, 0.0],
            auto: false,
            kind: TangentKind::Smooth,
        },
    ])
}

fn rig(path: MotionPath) -> (World, Entity, TimelineDoc) {
    let mut w = World::new();
    let e = w.spawn(Transform::from_translation(Vec2::ZERO)).id();
    let mut doc = TimelineDoc::new();
    doc.bind(e.to_bits(), PropKind::Position);
    let total = path.length() as f32;
    for (t, s) in [(0.0, 0.0_f32), (2.0, total)] {
        doc.insert_key(
            e.to_bits(),
            PropKind::Position,
            RationalTime::from_seconds(t),
            AnimValue::Float(s),
            Interp::Linear,
        );
    }
    doc.bindings_mut()[0].path = Some(path);
    (w, e, doc)
}

fn rot(w: &World, e: Entity) -> f32 {
    w.get::<Transform>(e).unwrap().rotation
}

/// **Opt-in**: sem pedir, nada gira. Girar um objeto sem que ninguém peça reescreve a
/// pose que o artista autorou, e a rotação é dele até dizer o contrário.
#[test]
fn nothing_turns_until_it_is_asked_for() {
    let (mut w, e, mut doc) = rig(quarter_turn());
    w.get_mut::<Transform>(e).unwrap().rotation = 1.234;
    assert_eq!(doc.auto_orient(e.to_bits()), AutoOrient::Off);

    apply_from_doc(&mut w, &mut doc, 1.0);
    assert_eq!(rot(&w, e), 1.234, "a rotação autorada foi sobrescrita");

    assert_eq!(doc.set_auto_orient(e.to_bits(), true), AutoOrient::Active);
    apply_from_doc(&mut w, &mut doc, 1.0);
    assert_ne!(rot(&w, e), 1.234, "pedido e não girou");
}

/// **O objeto encara para onde vai.** No começo do quarto de volta ele aponta para +X
/// (0 rad) e no fim para +Y (π/2) — e o oráculo é o ÂNGULO, não "mudou".
#[test]
fn the_object_faces_the_way_the_path_goes() {
    let (mut w, e, mut doc) = rig(quarter_turn());
    doc.set_auto_orient(e.to_bits(), true);

    apply_from_doc(&mut w, &mut doc, 0.0);
    let start = rot(&w, e);
    apply_from_doc(&mut w, &mut doc, 2.0);
    let end = rot(&w, e);
    println!("MEDIDO  inicio {start:.6} rad, fim {end:.6} rad");
    assert!(start.abs() < 1e-5, "no começo a curva vai para +X: {start}");
    assert!(
        (end - std::f32::consts::FRAC_PI_2).abs() < 1e-5,
        "no fim ela sobe em +Y (π/2): {end}"
    );
    // E no meio, algo estritamente entre os dois — o giro é contínuo, não um salto.
    apply_from_doc(&mut w, &mut doc, 1.0);
    let mid = rot(&w, e);
    assert!((0.05..1.5).contains(&mid), "meio do giro: {mid}");
}

/// ⚠️ **A RECUSA, nomeada.** Girar para a tangente escreve `Transform.rotation`, que é
/// o que uma track de Rotation escreve — dois autores do mesmo campo, e o de trás vence
/// em silêncio. Resolver por ORDEM seria a pior maneira: funcionaria, ninguém saberia
/// por quê, e inverter a ordem um dia mudaria a animação de alguém sem nada no diff.
#[test]
fn a_rotation_track_blocks_the_auto_orient_and_says_so() {
    let (mut w, e, mut doc) = rig(quarter_turn());
    doc.set_auto_orient(e.to_bits(), true);
    doc.insert_key(
        e.to_bits(),
        PropKind::Rotation,
        RationalTime::from_seconds(0.0),
        AnimValue::Float(0.75),
        Interp::Hold,
    );

    assert_eq!(
        doc.auto_orient(e.to_bits()),
        AutoOrient::BlockedByRotationTrack,
        "a recusa tem de ter NOME — um toggle ligado sem efeito é pior que um desligado"
    );
    apply_from_doc(&mut w, &mut doc, 2.0);
    assert!(
        (rot(&w, e) - 0.75).abs() < 1e-5,
        "a track de Rotation perdeu o campo dela para o auto-orient: {}",
        rot(&w, e)
    );
    // E o pedido do artista SOBREVIVE: apagar a track de Rotation o devolve, em vez de
    // o exigir de novo.
    doc.unbind(e.to_bits(), PropKind::Rotation);
    assert_eq!(doc.auto_orient(e.to_bits()), AutoOrient::Active);
}

/// ⚠️ **Numa CÚSPIDE o ângulo se SEGURA, e isso é o bug publicado do AE não
/// acontecendo** (*"flips when stopping motion"*).
///
/// Lá o ângulo vem do vetor VELOCIDADE, que some quando o objeto para; aqui vem da
/// GEOMETRIA da curva, que continua lá com o objeto parado em cima dela. O único lugar
/// sem direção é uma cúspide de verdade — e ali nada é escrito, o que É segurar o
/// último ângulo válido, **sem estado nenhum** para guardar nem invalidar num scrub.
#[test]
fn a_cusp_holds_the_last_angle_instead_of_inventing_one() {
    // Um caminho que PARA: as duas âncoras coincidem, então a velocidade da curva é
    // zero em toda parte e não existe tangente em lugar nenhum.
    let dead = MotionPath::new(vec![
        PathAnchor::corner([4.0, 4.0]),
        PathAnchor::corner([4.0, 4.0]),
    ]);
    let (mut w, e, mut doc) = rig(dead);
    doc.set_auto_orient(e.to_bits(), true);
    w.get_mut::<Transform>(e).unwrap().rotation = 0.9;

    for t in [0.0, 0.5, 1.0, 2.0] {
        apply_from_doc(&mut w, &mut doc, t);
        assert_eq!(
            rot(&w, e),
            0.9,
            "em t={t} um ângulo foi INVENTADO onde não há direção — é o pico solto"
        );
    }
    // ...e a posição continua a ser escrita: só o ângulo se abstém.
    assert_eq!(
        [
            w.get::<Transform>(e).unwrap().translation.x,
            w.get::<Transform>(e).unwrap().translation.y
        ],
        [4.0, 4.0]
    );
}

/// **Andar para trás não vira o objeto**, e é decisão, não omissão.
///
/// O ângulo vem da GEOMETRIA da curva — que tem um sentido próprio — e não do vetor
/// velocidade. Um flip automático de 180° no meio de uma track é uma descontinuidade
/// que o artista não controla; querer virar é shapear o caminho ou autorar Rotation
/// (que então RECUSA o auto-orient, e é o mesmo pedido dito uma vez só).
#[test]
fn travelling_backwards_does_not_flip_the_object() {
    let (mut w, e, mut doc) = rig(quarter_turn());
    doc.set_auto_orient(e.to_bits(), true);
    apply_from_doc(&mut w, &mut doc, 0.5);
    let going = rot(&w, e);

    // A MESMA trajetória, percorrida ao contrário.
    let target = doc.bindings()[0].target;
    let total = doc.position_path(e.to_bits()).unwrap().length() as f32;
    let ids: Vec<_> = doc.active_clip().track(target).unwrap().ids().to_vec();
    let track = doc.active_clip_mut().track_mut(target).unwrap();
    track.set_value(ids[0], AnimValue::Float(total));
    track.set_value(ids[1], AnimValue::Float(0.0));

    apply_from_doc(&mut w, &mut doc, 1.5);
    let coming = rot(&w, e);
    println!("MEDIDO  indo {going:.6} rad, voltando {coming:.6} rad");
    assert!(
        (going - coming).abs() < 1e-5,
        "o objeto virou ao andar para trás ({going} -> {coming}) — o ângulo passou a \
         vir da VELOCIDADE em vez da geometria, e com ele volta o flip do AE"
    );
}
