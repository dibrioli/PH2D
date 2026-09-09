//! ⭐⭐ **QUANTO CUSTA UMA PONTA A MAIS NUMA ESTRELA, E QUANTO CUSTA UM ELIPSÓIDE ACHATADO**
//! (W103) — as duas sondas que os números de [`ph2d_field::MAX_STAR_POINTS`] e da nota do
//! [`ph2d_field_eval::ops::sd_ellipsoid`] citam.
//!
//! ⚠️ **A régua é a MESMA do prisma** ([`measure_prism_sides`](measure_prism_sides.rs)): o cilindro
//! exato vale `1,00×`, porque é a forma redonda mais barata que este módulo tem e é contra ela que
//! o teto de lados foi calibrado. Comparar duas famílias com réguas diferentes não escolheria nada.
//!
//! ```text
//! cargo test -p ph2d-field-eval --release --test measure_star_points -- --ignored --nocapture
//! ```

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Xform};

/// `(nós da árvore, ns por ponto)` — a mediana de 7 corridas sobre `2^18` pontos, com uma corrida a
/// frio antes de medir.
fn cost_of(tree: fidget::context::Tree) -> (usize, f64) {
    use fidget::shape::EzShape;
    const N: usize = 1 << 18;
    let coord =
        |i: usize, k: usize| -0.9 + 1.8 * (((i * 7919 + k * 104_729) % 1024) as f32) / 1024.0;
    let xs: Vec<f32> = (0..N).map(|i| coord(i, 0)).collect();
    let ys: Vec<f32> = (0..N).map(|i| coord(i, 1)).collect();
    let zs: Vec<f32> = (0..N).map(|i| coord(i, 2)).collect();

    let mut ctx = fidget::context::Context::new();
    let _ = ctx.import(&tree);
    let nos = ctx.len();

    let shape = ph2d_field_eval::Engine::from(tree);
    let tape = shape.ez_float_slice_tape();
    let mut eval = ph2d_field_eval::Engine::new_float_slice_eval();
    let _ = eval.eval(&tape, &xs, &ys, &zs).expect("avalia");
    let mut a: Vec<f64> = (0..7)
        .map(|_| {
            let t0 = std::time::Instant::now();
            let _ = eval.eval(&tape, &xs, &ys, &zs).expect("avalia");
            t0.elapsed().as_secs_f64() * 1.0e9 / N as f64
        })
        .collect();
    a.sort_by(f64::total_cmp);
    (nos, a[3])
}

fn cost(p: Primitive) -> (usize, f64) {
    let doc = FieldDoc::new(
        vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p))],
        NodeId(0),
    )
    .expect("a peça");
    cost_of(ph2d_field_eval::compile(&doc))
}

