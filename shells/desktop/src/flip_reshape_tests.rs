//! Testes da FRONTEIRA da escultura (o que o solver não pode provar sozinho): a
//! conversão de unidades e o roteamento dos oito pincéis. Módulo-irmão pelo cap de LOC.

use super::*;
use ph2d_tool_flip::ReshapeKind as ToolKind;

fn style(kind: ToolKind, width_px: f64, strength: f32) -> FlipStyleSnapshot {
    FlipStyleSnapshot {
        mode: ph2d_tool_flip::FlipMode::Reshape,
        reshape: kind,
        width_px,
        opacity: strength,
        ..Default::default()
    }
}

/// **O raio do pincel é METADE do Size, em unidades LOCAIS.**
///
/// Três conversões se compõem aqui, e nenhuma delas é opcional: o Size é o DIÂMETRO
/// em px de tela (como na borracha), o zoom da câmera dá `px_to_world`, e a escala do
/// objeto (ADR-0111) recua o mundo para o espaço local do desenho. Errar qualquer uma
/// dá um pincel que não alcança nada (ou que alcança o desenho inteiro) — e foi
/// exatamente essa classe de erro que matou o balde no produto (BUGS #10a).
#[test]
fn the_brush_radius_is_half_the_size_in_local_units() {
    // Câmera default: 10 unidades de mundo na altura de uma janela de 1080p.
    let px_to_world = 10.0f32 / 1080.0;
    let s = style(ToolKind::Smooth, 40.0, 1.0);

    // Objeto na identidade: raio = 20 px de tela em unidades de mundo.
    let p = params_from(&s, px_to_world, &Xform::IDENTITY, false);
    let want = 20.0 * px_to_world;
    assert!(
        (p.radius - want).abs() < 1e-6,
        "raio {} != {want} (metade do Size, em mundo)",
        p.radius
    );
    assert!((p.px_to_local - px_to_world).abs() < 1e-9);

    // Objeto ESCALADO 2× pelo gizmo: o `world_to_local` dele encolhe por 0,5, então o
    // MESMO pincel de 40 px de tela cobre METADE das unidades locais.
    let scaled = Xform([0.5, 0.0, 0.0, 0.5, 0.0, 0.0]);
    let p2 = params_from(&s, px_to_world, &scaled, false);
    assert!(
        (p2.radius - want * 0.5).abs() < 1e-6,
        "a escala do objeto tem de recuar o raio: {} != {}",
        p2.radius,
        want * 0.5
    );
}

/// A força do pincel é o **Strength** do painel (o `opacity` da tool — a borracha faz
/// igual), e o Ctrl é o invert.
#[test]
fn strength_and_ctrl_reach_the_solver() {
    let s = style(ToolKind::Thickness, 20.0, 0.25);
    let p = params_from(&s, 0.01, &Xform::IDENTITY, false);
    assert!((p.strength - 0.25).abs() < 1e-6, "a forca nao chegou");
    assert!(!p.invert);
    let p = params_from(&s, 0.01, &Xform::IDENTITY, true);
    assert!(p.invert, "o Ctrl nao chegou ao solver");
}

/// **Os oito pincéis chegam ao solver — todos.** Um `match` de tradução com um braço
/// errado é invisível para o compilador (os dois enums têm os mesmos nomes) e o
/// usuário só descobre clicando: escolhe Twist e o traço engrossa. Aqui cada variante
/// da tool é mapeada e conferida contra a do solver, uma a uma.
#[test]
fn every_tool_brush_maps_to_its_own_solver_brush() {
    let pairs = [
        (ToolKind::Smooth, ReshapeKind::Smooth),
        (ToolKind::Push, ReshapeKind::Push),
        (ToolKind::Grab, ReshapeKind::Grab),
        (ToolKind::Pinch, ReshapeKind::Pinch),
        (ToolKind::Twist, ReshapeKind::Twist),
        (ToolKind::Thickness, ReshapeKind::Thickness),
        (ToolKind::Strength, ReshapeKind::Strength),
        (ToolKind::Randomize, ReshapeKind::Randomize),
    ];
    for (tool, solver) in pairs {
        let p = params_from(&style(tool, 20.0, 1.0), 0.01, &Xform::IDENTITY, false);
        assert_eq!(p.kind, solver, "{tool:?} caiu no pincel errado");
    }
    // E o mapa é uma BIJEÇÃO: oito pincéis distintos, nenhum colapsado no vizinho.
    let mut kinds: Vec<ReshapeKind> = pairs.iter().map(|(t, _)| kind_of(*t)).collect();
    kinds.sort_by_key(|k| format!("{k:?}"));
    kinds.dedup();
    assert_eq!(
        kinds.len(),
        8,
        "dois pinceis da tool caem no MESMO do solver"
    );
}

