//! Os gates do motor de interpolação.
//!
//! O que eles medem é o que o **olho** vê, não o que o código faz:
//!
//! 1. as pontas são as formas originais (`t=0` é A, `t=1` é B) — sem isso nada mais importa;
//! 2. a forma **não gira** no meio do caminho (é o sintoma nº 1 da correspondência errada);
//! 3. a forma **não vira do avesso** (o sintoma nº 2 — sentidos de percurso opostos);
//! 4. as **quinas sobrevivem** (é onde a reamostragem uniforme do flubber perde);
//! 5. o **escape manual** existe e faz o que promete.

use super::*;
use ph2d_vec_scene::{ShapeKind, cook};

/// Uma forma do catálogo, centrada em `c`.
fn shape(kind: ShapeKind, c: [f64; 2], half: [f64; 2], params: &[f64]) -> VecPath {
    cook(
        kind,
        [c[0] - half[0], c[1] - half[1]],
        [c[0] + half[0], c[1] + half[1]],
        params,
    )
}

fn square(c: [f64; 2], r: f64) -> VecPath {
    shape(ShapeKind::Rectangle, c, [r, r], &[])
}

fn circle(c: [f64; 2], r: f64) -> VecPath {
    shape(ShapeKind::Ellipse, c, [r, r], &[])
}

/// O maior deslocamento entre os pontos correspondentes de duas formas, amostradas por arco.
/// É a métrica do "quanto isto se parece com aquilo" — e ela não depende da contagem de
/// vértices, que é justamente o que muda no morph.
fn max_gap(a: &VecPath, b: &VecPath) -> f64 {
    let (oa, ob) = (Outline::of(a).unwrap(), Outline::of(b).unwrap());
    let corr = search(&oa, &ob);
    let target = if corr.reversed { ob.reversed() } else { ob };
    (0..256)
        .map(|k| {
            let u = k as f64 / 256.0;
            let (pa, pb) = (oa.at(u), target.at(wrap(u - corr.phase)));
            (pa - pb).hypot()
        })
        .fold(0.0f64, f64::max)
}

/// O centroide amostrado por arco (o "onde a forma está").
fn centroid(p: &VecPath) -> [f64; 2] {
    let o = Outline::of(p).unwrap();
    let (mut x, mut y) = (0.0, 0.0);
    for k in 0..256 {
        let pt = o.at(k as f64 / 256.0);
        x += pt.x;
        y += pt.y;
    }
    [x / 256.0, y / 256.0]
}

/// **As pontas são as formas originais.** Um morph que não devolve A em `t=0` está mentindo
/// sobre tudo o mais.
#[test]
fn the_ends_of_the_morph_are_the_shapes_themselves() {
    let (a, b) = (square([0.0, 0.0], 1.0), circle([4.0, 0.0], 1.0));
    let at0 = morph(&a, &b, 0.0, BlendOpts::default()).expect("t=0");
    let at1 = morph(&a, &b, 1.0, BlendOpts::default()).expect("t=1");

    assert!(
        max_gap(&at0, &a) < 1e-6,
        "t=0 tem de ser A (gap {})",
        max_gap(&at0, &a)
    );
    assert!(
        max_gap(&at1, &b) < 1e-6,
        "t=1 tem de ser B (gap {})",
        max_gap(&at1, &b)
    );
}

/// **A forma não GIRA no meio do caminho** — o sintoma nº 1 da correspondência errada.
///
/// Um quadrado interpolado com uma cópia dele mesmo, deslocada, tem de simplesmente **andar**:
/// todo ponto anda em linha reta, e a forma do meio é o mesmo quadrado no meio do caminho. Se a
/// correspondência escolher a quina errada, o quadrado roda 90° (ou 180°) durante a viagem, e o
/// meio deixa de ser um quadrado do mesmo tamanho.
#[test]
fn a_square_walking_to_a_square_does_not_spin() {
    let (a, b) = (square([0.0, 0.0], 1.0), square([6.0, 0.0], 1.0));
    let mid = morph(&a, &b, 0.5, BlendOpts::default()).expect("o meio");

    let expected = square([3.0, 0.0], 1.0);
    let gap = max_gap(&mid, &expected);
    assert!(
        gap < 1e-6,
        "o quadrado do meio não é o quadrado no meio do caminho (gap {gap}) — a correspondência \
         girou a forma"
    );
}

/// **A forma não vira do AVESSO.** Duas formas com sentidos de percurso opostos: sem escolher o
/// sentido, todo ponto atravessa a forma inteira e ela colapsa num nó no meio do caminho.
///
/// O gate mede o colapso: a área varrida pelo meio não pode ser ridícula perto das pontas.
#[test]
fn opposite_winding_does_not_collapse_the_middle() {
    let a = circle([0.0, 0.0], 1.0);
    let mut b = circle([5.0, 0.0], 1.0);
    b.verts.reverse(); // o mesmo círculo, percorrido ao contrário
    for v in &mut b.verts {
        std::mem::swap(&mut v.in_handle, &mut v.out_handle);
    }

    let mid = morph(&a, &b, 0.5, BlendOpts::default()).expect("o meio");
    // O meio tem de ser um círculo de raio ~1 em (2.5, 0) — e não um ponto.
    let c = centroid(&mid);
    let o = Outline::of(&mid).unwrap();
    let radius = (0..64)
        .map(|k| {
            let p = o.at(k as f64 / 64.0);
            ((p.x - c[0]).powi(2) + (p.y - c[1]).powi(2)).sqrt()
        })
        .fold(0.0f64, f64::max);
    assert!(
        radius > 0.9,
        "o meio colapsou (raio {radius}) — os dois contornos estão sendo percorridos em \
         sentidos opostos e cada ponto atravessa a forma inteira"
    );
}

