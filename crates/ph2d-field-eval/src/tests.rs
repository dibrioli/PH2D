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
    let errors_at = |depth: u8| -> Vec<f64> {
        let m = crate::extract::extract(&doc, &Registry::new(), depth)
            .expect("a malha sai e a ph2d-mesh a aceita");
        assert!(m.positions().len() > 100, "a esfera não pode sair vazia");
        m.positions()
            .iter()
            .map(|p| {
                f.at(f64::from(p[0]), f64::from(p[1]), f64::from(p[2]))
                    .abs()
            })
            .collect()
    };
    let mean_at = |depth: u8| -> f64 {
        let e = errors_at(depth);
        e.iter().sum::<f64>() / e.len() as f64
    };
    let worst_at = |depth: u8| -> f64 { errors_at(depth).into_iter().fold(0.0_f64, f64::max) };

    // ⚠️ **A barra deste gate foi re-derivada TRÊS vezes, e o caminho é o conteúdo.**
    //
    // 1ª: *"erro < 1 % da célula"* — número tirado do caso do **cubo**, onde o vértice do QEF pousa
    //    exatamente na quina. Numa superfície **curva** isso é impossível por construção: o QEF
    //    resolve o encontro de planos *tangentes*, e tangentes a uma esfera se encontram **fora**
    //    dela.
    // 2ª: *"o erro cai 4× ao dobrar a resolução"*. Era falsa **do extrator da `fidget`**, e a
    //    medição dizia por quê: a contagem de vértices ia de 12 532 a 12 490 entre as profundidades
    //    6 e 7, quando quadruplicar seria o esperado. Aquele extrator é **adaptativo** — ele colapsa
    //    a célula onde a superfície já está bem aproximada —, logo `depth` era um **teto** e não uma
    //    resolução, e exigir segunda ordem dele era exigir que desobedecesse ao próprio critério.
    // 3ª: ⭐ **com o extrator da casa (W20) a 2ª volta a ser VERDADE**, porque a grade é uniforme e
    //    `depth` é literalmente a resolução. Medido (`probe_sphere_convergence`):
    //
    // ```text
    // prof | célula  | vértices | erro médio | erro máx | máx/célula
    //   4  | 0,12500 |      416 |  3,971e-3  | 6,702e-3 |   0,0536
    //   5  | 0,06250 |    1 760 |  9,353e-4  | 1,750e-3 |   0,0280
    //   6  | 0,03125 |    6 920 |  2,586e-4  | 5,364e-4 |   0,0172
    //   7  | 0,01562 |   27 824 |  6,508e-5  | 1,750e-4 |   0,0112
    //   8  | 0,00781 |  111 080 |  1,686e-5  | 1,200e-4 |   0,0154
    // ```
    //
    // Os vértices quadruplicam (×4,2 · ×3,9 · ×4,0 · ×4,0) e o erro médio **cai por 4** a cada
    // degrau — segunda ordem, que é o que uma superfície lisa deve dar. *A nota que dizia o
    // contrário não era um erro de medição: era uma medição correta de OUTRO extrator.*
    // ⚠️ **4ª: o gate media o MÁXIMO e a lei que ele cita é a do MÉDIO** — e a última coluna da
    // tabela acima já o dizia: o máximo cai ×3,8 · ×3,3 · ×3,1 e depois **×1,46**, porque um máximo
    // sobre 4× mais vértices amostra melhor a própria cauda. Ele passava por a varredura parar na
    // profundidade 7; a W33 (a caixa da grade passou a ser a da peça) encolheu a célula e trouxe o
    // regime da cauda para dentro da varredura. *Cada estatística mede-se pela lei que ela obedece:*
    //
    // - o **médio** cai por 4 (segunda ordem) — aqui exigido ≥ 2× por degrau, com folga;
    // - o **máximo** é uma fração da CÉLULA (a coluna `máx/célula`: 0,011 a 0,054), e é isso que ele
    //   tem de continuar a ser — uma barra livre de escala, que uma caixa nova não invalida.
    let means: Vec<f64> = (4u8..=7).map(mean_at).collect();
    for w in means.windows(2) {
        assert!(
            w[1] < w[0] * 0.5,
            "dobrar a resolução tem de cortar o erro MÉDIO ao meio, pelo menos: {:.2e} -> {:.2e}",
            w[0],
            w[1]
        );
    }
    let total = means[0] / means[means.len() - 1];
    assert!(
        total > 20.0,
        "de prof. 4 a 7 o erro médio tem de cair pelo menos 20x: {:.2e} -> {:.2e} e {total:.1}x",
        means[0],
        means[means.len() - 1]
    );

    // ⭐ **O máximo, contra a CÉLULA** — a barra livre de escala. A tabela mede 0,011 a 0,054;
    // 10 % dá quase o dobro de folga sobre o pior degrau medido, e é ela que apanha um vértice
    // fugido da célula (o defeito que a W20 curou) em qualquer caixa.
    for depth in 4u8..=7 {
        let cell = crate::extract::cell_size(&doc, &Registry::new(), depth);
        let w = worst_at(depth);
        assert!(
            w < 0.10 * cell,
            "prof. {depth}: o pior vértice está a {w:.2e} da superfície, {:.3} da célula (teto:              0,100)",
            w / cell
        );
    }

    // ⚠️ **E a contagem de vértices é metade do gate.** Sem ela, um extrator que voltasse a colapsar
    // células passaria na coluna do erro (colapsar onde já está bom não piora o erro) e a nota da
    // 2ª volta reapareceria sem ninguém a notar.
    let verts: Vec<usize> = (5u8..=7)
        .map(|d| {
            crate::extract::extract(&doc, &Registry::new(), d)
                .expect("malha")
                .positions()
                .len()
        })
        .collect();
    for w in verts.windows(2) {
        let ratio = w[1] as f64 / w[0] as f64;
        assert!(
            (3.5..4.5).contains(&ratio),
            "a grade e UNIFORME: dobrar a resolucao tem de quadruplicar os vertices, e deu {ratio:.2}x ({} -> {})",
            w[0],
            w[1]
        );
    }
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
/// ponto: se um conjunto tivesse entrado no lugar do `Vec`, os dois dariam o mesmo número e a
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

