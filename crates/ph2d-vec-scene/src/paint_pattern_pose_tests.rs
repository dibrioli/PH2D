//! ⭐⭐ **O PADRÃO CAVALGA A POSE DA FORMA** — os gates do assunto (plano 33).
//!
//! Irmão do [`super::paint_pattern_tests`], e o corte é por RESPONSABILIDADE: aquele mede o **dado**
//! (o tamanho do `Paint`, o período da colmeia, a serialização, o enquadramento do `Clamp`); este
//! mede o que acontece ao padrão quando a **forma** se move, roda ou escala.
//!
//! ⚠️ É a lei que o `paint.rs` já escrevia para os gradientes — *a geometria de um preenchimento
//! vive no espaço das ÂNCORAS e transforma junto com o path* —, e o padrão é o único preenchimento
//! desta casa que conserva também a **orientação**.

use super::paint_pattern_tests::fill;
use super::{Paint, VecPath, VecPathId, VecScene, VecVertex};

/// Uma forma quadrada com o padrão de referência já vestido.
fn scene_with_pattern(f: super::PatternFill) -> (VecScene, VecPathId) {
    let mut scene = VecScene::default();
    let id = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: Some(Paint::Pattern(Box::new(f))),
        ..VecPath::default()
    });
    (scene, id)
}

/// ⭐⭐ **O padrão CONSERVA A ORIENTAÇÃO quando a forma roda** — e é o único preenchimento desta
/// casa que o faz.
///
/// O gradiente radial não pode: um radial do peniko **é circular** e não tem onde guardar um
/// ângulo, e é por isso que o `transform_fill_geometry` lhe passa um `radius_scale` médio. O padrão
/// tem o campo, então a sonda do afim (as imagens dos dois eixos unitários) dá-lhe a resposta exacta.
#[test]
fn rotating_the_shape_rotates_the_pattern_with_it() {
    let (mut scene, id) = scene_with_pattern(fill());
    let quarter = std::f64::consts::FRAC_PI_2;
    assert!(scene.rotate_path_by(id, quarter, [0.0, 0.0]));
    let Some(Paint::Pattern(p)) = &scene.paths()[0].fill else {
        panic!("especie trocada")
    };
    assert!(
        (p.angle - quarter).abs() < 1e-9,
        "o padrao nao rodou com a forma: {}",
        p.angle
    );
    assert!(
        (p.size[0] - 10.0).abs() < 1e-9 && (p.size[1] - 20.0).abs() < 1e-9,
        "uma rotacao nao pode mudar o tamanho do ladrilho: {:?}",
        p.size
    );
}

/// **Escalar a forma escala o ladrilho, POR EIXO** — e a escala não-uniforme é o fenómeno que a
/// fixtura tinha de conter (plano 33 §5.1).
#[test]
fn scaling_the_shape_scales_the_tile_per_axis() {
    let (mut scene, id) = scene_with_pattern(fill());
    assert!(scene.scale_path(id, 3.0, 0.5, [0.0, 0.0]));
    let Some(Paint::Pattern(p)) = &scene.paths()[0].fill else {
        panic!("especie trocada")
    };
    assert!(
        (p.size[0] - 30.0).abs() < 1e-9 && (p.size[1] - 10.0).abs() < 1e-9,
        "o ladrilho nao seguiu a escala por eixo: {:?}",
        p.size
    );
    assert!(
        p.angle.abs() < 1e-9,
        "uma escala positiva nao pode rodar o padrao"
    );
}

/// **Mover a forma move o padrão com ela** — a lei que o `paint.rs` já escreveu para os gradientes,
/// e o oposto do defeito da origem-da-régua do Illustrator.
#[test]
fn moving_the_shape_moves_the_pattern_with_it() {
    let (mut scene, id) = scene_with_pattern(fill());
    assert!(scene.translate_path(id, 7.0, -3.0));
    let Some(Paint::Pattern(p)) = &scene.paths()[0].fill else {
        panic!("especie trocada")
    };
    assert_eq!(p.origin, [7.0, -3.0]);
    assert!(p.angle.abs() < 1e-9 && (p.size[0] - 10.0).abs() < 1e-9);
}

/// ⭐⭐⭐ **A ESTAMPA DO CONTORNO cavalga a pose igual à do preenchimento** (auditoria de
/// 2026-08-30).
///
/// ⛔ A lei do padrão estava escrita **só para `path.fill`**: rodar ou escalar uma forma cujo
/// CONTORNO tem estampa deixava-a exactamente onde estava, enquanto a do preenchimento seguia. Uma
/// forma com as **duas** tintas estampadas rodava com metade do desenho.
///
/// ⚠️ **A régua compara as DUAS tintas na MESMA forma, e é isso que a torna um oráculo.** Afirmar
/// só o ângulo do traço mediria a minha aritmética; afirmar que ele é **igual ao do preenchimento**
/// mede a propriedade — *as duas tintas são a mesma lei* — e continua a valer se a lei mudar.
#[test]
fn the_strokes_pattern_rides_the_pose_exactly_like_the_fills() {
    let mut scene = VecScene::default();
    let id = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: Some(Paint::Pattern(Box::new(fill()))),
        stroke: Some({
            let mut s = crate::StrokeSpec::new(crate::Rgba8::new(9, 9, 9, 255), 1.0);
            s.paint = crate::StrokePaint::Pattern(Box::new(fill()));
            s
        }),
        ..VecPath::default()
    });

    let quarter = std::f64::consts::FRAC_PI_2;
    assert!(scene.rotate_path_by(id, quarter, [0.0, 0.0]));
    assert!(scene.scale_path(id, 3.0, 0.5, [0.0, 0.0]));

    let path = &scene.paths()[0];
    let Some(Paint::Pattern(do_fill)) = &path.fill else {
        panic!("o preenchimento trocou de especie")
    };
    let do_traco = path
        .stroke
        .as_ref()
        .and_then(crate::StrokeSpec::pattern)
        .expect("o traco continua estampado");

    assert!(
        (do_traco.angle - do_fill.angle).abs() < 1e-12,
        "o angulo do traco ({}) ficou para tras do preenchimento ({}) - a estampa do contorno nao \
         seguiu a forma",
        do_traco.angle,
        do_fill.angle
    );
    assert!(
        (do_traco.size[0] - do_fill.size[0]).abs() < 1e-12
            && (do_traco.size[1] - do_fill.size[1]).abs() < 1e-12,
        "o tamanho do ladrilho do traco {:?} divergiu do preenchimento {:?}",
        do_traco.size,
        do_fill.size
    );
    assert!(
        (do_traco.origin[0] - do_fill.origin[0]).abs() < 1e-12
            && (do_traco.origin[1] - do_fill.origin[1]).abs() < 1e-12,
        "a origem do traco {:?} divergiu do preenchimento {:?}",
        do_traco.origin,
        do_fill.origin
    );
    // ⚠️ E o CONTROLO: a pose de facto mudou. Sem isto, duas estampas paradas passariam iguais.
    assert!(
        (do_fill.angle - quarter).abs() > 1e-12 || do_fill.size[0] > 10.0,
        "a pose nao se mexeu - o gate compara duas coisas paradas e aprova-se sozinho"
    );
}
