//! Gates do tween (`tween.rs`).
//!
//! O que cada um pina é uma propriedade do que se VÊ no inbetween — extremos que fecham,
//! furo que acompanha o contorno, órfão que não pisca — nunca a fórmula reescrita ao lado.

use super::*;
use crate::ids::FlipObjectId;

fn stroke(pts: &[(f32, f32)]) -> FlipStroke {
    let mut s = FlipStroke::new();
    for &(x, y) in pts {
        s.push_default(Vec2::new(x, y));
    }
    s
}

fn drawing(strokes: Vec<FlipStroke>) -> FlipDrawing {
    let mut d = FlipDrawing::new();
    d.strokes = strokes;
    d
}

fn pts(s: &FlipStroke) -> Vec<(f32, f32)> {
    s.positions().iter().map(|p| (p.x, p.y)).collect()
}

/// **A propriedade que justifica o padding:** nos extremos (`t=0`/`t=1`) o
/// tween reproduz os desenhos originais PONTO A PONTO, mesmo com contagens
/// diferentes. (Uma reamostragem uniforme falharia aqui.)
#[test]
fn the_endpoints_are_reproduced_exactly() {
    let a = drawing(vec![stroke(&[(0.0, 0.0), (10.0, 0.0)])]); // 2 pontos
    let b = drawing(vec![stroke(&[(0.0, 5.0), (5.0, 5.0), (10.0, 5.0)])]); // 3
    let o = TweenOptions::default();

    let at0 = tween_drawing(&a, &b, 0.0, o);
    assert_eq!(at0.strokes[0].len(), 3, "conta pelo MAIOR");
    // Os 2 pontos de A saem exatos; o extra caiu no meio do segmento.
    assert_eq!(
        pts(&at0.strokes[0]),
        vec![(0.0, 0.0), (5.0, 0.0), (10.0, 0.0)]
    );

    let at1 = tween_drawing(&a, &b, 1.0, o);
    assert_eq!(pts(&at1.strokes[0]), pts(&b.strokes[0]), "B, ponto a ponto");
}

/// O meio é o meio (com easing linear).
#[test]
fn the_middle_is_the_average() {
    let a = drawing(vec![stroke(&[(0.0, 0.0), (10.0, 0.0)])]);
    let b = drawing(vec![stroke(&[(0.0, 10.0), (10.0, 10.0)])]);
    let mid = tween_drawing(&a, &b, 0.5, TweenOptions::default());
    assert_eq!(pts(&mid.strokes[0]), vec![(0.0, 5.0), (10.0, 5.0)]);
}

/// **Auto-flip:** B desenhado ao contrário (mesma forma, ordem invertida). Sem
/// o flip, o meio colapsaria num X; com ele, o traço só se translada.
#[test]
fn auto_flip_untangles_a_reversed_stroke() {
    let a = drawing(vec![stroke(&[(0.0, 0.0), (10.0, 0.0)])]);
    let b = drawing(vec![stroke(&[(10.0, 10.0), (0.0, 10.0)])]); // invertido
    let mid = tween_drawing(&a, &b, 0.5, TweenOptions::default());
    assert_eq!(
        pts(&mid.strokes[0]),
        vec![(0.0, 5.0), (10.0, 5.0)],
        "com flip: translação limpa"
    );
    // **Sem auto-flip o par vira uma ROTAÇÃO, não mais um colapso** — a espiral do v2
    // lê "ponta 0 de A vai para a ponta 0 de B" como meia-volta e a percorre pelo arco.
    // (No v1 este mesmo caso colapsava em `(5,5),(5,5)`, e era o que este gate pinava.)
    //
    // O auto-flip continua sendo o certo, e é por isso que ele continua LIGADO por
    // default: o artista desenhou a mesma forma no outro sentido querendo uma
    // TRANSLAÇÃO, e um giro de 180° é uma resposta bonita para a pergunta errada.
    let no_flip = TweenOptions {
        auto_flip: false,
        ..Default::default()
    };
    let bad = tween_drawing(&a, &b, 0.5, no_flip);
    let p = pts(&bad.strokes[0]);
    let seg = ((p[1].0 - p[0].0).powi(2) + (p[1].1 - p[0].1).powi(2)).sqrt();
    assert!(
        (seg - 10.0).abs() < 0.5,
        "sem flip o traço deveria GIRAR (comprimento preservado), e mediu {seg:.2}: {p:?}"
    );
    assert_ne!(
        p,
        pts(&mid.strokes[0]),
        "girar não é a mesma coisa que transladar"
    );
}

