//! ⭐⭐⭐ **QUANTA ARESTA VIVA SOBRA DEPOIS DO MAIOR FILETE** (W104) — o instrumento que responde
//! ao *«revisar todas»* do Enio (2026-08-29, com duas fotos: as quinas da estrela e as arestas
//! laterais da pirâmide).
//!
//! # ⚠️ Por que ele NÃO percorre uma lista de arestas
//!
//! A pergunta *«o filete alcança todas as arestas desta forma?»* respondida por uma lista escrita à
//! mão só encontra as arestas que **quem a escreveu já conhecia** — e o report do Enio é
//! precisamente sobre arestas que ninguém tinha listado. ⇒ a sonda **acha** as arestas: ela
//! projecta pontos na superfície e mede a **variação da normal** numa vizinhança pequena. Numa
//! superfície lisa (plana ou filetada de raio `r ≫ d`) a normal roda `~2d/r`; numa aresta viva ela
//! **salta** o ângulo diedro, seja qual for `d`.
//!
//! ⇒ a coluna que interessa é *«que fração da superfície está sobre um vinco»*, e ela tem de ir a
//! ~0 quando o filete está no máximo. Uma forma que fique com `20 %` tem arestas que o `round` não
//! alcança — que é exactamente o que as fotos mostram.
//!
//! ```text
//! cargo test -p ph2d-field-eval --release --test measure_sharp_edges -- --ignored --nocapture
//! ```

use fidget::shape::EzShape;
use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, PrimitiveKind, Xform};

/// O ângulo de normal acima do qual dois pontos vizinhos estão sobre um **vinco**.
///
/// ⚠️ Ele é do **instrumento** e derivado do passo: com `d = 0,01` e o filete mínimo interessante
/// (`0,05`), uma superfície filetada roda `2·d/r ≈ 23°`… ⛔ o que colidiria com a barra. Por isso a
/// sonda mede com `d` **muito menor que o filete** (ver [`STEP`]): a `d = 0,004` sobre `r = 0,25` a
/// rotação é `1,8°`, e uma aresta viva de 90° continua a marcar 90°.
const CREASE_DEG: f64 = 25.0;

/// A distância tangencial entre os dois pontos cuja normal se compara.
const STEP: f64 = 0.004;

/// A projecção de Newton na direcção da normal, até o campo ser ~0.
fn project(batch: &mut impl FnMut(&[[f64; 3]]) -> Vec<f64>, pts: &mut [[f64; 3]]) {
    for _ in 0..24 {
        let f = batch(pts);
        let n = normals(batch, pts);
        for (i, q) in pts.iter_mut().enumerate() {
            for a in 0..3 {
                q[a] -= f[i] * n[i][a];
            }
        }
    }
}

/// A normal unitária por diferenças centrais, em lote.
fn normals(batch: &mut impl FnMut(&[[f64; 3]]) -> Vec<f64>, pts: &[[f64; 3]]) -> Vec<[f64; 3]> {
    const H: f64 = 1.0e-4;
    let mut probe = Vec::with_capacity(pts.len() * 6);
    for q in pts {
        for a in 0..3 {
            for s in [-1.0, 1.0] {
                let mut w = *q;
                w[a] += s * H;
                probe.push(w);
            }
        }
    }
    let f = batch(&probe);
    (0..pts.len())
        .map(|i| {
            let g = [
                f[i * 6 + 1] - f[i * 6],
                f[i * 6 + 3] - f[i * 6 + 2],
                f[i * 6 + 5] - f[i * 6 + 4],
            ];
            let l = (g[0] * g[0] + g[1] * g[1] + g[2] * g[2]).sqrt();
            if l > 1.0e-12 {
                [g[0] / l, g[1] / l, g[2] / l]
            } else {
                [0.0, 0.0, 1.0]
            }
        })
        .collect()
}

