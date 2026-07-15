//! Os gates do motor de interpolação.
//!
//! O que eles medem é o que o **olho** vê, não o que o código faz:
//!
//! 1. as pontas são as formas originais (`t=0` é A, `t=1` é B) — sem isso nada mais importa;
//! 2. a forma **não gira** no meio do caminho (é o sintoma nº 1 da correspondência errada);
//! 3. a forma **não vira do avesso** (o sintoma nº 2 — sentidos de percurso opostos);
//! 4. as **quinas sobrevivem** (é onde a reamostragem uniforme do flubber perde);
//! 5. as **pontas** e a **cobertura** do pareamento.

use super::*;
use crate::matching::{map_forward, search};
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
            let (pa, pb) = (oa.at(u), target.at(map_forward(&corr.knots, u)));
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
    let at0 = morph(&a, &b, 0.0).expect("t=0");
    let at1 = morph(&a, &b, 1.0).expect("t=1");

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
    let mid = morph(&a, &b, 0.5).expect("o meio");

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

    let mid = morph(&a, &b, 0.5).expect("o meio");
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
    let at0 = morph(&a, &b, 0.0).expect("t=0");

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
    let with_pent = morph(&a, &pent, 0.0).expect("t=0 contra o pentágono");
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

// (O "escape manual" — o `offset`/Rotate Match — foi REMOVIDO 2026-07-14. A correspondência é 100%
// automática; o ajuste, no modelo vivo, é editar as formas-fonte. Os gates que provavam o offset
// saíram com ele.)

/// Uma forma degenerada não derruba o motor — devolve `None`, e o chamador não faz nada.
#[test]
fn a_degenerate_shape_is_refused_not_crashed() {
    let a = square([0.0, 0.0], 1.0);
    let empty = VecPath::default();
    assert!(morph(&a, &empty, 0.5).is_none());
    assert!(morph(&empty, &a, 0.5).is_none());
    assert!(steps(&a, &empty, 3).is_empty());
}