/// Cordas quase paralelas (< 15°) não decidem por cruzamento — decide a
/// distância. Aqui as pontas próximas já estão pareadas: NÃO inverte.
#[test]
fn nearly_parallel_chords_are_decided_by_distance() {
    let a = stroke(&[(0.0, 0.0), (10.0, 0.0)]);
    let b = stroke(&[(0.2, 1.0), (10.1, 1.0)]);
    assert!(!should_flip(&a, &b));
}

/// Traço de A sem par vira cópia estática (não some, não pisca).
#[test]
fn unpaired_strokes_of_a_stay_static() {
    let a = drawing(vec![
        stroke(&[(0.0, 0.0), (1.0, 0.0)]),
        stroke(&[(5.0, 5.0), (6.0, 5.0)]),
    ]);
    let b = drawing(vec![stroke(&[(0.0, 2.0), (1.0, 2.0)])]);
    let mid = tween_drawing(&a, &b, 0.5, TweenOptions::default());
    assert_eq!(mid.strokes.len(), 2);
    assert_eq!(pts(&mid.strokes[1]), pts(&a.strokes[1]), "cópia estática");
    // Órfão de B não aparece (default); com fade_orphans, aparece esmaecido.
    let o = TweenOptions {
        fade_orphans: true,
        ..Default::default()
    };
    let with = tween_drawing(&b, &a, 0.5, o); // agora B é o maior
    assert_eq!(with.strokes.len(), 2);
    assert!(
        (with.strokes[1].opacities()[0] - 0.5).abs() < 1e-6,
        "fade-in"
    );
}

/// O padding reparte os extras ∝ comprimento: o segmento longo leva mais.
#[test]
fn padding_favours_the_longer_segment() {
    let s = stroke(&[(0.0, 0.0), (1.0, 0.0), (11.0, 0.0)]); // segs 1 e 10
    let out = sample_padded(&s, 6, false);
    assert_eq!(out.len(), 6);
    // Os 3 originais estão lá.
    for p in [(0.0, 0.0), (1.0, 0.0), (11.0, 0.0)] {
        assert!(
            out.iter().any(|q| (q.pos.x - p.0).abs() < 1e-6),
            "original {p:?} preservado"
        );
    }
    // O segundo segmento (10× mais longo) ficou com quase todos os extras.
    let in_second = out
        .iter()
        .filter(|p| p.pos.x > 1.0 && p.pos.x < 11.0)
        .count();
    assert!(
        in_second >= 2,
        "extras foram pro segmento longo: {in_second}"
    );
}

/// A op de documento: N inbetweens BREAKDOWN entre duas chaves, e re-tweenar
/// regenera (não empilha).
#[test]
fn the_document_op_creates_breakdowns_and_is_idempotent() {
    let mut o = FlipObject::new(FlipObjectId(1), "O");
    let l = o.add_layer("L");
    let a = o
        .insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe)
        .unwrap();
    let b = o
        .insert_frame(l, 8, Hold::Implicit, KeyKind::Keyframe)
        .unwrap();
    o.drawing_mut(a)
        .unwrap()
        .strokes
        .push(stroke(&[(0.0, 0.0), (10.0, 0.0)]));
    o.drawing_mut(b)
        .unwrap()
        .strokes
        .push(stroke(&[(0.0, 10.0), (10.0, 10.0)]));

    let req = TweenRequest {
        layer: l,
        from: 0,
        to: 8,
        count: 3,
        options: TweenOptions::default(),
    };
    assert_eq!(o.tween(req), 3);
    let cells = o.layer(l).unwrap().cells();
    let frames: Vec<Frame> = cells.iter().map(|(k, _, _)| *k).collect();
    assert_eq!(frames, vec![0, 2, 4, 6, 8], "espaçados no intervalo");
    // Tipos: os do meio são BREAKDOWN.
    let kinds: Vec<KeyKind> = frames
        .iter()
        .map(|f| o.layer(l).unwrap().frames()[f].kind)
        .collect();
    assert_eq!(kinds[2], KeyKind::Breakdown);
    assert_eq!(kinds[0], KeyKind::Keyframe);
    // O do meio (t=0.5) está no meio do caminho.
    let mid_id = o.drawing_at(l, 4).unwrap();
    assert_eq!(
        pts(&o.drawing(mid_id).unwrap().strokes[0]),
        vec![(0.0, 5.0), (10.0, 5.0)]
    );

    // Re-tweenar com outra contagem REGENERA (não tweena os breakdowns).
    let again = TweenRequest { count: 1, ..req };
    assert_eq!(o.tween(again), 1);
    let frames: Vec<Frame> = o
        .layer(l)
        .unwrap()
        .cells()
        .iter()
        .map(|(k, _, _)| *k)
        .collect();
    assert_eq!(frames, vec![0, 4, 8], "os antigos sumiram");
    // E o inbetween novo é o do MEIO do caminho — prova de que a regeneração
    // interpolou entre os EXTREMOS (0 e 8), não entre a chave e um breakdown
    // sobrevivente. Com os ids remapeados pela compactação, um `da`/`db` resolvido
    // ANTES apontaria para o desenho errado e este valor sairia torto.
    let mid = o.drawing_at(l, 4).unwrap();
    assert_eq!(
        pts(&o.drawing(mid).unwrap().strokes[0]),
        vec![(0.0, 5.0), (10.0, 5.0)],
        "o inbetween regenerado é a média dos extremos"
    );
}

