//! **A cena `PH2D_PATH_SMOKE=1` afirma números; aqui eles são MEDIDOS.**
//!
//! Duas cenas desta jornada de física afirmaram coisas que a medição desmentiu, e a
//! regra que saiu dali vale para toda cena nova: a sonda headless roda ANTES de a
//! mensagem ser escrita. O que o prólogo anuncia é o que sai daqui.

use super::{author, demo_path};
use ph2d_anim::{AnimValue, AttributeEvaluator};
use ph2d_timeline::{PropKind, TimelineDoc};

/// Onde o objeto está no instante `t`, pela MESMA composição que o apply faz.
fn at(doc: &TimelineDoc, bits: u64, t: f64) -> [f32; 2] {
    let b = doc
        .bindings()
        .iter()
        .find(|b| b.entity == bits && b.prop == PropKind::Position)
        .expect("binding");
    let track = doc.active_clip().track(b.target).expect("track");
    let AnimValue::Float(s) = track.sample(t) else {
        panic!("a track de Position é escalar")
    };
    b.path
        .as_ref()
        .expect("caminho")
        .at(f64::from(s))
        .expect("ponto")
        .point
}

fn rig() -> (TimelineDoc, u64) {
    let bits = 7_u64;
    let mut doc = TimelineDoc::new();
    author(&mut doc, bits, &demo_path());
    (doc, bits)
}

/// Os dois números do prólogo: quantas âncoras e quanto percurso.
#[test]
fn the_scene_says_what_it_built() {
    let path = demo_path();
    println!(
        "MEDIDO  {} ancoras, {:.4} unidades de percurso",
        path.len(),
        path.length()
    );
    assert_eq!(path.len(), 4);
    // Medido: 17,6734. A corda de ponta a ponta mede 12,6491, então o S de fato
    // desvia — 40% a mais de percurso do que a linha reta entre as pontas.
    assert!(
        (path.length() - 17.6734).abs() < 0.01,
        "o percurso mudou: {:.4}",
        path.length()
    );
    assert!(
        path.length() > 13.9,
        "um S que mede quase a corda (12,65) é uma RETA, e a cena não demonstra nada \
         sobre forma"
    );
}

/// **O que a cena manda olhar tem de estar lá.** O ease existe para que o espaçamento
/// dos pontos DIGA alguma coisa; se ele for fraco, a instrução do prólogo (*"juntos
/// nas pontas, esparramados no meio"*) é uma promessa que a tela não cumpre.
#[test]
fn the_dots_really_do_bunch_at_the_ends_and_spread_in_the_middle() {
    let (doc, bits) = rig();
    let fps = doc.fps_display;
    let frames = (3.0 * fps).round() as usize;
    let centres: Vec<[f32; 2]> = (0..=frames)
        .map(|k| at(&doc, bits, 3.0 * k as f64 / frames as f64))
        .collect();
    let gaps: Vec<f32> = centres
        .windows(2)
        .map(|w| ((w[1][0] - w[0][0]).powi(2) + (w[1][1] - w[0][1]).powi(2)).sqrt())
        .collect();
    let n = gaps.len();
    let edge = (gaps[0] + gaps[n - 1]) / 2.0;
    let middle = gaps[n / 2];
    println!(
        "MEDIDO  vao nas pontas {edge:.4}, no meio {middle:.4} ({:.0}x)",
        middle / edge
    );
    assert!(
        middle > 8.0 * edge,
        "os pontos do meio ({middle:.4}) mal se esparramam contra os das pontas \
         ({edge:.4}) — o prólogo manda olhar uma coisa que não está lá"
    );
}

/// A cena começa e termina **nas pontas do caminho**: o objeto que o smoke posiciona
/// em `(-6, -2)` é o mesmo ponto da primeira âncora, então nada salta no frame 0.
#[test]
fn the_journey_runs_from_end_to_end_with_no_jump_at_the_start() {
    let (doc, bits) = rig();
    let path = demo_path();
    let start = at(&doc, bits, 0.0);
    let end = at(&doc, bits, 3.0);
    let d = |a: [f32; 2], b: [f32; 2]| ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt();
    assert!(
        d(start, path.anchors()[0].anchor) < 1e-4,
        "o percurso começa em {start:?}, e o sprite é posto em (-6, -2)"
    );
    assert!(
        d(end, path.anchors()[path.len() - 1].anchor) < 1e-3,
        "o percurso termina em {end:?}, não na última âncora"
    );
}

/// **A cena `=2` mostra DUAS coisas diferentes, e a sonda confere que são diferentes.**
///
/// Um par (o que gira × o recusado) é o que torna "recusado" visível; se os dois
/// resolvessem igual, o prólogo estaria a mandar olhar uma diferença que não existe.
#[test]
fn the_orient_scene_really_puts_an_active_one_next_to_a_refused_one() {
    use ph2d_anim::{Interp, RationalTime};
    use ph2d_timeline::AutoOrient;

    let (follower, blocked) = (11_u64, 12_u64);
    let mut doc = TimelineDoc::new();
    let path = demo_path();
    for bits in [follower, blocked] {
        author(&mut doc, bits, &path);
        doc.set_auto_orient(bits, true);
    }
    doc.insert_key(
        blocked,
        PropKind::Rotation,
        RationalTime::from_seconds(0.0),
        AnimValue::Float(0.0),
        Interp::Hold,
    );

    assert_eq!(doc.auto_orient(follower), AutoOrient::Active);
    assert_eq!(
        doc.auto_orient(blocked),
        AutoOrient::BlockedByRotationTrack,
        "as duas setas resolvem igual — a cena manda olhar uma diferença que não existe"
    );

    // E o que a cena promete VER: a laranja vira ao longo do S. O ângulo tem de varrer
    // um arco de verdade, senão "ela encara para onde vai" é invisível num S suave.
    let mut w = ph2d_ecs::World::new();
    let e = w.spawn(ph2d_ecs::Transform::default()).id();
    let mut solo = TimelineDoc::new();
    author(&mut solo, e.to_bits(), &path);
    solo.set_auto_orient(e.to_bits(), true);
    let angles: Vec<f32> = (0..=6)
        .map(|k| {
            ph2d_timeline::apply_from_doc(&mut w, &mut solo, 3.0 * f64::from(k) / 6.0);
            w.get::<ph2d_ecs::Transform>(e).unwrap().rotation
        })
        .collect();
    let sweep = angles.iter().copied().fold(f32::MIN, f32::max)
        - angles.iter().copied().fold(f32::MAX, f32::min);
    println!("MEDIDO  o giro varre {:.1}°", sweep.to_degrees());
    assert!(
        sweep.to_degrees() > 60.0,
        "a seta vira só {:.1}° ao longo do S — o giro não se vê",
        sweep.to_degrees()
    );
}
