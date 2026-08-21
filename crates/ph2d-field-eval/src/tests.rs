//! Os gates da ponte.
//!
//! ⚠️ **Estes são a promessa do módulo virada em teste.** A W0 mediu, num spike, que o
//! arredondamento entrega o raio pedido a 0,00 % — e uma medição de spike é uma anedota: ela vale
//! no dia em que foi feita. Aqui ela vira **assertiva executável**, e passa a valer todo dia.
//!
//! Nenhum destes pergunta "compila?". Cada um afirma um número que, mudando, torna a ferramenta
//! mentirosa **em silêncio** — um raio que não é o raio não dá erro, dá uma peça errada.

use super::*;
use ph2d_field::{Blend, FieldDoc, NodeId, Op, Primitive, Xform};

/// Tolerância. O campo é avaliado em `f32` dentro do motor, então ~1e-6 é o piso honesto.
const EPS: f64 = 2e-6;

fn combine(op: Op, children: Vec<NodeId>) -> Node {
    Node::new(Xform::IDENTITY, NodeKind::Combine { op, children })
}

/// Duas caixas em **L**, cuja quina côncava fica exatamente na origem.
///
/// Perto dela o campo de cada caixa é o de um meio-espaço (`y ≤ 0` e `x ≤ 0`), o que torna o filete
/// **analiticamente conhecido**: o arco tem centro em `(r, r)` e raio `r`.
fn elbow(blend: Blend) -> FieldDoc {
    FieldDoc::new(
        vec![
            leaf(
                Primitive::Box {
                    half: [1.0, 0.5, 0.5],
                    round: 0.0,
                },
                Xform::at(0.0, -0.5, 0.0),
            ),
            leaf(
                Primitive::Box {
                    half: [0.5, 1.0, 0.5],
                    round: 0.0,
                },
                Xform::at(-0.5, 0.0, 0.0),
            ),
            combine(Op::Union(blend), vec![NodeId(0), NodeId(1)]),
        ],
        NodeId(2),
    )
    .expect("cotovelo é um documento válido")
}

/// ⭐ **A promessa central do módulo.** O filete interno entrega o raio pedido — e o gate prova por
/// DOIS pontos independentes, porque um só poderia passar por acidente algébrico:
///
/// 1. no **centro do arco** `(r, r)` o campo vale `+r` (o centro de um filete fica FORA do sólido);
/// 2. no ponto do arco **mais próximo da origem** o campo vale `0` — é ele que diz que a superfície
///    passa onde deve, e não apenas que um número bate.
#[test]
fn an_exact_internal_fillet_delivers_the_radius_asked() {
    for r in [0.05_f64, 0.1, 0.2] {
        let f = Field::new(&elbow(Blend::Exact { radius: r as f32 }));

        let at_centre = f.at(r, r, 0.0);
        assert!(
            (at_centre - r).abs() < EPS,
            "centro do arco: esperava +{r}, veio {at_centre}"
        );

        // O ponto do arco na diagonal: distância `r` do centro, na direção da origem.
        let d = r * (1.0 - std::f64::consts::FRAC_1_SQRT_2);
        let on_surface = f.at(d, d, 0.0);
        assert!(
            on_surface.abs() < EPS,
            "superfície do filete em ({d}, {d}): esperava 0, veio {on_surface}"
        );
    }
}

/// ⭐ O outro lado, e ele é um operador **diferente** — não o mesmo com o sinal trocado.
///
/// Numa caixa arredondada o canto é um oitavo de esfera de centro `(h−r, h−r, h−r)`, e esse centro
/// fica **DENTRO** do sólido (o do filete fica fora). Logo o campo lá vale `−r`.
#[test]
fn an_exact_external_round_delivers_the_radius_asked() {
    for (half, r) in [(0.45_f64, 0.04_f64), (0.45, 0.08), (0.45, 0.2), (0.5, 0.12)] {
        let doc = FieldDoc::new(
            vec![leaf(
                Primitive::Box {
                    half: [half as f32; 3],
                    round: r as f32,
                },
                Xform::IDENTITY,
            )],
            NodeId(0),
        )
        .expect("caixa arredondada válida");
        let c = half - r;
        let v = Field::new(&doc).at(c, c, c);
        assert!(
            (v + r).abs() < EPS,
            "centro do canto de (half {half}, r {r}): esperava −{r}, veio {v}"
        );
    }
}

/// ⚠️ **O `k` do orgânico NÃO é um raio, e este gate pina o quanto ele erra.**
///
/// A conta é fechada, não empírica: no ponto em que as duas superfícies estão à mesma distância,
/// o smooth-min polinomial vale `r − k/4`. Com `k = r` isso é **3/4 do pedido** — os 25 % que a W0
/// mediu, agora derivados.
///
/// Se alguém "consertar" isto em silêncio, o gate acusa; se alguém o **calibrar** de propósito
/// (×4/3), o gate é o lugar onde a decisão fica escrita.
#[test]
fn the_organic_blend_falls_short_by_exactly_k_over_four() {
    for r in [0.05_f64, 0.1, 0.2] {
        let f = Field::new(&elbow(Blend::Organic { k: r as f32 }));
        let at_centre = f.at(r, r, 0.0);
        let expected = r - r / 4.0;
        assert!(
            (at_centre - expected).abs() < EPS,
            "orgânico com k={r}: esperava {expected} (= 3/4 de {r}), veio {at_centre}"
        );
    }
}

/// Subtração **remove material de verdade** — e o valor no centro do furo é a distância à parede
/// dele, não um sinal qualquer.
#[test]
fn a_difference_removes_material_and_the_hole_wall_is_where_it_should_be() {
    let doc = FieldDoc::new(
        vec![
            leaf(
                Primitive::Box {
                    half: [0.5; 3],
                    round: 0.0,
                },
                Xform::IDENTITY,
            ),
            leaf(
                Primitive::Cylinder {
                    radius: 0.2,
                    half_height: 2.0,
                    round: 0.0,
                },
                Xform::IDENTITY,
            ),
            combine(Op::Difference(Blend::Sharp), vec![NodeId(0), NodeId(1)]),
        ],
        NodeId(2),
    )
    .expect("caixa furada válida");
    let f = Field::new(&doc);

    let centre = f.at(0.0, 0.0, 0.0);
    assert!(
        (centre - 0.2).abs() < EPS,
        "no eixo do furo o campo tem de valer +0,2 (a parede): veio {centre}"
    );
    assert!(
        f.at(0.35, 0.0, 0.0) < 0.0,
        "fora do furo e dentro da caixa continua sendo material"
    );
}