#[test]
#[ignore]
fn measure_star_points() {
    let cilindro = cost(Primitive::Cylinder {
        radius: 0.45,
        half_height: 0.3,
        round: 0.05,
        chamfer: 0.0,
    });
    println!("  forma                  | semiplanos |  nós |  ns/ponto | × o cilindro");
    println!(
        "{:>22} | {:>10} | {:4} | {:6.2} ns | {:11.2}×",
        "cilindro (exacto)", "-", cilindro.0, cilindro.1, 1.0
    );
    for n in [3_u32, 4, 5, 6, 8, 10, 12, 16, 24, 32] {
        // ⚠️ Acima do teto o documento recusa (é a cerca do produto), então a sonda monta a árvore
        // **directamente** — a mesma razão que a sonda do prisma registou: *uma sonda que só alcança
        // o lado de dentro do limite não pode dizer que o limite está no sítio certo*.
        let (nos, ns) = if n <= ph2d_field::MAX_STAR_POINTS {
            cost(Primitive::Star {
                points: n,
                outer: 0.45,
                inner: 0.18,
                half_height: 0.3,
                round: 0.02,
                chamfer: 0.0,
                corner_chamfer: 0.0,
            })
        } else {
            cost_of(ph2d_field_eval::ops::sd_star(
                n, 0.45, 0.18, 0.3, 0.02, 0.0, 0.0,
            ))
        };
        let cerca = if n > ph2d_field::MAX_STAR_POINTS {
            " (fora da cerca)"
        } else {
            ""
        };
        println!(
            "{:>22} | {:>10} | {nos:4} | {ns:6.2} ns | {:11.2}×{cerca}",
            format!("estrela de {n} pontas"),
            4 * n,
            ns / cilindro.1
        );
    }

    // ⭐ E as duas formas **sem contagem**, para o teto delas não ser um palpite tampouco.
    for (nome, p) in [
        (
            "gaiola de caixa",
            Primitive::BoxFrame {
                half: [0.45, 0.45, 0.45],
                thickness: 0.13,
                round: 0.02,
                chamfer: 0.0,
            },
        ),
        (
            "elipsóide 1:1:1",
            Primitive::Ellipsoid {
                radii: [0.45, 0.45, 0.45],
            },
        ),
        (
            "elipsóide 1:4",
            Primitive::Ellipsoid {
                radii: [0.45, 0.1125, 0.45],
            },
        ),
        (
            "elipsóide 1:16",
            Primitive::Ellipsoid {
                radii: [0.45, 0.028_125, 0.45],
            },
        ),
    ] {
        let (nos, ns) = cost(p);
        println!(
            "{nome:>22} | {:>10} | {nos:4} | {ns:6.2} ns | {:11.2}×",
            "-",
            ns / cilindro.1
        );
    }
}

/// ⭐⭐⭐ **O QUE UM ELIPSÓIDE ACHATADO CUSTA À MARCHA** — e este é o recurso de que o limite dele
/// seria feito, se ele tivesse um.
///
/// A [`ph2d_field_eval::ops::sd_ellipsoid`] é um **subestimador** por um fator que é exatamente
/// `min(s)/max(s)` na pior direção. Subestimar não erra: custa **passos**. Esta sonda conta os
/// passos de uma marcha de esferas até à superfície, a partir de fora, na direção em que o
/// subestimador é pior — e é isso que diz se um elipsóide de `1:16` é utilizável ou uma armadilha.
#[test]
#[ignore]
fn measure_ellipsoid_march() {
    use fidget::shape::EzShape;
    let passos = |radii: [f32; 3]| -> (usize, f64) {
        let doc = FieldDoc::new(
            vec![Node::new(
                Xform::IDENTITY,
                NodeKind::Leaf(Primitive::Ellipsoid { radii }),
            )],
            NodeId(0),
        )
        .expect("a peça");
        let shape = ph2d_field_eval::Engine::from(ph2d_field_eval::compile(&doc));
        let tape = shape.ez_float_slice_tape();
        let mut eval = ph2d_field_eval::Engine::new_float_slice_eval();
        // ⚠️ **Ao longo do eixo MAIOR** — é ali que o campo é escalado pelo menor semi-eixo, logo é
        // onde o subestimador é pior. Uma marcha pelo eixo menor mediria o caso fácil.
        let mut x = 1.5_f32;
        let mut n = 0;
        while n < 4096 {
            let d = eval.eval(&tape, &[x], &[0.0], &[0.0]).expect("avalia")[0];
            if d < 1.0e-4 {
                break;
            }
            x -= d;
            n += 1;
        }
        (n, f64::from(x))
    };
    println!("  elipsóide  | passos até à superfície | x final (o raio é 0,45)");
    for k in [
        1.0_f32,
        0.5,
        0.25,
        0.125,
        0.0625,
        0.031_25,
        0.015_625,
        0.007_812_5,
    ] {
        let (n, x) = passos([0.45, 0.45 * k, 0.45 * k]);
        println!("  1:{:>5.0}    | {n:>23} | {x:.6}", 1.0 / k);
    }
}

