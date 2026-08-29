//! ⭐⭐⭐ **AS TRÊS FORMAS DA W103** — a estrela, a gaiola e o elipsóide, medidas onde cada uma
//! promete alguma coisa que nenhuma das outras promete.
//!
//! ⚠️ O censo derivado ([`the_census_of_every_primitive`](the_census_of_every_primitive.rs)) já
//! pergunta as quatro coisas que valem para **todas** (marcha, bordo, filete que tira material,
//! passo inteiro para quem não infla). O que fica aqui é o que é **próprio** de cada uma:
//!
//! - a estrela é uma **união** de convexos, e o que se mede é que ela não é um decágono;
//! - a gaiola é a única forma **oca** do módulo, e o que se mede é o vazio;
//! - o elipsóide é o único campo **subestimador por construção**, e o que se mede é que ele nunca
//!   sobrestima — que é a direção que fura.

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Xform};
use ph2d_field_eval::Field;

fn field_of(p: Primitive) -> Field {
    let doc = FieldDoc::new(
        vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p))],
        NodeId(0),
    )
    .expect("a peça");
    Field::new(&doc)
}

/// Onde a superfície cruza o raio que sai da origem na direção `(dx, dy)`, no plano `z = 0`.
///
/// ⚠️ **Bissecção sobre o SINAL**, e não uma leitura do valor: o campo de uma união é um
/// subestimador longe da peça, e ler `f` como se fosse a distância mediria o erro do subestimador em
/// vez do sítio da superfície.
fn boundary_radius(f: &Field, dx: f64, dy: f64, ate: f64) -> f64 {
    let (mut lo, mut hi) = (0.0, ate);
    assert!(f.at(0.0, 0.0, 0.0) < 0.0, "a origem tem de estar dentro");
    assert!(
        f.at(dx * hi, dy * hi, 0.0) > 0.0,
        "o fim do raio tem de estar fora"
    );
    for _ in 0..60 {
        let m = 0.5 * (lo + hi);
        if f.at(dx * m, dy * m, 0.0) < 0.0 {
            lo = m;
        } else {
            hi = m;
        }
    }
    0.5 * (lo + hi)
}

const STAR_POINTS: u32 = 5;
const OUTER: f64 = 0.45;
const INNER: f64 = 0.18;
const HALF_H: f64 = 0.25;

fn a_star(round: f32) -> Primitive {
    Primitive::Star {
        points: STAR_POINTS,
        outer: OUTER as f32,
        inner: INNER as f32,
        half_height: HALF_H as f32,
        round,
    }
}

/// ⭐⭐⭐ **UMA ESTRELA DE 5 PONTAS NÃO É UM DECÁGONO** — e é este gate que dá sentido à primitiva.
///
/// # ⚠️ Por que as pontas e os vales não bastam
///
/// Uma forma que acertasse o raio nas dez direções notáveis (`outer` nas pontas, `inner` nos vales) e
/// as ligasse por um **arco** passaria qualquer gate que só amostrasse essas dez direções. O que
/// distingue a estrela é o que há **entre** elas: a fronteira é o **segmento de recta** que liga a
/// ponta ao vale, e o raio dele em polares é `1/(a·cos θ + b·sin θ)`, que é uma função conhecida.
///
/// ⇒ este gate mede **nove ângulos por sector**, incluindo os intermédios, contra a recta analítica.
#[test]
fn a_five_point_star_is_not_a_ten_sided_polygon() {
    let f = field_of(a_star(0.0));
    let beta = std::f64::consts::PI / f64::from(STAR_POINTS);
    // A recta que liga a ponta `(OUTER, 0)` ao vale `(INNER, beta)`: `a·x + b·y = 1`.
    let a = 1.0 / OUTER;
    let b = (1.0 - INNER * beta.cos() / OUTER) / (INNER * beta.sin());
    let mut pior = 0.0_f64;
    for k in 0..STAR_POINTS {
        let phi = std::f64::consts::TAU * f64::from(k) / f64::from(STAR_POINTS);
        for i in 0..=8 {
            let t = f64::from(i) / 8.0 * beta;
            let esperado = 1.0 / (a * t.cos() + b * t.sin());
            for lado in [1.0, -1.0] {
                let ang = phi + lado * t;
                let medido = boundary_radius(&f, ang.cos(), ang.sin(), 1.0);
                pior = pior.max((medido - esperado).abs());
            }
        }
    }
    assert!(
        pior < 2.0e-4,
        "a fronteira desvia {pior:.6} da recta ponta→vale — a estrela não é um polígono de {} \
         lados, e uma fronteira curva entre as duas seria exactamente isso",
        2 * STAR_POINTS
    );
}