/// O falloff multiframe sai daqui em `1.0` (a tira seleciona um quadro só) — e não
/// em `0.0`, que zeraria toda influência e faria os oito pincéis não fazerem nada.
#[test]
fn the_multiframe_falloff_defaults_to_the_active_frame() {
    let p = params_from(
        &style(ToolKind::Push, 20.0, 1.0),
        0.01,
        &Xform::IDENTITY,
        false,
    );
    assert_eq!(p.frame_falloff, 1.0);
}

// ─────────────────────────────────────────────────────────────────────────────
// A COSTURA COM O DOCUMENTO (o que `params_from` sozinho não prova): o autokey.
// ─────────────────────────────────────────────────────────────────────────────

use ph2d_core::Playhead;
use ph2d_flip::{FlipDoc, FlipStroke, Hold, KeyKind, Point, Rgba};

/// Um documento com uma linha reta de 5 pontos numa chave no quadro 0.
fn doc_with_line() -> (FlipDoc, LayerId) {
    let mut doc = FlipDoc::new();
    let oid = doc.push_object("O");
    let obj = doc.object_mut(oid).unwrap();
    let l = obj.add_layer("L");
    let d = obj
        .insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe)
        .unwrap();
    let mut s = FlipStroke::new();
    for x in 0..5 {
        s.push_point(Point {
            pos: Vec2::new(x as f32, 0.0),
            width: 6.0,
            opacity: 1.0,
            color: Rgba::WHITE,
        });
    }
    obj.drawing_mut(d).unwrap().strokes.push(s);
    (doc, l)
}

fn strokes_at(doc: &FlipDoc, l: LayerId, frame: i32) -> Vec<FlipStroke> {
    let obj = doc.objects().first().unwrap();
    let did = obj.layer(l).unwrap().drawing_at(frame).unwrap();
    obj.drawing(did).unwrap().strokes.clone()
}

/// **O gesto muda o desenho de verdade** (e não só o solver em memória): um Push no
/// meio da linha empurra os pontos do desenho ATIVO.
#[test]
fn a_sculpt_gesture_edits_the_active_drawing() {
    let (mut doc, l) = doc_with_line();
    let p = ReshapeParams {
        kind: ReshapeKind::Push,
        radius: 2.0,
        strength: 1.0,
        ..Default::default()
    };
    let strip = crate::flip_strip::FlipStrip::default();
    let ph = Playhead::new(1.0 / f64::from(ph2d_flip::DEFAULT_FPS));
    let s = InputSample {
        pos: Vec2::new(2.0, 0.0),
        delta: Vec2::new(0.0, 1.0),
        pressure: 1.0,
    };
    let (_, _, mut sess) =
        reshape_begin(&mut doc, &ph, Some(l), &strip, &p, &s).expect("ha desenho na camada ativa");
    assert!(
        strokes_at(&doc, l, 0)[0].positions()[2].y > 0.5,
        "o pen-down ja esculpe"
    );

    // E um move continua o gesto no MESMO desenho.
    let oid = doc.objects().first().unwrap().id;
    let did = doc
        .objects()
        .first()
        .unwrap()
        .layer(l)
        .unwrap()
        .drawing_at(0)
        .unwrap();
    reshape_sample(&mut doc, (oid, did), &mut sess, &p, &s);
    assert!(
        strokes_at(&doc, l, 0)[0].positions()[2].y > 1.0,
        "a 2a amostra tinha de somar (a dose e por AMOSTRA)"
    );
}