#[test]
fn a_translation_moves_the_surface_exactly() {
    let doc = FieldDoc::new(
        vec![leaf(
            Primitive::Sphere { radius: 0.3 },
            Xform::at(0.5, -0.2, 0.1),
        )],
        NodeId(0),
    )
    .expect("esfera transladada");
    let f = Field::new(&doc);
    assert!(
        (f.at(0.5, -0.2, 0.1) + 0.3).abs() < EPS,
        "o centro andou junto"
    );
    assert!(f.at(0.8, -0.2, 0.1).abs() < EPS, "a superfície andou junto");
}

/// ⚠️ **A metade da conta que se esquece.** Escalar um nó divide o ponto por `s` **e multiplica o
/// valor** por `s`. Sem a segunda metade o campo deixa de ser distância: uma esfera de raio 1
/// escalada a 0,5 mediria −1 no centro em vez de −0,5, e toda casca e todo raio a jusante mentiriam
/// junto.
#[test]
fn a_uniform_scale_multiplies_the_distance_not_just_the_point() {
    let doc = FieldDoc::new(
        vec![leaf(
            Primitive::Sphere { radius: 1.0 },
            Xform {
                scale: 0.5,
                ..Xform::IDENTITY
            },
        )],
        NodeId(0),
    )
    .expect("esfera escalada");
    let f = Field::new(&doc);
    assert!(
        (f.at(0.0, 0.0, 0.0) + 0.5).abs() < EPS,
        "no centro tem de valer −0,5 (o raio DEPOIS da escala), veio {}",
        f.at(0.0, 0.0, 0.0)
    );
    assert!(f.at(0.5, 0.0, 0.0).abs() < EPS, "a superfície está em 0,5");
    let g = f.gradient_norm(0.4, 0.0, 0.0, 1e-4);
    assert!(
        (g - 1.0).abs() < 1e-3,
        "a escala não pode estragar ‖∇f‖ = 1: veio {g}"
    );
}

#[test]
fn a_rotation_turns_the_shape_and_keeps_the_distance() {
    // 90° em torno de Z: uma barra ao longo de X passa a estar ao longo de Y.
    let s = std::f64::consts::FRAC_1_SQRT_2 as f32;
    let doc = FieldDoc::new(
        vec![leaf(
            Primitive::Box {
                half: [0.5, 0.1, 0.1],
                round: 0.0,
            },
            Xform {
                rotation: [0.0, 0.0, s, s],
                ..Xform::IDENTITY
            },
        )],
        NodeId(0),
    )
    .expect("barra girada");
    let f = Field::new(&doc);
    assert!(f.at(0.0, 0.4, 0.0) < 0.0, "depois de girar, o comprido é Y");
    assert!(f.at(0.4, 0.0, 0.0) > 0.0, "e X passou a ser o curto");

    // ⚠️ **A sonda de gradiente NÃO pode pousar na crista medial.** O primeiro ponto que escolhi
    // aqui foi `(0, 0,4, 0)`, que dentro desta barra fica **equidistante de três faces** — e a SDF
    // não é derivável ali, por definição: é onde o `max` troca de braço. A diferença central
    // atravessa o vinco e devolve **metade** da inclinação (medido: 0,5000), o que parecia um bug
    // de rotação e era um bug de *onde eu medi*. Em `0,45` a face mais próxima é única (−0,05
    // contra −0,10), e a derivada existe.
    let g = f.gradient_norm(0.0, 0.45, 0.0, 1e-4);
    assert!((g - 1.0).abs() < 1e-3, "rotação não pode mudar ‖∇f‖: {g}");
}

/// **HR-5.** O mesmo documento compila para o mesmo campo — é o que o replay-hash do CI exige, e o
/// que impede um `HashMap` de entrar por descuido no caminho da compilação.
#[test]
fn compiling_twice_gives_the_same_field() {
    let doc = elbow(Blend::Exact { radius: 0.1 });
    let (a, b) = (Field::new(&doc), Field::new(&doc));
    for i in 0..64 {
        let t = f64::from(i) / 64.0 - 0.5;
        assert_eq!(a.at(t, t * 0.7, t * -0.3), b.at(t, t * 0.7, t * -0.3));
    }
}