/// ⭐ **A ponta e o vale estão onde foram pedidos** — o controle do gate acima, medido nas duas
/// direções notáveis.
///
/// ⛔ E a metade que o torna útil: **trocar `outer` por `inner` reprova**. Sem ela, um construtor
/// que lesse os dois campos ao contrário faria uma estrela igualmente válida, rodada de meio sector.
#[test]
fn the_tips_reach_the_outer_radius_and_the_valleys_the_inner_one() {
    let f = field_of(a_star(0.0));
    let beta = std::f64::consts::PI / f64::from(STAR_POINTS);
    for k in 0..STAR_POINTS {
        let phi = std::f64::consts::TAU * f64::from(k) / f64::from(STAR_POINTS);
        let ponta = boundary_radius(&f, phi.cos(), phi.sin(), 1.0);
        let vale = boundary_radius(&f, (phi + beta).cos(), (phi + beta).sin(), 1.0);
        assert!(
            (ponta - OUTER).abs() < 1.0e-4,
            "a ponta {k} está a {ponta:.6} e foi pedida a {OUTER}"
        );
        assert!(
            (vale - INNER).abs() < 1.0e-4,
            "o vale {k} está a {vale:.6} e foi pedido a {INNER}"
        );
        assert!(
            (ponta - vale).abs() > 0.1,
            "a ponta e o vale deram o mesmo raio — isto é um polígono, não uma estrela"
        );
    }
}

/// ⭐⭐ **SÓ O ARO ARREDONDA; a quina VERTICAL da ponta fica viva** — a divisão que o `round` desta
/// forma promete, e a mesma do prisma.
///
/// ⚠️ Ela precisa das **duas** metades: sem a segunda, um filete que arredondasse tudo passaria; sem
/// a primeira, um filete inerte passaria (e foi exactamente esse o defeito da W101, apanhado pelo
/// censo).
#[test]
fn only_the_rim_of_a_star_is_rounded_and_the_vertical_tip_stays_sharp() {
    let round = 0.05_f32;
    let limite = ph2d_field::round_limit(&a_star(0.0)).expect("a estrela tem filete");
    assert!(
        round < limite,
        "a fixtura tem de caber no limite ({limite:.4}) para medir o que quer"
    );
    let vivo = field_of(a_star(0.0));
    let redondo = field_of(a_star(round));
    let ponta = (OUTER, 0.0);
    // A meia-altura, a quina da ponta é uma aresta **vertical** — nenhum dos dois a toca.
    for (nome, f) in [("vivo", &vivo), ("redondo", &redondo)] {
        let r = boundary_radius(f, 1.0, 0.0, 1.0);
        assert!(
            (r - OUTER).abs() < 1.0e-4,
            "«{nome}»: a ponta a meia-altura mede {r:.6} e foi pedida a {OUTER} — o filete do ARO \
             não pode comer a quina vertical"
        );
    }
    // Junto à tampa, o filete TEM de ter comido.
    let z = HALF_H * 0.999;
    assert!(
        vivo.at(ponta.0 * 0.999, 0.0, z) < 0.0,
        "sem filete, a quina da ponta junto à tampa está dentro"
    );
    assert!(
        redondo.at(ponta.0 * 0.999, 0.0, z) > 0.0,
        "com filete de {round}, a quina da ponta junto à tampa tem de estar FORA — o aro arredonda"
    );
}

const FH: [f64; 3] = [0.45, 0.35, 0.4];
const THICK: f64 = 0.12;

fn a_frame(round: f32) -> Primitive {
    Primitive::BoxFrame {
        half: [FH[0] as f32, FH[1] as f32, FH[2] as f32],
        thickness: THICK as f32,
        round,
    }
}

/// ⭐⭐⭐ **A GAIOLA É OCA, E AS DOZE ARESTAS ESTÃO LÁ** — a única forma do módulo cujo centro está
/// **fora** da peça.
///
/// ⚠️ Foi esta propriedade que reprovou o censo do filete na W103: ele perguntava `f(0,0,0) < 0`
/// (*«o centro da peça está dentro»*), que é uma afirmação sobre sólidos **maciços**. *Uma sonda que
/// amostra um ponto escolhido a olho carrega, sem o dizer, a forma que o autor tinha em mente.*
#[test]
fn a_box_frame_is_hollow_and_its_twelve_edges_are_solid() {
    let f = field_of(a_frame(0.0));
    assert!(
        f.at(0.0, 0.0, 0.0) > 0.0,
        "o miolo da gaiola tem de estar VAZIO"
    );
    // O centro de cada uma das seis faces também é vazio.
    for eixo in 0..3 {
        for lado in [1.0, -1.0] {
            let mut p = [0.0; 3];
            p[eixo] = lado * FH[eixo] * 0.999;
            assert!(
                f.at(p[0], p[1], p[2]) > 0.0,
                "o centro da face {eixo}{lado:+} tem de estar vazio — a gaiola não tem tampa"
            );
        }
    }
    // E o meio de cada uma das doze vigas está cheio.
    let dentro = FH.map(|h| h - THICK * 0.5);
    let mut vigas = 0;
    for eixo in 0..3 {
        for sa in [1.0_f64, -1.0] {
            for sb in [1.0_f64, -1.0] {
                let mut p = [0.0; 3];
                let (a, b) = ((eixo + 1) % 3, (eixo + 2) % 3);
                p[a] = sa * dentro[a];
                p[b] = sb * dentro[b];
                assert!(
                    f.at(p[0], p[1], p[2]) < 0.0,
                    "o meio da viga (eixo {eixo}, {sa:+}{sb:+}) tem de estar CHEIO"
                );
                vigas += 1;
            }
        }
    }
    assert_eq!(vigas, 12, "uma gaiola tem doze arestas");
}