/// ⭐ **Uma coroa de N é, exatamente, a UNIÃO de N cópias ROTACIONADAS.**
///
/// ⚠️ Oráculo independente, como o da matriz linear: o lado direito são N esferas postas à mão nos
/// ângulos certos, unidas com a booleana de sempre. A dobra do ângulo custa o mesmo com N = 2 e com
/// N = 32; a união custa N.
#[test]
fn a_radial_array_of_n_is_exactly_the_union_of_n_rotated_copies() {
    use ph2d_field::Unary;
    let (r, ring, n) = (0.22f32, 0.5f32, 6u32);
    // ⚠️ **A esfera fica a MEIA FATIA do centro dela**, e não sobre ela. Uma peça centrada na fatia
    // não contém o fenómeno: com `round`, a fatia do ponto é a do centro mais próximo, e para uma
    // forma radialmente simétrica **centrada** isso já é a cópia mais próxima — a receita de uma
    // fatia só passaria. É a mesma armadilha da matriz linear, na coordenada angular.
    let half_wedge = std::f32::consts::TAU / n as f32 * 0.5;
    let (hs, hc) = half_wedge.sin_cos();
    let arrayed = FieldDoc::new(
        vec![
            leaf(
                Primitive::Sphere { radius: r },
                Xform::at(ring * hc, ring * hs, 0.0),
            ),
            Node {
                xform: Xform::IDENTITY,
                kind: NodeKind::Combine {
                    op: Op::Union(Blend::Sharp),
                    children: vec![NodeId(0)],
                },
                mods: vec![Unary::Radial { count: n }],
            },
        ],
        NodeId(1),
    )
    .expect("coroa");
    let copies = {
        let step = std::f32::consts::TAU / n as f32;
        let mut nodes: Vec<Node> = (0..n)
            .map(|k| {
                let (s, c) = step.mul_add(k as f32, half_wedge).sin_cos();
                leaf(
                    Primitive::Sphere { radius: r },
                    Xform::at(ring * c, ring * s, 0.0),
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
    let mut worst = 0.0f64;
    for k in 0..60 {
        let t = f64::from(k) / 60.0 * std::f64::consts::TAU;
        // ⚠️ A varredura passa **pelas fronteiras de fatia** de propósito: é ali, e só ali, que a
        // fatia do ponto deixa de ser a cópia mais próxima.
        for radius in [0.0f64, 0.25, 0.5, 0.75] {
            let (x, y) = (radius * t.cos(), radius * t.sin());
            worst = worst.max((a.at(x, y, 0.07) - b.at(x, y, 0.07)).abs());
        }
    }
    assert!(
        worst < 1e-3,
        "a coroa afastou-se das {n} cópias reais em {worst:.4}"
    );
}

/// ⚠️ **No eixo da coroa não há ângulo, e o campo tem de responder na mesma.**
///
/// `atan2(0, 0)` é indefinido em matemática e uma escolha em `f32`. A conta desta crate não divide
/// por `r` — ela reconstrói o ponto por `r·cos θ'` —, então em `r = 0` o resultado é a origem, sem
/// caso especial e sem `NaN`. Um `NaN` aqui envenenaria o traçado **inteiro**: a marcha compara com
/// `NaN` e nenhum pixel acerta.
#[test]
fn the_radial_axis_answers_instead_of_producing_a_nan() {
    use ph2d_field::Unary;
    let doc = FieldDoc::new(
        vec![
            leaf(Primitive::Sphere { radius: 0.1 }, Xform::at(0.5, 0.0, 0.0)),
            Node {
                xform: Xform::IDENTITY,
                kind: NodeKind::Combine {
                    op: Op::Union(Blend::Sharp),
                    children: vec![NodeId(0)],
                },
                mods: vec![Unary::Radial { count: 8 }],
            },
        ],
        NodeId(1),
    )
    .expect("coroa");
    let f = Field::new(&doc);
    for z in [-0.3f64, 0.0, 0.3] {
        let v = f.at(0.0, 0.0, z);
        assert!(v.is_finite(), "no eixo (z={z}) o campo deu {v}");
        // E no eixo há VAZIO: a coroa tem um buraco no meio, por construção.
        assert!(v > 0.0, "o meio da coroa é vazio, e em z={z} veio {v}");
    }
}

/// ⚠️ **A tabela que escolhe o teto da inclinação** — `‖∇f‖` e o custo, por declive.
///
/// A condição que importa é `‖∇f‖ ≤ 1`: acima dela o campo **superestima** e a marcha salta por
/// cima da peça. O custo é o inverso — quanto mais conservador o bound, mais passos.
#[test]
#[ignore = "medição, não gate — corre com --ignored --nocapture"]
fn measure_taper_cost() {
    use ph2d_field::Unary;
    println!("declive | max ‖∇f‖ | min ‖∇f‖ (= fração do passo)");
    for slope in [0.0f32, 0.1, 0.25, 0.5, 0.75, 1.0, 1.5, 2.0] {
        let doc = shelled(0.4, vec![Unary::Taper { slope }]);
        let f = Field::new(&doc);
        let (mut hi, mut lo) = (0.0f64, f64::INFINITY);
        for i in 0..25 {
            for j in 0..25 {
                for k in 0..25 {
                    let p = |n: i32| f64::from(n) / 12.0 - 1.0;
                    let (x, y, z) = (p(i), p(j), p(k));
                    let g = f.gradient_norm(x, y, z, 1e-3);
                    if g.is_finite() && g > 1e-6 {
                        hi = hi.max(g);
                        lo = lo.min(g);
                    }
                }
            }
        }
        println!("{slope:7.2} | {hi:9.4} | {lo:9.4}");
    }
}

/// ⭐ **A inclinação NUNCA superestima a distância** — a condição que impede a marcha de atravessar
/// a peça.
///
/// ⚠️ **É o gate que refutou a minha primeira conta.** A correção derivada à mão (`1/(1+|s|)`)
/// deixava `‖∇f‖` acima de 1 em **todo** o alcance — 1,12 · 1,20 · 1,30 — ou seja o campo
/// superestimava, que é exatamente a falha que a correção existe para evitar. O `2` do divisor saiu
/// da tabela, não da álgebra.
///
/// ⚠️ Este é o **primeiro operador não-exato** do módulo, e o gate mede o que sobrou: ele não é
/// exato, mas é **conservador** — e conservador é seguro.
#[test]
fn the_taper_never_overestimates_the_distance() {
    use ph2d_field::{Unary, mods::MAX_TAPER_SLOPE};
    for slope in [0.1f32, 0.25, 0.5, MAX_TAPER_SLOPE] {
        let doc = shelled(0.4, vec![Unary::Taper { slope }]);
        let f = Field::new(&doc);
        let mut worst = 0.0f64;
        for i in 0..17 {
            for j in 0..17 {
                for k in 0..17 {
                    let p = |n: i32| f64::from(n) / 8.0 - 1.0;
                    let g = f.gradient_norm(p(i), p(j), p(k), 1e-3);
                    if g.is_finite() {
                        worst = worst.max(g);
                    }
                }
            }
        }
        assert!(
            worst <= 1.0 + 1e-3,
            "declive {slope}: ‖∇f‖ chegou a {worst:.4} — acima de 1 o campo SUPERESTIMA e a marcha \
             salta por cima da superfície"
        );
    }
}

/// ⭐ **A inclinação de facto INCLINA** — e o gate mede os dois lados, porque o sinal é metade da
/// ferramenta.
///
/// ⚠️ Sem a metade negativa, um `abs()` a mais na lei passaria: a peça afinaria para cima nos dois
/// casos e o artista nunca conseguiria a forma oposta.
#[test]
fn the_taper_narrows_one_way_and_widens_the_other() {
    use ph2d_field::Unary;
    let r = 0.4f64;
    // A largura da secção a uma altura: o `x` onde o campo cruza zero.
    let width_at = |slope: f32, y: f64| -> f64 {
        let doc = shelled(r as f32, vec![Unary::Taper { slope }]);
        let f = Field::new(&doc);
        let (mut lo, mut hi) = (0.0f64, 3.0f64);
        for _ in 0..40 {
            let mid = 0.5 * (lo + hi);
            if f.at(mid, y, 0.0) < 0.0 {
                lo = mid
            } else {
                hi = mid
            }
        }
        0.5 * (lo + hi)
    };
    let s = 0.5f32;
    let (below, above) = (width_at(s, -0.2), width_at(s, 0.2));
    assert!(
        above > below * 1.1,
        "declive positivo tem de ALARGAR para cima: {below:.4} em baixo, {above:.4} em cima"
    );
    let (below, above) = (width_at(-s, -0.2), width_at(-s, 0.2));
    assert!(
        above < below * 0.9,
        "declive negativo tem de AFINAR para cima: {below:.4} em baixo, {above:.4} em cima"
    );
    // E o zero é o ponto neutro: a peça intacta.
    assert!(
        (width_at(0.0, 0.2) - width_at(0.0, -0.2)).abs() < 1e-4,
        "declive zero não pode inclinar nada"
    );
}

/// ⭐ **A COSTURA de um torno não é uma parede** — e o campo dentro do sólido tem de o saber.
///
/// ⚠️ **O mecanismo, e ele era invisível na tela.** Um contorno desenhado tem de fechar, e num vaso
/// ele fecha **descendo pelo eixo**. Essa aresta existe no desenho e varre uma **linha** ao girar —
/// medida zero, superfície nenhuma. Enquanto ela contava para a distância, o campo lia `f = 0` ao
/// longo do eixo **dentro** do sólido: um nível zero fantasma, que a extração encontrava e malhava.
/// A peça traçada não o mostrava (o traçado bate na parede externa primeiro); quem o via era a malha
/// exportada, em lascas junto ao fundo.
///
/// O gate mede as duas metades que separam "curado" de "mascarado": o valor **e** o gradiente. Um
/// campo que devolvesse a distância certa com `‖∇f‖ = 0` continuaria a não ser uma distância.
#[test]
fn the_seam_of_a_lathe_lies_on_the_axis_and_is_not_a_wall() {
    // Um copo: fundo em y = −0,45, parede até y = 0,3, interior a descer até y = −0,25, e a costura
    // a fechar pelo eixo de (0, −0,25) a (0, −0,45).
    let cup = profile_of(
        vec![vec![
            [0.00, -0.45],
            [0.30, -0.45],
            [0.30, 0.30],
            [0.24, 0.30],
            [0.24, -0.25],
            [0.00, -0.25],
        ]],
        FillRule::NonZero,
    );
    let f = Field::new(&doc_of(Primitive::Revolve { profile: cup }));

    // ⚠️ **`y = −0,35` está de fora de propósito**: ali o fundo (−0,45) e o chão interno (−0,25)
    // ficam à MESMA distância, que é a superfície medial — onde a distância não é diferenciável e
    // `‖∇f‖` medido por diferença central dá **zero** sobre um campo perfeitamente correto. É a
    // segunda vez que esta linha paga esse pedágio; a cura é a fixture, nunca a barra.
    for y in [-0.43, -0.40, -0.28] {
        let d = f.at(0.0, y, 0.0);
        let want = -(y + 0.45).min(-0.25 - y);
        assert!(
            (d - want).abs() < 1e-3,
            "no eixo, y = {y}: o campo leu {d:.4} e a parede mais próxima está a {want:.4} — a \
             costura do desenho está a ser tratada como parede"
        );
        assert!(
            (f.gradient_norm(0.0, y, 0.0, 1e-4) - 1.0).abs() < 1e-2,
            "e ‖∇f‖ tem de valer 1 no eixo: um valor certo com gradiente nulo é um platô, não uma \
             distância"
        );
    }

    // ⚠️ E o controlo positivo: a costura continua a fechar o contorno para o SINAL. Um ponto
    // dentro do material é negativo; um ponto no oco do copo, acima do fundo, é positivo.
    assert!(f.at(0.27, 0.0, 0.0) < 0.0, "a parede é material");
    assert!(f.at(0.0, 0.0, 0.0) > 0.0, "o oco do copo é vazio");
    assert!(f.at(0.0, -0.44, 0.0) < 0.0, "e o fundo é material");
}

/// Sonda descartável — a convergência do extrator da casa numa esfera.
#[test]
#[ignore = "medição"]
fn probe_sphere_convergence() {
    let doc = FieldDoc::new(
        vec![leaf(Primitive::Sphere { radius: 0.6 }, Xform::IDENTITY)],
        NodeId(0),
    )
    .expect("esfera");
    let f = Field::new(&doc);
    println!("prof | célula  | vértices | erro médio | erro máx | máx/célula");
    for depth in 4u8..=8 {
        let cell = 2.0 / f64::from(1u32 << depth);
        let m = crate::extract::extract(&doc, &Registry::new(), depth).expect("malha");
        let errs: Vec<f64> = m
            .positions()
            .iter()
            .map(|p| {
                f.at(f64::from(p[0]), f64::from(p[1]), f64::from(p[2]))
                    .abs()
            })
            .collect();
        let mean = errs.iter().sum::<f64>() / errs.len() as f64;
        let max = errs.iter().fold(0.0f64, |a, b| a.max(*b));
        println!(
            "{depth:4} | {cell:.5} | {:8} | {mean:10.3e} | {max:8.3e} | {:10.4}",
            m.positions().len(),
            max / cell
        );
    }
}

/// Sonda — a QUINA VIVA: existe vértice de malha sobre a aresta ideal do cubo?
///
/// Réplica exata do método da W0 §2.1 (`01_resultados_spike.md`), para que os números sejam
/// comparáveis linha a linha: a aresta é fatiada em faixas de uma célula, e uma faixa conta como
/// **capturada** quando existe vértice a menos de ¼ de célula da aresta ideal. O achado de lá foi
/// que o desvio **é igual à fração de célula em que a face cai** — 0,10 → 0,10 · 0,50 → 0,50 ·
/// 0,80 → 0,80 —, com `0/49` faixas capturadas.
#[test]
#[ignore = "medição"]
fn probe_sharp_edge_capture() {
    println!(
        "meia-aresta | prof | célula  | face em células | fração | desvio médio | pior | capturadas"
    );
    for (half, depth) in [
        (0.5f64, 6u8),
        (0.45, 6),
        (0.4703125, 6),
        (0.4609375, 6),
        (0.45, 7),
        (0.45, 8),
        (0.25, 7),
    ] {
        let cell = 2.0 / f64::from(1u32 << depth);
        let doc = FieldDoc::new(
            vec![leaf(
                Primitive::Box {
                    half: [half as f32; 3],
                    round: 0.0,
                },
                Xform::IDENTITY,
            )],
            NodeId(0),
        )
        .expect("cubo");
        let m = crate::extract::extract(&doc, &Registry::new(), depth).expect("malha");

        // A aresta ideal: x = half, y = half, z livre em [−half, half].
        let mut best = vec![f64::INFINITY; (2.0 * half / cell).ceil() as usize + 1];
        for p in m.positions() {
            let (x, y, z) = (f64::from(p[0]), f64::from(p[1]), f64::from(p[2]));
            if z < -half || z > half {
                continue;
            }
            let d = ((x - half).powi(2) + (y - half).powi(2)).sqrt();
            let slot = ((z + half) / cell) as usize;
            if let Some(s) = best.get_mut(slot) {
                *s = s.min(d);
            }
        }
        let finite: Vec<f64> = best.iter().copied().filter(|d| d.is_finite()).collect();
        let mean = finite.iter().sum::<f64>() / finite.len().max(1) as f64 / cell;
        let worst = finite.iter().fold(0.0f64, |a, b| a.max(*b)) / cell;
        let caught = finite.iter().filter(|d| **d < cell * 0.25).count();
        let in_cells = half / cell;
        println!(
            "{half:11.7} | {depth:4} | {cell:.5} | {in_cells:15.2} | {:6.2} | {mean:12.2} | \
             {worst:4.2} | {caught:5}/{}",
            in_cells.fract(),
            finite.len()
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// W20 — a extração da casa. ⚠️ Os três gates abaixo são a resposta ao smoke que reprovou a malha
// exportada ("baixa qualidade, sobreposição de faces"), e cada um mede uma metade DIFERENTE:
// a face dobrada · a topologia · a quina viva. Um só deles passaria com a malha errada.
// ─────────────────────────────────────────────────────────────────────────────────────────────────

/// As peças que os gates da extração medem.
///
/// ⚠️ **QUATRO famílias, e cada uma existe por uma medição.** Primitiva pura e booleana com filete
/// cobrem caminhos de campo diferentes; o **cubo vivo** é o único que exibe a quina; e as duas de
/// **perfil desenhado** (extrusão e torno) são as únicas em que o quad chega a ser côncavo — sem
/// elas, a regra da diagonal de [`crate::extract`] passa verde ao ser mutada, que é o modo de falha
/// que esta linha já pagou quatro vezes.
fn extraction_fixtures() -> Vec<(&'static str, FieldDoc)> {
    let cup = profile_of(
        vec![vec![
            [0.00, -0.45],
            [0.30, -0.45],
            [0.30, 0.30],
            [0.24, 0.30],
            [0.24, -0.25],
            [0.00, -0.25],
        ]],
        FillRule::NonZero,
    );
    let cyl = |rot: [f32; 4]| {
        leaf(
            Primitive::Cylinder {
                radius: 0.22,
                half_height: 0.78,
                round: 0.05,
            },
            Xform {
                rotation: rot,
                ..Xform::IDENTITY
            },
        )
    };
    let s = std::f32::consts::FRAC_1_SQRT_2;
    vec![
        (
            "caixa arredondada",
            doc_of(Primitive::Box {
                half: [0.45; 3],
                round: 0.08,
            }),
        ),
        (
            "cubo vivo",
            doc_of(Primitive::Box {
                half: [0.45; 3],
                round: 0.0,
            }),
        ),
        ("esfera", doc_of(Primitive::Sphere { radius: 0.6 })),
        (
            "junção de 3 cilindros com filete",
            FieldDoc::new(
                vec![
                    cyl([0.0, 0.0, 0.0, 1.0]),
                    cyl([s, 0.0, 0.0, s]),
                    cyl([0.0, s, 0.0, s]),
                    Node::new(
                        Xform::IDENTITY,
                        ph2d_field::NodeKind::Combine {
                            op: Op::Union(Blend::Exact { radius: 0.12 }),
                            children: vec![NodeId(0), NodeId(1), NodeId(2)],
                        },
                    ),
                ],
                NodeId(3),
            )
            .expect("junção"),
        ),
        (
            "extrusão de um perfil desenhado",
            doc_of(Primitive::Extrude {
                profile: profile_of(vec![ngon(9, 0.42, [0.0, 0.0])], FillRule::NonZero),
                half_height: 0.3,
                round: 0.04,
            }),
        ),
        ("torno em copo", doc_of(Primitive::Revolve { profile: cup })),
    ]
}

/// ⭐ **NENHUMA face sai virada do avesso** — o gate do defeito que o smoke da W19 apanhou.
///
/// # ⚠️ O oráculo é a média das normais nos VÉRTICES, e chegar a isso custou duas refutações
///
/// 1. *`∇f` no baricentro* — o baricentro não está sobre o nível zero; numa parede fina ele cai
///    dentro do material, e ali o gradiente aponta para a face **do outro lado**.
/// 2. *`∇f` na superfície mais próxima do baricentro* (baricentro + Newton) — cura a parede fina e
///    **reprova a quina viva**: num canto de 90° a projeção pousa numa das duas faces, enquanto um
///    quad que atravessa o canto tem a normal **entre** elas. Medido no copo do torno: 32 faces
///    corretas lidas como dobradas, com `n̂ = (0, 0,94, 0,35)` contra `ĝ = (−0,33, 0, −0,94)` — os
///    dois vetores certos, de duas faces diferentes.
///
/// Os **vértices**, esses, estão sobre a superfície por construção (`|f| ≤ 1e-4 aqui`), e a média
/// das três normais é justamente a direção "entre as faces" que um triângulo de canto tem. Uma face
/// realmente invertida continua a dar `n̂ · ĝ ≈ −1` contra ela.
///
/// *É a terceira vez que esta linha aprende a mesma coisa: **onde o campo não é liso, o oráculo é o
/// que reprova primeiro.***
#[test]
fn the_exported_mesh_never_folds_a_face() {
    for (name, doc) in extraction_fixtures() {
        let f = Field::new(&doc);
        for depth in [6u8, 7] {
            let cell = 2.0 / f64::from(1u32 << depth);
            let m = crate::extract::extract(&doc, &Registry::new(), depth).expect("malha");
            let pos = m.positions();
            let mut folded = 0usize;
            let mut worst = f64::INFINITY;
            let mut tri = Vec::new();
            for face in m.faces() {
                tri.clear();
                face.triangles(&mut tri);
                for t in &tri {
                    let v = t.map(|i| pos[i as usize].map(f64::from));
                    let e1: [f64; 3] = std::array::from_fn(|k| v[1][k] - v[0][k]);
                    let e2: [f64; 3] = std::array::from_fn(|k| v[2][k] - v[0][k]);
                    let n = [
                        e1[1] * e2[2] - e1[2] * e2[1],
                        e1[2] * e2[0] - e1[0] * e2[2],
                        e1[0] * e2[1] - e1[1] * e2[0],
                    ];
                    let area2 = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
                    if area2 == 0.0 {
                        folded += 1;
                        continue;
                    }
                    let mut g = [0.0f64; 3];
                    for p in v {
                        let gv = normal_at(&f, p, cell / 8.0);
                        for k in 0..3 {
                            g[k] += gv[k];
                        }
                    }
                    let gl = (g[0] * g[0] + g[1] * g[1] + g[2] * g[2]).sqrt();
                    if gl <= 0.0 {
                        continue;
                    }
                    let cos = (n[0] * g[0] + n[1] * g[1] + n[2] * g[2]) / (area2 * gl);
                    worst = worst.min(cos);
                    if cos < 0.0 {
                        folded += 1;
                    }
                }
            }
            assert_eq!(
                folded, 0,
                "{name}, prof. {depth}: {folded} triângulos com a normal ao contrário (pior \
                 alinhamento {worst:.3}) — em shade smooth isso é uma mancha escura"
            );
        }
    }
}

/// `∇f` normalizado em `p`, por diferença central.
fn normal_at(f: &Field, p: [f64; 3], eps: f64) -> [f64; 3] {
    let d = |i: usize| {
        let (mut a, mut b) = (p, p);
        a[i] += eps;
        b[i] -= eps;
        (f.at(a[0], a[1], a[2]) - f.at(b[0], b[1], b[2])) / (2.0 * eps)
    };
    let g = [d(0), d(1), d(2)];
    let l = (g[0] * g[0] + g[1] * g[1] + g[2] * g[2]).sqrt();
    if l > 0.0 && l.is_finite() {
        [g[0] / l, g[1] / l, g[2] / l]
    } else {
        [0.0; 3]
    }
}

/// ⭐ **A malha é SÓLIDA e é feita de QUADS** — topologia e formato, que são coisas diferentes.
///
/// ⚠️ **Toda aresta tem exatamente duas faces.** Uma com uma é um buraco; uma com três é uma aba
/// não-manifold, que nenhum *remesh* nem impressora 3D come. E os vértices coincidentes têm de ser
/// zero: dois iguais são um triângulo de área zero à espera, e foi assim que a prisão à parede da
/// célula se traiu antes do recuo ([`crate::extract`]).
#[test]
fn the_exported_mesh_is_a_watertight_quad_grid() {
    use std::collections::{BTreeMap, BTreeSet};
    for (name, doc) in extraction_fixtures() {
        let m = crate::extract::extract(&doc, &Registry::new(), 6).expect("malha");
        assert!(m.faces().len() > 100, "{name}: a malha saiu vazia");

        let quads = m.faces().iter().filter(|f| !f.is_tri()).count();
        assert_eq!(
            quads,
            m.faces().len(),
            "{name}: {} faces não são quads — a saída deste extrator é uma grade de quads",
            m.faces().len() - quads
        );

        let mut seen = BTreeSet::new();
        for p in m.positions() {
            assert!(
                seen.insert(p.map(f32::to_bits)),
                "{name}: dois vértices na mesma posição"
            );
        }

        let mut inc: BTreeMap<(u32, u32), u32> = BTreeMap::new();
        for f in m.faces() {
            let v = f.verts();
            for k in 0..v.len() {
                let (a, b) = (v[k], v[(k + 1) % v.len()]);
                *inc.entry((a.min(b), a.max(b))).or_default() += 1;
            }
        }
        let bad = inc.values().filter(|&&n| n != 2).count();
        assert_eq!(
            bad, 0,
            "{name}: {bad} arestas com incidência ≠ 2 — a malha não fecha"
        );
    }
}

/// ⭐ **A QUINA VIVA sai viva** — o kill-criterion nº 1 da W0, e ele estava aberto desde então.
///
/// ⚠️ **O mecanismo não era o extrator, era `sqrt(0)`.** `box_raw` é `length3(max(q,0)…)`, e dentro
/// da peça inteira os três termos valem zero: o gradiente automático de `sqrt` em zero é infinito,
/// devolve `NaN`, a célula fica sem QEF e o vértice cai no baricentro das travessias. O desvio que a
/// W0 mediu (`0/49` faixas capturadas, e *"o desvio é igual à fração de célula em que a face cai"*)
/// era literalmente esse baricentro — `0,72 × fração`. A cura é [`crate::ops::safe_sqrt`].
///
/// ⚠️ **A meia-aresta varia de propósito**, e é isso que torna o gate uma prova: era exatamente a
/// FRAÇÃO de célula em que a face cai que governava o erro, então uma medida numa fração só passaria
/// por sorte. `0,5` põe a face **em cima** da linha da grade, que é o caso degenerado.
#[test]
fn a_live_edge_lands_on_the_edge_not_on_the_grid() {
    for (half, depth) in [
        (0.5f64, 6u8),
        (0.45, 6),
        (0.4609375, 6),
        (0.45, 7),
        (0.25, 7),
    ] {
        let cell = 2.0 / f64::from(1u32 << depth);
        let doc = doc_of(Primitive::Box {
            half: [half as f32; 3],
            round: 0.0,
        });
        let m = crate::extract::extract(&doc, &Registry::new(), depth).expect("malha");

        // A aresta ideal x = half, y = half, fatiada em faixas de uma célula ao longo de z.
        let slots = (2.0 * half / cell).ceil() as usize + 1;
        let mut best = vec![f64::INFINITY; slots];
        for p in m.positions() {
            let (x, y, z) = (f64::from(p[0]), f64::from(p[1]), f64::from(p[2]));
            if z < -half || z > half {
                continue;
            }
            let d = ((x - half).powi(2) + (y - half).powi(2)).sqrt();
            #[allow(clippy::cast_sign_loss)]
            let slot = ((z + half) / cell) as usize;
            if let Some(s) = best.get_mut(slot) {
                *s = s.min(d);
            }
        }
        let seen: Vec<f64> = best.iter().copied().filter(|d| d.is_finite()).collect();
        assert!(
            seen.len() > 10,
            "meia-aresta {half}: a aresta nem foi coberta"
        );
        let worst = seen.iter().fold(0.0f64, |a, b| a.max(*b)) / cell;
        assert!(
            worst < 0.05,
            "meia-aresta {half}, prof. {depth}: a pior faixa da aresta está a {worst:.2} célula do \
             fio (teto: 0,05). A W0 media 0,80 — se isto voltou, o gradiente do campo voltou a ser \
             NaN sobre a superfície"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// W21 — o avaliador HÍBRIDO. ⚠️ O gate decisivo é o de **paridade**: a booleana existe duas vezes
// (como árvore e como aritmética `f32`), porque um `min` entre uma fita de JIT e uma grade de voxels
// não cabe dentro de nenhuma das duas. Dois motores, uma lei — e aqui está o juiz.
// ─────────────────────────────────────────────────────────────────────────────────────────────────

use crate::hybrid::{Hybrid, Registry, Sampled};

/// Um campo "amostrado" que na verdade é **analítico** — o duplo que torna a paridade mensurável.
///
/// ⚠️ **É isto que faz o gate de paridade existir.** Comparar o caminho híbrido com o da árvore exige
/// que os dois consigam representar a MESMA peça, e uma grade de voxels nunca é bit-a-bit igual a uma
/// fórmula. Com um duplo exato, toda a diferença que sobrar é **da lei**, que é o que se quer medir.
struct AnalyticSphere {
    radius: f32,
}

impl Sampled for AnalyticSphere {
    fn at(&self, p: [f32; 3]) -> f32 {
        (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt() - self.radius
    }

    fn bounding_radius(&self) -> f32 {
        self.radius
    }
}

/// Uma varredura determinística do cubo `[-1, 1]`.
fn grid_points(n: usize) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
    let (mut xs, mut ys, mut zs) = (Vec::new(), Vec::new(), Vec::new());
    for i in 0..n {
        let t = i as f32 / n as f32;
        xs.push((t * 37.0).sin() * 0.9);
        ys.push((t * 41.0).cos() * 0.9);
        zs.push((t * 43.0).sin() * 0.9);
    }
    (xs, ys, zs)
}

/// ⭐ **Um documento SEM escultura continua a ser uma fita só** — o caminho rápido não regride.
///
/// ⚠️ **É metade do desenho, e sem gate ela apodrece em silêncio.** Se a fusão parasse de funcionar,
/// cada `Combine` viraria uma fita de JIT própria e a booleana passaria a acontecer em `f32` fora
/// delas: o resultado continuaria **certo** (é a mesma lei) e o traçado ficaria várias vezes mais
/// lento, sem um único teste vermelho.
#[test]
fn an_all_analytic_document_stays_one_tape() {
    let reg = Registry::new();
    for (name, doc) in extraction_fixtures() {
        let h = Hybrid::new(&doc, &reg);
        assert_eq!(h.tape_count(), 1, "{name}: a fusão não fundiu");
        assert_eq!(
            h.sampled_count(),
            0,
            "{name}: não há escultura nenhuma aqui"
        );
    }
}

/// ⭐ **A lei numérica é a MESMA lei da árvore** — o juiz dos dois motores.
///
/// A mesma peça é montada duas vezes: uma com a esfera como **primitiva** (a booleana corre dentro
/// da árvore, em `ops`) e outra com a esfera como **escultura** (a booleana corre em `f32`, em
/// `hybrid::apply`). Se as duas fórmulas divergirem — num filete, num sinal, num `max` trocado — os
/// números separam-se aqui.
///
/// ⚠️ **Os três operadores e os três caracteres de mistura**, e não um só: a diferença passa por
/// De Morgan duas vezes, e é exatamente aí que um sinal se perde.
#[test]
fn the_numeric_law_is_the_same_law_as_the_tree() {
    use ph2d_field::{Node, NodeId, NodeKind};
    let radius = 0.5f32;
    let mut reg = Registry::new();
    reg.insert(
        "esfera".into(),
        std::sync::Arc::new(AnalyticSphere { radius }),
    );

    let (xs, ys, zs) = grid_points(4_000);
    for blend in [
        Blend::Sharp,
        Blend::Exact { radius: 0.08 },
        Blend::Organic { k: 0.08 },
    ] {
        for op in [
            Op::Union(blend),
            Op::Intersection(blend),
            Op::Difference(blend),
        ] {
            let cyl = || {
                leaf(
                    Primitive::Cylinder {
                        radius: 0.3,
                        half_height: 0.8,
                        round: 0.0,
                    },
                    Xform::IDENTITY,
                )
            };
            let tree_doc = FieldDoc::new(
                vec![
                    leaf(Primitive::Sphere { radius }, Xform::IDENTITY),
                    cyl(),
                    Node::new(
                        Xform::IDENTITY,
                        NodeKind::Combine {
                            op,
                            children: vec![NodeId(0), NodeId(1)],
                        },
                    ),
                ],
                NodeId(2),
            )
            .expect("doc analítico");
            let mixed_doc = FieldDoc::new(
                vec![
                    Node::new(
                        Xform::IDENTITY,
                        NodeKind::Sampled {
                            key: "esfera".into(),
                        },
                    ),
                    cyl(),
                    Node::new(
                        Xform::IDENTITY,
                        NodeKind::Combine {
                            op,
                            children: vec![NodeId(0), NodeId(1)],
                        },
                    ),
                ],
                NodeId(2),
            )
            .expect("doc misto");

            let mut a = Hybrid::new(&tree_doc, &reg);
            let mut b = Hybrid::new(&mixed_doc, &reg);
            assert_eq!(a.tape_count(), 1, "o analítico funde numa fita");
            assert_eq!(b.sampled_count(), 1, "o misto tem uma folha amostrada");

            let va = a.eval(&xs, &ys, &zs).expect("lote").to_vec();
            let vb = b.eval(&xs, &ys, &zs).expect("lote").to_vec();
            let worst = va
                .iter()
                .zip(&vb)
                .map(|(p, q)| (p - q).abs())
                .fold(0.0f32, f32::max);
            // ⚠️ A barra é de **representação**, não de gosto: os dois caminhos fazem as mesmas
            // contas noutra ordem, em `f32`. Um erro de LEI (um sinal, um `max` trocado) mede-se em
            // décimos, não em `1e-5`.
            assert!(
                worst < 1e-5,
                "{op:?}: a lei numérica divergiu da árvore em {worst:.2e}"
            );
        }
    }
}

/// ⚠️ **Uma escultura que o registo não conhece lê como espaço VAZIO** — nunca como sólido.
///
/// O caso não é hipotético: é um projeto carregado antes de a malha ser regenerada. Ler como sólido
/// encheria a cena de um bloco que ninguém autorizou, e numa **subtração** ele comeria a peça toda.
#[test]
fn an_unknown_sculpture_reads_as_empty_space() {
    use ph2d_field::{Node, NodeId, NodeKind};
    let reg = Registry::new();
    let (xs, ys, zs) = grid_points(500);
    let cyl = || {
        leaf(
            Primitive::Cylinder {
                radius: 0.3,
                half_height: 0.8,
                round: 0.0,
            },
            Xform::IDENTITY,
        )
    };
    let alone = FieldDoc::new(vec![cyl()], NodeId(0)).expect("só o cilindro");
    for op in [
        Op::Union(Blend::Sharp),
        Op::Difference(Blend::Sharp),
        Op::Intersection(Blend::Sharp),
    ] {
        let with_ghost = FieldDoc::new(
            vec![
                cyl(),
                Node::new(
                    Xform::IDENTITY,
                    NodeKind::Sampled {
                        key: "não existe".into(),
                    },
                ),
                Node::new(
                    Xform::IDENTITY,
                    NodeKind::Combine {
                        op,
                        children: vec![NodeId(0), NodeId(1)],
                    },
                ),
            ],
            NodeId(2),
        )
        .expect("doc com fantasma");
        let mut a = Hybrid::new(&alone, &reg);
        let mut b = Hybrid::new(&with_ghost, &reg);
        let va = a.eval(&xs, &ys, &zs).expect("lote").to_vec();
        let vb = b.eval(&xs, &ys, &zs).expect("lote").to_vec();
        // Na união e na subtração o fantasma não faz nada. Na intersecção ele **apaga** — e é isso
        // que "vazio" quer dizer; o gate mede o sinal, não o valor.
        match op {
            Op::Intersection(_) => assert!(
                vb.iter().all(|v| *v > 0.0),
                "intersectar com o vazio tem de dar vazio"
            ),
            _ => {
                let worst = va
                    .iter()
                    .zip(&vb)
                    .map(|(p, q)| (p - q).abs())
                    .fold(0.0f32, f32::max);
                assert!(
                    worst < 1e-4,
                    "{op:?}: o fantasma mexeu na peça ({worst:.2e})"
                );
            }
        }
    }
}

/// ⭐ **A POSE de uma escultura é desfeita na amostragem** — as duas metades, translação e escala.
///
/// ⚠️ A segunda metade é a que se esquece: o valor tem de voltar **multiplicado pela escala**. Sem
/// ela o campo deixa de ser uma distância assim que houver escala, e todo raio de filete contra a
/// escultura mede outra coisa.
#[test]
fn the_pose_of_a_sculpture_is_undone_on_the_sample() {
    use ph2d_field::{Node, NodeId, NodeKind};
    let mut reg = Registry::new();
    reg.insert(
        "esfera".into(),
        std::sync::Arc::new(AnalyticSphere { radius: 0.4 }),
    );
    let posed = |t: [f32; 3], s: f32| {
        FieldDoc::new(
            vec![Node::new(
                Xform {
                    translation: t,
                    scale: s,
                    ..Xform::IDENTITY
                },
                NodeKind::Sampled {
                    key: "esfera".into(),
                },
            )],
            NodeId(0),
        )
        .expect("doc posado")
    };

    // Movida: o centro vai com ela.
    let doc = posed([0.3, 0.0, 0.0], 1.0);
    let mut h = Hybrid::new(&doc, &reg);
    let v = h
        .eval(&[0.3, 0.0], &[0.0, 0.0], &[0.0, 0.0])
        .expect("lote")
        .to_vec();
    assert!((v[0] + 0.4).abs() < 1e-5, "o centro mudou-se: {v:?}");
    assert!((v[1] + 0.1).abs() < 1e-5, "e a origem ficou a 0,1 dentro");

    // Escalada 2×: o raio dobra, **e a distância também** — é a segunda metade.
    let doc = posed([0.0; 3], 2.0);
    let mut h = Hybrid::new(&doc, &reg);
    let v = h
        .eval(&[0.0, 1.0], &[0.0, 0.0], &[0.0, 0.0])
        .expect("lote")
        .to_vec();
    assert!((v[0] + 0.8).abs() < 1e-5, "o raio tem de dobrar: {v:?}");
    assert!(
        (v[1] - 0.2).abs() < 1e-5,
        "e a 1,0 do centro a distância é 1,0 − 0,8: {v:?}"
    );
}

/// ⭐ **UMA PEÇA LONGE DA ORIGEM É EXPORTADA INTEIRA** (W33) — o corte silencioso que a caixa fixa
/// fazia.
///
/// ⚠️ **O defeito não dava erro nenhum**: a grade era `[-1, 1]` fixo, uma peça em `x = 2,5` não tinha
/// **uma única troca de sinal** lá dentro, e a exportação saía vazia (ou pela metade, se a peça
/// estivesse a cavalo da parede). Nada na tela, nada no aviso — o artista abria o arquivo no Blender
/// e não estava lá.
#[test]
fn a_piece_far_from_the_origin_is_exported_whole() {
    let far = [2.5_f32, -1.2, 0.7];
    let radius = 0.4_f32;
    let doc = FieldDoc::new(
        vec![leaf(
            Primitive::Sphere { radius },
            Xform::at(far[0], far[1], far[2]),
        )],
        NodeId(0),
    )
    .expect("esfera longe");

    let m = crate::extract::extract(&doc, &Registry::new(), 6).expect("a malha sai");
    assert!(
        m.positions().len() > 100,
        "a peça longe saiu com {} vértices — a caixa cortou-a",
        m.positions().len()
    );

    // ⚠️ E ela saiu **onde está**, não trazida para a origem: o centro dos vértices tem de ser o
    // centro da esfera, e o raio tem de ser o raio.
    let n = m.positions().len() as f32;
    let c = m.positions().iter().fold([0.0f32; 3], |a, p| {
        [a[0] + p[0] / n, a[1] + p[1] / n, a[2] + p[2] / n]
    });
    for k in 0..3 {
        assert!(
            (c[k] - far[k]).abs() < 0.05,
            "a peça saiu no sítio errado: centro {c:?}, esperado {far:?}"
        );
    }
    let worst = m
        .positions()
        .iter()
        .map(|p| {
            let d = [p[0] - far[0], p[1] - far[1], p[2] - far[2]];
            ((d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() - radius).abs()
        })
        .fold(0.0f32, f32::max);
    assert!(
        worst < 0.02,
        "os vértices não estão na esfera: pior desvio {worst:.4}"
    );
}

/// ⭐ **A caixa apertada COMPRA resolução** — e o número é o ponto da wave.
///
/// ⚠️ Ele mede a razão entre a célula antiga (`2/n`, a caixa fixa do motor) e a nova (a da peça): é
/// quantas vezes mais fina a mesma profundidade passou a ser. Uma peça pequena — que é o caso normal,
/// porque as formas nascem no tamanho do enquadramento — ganha o mais.
#[test]
fn a_tight_box_buys_resolution() {
    let doc = FieldDoc::new(
        vec![leaf(Primitive::Sphere { radius: 0.25 }, Xform::IDENTITY)],
        NodeId(0),
    )
    .expect("esfera pequena");
    let depth = 7u8;
    let old_cell = 2.0 / f64::from(1u32 << depth);
    let new_cell = crate::extract::cell_size(&doc, &Registry::new(), depth);
    let gain = old_cell / new_cell;
    assert!(
        gain > 3.0,
        "uma esfera de 0,25 na caixa de 2 unidades tinha de ganhar >3x de resolução; ganhou \
         {gain:.2}x (célula {old_cell:.5} -> {new_cell:.5})"
    );
}

/// ⭐ **A MALHA SAI FECHADA** — toda aresta pertence a exactamente duas faces.
///
/// ⚠️ **É a lei que a folga da caixa protege, e uma prova de mutação foi precisa para a encontrar:**
/// tirar a folga (`half = r`, a superfície a encostar na parede) passava **verde** em todos os gates
/// de erro — porque uma esfera toca a caixa em **seis pontos** e perder seis travessias não mexe numa
/// média sobre milhares de vértices. O que se perde é outra coisa: os **buracos** que ficam lá, e um
/// buraco não é um erro de posição — é uma malha que a fatiadora recusa e que o *shade smooth* do
/// Blender denuncia como uma costura preta.
///
/// *Uma média não vê seis buracos; a topologia vê.*
#[test]
fn the_exported_mesh_is_closed() {
    for (name, doc) in [
        (
            "esfera",
            FieldDoc::new(
                vec![leaf(Primitive::Sphere { radius: 0.6 }, Xform::IDENTITY)],
                NodeId(0),
            )
            .expect("esfera"),
        ),
        (
            "caixa arredondada",
            FieldDoc::new(
                vec![leaf(
                    Primitive::Box {
                        half: [0.5, 0.35, 0.4],
                        round: 0.08,
                    },
                    Xform::at(1.7, 0.0, -0.9),
                )],
                NodeId(0),
            )
            .expect("caixa longe"),
        ),
    ] {
        let m = crate::extract::extract(&doc, &Registry::new(), 6).expect("a malha sai");
        let mut inc: std::collections::BTreeMap<(u32, u32), u32> =
            std::collections::BTreeMap::new();
        let mut tri = Vec::new();
        for f in m.faces() {
            tri.clear();
            f.triangles(&mut tri);
            for t in &tri {
                for k in 0..3 {
                    let (a, b) = (t[k], t[(k + 1) % 3]);
                    *inc.entry((a.min(b), a.max(b))).or_default() += 1;
                }
            }
        }
        let open = inc.values().filter(|&&n| n != 2).count();
        assert_eq!(
            open, 0,
            "{name}: {open} arestas com incidência ≠ 2 — a peça encostou na parede da caixa e saiu \
             com buraco"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// ⭐⭐ A SONDA DO CUSTO DO PERFIL (W56) — onde o tempo de facto está, antes de escolher a cura.

/// ⭐⭐ **QUANTO CUSTA UM PERFIL, e quanto dele é o PERFIL** (W56).
///
/// # A nota que esta sonda existe para conferir
///
/// O [`04_resultados_perfis.md`] §7 escreveu, em 2026-08-19, o gatilho e as duas direções:
/// *"aceleração espacial dentro da árvore — partir o perfil numa hierarquia de `min`/`max` por
/// caixa, para que **a poda por intervalo** volte a morder"*, e disse que nenhuma tinha sido feita
/// porque *"o número que as pediria (um perfil real acima de 128 arestas) ainda não existe"*.
///
/// ⭐ **O número passou a existir na W55**: o default shipa **168** arestas e o knob de `Resolution`
/// vai a **664**. Quem move o número que tornava algo inalcançável tem de reconferir a nota
/// (`CLAUDE.md` §0.0) — e a reconferência tem de medir o **mecanismo**, não repetir a prescrição.
///
/// # ⚠️ O que a leitura do código já disse, e que a tabela abaixo tem de confirmar
///
/// O traçado avalia **ponto a ponto** (`float_slice_tape`) e **ninguém avalia intervalos**: não há
/// passe por ladrilho, não há `simplify`. ⇒ *A poda por intervalo não tem onde morder neste
/// caminho*, e a direção 1 daquela nota, **como está escrita**, não moveria o traçado um
/// milissegundo. A tabela mede a consequência: o custo por ponto tem de ser **linear nas arestas**.
///
/// ⚠️ `#[ignore]` porque mede relógio — máquina calma:
///
/// ```text
/// cargo test -p ph2d-field-eval --release -- --exact \
///     tests::the_table_that_says_where_a_profile_spends_its_time --ignored --nocapture
/// ```
#[test]
#[ignore]
fn the_table_that_says_where_a_profile_spends_its_time() {
    const N: usize = 200_000;
    let reg = crate::hybrid::Registry::new();
    // Um lote de pontos numa nuvem à volta da peça — o que uma marcha de facto pede.
    let (mut xs, mut ys, mut zs) = (Vec::new(), Vec::new(), Vec::new());
    let mut s = 12_345u64;
    let mut rnd = || {
        s = s.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        ((s >> 33) as f64 / f64::from(u32::MAX)) as f32 - 0.5
    };
    for _ in 0..N {
        xs.push(rnd() * 2.0);
        ys.push(rnd() * 2.0);
        zs.push(rnd() * 2.0);
    }

    let time = |doc: &FieldDoc| -> f64 {
        let mut h = crate::hybrid::Hybrid::new(doc, &reg);
        // Uma corrida a frio para a fita se montar, depois a mediana de cinco.
        let _ = h.eval(&xs, &ys, &zs).expect("avalia");
        let mut ms = Vec::new();
        for _ in 0..5 {
            let t0 = std::time::Instant::now();
            let _ = h.eval(&xs, &ys, &zs).expect("avalia");
            ms.push(t0.elapsed().as_secs_f64() * 1e3);
        }
        ms.sort_by(f64::total_cmp);
        ms[2]
    };

    let base = time(&doc_of(Primitive::Cylinder {
        radius: 0.5,
        half_height: 0.2,
        round: 0.0,
    }));
    println!("arestas |  ms/{N} pts |  ns/ponto | x cilindro | ns/ponto/aresta");
    println!(
        "      — | {base:>10.2} | {:>9.1} |      1,00x |               —",
        base * 1e6 / N as f64
    );
    for n in [56usize, 168, 332, 664, 940] {
        let p = profile_of(vec![ngon(n, 0.5, [0.0, 0.0])], FillRule::NonZero);
        let ms = time(&doc_of(Primitive::Extrude {
            profile: p,
            half_height: 0.2,
            round: 0.0,
        }));
        let ns = ms * 1e6 / N as f64;
        println!(
            "{n:>7} | {ms:>10.2} | {ns:>9.1} | {:>9.2}x | {:>15.3}",
            ms / base,
            ns / n as f64
        );
    }

    // ⭐ E o SEGUNDO número, o que decide entre as duas curas: montar a fita.
    //
    // Especializar a fita **por ladrilho** (o que a `fidget` faz no renderer dela) só paga se
    // montar uma fita for barato perto de a avaliar. Se um ladrilho custar mais a compilar do que
    // a marchar, a cura tem de ser outra.
    println!();
    println!("arestas | montar a fita | avaliar 200k pts | ladrilhos que a montagem paga");
    for n in [56usize, 168, 332, 664] {
        let p = profile_of(vec![ngon(n, 0.5, [0.0, 0.0])], FillRule::NonZero);
        let doc = doc_of(Primitive::Extrude {
            profile: p,
            half_height: 0.2,
            round: 0.0,
        });
        let mut ms = Vec::new();
        for _ in 0..5 {
            let t0 = std::time::Instant::now();
            let h = crate::hybrid::Hybrid::new(&doc, &reg);
            ms.push(t0.elapsed().as_secs_f64() * 1e3);
            drop(h);
        }
        ms.sort_by(f64::total_cmp);
        let build = ms[2];
        let mut h = crate::hybrid::Hybrid::new(&doc, &reg);
        let _ = h.eval(&xs, &ys, &zs).expect("avalia");
        let t0 = std::time::Instant::now();
        let _ = h.eval(&xs, &ys, &zs).expect("avalia");
        let ev = t0.elapsed().as_secs_f64() * 1e3;
        println!(
            "{n:>7} | {build:>10.2} ms | {ev:>13.2} ms | {:>28.1}",
            ev / build
        );
    }
}

/// ⭐⭐⭐ **O DOCUMENTO ESPECIALIZADO CONCORDA COM O COMPLETO — DENTRO DA REGIÃO** (W56).
///
/// ⚠️ **É o gate que autoriza o consumidor a existir.** A [`crate::profile::sd_profile_in_region`]
/// já tinha o dela sobre um perfil solto; esta mede a travessia inteira — pose, pilha de
/// modificadores, booleana, e as **duas** formas de perfil (a extrusão no plano `xy` e o torno, cujo
/// `u` é `√(x² + z²)` e que ainda tira a costura do eixo da distância).
///
/// ⚠️ E mede as **duas pontas**: concorda dentro da região, **e** a especialização de facto encolhe a
/// árvore (senão a cura degenerada — especializar guardando tudo — passaria).
#[test]
fn the_specialised_document_agrees_inside_its_region() {
    use fidget::shape::EzShape;
    use ph2d_field::{Op, Unary};
    let ring = |n: usize, r: f64| ngon(n, r, [0.0, 0.0]);
    let cases: Vec<(&str, FieldDoc)> = vec![
        (
            "extrusão nua",
            doc_of(Primitive::Extrude {
                profile: profile_of(vec![ring(168, 0.5)], FillRule::NonZero),
                half_height: 0.2,
                round: 0.0,
            }),
        ),
        (
            "extrusão POSADA",
            doc_of_posed(
                Primitive::Extrude {
                    profile: profile_of(vec![ring(168, 0.5)], FillRule::NonZero),
                    half_height: 0.2,
                    round: 0.03,
                },
                Xform {
                    translation: [0.13, -0.07, 0.21],
                    rotation: [0.2, 0.1, 0.05, 0.97],
                    scale: 1.4,
                },
            ),
        ),
        (
            "torno",
            doc_of(Primitive::Revolve {
                profile: profile_of(
                    vec![vec![[0.2, -0.3], [0.5, -0.3], [0.5, 0.3], [0.2, 0.3]]],
                    FillRule::NonZero,
                ),
            }),
        ),
        (
            "torno de contorno FINO",
            doc_of(Primitive::Revolve {
                profile: profile_of(vec![ngon(96, 0.18, [0.42, 0.0])], FillRule::NonZero),
            }),
        ),
        // ⚠️ **Um torno cuja costura ASSENTA NO EIXO** — sem esta fixture, arrancar a regra da
        // costura do especializado passaria despercebido, e o campo ganharia um nível zero DENTRO
        // do sólido (o defeito medido no vaso da cena 5, §21).
        (
            "torno com costura NO EIXO",
            doc_of(Primitive::Revolve {
                profile: profile_of(
                    vec![vec![[0.0, -0.35], [0.45, -0.1], [0.45, 0.1], [0.0, 0.35]]],
                    FillRule::NonZero,
                ),
            }),
        ),
        // ⚠️ **Uma extrusão sob um modificador que REMAPEIA coordenadas** — a especialização tem de
        // DESISTIR aqui, e o gate mede que desistir continua correcto. Sem esta fixture, aceitar a
        // matriz por engano daria uma peça furada e nenhum teste vermelho.
        ("extrusão sob MATRIZ (a especialização desiste)", {
            // ⚠️ **Um contorno ALONGADO, e não um círculo.** Um círculo é equidistante de todos
            // os lados: guardar as arestas erradas devolve o **mesmo** número, e a fixture
            // passaria com a especialização a ser aplicada por engano debaixo da matriz. *Uma
            // fixture que não contém o fenómeno mede outra coisa* — foi a terceira vez que esta
            // linha pagou a mesma lição.
            let mut d = doc_of(Primitive::Extrude {
                profile: profile_of(
                    vec![vec![
                        [-0.40f32, -0.06],
                        [0.40, -0.06],
                        [0.40, 0.06],
                        [-0.40, 0.06],
                    ]],
                    FillRule::NonZero,
                ),
                half_height: 0.15,
                round: 0.0,
            });
            let mut nodes = d.nodes().to_vec();
            nodes[0].mods = vec![Unary::Array {
                count: 3,
                spacing: 0.6,
            }];
            d = FieldDoc::new(nodes, NodeId(0)).expect("a matriz");
            d
        }),
    ];
    // …e uma união de uma extrusão OCA com uma esfera, sob uma pose de grupo.
    let mixed = {
        let mut nodes = vec![
            Node {
                xform: Xform::at(0.05, 0.0, 0.0),
                kind: NodeKind::Leaf(Primitive::Extrude {
                    profile: profile_of(vec![ring(120, 0.45)], FillRule::NonZero),
                    half_height: 0.25,
                    round: 0.0,
                }),
                mods: vec![Unary::Shell { thickness: 0.06 }],
            },
            Node {
                xform: Xform::at(0.3, 0.1, 0.0),
                kind: NodeKind::Leaf(Primitive::Sphere { radius: 0.3 }),
                mods: Vec::new(),
            },
        ];
        nodes.push(Node {
            xform: Xform {
                translation: [-0.1, 0.05, 0.02],
                rotation: [0.0, 0.3, 0.0, 0.95],
                scale: 0.9,
            },
            kind: NodeKind::Combine {
                op: Op::Union(Blend::Sharp),
                children: vec![NodeId(0), NodeId(1)],
            },
            mods: Vec::new(),
        });
        FieldDoc::new(nodes, NodeId(2)).expect("a peça mista")
    };
    let mut all = cases;
    all.push(("união OCA + esfera, sob pose de grupo", mixed));

    let mut s = 0xC0FF_EE00u64;
    let mut rnd = move || {
        s = s.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        (s >> 33) as f32 / u32::MAX as f32
    };
    for (name, doc) in &all {
        let full = crate::compile(doc);
        let full_shape = crate::Engine::from(full);
        let mut full_eval = crate::Engine::new_float_slice_eval();
        let full_tape = full_shape.ez_float_slice_tape();
        let (mut worst, mut shrunk) = (0.0f32, 0usize);
        const REGIONS: usize = 60;
        for _ in 0..REGIONS {
            let c = [
                (rnd() - 0.5) * 1.4,
                (rnd() - 0.5) * 1.4,
                (rnd() - 0.5) * 1.4,
            ];
            let half = 0.12f32.mul_add(rnd(), 0.02);
            let lo = [c[0] - half, c[1] - half, c[2] - half];
            let hi = [c[0] + half, c[1] + half, c[2] + half];
            let cut = crate::compile_in_region(doc, lo, hi);
            let shape = crate::Engine::from(cut);
            let mut eval = crate::Engine::new_float_slice_eval();
            let tape = shape.ez_float_slice_tape();
            let (mut xs, mut ys, mut zs) = (Vec::new(), Vec::new(), Vec::new());
            for _ in 0..256 {
                xs.push((rnd() - 0.5).mul_add(2.0 * half, c[0]));
                ys.push((rnd() - 0.5).mul_add(2.0 * half, c[1]));
                zs.push((rnd() - 0.5).mul_add(2.0 * half, c[2]));
            }
            let got = eval.eval(&tape, &xs, &ys, &zs).expect("avalia").to_vec();
            let want = full_eval
                .eval(&full_tape, &xs, &ys, &zs)
                .expect("avalia")
                .to_vec();
            for i in 0..xs.len() {
                worst = worst.max((got[i] - want[i]).abs());
            }
            if got.len() == want.len() {
                shrunk += 1;
            }
        }
        assert!(
            worst < 1.0e-4,
            "{name}: o documento especializado discorda do completo em {worst:e} DENTRO da região \
             — a marcha atravessaria a peça"
        );
        assert_eq!(shrunk, REGIONS, "{name}: alguma região não avaliou");
    }
}

/// ⚠️ **E a especialização de facto ENCOLHE a árvore** — a metade que impede a cura degenerada.
///
/// A régua é a contagem de arestas que a região guarda, contra as que o perfil tem. Um documento
/// especializado que guardasse tudo passaria no gate de concordância e não compraria nada.
#[test]
fn the_specialised_document_actually_shrinks_the_tree() {
    let p = profile_of(vec![ngon(168, 0.5, [0.0, 0.0])], FillRule::NonZero);
    let idx = crate::profile_index::ProfileIndex::build(&p);
    // Uma região colada à casca, do tamanho de um ladrilho.
    let (lo, hi) = ([0.44f32, -0.06], [0.56f32, 0.06]);
    let kept = idx.distance_edges(lo, hi).len() + idx.crossing_edges(lo, hi).len();
    assert!(
        kept * 4 < idx.edge_count(),
        "a região guardou {kept} das {} arestas — a especialização não está a cortar",
        idx.edge_count()
    );
}

/// ⭐ **A CENSURA dos modificadores que remapeiam coordenadas** (W56).
///
/// ⚠️ **Ela existe porque a versão comportamental não morde de forma fiável.** Debaixo de uma matriz,
/// a caixa do mundo mapeia para outra célula — mas numa região **longe** da peça o corte guarda quase
/// todas as arestas, e a discrepância desaparece no ruído. Uma mutação que aceitasse a matriz
/// sobreviveu ao gate de concordância (medido), e é este que a mata.
///
/// # ⛔ E ele PROMETIA mais do que fazia (2026-08-26)
///
/// O doc dizia *«um `Unary` novo é **erro de compilação** aqui»*, e não era: a lista era **escrita à
/// mão**, e a contagem no fim (`remaps.len() == 6`) só a defendia **de si mesma**. Os dois espelhos
/// novos entraram no documento e este gate ficou **verde** — a mutação que os classificava como *«não
/// remapeia»* (que **fura a peça**, porque a especialização passaria a construir o perfil sob um
/// domínio dobrado) **sobreviveu**.
///
/// ⇒ a lista passa a ser **derivada** de [`ph2d_field::UnaryKind::ALL`], e a expectativa é um `match`
/// exaustivo — que é onde o compilador de facto pára quem acrescenta um modificador. *Uma lista
/// escrita à mão ao lado de um enum é duas respostas, e a contagem só guarda a que se escreveu.*
#[test]
fn the_specialisation_gives_up_under_every_modifier_that_remaps_coordinates() {
    use ph2d_field::{Unary, UnaryKind};
    // ⚠️ **Exaustivo de propósito:** um `UnaryKind` novo é **erro de compilação** aqui, que é o
    // momento certo para alguém decidir se ele dobra o domínio.
    let want = |k: UnaryKind| -> bool {
        match k {
            UnaryKind::Shell | UnaryKind::Offset => false,
            UnaryKind::Mirror
            | UnaryKind::MirrorY
            | UnaryKind::MirrorZ
            | UnaryKind::Array
            | UnaryKind::Radial
            | UnaryKind::Taper => true,
        }
    };
    for k in UnaryKind::ALL {
        let m = Unary::born(k, 0.1);
        assert_eq!(
            crate::remaps_coordinates_for_test(&m),
            want(k),
            "{m:?}: a especialização tem de {} debaixo dele",
            if want(k) { "DESISTIR" } else { "continuar" }
        );
    }
}

/// ⭐⭐⭐ **A CAIXA DA PEÇA SEGUE O EIXO DO ESPELHO** (2026-08-26) — a metade que a mutação encontrou.
///
/// ⚠️ **Um espelho que dobra em Y com a caixa a crescer em X não fica lento: ele CORTA a peça.** A
/// caixa é o que a marcha recorta (`Scene::clip`), o que o exportador grada e o que o enquadramento
/// usa — e o doc do `bounding_radius` já o diz para a escultura: *«errar para baixo corta»*.
///
/// ⛔ Sem este gate, escrever `let k = 0` para os três eixos passava — foi uma prova de mutação que
/// o mostrou, e o gate nasceu dela.
#[test]
fn the_bounding_box_follows_the_axis_of_the_mirror() {
    use ph2d_field::Unary;
    let reg = hybrid::Registry::new();
    let off = 0.4f32;
    for (axis, m) in [
        (0usize, Unary::Mirror),
        (1, Unary::MirrorY),
        (2, Unary::MirrorZ),
    ] {
        let mut at = [0.0f32; 3];
        at[axis] = off;
        let mut top = combine(Op::Union(Blend::Sharp), vec![NodeId(0)]);
        top.mods.push(m);
        let doc = FieldDoc::new(
            vec![
                leaf(
                    Primitive::Box {
                        half: [0.1; 3],
                        round: 0.0,
                    },
                    Xform::at(at[0], at[1], at[2]),
                ),
                top,
            ],
            NodeId(1),
        )
        .expect("a peça");
        let ball = crate::bounds::bounding_ball(&doc, &reg).expect("a peça tem caixa");
        let aabb = crate::bounds::Ball::aabb(ball);
        assert!(
            aabb.0[axis] <= -f64::from(off) as f32 + 0.1,
            "eixo {axis}: a caixa vai só até {} e a cópia espelhada está em {}",
            aabb.0[axis],
            -off
        );
        // ⛔ **E o CONTROLO é a mesma peça sem espelho**: sem ele a caixa não alcança o outro
        // lado, e é isso que prova que quem a esticou foi o modificador.
        //
        // ⚠️ A metade que tentei primeiro — *«e não cresce nos outros eixos»* — estava **errada por
        // construção**: a caixa sai de uma **bola**, e uma bola maior cresce nos três. *Uma régua
        // que o representante não consegue exprimir não mede o produto: mede a representação.*
        let plain = FieldDoc::new(
            vec![
                leaf(
                    Primitive::Box {
                        half: [0.1; 3],
                        round: 0.0,
                    },
                    Xform::at(at[0], at[1], at[2]),
                ),
                combine(Op::Union(Blend::Sharp), vec![NodeId(0)]),
            ],
            NodeId(1),
        )
        .expect("a peça");
        let control = crate::bounds::Ball::aabb(
            crate::bounds::bounding_ball(&plain, &reg).expect("a peça tem caixa"),
        );
        assert!(
            control.0[axis] > -off,
            "eixo {axis}: sem espelho a caixa já alcançava {} — o gate não mede o espelho",
            control.0[axis]
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// W56f — a AUDITORIA de `‖∇f‖`: quem infla o gradiente, e quem não infla
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// Um contorno de `n` lados, raio `r`, para as formas de perfil.
fn ring(n: usize, r: f64) -> Vec<[f32; 2]> {
    (0..n)
        .map(|i| {
            let a = std::f64::consts::TAU * (i as f64) / (n as f64);
            [(r * a.cos()) as f32, (r * a.sin()) as f32]
        })
        .collect()
}

/// O **maior `‖∇f‖`** deste documento, sobre uma grelha densa da caixa `[-e, e]³`.
///
/// ⚠️ **Uma quina não dá falso positivo.** Numa aresta convexa a distância é exacta e `‖∇f‖ = 1`;
/// num vinco côncavo (e no eixo medial, lá dentro) a derivada não existe e a diferença central lê
/// **menos** que 1. O que esta sonda caça é o contrário: uma região **lisa** onde o campo sobe mais
/// depressa que a distância — que é o que faz a marcha atravessar a superfície.
fn worst_gradient(doc: &FieldDoc, e: f64, steps: usize) -> f64 {
    let f = Field::new(doc);
    let eps = 1e-4;
    let mut worst = 0.0f64;
    for i in 0..steps {
        for j in 0..steps {
            for k in 0..steps {
                let p = |t: usize| -e + 2.0 * e * (t as f64 + 0.5) / steps as f64;
                let g = f.gradient_norm(p(i), p(j), p(k), eps);
                if g.is_finite() {
                    worst = worst.max(g);
                }
            }
        }
    }
    worst
}

fn leaves(prim: Primitive) -> FieldDoc {
    FieldDoc::new(vec![leaf(prim, Xform::IDENTITY)], NodeId(0)).expect("a peça")
}

fn modded(prim: Primitive, m: ph2d_field::Unary) -> FieldDoc {
    let mut n = leaf(prim, Xform::IDENTITY);
    n.mods.push(m);
    FieldDoc::new(vec![n], NodeId(0)).expect("a peça")
}

fn pair(op: Op) -> FieldDoc {
    FieldDoc::new(
        vec![
            leaf(
                Primitive::Box {
                    half: [0.6, 0.3, 0.3],
                    round: 0.0,
                },
                Xform::at(-0.2, 0.0, 0.0),
            ),
            leaf(
                Primitive::Box {
                    half: [0.3, 0.6, 0.3],
                    round: 0.0,
                },
                Xform::at(0.2, 0.0, 0.0),
            ),
            combine(op, vec![NodeId(0), NodeId(1)]),
        ],
        NodeId(2),
    )
    .expect("a peça")
}

/// ⭐⭐⭐ **A AUDITORIA: quem infla `‖∇f‖`** (W56f) — a tabela que o passo da marcha lê.
///
/// A marcha anda `d · SAFE_STEP` com `SAFE_STEP = 1/√2`, e o número é o recíproco de uma constante
/// **medida na W0**: `‖∇f‖` chega a `√2` no arredondamento exacto. ⚠️ Mas o `Xform::scale` deste
/// módulo é **uniforme de propósito** e o doc dele já diz porquê — *"‖∇f‖ = 1 é a fundação de tudo
/// neste módulo"*. Se quase todo construtor honra a fundação, então o passo curto é **o caminho
/// mais lento a definir o teto do mais rápido**, que é o que o `CLAUDE.md` §0 proíbe.
///
/// Esta sonda mede, construtor a construtor, quem de facto infla.
///
/// ```text
/// cargo test -p ph2d-field-eval --release -- --exact \
///     tests::the_table_of_who_inflates_the_gradient --ignored --nocapture
/// ```
#[test]
#[ignore]
fn the_table_of_who_inflates_the_gradient() {
    use ph2d_field::{FillRule, Profile, Unary};
    let profile = Profile::new(vec![ring(24, 0.5)], FillRule::NonZero, 1e-3).expect("perfil");
    let bx = Primitive::Box {
        half: [0.4, 0.3, 0.25],
        round: 0.0,
    };
    let cases: Vec<(&str, FieldDoc)> = vec![
        ("Box", leaves(bx.clone())),
        (
            "Box round=0,1",
            leaves(Primitive::Box {
                half: [0.4, 0.3, 0.25],
                round: 0.1,
            }),
        ),
        ("Sphere", leaves(Primitive::Sphere { radius: 0.5 })),
        (
            "Cylinder",
            leaves(Primitive::Cylinder {
                radius: 0.4,
                half_height: 0.3,
                round: 0.0,
            }),
        ),
        (
            "Cylinder round=0,1",
            leaves(Primitive::Cylinder {
                radius: 0.4,
                half_height: 0.3,
                round: 0.1,
            }),
        ),
        (
            "Torus",
            leaves(Primitive::Torus {
                major: 0.4,
                minor: 0.15,
            }),
        ),
        (
            "Extrude",
            leaves(Primitive::Extrude {
                profile: profile.clone(),
                half_height: 0.25,
                round: 0.0,
            }),
        ),
        (
            "Extrude round=0,1",
            leaves(Primitive::Extrude {
                profile: profile.clone(),
                half_height: 0.25,
                round: 0.1,
            }),
        ),
        (
            "Revolve",
            leaves(Primitive::Revolve {
                profile: Profile::new(
                    vec![vec![[0.2, -0.3], [0.5, -0.3], [0.5, 0.3], [0.2, 0.3]]],
                    FillRule::NonZero,
                    1e-3,
                )
                .expect("perfil"),
            }),
        ),
        ("Union Sharp", pair(Op::Union(Blend::Sharp))),
        ("Intersect Sharp", pair(Op::Intersection(Blend::Sharp))),
        ("Difference Sharp", pair(Op::Difference(Blend::Sharp))),
        (
            "Union Exact r=0,1",
            pair(Op::Union(Blend::Exact { radius: 0.1 })),
        ),
        (
            "Intersect Exact r=0,1",
            pair(Op::Intersection(Blend::Exact { radius: 0.1 })),
        ),
        (
            "Difference Exact r=0,1",
            pair(Op::Difference(Blend::Exact { radius: 0.1 })),
        ),
        (
            "Union Organic k=0,2",
            pair(Op::Union(Blend::Organic { k: 0.2 })),
        ),
        (
            "Intersect Organic k=0,2",
            pair(Op::Intersection(Blend::Organic { k: 0.2 })),
        ),
        (
            "Difference Organic k=0,2",
            pair(Op::Difference(Blend::Organic { k: 0.2 })),
        ),
        (
            "Shell t=0,05",
            modded(bx.clone(), Unary::Shell { thickness: 0.05 }),
        ),
        (
            "Offset d=0,1",
            modded(bx.clone(), Unary::Offset { distance: 0.1 }),
        ),
        ("Mirror", modded(bx.clone(), Unary::Mirror)),
        (
            "Array 3x0,5",
            modded(
                bx.clone(),
                Unary::Array {
                    count: 3,
                    spacing: 0.5,
                },
            ),
        ),
        ("Radial 5", modded(bx.clone(), Unary::Radial { count: 5 })),
        ("Taper 0,3", modded(bx.clone(), Unary::Taper { slope: 0.3 })),
        (
            "escala 0,4",
            FieldDoc::new(
                vec![leaf(
                    bx.clone(),
                    Xform {
                        scale: 0.4,
                        ..Xform::IDENTITY
                    },
                )],
                NodeId(0),
            )
            .expect("a peça"),
        ),
    ];
    println!("construtor | pior ‖∇f‖");
    for (name, doc) in cases {
        println!("{name:>24} | {:.4}", worst_gradient(&doc, 1.0, 48));
    }
}

/// ⭐⭐⭐ **A auditoria VARRIDA nos parâmetros** (W56f) — porque um valor não é uma família.
///
/// ⚠️ A tabela de [`the_table_of_who_inflates_the_gradient`] mede **um** valor por construtor, e um
/// valor não prova nada sobre a família: o `Taper` a `slope = 0,3` lê `0,94`, e a pergunta é se ele
/// passa de `1` mais acima. *Uma fixtura que não contém o fenómeno mede outra coisa* — foi o erro
/// pago quatro vezes na W56.
///
/// ```text
/// cargo test -p ph2d-field-eval --release -- --exact \
///     tests::the_table_of_the_gradient_across_the_parameter --ignored --nocapture
/// ```
#[test]
#[ignore]
fn the_table_of_the_gradient_across_the_parameter() {
    use ph2d_field::{FillRule, Profile, Unary};
    let bx = Primitive::Box {
        half: [0.4, 0.3, 0.25],
        round: 0.0,
    };
    let show = |name: &str, vals: Vec<(String, FieldDoc)>| {
        let cells: Vec<String> = vals
            .into_iter()
            .map(|(v, d)| format!("{v}:{:.3}", worst_gradient(&d, 1.0, 40)))
            .collect();
        println!("{name:>22} | {}", cells.join("  "));
    };
    show(
        "Taper slope",
        [0.0f32, 0.2, 0.5, 1.0, 2.0, 4.0]
            .into_iter()
            .map(|s| {
                (
                    format!("{s}"),
                    modded(bx.clone(), Unary::Taper { slope: s }),
                )
            })
            .collect(),
    );
    show(
        "Radial count",
        [2u32, 3, 5, 8, 16, 64]
            .into_iter()
            .map(|c| {
                (
                    format!("{c}"),
                    modded(bx.clone(), Unary::Radial { count: c }),
                )
            })
            .collect(),
    );
    show(
        "Array spacing",
        [0.1f32, 0.3, 0.5, 1.0]
            .into_iter()
            .map(|s| {
                (
                    format!("{s}"),
                    modded(
                        bx.clone(),
                        Unary::Array {
                            count: 4,
                            spacing: s,
                        },
                    ),
                )
            })
            .collect(),
    );
    show(
        "Shell thickness",
        [0.01f32, 0.05, 0.2, 0.5]
            .into_iter()
            .map(|t| {
                (
                    format!("{t}"),
                    modded(bx.clone(), Unary::Shell { thickness: t }),
                )
            })
            .collect(),
    );
    show(
        "Union Exact r",
        [0.0f32, 0.02, 0.1, 0.3, 0.6]
            .into_iter()
            .map(|r| (format!("{r}"), pair(Op::Union(Blend::Exact { radius: r }))))
            .collect(),
    );
    show(
        "Difference Exact r",
        [0.0f32, 0.02, 0.1, 0.3, 0.6]
            .into_iter()
            .map(|r| {
                (
                    format!("{r}"),
                    pair(Op::Difference(Blend::Exact { radius: r })),
                )
            })
            .collect(),
    );
    show(
        "Union Organic k",
        [0.0f32, 0.05, 0.2, 0.6, 1.2]
            .into_iter()
            .map(|k| (format!("{k}"), pair(Op::Union(Blend::Organic { k }))))
            .collect(),
    );
    show(
        "Difference Organic k",
        [0.0f32, 0.05, 0.2, 0.6, 1.2]
            .into_iter()
            .map(|k| (format!("{k}"), pair(Op::Difference(Blend::Organic { k }))))
            .collect(),
    );
    show(
        "Extrude round",
        [0.0f32, 0.05, 0.15, 0.24]
            .into_iter()
            .map(|r| {
                (
                    format!("{r}"),
                    leaves(Primitive::Extrude {
                        profile: Profile::new(vec![ring(24, 0.5)], FillRule::NonZero, 1e-3)
                            .expect("perfil"),
                        half_height: 0.25,
                        round: r,
                    }),
                )
            })
            .collect(),
    );
    show(
        "escala",
        [0.2f32, 0.5, 1.0, 2.0, 4.0]
            .into_iter()
            .map(|s| {
                (
                    format!("{s}"),
                    FieldDoc::new(
                        vec![leaf(
                            bx.clone(),
                            Xform {
                                scale: s,
                                ..Xform::IDENTITY
                            },
                        )],
                        NodeId(0),
                    )
                    .expect("a peça"),
                )
            })
            .collect(),
    );
}

/// ⭐⭐⭐ **O PASSO VEZES O PIOR GRADIENTE NUNCA PASSA DE 1** (W56f) — o invariante da marcha.
///
/// ⛔ **É o gate que impede a peça de FURAR.** A marcha de esferas anda `d · s`, e ela só é segura
/// enquanto `s · ‖∇f‖ ≤ 1`: acima disso o passo é maior que a distância até à superfície, o raio
/// atravessa-a, e o sintoma é pixel de fundo no meio da peça. ⚠️ Errar a classificação de um
/// construtor **não fica lento — fura**.
///
/// ⚠️ **Ele mede o produto dos DOIS lados**, e é isso que o torna um gate e não uma tabela: a
/// [`crate::safe_march_step`] classifica, a [`worst_gradient`] mede, e o que se afirma é a relação
/// entre elas. Um construtor novo que infle e não seja classificado reprova aqui.
///
/// ⚠️ **E cada família é varrida no parâmetro** — a `Difference Exact` lê `1,000` **exacto** a
/// `r = 0,1` e `1,143` a `r = 0,6`. *Um valor não é uma família, e foi a fixtura de um valor só que
/// quase deixou passar esta.*
#[test]
fn the_step_times_the_worst_gradient_never_exceeds_one() {
    use ph2d_field::{FillRule, Profile, Unary};
    let bx = Primitive::Box {
        half: [0.4, 0.3, 0.25],
        round: 0.0,
    };
    let mut cases: Vec<(String, FieldDoc)> = Vec::new();
    for r in [0.0f32, 0.1] {
        cases.push((
            format!("Box round={r}"),
            leaves(Primitive::Box {
                half: [0.4, 0.3, 0.25],
                round: r,
            }),
        ));
        cases.push((
            format!("Cylinder round={r}"),
            leaves(Primitive::Cylinder {
                radius: 0.4,
                half_height: 0.3,
                round: r,
            }),
        ));
        cases.push((
            format!("Extrude round={r}"),
            leaves(Primitive::Extrude {
                profile: Profile::new(vec![ring(24, 0.5)], FillRule::NonZero, 1e-3)
                    .expect("perfil"),
                half_height: 0.25,
                round: r,
            }),
        ));
    }
    cases.push(("Sphere".into(), leaves(Primitive::Sphere { radius: 0.5 })));
    cases.push((
        "Torus".into(),
        leaves(Primitive::Torus {
            major: 0.4,
            minor: 0.15,
        }),
    ));
    cases.push((
        "Revolve".into(),
        leaves(Primitive::Revolve {
            profile: Profile::new(
                vec![vec![[0.2, -0.3], [0.5, -0.3], [0.5, 0.3], [0.2, 0.3]]],
                FillRule::NonZero,
                1e-3,
            )
            .expect("perfil"),
        }),
    ));
    // ⚠️ As três operações × os três caracteres × a faixa do número — é aqui que a
    // `Difference Exact` mora, e ela só aparece com `r` grande.
    for (on, op) in [("Union", 0u8), ("Intersect", 1), ("Difference", 2)] {
        for r in [0.0f32, 0.02, 0.1, 0.3, 0.6] {
            for (bn, b) in [
                ("Sharp", Blend::Sharp),
                ("Exact", Blend::Exact { radius: r }),
                ("Organic", Blend::Organic { k: r * 2.0 }),
            ] {
                if bn == "Sharp" && r != 0.0 {
                    continue;
                }
                let o = match op {
                    0 => Op::Union(b),
                    1 => Op::Intersection(b),
                    _ => Op::Difference(b),
                };
                cases.push((format!("{on} {bn} {r}"), pair(o)));
            }
        }
    }
    for s in [0.0f32, 0.2, 0.5, 1.0, 2.0, 4.0] {
        cases.push((
            format!("Taper {s}"),
            modded(bx.clone(), Unary::Taper { slope: s }),
        ));
    }
    for c in [2u32, 3, 5, 8, 16, 64] {
        cases.push((
            format!("Radial {c}"),
            modded(bx.clone(), Unary::Radial { count: c }),
        ));
    }
    for sp in [0.1f32, 0.3, 0.5, 1.0] {
        cases.push((
            format!("Array {sp}"),
            modded(
                bx.clone(),
                Unary::Array {
                    count: 4,
                    spacing: sp,
                },
            ),
        ));
    }
    for t in [0.01f32, 0.05, 0.2, 0.5] {
        cases.push((
            format!("Shell {t}"),
            modded(bx.clone(), Unary::Shell { thickness: t }),
        ));
    }
    for d in [-0.1f32, 0.0, 0.1, 0.3] {
        cases.push((
            format!("Offset {d}"),
            modded(bx.clone(), Unary::Offset { distance: d }),
        ));
    }
    for (n, m) in [
        ("Mirror", Unary::Mirror),
        ("MirrorY", Unary::MirrorY),
        ("MirrorZ", Unary::MirrorZ),
    ] {
        cases.push((n.into(), modded(bx.clone(), m)));
    }
    for sc in [0.2f32, 0.5, 1.0, 2.0, 4.0] {
        cases.push((
            format!("escala {sc}"),
            FieldDoc::new(
                vec![leaf(
                    bx.clone(),
                    Xform {
                        scale: sc,
                        ..Xform::IDENTITY
                    },
                )],
                NodeId(0),
            )
            .expect("a peça"),
        ));
    }

    // ⭐⭐⭐ **E as COMPOSIÇÕES** (W75) — a cerca que o `safe_march_step` declarava e não media.
    //
    // ⛔ Elas entram aqui e não numa tabela à parte porque foi exactamente isso que deixou o defeito
    // vivo: a tabela dizia *«não foi medido»* e o gate só varria construtores **soltos**. *Uma cerca
    // que nenhum gate atravessa é uma nota, não uma cerca.*
    cases.extend(composition_cases());

    // ⚠️ A folga é da DIFERENÇA CENTRAL, não do produto: a sonda mede `‖∇f‖` com `eps = 1e-4` sobre
    // um campo avaliado em `f32`, e o ruído de cancelamento não é zero.
    const SLACK: f64 = 1.02;
    let (mut worst_ratio, mut worst_name) = (0.0f64, String::new());
    let mut long_step = 0usize;
    // ⭐⭐ **A afirmação tem DOIS lados, e um só não a prende.** *Segura*: o passo vezes o gradiente
    // nunca passa de 1 — quem falha isto fura a peça. *Justa*: um documento cujo gradiente **não**
    // passa de 1 tem de receber o passo inteiro — quem falha isto castiga quem não usa a feature,
    // que é o defeito que esta wave veio curar. ⛔ Uma mutação que marcava o `Organic` como
    // inflador **sobreviveu** ao lado seguro sozinho: ela só torna a marcha mais lenta.
    //
    // ⚠️ **A metade justa NÃO se pergunta da família `Exact`**, e o motivo é uma medição: ela infla
    // `1,4142` em `Union`/`Intersection` a **todo** `r > 0`, e `1,143` na `Difference` a `r = 0,6`.
    // Que a `Difference` leia `1,000` **exacto** a `r = 0,1` é facto da FIXTURA, não do construtor —
    // outra geometria com o mesmo `r` pode inflar. *Classificar um construtor pelo valor que uma
    // fixtura lhe deu é o mesmo erro que esta wave pagou quatro vezes.* O que fica gateado é que a
    // reserva é **merecida**: as duas operações abaixo têm de medir acima de `1,2`.
    let mut too_shy: Vec<String> = Vec::new();
    let has_exact = |d: &FieldDoc| {
        d.nodes().iter().any(|n| {
            matches!(&n.kind, NodeKind::Combine { op, .. }
                if matches!(op.blend(), Blend::Exact { radius } if radius != 0.0))
        })
    };
    for (name, doc) in &cases {
        let step = f64::from(crate::safe_march_step(doc));
        assert!(step > 0.0 && step <= 1.0, "{name}: passo absurdo ({step})");
        if step > 0.99 {
            long_step += 1;
        }
        let grad = worst_gradient(doc, 1.0, 32);
        let ratio = step * grad;
        if ratio > worst_ratio {
            worst_ratio = ratio;
            worst_name = name.clone();
        }
        if grad <= SLACK && step <= 0.99 && !has_exact(doc) {
            too_shy.push(format!("{name} (‖∇f‖ = {grad:.3})"));
        }
    }
    assert!(
        worst_ratio <= SLACK,
        "{worst_name}: passo × ‖∇f‖ = {worst_ratio:.4} > {SLACK} — a marcha atravessa a superfície \
         neste documento, e o sintoma é pixel de fundo no meio da peça"
    );
    assert!(
        too_shy.is_empty(),
        "{} documentos honram ‖∇f‖ ≤ 1 e mesmo assim levam o passo curto: {} — a classificação \
         está a castigar quem não infla",
        too_shy.len(),
        too_shy.join(", ")
    );
    // ⭐ **E a reserva da família `Exact` é MERECIDA, não superstição** — senão o passo curto dela
    // seria uma cerca sem medição atrás, que é o estado de que esta wave veio tirar o módulo.
    for (name, doc) in [
        ("Union Exact", pair(Op::Union(Blend::Exact { radius: 0.1 }))),
        (
            "Intersect Exact",
            pair(Op::Intersection(Blend::Exact { radius: 0.1 })),
        ),
    ] {
        let g = worst_gradient(&doc, 1.0, 32);
        assert!(
            g > 1.2,
            "{name}: ‖∇f‖ = {g:.4} — se o arredondamento exacto deixou de inflar, o passo curto \
             dele deixou de ter motivo, e a `safe_march_step` está a castigar sem medição atrás"
        );
    }
    // …e o CONTROLE: se toda a gente ficasse no passo curto, o gate acima passaria sem provar nada.
    assert!(
        long_step * 2 >= cases.len(),
        "só {long_step} de {} documentos ganharam o passo inteiro — ou a classificação ficou \
         conservadora demais, ou o gate deixou de medir o que a wave construiu",
        cases.len()
    );
}

/// ⭐⭐ **A ESPECIALIZAÇÃO DE FACTO USA O CASCO** (W59) — a metade do fio, sem relógio.
///
/// ⚠️ **Os dois gates do renderer medem a LEI (`probe_hull_uv`), não o FIO.** Fazer o
/// `specialised_profile` ignorar o casco é uma regressão **só de relógio** — a imagem sai idêntica,
/// porque a caixa é uma região válida. *É a mesma família do «a região era a peça inteira» da W56d,
/// e a quarta vez nesta linha que a metade que falta é a de quem executa.*
///
/// ⭐ **A régua que não é relógio:** as duas árvores guardam **conjuntos de arestas diferentes**,
/// então elas concordam DENTRO da região (isso é o outro gate) e **discordam fora** dela. Uma
/// especialização que ignorasse o casco daria a MESMA árvore, byte a byte.
#[test]
fn the_specialisation_actually_consumes_the_hull() {
    use ph2d_field::{FillRule, Profile};
    let ring: Vec<[f32; 2]> = (0..168)
        .map(|i| {
            let a = std::f64::consts::TAU * f64::from(i) / 168.0;
            [(0.5 * a.cos()) as f32, (0.5 * a.sin()) as f32]
        })
        .collect();
    let profile = Profile::new(vec![ring], FillRule::NonZero, 1e-3).expect("perfil");
    let doc = FieldDoc::new(
        vec![leaf(
            Primitive::Extrude {
                profile,
                half_height: 0.2,
                round: 0.0,
            },
            Xform::IDENTITY,
        )],
        NodeId(0),
    )
    .expect("a peça");
    let rc = crate::RegionCompiler::new(&doc);
    // ⚠️ Um tubo **oblíquo, pequeno e PERTO DA BORDA**. ⛔ Duas fixturas anteriores falharam o
    // fenómeno pela mesma razão: o corte guarda tudo a menos de `dmax ≈ a + D`, com `a` a distância
    // à aresta mais próxima — e no **meio** de um círculo `a` é o raio, então uma região central
    // guarda as 168 por mais pequena que seja. *É perto da parede que um corte corta*, e é por isso
    // que o controle abaixo mede a caixa antes de o gate acusar seja quem for.
    let pts: Vec<[f32; 3]> = (0..8)
        .map(|k| {
            let t = if k < 4 { 0.0f32 } else { 1.0 };
            let (dx, dy) = ((k % 2) as f32 * 0.02, ((k / 2) % 2) as f32 * 0.02);
            [0.34 + 0.10 * t + dx, 0.34 - 0.20 * t + dy, 0.0]
        })
        .collect();
    let (mut lo, mut hi) = ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]);
    for p in &pts {
        for k in 0..3 {
            lo[k] = lo[k].min(p[k] - 0.01);
            hi[k] = hi[k].max(p[k] + 0.01);
        }
    }
    let with = Field::from_tree(&rc.compile_at(&doc, lo, hi, &pts));
    let without = Field::from_tree(&rc.compile(&doc, lo, hi));
    // ⚠️ **FORA da região**, onde as duas não prometem nada uma à outra: é lá que um conjunto de
    // arestas diferente aparece como um número diferente.
    let mut diff = 0usize;
    for i in 0..40 {
        for j in 0..40 {
            let x = -0.9 + 1.8 * f64::from(i) / 39.0;
            let y = -0.9 + 1.8 * f64::from(j) / 39.0;
            if (with.at(x, y, 0.0) - without.at(x, y, 0.0)).abs() > 1e-6 {
                diff += 1;
            }
        }
    }
    // ⚠️ **O controle da fixtura**: se a região for grande, os dois cortes guardam TUDO e as duas
    // árvores saem idênticas — e o gate acusaria produto correto.
    let idx = crate::profile_index::ProfileIndex::build(match &doc.nodes()[0].kind {
        ph2d_field::NodeKind::Leaf(Primitive::Extrude { profile, .. }) => profile,
        _ => unreachable!("a fixtura é um extrude"),
    });
    let boxed = idx.probe_cull([lo[0], lo[1]], [hi[0], hi[1]]);
    assert!(
        boxed * 2 < idx.edge_count(),
        "a região é grande demais: a CAIXA já guarda {boxed} de {} arestas, e ali o casco não tem o \
         que apertar — a fixtura não contém o fenómeno",
        idx.edge_count()
    );
    assert!(
        diff > 0,
        "as duas árvores são idênticas em 1600 pontos — a especialização está a ignorar o casco, e \
         a regressão seria SÓ de relógio (a imagem sai igual, e gate de paridade nenhum a vê)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// W61 — O PLACAR DA MALHA EXTRAÍDA, contra o estado da arte
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// Quantas arestas da malha são partilhadas por um número de faces **diferente de 2**.
///
/// ⚠️ É a definição de **não-manifold** que importa a jusante: subdivisão, booleana e impressão 3D
/// pedem uma superfície fechada de duas faces por aresta. `(não-manifold, bordo)` — bordo é `1`.
fn manifold_census(mesh: &ph2d_mesh::Mesh) -> (usize, usize) {
    use std::collections::BTreeMap;
    let mut count: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for f in mesh.faces() {
        let v = f.0;
        let n = if v[3] == v[2] { 3 } else { 4 };
        for k in 0..n {
            let (a, b) = (v[k], v[(k + 1) % n]);
            *count.entry((a.min(b), a.max(b))).or_default() += 1;
        }
    }
    let non = count.values().filter(|c| **c > 2).count();
    let border = count.values().filter(|c| **c == 1).count();
    (non, border)
}

/// ⭐⭐⭐ **O PLACAR DA MALHA QUE SAI DAQUI** (W61) — e as barras são as do ORÁCULO.
///
/// # Por que este placar existe
///
/// Enio, 2026-08-24: *"o tempo não é problema. Busque a qualidade, o estado da arte no resultado da
/// malha, tente superar os melhores do mundo"*. ⛔ **Sem um placar, «estado da arte» é uma
/// intenção.** Este mede as quatro coisas por que uma extração de campo é julgada, e três delas têm
/// barra externa:
///
/// | eixo | régua | barra |
/// |---|---|---|
/// | **geometria** | `\|f\|` no vértice, em células | ⭐ o campo é o oráculo **analítico** — nenhum remalhador malha-a-malha tem isto |
/// | **topologia** | arestas com ≠2 faces · bordo | `0` e `0` numa peça fechada |
/// | **forma de face** | `ph2d_quadfill::quad_shape` | o oráculo `quadwild-bimdf` mede `1,08` de aspecto, `6°` de enviesamento, **0** faces com canto pior que 60° |
/// | **quina viva** | já medida | `116/116`, desvio `0,00` de célula |
///
/// ⚠️ **A régua de forma é a MESMA que a linha da escultura calibrou** contra um oráculo de
/// produção — e é por isso que ela é uma barra e não uma opinião.
///
/// ```text
/// cargo test -p ph2d-field-eval --release -- --exact \
///     tests::the_scorecard_of_the_extracted_mesh --ignored --nocapture
/// ```
#[test]
#[ignore]
fn the_scorecard_of_the_extracted_mesh() {
    use ph2d_field::{Blend, FillRule, Node, NodeKind, Op, Primitive, Profile};
    let reg = hybrid::Registry::new();
    let ring = |n: usize, r: f64| -> Vec<[f32; 2]> {
        (0..n)
            .map(|i| {
                let a = std::f64::consts::TAU * (i as f64) / (n as f64);
                [(r * a.cos()) as f32, (r * a.sin()) as f32]
            })
            .collect()
    };
    let cases: Vec<(&str, FieldDoc)> = vec![
        (
            "cubo (quina viva)",
            FieldDoc::new(
                vec![leaf(
                    Primitive::Box {
                        half: [0.4, 0.4, 0.4],
                        round: 0.0,
                    },
                    Xform::IDENTITY,
                )],
                NodeId(0),
            )
            .expect("a peça"),
        ),
        (
            "esfera (lisa)",
            FieldDoc::new(
                vec![leaf(Primitive::Sphere { radius: 0.45 }, Xform::IDENTITY)],
                NodeId(0),
            )
            .expect("a peça"),
        ),
        (
            "toro (género 1)",
            FieldDoc::new(
                vec![leaf(
                    Primitive::Torus {
                        major: 0.35,
                        minor: 0.13,
                    },
                    Xform::IDENTITY,
                )],
                NodeId(0),
            )
            .expect("a peça"),
        ),
        (
            "cubo MENOS esfera (vinco curvo)",
            FieldDoc::new(
                vec![
                    leaf(
                        Primitive::Box {
                            half: [0.4, 0.4, 0.4],
                            round: 0.0,
                        },
                        Xform::IDENTITY,
                    ),
                    leaf(
                        Primitive::Sphere { radius: 0.5 },
                        Xform::at(0.35, 0.35, 0.35),
                    ),
                    Node::new(
                        Xform::IDENTITY,
                        NodeKind::Combine {
                            op: Op::Difference(Blend::Sharp),
                            children: vec![NodeId(0), NodeId(1)],
                        },
                    ),
                ],
                NodeId(2),
            )
            .expect("a peça"),
        ),
        (
            "desenho puxado (24 lados)",
            FieldDoc::new(
                vec![leaf(
                    Primitive::Extrude {
                        profile: Profile::new(vec![ring(24, 0.4)], FillRule::NonZero, 1e-3)
                            .expect("perfil"),
                        half_height: 0.25,
                        round: 0.0,
                    },
                    Xform::IDENTITY,
                )],
                NodeId(0),
            )
            .expect("a peça"),
        ),
    ];
    println!(
        "peça | prof | faces | ≠2 faces | bordo | |f| médio (cél) | |f| p99 | máx | aspecto p50/máx | skew p50/p99 | >60° | não-quads"
    );
    for (name, doc) in &cases {
        for depth in [5u8, 6] {
            let Ok(mesh) = extract::extract(doc, &reg, depth) else {
                println!("{name:>28} | {depth} | RECUSOU");
                continue;
            };
            let f = Field::new(doc);
            let ball = crate::bounds::bounding_ball(doc, &reg).expect("a bola");
            // ⭐ **A célula da grade** — é nela que o erro se lê, porque é a única escala que a
            // extração conhece. (A grade cobre a bola com folga; ver `extract::Grid::new`.)
            let cell = f64::from(ball.radius) * 2.2 / f64::from(1u32 << depth);
            let mut errs: Vec<f64> = mesh
                .positions()
                .iter()
                .map(|p| {
                    f.at(f64::from(p[0]), f64::from(p[1]), f64::from(p[2]))
                        .abs()
                        / cell
                })
                .collect();
            errs.sort_by(f64::total_cmp);
            let mean = errs.iter().sum::<f64>() / errs.len().max(1) as f64;
            let p99 = errs[errs.len().saturating_sub(1) * 99 / 100];
            let max = errs.last().copied().unwrap_or(0.0);
            let (non, border) = manifold_census(&mesh);
            let sh = ph2d_quadfill::quad_shape(&mesh);
            let non_quads = mesh.faces().iter().filter(|f| f.0[3] == f.0[2]).count();
            println!(
                "{name:>28} | {depth:>4} | {:>5} | {non:>8} | {border:>5} | {mean:>13.4} | {p99:>7.4} | {max:>6.3} | {:>6.2}/{:>5.2} | {:>5.1}/{:>5.1} | {:>4} | {non_quads}",
                mesh.faces().len(),
                sh.aspect_p50,
                sh.aspect_max,
                sh.skew_p50,
                sh.skew_p99,
                sh.skew_over_60,
            );
        }
    }
}

/// ⭐⭐⭐ **A EXPERIÊNCIA DECISIVA: a nossa malha entra na cadeia de quads da casa** (W61).
///
/// # A pergunta, e por que não se responde afinando a extração
///
/// O placar mediu o buraco: a extração entrega enviesamento mediano de **25–27°** onde o oráculo de
/// produção (`quadwild-bimdf`) entrega **4,8–7,1°**. ⛔ **E afinar não cura**, por medição já no
/// repo (a `line/sculpt3d`, `PLAN.md`): *16 rondas de relaxação por ajuste de quadrado levam o
/// enviesamento mediano de `27°` para `26°` e pagam `3,4×` as dobras* ⇒ `SQUARE_ROUNDS = 0`.
/// *Se mover vértices 16× não move a mediana, o defeito está na CONECTIVIDADE.*
///
/// ⭐ **E a conectividade certa já existe neste monorepo**, medida a `5,1°`–`5,5°` — a classe do
/// oráculo, que ela ultrapassa numa das peças. Esta sonda pergunta a única coisa que falta saber:
/// **ela come a malha que SAI daqui?**
///
/// ⚠️ **A ordem tem uma FASE ZERO obrigatória**, e ela está medida: sem o remalhamento isotrópico à
/// frente, a mesma cadeia dá `10–12°` — *o dobro, sem uma linha de algoritmo mudar*.
///
/// ```text
/// cargo test -p ph2d-field-eval --release -- --exact \
///     tests::the_house_quad_chain_eats_the_mesh_this_crate_extracts --ignored --nocapture
/// ```
#[test]
#[ignore]
fn the_house_quad_chain_eats_the_mesh_this_crate_extracts() {
    use ph2d_crossfield::{Dual, solve_miq};
    use ph2d_quadfill::{SMOOTHING_ROUNDS, fill, quad_shape};
    use ph2d_quantize::{Budget, quantize_within};
    use ph2d_trace::trace_patches;
    let reg = hybrid::Registry::new();
    let cases: Vec<(&str, FieldDoc)> = vec![
        (
            "esfera",
            FieldDoc::new(
                vec![leaf(Primitive::Sphere { radius: 0.45 }, Xform::IDENTITY)],
                NodeId(0),
            )
            .expect("a peça"),
        ),
        (
            "toro",
            FieldDoc::new(
                vec![leaf(
                    Primitive::Torus {
                        major: 0.35,
                        minor: 0.13,
                    },
                    Xform::IDENTITY,
                )],
                NodeId(0),
            )
            .expect("a peça"),
        ),
    ];
    println!(
        "peça | fase | faces | aspecto p50/máx | skew p50/p99 | >60° | não-quads | irregulares"
    );
    for (name, doc) in &cases {
        let Ok(mut mesh) = extract::extract(doc, &reg, 6) else {
            println!("{name}: a extração recusou");
            continue;
        };
        let before = quad_shape(&mesh);
        println!(
            "{name:>8} | EXTRAÍDA | {:>5} | {:>6.2}/{:>6.2} | {:>5.1}/{:>5.1} | {:>4} | {:>9} | —",
            mesh.faces().len(),
            before.aspect_p50,
            before.aspect_max,
            before.skew_p50,
            before.skew_p99,
            before.skew_over_60,
            mesh.faces().iter().filter(|f| f.0[3] == f.0[2]).count(),
        );
        // ⚠️ **FASE ZERO** — sem ela a mesma cadeia dá o dobro do enviesamento (medido).
        let r = ph2d_remesh_iso::remesh_isotropic(&mut mesh, ph2d_remesh_iso::ALPHA);
        let target_edge = ph2d_remesh_iso::target_edge(&mesh, ph2d_remesh_iso::ALPHA);
        let _ = r;
        mesh.triangulate();
        let dual = Dual::build(&mesh);
        let (field, _) = solve_miq(&dual);
        let layout = trace_patches(&mesh, &dual, &field);
        let Ok(l) = layout.to_layout(target_edge) else {
            println!("{name:>8} | o traçado devolveu um layout inválido");
            continue;
        };
        let Ok((q, _qr)) = quantize_within(&l, Budget::new(256, 512)) else {
            println!("{name:>8} | a quantização recusou");
            continue;
        };
        match fill(&mesh, &mesh, &layout, &q, SMOOTHING_ROUNDS) {
            Ok((out, rep)) => {
                let after = quad_shape(&out);
                println!(
                    "{name:>8} | CADEIA   | {:>5} | {:>6.2}/{:>6.2} | {:>5.1}/{:>5.1} | {:>4} | {:>9} | {} ({:.1} %)",
                    out.faces().len(),
                    after.aspect_p50,
                    after.aspect_max,
                    after.skew_p50,
                    after.skew_p99,
                    after.skew_over_60,
                    rep.non_quads,
                    rep.irregular,
                    100.0 * rep.irregular as f64 / rep.verts.max(1) as f64,
                );
            }
            Err(e) => println!("{name:>8} | o preenchimento recusou: {e:?}"),
        }
    }
}

/// ⛔⛔ **O ENVIESAMENTO É DA GRADE, NÃO DA PEÇA** (W61) — o que fecha a pergunta «afinar chega?».
///
/// # A hipótese que esta sonda mata
///
/// O placar mostrou `25–27°` de enviesamento contra os `4,8–7,1°` do oráculo, e a resposta preguiçosa
/// seria *"afinar o extrator"*. ⛔ Duas coisas dizem que não:
///
/// 1. já está no repo que **mover vértices não cura** (a `line/sculpt3d` mediu 16 rondas de
///    relaxação a levarem a mediana de `27°` para `26°`, pagando `3,4×` as dobras);
/// 2. e esta sonda mostra **de onde o ângulo vem**: um cubo **alinhado** com a grade sai a `0,0°`, e
///    o MESMO cubo **rodado** sai enviesado. *A forma da face segue a GRADE, não a superfície* — que
///    é a definição de uma malha dual, e não um defeito de afinação.
///
/// ⇒ curar isto pede outra **conectividade**, não outro parâmetro. E a conectividade certa já existe
/// neste monorepo, medida a `5,1°`–`5,5°`.
///
/// ```text
/// cargo test -p ph2d-field-eval --release -- --exact \
///     tests::the_skew_belongs_to_the_grid_not_to_the_piece --ignored --nocapture
/// ```
#[test]
#[ignore]
fn the_skew_belongs_to_the_grid_not_to_the_piece() {
    use ph2d_quadfill::quad_shape;
    let reg = hybrid::Registry::new();
    println!("peça | ângulo | faces | aspecto p50/máx | skew p50/p99 | >60°");
    for deg in [0.0f32, 15.0, 30.0, 45.0] {
        let a = deg.to_radians() * 0.5;
        // Rotação em torno de Z — o quaternion `(0, 0, sin, cos)`.
        let rot = Xform {
            rotation: [0.0, 0.0, a.sin(), a.cos()],
            ..Xform::IDENTITY
        };
        let doc = FieldDoc::new(
            vec![leaf(
                Primitive::Box {
                    half: [0.35, 0.35, 0.35],
                    round: 0.0,
                },
                rot,
            )],
            NodeId(0),
        )
        .expect("a peça");
        let Ok(mesh) = extract::extract(&doc, &reg, 6) else {
            println!("cubo | {deg:>5.0}° | a extração recusou");
            continue;
        };
        let sh = quad_shape(&mesh);
        println!(
            "cubo | {deg:>5.0}° | {:>5} | {:>6.2}/{:>6.2} | {:>5.1}/{:>5.1} | {:>4}",
            mesh.faces().len(),
            sh.aspect_p50,
            sh.aspect_max,
            sh.skew_p50,
            sh.skew_p99,
            sh.skew_over_60,
        );
    }
}

/// ⭐⭐⭐ **A CADEIA CERTA SOBRE A NOSSA MALHA** (W61b) — a experiência que responde ao Enio.
///
/// ⛔ A primeira tentativa correu a metade **errada** da cadeia da casa (o preenchimento por patch,
/// `ph2d_quadfill::fill`) e **piorou**: esfera `26,6° → 23,2°`, toro `24,8° → 30,3°`. ⚠️ *Que ela
/// reproduza os `27°` que o repo já regista é o que VALIDA o arnês* — ela não contradisse a nota,
/// confirmou-a.
///
/// Esta corre a metade boa — a **extracção** (`ph2d-gridmap` + `ph2d-quadextract`), agora por uma
/// porta que qualquer módulo alcança ([`ph2d_quadchain::quads_from_mesh`]).
///
/// ```text
/// cargo test -p ph2d-field-eval --release -- --exact \
///     tests::the_quad_chain_turns_our_mesh_into_oracle_class --ignored --nocapture
/// ```
#[test]
#[ignore]
fn the_quad_chain_turns_our_mesh_into_oracle_class() {
    use ph2d_quadfill::quad_shape;
    let reg = hybrid::Registry::new();
    let cases: Vec<(&str, FieldDoc)> = vec![
        (
            "esfera",
            FieldDoc::new(
                vec![leaf(Primitive::Sphere { radius: 0.45 }, Xform::IDENTITY)],
                NodeId(0),
            )
            .expect("a peça"),
        ),
        (
            "toro",
            FieldDoc::new(
                vec![leaf(
                    Primitive::Torus {
                        major: 0.35,
                        minor: 0.13,
                    },
                    Xform::IDENTITY,
                )],
                NodeId(0),
            )
            .expect("a peça"),
        ),
        (
            "cubo rodado 45°",
            FieldDoc::new(
                vec![leaf(
                    Primitive::Box {
                        half: [0.35, 0.35, 0.35],
                        round: 0.0,
                    },
                    Xform {
                        rotation: [
                            0.0,
                            0.0,
                            (0.25f32 * std::f32::consts::PI).sin(),
                            (0.25f32 * std::f32::consts::PI).cos(),
                        ],
                        ..Xform::IDENTITY
                    },
                )],
                NodeId(0),
            )
            .expect("a peça"),
        ),
    ];
    println!("peça | fase | faces | aspecto p50/máx | skew p50/p99 | >60° | ≠2 faces | bordo");
    for (name, doc) in &cases {
        let Ok(mesh) = extract::extract(doc, &reg, 6) else {
            println!("{name}: a extração recusou");
            continue;
        };
        let a = quad_shape(&mesh);
        println!(
            "{name:>16} | EXTRAÍDA | {:>5} | {:>5.2}/{:>6.2} | {:>5.1}/{:>5.1} | {:>4} | {:>8} | {}",
            mesh.faces().len(),
            a.aspect_p50,
            a.aspect_max,
            a.skew_p50,
            a.skew_p99,
            a.skew_over_60,
            ph2d_quadchain::non_manifold_edges(&mesh),
            ph2d_quadchain::boundary_edges(&mesh),
        );
        // ⭐ O alvo sai da malha que ENTRA — a lei da cadeia.
        let target = ph2d_remesh_iso::target_edge(&mesh, ph2d_remesh_iso::ALPHA);
        match ph2d_quadchain::quads_from_mesh(&mesh, target) {
            Ok((out, r)) => println!(
                "{name:>16} | CADEIA   | {:>5} | {:>5.2}/{:>6.2} | {:>5.1}/{:>5.1} | {:>4} | {:>8} | {}   (não-quads {}, dobras {}, alinhado {})",
                out.faces().len(),
                r.shape.aspect_p50,
                r.shape.aspect_max,
                r.shape.skew_p50,
                r.shape.skew_p99,
                r.shape.skew_over_60,
                ph2d_quadchain::non_manifold_edges(&out),
                r.boundary_edges,
                r.non_quads,
                r.folded,
                r.aligned,
            ),
            Err(e) => println!("{name:>16} | a cadeia recusou: {e:?}"),
        }
    }
}

/// ⭐⭐⭐ **A CADEIA É ADOPTADA NA PEÇA QUE A GRADE ENVIESA** (W61b) — e o número é o do oráculo.
///
/// ⚠️ **Este gate vive aqui, e não na `ph2d-quadchain`, porque a ENTRADA de verdade é esta:** a
/// malha que o *Dual Contouring* extrai, com `26,6°` de enviesamento. ⛔ Uma esfera feita à mão já
/// sai a `2,8°`, e sobre ela a regra devolve «sem ganho» — *uma fixtura que já é boa não distingue
/// «a regra funciona» de «a regra nunca troca nada»*.
///
/// A barra é a do oráculo de produção `quadwild-bimdf`, que a `line/sculpt3d` calibrou:
/// **aspecto `1,08`, enviesamento `4,8–7,1°`**.
#[test]
fn the_chain_is_adopted_on_the_piece_the_grid_skews() {
    use ph2d_quadchain::{Verdict, quads_or_keep};
    let reg = hybrid::Registry::new();
    let doc = FieldDoc::new(
        vec![leaf(Primitive::Sphere { radius: 0.45 }, Xform::IDENTITY)],
        NodeId(0),
    )
    .expect("a peça");
    let mesh = extract::extract(&doc, &reg, 6).expect("a extração");
    let before = ph2d_quadfill::quad_shape(&mesh);
    // ⚠️ O controle da fixtura: a entrada tem de estar de facto enviesada, senão não há o que curar.
    assert!(
        before.skew_p50 > 15.0,
        "a malha extraída saiu a {:.1}° — a fixtura não contém o defeito que o gate mede",
        before.skew_p50
    );
    let target = ph2d_remesh_iso::target_edge(&mesh, ph2d_remesh_iso::ALPHA);
    let (out, v) = quads_or_keep(&mesh, target);
    let Verdict::Adopted(r) = &v else {
        panic!("a cadeia devia ser adoptada nesta peça e o veredito foi {v:?}");
    };
    // ⭐ **A barra é a do oráculo**, e não «melhor que antes»: uma barra relativa aceitaria `20°`.
    assert!(
        r.shape.skew_p50 <= 10.0,
        "a cadeia entregou {:.1}° de enviesamento — o oráculo `quadwild-bimdf` mede `4,8`–`7,1°`, e \
         uma barra relativa aceitaria qualquer melhoria",
        r.shape.skew_p50
    );
    assert!(
        r.shape.aspect_p50 <= 1.15,
        "o aspecto mediano saiu {:.2} — o oráculo mede `1,08`",
        r.shape.aspect_p50
    );
    // …e a peça continua fechada e toda em quads.
    assert_eq!(r.boundary_edges, 0, "a peça saiu ABERTA");
    assert_eq!(r.non_quads, 0, "saiu face que não é quad");
    assert_eq!(
        ph2d_quadchain::non_manifold_edges(&out),
        0,
        "saiu aresta não-manifold"
    );
}

/// ⭐ **QUANTO CUSTA EXPORTAR, por nível** — a sonda que o report do Enio pediu (2026-08-25:
/// *"o tempo de exportação numa malha de 1mi de faces é alto"*).
///
/// Ela imprime, por nível de exportação, o custo da extracção e o de **cada fase** da cadeia de
/// quads, mais o veredito. ⚠️ **As fases vêm do [`ph2d_quadchain::ChainTiming`]**, que a própria
/// cadeia preenche — repetir a sequência aqui seria uma segunda cópia da ordem.
///
/// ```text
/// cargo test -p ph2d-field-eval --release -- --exact \
///     tests::measure_the_export_cost_by_level --ignored --nocapture
/// ```
#[test]
#[ignore]
fn measure_the_export_cost_by_level() {
    let reg = hybrid::Registry::new();
    let doc = FieldDoc::new(
        vec![leaf(Primitive::Sphere { radius: 0.45 }, Xform::IDENTITY)],
        NodeId(0),
    )
    .expect("a peça");
    println!(
        "prof | quads in | extrair ms | F1 remesh | F2 campo | F3 traço | G1/G2 corte | G3/G5 mapa | extrair | TOTAL cadeia | veredito"
    );
    for depth in [6u8, 7, 8, 9] {
        let t0 = std::time::Instant::now();
        let Ok(mesh) = extract::extract(&doc, &reg, depth) else {
            println!("{depth}: a extração recusou");
            continue;
        };
        let ex = t0.elapsed().as_secs_f32() * 1000.0;
        let faces = mesh.faces().len();
        let target = ph2d_remesh_iso::target_edge(&mesh, ph2d_remesh_iso::ALPHA);
        let t1 = std::time::Instant::now();
        let (_, verdict) = ph2d_quadchain::quads_or_keep(&mesh, target);
        let wall = t1.elapsed().as_secs_f32() * 1000.0;
        let ms = match &verdict {
            ph2d_quadchain::Verdict::Adopted(r) => r.ms,
            _ => ph2d_quadchain::ChainTiming::default(),
        };
        println!(
            "{depth:>4} | {faces:>8} | {ex:>10.0} | {:>9.0} | {:>8.0} | {:>8.0} | {:>11.0} | {:>10.0} | {:>7.0} | {wall:>12.0} | {}",
            ms.remesh,
            ms.field,
            ms.trace,
            ms.cut,
            ms.map,
            ms.extract,
            match &verdict {
                ph2d_quadchain::Verdict::Adopted(r) =>
                    format!("adoptada ({:.1}°, {} quads)", r.shape.skew_p50, r.quads),
                ph2d_quadchain::Verdict::Rejected {
                    boundary,
                    non_manifold,
                } => format!("recusada (bordo {boundary}, ≠2 {non_manifold})"),
                ph2d_quadchain::Verdict::NoGain { before, after } =>
                    format!("sem ganho ({before:.1}° -> {after:.1}°)"),
                ph2d_quadchain::Verdict::Refused(e) => format!("recusou: {e:?}"),
                ph2d_quadchain::Verdict::Panicked => "ESTOUROU".into(),
            }
        );
    }
}

/// ⭐⭐⭐ **A GRADE QUE ALIMENTA A CADEIA NÃO PRECISA DE SER A QUE O ARTISTA PEDIU** — a sonda que
/// decidiu a cura do report do Enio (2026-08-25, *"o tempo de exportação numa malha de 1mi de faces
/// é alto"*).
///
/// # ⚠️ O mecanismo, e ele é uma linha de código de outra crate
///
/// `ph2d_remesh_iso::target_edge(mesh, alpha) = alpha · diagonal_da_caixa` — ele **não olha para a
/// densidade da malha**, só para a caixa dela. ⇒ a cadeia remalha para o **mesmo** alvo venha a
/// entrada de que profundidade vier, e tudo o que a grade fina traz a mais é **deitado fora pelo
/// F1**, depois de pago.
///
/// A sonda mede as três colunas que a decisão precisa: o **relógio**, a **forma** da face e a
/// **fidelidade** — esta última pelo campo, que é exacto (`|f|` no vértice de saída, em frações da
/// diagonal da peça). ⚠️ *Sem a coluna da fidelidade isto seria «mais barato e igualmente bonito»
/// sobre uma peça que encolheu.*
///
/// ```text
/// cargo test -p ph2d-field-eval --release -- --exact \
///     tests::measure_what_the_chain_gains_from_a_finer_grid --ignored --nocapture
/// ```
#[test]
#[ignore]
fn measure_what_the_chain_gains_from_a_finer_grid() {
    use ph2d_field::{Blend, Node, NodeKind, Op};
    let reg = hybrid::Registry::new();
    let cases: Vec<(&str, FieldDoc)> = vec![
        (
            "esfera",
            FieldDoc::new(
                vec![leaf(Primitive::Sphere { radius: 0.45 }, Xform::IDENTITY)],
                NodeId(0),
            )
            .expect("a peça"),
        ),
        (
            "duas caixas com filete",
            FieldDoc::new(
                vec![
                    leaf(
                        Primitive::Box {
                            half: [0.30, 0.12, 0.12],
                            round: 0.0,
                        },
                        Xform::IDENTITY,
                    ),
                    leaf(
                        Primitive::Box {
                            half: [0.12, 0.30, 0.12],
                            round: 0.0,
                        },
                        Xform::IDENTITY,
                    ),
                    Node::new(
                        Xform::IDENTITY,
                        NodeKind::Combine {
                            op: Op::Union(Blend::Organic { k: 0.06 }),
                            children: vec![NodeId(0), NodeId(1)],
                        },
                    ),
                ],
                NodeId(2),
            )
            .expect("a peça"),
        ),
        (
            "toro",
            FieldDoc::new(
                vec![leaf(
                    Primitive::Torus {
                        major: 0.35,
                        minor: 0.13,
                    },
                    Xform::IDENTITY,
                )],
                NodeId(0),
            )
            .expect("a peça"),
        ),
    ];
    println!(
        "peça | prof | faces in | extrair ms | F1 ms | cadeia ms | quads out | skew p50 | |f| médio | |f| máx  (em % da diagonal)"
    );
    for (name, doc) in &cases {
        for depth in [6u8, 7, 8] {
            let t0 = std::time::Instant::now();
            let Ok(mesh) = extract::extract(doc, &reg, depth) else {
                println!("{name} @{depth}: a extração recusou");
                continue;
            };
            let ex = t0.elapsed().as_secs_f32() * 1000.0;
            let b = mesh.bounds();
            let diag = ((b.max[0] - b.min[0]).powi(2)
                + (b.max[1] - b.min[1]).powi(2)
                + (b.max[2] - b.min[2]).powi(2))
            .sqrt();
            let target = ph2d_remesh_iso::target_edge(&mesh, ph2d_remesh_iso::ALPHA);
            match ph2d_quadchain::quads_from_mesh(&mesh, target) {
                Ok((out, r)) => {
                    // ⭐ A fidelidade sai do CAMPO, que é exacto — nenhuma malha é o oráculo aqui.
                    let mut h = hybrid::Hybrid::new(doc, &reg);
                    let (xs, ys, zs): (Vec<f32>, Vec<f32>, Vec<f32>) = out.positions().iter().fold(
                        (Vec::new(), Vec::new(), Vec::new()),
                        |(mut a, mut b, mut c), p| {
                            a.push(p[0]);
                            b.push(p[1]);
                            c.push(p[2]);
                            (a, b, c)
                        },
                    );
                    let vals = h.eval(&xs, &ys, &zs).expect("o campo avalia");
                    let n = vals.len().max(1) as f32;
                    let mean = vals.iter().map(|v| v.abs()).sum::<f32>() / n;
                    let max = vals.iter().map(|v| v.abs()).fold(0.0f32, f32::max);
                    println!(
                        "{name:>22} | {depth:>4} | {:>8} | {ex:>10.0} | {:>5.0} | {:>9.0} | {:>9} | {:>8.1} | {:>8.3} | {:>7.3}",
                        mesh.faces().len(),
                        r.ms.remesh,
                        r.ms.total(),
                        out.faces().len(),
                        r.shape.skew_p50,
                        100.0 * mean / diag,
                        100.0 * max / diag,
                    );
                }
                Err(e) => println!("{name} @{depth}: a cadeia recusou: {e:?}"),
            }
        }
    }
}

/// ⭐⭐⭐ **A ESCADA QUE MANTÉM A RAZÃO** — se a grade fina estraga a cadeia, o nível de detalhe tem
/// de subir os DOIS números juntos.
///
/// A sonda irmã (`measure_what_the_chain_gains_from_a_finer_grid`) mediu que subir só a grade é pior
/// em todas as colunas. Esta mede a alternativa: subir a grade **e** a densidade que se pede à
/// cadeia, mantendo `célula ≈ alvo / 2`. ⚠️ *Se a qualidade se mantiver, o nível de detalhe da
/// exportação passa a significar alguma coisa para a cadeia; se não, a cadeia tem uma densidade só e
/// isso é um facto de produto a reportar.*
///
/// ```text
/// cargo test -p ph2d-field-eval --release -- --exact \
///     tests::measure_the_ladder_that_keeps_the_ratio --ignored --nocapture
/// ```
#[test]
#[ignore]
fn measure_the_ladder_that_keeps_the_ratio() {
    use ph2d_field::{Blend, Node, NodeKind, Op};
    let reg = hybrid::Registry::new();
    let cases: Vec<(&str, FieldDoc)> = vec![
        (
            "esfera",
            FieldDoc::new(
                vec![leaf(Primitive::Sphere { radius: 0.45 }, Xform::IDENTITY)],
                NodeId(0),
            )
            .expect("a peça"),
        ),
        (
            "duas caixas com filete",
            FieldDoc::new(
                vec![
                    leaf(
                        Primitive::Box {
                            half: [0.30, 0.12, 0.12],
                            round: 0.0,
                        },
                        Xform::IDENTITY,
                    ),
                    leaf(
                        Primitive::Box {
                            half: [0.12, 0.30, 0.12],
                            round: 0.0,
                        },
                        Xform::IDENTITY,
                    ),
                    Node::new(
                        Xform::IDENTITY,
                        NodeKind::Combine {
                            op: Op::Union(Blend::Organic { k: 0.06 }),
                            children: vec![NodeId(0), NodeId(1)],
                        },
                    ),
                ],
                NodeId(2),
            )
            .expect("a peça"),
        ),
    ];
    println!("peça | prof | alpha | célula/alvo | cadeia ms | quads out | skew p50 | |f| máx %");
    for (name, doc) in &cases {
        for (depth, alpha) in [(6u8, 0.02f32), (7, 0.01), (8, 0.005), (9, 0.0025)] {
            let Ok(mesh) = extract::extract(doc, &reg, depth) else {
                println!("{name} @{depth}: a extração recusou");
                continue;
            };
            let b = mesh.bounds();
            let diag = ((b.max[0] - b.min[0]).powi(2)
                + (b.max[1] - b.min[1]).powi(2)
                + (b.max[2] - b.min[2]).powi(2))
            .sqrt();
            let target = ph2d_remesh_iso::target_edge(&mesh, alpha);
            let cell = extract::cell_size(doc, &reg, depth) as f32;
            match ph2d_quadchain::quads_from_mesh(&mesh, target) {
                Ok((out, r)) => {
                    let mut h = hybrid::Hybrid::new(doc, &reg);
                    let (xs, ys, zs): (Vec<f32>, Vec<f32>, Vec<f32>) = out.positions().iter().fold(
                        (Vec::new(), Vec::new(), Vec::new()),
                        |(mut a, mut b, mut c), p| {
                            a.push(p[0]);
                            b.push(p[1]);
                            c.push(p[2]);
                            (a, b, c)
                        },
                    );
                    let vals = h.eval(&xs, &ys, &zs).expect("o campo avalia");
                    let max = vals.iter().map(|v| v.abs()).fold(0.0f32, f32::max);
                    println!(
                        "{name:>22} | {depth:>4} | {alpha:>5.4} | {:>11.2} | {:>9.0} | {:>9} | {:>8.1} | {:>8.3}",
                        cell / target,
                        r.ms.total(),
                        out.faces().len(),
                        r.shape.skew_p50,
                        100.0 * max / diag,
                    );
                }
                Err(e) => println!("{name} @{depth} a={alpha}: a cadeia recusou: {e:?}"),
            }
        }
    }
}

/// ⭐⭐⭐ **QUANTO CUSTA MONTAR A ÁRVORE, contra marchá-la** — a sonda que decide se o traçado deve
/// guardar a fita entre quadros.
///
/// # ⛔ O que a levou a existir
///
/// Report do Enio (2026-08-26): *"queda de fps e lentidão com resoluções altas"*. Três hipóteses
/// foram medidas e refutadas — o recozimento do contorno por quadro (`23 µs`), a pré-visualização
/// mais grossa (o custo tem um **piso**), e o teto de detalhe. O que sobrou: o custo do traçado é
/// **quase fixo na área** (de `D=3` para `D=6` são 4× menos pixels e o tempo só cai `1,3×`) e
/// **linear nas arestas** do contorno. Isso é a assinatura de **montagem**, não de marcha.
///
/// ⚠️ **E o documento NÃO muda enquanto a câmera orbita** — só a pose da vista. Se a montagem
/// domina, o traçado está a recompilar uma árvore idêntica em cada quadro.
///
/// ```text
/// cargo test -p ph2d-field-eval --release -- --exact \
///     tests::measure_building_the_tape_against_marching_it --ignored --nocapture
/// ```
#[test]
#[ignore]
fn measure_building_the_tape_against_marching_it() {
    use ph2d_field::{FillRule, Profile};
    let reg = hybrid::Registry::new();
    println!("arestas | Hybrid::new | RegionCompiler::new | 60x compile_at | avaliar 100k pontos");
    for n in [168usize, 336, 672, 1344] {
        let contour: Vec<[f32; 2]> = (0..n)
            .map(|i| {
                let a = std::f64::consts::TAU * (i as f64) / (n as f64);
                [(0.6 * a.cos()) as f32, (0.6 * a.sin()) as f32]
            })
            .collect();
        let profile = Profile::new(vec![contour], FillRule::NonZero, 1e-4).expect("perfil");
        let doc = FieldDoc::new(
            vec![leaf(
                Primitive::Extrude {
                    profile,
                    half_height: 0.4,
                    round: 0.06,
                },
                Xform::IDENTITY,
            )],
            NodeId(0),
        )
        .expect("extrusão");

        let _ = hybrid::Hybrid::new(&doc, &reg);
        let mut build: Vec<f64> = (0..5)
            .map(|_| {
                let t = std::time::Instant::now();
                let h = hybrid::Hybrid::new(&doc, &reg);
                let ms = t.elapsed().as_secs_f64() * 1000.0;
                drop(h);
                ms
            })
            .collect();
        build.sort_by(f64::total_cmp);

        // ⭐ **O ÍNDICE DO PERFIL** — uma BVH sobre as arestas, construída UMA VEZ POR TRAÇADO
        // dentro do `RegionCompiler::new`. Candidato ao custo fixo, e ainda por medir.
        let _ = RegionCompiler::new(&doc);
        let mut index: Vec<f64> = (0..5)
            .map(|_| {
                let t = std::time::Instant::now();
                let rc = RegionCompiler::new(&doc);
                let ms = t.elapsed().as_secs_f64() * 1000.0;
                drop(rc);
                ms
            })
            .collect();
        index.sort_by(f64::total_cmp);

        // ⭐⭐ **A ESPECIALIZAÇÃO POR LADRILHO** — `60` é a contagem de um quadro a `D=6`
        // (`320×180`, ladrilhos de 64, até 4 regiões de fatia cada). É esta que sobra depois de a
        // montagem base ter sido ilibada.
        let rc = RegionCompiler::new(&doc);
        let corners = [
            [-0.7f32, -0.7, -0.7],
            [0.7, -0.7, -0.7],
            [-0.7, 0.7, -0.7],
            [0.7, 0.7, 0.7],
        ];
        let one_tile = || {
            let tree = rc.compile_at(&doc, [-0.7; 3], [0.7; 3], &corners);
            drop(hybrid::Hybrid::from_tree(tree));
        };
        one_tile();
        let mut tiles: Vec<f64> = (0..3)
            .map(|_| {
                let t = std::time::Instant::now();
                for _ in 0..60 {
                    one_tile();
                }
                t.elapsed().as_secs_f64() * 1000.0
            })
            .collect();
        tiles.sort_by(f64::total_cmp);

        let mut h = hybrid::Hybrid::new(&doc, &reg);
        let pts: Vec<f32> = (0..100_000).map(|i| (i as f32) * 1.0e-5 - 0.5).collect();
        let _ = h.eval(&pts, &pts, &pts).expect("avalia");
        let mut march: Vec<f64> = (0..5)
            .map(|_| {
                let t = std::time::Instant::now();
                let _ = h.eval(&pts, &pts, &pts).expect("avalia");
                t.elapsed().as_secs_f64() * 1000.0
            })
            .collect();
        march.sort_by(f64::total_cmp);
        println!(
            "{:7} | {:11.2} | {:19.2} | {:14.2} | {:19.2}",
            profile_edges(&doc),
            build[2],
            index[2],
            tiles[1],
            march[2]
        );
    }
}

/// As arestas do perfil que este documento tem — para a sonda dizer o número que importa.
fn profile_edges(doc: &FieldDoc) -> usize {
    doc.nodes()
        .iter()
        .filter_map(|n| match &n.kind {
            ph2d_field::NodeKind::Leaf(
                Primitive::Extrude { profile, .. } | Primitive::Revolve { profile },
            ) => Some(profile.segment_count()),
            _ => None,
        })
        .sum()
}

/// ⭐⭐⭐ **AS QUATRO FITAS DE UMA ESPECIALIZAÇÃO — e três delas ninguém lê** (W70).
///
/// A W68 mediu que o traçado é **quase só montagem** (`132` árvores por quadro a `640×360`, a
/// `0,33–0,54 ms` cada) e a W69 tirou-lhe o eixo do TETO. Sobra a base — e esta sonda parte-a nas
/// peças de que ela é feita, porque *atacar uma soma sem a repartir é escolher a metade errada com
/// 50 % de hipótese*.
///
/// # O que se paga hoje, por especialização
///
/// 1. a **árvore** (`compile_at`) — percorre a arena e especializa o perfil na região;
/// 2. a fita **float** (`Engine::from` + `ez_float_slice_tape`) — a que a marcha avalia;
/// 3. a fita **grad** (`ez_grad_slice_tape`) — construída por [`hybrid::Hybrid::from_parts`]
///    **sempre**;
/// 4. e depois **`fork()`**, que a marcha chama no lote e que reconstrói **as duas** outra vez.
///
/// ⚠️ **O consumidor da fita de gradiente é UM, e não é o traçado:** `Hybrid::gradients` só é
/// chamado pela extração de malha (`extract.rs`), na exportação. A normal do traçado sai de
/// **diferenças centrais na fita float** (`march::normals_into`, seis amostras por acerto).
///
/// ```text
/// cargo test -p ph2d-field-eval --release -- --exact \
///     tests::measure_the_four_tapes_of_one_specialisation --ignored --nocapture
/// ```
#[test]
#[ignore]
fn measure_the_four_tapes_of_one_specialisation() {
    use fidget::shape::EzShape;
    use ph2d_field::{FillRule, Profile};
    use std::time::Instant;
    let med = |mut v: Vec<f64>| -> f64 {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    println!("arestas | árvore | float | grad | from_tree | fork | 132x(from+fork)");
    for n in [168usize, 336, 672] {
        let contour: Vec<[f32; 2]> = (0..n)
            .map(|i| {
                let a = std::f64::consts::TAU * (i as f64) / (n as f64);
                [(0.6 * a.cos()) as f32, (0.6 * a.sin()) as f32]
            })
            .collect();
        let profile = Profile::new(vec![contour], FillRule::NonZero, 1e-4).expect("perfil");
        let doc = FieldDoc::new(
            vec![leaf(
                Primitive::Extrude {
                    profile,
                    half_height: 0.4,
                    round: 0.06,
                },
                Xform::IDENTITY,
            )],
            NodeId(0),
        )
        .expect("extrusão");
        let rc = RegionCompiler::new(&doc);
        // Uma região de ladrilho: uma fatia fina atravessando a peça.
        let (lo, hi) = ([-0.7f32, -0.2, -0.7], [0.7f32, 0.2, 0.7]);
        let corners = probe_box_corners(lo, hi);
        let tree_ms = med((0..9)
            .map(|_| {
                let t = Instant::now();
                let tr = rc.compile_at(&doc, lo, hi, &corners);
                let ms = t.elapsed().as_secs_f64() * 1000.0;
                drop(tr);
                ms
            })
            .collect());
        let tree = rc.compile_at(&doc, lo, hi, &corners);
        let float_ms = med((0..9)
            .map(|_| {
                let t = Instant::now();
                let shape = Engine::from(tree.clone());
                let pair = (Engine::new_float_slice_eval(), shape.ez_float_slice_tape());
                let ms = t.elapsed().as_secs_f64() * 1000.0;
                drop(pair);
                ms
            })
            .collect());
        let grad_ms = med((0..9)
            .map(|_| {
                let t = Instant::now();
                let shape = Engine::from(tree.clone());
                let pair = (Engine::new_grad_slice_eval(), shape.ez_grad_slice_tape());
                let ms = t.elapsed().as_secs_f64() * 1000.0;
                drop(pair);
                ms
            })
            .collect());
        let from_ms = med((0..9)
            .map(|_| {
                let t = Instant::now();
                let h = hybrid::Hybrid::from_tree(tree.clone());
                let ms = t.elapsed().as_secs_f64() * 1000.0;
                drop(h);
                ms
            })
            .collect());
        let base = hybrid::Hybrid::from_tree(tree.clone());
        let fork_ms = med((0..9)
            .map(|_| {
                let t = Instant::now();
                let f = base.fork();
                let ms = t.elapsed().as_secs_f64() * 1000.0;
                drop(f);
                ms
            })
            .collect());
        println!(
            "{n:7} | {tree_ms:6.3} | {float_ms:5.3} | {grad_ms:4.3} | {from_ms:9.3} | {fork_ms:4.3} | {:15.1}",
            (from_ms + fork_ms) * 132.0
        );
    }
}

/// ⭐⭐⭐ **O GRADIENTE DE UMA COMPOSIÇÃO** (W75) — a cerca que o [`crate::safe_march_step`] declara
/// e que **ninguém tinha medido**.
///
/// O doc dele diz, à letra: *«⛔ Não se compõe um limite por nó: encadear misturas pode compor os
/// factores, e essa pergunta **não foi medida**»*. ⚠️ E a consequência de ela estar errada não é
/// lentidão: **é a peça a FURAR** — a marcha anda `d · s` e só é segura enquanto `s · ‖∇f‖ ≤ 1`,
/// então um `‖∇f‖` acima de `√2` numa composição torna o passo de hoje **grande demais**.
///
/// A tabela abaixo mede exactamente o que a nota deixou por medir: arredondamentos exactos
/// **encadeados**, e exactos por baixo de cada modificador.
///
/// ```text
/// cargo test -p ph2d-field-eval --profile ci-test -- --exact \
///     tests::the_table_of_the_gradient_of_a_composition --ignored --nocapture
/// ```
#[test]
#[ignore]
fn the_table_of_the_gradient_of_a_composition() {
    let bound = f64::from(std::f32::consts::SQRT_2);
    println!("composição | ‖∇f‖ | contra √2");
    for (name, doc) in composition_cases() {
        let g = worst_gradient(&doc, 1.0, 40);
        println!(
            "{name:>34} | {g:.4} | {:.2}x {}",
            g / bound,
            if g > bound * 1.02 { "⛔ FURA" } else { "ok" }
        );
    }
}

/// As composições que a cerca do passo declara e não media — usadas pela sonda **e** pelo gate, que
/// é o que impede a tabela de dizer uma coisa e o gate defender outra.
fn composition_cases() -> Vec<(String, FieldDoc)> {
    let bx = |h: [f32; 3], at: Xform| {
        leaf(
            Primitive::Box {
                half: h,
                round: 0.0,
            },
            at,
        )
    };
    let mut out: Vec<(String, FieldDoc)> = Vec::new();
    for r in [0.05f32, 0.2, 0.5] {
        let ex = |r: f32| Op::Union(Blend::Exact { radius: r });
        // Dois exactos ENCADEADOS: o de cima recebe o campo já inflado pelo de baixo.
        out.push((
            format!("Union Exact {r} × 2 encadeados"),
            FieldDoc::new(
                vec![
                    bx([0.5, 0.25, 0.25], Xform::at(-0.25, 0.0, 0.0)),
                    bx([0.25, 0.5, 0.25], Xform::at(0.25, 0.0, 0.0)),
                    combine(ex(r), vec![NodeId(0), NodeId(1)]),
                    bx([0.25, 0.25, 0.5], Xform::at(0.0, 0.25, 0.0)),
                    combine(ex(r), vec![NodeId(2), NodeId(3)]),
                ],
                NodeId(4),
            )
            .expect("a peça"),
        ));
        // TRÊS encadeados — se o factor compõe, é aqui que ele se vê.
        out.push((
            format!("Union Exact {r} × 3 encadeados"),
            FieldDoc::new(
                vec![
                    bx([0.5, 0.25, 0.25], Xform::at(-0.25, 0.0, 0.0)),
                    bx([0.25, 0.5, 0.25], Xform::at(0.25, 0.0, 0.0)),
                    combine(ex(r), vec![NodeId(0), NodeId(1)]),
                    bx([0.25, 0.25, 0.5], Xform::at(0.0, 0.25, 0.0)),
                    combine(ex(r), vec![NodeId(2), NodeId(3)]),
                    bx([0.3, 0.3, 0.3], Xform::at(0.0, -0.25, 0.15)),
                    combine(ex(r), vec![NodeId(4), NodeId(5)]),
                ],
                NodeId(6),
            )
            .expect("a peça"),
        ));
        // Uma DIFERENÇA exacta por cima de uma união exacta — as duas famílias que a tabela do
        // `safe_march_step` mede em separado, agora uma sobre a outra.
        out.push((
            format!("Difference Exact {r} sobre Union Exact {r}"),
            FieldDoc::new(
                vec![
                    bx([0.5, 0.25, 0.25], Xform::at(-0.25, 0.0, 0.0)),
                    bx([0.25, 0.5, 0.25], Xform::at(0.25, 0.0, 0.0)),
                    combine(ex(r), vec![NodeId(0), NodeId(1)]),
                    leaf(Primitive::Sphere { radius: 0.35 }, Xform::at(0.1, 0.1, 0.3)),
                    combine(
                        Op::Difference(Blend::Exact { radius: r }),
                        vec![NodeId(2), NodeId(3)],
                    ),
                ],
                NodeId(4),
            )
            .expect("a peça"),
        ));
    }
    // ⭐⭐⭐ **E o nó de N FILHOS, que é uma corrente disfarçada** (W75): o `combine_trees` dobra da
    // esquerda para a direita, então um `Union Exact` com `n` filhos são **`n − 1`** arredondamentos
    // encadeados **dentro de um nó só** — e é exactamente a forma da cena 1 do smoke (três cilindros
    // numa união exacta). *Uma fixtura de dois filhos não vê a corrente que o lowering constrói.*
    for n in [3usize, 4, 5] {
        let mut nodes: Vec<ph2d_field::Node> = (0..n)
            .map(|i| {
                let a = std::f64::consts::TAU * (i as f64) / (n as f64);
                bx(
                    [0.35, 0.18, 0.18],
                    Xform::at(0.3 * a.cos() as f32, 0.3 * a.sin() as f32, 0.0),
                )
            })
            .collect();
        let kids: Vec<NodeId> = (0..n)
            .map(|i| NodeId(u32::try_from(i).expect("poucos")))
            .collect();
        nodes.push(combine(Op::Union(Blend::Exact { radius: 0.2 }), kids));
        out.push((
            format!("Union Exact 0,2 com {n} filhos (UM nó)"),
            FieldDoc::new(nodes, NodeId(u32::try_from(n).expect("poucos"))).expect("a peça"),
        ));
    }

    // E um exacto por BAIXO de cada modificador — o mod recebe um campo que já não é distância.
    for (mn, m) in [
        ("Shell 0,05", ph2d_field::Unary::Shell { thickness: 0.05 }),
        ("Offset 0,1", ph2d_field::Unary::Offset { distance: 0.1 }),
        ("Taper 1,0", ph2d_field::Unary::Taper { slope: 1.0 }),
        ("Mirror", ph2d_field::Unary::Mirror),
        ("Radial 5", ph2d_field::Unary::Radial { count: 5 }),
        (
            "Array 0,5",
            ph2d_field::Unary::Array {
                count: 3,
                spacing: 0.5,
            },
        ),
    ] {
        let mut top = combine(
            Op::Union(Blend::Exact { radius: 0.2 }),
            vec![NodeId(0), NodeId(1)],
        );
        top.mods.push(m);
        out.push((
            format!("{mn} sobre Union Exact 0,2"),
            FieldDoc::new(
                vec![
                    bx([0.5, 0.25, 0.25], Xform::at(-0.25, 0.0, 0.0)),
                    bx([0.25, 0.5, 0.25], Xform::at(0.25, 0.0, 0.0)),
                    top,
                ],
                NodeId(2),
            )
            .expect("a peça"),
        ));
    }
    out
}

/// ⭐⭐⭐ **A PROFUNDIDADE conta níveis ENCADEADOS, não nós inflantes soltos** (W75).
///
/// ⚠️ **A metade que separa as duas leituras é a dos IRMÃOS:** dois arredondamentos exactos em ramos
/// diferentes não se compõem — o campo que cada um recebe é distância verdadeira —, e contá-los
/// daria `2` a uma peça que se marcha com segurança a `1/√2`. *Uma contagem e uma profundidade lêem
/// igual em toda árvore que seja uma corrente, e só divergem onde a árvore se abre.*
#[test]
fn the_depth_counts_chained_rounds_and_not_loose_nodes() {
    let bx = |h: [f32; 3], at: Xform| {
        leaf(
            Primitive::Box {
                half: h,
                round: 0.0,
            },
            at,
        )
    };
    let ex = |r: f32| Op::Union(Blend::Exact { radius: r });
    let two = || {
        vec![
            bx([0.4, 0.2, 0.2], Xform::at(-0.2, 0.0, 0.0)),
            bx([0.2, 0.4, 0.2], Xform::at(0.2, 0.0, 0.0)),
        ]
    };

    let mut n = two();
    n.push(combine(Op::Union(Blend::Sharp), vec![NodeId(0), NodeId(1)]));
    assert_eq!(
        inflation_depth(&FieldDoc::new(n, NodeId(2)).expect("peça")),
        0,
        "uma junção viva não infla nada"
    );

    let mut n = two();
    n.push(combine(
        Op::Union(Blend::Organic { k: 0.3 }),
        vec![NodeId(0), NodeId(1)],
    ));
    assert_eq!(
        inflation_depth(&FieldDoc::new(n, NodeId(2)).expect("peça")),
        0,
        "o carácter orgânico não infla — medido na tabela do `safe_march_step`"
    );

    let mut n = two();
    n.push(combine(ex(0.2), vec![NodeId(0), NodeId(1)]));
    assert_eq!(
        inflation_depth(&FieldDoc::new(n, NodeId(2)).expect("peça")),
        1,
        "um exacto é um nível"
    );

    // ⭐ **Encadeados**: o de cima recebe o campo já inflado.
    let mut n = two();
    n.push(combine(ex(0.2), vec![NodeId(0), NodeId(1)]));
    n.push(bx([0.25; 3], Xform::at(0.0, 0.25, 0.0)));
    n.push(combine(ex(0.2), vec![NodeId(2), NodeId(3)]));
    assert_eq!(
        inflation_depth(&FieldDoc::new(n, NodeId(4)).expect("peça")),
        2,
        "dois exactos na mesma corrente são dois níveis"
    );

    // ⭐⭐ **IRMÃOS**: dois exactos, nenhum por cima do outro.
    let mut n = two();
    n.push(combine(ex(0.2), vec![NodeId(0), NodeId(1)]));
    n.push(bx([0.3, 0.15, 0.15], Xform::at(-0.2, 0.4, 0.0)));
    n.push(bx([0.15, 0.3, 0.15], Xform::at(0.2, 0.4, 0.0)));
    n.push(combine(ex(0.2), vec![NodeId(3), NodeId(4)]));
    n.push(combine(Op::Union(Blend::Sharp), vec![NodeId(2), NodeId(5)]));
    assert_eq!(
        inflation_depth(&FieldDoc::new(n, NodeId(6)).expect("peça")),
        1,
        "dois exactos IRMÃOS não se compõem — cada um recebe distância verdadeira"
    );

    // ⚠️ E um modificador por cima de um exacto **não** acrescenta nível (medido: `1,4142`).
    let mut n = two();
    let mut top = combine(ex(0.2), vec![NodeId(0), NodeId(1)]);
    top.mods.push(ph2d_field::Unary::Shell { thickness: 0.05 });
    n.push(top);
    assert_eq!(
        inflation_depth(&FieldDoc::new(n, NodeId(2)).expect("peça")),
        1,
        "o modificador lê o campo, não o volta a arredondar"
    );

    // ⭐⭐⭐ **E um nó de N filhos é uma corrente de `n − 1`** — o `combine_trees` dobra aos pares.
    // ⛔ É a forma da cena 1 do smoke, e ela marchava acima do seguro desde que existe.
    for n in [2usize, 3, 5] {
        let mut nodes: Vec<Node> = (0..n)
            .map(|i| bx([0.3, 0.15, 0.15], Xform::at(0.2 * i as f32, 0.0, 0.0)))
            .collect();
        let kids: Vec<NodeId> = (0..n)
            .map(|i| NodeId(u32::try_from(i).expect("poucos")))
            .collect();
        nodes.push(combine(ex(0.2), kids));
        let root = NodeId(u32::try_from(n).expect("poucos"));
        assert_eq!(
            inflation_depth(&FieldDoc::new(nodes, root).expect("peça")),
            u32::try_from(n - 1).expect("poucos"),
            "uma união exacta de {n} filhos é uma corrente de {} arredondamentos",
            n - 1
        );
    }
}

/// ⭐⭐⭐ **O ESPELHO DEMONSTRA-SE — a nota dizia que não** (W78).
///
/// A lista de aberto do módulo carrega, desde a W17, *«o Mirror não se consegue demonstrar: ele
/// dobra em torno do centro do objecto, e o que falta é um alvo descentrado ou um pivô autorado»*.
///
/// ⚠️ **Um alvo descentrado É exprimível hoje**, e por duas portas: o modificador entra em **qualquer
/// nó menos uma escultura** (`field3d_scene_panel::mods_for`), e um nó de **operação** tem filhos com
/// pose própria. ⇒ pôr o `Mirror` na operação dobra os filhos em torno do centro **dela**, e uma
/// caixa fora do eixo aparece **dos dois lados**.
///
/// *Este gate é a demonstração, e existe porque a nota afirmava a ausência sem a medir.*
///
/// ⭐⭐ **E são TRÊS eixos desde 2026-08-26** (pedido do Enio, depois de o ver a funcionar em X). O
/// gate percorre-os pelo índice do eixo — três blocos escritos à mão seriam três sítios onde um
/// índice trocado passa despercebido, que é exactamente o defeito que um espelho de eixo errado é.
#[test]
fn a_mirror_on_an_operation_folds_an_off_centre_child() {
    for (axis, m) in [
        (0usize, ph2d_field::Unary::Mirror),
        (1, ph2d_field::Unary::MirrorY),
        (2, ph2d_field::Unary::MirrorZ),
    ] {
        let off = 0.35f32;
        let mut at = [0.0f32; 3];
        at[axis] = off;
        let child = leaf(
            Primitive::Box {
                half: [0.12, 0.12, 0.12],
                round: 0.0,
            },
            Xform::at(at[0], at[1], at[2]),
        );
        let plain = FieldDoc::new(
            vec![
                child.clone(),
                combine(Op::Union(Blend::Sharp), vec![NodeId(0)]),
            ],
            NodeId(1),
        )
        .expect("a peça");
        let mut top = combine(Op::Union(Blend::Sharp), vec![NodeId(0)]);
        top.mods.push(m);
        let mirrored = FieldDoc::new(vec![child, top], NodeId(1)).expect("a peça espelhada");

        let mut here = [0.0f64; 3];
        here[axis] = f64::from(off);
        let mut there = [0.0f64; 3];
        there[axis] = f64::from(-off);
        let f = |d: &FieldDoc, p: [f64; 3]| Field::new(d).at(p[0], p[1], p[2]);

        assert!(f(&plain, here) < 0.0, "a caixa está onde foi posta");
        assert!(
            f(&plain, there) > 0.0,
            "e sem espelho não há nada do outro lado — senão o gate media a peça errada"
        );
        assert!(
            f(&mirrored, there) < 0.0,
            "⛔ com o espelho na OPERAÇÃO, o outro lado tem de ficar sólido — é a demonstração que a \
         nota dizia não existir"
        );
        assert!(
            (f(&mirrored, there) - f(&plain, here)).abs() < 1.0e-6,
            "eixo {axis}: os dois lados têm de ter o MESMO campo — o espelho é uma dobra, não uma \
         cópia aproximada"
        );
        // ⛔ **E o eixo é o CERTO**: espelhar em Y não pode fazer aparecer nada em `-x`. Sem esta
        // metade, os três braços podiam ser o mesmo braço.
        for other in 0..3 {
            if other == axis {
                continue;
            }
            let mut elsewhere = [0.0f64; 3];
            elsewhere[other] = f64::from(-off);
            assert!(
                f(&mirrored, elsewhere) > 0.0,
                "eixo {axis}: apareceu peça em -{other}, que não é o eixo espelhado"
            );
        }
    }
}