/// **O bug do smoke:** depois de gerar inbetweens, a chave "seguinte" para um novo
/// tween tem de ser o próximo **KEYFRAME** — não o breakdown vizinho. Sem isso, o
/// 2º Add interpolaria entre a chave 0 e o inbetween 2 (lixo entre 0 e 2), em vez
/// de regenerar o intervalo 0→8.
#[test]
fn the_next_tween_target_skips_the_breakdowns_it_just_made() {
    let mut o = FlipObject::new(FlipObjectId(1), "O");
    let l = o.add_layer("L");
    let a = o
        .insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe)
        .unwrap();
    let b = o
        .insert_frame(l, 8, Hold::Implicit, KeyKind::Keyframe)
        .unwrap();
    o.drawing_mut(a)
        .unwrap()
        .strokes
        .push(stroke(&[(0.0, 0.0), (10.0, 0.0)]));
    o.drawing_mut(b)
        .unwrap()
        .strokes
        .push(stroke(&[(0.0, 10.0), (10.0, 10.0)]));
    o.tween(TweenRequest {
        layer: l,
        from: 0,
        to: 8,
        count: 3,
        options: TweenOptions::default(),
    });
    let layer = o.layer(l).unwrap();
    // O próximo DESENHO depois de 0 é o breakdown em 2 …
    assert_eq!(layer.next_drawing_key(0), Some(2));
    // … mas o próximo KEYFRAME (o extremo do tween) continua sendo o 8.
    assert_eq!(layer.next_keyframe_key(0), Some(8));
    // E parado EM CIMA de um inbetween, o extremo A é a chave 0 (não o breakdown).
    assert_eq!(layer.keyframe_at_or_before(4), Some(0));
    assert_eq!(layer.next_keyframe_key(4), Some(8));
}