/// ⭐ **A gaiola tem o tamanho e a espessura que foram pedidos** — a face de fora está em `half`, e a
/// de dentro a `thickness` dela.
#[test]
fn the_frame_keeps_the_size_and_the_thickness_it_was_asked() {
    let f = field_of(a_frame(0.0));
    // Ao longo de +x, na linha de centro da viga do canto: a superfície está em `hx`.
    let (y, z) = (FH[1] - THICK * 0.5, FH[2] - THICK * 0.5);
    let (mut lo, mut hi) = (0.0_f64, FH[0] * 2.0);
    assert!(f.at(FH[0] * 0.5, y, z) < 0.0 && f.at(hi, y, z) > 0.0);
    for _ in 0..60 {
        let m = 0.5 * (lo + hi);
        if f.at(m, y, z) < 0.0 {
            lo = m;
        } else {
            hi = m;
        }
    }
    let fora = 0.5 * (lo + hi);
    assert!(
        (fora - FH[0]).abs() < 1.0e-4,
        "a face de fora está em {fora:.6} e foi pedida em {}",
        FH[0]
    );
    // E a parede de dentro da viga em Z: em `x = hx − thickness` ela acaba.
    assert!(
        f.at(FH[0] - THICK * 0.5, FH[1] - THICK * 0.5, 0.0) < 0.0,
        "o meio da viga em Z tem de estar cheio"
    );
    assert!(
        f.at(FH[0] - THICK * 1.2, FH[1] - THICK * 0.5, 0.0) > 0.0,
        "para lá da espessura a viga acabou — senão ela é mais grossa do que o pedido"
    );
}

const RADII: [f64; 3] = [0.5, 0.2, 0.35];

/// ⭐⭐⭐ **O CONJUNTO ZERO DO ELIPSÓIDE É EXATO** — `Σ(pᵢ/rᵢ)² = 1` é a definição da forma, e o
/// campo tem de a honrar em toda a direção, não nos três eixos.
///
/// ⛔ E o controle: com os raios **trocados**, os mesmos pontos deixam de estar na superfície. Sem
/// ele, um construtor que lesse `radii` por outra ordem passaria — a forma seria igualmente um
/// elipsóide, e apenas o errado.
#[test]
fn an_ellipsoid_is_exactly_zero_on_its_own_surface() {
    let f = field_of(Primitive::Ellipsoid {
        radii: RADII.map(|v| v as f32),
    });
    let mut pior = 0.0_f64;
    let mut trocado_pior = 0.0_f64;
    for i in 0..64_u32 {
        // Direções espalhadas sem trigonometria repetida: a espiral de Fibonacci.
        let z = 1.0 - 2.0 * (f64::from(i) + 0.5) / 64.0;
        let r = (1.0 - z * z).max(0.0).sqrt();
        let a = 2.399_963_2 * f64::from(i);
        let u = [r * a.cos(), r * a.sin(), z];
        let p = [u[0] * RADII[0], u[1] * RADII[1], u[2] * RADII[2]];
        pior = pior.max(f.at(p[0], p[1], p[2]).abs());
        let q = [u[0] * RADII[1], u[1] * RADII[2], u[2] * RADII[0]];
        trocado_pior = trocado_pior.max(f.at(q[0], q[1], q[2]).abs());
    }
    assert!(
        pior < 1.0e-5,
        "a superfície do elipsóide erra {pior:.7} — ela é o conjunto `Σ(pᵢ/rᵢ)² = 1`, exato"
    );
    assert!(
        trocado_pior > 1.0e-2,
        "com os raios trocados o campo continua a dar zero ({trocado_pior:.7}) — este gate não \
         distingue os eixos"
    );
}