/// ⭐ **A TRAVESSIA, uma só** — devolve os pontos sobre um vinco (com o ângulo de cada) e quantas
/// amostras chegaram à superfície.
///
/// ⚠️ **A tabela e o gate correm esta MESMA travessia com finuras diferentes**, e a razão é medida:
/// a `4096 × 6` o gate custava **6 s** em debug, e um teste que ficou lento é uma medição de custo
/// que ninguém pediu. A `2048 × 4` a leitura da estrela mexe-se `0,1 pp` — a banda do vinco tem
/// largura suficiente para não depender da densidade.
fn traverse(p: &Primitive, seeds: usize, ring: usize) -> (Vec<([f64; 3], f64)>, usize, f64) {
    let doc = FieldDoc::new(
        vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p.clone()))],
        NodeId(0),
    )
    .expect("a peça");
    let shape = ph2d_field_eval::Engine::from(ph2d_field_eval::compile(&doc));
    let tape = shape.ez_float_slice_tape();
    let mut ev = ph2d_field_eval::Engine::new_float_slice_eval();
    let mut batch = |q: &[[f64; 3]]| -> Vec<f64> {
        let xs: Vec<f32> = q.iter().map(|w| w[0] as f32).collect();
        let ys: Vec<f32> = q.iter().map(|w| w[1] as f32).collect();
        let zs: Vec<f32> = q.iter().map(|w| w[2] as f32).collect();
        ev.eval(&tape, &xs, &ys, &zs)
            .expect("avalia")
            .iter()
            .map(|v| f64::from(*v))
            .collect()
    };

    let r = f64::from(ph2d_field::bounding_radius(p)) * 1.05;
    // ⭐ **DUAS famílias de sementes, e a segunda nasceu de uma mutação que SOBREVIVEU.**
    //
    // A primeira são **direcções**: um ponto por direcção, à distância do bordo, que converge para a
    // silhueta vista dali. ⛔ Ela é **cega ao vinco CÔNCAVO**: de fora, o gradiente aponta para a
    // face mais próxima e **escorrega para longe** do entalhe. Medido — apagar o arredondamento do
    // vale de uma estrela move a leitura de `1,4 %` para `1,3 %`, que é ruído: a mutação sobreviveu
    // a este gate e só morreu no gate analítico do vale.
    //
    // ⇒ a segunda família são pontos de uma **grelha DENTRO da caixa**: de dentro, o gradiente
    // aponta para a superfície mais próxima, e os pontos que caem no cone de influência do entalhe
    // projectam-se **nele**. *Uma sonda que só olha de fora só encontra o que é convexo.*
    let mut pts: Vec<[f64; 3]> = (0..seeds)
        .map(|i| {
            let z = 1.0 - 2.0 * (i as f64 + 0.5) / seeds as f64;
            let s = (1.0 - z * z).max(0.0).sqrt();
            let a = 2.399_963_229_728_653 * i as f64;
            [s * a.cos() * r, s * a.sin() * r, z * r]
        })
        .collect();
    let lado = (seeds as f64).cbrt().round().max(2.0) as i32;
    for i in 0..lado {
        for j in 0..lado {
            for k in 0..lado {
                let c = |v: i32| (f64::from(v) / f64::from(lado - 1)).mul_add(2.0, -1.0) * r * 0.9;
                pts.push([c(i), c(j), c(k)]);
            }
        }
    }
    project(&mut batch, &mut pts);
    let normais = normals(&mut batch, &pts);

    // O anel tangente: `ring` pontos a `STEP` de cada um, projectados de volta.
    let mut anel: Vec<[f64; 3]> = Vec::with_capacity(pts.len() * ring);
    for (q, n) in pts.iter().zip(&normais) {
        let (t1, t2) = tangents(*n);
        for k in 0..ring {
            let a = std::f64::consts::TAU * k as f64 / ring as f64;
            anel.push([
                q[0] + STEP * (t1[0] * a.cos() + t2[0] * a.sin()),
                q[1] + STEP * (t1[1] * a.cos() + t2[1] * a.sin()),
                q[2] + STEP * (t1[2] * a.cos() + t2[2] * a.sin()),
            ]);
        }
    }
    project(&mut batch, &mut anel);
    let anel_n = normals(&mut batch, &anel);
    let dentro = batch(&pts);

    // ⚠️ O pior ângulo é sobre **TODAS** as amostras, e não só sobre as que passam a barra: quando
    // nada a passa, «o pior é zero» seria a leitura errada — o número que interessa ali é *quão
    // perto do vinco a superfície ainda chega*.
    let (mut vincos, mut total, mut pior) = (Vec::new(), 0_usize, 0.0_f64);
    for (i, n) in normais.iter().enumerate() {
        // ⚠️ Só conta o ponto que **chegou** à superfície: uma semente que não convergiu mediria a
        // normal de um sítio qualquer.
        if dentro[i].abs() > 1.0e-3 {
            continue;
        }
        total += 1;
        let mut maior = 0.0_f64;
        for k in 0..ring {
            let v = anel_n[i * ring + k];
            let dot = (n[0] * v[0] + n[1] * v[1] + n[2] * v[2]).clamp(-1.0, 1.0);
            maior = maior.max(dot.acos().to_degrees());
        }
        pior = pior.max(maior);
        if maior > CREASE_DEG {
            vincos.push((pts[i], maior));
        }
    }
    vincos.sort_by(|a, b| b.1.total_cmp(&a.1));
    (vincos, total, pior)
}