/// ⭐⭐⭐ **A FÓRMULA PUBLICADA DO ELIPSÓIDE, MEDIDA CONTRA A NOSSA** — porque «preferi uma prova»
/// não é motivo, e a alternativa tinha de ser experimentada.
///
/// A referência (IQ, *distance functions*) publica `k0·(k0−1)/k1`, com `k0 = |p/r|` e `k1 = |p/r²|`
/// — bem mais **apertada** que a nossa (a esfera reescalada por `min(r)`), e portanto mais barata de
/// marchar. Esta sonda mede as duas onde importa: o **centro da peça** e o **pior gradiente**.
///
/// ⚠️ `k1` é `0` na origem, e `k0` também: a fórmula é `0/0` **exatamente no centro do sólido**, que
/// é o ponto que toda grade centrada amostra. O piso do `sqrt` desta crate transforma o `NaN` num
/// **zero** — um zero no meio de uma peça é uma superfície fantasma para quem procura troca de
/// sinal.
#[test]
#[ignore]
fn measure_ellipsoid_against_the_published_formula() {
    use fidget::context::Tree;
    use fidget::shape::EzShape;
    fn iq(radii: [f64; 3]) -> Tree {
        let over = |t: Tree, r: f64| t * Tree::constant(1.0 / r);
        let sq = |t: Tree| t.square();
        let k0 = (sq(over(Tree::x(), radii[0]))
            + sq(over(Tree::y(), radii[1]))
            + sq(over(Tree::z(), radii[2])))
        .max(1.0e-30)
        .sqrt();
        let k1 = (sq(over(Tree::x(), radii[0] * radii[0]))
            + sq(over(Tree::y(), radii[1] * radii[1]))
            + sq(over(Tree::z(), radii[2] * radii[2])))
        .max(1.0e-30)
        .sqrt();
        k0.clone() * (k0 - Tree::constant(1.0)) / k1
    }
    let sonda = |tree: Tree| -> (f32, f64) {
        let shape = ph2d_field_eval::Engine::from(tree);
        let tape = shape.ez_float_slice_tape();
        let mut eval = ph2d_field_eval::Engine::new_float_slice_eval();
        let centro = eval.eval(&tape, &[0.0], &[0.0], &[0.0]).expect("avalia")[0];
        // O pior gradiente por diferenças centrais, numa nuvem à volta da peça.
        const H: f32 = 1.0e-3;
        let mut pior = 0.0_f64;
        let step = 2.0_f32 / 24.0;
        for i in 0..24_u8 {
            for j in 0..24_u8 {
                for k in 0..24_u8 {
                    let p = [
                        f32::from(i) * step - 1.0,
                        f32::from(j) * step - 1.0,
                        f32::from(k) * step - 1.0,
                    ];
                    // ⚠️ Os seis pontos do estêncil vão numa avaliação só, e cada eixo tem de
                    // aparecer nas TRÊS listas: a fita recebe triplos, não uma coordenada.
                    let xs = [p[0] - H, p[0] + H, p[0], p[0], p[0], p[0]];
                    let ys = [p[1], p[1], p[1] - H, p[1] + H, p[1], p[1]];
                    let zs = [p[2], p[2], p[2], p[2], p[2] - H, p[2] + H];
                    let Ok(o) = eval.eval(&tape, &xs, &ys, &zs) else {
                        continue;
                    };
                    let d = |a: usize| f64::from(o[a + 1] - o[a]);
                    let g = [d(0), d(2), d(4)];
                    let n = (g[0] * g[0] + g[1] * g[1] + g[2] * g[2]).sqrt() / f64::from(2.0 * H);
                    if n.is_finite() && n > pior {
                        pior = n;
                    }
                }
            }
        }
        (centro, pior)
    };
    println!(
        "  elipsóide  |            f(centro) esperado |  a NOSSA f(0) | pior ‖∇f‖ |  a do IQ f(0) | pior ‖∇f‖"
    );
    for k in [1.0_f64, 0.5, 0.25, 0.125] {
        let radii = [0.45, 0.45 * k, 0.45 * k];
        let (c_nosso, g_nosso) = sonda(ph2d_field_eval::ops::sd_ellipsoid(radii));
        let (c_iq, g_iq) = sonda(iq(radii));
        println!(
            "  1:{:>5.0}    | {:29.6} | {c_nosso:13.6} | {g_nosso:9.4} | {c_iq:13.6} | {g_iq:9.4}",
            1.0 / k,
            -(0.45 * k)
        );
    }
}