/// A malha sai, é aceite pela validação da casa, e **os vértices estão sobre a superfície**.
///
/// ⚠️ Este gate mede os VÉRTICES, não o baricentro dos triângulos: um triângulo que atravessa uma
/// quina viva tem os três vértices exatos e o baricentro dentro do sólido, por geometria de cortar
/// um canto. Medir pelo baricentro misturaria "a malha errou" com "a quina existe".
#[test]
fn the_exported_mesh_sits_on_the_surface() {
    let doc = FieldDoc::new(
        vec![leaf(Primitive::Sphere { radius: 0.6 }, Xform::IDENTITY)],
        NodeId(0),
    )
    .expect("esfera");
    let f = Field::new(&doc);
    let worst_at = |depth: u8| -> f64 {
        let m = mesh(&doc, depth).expect("a malha sai e a ph2d-mesh a aceita");
        assert!(m.positions().len() > 100, "a esfera não pode sair vazia");
        m.positions()
            .iter()
            .map(|p| {
                f.at(f64::from(p[0]), f64::from(p[1]), f64::from(p[2]))
                    .abs()
            })
            .fold(0.0_f64, f64::max)
    };

    // ⚠️ **A barra deste gate foi re-derivada DUAS vezes, e o caminho até ela é o conteúdo.**
    //
    // 1ª tentativa: *"erro < 1 % da célula"* — número tirado do caso do **cubo**, onde o vértice do
    //    QEF pousa exatamente na quina. Numa superfície **curva** isso é impossível por construção:
    //    o QEF resolve o encontro de planos *tangentes*, e tangentes a uma esfera se encontram
    //    **fora** dela. Medido: 3 % da célula, não 1 %.
    // 2ª tentativa: *"o erro cai 4× ao dobrar a resolução"* (segunda ordem). Também falso — e a
    //    medição disse **por quê**, o que é o achado de verdade:
    //
    // ```text
    // prof | célula  | vértices | erro médio | erro máx
    //   4  | 0,12500 |      830 |   2,49e-3  |  7,33e-3
    //   5  | 0,06250 |    3 518 |   5,83e-4  |  1,95e-3
    //   6  | 0,03125 |   12 532 |   1,64e-4  |  1,19e-3
    //   7  | 0,01562 |   13 490 |   1,00e-4  |  7,90e-4
    // ```
    //
    // **Olhe a coluna dos vértices, não a do erro:** de 6 para 7 ela vai de 12 532 para 13 490,
    // quando quadruplicar seria o esperado. O extrator é **adaptativo**: ele para de subdividir
    // onde a superfície já está bem aproximada, e colapsa a célula. Logo `depth` é um **teto**, não
    // uma resolução — e exigir convergência de segunda ordem dele é exigir que ele desobedeça ao
    // próprio critério. (Que ele subdivide quando precisa está medido noutro lugar: a junção de
    // três cilindros dá 372 mil triângulos na profundidade 8.)
    //
    // O que **é** verdade, e portanto o que se afirma: o erro **cai monotonicamente** com a
    // profundidade, e cai bastante no conjunto do intervalo. Um extrator que parasse de convergir,
    // ou que piorasse ao refinar, reprova aqui.
    let errs: Vec<f64> = (4u8..=7).map(worst_at).collect();
    for w in errs.windows(2) {
        assert!(
            w[1] < w[0],
            "refinar não pode PIORAR a malha: {:.2e} -> {:.2e}",
            w[0],
            w[1]
        );
    }
    let total = errs[0] / errs[errs.len() - 1];
    assert!(
        total > 5.0,
        "de prof. 4 a 7 o erro tem de cair pelo menos 5×: {:.2e} -> {:.2e} é {total:.1}×",
        errs[0],
        errs[errs.len() - 1]
    );
    assert!(
        errs[1] < 0.0625 * 0.05,
        "pior vértice na prof. 5 a {} da superfície (teto: 5 % da célula)",
        errs[1]
    );
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// W3 — os perfis, e as duas formas que eles geram
//
// ⚠️ O gate decisivo desta wave é o de **oráculo independente**: um polígono regular extrudado tem
// de ser o cilindro que ele aproxima, e um polígono regular revolvido tem de ser o toro que ele
// traça. Os dois lados são código completamente diferente — a fórmula analítica de `ops.rs` e a
// soma sobre arestas de `profile.rs` —, então concordarem não é tautologia.
// ─────────────────────────────────────────────────────────────────────────────────────────────────

use ph2d_field::{FillRule, Profile};

/// Polígono regular de `n` lados **inscrito** no círculo de raio `r`, centrado em `c`, com o
/// vértice 0 no ângulo 0.
///
/// Inscrito, e não circunscrito, porque é o que torna o erro **derivável**: a flecha (sagita)
/// `r·(1 − cos(π/n))` é exatamente o quanto o meio da aresta fica aquém do círculo.
fn ngon(n: usize, r: f64, c: [f64; 2]) -> Vec<[f32; 2]> {
    (0..n)
        .map(|i| {
            let a = std::f64::consts::TAU * (i as f64) / (n as f64);
            [(c[0] + r * a.cos()) as f32, (c[1] + r * a.sin()) as f32]
        })
        .collect()
}

/// A flecha: o quanto o meio da aresta de um `n`-gono inscrito fica aquém do círculo de raio `r`.
fn sagitta(n: usize, r: f64) -> f64 {
    r * (1.0 - (std::f64::consts::PI / (n as f64)).cos())
}

fn profile_of(contours: Vec<Vec<[f32; 2]>>, fill: FillRule) -> Profile {
    Profile::new(contours, fill, 1e-3).expect("perfil de teste é válido")
}

fn doc_of(p: Primitive) -> FieldDoc {
    doc_of_posed(p, Xform::IDENTITY)
}

fn doc_of_posed(p: Primitive, x: Xform) -> FieldDoc {
    FieldDoc::new(vec![leaf(p, x)], NodeId(0)).expect("documento de uma folha")
}

/// ⭐ **O oráculo independente da extrusão.** Um `n`-gono inscrito, puxado, TEM de ser o cilindro
/// que ele aproxima — e tem de errar exatamente a flecha, nem mais nem menos.
///
/// Três afirmações, e cada uma mata um bug diferente:
///
/// 1. no **vértice**, sobre o círculo, o campo é `0` — a superfície passa onde o perfil passa;
/// 2. no **meio da aresta**, sobre o círculo, o campo é `+sagita` — o erro é o previsto pela
///    geometria, e não um número qualquer;
/// 3. sobre uma grelha 3D inteira, `|extrusão − cilindro| ≤ sagita`, o que é a distância de
///    Hausdorff entre os dois sólidos. Um sinal trocado em qualquer região reprova aqui.
#[test]
fn an_extruded_polygon_is_the_cylinder_it_approximates() {
    let (r, h) = (0.5_f64, 0.4_f64);
    for n in [8_usize, 16, 32] {
        let prof = profile_of(vec![ngon(n, r, [0.0, 0.0])], FillRule::NonZero);
        let f = Field::new(&doc_of(Primitive::Extrude {
            profile: prof,
            half_height: h as f32,
            round: 0.0,
        }));
        let s = sagitta(n, r);

        // (1) e (2): sobre o círculo, no vértice e no meio da aresta.
        let step = std::f64::consts::TAU / (n as f64);
        for k in 0..n {
            let at_angle = |a: f64| f.at(r * a.cos(), r * a.sin(), 0.0);
            let vertex = at_angle(step * (k as f64));
            assert!(
                vertex.abs() < EPS,
                "n={n}: o vértice {k} tem de estar SOBRE a superfície, e mediu {vertex:e}"
            );
            let middle = at_angle(step * (k as f64 + 0.5));
            assert!(
                (middle - s).abs() < EPS,
                "n={n}: o meio da aresta {k} tem de ficar a exatamente a flecha ({s:.6}) do \
                 círculo, e mediu {middle:.6}"
            );
        }

        // (3) o oráculo, sobre uma grelha.
        let cyl = Field::new(&doc_of(Primitive::Cylinder {
            radius: r as f32,
            half_height: h as f32,
            round: 0.0,
        }));
        let mut worst = 0.0_f64;
        for i in -6..=6 {
            for j in -6..=6 {
                for k in -6..=6 {
                    let (x, y, z) = (
                        f64::from(i) * 0.12,
                        f64::from(j) * 0.12,
                        f64::from(k) * 0.12,
                    );
                    worst = worst.max((f.at(x, y, z) - cyl.at(x, y, z)).abs());
                }
            }
        }
        assert!(
            worst <= s + EPS,
            "n={n}: o polígono e o círculo distam no máximo a flecha ({s:.6}); a pior amostra \
             deu {worst:.6}"
        );
    }
}

/// ⭐ **O oráculo independente da revolução**, e ele prova mais do que o da extrusão: a
/// substituição `x → √(x²+z²)` tem de dar a distância EXATA, não uma aproximação — e o toro
/// analítico é quem diz.
#[test]
fn a_revolved_polygon_is_the_torus_it_traces() {
    let (major, minor) = (0.6_f64, 0.2_f64);
    for n in [16_usize, 32, 64] {
        let prof = profile_of(vec![ngon(n, minor, [major, 0.0])], FillRule::NonZero);
        let f = Field::new(&doc_of(Primitive::Revolve { profile: prof }));
        // ⚠️ **Os dois eixos NÃO são o mesmo, e é de propósito.** O `Torus` da casa tem o anel no
        // plano XY (eixo de revolução = Z), como o `Cylinder`; o `Revolve` gira em torno de **Y**,
        // porque o plano de desenho do perfil é o XY e o eixo tem de estar DENTRO dele (ver a nota
        // de `Primitive::Revolve`). Comparar os dois exige pôr o toro de pé: 90° em torno de X leva
        // o anel de XY para XZ.
        //
        // Este quarto de volta é o gate: sem ele o erro medido é da ORDEM DA PEÇA (0,83 numa peça
        // de raio 0,8), e um erro dessa magnitude é a assinatura de dois eixos diferentes — não de
        // uma fórmula errada.
        let k = std::f64::consts::FRAC_1_SQRT_2 as f32;
        let torus = Field::new(&doc_of_posed(
            Primitive::Torus {
                major: major as f32,
                minor: minor as f32,
            },
            Xform {
                rotation: [k, 0.0, 0.0, k],
                ..Xform::IDENTITY
            },
        ));
        let s = sagitta(n, minor);

        // O vértice 0 do perfil está em (major + minor, 0); girado, ele é o ponto
        // (major + minor, 0, 0) — que tem de estar SOBRE a superfície.
        let on = f.at(major + minor, 0.0, 0.0);
        assert!(
            on.abs() < EPS,
            "n={n}: o vértice do perfil tem de sobreviver à revolução, e mediu {on:e}"
        );

        let mut worst = 0.0_f64;
        for i in -8..=8 {
            for j in -8..=8 {
                for k in -8..=8 {
                    let (x, y, z) = (
                        f64::from(i) * 0.13,
                        f64::from(j) * 0.13,
                        f64::from(k) * 0.13,
                    );
                    worst = worst.max((f.at(x, y, z) - torus.at(x, y, z)).abs());
                }
            }
        }
        assert!(
            worst <= s + EPS,
            "n={n}: a revolução do polígono e o toro distam no máximo a flecha ({s:.6}); a pior \
             amostra deu {worst:.6}"
        );
        assert!(
            worst > 0.1 * s,
            "n={n}: um erro de {worst:e} contra uma flecha de {s:e} é BOM DEMAIS — a grelha não \
             está a tocar a superfície, e o gate acima estaria a passar sem medir nada"
        );
    }
}

/// ⭐ **O caso que separa um sinal certo de um sinal por acaso: o entalhe de um L.**
///
/// O ponto `(1,5 · 1,5)` está **fora** da peça e cercado por ela em dois lados. Uma regra de sinal
/// tirada da aresta mais próxima erra ali; o winding number acerta. E como as arestas são todas
/// eixo-alinhadas, a distância é **exata e conhecida**, não uma tolerância.
#[test]
fn the_sign_survives_the_notch_of_a_concave_profile() {
    // (0,0) (2,0) (2,1) (1,1) (1,2) (0,2) — anti-horário, área 3.
    let l = vec![
        [0.0_f32, 0.0],
        [2.0, 0.0],
        [2.0, 1.0],
        [1.0, 1.0],
        [1.0, 2.0],
        [0.0, 2.0],
    ];
    for fill in [FillRule::NonZero, FillRule::EvenOdd] {
        let f = Field::new(&doc_of(Primitive::Extrude {
            profile: profile_of(vec![l.clone()], fill),
            half_height: 10.0,
            round: 0.0,
        }));
        // Dentro do braço de baixo, e dentro do braço da esquerda: −0,5 nos dois.
        for (x, y) in [(0.5, 0.5), (1.5, 0.5), (0.5, 1.5)] {
            let v = f.at(x, y, 0.0);
            assert!(
                (v + 0.5).abs() < EPS,
                "{fill:?}: ({x},{y}) está dentro, a 0,5 da parede, e mediu {v:.6}"
            );
        }
        // ⭐ O entalhe: FORA, apesar de ter peça a oeste e a sul.
        let notch = f.at(1.5, 1.5, 0.0);
        assert!(
            (notch - 0.5).abs() < EPS,
            "{fill:?}: o entalhe (1,5 · 1,5) está FORA, a 0,5 da peça, e mediu {notch:.6}"
        );
        // A quina reflexa é superfície.
        let corner = f.at(1.0, 1.0, 0.0);
        assert!(
            corner.abs() < EPS,
            "{fill:?}: a quina reflexa é superfície, e mediu {corner:e}"
        );
    }
}

/// ⚠️ **A aresta de FECHO existe** — e este gate a mede onde a ausência dela é visível.
///
/// Num triângulo `(0,0) (1,0) (0,1)`, a aresta que fecha é a de `x = 0`. Um ponto a oeste dela dista
/// 0,25; se o laço esquecesse o segmento último→primeiro, o mais próximo passaria a ser um
/// **vértice**, a 0,559 — mais que o dobro. É um bug clássico e ele não dá erro nenhum.
#[test]
fn the_closing_edge_of_a_contour_exists() {
    let tri = vec![[0.0_f32, 0.0], [1.0, 0.0], [0.0, 1.0]];
    let f = Field::new(&doc_of(Primitive::Extrude {
        profile: profile_of(vec![tri], FillRule::NonZero),
        half_height: 10.0,
        round: 0.0,
    }));
    let v = f.at(-0.25, 0.5, 0.0);
    assert!(
        (v - 0.25).abs() < EPS,
        "sem a aresta de fecho isto mede ~0,559 (a distância a um vértice); mediu {v:.6}"
    );
}

/// ⭐ **As duas regras de preenchimento existem, e discordam onde têm de discordar.**
///
/// Dois quadrados concêntricos com a **mesma** orientação: sob `EvenOdd` o de dentro é buraco (a
/// paridade alterna), sob `NonZero` é sólido (o enrolamento é 2). Um gate que só testasse a regra
/// default passaria com a outra por implementar — e o sintoma seria um buraco que aparece cheio,
/// em silêncio.
#[test]
fn the_two_fill_rules_disagree_exactly_where_the_winding_says_so() {
    let square = |a: f32| vec![[-a, -a], [a, -a], [a, a], [-a, a]];
    let both_ccw = vec![square(1.0), square(0.5)];

    let hole = Field::new(&doc_of(Primitive::Extrude {
        profile: profile_of(both_ccw.clone(), FillRule::EvenOdd),
        half_height: 10.0,
        round: 0.0,
    }));
    let solid = Field::new(&doc_of(Primitive::Extrude {
        profile: profile_of(both_ccw, FillRule::NonZero),
        half_height: 10.0,
        round: 0.0,
    }));
    let (h, s) = (hole.at(0.0, 0.0, 0.0), solid.at(0.0, 0.0, 0.0));
    assert!(
        (h - 0.5).abs() < EPS,
        "even-odd: o miolo é BURACO, a 0,5 da parede; mediu {h:.6}"
    );
    assert!(
        (s + 0.5).abs() < EPS,
        "non-zero com as duas voltas no mesmo sentido: o miolo é SÓLIDO; mediu {s:.6}"
    );

    // E sob `NonZero`, inverter o contorno de dentro reproduz o buraco — que é como um compound
    // path desenhado à mão o exprime.
    let mut inner_cw = square(0.5);
    inner_cw.reverse();
    let carved = Field::new(&doc_of(Primitive::Extrude {
        profile: profile_of(vec![square(1.0), inner_cw], FillRule::NonZero),
        half_height: 10.0,
        round: 0.0,
    }));
    let c = carved.at(0.0, 0.0, 0.0);
    assert!(
        (c - 0.5).abs() < EPS,
        "non-zero com o contorno de dentro invertido: buraco; mediu {c:.6}"
    );
}

/// ⭐ **O aro da extrusão entrega o raio pedido**, pelo mesmo par de pontos independentes que os
/// gates do arredondamento externo usam: o centro do arco (onde o campo vale `−r`) e um ponto do
/// próprio arco (onde vale `0`).
#[test]
fn an_extruded_rim_delivers_the_radius_asked() {
    let (a, h) = (0.5_f64, 0.4_f64);
    for r in [0.05_f64, 0.1, 0.2] {
        let square = vec![
            [-(a as f32), -(a as f32)],
            [a as f32, -(a as f32)],
            [a as f32, a as f32],
            [-(a as f32), a as f32],
        ];
        let f = Field::new(&doc_of(Primitive::Extrude {
            profile: profile_of(vec![square], FillRule::NonZero),
            half_height: h as f32,
            round: r as f32,
        }));
        // O centro do arco do aro, na secção (x, z): fica DENTRO, a `r` da superfície.
        let centre = f.at(a - r, 0.0, h - r);
        assert!(
            (centre + r).abs() < EPS,
            "r={r}: o centro do arco do aro vale −r; mediu {centre:.6}"
        );
        // E um ponto do arco, a 45°, é superfície.
        let k = std::f64::consts::FRAC_1_SQRT_2;
        let on = f.at(a - r + r * k, 0.0, h - r + r * k);
        assert!(
            on.abs() < EPS,
            "r={r}: o ponto a 45° do aro é superfície; mediu {on:e}"
        );
        // ⚠️ E o raio NÃO encolheu a peça: a parede continua em `x = a`.
        let wall = f.at(a, 0.0, 0.0);
        assert!(
            wall.abs() < EPS,
            "r={r}: arredondar o aro não pode mover a parede; mediu {wall:e}"
        );
    }
}

/// O tamanho da árvore em função do número de arestas — **é ele que manda no custo do traçado**.
///
/// `#[ignore]`: é medição, não afirmação. Roda-se quando se quer o número.
#[test]
#[ignore]
fn measure_profile_tree_size() {
    println!("arestas | nós da árvore | nós por aresta");
    for n in [8_usize, 16, 32, 64, 128, 256] {
        let prof = profile_of(vec![ngon(n, 0.5, [0.0, 0.0])], FillRule::NonZero);
        let tree = compile(&doc_of(Primitive::Extrude {
            profile: prof,
            half_height: 0.4,
            round: 0.05,
        }));
        let mut ctx = fidget::context::Context::new();
        let _ = ctx.import(&tree);
        let len = ctx.len();
        println!("{n:7} | {len:13} | {:.1}", len as f64 / n as f64);
    }
}

/// ⭐ **O campo e a pose concordam sobre onde um nó ESTÁ.**
///
/// ⚠️ São duas contas inversas uma da outra e escritas em sítios diferentes: o avaliador leva o
/// ponto para o espaço local (`p' = R⁻¹(p − t)/s`) e o [`Xform::apply`] leva um ponto local para o
/// mundo (`t + R·(s·p)`). Se elas discordarem, a alça do gizmo pousa num sítio e a superfície
/// aparece noutro — e nada fica vermelho, porque cada metade está certa sozinha.
///
/// A pose usada tem **rotação e escala**, não só translação: uma translação sozinha passaria mesmo
/// com a rotação transposta ao contrário.
#[test]
fn the_gizmo_and_the_field_agree_on_where_a_node_is() {
    let s = std::f32::consts::FRAC_1_SQRT_2;
    let xform = Xform {
        translation: [0.4, -0.25, 0.1],
        rotation: [s, 0.0, 0.0, s],
        scale: 1.5,
    };
    let doc = FieldDoc::new(
        vec![leaf(Primitive::Sphere { radius: 0.2 }, xform)],
        NodeId(0),
    )
    .expect("esfera posada");
    let f = Field::new(&doc);
    let f = |x: f32, y: f32, z: f32| f.at(f64::from(x), f64::from(y), f64::from(z));

    // O centro local da esfera é a origem: o campo ali tem de valer −raio·escala.
    let c = xform.apply([0.0, 0.0, 0.0]);
    assert!(
        (f(c[0], c[1], c[2]) + 0.3).abs() < 1e-5,
        "no centro o campo tem de valer −0,3 (raio 0,2 × escala 1,5) e vale {}",
        f(c[0], c[1], c[2])
    );

    // E um ponto NA superfície local vale zero em mundo — é aqui que a rotação entra.
    for local in [[0.2f32, 0.0, 0.0], [0.0, 0.2, 0.0], [0.0, 0.0, 0.2]] {
        let p = xform.apply(local);
        assert!(
            f(p[0], p[1], p[2]).abs() < 1e-5,
            "o ponto local {local:?} devia estar NA superfície e o campo dá {}",
            f(p[0], p[1], p[2])
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// OS MODIFICADORES — a casca e o afastamento.
// ─────────────────────────────────────────────────────────────────────────────

/// Uma esfera de raio `r`, com a pilha de modificadores dada.
fn shelled(r: f32, mods: Vec<ph2d_field::Unary>) -> FieldDoc {
    FieldDoc::new(
        vec![Node {
            xform: Xform::IDENTITY,
            kind: NodeKind::Leaf(Primitive::Sphere { radius: r }),
            mods,
        }],
        NodeId(0),
    )
    .expect("esfera com modificadores")
}

/// ⭐ **A casca de uma esfera É, exatamente, a subtração de duas esferas analíticas.**
///
/// ⚠️ **Oráculo INDEPENDENTE**, e é a disciplina desta crate (o mesmo padrão do `n`-gono extrudado
/// contra o `Cylinder`): o lado esquerdo é `|f| − t/2` sobre uma esfera; o direito é
/// `esfera(r + t/2) menos esfera(r − t/2)`, escrito com as primitivas e a booleana que já existiam.
/// Um gate que comparasse a casca **consigo mesma** provaria que o código faz o que o código faz.
///
/// A igualdade é **exata**, não aproximada, e a conta está no doc de [`ph2d_field::mods`] — é essa
/// exatidão que faz a casca não poder falhar, que é a razão de o módulo ser um campo.
#[test]
fn the_shell_of_a_sphere_is_exactly_the_difference_of_two_analytic_spheres() {
    let (r, t) = (0.5f32, 0.12f32);
    let shell = shelled(r, vec![ph2d_field::Unary::Shell { thickness: t }]);
    let oracle = FieldDoc::new(
        vec![
            leaf(
                Primitive::Sphere {
                    radius: r + t * 0.5,
                },
                Xform::IDENTITY,
            ),
            leaf(
                Primitive::Sphere {
                    radius: r - t * 0.5,
                },
                Xform::IDENTITY,
            ),
            combine(Op::Difference(Blend::Sharp), vec![NodeId(0), NodeId(1)]),
        ],
        NodeId(2),
    )
    .expect("duas esferas");

    let a = Field::new(&shell);
    let b = Field::new(&oracle);
    // Uma amostra que atravessa a parede, o vazio de dentro e o lado de fora.
    for k in 0..40 {
        let x = f64::from(k) / 20.0 - 1.0;
        for (y, z) in [(0.0, 0.0), (0.13, -0.07), (-0.3, 0.21)] {
            let (p, q) = (a.at(x, y, z), b.at(x, y, z));
            assert!(
                (p - q).abs() < EPS,
                "em ({x:.3}, {y}, {z}) a casca deu {p:.8} e as duas esferas deram {q:.8}"
            );
        }
    }
}

/// ⭐ **A parede mede a espessura PEDIDA** — medida ao longo de um raio, e não afirmada.
///
/// ⚠️ E o gate mede o **centro** da parede também: `|f| − t/2` deixa metade para dentro e metade
/// para fora da superfície que lá estava. Uma casca só-para-dentro (`|f + t| − t`) passaria na
/// primeira metade e reprovaria nesta — e é uma decisão de produto diferente, escrita no doc de
/// [`ph2d_field::Unary::Shell`].
#[test]
fn the_wall_measures_the_thickness_that_was_asked_for() {
    let (r, t) = (0.5f64, 0.12f64);
    let doc = shelled(
        r as f32,
        vec![ph2d_field::Unary::Shell {
            thickness: t as f32,
        }],
    );
    let f = Field::new(&doc);

    for (name, radius) in [
        ("parede de fora", r + t / 2.0),
        ("parede de dentro", r - t / 2.0),
    ] {
        let v = f.at(radius, 0.0, 0.0);
        assert!(
            v.abs() < EPS,
            "{name}: a superfície tinha de estar em {radius}, e o campo lá vale {v:.8}"
        );
    }
    // No meio da parede — onde a superfície original estava — o campo vale menos meia espessura.
    let mid = f.at(r, 0.0, 0.0);
    assert!(
        (mid + t / 2.0).abs() < EPS,
        "a parede é CENTRADA: no raio original o campo devia valer −t/2 ({:.4}) e vale {mid:.8}",
        -t / 2.0
    );
    // E o interior é vazio: no centro da esfera o campo é positivo.
    assert!(
        f.at(0.0, 0.0, 0.0) > 0.0,
        "o miolo tem de estar VAZIO — é isso que uma casca é"
    );
}

/// ⭐ **O afastamento move a superfície pela distância pedida** — nos dois sentidos.
///
/// ⚠️ **Os dois sentidos**, e é metade da razão de o afastamento existir: encolher é o gesto de
/// folga de encaixe, e um gate só do lado positivo passaria com o negativo recusado.
#[test]
fn the_offset_moves_the_surface_by_the_distance_asked_in_both_directions() {
    let r = 0.5f64;
    for d in [0.2f64, -0.2, 0.0] {
        let doc = shelled(
            r as f32,
            vec![ph2d_field::Unary::Offset { distance: d as f32 }],
        );
        let f = Field::new(&doc);
        let want = r + d;
        assert!(
            f.at(want, 0.0, 0.0).abs() < EPS,
            "com afastamento {d} a superfície tinha de estar em {want}, e o campo lá vale {:.8}",
            f.at(want, 0.0, 0.0)
        );
    }
}

/// ⭐ **A pilha corre NA ORDEM**, e trocar a ordem dá outra peça.
///
/// ⚠️ É a razão de ela ser uma lista e não um conjunto. O gate compara os dois arranjos no MESMO
/// ponto: se um `HashSet` tivesse entrado no lugar do `Vec`, os dois dariam o mesmo número e a
/// escolha teria sido feita em silêncio por uma ordem de iteração.
#[test]
fn the_stack_runs_in_order_and_swapping_it_gives_another_part() {
    use ph2d_field::Unary::{Offset, Shell};
    let (r, t, d) = (0.5f32, 0.1f32, 0.15f32);
    let shell_then_offset = shelled(r, vec![Shell { thickness: t }, Offset { distance: d }]);
    let offset_then_shell = shelled(r, vec![Offset { distance: d }, Shell { thickness: t }]);
    let (a, b) = (
        Field::new(&shell_then_offset),
        Field::new(&offset_then_shell),
    );
    // Encascar e depois afastar engrossa a parede em 2·d; afastar e depois encascar mantém a
    // espessura e muda o raio. No raio original os dois têm de discordar.
    let (p, q) = (a.at(f64::from(r), 0.0, 0.0), b.at(f64::from(r), 0.0, 0.0));
    assert!(
        (p - q).abs() > f64::from(d) * 0.5,
        "as duas ordens deram praticamente o mesmo ({p:.6} e {q:.6}) — a pilha não está a ser \
         percorrida na ordem, ou o gate escolheu um ponto onde elas coincidem"
    );
}

/// **Sem modificadores, o campo é BYTE a byte o de antes.**
///
/// ⚠️ É a metade que garante que a pilha não custa nada a quem não a usa — e é o controle negativo
/// dos gates acima: sem ele, um `stacked` que aplicasse sempre uma casca de zero passaria neles.
#[test]
fn an_empty_stack_leaves_the_field_untouched() {
    let with = shelled(0.5, Vec::new());
    let plain = FieldDoc::new(
        vec![leaf(Primitive::Sphere { radius: 0.5 }, Xform::IDENTITY)],
        NodeId(0),
    )
    .expect("esfera");
    let (a, b) = (Field::new(&with), Field::new(&plain));
    for k in 0..20 {
        let x = f64::from(k) / 10.0 - 1.0;
        assert_eq!(
            a.at(x, 0.07, -0.11),
            b.at(x, 0.07, -0.11),
            "uma pilha vazia tem de ser o campo intacto, em {x}"
        );
    }
}

/// ⭐ **Uma matriz de N é, exatamente, a UNIÃO de N cópias transladadas.**
///
/// ⚠️ **Oráculo independente**, como o da casca: o lado esquerdo é uma dobra do domínio (uma
/// avaliação da forma); o direito são N esferas escritas à mão, unidas com a booleana que já
/// existia. A dobra custa o mesmo com N=2 e com N=64 — a união custa N — e é **essa** a razão de a
/// matriz existir num campo.
#[test]
fn an_array_of_n_is_exactly_the_union_of_n_translated_copies() {
    use ph2d_field::Unary;
    let (r, s, n) = (0.15f32, 0.6f32, 4u32);
    let array = shelled(
        r,
        vec![Unary::Array {
            count: n,
            spacing: s,
        }],
    );
    let copies = {
        let mut nodes: Vec<Node> = (0..n)
            .map(|k| {
                leaf(
                    Primitive::Sphere { radius: r },
                    Xform::at(s * k as f32, 0.0, 0.0),
                )
            })
            .collect();
        let kids: Vec<NodeId> = (0..n).map(NodeId).collect();
        nodes.push(combine(Op::Union(Blend::Sharp), kids));
        FieldDoc::new(nodes, NodeId(n)).expect("as cópias à mão")
    };

    let (a, b) = (Field::new(&array), Field::new(&copies));
    for k in 0..60 {
        let x = f64::from(k) / 20.0 - 0.6;
        for (y, z) in [(0.0, 0.0), (0.1, -0.05), (-0.22, 0.17)] {
            let (p, q) = (a.at(x, y, z), b.at(x, y, z));
            assert!(
                (p - q).abs() < EPS,
                "em ({x:.3}, {y}, {z}) a matriz deu {p:.8} e as {n} cópias deram {q:.8}"
            );
        }
    }
}

/// ⭐ **A matriz é FINITA** — ela não repete para sempre, e o gate mede as duas pontas.
///
/// ⚠️ A receita clássica de repetição (`mod`) é **infinita**, e uma matriz infinita não é uma peça:
/// ela enche o quadro e o artista não tem como a parar. O índice preso é o que a torna um objeto.
#[test]
fn the_array_stops_at_the_count_instead_of_repeating_forever() {
    use ph2d_field::Unary;
    let (r, s, n) = (0.15f64, 0.6f64, 3u32);
    let doc = shelled(
        r as f32,
        vec![Unary::Array {
            count: n,
            spacing: s as f32,
        }],
    );
    let f = Field::new(&doc);
    // Dentro: o centro de cada cópia vale −r.
    for k in 0..n {
        let c = s * f64::from(k);
        assert!(
            (f.at(c, 0.0, 0.0) + r).abs() < EPS,
            "a cópia {k} tinha de estar em x={c}"
        );
    }
    // Fora, dos DOIS lados: onde a quarta cópia estaria, e onde estaria a de índice −1.
    for outside in [-s, s * f64::from(n)] {
        assert!(
            f.at(outside, 0.0, 0.0) > 0.0,
            "em x={outside} não pode haver peça — a matriz tem {n} cópias, não infinitas"
        );
    }
}

/// ⭐ **Uma forma FORA do centro da célula mede até à cópia mais PRÓXIMA** — e é isto que a segunda
/// célula compra.
///
/// # ⚠️ Duas coisas que este gate ensinou, e as duas eram do gate
///
/// **1. A primeira versão media `‖∇f‖ = 1` na costura, e reprovava sobre um campo CORRETO.** No
/// plano entre duas cópias está o **eixo medial**, onde uma distância assinada é legitimamente
/// não-diferenciável: `∂f/∂x` é zero ali por simetria, e a diferença central mede 0. `‖∇f‖ = 1` vale
/// *quase* em todo lado, não em todo lado — e o gate escolheu exatamente o ponto da exceção.
///
/// **2. E a fixture não continha o fenómeno.** Com uma **esfera centrada na célula**, a receita de
/// uma célula só já é exata: com `round`, a célula do ponto é sempre a do centro mais próximo, e
/// para uma forma radialmente simétrica o centro mais próximo é a cópia mais próxima. O defeito só
/// aparece com a forma **descentrada** — aqui, uma esfera pendurada num grupo. Medido: em
/// `x = 0,31` a célula própria dá **0,54** e a vizinha dá **0,06**, nove vezes menos.
///
/// ⚠️ Superestimar é o erro **caro** numa marcha de raios: o passo salta por cima da superfície, e o
/// sintoma é a peça **com buracos** — não um erro.
#[test]
fn an_off_centre_shape_still_measures_to_the_nearest_copy() {
    use ph2d_field::Unary;
    let (r, s, n, off) = (0.1f32, 0.6f32, 3u32, 0.25f32);
    // Um GRUPO com a esfera pendurada fora do centro: é o que torna a célula assimétrica.
    let arrayed = FieldDoc::new(
        vec![
            leaf(Primitive::Sphere { radius: r }, Xform::at(off, 0.0, 0.0)),
            Node {
                xform: Xform::IDENTITY,
                kind: NodeKind::Combine {
                    op: Op::Union(Blend::Sharp),
                    children: vec![NodeId(0)],
                },
                mods: vec![Unary::Array {
                    count: n,
                    spacing: s,
                }],
            },
        ],
        NodeId(1),
    )
    .expect("grupo descentrado, arrayado");
    // O oráculo: as mesmas cópias escritas à mão.
    let copies = {
        let mut nodes: Vec<Node> = (0..n)
            .map(|k| {
                leaf(
                    Primitive::Sphere { radius: r },
                    Xform::at(off + s * k as f32, 0.0, 0.0),
                )
            })
            .collect();
        nodes.push(combine(
            Op::Union(Blend::Sharp),
            (0..n).map(NodeId).collect(),
        ));
        FieldDoc::new(nodes, NodeId(n)).expect("as cópias à mão")
    };

    let (a, b) = (Field::new(&arrayed), Field::new(&copies));
    // ⚠️ A varredura passa **pelas fronteiras de célula** de propósito (`±s/2` do centro): é ali, e
    // só ali, que a célula própria deixa de ser a cópia mais próxima.
    let mut worst = 0.0f64;
    for k in 0..120 {
        let x = f64::from(k) / 50.0 - 0.5;
        let (p, q) = (a.at(x, 0.0, 0.0), b.at(x, 0.0, 0.0));
        worst = worst.max((p - q).abs());
    }
    assert!(
        worst < 1e-3,
        "a matriz descentrada afastou-se das cópias reais em {worst:.4} — a célula vizinha não está \
         a ser olhada, e a marcha vai saltar por cima da peça"
    );
}

/// ⭐ **O espelho é uma dobra EXATA do domínio** — o que existe de um lado passa a existir dos dois.
///
/// ⚠️ **A fixture tem de ter a forma FORA do plano do espelho**, e isso obriga a um grupo: a pilha
/// corre **antes** da pose do próprio nó, então deslocar a folha pela pose dela não a tira do plano
/// — a dobra veria a esfera já centrada. É a mesma armadilha da matriz, e é o uso real (espelhar um
/// **grupo** é o gesto; espelhar uma folha centrada em si é um no-op por construção).
#[test]
fn the_mirror_folds_the_domain_exactly() {
    use ph2d_field::Unary;
    let (r, off) = (0.12f64, 0.5f64);
    let doc = FieldDoc::new(
        vec![
            leaf(
                Primitive::Sphere { radius: r as f32 },
                Xform::at(off as f32, 0.0, 0.0),
            ),
            Node {
                xform: Xform::IDENTITY,
                kind: NodeKind::Combine {
                    op: Op::Union(Blend::Sharp),
                    children: vec![NodeId(0)],
                },
                mods: vec![Unary::Mirror],
            },
        ],
        NodeId(1),
    )
    .expect("grupo espelhado");
    let f = Field::new(&doc);

    // O original ficou onde estava…
    assert!(
        (f.at(off, 0.0, 0.0) + r).abs() < EPS,
        "o lado original tem de ficar intacto"
    );
    // …e há uma cópia do outro lado, no simétrico.
    assert!(
        (f.at(-off, 0.0, 0.0) + r).abs() < EPS,
        "o espelho tinha de pôr uma cópia em x = −{off}"
    );
    // E entre as duas há vazio: a dobra não preenche o meio.
    assert!(f.at(0.0, 0.0, 0.0) > 0.0, "o plano do espelho fica vazio");
}