/// **O buraco anda junto com o contorno.**
///
/// O `clone_attrs` traz os furos de A verbatim. Num "O" que atravessa a tela com dois
/// keys + tween, o contorno externo interpolava e o furo ficava PARADO em A: os
/// inbetweens mostravam um "O" sólido (o furo saiu de dentro da forma) mais uma
/// mancha de cor solta viajando pelo caminho — o even-odd conta o anel órfão como
/// região preenchida.
#[test]
fn a_hole_travels_with_its_outer_contour() {
    let ring = |cx: f32, r: f32| -> Vec<Vec2> {
        vec![
            Vec2::new(cx - r, -r),
            Vec2::new(cx + r, -r),
            Vec2::new(cx + r, r),
            Vec2::new(cx - r, r),
        ]
    };
    let make = |cx: f32| -> FlipDrawing {
        let mut d = FlipDrawing::new();
        let mut s = FlipStroke::new();
        for p in ring(cx, 10.0) {
            s.push_point(Point {
                pos: p,
                width: 0.0,
                opacity: 1.0,
                color: Rgba::BLACK,
            });
        }
        s.closed = true;
        s.hide_stroke = true;
        s.fill = Some(crate::stroke::Fill {
            color: Rgba::BLACK,
            opacity: 1.0,
        });
        s.holes = vec![ring(cx, 4.0)];
        d.strokes.push(s);
        d
    };
    let a = make(0.0);
    let b = make(100.0);
    let mid = tween_drawing(&a, &b, 0.5, TweenOptions::default());
    let s = &mid.strokes[0];

    // O contorno externo está no meio do caminho…
    let cx_outer: f32 = s.positions().iter().map(|p| p.x).sum::<f32>() / s.positions().len() as f32;
    assert!(
        (cx_outer - 50.0).abs() < 1.0,
        "o contorno externo deveria estar em x=50, esta em {cx_outer}"
    );
    // … e o FURO tem de estar no meio do caminho TAMBÉM (senão saiu da forma).
    assert_eq!(s.holes.len(), 1, "o furo sumiu");
    let cx_hole: f32 = s.holes[0].iter().map(|p| p.x).sum::<f32>() / s.holes[0].len() as f32;
    assert!(
        (cx_hole - 50.0).abs() < 1.0,
        "o furo ficou para tras em x={cx_hole} (o contorno foi para 50): ele saiu de \
         dentro da forma e virou uma mancha solta"
    );
}

// ── Tween v2: a costura (correspondência + espiral dentro do desenho) ─────────

/// Gira `deg` graus em torno de `c`.
fn turned(s: &FlipStroke, c: Vec2, deg: f32) -> FlipStroke {
    let r = deg.to_radians();
    let (sin, cos) = (r.sin(), r.cos());
    let mut out = s.clone();
    for p in out.positions_mut() {
        let d = *p - c;
        *p = c + Vec2::new(cos * d.x - sin * d.y, sin * d.x + cos * d.y);
    }
    out
}

fn arc_len(s: &FlipStroke) -> f32 {
    s.positions()
        .windows(2)
        .map(|w| (w[1] - w[0]).length())
        .sum()
}

/// **A entrega da wave, vista no DESENHO:** um braço preso ao corpo gira 120°, e o
/// inbetween mostra o braço com o comprimento inteiro — não a corda entre as duas poses.
#[test]
fn a_swinging_arm_keeps_its_length_through_the_whole_drawing() {
    let body = stroke(&[(0.0, 0.0), (0.0, -60.0)]);
    let arm = stroke(&[(0.0, 0.0), (60.0, 0.0)]);
    let a = drawing(vec![body.clone(), arm.clone()]);
    let b = drawing(vec![body, turned(&arm, Vec2::ZERO, 120.0)]);

    let mid = tween_drawing(&a, &b, 0.5, TweenOptions::default());
    let l = arc_len(&mid.strokes[1]);
    assert!(
        (l - 60.0).abs() < 1.0,
        "o braço encolheu para {l:.1} de 60 (o lerp daria ~34)"
    );
    // O corpo, que não se moveu, continua exatamente onde estava.
    assert_eq!(pts(&mid.strokes[0]), pts(&a.strokes[0]));
}

/// **A correspondência atravessa o desenho inteiro:** com a ordem de desenho TROCADA em B,
/// o inbetween ainda mostra o braço no lugar do braço. Por índice, o meio do caminho seria
/// um X atravessando a figura.
#[test]
fn the_inbetween_follows_shapes_not_indices() {
    let arm = |y: f32| stroke(&[(0.0, y), (40.0, y)]);
    let leg = |y: f32| stroke(&[(0.0, y), (0.0, y - 50.0)]);
    let a = drawing(vec![arm(80.0), leg(0.0)]);
    let b = drawing(vec![leg(0.0), arm(100.0)]); // ordem trocada; o braço subiu 20
    let mid = tween_drawing(&a, &b, 0.5, TweenOptions::default());

    // O traço 0 da saída É o braço de A, e ele subiu METADE do caminho.
    let p = pts(&mid.strokes[0]);
    assert!(
        (p[0].1 - 90.0).abs() < 0.5 && (p[1].1 - 90.0).abs() < 0.5,
        "o braço do meio deveria estar em y=90: {p:?}"
    );
    // E a perna ficou parada (ela não mudou entre os quadros).
    assert_eq!(pts(&mid.strokes[1]), pts(&a.strokes[1]));
}