// ─────────────────────────── W141 ───────────────────────────

/// A altura da face de cima sobre `(x, y)` — `None` onde não há peça.
fn topo_da_estrela(f: &ph2d_field_eval::Field, x: f64, y: f64, h: f64) -> Option<f64> {
    if f.at(x, y, h * 2.0) < 0.0 || f.at(x, y, 0.0) >= 0.0 {
        return None;
    }
    let (mut lo, mut hi) = (0.0_f64, h * 2.0);
    for _ in 0..50 {
        let m = f64::midpoint(lo, hi);
        if f.at(x, y, m) < 0.0 {
            lo = m;
        } else {
            hi = m;
        }
    }
    Some(lo)
}

/// ⭐⭐⭐ **O CHANFRO DA ESTRELA FICA NAS BORDAS, e não entra no MIOLO** (W141).
///
/// # ⛔⛔ O report do Enio (08/09, três fotos)
///
/// > *«o Chamfer múltiplo acaba criando um padrão complexo que afeta até o miolo da estrela. Não
/// > fica apenas nas bordas externas.»*
///
/// ⭐ **A régua é a ALTURA DA FACE DE CIMA, ponto a ponto.** Numa chapa a tampa é plana; o chanfro
/// do aro encolhe o contorno dela e a tampa que sobra **continua plana**. ⇒ *qualquer variação de
/// altura no miolo é uma faceta que não devia estar lá.*
///
/// # ⭐ A causa, e o que esta cura fecha
///
/// A costura entre duas pipas vizinhas é afastada por uma **folga**, e ela estava dimensionada só
/// pelo `round` — escrita na W104-bis, **antes de o chanfro existir**. Com a folga a cobrir os dois
/// recuos:
///
/// | `chamfer` (round = 0) | antes | **depois** |
/// |---|---:|---:|
/// | `0,02` | 1,2 % | **0,0 %** |
/// | `0,04` | 12,1 % | **0,3 %** |
/// | `0,06` (o tecto) | 30,9 % | 4,6 % |
///
/// ⚠️ **A barra deste gate é `2/3` do tecto**, que é onde a cura leva o miolo a ficar plano com
/// folga. ⛔ **O que FICA aberto e está medido:** com **filete E chanfro juntos** o miolo ainda sai
/// (`22,3 %` a `0,03`+`0,03`) — a causa é outra e vive a montante, no campo das paredes da estrela,
/// que é um **minorante frouxo** por dentro (ele lê `−0,0148` em pontos que estão longe de toda a
/// superfície). Ver o [doc 06 §141](../../../docs/3DModeling/06_resultados_cena_e_gizmo.md).
#[test]
fn the_star_chamfer_leaves_the_middle_of_the_top_face_flat() {
    let (outer, inner, h) = (0.45_f32, 0.18_f32, 0.25_f32);
    let limite = ph2d_field::round_limit(&Primitive::Star {
        points: 5,
        outer,
        inner,
        half_height: h,
        round: 0.0,
        chamfer: 0.0,
        corner_chamfer: 0.0,
    })
    .expect("a estrela tem tecto");
    for fracao in [0.3_f32, 0.5, 0.66] {
        let chamfer = limite * fracao;
        let doc = FieldDoc::new(
            vec![Node::new(
                Xform::IDENTITY,
                NodeKind::Leaf(Primitive::Star {
                    points: 5,
                    outer,
                    inner,
                    half_height: h,
                    round: 0.0,
                    chamfer,
                    corner_chamfer: 0.0,
                }),
            )],
            NodeId(0),
        )
        .expect("a estrela");
        let f = ph2d_field_eval::Field::new(&doc);
        let (mut n, mut fora) = (0_u32, 0_u32);
        const M: usize = 70;
        for i in 0..M {
            for j in 0..M {
                let c = |t: usize| f64::from(inner) * (2.0 * (t as f64) / (M - 1) as f64 - 1.0);
                let (x, y) = (c(i), c(j));
                if x.hypot(y) > f64::from(inner) * 0.92 {
                    continue;
                }
                // ⭐⭐⭐ **FORA a coroa que um canto CÔNCAVO legitimamente come** (W144).
                //
                // Num vale, um chanfro de recuo `c` remove um disco de raio `c` à volta do vértice
                // — isso é a geometria do chanfro, não o defeito que este gate persegue. ⛔ Antes da
                // W144 o `chamfer` também chanfrava o vale e a coroa saía do enquadramento sozinha;
                // com as duas famílias separadas ela cai **dentro** da janela do miolo, e contá-la é
                // contar a forma. ⚠️ **Medido:** `4,24 %` cru contra **`0,36 %`** sem a coroa.
                let no_vale = (0..5).any(|k| {
                    let a = std::f64::consts::PI / 5.0 + std::f64::consts::TAU * f64::from(k) / 5.0;
                    let (vx, vy) = (f64::from(inner) * a.cos(), f64::from(inner) * a.sin());
                    (x - vx).hypot(y - vy) < f64::from(chamfer)
                });
                if no_vale {
                    continue;
                }
                if let Some(z) = topo_da_estrela(&f, x, y, f64::from(h)) {
                    n += 1;
                    if (f64::from(h) - z).abs() > 1.0e-4 {
                        fora += 1;
                    }
                }
            }
        }
        assert!(n > 300, "a fixtura tem de amostrar o miolo (leu {n})");
        let pct = 100.0 * f64::from(fora) / f64::from(n);
        assert!(
            pct <= 1.0,
            "chanfro {chamfer:.4} ({fracao} do tecto): {pct:.1} % do MIOLO da tampa saiu do plano \
             — o chanfro é das bordas EXTERNAS, e está a entrar na peça"
        );
    }
}