/// `(% da superfície sobre um vinco, pior ângulo, amostras que chegaram à superfície)`.
fn probe_with(p: &Primitive, seeds: usize, ring: usize) -> (f64, f64, usize) {
    let (vincos, total, pior) = traverse(p, seeds, ring);
    let frac = if total == 0 {
        f64::NAN
    } else {
        100.0 * vincos.len() as f64 / total as f64
    };
    (frac, pior, total)
}

fn main_probe(p: &Primitive) -> (f64, f64, usize) {
    probe_with(p, 4096, 6)
}

fn crease_points(p: &Primitive) -> Vec<([f64; 3], f64)> {
    traverse(p, 4096, 6).0
}

fn tangents(n: [f64; 3]) -> ([f64; 3], [f64; 3]) {
    let a = if n[2].abs() < 0.9 {
        [0.0, 0.0, 1.0]
    } else {
        [1.0, 0.0, 0.0]
    };
    let cross = |u: [f64; 3], v: [f64; 3]| {
        [
            u[1] * v[2] - u[2] * v[1],
            u[2] * v[0] - u[0] * v[2],
            u[0] * v[1] - u[1] * v[0],
        ]
    };
    let norm = |v: [f64; 3]| {
        let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        [v[0] / l, v[1] / l, v[2] / l]
    };
    let t1 = norm(cross(n, a));
    (t1, norm(cross(n, t1)))
}