/// ⭐⭐⭐ **O ELIPSÓIDE NUNCA SOBRESTIMA** — a única direção de erro que fura a marcha.
///
/// ⚠️ **Contra uma TESTEMUNHA, e não contra a distância exata**: a distância exata a um elipsóide
/// resolve uma quártica (é por isso que a forma é um subestimador). Mas `|p − s|` para **qualquer**
/// ponto `s` da superfície é um majorante da distância verdadeira ⇒ se `f(p) <= |p − s|` para todos
/// os `s` amostrados, `f` não sobrestima. *Uma desigualdade contra uma testemunha prova o lado que
/// importa sem precisar do oráculo que não existe.*
#[test]
fn the_ellipsoid_never_overestimates_the_distance() {
    let f = field_of(Primitive::Ellipsoid {
        radii: RADII.map(|v| v as f32),
    });
    let superficie: Vec<[f64; 3]> = (0..256_u32)
        .map(|i| {
            let z = 1.0 - 2.0 * (f64::from(i) + 0.5) / 256.0;
            let r = (1.0 - z * z).max(0.0).sqrt();
            let a = 2.399_963_2 * f64::from(i);
            [r * a.cos() * RADII[0], r * a.sin() * RADII[1], z * RADII[2]]
        })
        .collect();
    let mut folga_maxima = 0.0_f64;
    for i in 0..24_u8 {
        for j in 0..24_u8 {
            for k in 0..24_u8 {
                let c = |v: u8| f64::from(v) / 23.0 * 2.0 - 1.0;
                let p = [c(i), c(j), c(k)];
                let d = f.at(p[0], p[1], p[2]);
                if d <= 0.0 {
                    continue;
                }
                let mais_perto = superficie
                    .iter()
                    .map(|s| {
                        ((p[0] - s[0]).powi(2) + (p[1] - s[1]).powi(2) + (p[2] - s[2]).powi(2))
                            .sqrt()
                    })
                    .fold(f64::INFINITY, f64::min);
                assert!(
                    d <= mais_perto + 1.0e-4,
                    "em {p:?} o campo diz {d:.6} e há superfície a {mais_perto:.6} — ele \
                     SOBRESTIMA, e a marcha atravessa"
                );
                folga_maxima = folga_maxima.max(mais_perto - d);
            }
        }
    }
    // ⛔ O CONTROLE: se o campo fosse a distância exata, a folga seria ~0 e o gate acima não mediria
    // nada. Ele mede porque o subestimador é REAL — e a folga é da ordem de `1 − min/max`.
    assert!(
        folga_maxima > 0.05,
        "a folga máxima foi {folga_maxima:.6} — sem subestimação a desigualdade acima é trivial"
    );
}

/// ⭐ **O documento RECUSA um vale fora da ponta** — a única parede deste módulo que é outro campo.
#[test]
fn a_star_with_its_valley_outside_its_tip_is_refused() {
    let mau = Primitive::Star {
        points: 5,
        outer: 0.3,
        inner: 0.3,
        half_height: 0.2,
        round: 0.0,
    };
    assert!(
        FieldDoc::new(
            vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(mau))],
            NodeId(0)
        )
        .is_err(),
        "com o vale no raio da ponta a união devolve o polígono dos vales — uma estrela que deixou \
         de ser uma estrela, em silêncio"
    );
    let grosso = Primitive::BoxFrame {
        half: [0.3, 0.3, 0.3],
        thickness: 0.31,
        round: 0.0,
    };
    assert!(
        FieldDoc::new(
            vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(grosso))],
            NodeId(0)
        )
        .is_err(),
        "uma viga mais grossa do que a meia-extensão fecha a gaiola — e a dobra por `abs` deixa de \
         ser exata"
    );
}

/// ⭐⭐ **Baixar a PONTA empurra o VALE** — a coerção nos dois sentidos.
///
/// ⚠️ Recusar seria a resposta errada: o slider do vale já pára no raio da ponta, então o valor
/// inválido só chega por um arrasto do **outro** controlo — e uma recusa ali pararia o arrasto sem
/// dizer porquê, num campo em que o artista nem estava a tocar.
#[test]
fn dragging_the_tip_down_past_the_valley_carries_the_valley_with_it() {
    let mut p = a_star(0.0);
    // O índice 1 é a ponta; o 2 é o vale.
    ph2d_field::set_dim(&mut p, 0, 1, 0.1).expect("a ponta aceita");
    let Primitive::Star { outer, inner, .. } = p else {
        unreachable!("continua uma estrela")
    };
    assert!(
        outer > inner,
        "a ponta ficou em {outer} e o vale em {inner} — a estrela inverteu-se"
    );
    assert!(
        FieldDoc::new(
            vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p))],
            NodeId(0)
        )
        .is_ok(),
        "o que a coerção deixa passar tem de ser um documento válido"
    );
}