/// ⭐⭐⭐ **O CHANFRO DAS PONTAS TEM FAIXA PRÓPRIA, E ELA É EXACTA** (W143, pedido do Enio de 09/09).
///
/// # ⛔ Por que reutilizar o tecto do outro chanfro seria o oposto do pedido
///
/// O pedido nomeia *«esse controle **maior**»*. O `round_limit` da estrela vale `0,06332` e nasce da
/// **erosão da tampa** (até onde o filete ainda arredonda uma estrela); a ponta é uma aresta
/// **vertical**, e o que a limita é ela **encontrar o vale**:
///
/// ```text
/// outer − tip·cos α_ponta  >  inner + chamfer·cos α_vale
/// ```
///
/// ⚠️ As duas metades foram medidas contra a forma: a ponta recua `0,029903` por unidade de chanfro
/// contra `0,0299034` da conta, e o vale sobe de `0,18` para `0,19808` contra `0,18 + 0,01808`.
#[test]
fn the_corner_chamfer_has_a_range_of_its_own_and_it_is_exact() {
    let base = Primitive::Star {
        points: 5,
        outer: 0.45,
        inner: 0.18,
        half_height: 0.25,
        round: 0.0,
        chamfer: 0.0,
        corner_chamfer: 0.0,
    };
    let faces = ph2d_field::round_limit(&base).expect("a estrela tem filete");
    let c = faces * 0.5;
    // ⛔⛔ **A PEÇA tem de ter o chanfro das faces posto ANTES de se medir o tecto das pontas** — o
    // tecto DEPENDE dele (o vale avança `chamfer·cos α_vale`), e medir um sobre uma peça sem o
    // outro é a régua a falar de outra peça. *Custou uma corrida vermelha a descobrir.*
    let mut base = base;
    let ic = ph2d_field::dims(&base)
        .iter()
        .position(|d| d.key == "field.dim.chamfer")
        .expect("a fileira das faces existe");
    ph2d_field::set_dim(&mut base, 0, ic, c).expect("o chanfro das faces cabe");
    let pontas = ph2d_field::star_corner_chamfer_limit(5, 0.45, 0.18);
    println!(
        "  tecto das faces {faces:.5} | tecto das pontas {pontas:.5} ({:.2}x)",
        pontas / faces
    );
    // ⚠️ **A barra desceu de `3,0×` para `2,0×` na W144, e o número não foi afrouxado — a LEI
    // mudou:** o vale juntou-se à ponta no mesmo controlo, e um tecto que agora tem de conter os
    // DOIS recuos é menor (`(outer − inner)/(cos α_ponta + cos α_vale)` contra
    // `(outer − inner)/cos α_ponta`), e ainda leva a folga medida do dobrar aos pares.
    assert!(
        pontas > faces * 2.0,
        "o controlo do contorno tem de ser MAIOR que o das faces — ele deu {pontas:.5} contra \
         {faces:.5}, e o pedido de 09/09 fala num «controle maior»"
    );
    // ⭐ **A conta, contra a geometria da forma** — e não contra ela própria.
    let beta = std::f32::consts::PI / 5.0;
    let u = (0.45_f32 * 0.45 + 0.18 * 0.18 - 2.0 * 0.45 * 0.18 * beta.cos()).sqrt();
    let alfa = (0.18 * beta.sin() / u).asin();
    // ⚠️ **A conta leva a folga MEDIDA do dobrar aos pares** — ver `FOLD_SAFETY` no
    // [`ph2d_field::star_corner_chamfer_limit`]: a lei geométrica sozinha deixa a estrela partir-se
    // em `0,7483` dela nas 112 do corpus.
    let previsto = (0.45 - 0.18) / (alfa.cos() + (alfa + beta).cos().abs()) * 0.72;
    assert!(
        (pontas - previsto).abs() < 1.0e-5,
        "o tecto deu {pontas:.6} e a geometria da estrela pede {previsto:.6}"
    );
    // ⛔⛔ **E o documento RECUSA acima dele** — sem isto o tecto seria uma nota, não uma cerca.
    let mut acima = base.clone();
    let i = ph2d_field::dims(&acima)
        .iter()
        .position(|d| d.key == "field.dim.corner_chamfer")
        .expect("a fileira das pontas existe");
    assert!(
        ph2d_field::set_dim(&mut acima, 0, i, pontas * 1.01).is_err(),
        "o documento aceitou um chanfro de ponta acima do tecto — a estrela deixaria de ter pontas"
    );
    // ⭐ **E aceita mesmo debaixo dele, com a peça ainda a ser uma estrela.**
    let mut dentro = base.clone();
    ph2d_field::set_dim(&mut dentro, 0, i, pontas * 0.95).expect("aceita debaixo do tecto");
    let doc = FieldDoc::new(
        vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(dentro))],
        NodeId(0),
    )
    .expect("a peça é válida a 95 % do tecto");
    let f = ph2d_field_eval::Field::new(&doc);
    let raio = |ang: f64| -> f64 {
        let (mut lo, mut hi) = (0.0_f64, 1.0_f64);
        for _ in 0..60 {
            let m = f64::midpoint(lo, hi);
            if f.at(m * ang.cos(), m * ang.sin(), 0.0) <= 0.0 {
                lo = m;
            } else {
                hi = m;
            }
        }
        lo
    };
    let (ponta, vale) = (raio(0.0), raio(f64::from(beta)));
    println!("  a 95 % do tecto: raio da ponta {ponta:.5}, raio do vale {vale:.5}");
    assert!(
        ponta > vale,
        "a 95 % do tecto a ponta ({ponta:.5}) já não passa o vale ({vale:.5}) — o tecto está alto"
    );
}