/// O Blend do Illustrator: `n` passos, só os do MEIO, na ordem.
#[test]
fn steps_are_the_in_betweens_and_only_them() {
    let (a, b) = (square([0.0, 0.0], 1.0), square([8.0, 0.0], 1.0));
    let out = steps(&a, &b, 3);
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

    let mid = morph(&a, &b, 0.5).expect("o meio");
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

/// **O SMOKE DO ENIO: quadrado→estrela ondulava.**
///
/// As duas pontas são POLÍGONOS (só arestas retas), então **todo** intermediário tem de ser um
/// polígono. Media-se o contrário: até **0,24 unidade** de desvio da reta numa forma de tamanho 2
/// — 12% da forma. As intermediárias "não representavam a transição".
///
/// Duas causas, e as duas moram na mesma frase — *a reta do documento é a cúbica DEGENERADA*:
///
/// 1. `(P0,P0,P3,P3)` é reta, mas a **parametrização não é uniforme**. Cortada em posições de arco
///    diferentes, ela devolve controles em **frações diferentes** de cada aresta, e o lerp de
///    frações desalinhadas tira o controle de cima da corda. Conserto: a reta entra no motor na
///    forma **canônica** (⅓, ⅔), que é afim em `t` — e o corte de uma curva afim é afim.
/// 2. Toda borda de peça É uma âncora, e âncora é fronteira entre segmentos: perguntar "de quem é
///    este ponto?" ali é perguntar de que lado de um empate o `f64` caiu. Quando caía no segmento
///    anterior, a peça **colapsava num ponto** e a aresta sumia do pareamento (3 das 10 arestas da
///    estrela!). Conserto: o segmento se decide pelo **MEIO** da peça, que não é fronteira de nada.
#[test]
fn a_morph_between_two_polygons_is_a_polygon() {
    let a = square([0.0, 0.0], 1.0);
    let b = shape(ShapeKind::Star, [6.0, 0.0], [1.0, 1.0], &[5.0, 0.45, 0.0]);

    for step in 0..=10 {
        let t = f64::from(step) / 10.0;
        let m = morph(&a, &b, t).expect("o passo");
        let o = Outline::of(&m).unwrap();
        for (k, s) in o.segs.iter().enumerate() {
            let chord = s.p3 - s.p0;
            let len = chord.hypot();
            assert!(
                len > 1e-9,
                "t={t}: a aresta {k} colapsou num PONTO — uma aresta sumiu do pareamento"
            );
            for c in [s.p1, s.p2] {
                let v = c - s.p0;
                let off = (v.x * chord.y - v.y * chord.x).abs() / len;
                assert!(
                    off < 1e-9,
                    "t={t}: a aresta {k} ondulou {off} — as duas pontas são polígonos, então o \
                     meio do caminho também tem de ser"
                );
            }
        }
        // E o documento tem de VER um polígono: handles colapsados nas âncoras.
        for (k, v) in m.verts.iter().enumerate() {
            assert!(
                close(v.in_handle, v.anchor) && close(v.out_handle, v.anchor),
                "t={t}: o vértice {k} saiu com alça de curva — o documento escreve reta com a \
                 alça NA âncora, e é isso que o `is_line` do resto do repo testa"
            );
        }
    }
}

/// **Nenhuma peça do pareamento é um PONTO — e as peças COBREM o contorno inteiro.**
///
/// É a causa direta do "uma intermediária ficou gigante" e do "não representam a transição": uma
/// peça degenerada num dos lados significa que uma aresta inteira da OUTRA forma foi interpolada
/// contra o nada, e o pareamento seguinte anda torto.
///
/// # A cobertura é o oráculo, e a peça-ponto é só o sintoma
///
/// A peça degenerada não é o dano: o dano é o **arco que ficou sem peça nenhuma**. Quando a imagem
/// do corte não fecha o ciclo em `f64` (medido: `1 − 3,7e-14` em vez de `0`), o `cut` trunca a peça
/// na origem e **uma aresta inteira desaparece do pareamento** — e há como isso acontecer sem
/// deixar nenhuma peça-ponto para trás. Medir só o sintoma é apostar que os dois andam juntos.
///
/// Então o gate mede a **soma dos arcos das peças**: ela tem de ser o contorno todo, sem sobra e
/// sem falta.
#[test]
fn every_piece_of_the_pairing_is_a_real_piece() {
    let pairs = [
        (
            square([0.0, 0.0], 1.0),
            shape(ShapeKind::Star, [6.0, 0.0], [1.0, 1.0], &[5.0, 0.45, 0.0]),
        ),
        (
            circle([0.0, 0.0], 1.0),
            shape(ShapeKind::Polygon, [5.0, 0.0], [1.0, 1.0], &[7.0, 0.0]),
        ),
        (
            shape(ShapeKind::Heart, [0.0, 0.0], [1.0, 1.0], &[]),
            square([4.0, 3.0], 1.5),
        ),
    ];
    for (a, b) in &pairs {
        let (oa, ob) = (Outline::of(a).unwrap(), Outline::of(b).unwrap());
        let corr = search(&oa, &ob);
        let target = if corr.reversed { ob.reversed() } else { ob };
        // O MESMO pareamento que o produto usa — não um espelho dele.
        let (pieces_a, pieces_b) = pair_up(&oa, &target, &corr.knots).expect("o pareamento");

        for (side, o, pieces) in [("A", &oa, &pieces_a), ("B", &target, &pieces_b)] {
            for (k, p) in pieces.iter().enumerate() {
                assert!(
                    (p.p3 - p.p0).hypot() > 1e-9 || p.p1.distance(p.p0) > 1e-9,
                    "a peça {k} do lado {side} é um PONTO — uma aresta caiu fora do pareamento"
                );
            }
            let covered: f64 = pieces.iter().map(|p| p.arclen(ARCLEN_EPS)).sum();
            let gap = (covered - o.total).abs() / o.total;
            assert!(
                gap < 1e-9,
                "as peças do lado {side} cobrem {covered} de um contorno de {} ({:.4}% de \
                 diferença) — um pedaço do contorno ficou SEM peça, e a aresta que mora nele \
                 desapareceu do pareamento",
                o.total,
                gap * 100.0
            );
        }
    }
}

/// **A forma do meio não pode ESCAPAR das duas formas.**
///
/// Todo ponto do morph é `lerp(ponto de A, ponto de B, t)` — uma combinação **convexa**. Logo ele
/// mora, obrigatoriamente, dentro do casco das duas. Uma intermediária que "fica gigante" (o que o
/// Enio viu uma vez, e não se repetiu) **viola isso** — e um gate que só olha o caso que ele
/// conseguiu reproduzir não teria pego.
///
/// A varredura é ampla de propósito: o defeito era raro, e um evento raro só aparece em volume.
#[test]
fn no_intermediate_shape_ever_leaves_the_hull_of_the_two() {
    let kinds = [
        (ShapeKind::Rectangle, &[][..]),
        (ShapeKind::Ellipse, &[][..]),
        (ShapeKind::Polygon, &[5.0, 0.0][..]),
        (ShapeKind::Star, &[5.0, 0.45, 0.0][..]),
        (ShapeKind::Heart, &[][..]),
        (ShapeKind::Polygon, &[7.0, 0.0][..]),
    ];
    for (i, (ka, va)) in kinds.iter().enumerate() {
        for (j, (kb, vb)) in kinds.iter().enumerate() {
            // Poses e tamanhos diferentes: é onde as posições de arco deixam de ser "redondas" e
            // os empates de fronteira aparecem.
            let a = shape(*ka, [0.0, 0.0], [1.0 + i as f64 * 0.13, 1.0], va);
            let b = shape(
                *kb,
                [5.0 + j as f64 * 0.37, 1.1],
                [0.8, 1.0 + j as f64 * 0.11],
                vb,
            );
            let (lo, hi) = hull(&a, &b);
            for step in 0..=8 {
                let t = f64::from(step) / 8.0;
                let Some(m) = morph(&a, &b, t) else {
                    continue;
                };
                for v in &m.verts {
                    for p in [v.anchor, v.in_handle, v.out_handle] {
                        assert!(
                            p[0] >= lo[0] - 1e-6
                                && p[0] <= hi[0] + 1e-6
                                && p[1] >= lo[1] - 1e-6
                                && p[1] <= hi[1] + 1e-6,
                            "{ka:?}→{kb:?} em t={t}: o ponto {p:?} ESCAPOU do casco \
                             ({lo:?}..{hi:?}) — todo ponto do morph é combinação convexa de um \
                             ponto de A com um de B, e não pode sair de dentro dos dois"
                        );
                    }
                }
            }
        }
    }
}

/// A caixa que contém A e B (âncoras e alças — a curva nunca sai do casco dos controles).
fn hull(a: &VecPath, b: &VecPath) -> ([f64; 2], [f64; 2]) {
    let mut lo = [f64::MAX; 2];
    let mut hi = [f64::MIN; 2];
    for p in [a, b] {
        for v in &p.cooked().verts {
            for q in [v.anchor, v.in_handle, v.out_handle] {
                lo = [lo[0].min(q[0]), lo[1].min(q[1])];
                hi = [hi[0].max(q[0]), hi[1].max(q[1])];
            }
        }
    }
    (lo, hi)
}

/// **A QUEIXA DO ENIO: quina tem de casar com QUINA.**
///
/// > *"o algoritmo não subdivide a forma inicial com o número de pontos necessários para gerar
/// > formas intermediárias coerentes com a forma final"*
///
/// Ele está certo, e a causa é estrutural. A 1ª versão casava as formas por uma **rotação
/// rígida**: o quadrado tem quina a cada 0,25 de perímetro, a estrela a cada 0,10 — sob uma
/// rotação, **no máximo UMA** das 4 quinas do quadrado pode cair sobre um vértice da estrela. As
/// outras três caem no meio de uma aresta, as pontas da estrela nascem do meio das arestas retas
/// do quadrado, e o intermediário sai amassado.
///
/// Este gate exige o que o olho exige: **toda quina do quadrado casa com um vértice da estrela**.
/// Com a rotação, ele fica vermelho (1 de 4).
#[test]
fn every_corner_of_the_square_lands_on_a_vertex_of_the_star() {
    let a = square([0.0, 0.0], 1.0);
    let b = shape(ShapeKind::Star, [6.0, 0.0], [1.0, 1.0], &[5.0, 0.45, 0.0]);
    let (oa, ob) = (Outline::of(&a).unwrap(), Outline::of(&b).unwrap());
    let corr = search(&oa, &ob);
    let target = if corr.reversed { ob.reversed() } else { ob };
    let vb = target.anchors();

    let mut matched = 0;
    for ua in oa.anchors() {
        let v = map_forward(&corr.knots, ua);
        // A âncora mais próxima de B, em arco (ciclicamente).
        let gap = vb
            .iter()
            .map(|x| {
                let d = (v - x).abs();
                d.min(1.0 - d)
            })
            .fold(f64::MAX, f64::min);
        if gap < 1e-9 {
            matched += 1;
        }
    }
    assert_eq!(
        matched, 4,
        "só {matched} das 4 quinas do quadrado caíram sobre um vértice da estrela — as outras \
         caem no MEIO de uma aresta, e é por isso que a intermediária sai amassada"
    );
}

/// **E a subdivisão acontece:** os 6 vértices da estrela que não casaram com uma quina viram
/// pontos NOVOS nas arestas do quadrado. Sem isso não há como a ponta da estrela nascer.
#[test]
fn the_leftover_vertices_subdivide_the_smaller_shape() {
    let a = square([0.0, 0.0], 1.0);
    let b = shape(ShapeKind::Star, [6.0, 0.0], [1.0, 1.0], &[5.0, 0.45, 0.0]);
    let at0 = morph(&a, &b, 0.0).expect("t=0");

    // 4 quinas + 6 subdivisões = 10 (o número de âncoras da ESTRELA — é ela que manda a
    // resolução, porque é a mais detalhada).
    assert_eq!(
        at0.verts.len(),
        10,
        "o quadrado tem de sair subdividido nos pontos que a estrela pede"
    );
    // E as 4 quinas continuam LÁ, exatas — subdividir não pode arredondar quem tem quina.
    for corner in [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]] {
        assert!(
            at0.verts
                .iter()
                .any(|v| (v.anchor[0] - corner[0]).abs() < 1e-9
                    && (v.anchor[1] - corner[1]).abs() < 1e-9),
            "a quina {corner:?} sumiu na subdivisão"
        );
    }
}