/// A mesma lista do censo — uma peça representativa por família.
fn representative(k: PrimitiveKind) -> Option<Primitive> {
    Some(match k {
        PrimitiveKind::Box => Primitive::Box {
            half: [0.4, 0.3, 0.25],
            round: 0.0,
        },
        PrimitiveKind::Sphere => Primitive::Sphere { radius: 0.5 },
        PrimitiveKind::Cylinder => Primitive::Cylinder {
            radius: 0.4,
            half_height: 0.3,
            round: 0.0,
        },
        PrimitiveKind::Torus => Primitive::Torus {
            major: 0.4,
            minor: 0.15,
        },
        PrimitiveKind::Extrude | PrimitiveKind::Revolve => return None,
        PrimitiveKind::Cone => Primitive::Cone {
            bottom: 0.45,
            top: 0.12,
            half_height: 0.35,
            round: 0.0,
        },
        PrimitiveKind::Capsule => Primitive::Capsule {
            radius: 0.25,
            half_height: 0.4,
        },
        PrimitiveKind::Prism => Primitive::Prism {
            sides: 6,
            bottom: 0.45,
            top: 0.18,
            half_height: 0.3,
            round: 0.0,
        },
        PrimitiveKind::Wedge => Primitive::Wedge {
            half: [0.45, 0.3, 0.35],
            round: 0.0,
        },
        PrimitiveKind::TorusArc => Primitive::TorusArc {
            major: 0.4,
            minor: 0.15,
            angle: std::f32::consts::PI * 1.3,
            round: 0.0,
        },
        PrimitiveKind::Star => Primitive::Star {
            points: 5,
            outer: 0.45,
            inner: 0.18,
            half_height: 0.25,
            round: 0.0,
        },
        PrimitiveKind::BoxFrame => Primitive::BoxFrame {
            half: [0.45, 0.35, 0.4],
            thickness: 0.12,
            round: 0.0,
        },
        PrimitiveKind::Ellipsoid => Primitive::Ellipsoid {
            radii: [0.5, 0.2, 0.35],
        },
    })
}

/// A mesma forma com uma fracção do maior filete que o documento aceita — `None` se ela não tem
/// filete.
///
/// ⚠️ **A coluna do MÁXIMO sozinha mede o caso degenerado.** No máximo, o filete de um cone come a
/// tampa inteira e o de um prisma come metade do apótema: o que sobra ali é a forma-limite, não a
/// que o artista vê. A coluna de **metade** é a que representa o uso — e as duas juntas dizem se um
/// resíduo é do desenho ou da fronteira.
fn with_round(p: &Primitive, fracao: f32) -> Option<Primitive> {
    let limite = ph2d_field::round_limit(p)?;
    let mut shape = ph2d_field::NodeShape::Leaf(p.clone());
    ph2d_field::set_shape_radius(&mut shape, 0, limite * fracao).ok()?;
    let ph2d_field::NodeShape::Leaf(q) = shape else {
        return None;
    };
    Some(q)
}

fn with_max_round(p: &Primitive) -> Option<Primitive> {
    with_round(p, 0.999)
}

#[test]
#[ignore]
fn measure_sharp_edges() {
    println!("  forma          | SEM filete      | filete a METADE | filete no MÁXIMO | amostras");
    for k in PrimitiveKind::ALL {
        let Some(p) = representative(k) else {
            continue;
        };
        let (f0, w0, n0) = main_probe(&p);
        match (with_round(&p, 0.5), with_max_round(&p)) {
            (Some(meio), Some(q)) => {
                let (fm, wm, _) = main_probe(&meio);
                let (f1, w1, _) = main_probe(&q);
                let selo = if fm > 0.5 { "  <-- SOBRA" } else { "" };
                println!(
                    "  {:14} | {f0:5.1} % {w0:5.1}° | {fm:5.1} % {wm:5.1}° | {f1:5.1} % {w1:5.1}°                      | {n0:5}{selo}",
                    k.key()
                );
            }
            _ => println!(
                "  {:14} | {f0:5.1} % {w0:5.1}° | {:>13} | {:>13} | {n0:5}",
                k.key(),
                "(sem filete)",
                ""
            ),
        }
    }
}