/// **O órfão VIAJA com o vizinho enquanto some.** Um detalhe que desaparece (uma
/// sobrancelha, um brilho) tem de acompanhar a cabeça que anda; pregado no ar ele lê como
/// um erro de desenho, não como uma coisa que sumiu.
#[test]
fn a_vanishing_stroke_travels_with_its_neighbour_while_it_fades() {
    let head = |x: f32| stroke(&[(x, 0.0), (x + 50.0, 0.0)]);
    let brow = |x: f32| stroke(&[(x + 10.0, 20.0), (x + 30.0, 20.0)]);
    let a = drawing(vec![head(0.0), brow(0.0)]);
    let b = drawing(vec![head(200.0)]); // a cabeça anda; a sobrancelha some

    let o = TweenOptions {
        fade_orphans: true,
        ..Default::default()
    };
    let mid = tween_drawing(&a, &b, 0.5, o);
    assert_eq!(mid.strokes.len(), 2, "o órfão sumiu do desenho");
    let p = pts(&mid.strokes[1]);
    assert!(
        (p[0].0 - 110.0).abs() < 1.0,
        "a sobrancelha ficou para trás em x={:.1} (a cabeça foi para 100..150)",
        p[0].0
    );
    assert!(
        (mid.strokes[1].opacities()[0] - 0.5).abs() < 1e-6,
        "e ela tem de estar sumindo, não só viajando"
    );
}

/// O simétrico: o órfão de B CHEGA vindo de onde o vizinho estava, e em `u=1` está
/// exatamente onde o artista o desenhou.
#[test]
fn a_newborn_stroke_arrives_from_where_its_neighbour_was() {
    let head = |x: f32| stroke(&[(x, 0.0), (x + 50.0, 0.0)]);
    let a = drawing(vec![head(0.0)]);
    let b = drawing(vec![head(200.0), stroke(&[(210.0, 20.0), (230.0, 20.0)])]);
    let o = TweenOptions {
        fade_orphans: true,
        ..Default::default()
    };

    let mid = tween_drawing(&a, &b, 0.5, o);
    let p = pts(&mid.strokes[1]);
    assert!(
        (p[0].0 - 110.0).abs() < 1.0,
        "o recém-nascido deveria vir junto com a cabeça: x={:.1}",
        p[0].0
    );
    // Em u=1 ele está EXATAMENTE onde B o desenhou (o fade termina no desenho do artista).
    let end = tween_drawing(&a, &b, 1.0, o);
    assert_eq!(pts(&end.strokes[1]), pts(&b.strokes[1]));
}

/// **O furo gira com o contorno.** O gate irmão (translação) já existia; a espiral abriu um
/// jeito novo de errar — um furo com espiral PRÓPRIA giraria por conta e sairia da forma.
#[test]
fn a_hole_turns_with_its_contour_not_on_its_own() {
    let ring = |r: f32| -> Vec<Vec2> {
        vec![
            Vec2::new(-r, -r),
            Vec2::new(r, -r),
            Vec2::new(r, r),
            Vec2::new(-r, r),
        ]
    };
    let mut s = FlipStroke::new();
    for p in ring(20.0) {
        s.push_point(Point {
            pos: p + Vec2::new(60.0, 0.0),
            width: 0.0,
            opacity: 1.0,
            color: Rgba::BLACK,
        });
    }
    s.closed = true;
    s.holes = vec![
        ring(6.0)
            .iter()
            .map(|p| *p + Vec2::new(60.0, 0.0))
            .collect(),
    ];
    let a = drawing(vec![s.clone()]);
    let b = drawing(vec![turned(&s, Vec2::ZERO, 90.0)]);
    // O `turned` não mexe nos furos, então B carrega o furo girado à mão.
    let mut sb = b.strokes[0].clone();
    sb.holes = vec![
        a.strokes[0].holes[0]
            .iter()
            .map(|p| Vec2::new(-p.y, p.x))
            .collect(),
    ];
    let b = drawing(vec![sb]);

    let mid = tween_drawing(&a, &b, 0.5, TweenOptions::default());
    let s = &mid.strokes[0];
    // O contorno girou 45°: o centro dele foi de (60,0) para 45° no círculo de raio 60.
    let c_out: Vec2 =
        s.positions().iter().fold(Vec2::ZERO, |a, &p| a + p) / s.positions().len() as f32;
    let c_hole: Vec2 = s.holes[0].iter().fold(Vec2::ZERO, |a, &p| a + p) / s.holes[0].len() as f32;
    assert!(
        (c_out - c_hole).length() < 1.0,
        "o furo saiu do contorno: contorno {c_out:?} × furo {c_hole:?}"
    );
    let r = c_out.length();
    assert!(
        (r - 60.0).abs() < 1.0,
        "o contorno cortou caminho pela corda (raio {r:.1} de 60)"
    );
}

