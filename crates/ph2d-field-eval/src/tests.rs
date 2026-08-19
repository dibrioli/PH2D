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
    Node {
        xform: Xform::IDENTITY,
        kind: NodeKind::Combine { op, children },
    }
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