/// ⭐ **ONDE ficam os vincos que sobram** — em cilíndricas, para se ler contra a forma.
///
/// ⚠️ A tabela de cima diz *quanto* sobra; esta diz **onde**, que é a pergunta seguinte quando o
/// número não chega a zero. O report do Enio (1.ª foto) aponta para os vales junto à tampa, e é
/// exactamente ali que a sonda tem de os pôr — ou a leitura da tabela está a acusar outra coisa.
#[test]
#[ignore]
fn where_the_creases_are() {
    for (nome, p) in [
        (
            "prisma 6 (com taper)",
            Primitive::Prism {
                sides: 6,
                bottom: 0.45,
                top: 0.18,
                half_height: 0.3,
                round: 0.0,
            },
        ),
        (
            "prisma 6 (recto)",
            Primitive::Prism {
                sides: 6,
                bottom: 0.45,
                top: 0.45,
                half_height: 0.3,
                round: 0.0,
            },
        ),
        (
            "estrela 5",
            Primitive::Star {
                points: 5,
                outer: 0.45,
                inner: 0.18,
                half_height: 0.25,
                round: 0.0,
            },
        ),
    ] {
        let meio = with_round(&p, 0.5).expect("tem filete");
        let r = ph2d_field::round_limit(&p).expect("tem limite") * 0.5;
        let pontos = crease_points(&meio);
        println!("  {nome} — filete {r:.4}: {} vincos", pontos.len());
        for (c, ang) in pontos.iter().take(8) {
            let raio = (c[0] * c[0] + c[1] * c[1]).sqrt();
            let theta = c[1].atan2(c[0]).to_degrees();
            println!(
                "    r={raio:.4} θ={theta:+7.1}° z={:+.4} ângulo={ang:.1}°",
                c[2]
            );
        }
    }
}

/// ⭐⭐⭐ **O FILETE ALCANÇA TODA ARESTA DE TODA FORMA** — o gate que o report do Enio comprou.
///
/// # ⚠️ Por que ele mede a SUPERFÍCIE, e não uma lista de arestas
///
/// *«Fillet não funcionava em várias formas criadas. Melhor revisar todas»* (Enio, 2026-08-29, com
/// duas fotos). Uma lista escrita à mão de *«que arestas esta forma tem»* só encontra as que quem a
/// escreveu já conhecia — e o report é precisamente sobre arestas que ninguém tinha listado. Aqui a
/// sonda **acha** as arestas pela variação da normal, e a lista de formas sai de
/// [`PrimitiveKind::ALL`] ⇒ **uma primitiva nova entra sozinha**.
///
/// # A barra, e o resíduo que ela declara
///
/// Com o filete a **metade** do limite (o regime de uso; o máximo é a forma degenerada), a fração
/// de superfície sobre um vinco tem de ficar **abaixo de `2 %`**. Medido hoje: `0,0 %` em nove das
/// dez formas com filete, e **`1,4 %` na estrela** — o vértice de 3 vias onde a quina lateral
/// encontra o aro, o único sítio em que dois filetes se cruzam num ângulo agudo. ⚠️ O ângulo lá é
/// `35°`, e não `90°`: é uma mistura mais **apertada**, não uma aresta viva.
#[test]
fn the_fillet_reaches_every_edge_of_every_shape() {
    let mut com_aresta = 0;
    for k in PrimitiveKind::ALL {
        let Some(p) = representative(k) else {
            continue;
        };
        let (vivo, _, _) = probe_with(&p, 2048, 4);
        if vivo > 5.0 {
            com_aresta += 1;
        }
        let Some(meio) = with_round(&p, 0.5) else {
            // Sem filete: ela não pode ter aresta nenhuma para arredondar.
            assert!(
                vivo < 2.0,
                "«{}» não tem filete e tem {vivo:.1} % de aresta viva — ou ganha o controle, ou a \
                 ausência dele deixou de ser uma decisão",
                k.key()
            );
            continue;
        };
        let (depois, pior, _) = probe_with(&meio, 2048, 4);
        assert!(
            depois < 2.0,
            "«{}»: com o filete a metade do limite ainda há {depois:.1} % da superfície sobre um \
             vinco (pior {pior:.1}°) — o `round` não alcança alguma aresta desta forma",
            k.key()
        );
    }
    // ⛔ **O CONTROLE**: sem ele, uma sonda que devolvesse `0` para tudo passaria a suíte inteira
    // sem medir coisa nenhuma. Estas formas TÊM aresta viva quando o filete é zero.
    assert!(
        com_aresta >= 5,
        "a sonda só viu aresta viva em {com_aresta} formas — ela deixou de ver arestas"
    );
}