/// ⭐⭐⭐ **A FACETA DA PONTA SOBREVIVE AO FILETE ONDE A DAS FACES NÃO SOBREVIVERIA** (W143) — o facto
/// de produto que o pedido comprou.
///
/// # A lei que o motiva
///
/// O filete come a faceta do chanfro acima de `sin α (1+sin α)/cos α × chanfro`, e isso vale
/// **`0,462`** numa ponta de `19,17°` contra **`1,707`** num aro ortogonal — `3,7×` mais cedo. ⇒ com
/// um número só, o artista via as faces chanfradas e as pontas não.
#[test]
fn the_tip_keeps_a_facet_the_face_chamfer_alone_would_lose() {
    let base = Primitive::Star {
        points: 5,
        outer: 0.45,
        inner: 0.18,
        half_height: 0.25,
        round: 0.0,
        chamfer: 0.0,
        corner_chamfer: 0.0,
    };
    let faces = ph2d_field::round_limit(&base).expect("filete");
    let (c, r) = (faces * 0.5, faces * 0.25);
    let pontas = ph2d_field::star_corner_chamfer_limit(5, 0.45, 0.18);
    // A largura da faceta na ponta: na secção `z = 0` ela é o trecho em que `x` não se move.
    let faceta_com = |ch: f32, tip: f32| -> f64 {
        let doc = FieldDoc::new(
            vec![Node::new(
                Xform::IDENTITY,
                NodeKind::Leaf(Primitive::Star {
                    points: 5,
                    outer: 0.45,
                    inner: 0.18,
                    half_height: 0.25,
                    round: r,
                    chamfer: ch,
                    corner_chamfer: tip,
                }),
            )],
            NodeId(0),
        )
        .expect("a peça");
        let f = ph2d_field_eval::Field::new(&doc);
        let bordo = |y: f64| -> f64 {
            let (mut lo, mut hi) = (0.0_f64, 1.0_f64);
            for _ in 0..60 {
                let m = f64::midpoint(lo, hi);
                if f.at(m, y, 0.0) <= 0.0 {
                    lo = m;
                } else {
                    hi = m;
                }
            }
            lo
        };
        let x0 = bordo(0.0);
        let mut larg = 0.0_f64;
        for i in 1..=800 {
            let y = 0.45 * f64::from(i) / 800.0;
            if (bordo(y) - x0).abs() > 1.0e-4 {
                break;
            }
            larg = 2.0 * y;
        }
        larg
    };
    let faceta = |tip: f32| faceta_com(c, tip);
    // ⭐⭐ **O CHÃO É MEDIDO, e não escolhido** — ele é o que esta régua lê numa estrela que **não
    // tem chanfro nenhum**, e cuja ponta é portanto um arco puro. ⛔ Uma constante «dois passos da
    // grelha» lia `0,00225` sobre uma leitura real de `0,00338`, e reprovava produto correcto.
    let chao = faceta_com(0.0, 0.0);
    let sem = faceta(0.0);
    let com = faceta(pontas * 0.5);
    println!("  faceta na ponta: {sem:.5} sem o controlo, {com:.5} com ele (chão {chao:.5})");
    assert!(
        sem <= chao * 1.05,
        "com o «Corner Chamfer» a zero a ponta não pode ter faceta a mais do que a estrela SEM chanfro \
         nenhum tem, e ela mede {sem:.5} contra o chão medido de {chao:.5}"
    );
    // ⚠️ **`5×` e não `10×` desde a W144:** o tecto do controlo encolheu `1,39×` ao passar a conter
    // também o recuo do vale, e a faceta a meio dele encolheu na mesma proporção. Medido: `0,02250`
    // contra um chão de `0,00338`, que é `6,7×`.
    assert!(
        com > chao * 5.0,
        "com o «Corner Chamfer» a metade do tecto a ponta tem de ter uma faceta a sério, e ela mede \
         {com:.5} — é o facto de produto que o pedido do Enio de 09/09 comprou"
    );
}