/// **Um anel não é invertido pelas PONTAS.** Num traço fechado a "primeira" e a "última"
/// posição são vizinhas (a costura), então a corda entre elas não descreve percurso nenhum
/// — e o teste de cruzamento, que é bom para traço aberto, vira uma moeda.
///
/// ⚠️ Era um bug REAL que só a espiral tornou visível: o flip espúrio transformava uma
/// rotação limpa numa reflexão, o ajuste enxergava `σ ≈ 0` e caía na translação, e o anel
/// cortava caminho pela corda.
#[test]
fn a_closed_ring_is_not_flipped_by_its_endpoints() {
    let ring = |c: Vec2| {
        let mut s = FlipStroke::new();
        for p in [(-20.0, -20.0), (20.0, -20.0), (20.0, 20.0), (-20.0, 20.0)] {
            s.push_default(c + Vec2::new(p.0, p.1));
        }
        s.closed = true;
        s
    };
    let a = ring(Vec2::new(60.0, 0.0));
    // O mesmo anel, girado 90° em torno da origem (mesmo sentido de percurso).
    let b = turned(&a, Vec2::ZERO, 90.0);
    assert!(
        !crate::tween_flip::should_flip(&a, &b),
        "o anel foi invertido pelas pontas"
    );
}

/// E o que o caso fechado DE FATO tem de pegar: dois anéis percorridos em sentidos
/// contrários. Interpolar entre windings opostos vira a forma do avesso — o mesmo fato que
/// o `ph2d-vec-blend` mediu no Blend do vetor (a forma COLAPSA no meio).
#[test]
fn two_rings_wound_the_other_way_round_are_flipped() {
    let mut a = FlipStroke::new();
    for p in [(0.0, 0.0), (40.0, 0.0), (40.0, 40.0), (0.0, 40.0)] {
        a.push_default(Vec2::new(p.0, p.1));
    }
    a.closed = true;
    let mut b = a.clone();
    b.positions_mut().reverse(); // mesmo desenho, percorrido ao contrário
    assert!(
        crate::tween_flip::should_flip(&a, &b),
        "os dois sentidos opostos passaram batido"
    );
}

/// **Um traço que GIRA muito não é um traço desenhado ao contrário** — e as duas coisas
/// eram indistinguíveis para o teste de "direções opostas" (`da·db < 0` vale para qualquer
/// giro além de 90°).
///
/// ⚠️ O preço era invisível até a espiral: o flip espúrio colava a PONTA de A no COMEÇO de
/// B, o ajuste enxergava outro movimento, e o inbetween girava em torno do lugar errado.
/// Com o lerp o resultado já saía torto de qualquer jeito, então ninguém tinha o que
/// comparar.
#[test]
fn a_stroke_that_swings_past_ninety_degrees_is_not_a_reversed_stroke() {
    let shoulder = Vec2::new(0.0, 0.0);
    let a = stroke(&[(0.0, 0.0), (0.6, 0.0), (1.2, 0.0), (1.8, 0.0)]);
    for deg in [95.0, 120.0, 150.0, 179.0] {
        let b = turned(&a, shoulder, deg);
        assert!(
            !crate::tween_flip::should_flip(&a, &b),
            "um giro de {deg}° foi lido como traço invertido"
        );
    }
    // E o controle: o MESMO traço redesenhado ao contrário continua sendo invertido.
    let mut back = turned(&a, shoulder, 120.0);
    back.positions_mut().reverse();
    assert!(
        crate::tween_flip::should_flip(&a, &back),
        "o traço desenhado ao contrário passou batido"
    );
}