/// **UMA QUINA CONVEXA NUNCA CASA COM UM VÉRTICE REENTRANTE.**
///
/// O Enio, apontando os intermediários: *"os pontos das setas verdes, desde o início, deveriam
/// estar mais para dentro, como os das setas vermelhas"*.
///
/// Medido: **duas das quatro quinas do quadrado casavam com os VALES da estrela** — os vértices
/// reentrantes. O custo só olhava POSIÇÃO, e o vale estava angularmente mais perto do que a ponta
/// seguinte. Na tela: o quadrado **colapsa pelas quinas** e as pontas nascem do meio das arestas
/// retas. É o "amassado".
///
/// O termo que faltava é o de **bending** (Sederberg & Greenwood): a quina do quadrado vira +90°,
/// a ponta da estrela +144°, e o vale vira **para o outro lado**. Uma quina convexa tem de casar
/// com uma convexa — é diferença de TIPO, não de grau.
///
/// O gate mede o SINAL da virada de cada par casado. Sem o termo de bending, ele fica vermelho.
#[test]
fn no_convex_corner_ever_marries_a_reflex_vertex() {
    let pairs = [
        (
            square([0.0, 0.0], 1.0),
            shape(ShapeKind::Star, [6.0, 0.0], [1.0, 1.0], &[5.0, 0.45, 0.0]),
        ),
        (
            shape(ShapeKind::Polygon, [0.0, 0.0], [1.0, 1.0], &[6.0, 0.0]),
            shape(ShapeKind::Star, [5.0, 0.0], [1.2, 1.0], &[7.0, 0.4, 0.0]),
        ),
    ];
    for (a, b) in &pairs {
        let (oa, ob) = (Outline::of(a).unwrap(), Outline::of(b).unwrap());
        let corr = search(&oa, &ob);
        let target = if corr.reversed { ob.reversed() } else { ob };
        let (ua, ub) = (oa.anchors(), target.anchors());
        let (ta, tb) = (crate::matching::turns(&oa), crate::matching::turns(&target));

        for (ka, ua_k) in ua.iter().enumerate() {
            let v = map_forward(&corr.knots, *ua_k);
            let Some(kb) = ub.iter().position(|x| (x - v).abs() < 1e-9) else {
                continue; // esta quina caiu no meio de uma aresta: não é um par casado
            };
            let (sa, sb) = (ta[ka].0, tb[kb].0);
            if sa.abs() < 1e-6 || sb.abs() < 1e-6 {
                continue; // âncora suave: não há "lado" para concordar
            }
            assert!(
                sa.signum() == sb.signum(),
                "a quina {ka} (virada {sa:.3}) casou com o vértice {kb} (virada {sb:.3}) — os \
                 dois viram para lados OPOSTOS: uma quina convexa está colapsando dentro de um \
                 vértice reentrante, e é isso que amassa a forma no meio do caminho"
            );
        }
    }
}

// (`every_offset_gives_a_different_correspondence_in_the_same_direction` saiu com o `offset` —
// o escape manual foi removido, ver a nota acima.)