/// **As QUINAS sobrevivem.** É aqui que a reamostragem uniforme (o flubber) perde: ela põe os
/// pontos em posições de arco igualmente espaçadas, que não caem nas quinas — e o quadrado
/// chega do outro lado com os cantos comidos.
///
/// Interpolar um quadrado com um círculo em `t=0` tem de devolver o quadrado **exato**, com as
/// quatro quinas afiadas (nenhuma alça de controle solta na aresta).
#[test]
fn the_corners_survive_the_correspondence() {
    let (a, b) = (square([0.0, 0.0], 1.0), circle([0.0, 0.0], 1.0));
    let at0 = morph(&a, &b, 0.0, BlendOpts::default()).expect("t=0");

    // As 4 quinas do quadrado têm de estar LÁ, exatas.
    for corner in [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]] {
        let found = at0.verts.iter().any(|v| {
            (v.anchor[0] - corner[0]).abs() < 1e-6 && (v.anchor[1] - corner[1]).abs() < 1e-6
        });
        assert!(
            found,
            "a quina {corner:?} sumiu — a correspondência reamostrou por cima dela"
        );
    }
    // E o corte é na UNIÃO: contra um PENTÁGONO (5 âncoras, em posições de arco que não são as
    // do quadrado), o quadrado sai com as 4 quinas dele MAIS os cortes que o pentágono pediu.
    // (Contra o círculo do catálogo isso não aparece: as 4 âncoras dele caem exatamente nas
    // mesmas posições de arco das 4 quinas — a união é legitimamente 4.)
    let pent = shape(ShapeKind::Polygon, [0.0, 0.0], [1.0, 1.0], &[5.0, 0.0]);
    let with_pent = morph(&a, &pent, 0.0, BlendOpts::default()).expect("t=0 contra o pentágono");
    assert!(
        with_pent.verts.len() > 4,
        "o corte tem de ser na UNIÃO das âncoras das duas formas (achei {})",
        with_pent.verts.len()
    );
    for corner in [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]] {
        assert!(
            with_pent
                .verts
                .iter()
                .any(|v| (v.anchor[0] - corner[0]).abs() < 1e-6
                    && (v.anchor[1] - corner[1]).abs() < 1e-6),
            "a quina {corner:?} do quadrado não sobreviveu aos cortes do pentágono"
        );
    }
}

/// **O escape manual existe e MUDA a correspondência** (o `shapeIndex` do GSAP / *Map Nodes* do
/// Corel). Sem ele, o dia em que o automático errar o artista não tem saída nenhuma.
#[test]
fn the_manual_escape_actually_rotates_the_correspondence() {
    let (a, b) = (square([0.0, 0.0], 1.0), square([6.0, 0.0], 1.0));
    let auto = morph(&a, &b, 0.5, BlendOpts::default()).expect("auto");
    let turned = morph(
        &a,
        &b,
        0.5,
        BlendOpts {
            offset: 1,
            reverse: false,
        },
    )
    .expect("rodado uma âncora");

    // Rodar a correspondência em uma quina faz o quadrado GIRAR durante a viagem — então o meio
    // deixa de ser o quadrado limpo. É exatamente o poder que o escape dá ao artista (e, aqui,
    // a prova de que ele age).
    let gap = max_gap(&auto, &turned);
    assert!(
        gap > 0.1,
        "o `offset` não mudou nada (gap {gap}) — o escape manual é decorativo"
    );
}

/// Uma forma degenerada não derruba o motor — devolve `None`, e o chamador não faz nada.
#[test]
fn a_degenerate_shape_is_refused_not_crashed() {
    let a = square([0.0, 0.0], 1.0);
    let empty = VecPath::default();
    assert!(morph(&a, &empty, 0.5, BlendOpts::default()).is_none());
    assert!(morph(&empty, &a, 0.5, BlendOpts::default()).is_none());
    assert!(steps(&a, &empty, 3, BlendOpts::default()).is_empty());
}

/// O Blend do Illustrator: `n` passos, só os do MEIO, na ordem.
#[test]
fn steps_are_the_in_betweens_and_only_them() {
    let (a, b) = (square([0.0, 0.0], 1.0), square([8.0, 0.0], 1.0));
    let out = steps(&a, &b, 3, BlendOpts::default());
    assert_eq!(out.len(), 3);
    // t = ¼, ½, ¾ ⇒ os centros caminham 2, 4, 6.
    for (i, p) in out.iter().enumerate() {
        let want = 2.0 * (i + 1) as f64;
        let got = centroid(p)[0];
        assert!(
            (got - want).abs() < 1e-6,
            "o passo {i} devia estar em x={want}, está em {got}"
        );
    }
}

/// A cor caminha junto com a forma — é o que faz um blend parecer um blend, e não N cópias.
#[test]
fn the_fill_travels_with_the_shape() {
    let mut a = square([0.0, 0.0], 1.0);
    let mut b = square([6.0, 0.0], 1.0);
    a.fill = Some(Paint::solid(Rgba8::new(0, 0, 0, 255)));
    b.fill = Some(Paint::solid(Rgba8::new(200, 100, 50, 255)));

    let mid = morph(&a, &b, 0.5, BlendOpts::default()).expect("o meio");
    match mid.fill {
        Some(Paint::Solid(c)) => {
            assert_eq!(
                (c.r, c.g, c.b),
                (100, 50, 25),
                "a cor do meio é o meio da cor"
            );
        }
        other => panic!("o meio perdeu o preenchimento: {other:?}"),
    }
}