/// **Uma camada TRAVADA recusa a escultura** — e o chamador tem como saber (o `None`
/// vira o toast). Sem isto, o pincel "funcionaria" numa camada travada e o cadeado
/// seria decorativo.
#[test]
fn a_locked_layer_refuses_the_sculpt() {
    let (mut doc, l) = doc_with_line();
    let oid = doc.objects().first().unwrap().id;
    doc.object_mut(oid).unwrap().layer_mut(l).unwrap().locked = true;

    let p = ReshapeParams {
        kind: ReshapeKind::Push,
        radius: 5.0,
        strength: 1.0,
        ..Default::default()
    };
    let s = InputSample {
        pos: Vec2::new(2.0, 0.0),
        delta: Vec2::new(0.0, 1.0),
        pressure: 1.0,
    };
    let got = reshape_begin(
        &mut doc,
        &Playhead::new(1.0 / f64::from(ph2d_flip::DEFAULT_FPS)),
        Some(l),
        &crate::flip_strip::FlipStrip::default(),
        &p,
        &s,
    );
    assert!(got.is_none(), "a camada travada tinha de RECUSAR");
    assert_eq!(
        strokes_at(&doc, l, 0)[0].positions()[2],
        Vec2::new(2.0, 0.0),
        "e nada pode ter sido esculpido"
    );
}

/// **O autokey da escultura é `Modify` — a chave nova nasce DUPLICATA, nunca em
/// branco.**
///
/// É a regra que a W3 pagou caro para aprender (`docs/Flip/05 §4`): a caneta cria
/// chave em branco; borracha e ESCULTURA duplicam. Se o Reshape criasse uma chave
/// vazia no rabo de um hold, o usuário esculpiria o nada — enquanto o desenho que ele
/// VÊ continuaria intacto num quadro anterior.
///
/// Aqui: uma chave só, no quadro 0, com hold; o playhead está no quadro 5 (dentro do
/// hold, sem chave própria) e o AutoKey está ARMADO. Esculpir tem de produzir uma
/// chave nova no 5 **com o traço dentro** — e esculpido.
#[test]
fn sculpting_inside_a_hold_duplicates_the_visible_drawing_never_a_blank_one() {
    let (mut doc, l) = doc_with_line();
    let strip = crate::flip_strip::FlipStrip {
        autokey: true,
        ..Default::default()
    };

    // O playhead no quadro 5 — dentro do hold da chave do quadro 0.
    //
    // **O fps é o do OBJETO** (`DEFAULT_FPS` = 24), não um número conveniente: a 1ª
    // versão deste teste semeou o playhead a 12 fps, o objeto leu o tempo a 24, o
    // gesto esculpiu a chave do quadro **10** — e o teste, lendo o 5, acusou o código
    // de não esculpir. (`feedback_test_with_product_numbers_not_convenient_ones`.)
    let fps = f64::from(ph2d_flip::DEFAULT_FPS);
    let mut ph = Playhead::new(1.0 / fps);
    ph.seek_frame(5, fps);

    let p = ReshapeParams {
        kind: ReshapeKind::Push,
        radius: 2.0,
        strength: 1.0,
        ..Default::default()
    };
    let s = InputSample {
        pos: Vec2::new(2.0, 0.0),
        delta: Vec2::new(0.0, 1.0),
        pressure: 1.0,
    };
    reshape_begin(&mut doc, &ph, Some(l), &strip, &p, &s).expect("o autokey cria a chave");

    let here = strokes_at(&doc, l, 5);
    assert_eq!(
        here.len(),
        1,
        "a chave nova nasceu EM BRANCO: o usuario esculpiria o nada"
    );
    assert!(
        here[0].positions()[2].y > 0.5,
        "a duplicata nasceu, mas o gesto nao a esculpiu"
    );
    // E o quadro 0 (o original) fica INTACTO — a duplicata é que foi esculpida.
    assert_eq!(
        strokes_at(&doc, l, 0)[0].positions()[2],
        Vec2::new(2.0, 0.0),
        "a escultura vazou para o desenho ORIGINAL do hold"
    );
}
