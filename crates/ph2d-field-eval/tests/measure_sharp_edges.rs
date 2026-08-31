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

/// ⭐ **A TRAVESSIA, uma só** — devolve **todos** os pontos que chegaram à superfície, cada um com o
/// maior salto de normal que se mede à volta dele.
///
/// ⚠️ **Todos, e não só os que passam a barra de vinco**, e a diferença nasceu de uma mutação: um
/// gate que pergunte *«qual o pior salto NESTA região»* sobre a lista já filtrada mede o pior
/// **vinco**, e quando a cura funciona a lista fica vazia — o gate passa por não ter olhado. Quem
/// filtra é quem pergunta.
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

    let (mut pontos, mut total, mut pior) = (Vec::new(), 0_usize, 0.0_f64);
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
        pontos.push((pts[i], maior));
    }
    pontos.sort_by(|a, b| b.1.total_cmp(&a.1));
    (pontos, total, pior)
}

/// Os pontos que passam a barra de vinco — o filtro que cada consumidor aplica por si.
fn only_creases(pontos: &[([f64; 3], f64)]) -> Vec<([f64; 3], f64)> {
    pontos
        .iter()
        .filter(|(_, a)| *a > CREASE_DEG)
        .copied()
        .collect()
}

/// `(% da superfície sobre um vinco, pior ângulo, amostras que chegaram à superfície)`.
fn probe_with(p: &Primitive, seeds: usize, ring: usize) -> (f64, f64, usize) {
    let (pontos, total, pior) = traverse(p, seeds, ring);
    let frac = if total == 0 {
        f64::NAN
    } else {
        100.0 * only_creases(&pontos).len() as f64 / total as f64
    };
    (frac, pior, total)
}

fn main_probe(p: &Primitive) -> (f64, f64, usize) {
    probe_with(p, 4096, 6)
}

fn crease_points(p: &Primitive) -> Vec<([f64; 3], f64)> {
    only_creases(&traverse(p, 4096, 6).0)
}

/// ⭐⭐⭐ **A QUEBRA DE CURVATURA** — o que o olho lê como uma LINHA numa superfície lisa.
///
/// # Por que a variação da normal não chega
///
/// A sonda de vinco mede o salto da **normal**: ela acha aresta viva. ⚠️ Um filete circular não tem
/// aresta nenhuma na fronteira da faixa — a normal é contínua ali —, mas a **curvatura** salta de
/// zero (a face plana) para `1/r` (o arco). Isso é `G1` sem ser `G2`, e é exactamente o que produz a
/// banda de Mach que se vê como um risco no sombreado.
///
/// ⇒ esta sonda mede a **segunda diferença** da normal ao longo de uma tangente: `|n(+d) − 2n(0) +
/// n(−d)|/d`, adimensionalizada pelo tamanho da peça. Numa superfície de curvatura contínua ela é
/// pequena em toda parte; numa fronteira de faixa ela **espeta**.
fn curvature_break(p: &Primitive, seeds: usize) -> (f64, f64) {
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
    let mut pts: Vec<[f64; 3]> = (0..seeds)
        .map(|i| {
            let z = 1.0 - 2.0 * (i as f64 + 0.5) / seeds as f64;
            let s = (1.0 - z * z).max(0.0).sqrt();
            let a = 2.399_963_229_728_653 * i as f64;
            [s * a.cos() * r, s * a.sin() * r, z * r]
        })
        .collect();
    project(&mut batch, &mut pts);
    let n0 = normals(&mut batch, &pts);
    // Dois pontos por tangente, dos dois lados, projectados de volta.
    let d = STEP;
    let (mut mais, mut menos) = (Vec::new(), Vec::new());
    for (q, n) in pts.iter().zip(&n0) {
        let (t1, _) = tangents(*n);
        mais.push([q[0] + d * t1[0], q[1] + d * t1[1], q[2] + d * t1[2]]);
        menos.push([q[0] - d * t1[0], q[1] - d * t1[1], q[2] - d * t1[2]]);
    }
    project(&mut batch, &mut mais);
    project(&mut batch, &mut menos);
    let na = normals(&mut batch, &mais);
    let nb = normals(&mut batch, &menos);
    let dentro = batch(&pts);
    let (mut pior, mut soma, mut conta) = (0.0_f64, 0.0_f64, 0_usize);
    for i in 0..pts.len() {
        if dentro[i].abs() > 1.0e-3 {
            continue;
        }
        let mut acc = 0.0;
        for a in 0..3 {
            let s = na[i][a] - 2.0 * n0[i][a] + nb[i][a];
            acc += s * s;
        }
        // Adimensional: a segunda diferença por `d`, vezes o tamanho da peça.
        let v = acc.sqrt() / d * r;
        if v.is_finite() {
            pior = pior.max(v);
            soma += v;
            conta += 1;
        }
    }
    (pior, if conta == 0 { 0.0 } else { soma / conta as f64 })
}

/// ⭐ **ONDE ficam as quebras de curvatura** — a irmã de [`where_the_creases_are`], para a pergunta
/// que a normal não responde.
#[test]
#[ignore]
fn where_the_curvature_breaks_are() {
    let base = Primitive::Star {
        points: 5,
        outer: 0.45,
        inner: 0.18,
        half_height: 0.25,
        round: 0.0,
        chamfer: 0.0,
    };
    let p = with_round(&base, 0.999).expect("tem filete");
    let (pontos, _, _) = traverse(&p, 4096, 6);
    // Reaproveita a travessia só para as posições; a curvatura sai da sonda própria.
    let _ = pontos;
    let mut por_faixa = [(0.0_f64, 0_usize); 5];
    let quebras = curvature_points(&p, 4096);
    for (c, v) in &quebras {
        let raio = (c[0] * c[0] + c[1] * c[1]).sqrt();
        let i = if c[2].abs() > 0.24 {
            0 // tampa
        } else if raio > 0.38 {
            1 // ponta
        } else if raio < 0.22 {
            2 // vale
        } else if c[2].abs() > 0.20 {
            3 // aro
        } else {
            4 // parede
        };
        por_faixa[i].0 = por_faixa[i].0.max(*v);
        por_faixa[i].1 += 1;
    }
    for (i, nome) in ["tampa", "ponta", "vale", "aro", "parede"]
        .iter()
        .enumerate()
    {
        println!(
            "  {nome:8} | pior {:7.2} | {} pontos acima de 1,0",
            por_faixa[i].0, por_faixa[i].1
        );
    }
}

/// Os pontos cuja quebra de curvatura passa de `1,0`.
fn curvature_points(p: &Primitive, seeds: usize) -> Vec<([f64; 3], f64)> {
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
    let mut pts: Vec<[f64; 3]> = (0..seeds)
        .map(|i| {
            let z = 1.0 - 2.0 * (i as f64 + 0.5) / seeds as f64;
            let s = (1.0 - z * z).max(0.0).sqrt();
            let a = 2.399_963_229_728_653 * i as f64;
            [s * a.cos() * r, s * a.sin() * r, z * r]
        })
        .collect();
    project(&mut batch, &mut pts);
    let n0 = normals(&mut batch, &pts);
    let d = STEP;
    let (mut mais, mut menos) = (Vec::new(), Vec::new());
    for (q, n) in pts.iter().zip(&n0) {
        let (t1, _) = tangents(*n);
        mais.push([q[0] + d * t1[0], q[1] + d * t1[1], q[2] + d * t1[2]]);
        menos.push([q[0] - d * t1[0], q[1] - d * t1[1], q[2] - d * t1[2]]);
    }
    project(&mut batch, &mut mais);
    project(&mut batch, &mut menos);
    let na = normals(&mut batch, &mais);
    let nb = normals(&mut batch, &menos);
    let dentro = batch(&pts);
    let mut out = Vec::new();
    for i in 0..pts.len() {
        if dentro[i].abs() > 1.0e-3 {
            continue;
        }
        let mut acc = 0.0;
        for a in 0..3 {
            let s = na[i][a] - 2.0 * n0[i][a] + nb[i][a];
            acc += s * s;
        }
        let v = acc.sqrt() / d * r;
        if v.is_finite() && v > 1.0 {
            out.push((pts[i], v));
        }
    }
    out
}

/// ⭐ **A tabela da quebra de curvatura** — a irmã da tabela de vincos, para a pergunta que ela não
/// responde: *«o que se vê é uma aresta, ou é a fronteira de uma faixa de filete?»*
#[test]
#[ignore]
fn measure_curvature_breaks() {
    println!(
        "  forma          | limite/bordo | no MÁXIMO (pior/média) | ao MESMO filete relativo (14 %)"
    );
    for k in PrimitiveKind::ALL {
        let Some(p) = representative(k) else {
            continue;
        };
        let Some(limite) = ph2d_field::round_limit(&p) else {
            continue;
        };
        let bordo = ph2d_field::bounding_radius(&p);
        let Some(q) = with_round(&p, 0.999) else {
            continue;
        };
        let (pior, media) = curvature_break(&q, 4096);
        // ⭐ **A coluna que compara maçãs com maçãs**: o mesmo filete em fração do BORDO da peça.
        // Sem ela, uma forma cujo `round_limit` é 14 % do tamanho é comparada com outra cujo limite
        // é 62 % — e o que se lê é a largura da faixa, não a qualidade da mistura.
        let alvo = bordo * 0.14;
        let mesmo = if alvo < limite {
            let mut shape = ph2d_field::NodeShape::Leaf(p.clone());
            ph2d_field::set_shape_radius(&mut shape, 0, alvo).ok();
            let ph2d_field::NodeShape::Leaf(r) = shape else {
                continue;
            };
            let (a, b) = curvature_break(&r, 4096);
            format!("{a:8.2} / {b:6.2}")
        } else {
            format!("{:>17}", "(o limite é menor)")
        };
        println!(
            "  {:14} | {:11.1} % | {pior:8.2} / {media:6.2} | {mesmo}",
            k.key(),
            100.0 * limite / bordo
        );
    }
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
            chamfer: 0.0,
        },
        PrimitiveKind::Sphere => Primitive::Sphere { radius: 0.5 },
        PrimitiveKind::Cylinder => Primitive::Cylinder {
            radius: 0.4,
            half_height: 0.3,
            round: 0.0,
            chamfer: 0.0,
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
            chamfer: 0.0,
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
            chamfer: 0.0,
        },
        PrimitiveKind::Wedge => Primitive::Wedge {
            half: [0.45, 0.3, 0.35],
            round: 0.0,
            chamfer: 0.0,
        },
        PrimitiveKind::TorusArc => Primitive::TorusArc {
            major: 0.4,
            minor: 0.15,
            angle: std::f32::consts::PI * 1.3,
            round: 0.0,
            chamfer: 0.0,
        },
        PrimitiveKind::Star => Primitive::Star {
            points: 5,
            outer: 0.45,
            inner: 0.18,
            half_height: 0.25,
            round: 0.0,
            chamfer: 0.0,
        },
        PrimitiveKind::BoxFrame => Primitive::BoxFrame {
            half: [0.45, 0.35, 0.4],
            thickness: 0.12,
            round: 0.0,
            chamfer: 0.0,
        },
        PrimitiveKind::Ellipsoid => Primitive::Ellipsoid {
            radii: [0.5, 0.2, 0.35],
        },
        // ─────────────────────────── W106 ───────────────────────────
        // ⚠️ **Nascem com `round: 0,0`**, como todas as desta sonda: a coluna «SEM filete» é a
        // referência contra a qual as outras duas se leem, e uma peça que já chegasse arredondada
        // mediria a diferença errada.
        PrimitiveKind::Octahedron => Primitive::Octahedron {
            radius: 0.45,
            round: 0.0,
            chamfer: 0.0,
        },
        PrimitiveKind::RoundCone => Primitive::RoundCone {
            bottom: 0.35,
            top: 0.14,
            half_height: 0.3,
        },
        PrimitiveKind::CutSphere => Primitive::CutSphere {
            radius: 0.45,
            cut: 0.15,
            round: 0.0,
            chamfer: 0.0,
        },
        PrimitiveKind::HollowDome => Primitive::HollowDome {
            radius: 0.45,
            cut: 0.1,
            thickness: 0.1,
            round: 0.0,
            chamfer: 0.0,
        },
        PrimitiveKind::Link => Primitive::Link {
            major: 0.3,
            minor: 0.1,
            length: 0.25,
        },
        PrimitiveKind::SolidAngle => Primitive::SolidAngle {
            radius: 0.45,
            angle: 0.7,
            round: 0.0,
            chamfer: 0.0,
        },
        PrimitiveKind::Gear => Primitive::Gear {
            teeth: 7,
            root: 0.32,
            outer: 0.45,
            tooth: 0.45,
            half_height: 0.15,
            round: 0.0,
            chamfer: 0.0,
        },
        PrimitiveKind::Cross => Primitive::Cross {
            arm: 0.45,
            width: 0.14,
            half_height: 0.12,
            round: 0.0,
            chamfer: 0.0,
        },
        PrimitiveKind::Heart => Primitive::Heart {
            size: 0.3,
            half_height: 0.12,
            round: 0.0,
            chamfer: 0.0,
        },
        PrimitiveKind::Moon => Primitive::Moon {
            radius: 0.45,
            bite: 0.4,
            offset: 0.2,
            half_height: 0.12,
            round: 0.0,
            chamfer: 0.0,
        },
        PrimitiveKind::Drop => Primitive::Drop {
            radius: 0.22,
            height: 0.55,
            half_height: 0.12,
            round: 0.0,
            chamfer: 0.0,
        },
        PrimitiveKind::Pie => Primitive::Pie {
            radius: 0.45,
            angle: 1.0,
            half_height: 0.12,
            round: 0.0,
            chamfer: 0.0,
        },
        PrimitiveKind::Trapezoid => Primitive::Trapezoid {
            bottom: 0.45,
            top: 0.2,
            half_width: 0.3,
            half_height: 0.12,
            round: 0.0,
            chamfer: 0.0,
        },
        PrimitiveKind::Vesica => Primitive::Vesica {
            radius: 0.45,
            offset: 0.25,
            half_height: 0.12,
            round: 0.0,
            chamfer: 0.0,
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
                chamfer: 0.0,
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
                chamfer: 0.0,
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
                chamfer: 0.0,
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
/// ⛔⛔ **AS TRÊS QUE TÊM UM PONTO, E UM PONTO NÃO É UMA ARESTA** (W106) — a tolerância declarada,
/// com o número de cada uma e o mecanismo.
///
/// # Porque elas não passam, e porque isso não é um defeito por curar
///
/// As três têm um **ápice**: um vértice cónico onde a superfície encontra a si própria. É o que um
/// cone de gelado, uma fatia de tarte e uma gota **são** — tirá-lo dava outra forma.
///
/// ⭐⭐ **A medição prova que é o ápice, e não uma aresta órfã:** afiar a abertura do ângulo sólido
/// piora o número de forma monótona, exactamente como a nitidez do ponto:
///
/// | abertura | % da superfície em vinco | pior ângulo |
/// |---|---:|---:|
/// | `0,3 rad` | **27,5 %** | 83,8° |
/// | `0,7 rad` (o representante) | **10,9 %** | 63,2° |
///
/// ⚠️ **E o CONE FECHADO (`top = 0`) lê `0,0 %`** — medido, como controlo. Ele também tem ápice, e
/// escapa porque o dele cai **em cima da laje**: a interseção arredondada do
/// [`crate::ops::slab_and_walls`] apanha-o de caminho. O do ângulo sólido está na origem, onde não
/// há segunda superfície com que o intersectar. ⇒ *não é que um ápice não se arredonde — é que se
/// arredonda quando há algo com que o cruzar, e aqui não há.*
///
/// ⛔ **A cura conhecida não existe neste vocabulário:** dilatar o semiespaço do cone é **inerte**
/// (a lei da W104), e o `offset` dele não é um arco porque `ρ·cos θ − z·sin θ` **subestima** perto
/// do ápice (para um ponto no eixo abaixo dele, ela dá `−z·sin θ` onde a distância é `|z|`).
///
/// # ⚠️ A catraca, e a metade que a impede de virar LICENÇA
///
/// Esta lista **só encolhe**. E o `CLAUDE.md` §5.0 é explícito: *uma catraca sem censo de
/// obsolescência não desce, vira licença* — por isso o gate irmão
/// [`the_apex_exception_list_has_no_stale_entries`] pergunta, a cada corrida, se cada entrada
/// **ainda** estoura a barra. Uma que deixe de estourar tem de ser **apagada**.
const APEX_EXCEPTION: [(&str, f64); 3] = [
    // ⚠️ **As folgas saem do que o GATE mede**, e não da tabela da sonda: ela amostra `8192`
    // pontos e o gate `2048×4`, e a `pie` lê `1,1 %` numa escala e `2,57 %` na outra.
    // *Uma folga calibrada no instrumento errado descreve outra coisa.*
    ("solid_angle", 11.0),
    ("drop", 2.3),
    ("pie", 2.8),
];

/// A folga desta forma, se ela for uma das do ápice.
fn apex_slack(key: &str) -> Option<f64> {
    APEX_EXCEPTION
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| *v)
}

/// ⛔⛔ **A METADE QUE IMPEDE A CATRACA DE SUBIR:** cada entrada do [`APEX_EXCEPTION`] tem de
/// **ainda** estourar a barra normal, e tem de ficar **abaixo** da folga que declara.
///
/// ⚠️ Sem isto, uma entrada cuja causa foi curada fica lá para sempre a autorizar uma aresta viva
/// que já não existe — e a próxima forma com o mesmo nome herda a licença. Medido nesta casa em
/// 2026-08-30: a lista de folgas de LOC por ficheiro não tinha censo, e acusou **três** entradas
/// obsoletas na primeira corrida que o teve.
#[test]
fn the_apex_exception_list_has_no_stale_entries() {
    for (nome, folga) in APEX_EXCEPTION {
        let k = PrimitiveKind::ALL
            .iter()
            .find(|k| k.key() == nome)
            .unwrap_or_else(|| panic!("«{nome}» já não é uma forma — a entrada ficou órfã"));
        let p = representative(*k).unwrap_or_else(|| panic!("«{nome}» não tem representante"));
        let meio = with_round(&p, 0.5).unwrap_or_else(|| {
            panic!("«{nome}» deixou de ter filete — a entrada não descreve nada")
        });
        let (depois, _, _) = probe_with(&meio, 2048, 4);
        println!("  [apex] {nome}: {depois:.2} % (folga declarada {folga:.2} %)");
        assert!(
            depois >= 2.0,
            "«{nome}» já cumpre a barra normal ({depois:.1} % < 2 %) — APAGUE a entrada dele da              lista, senão ela vira licença para a próxima forma"
        );
        assert!(
            depois < folga,
            "«{nome}» piorou para {depois:.1} %, acima da folga declarada de {folga:.1} % — a              catraca SÓ ENCOLHE"
        );
    }
}

/// A mesma forma com o **chanfro** posto a `fracao` da parede, e o filete a `fillet` dela.
fn with_pair(p: &Primitive, chamfer: f32, fillet: f32) -> Option<Primitive> {
    let limite = ph2d_field::round_limit(p)?;
    let mut q = p.clone();
    for (chave, f) in [("field.dim.chamfer", chamfer), ("field.dim.round", fillet)] {
        let i = ph2d_field::dims(&q).iter().position(|d| d.key == chave)?;
        ph2d_field::set_dim(&mut q, 0, i, limite * f).ok()?;
    }
    Some(q)
}

/// ⭐⭐⭐ **O CHANFRO ALCANÇA TODA ARESTA DE TODA FORMA** — a irmã do
/// [`the_fillet_reaches_every_edge_of_every_shape`], e a régua é OUTRA.
///
/// # ⛔ Ele nasceu de um report do Enio (2026-08-30)
///
/// *«com um prisma veja que algumas arestas não receberam o fillet»*. As quinas **laterais** de um
/// prisma fecham num sítio do código e o **aro** noutro; o chanfro tinha sido ligado só ao segundo.
///
/// # ⚠️ «Sem vinco» é a régua ERRADA para um chanfro, e medi-lo mostrou-o
///
/// Um filete apaga a aresta; um chanfro **troca-a por duas**, de 135° cada. A fracção de superfície
/// sobre um vinco por isso **não** cai a zero, e uma barra copiada da irmã reprovaria produto
/// correcto.
///
/// ⛔ **E a fracção GLOBAL também não serve**: com o defeito o prisma cortava `22 %` do vinco e
/// curado corta `62 %`, mas há formas legítimas em `36–40 %` (cúpula, ângulo sólido, engrenagem) —
/// *um limiar entre `22` e `36` esgota-se na primeira forma nova.*
///
/// ⭐ A régua que separa é **por PONTO**: pega-se em cada ponto de vinco da forma **viva** e
/// pergunta-se se o chanfro o **cortou** (o campo chanfrado é positivo ali). Uma aresta esquecida
/// deixa os pontos dela dentro, e a contagem cai — seja qual for o tamanho dela.
#[test]
fn the_chamfer_reaches_every_edge_of_every_shape() {
    let mut fracos = Vec::new();
    let mut testadas = 0;
    for k in PrimitiveKind::ALL {
        let Some(p) = representative(k) else { continue };
        let Some(pct) = fraccao_cortada(&p) else {
            continue;
        };
        testadas += 1;
        println!("  [chanfro] {}: {pct:.1} % dos vincos cortados", k.key());
        // ⚠️ Uma forma com ÁPICE declarado tem piso próprio — ver [`CHANFRO_APICE`] e o censo dele.
        let barra = chanfro_apice(k.key()).unwrap_or(BARRA_DO_CHANFRO);
        if pct < barra {
            fracos.push(format!("{} {pct:.1} %", k.key()));
        }
    }
    assert!(
        testadas >= 15,
        "a sonda só achou vinco em {testadas} formas — ela deixou de ver arestas"
    );
    assert!(
        fracos.is_empty(),
        "nestas formas o chanfro deixou arestas por cortar: {fracos:?} — é o report do Enio de \
         2026-08-30 (o prisma tinha as quinas LATERAIS fechadas noutro sítio do código)"
    );
}

/// A fracção dos pontos de vinco que um chanfro a meia parede tem de cortar.
///
/// ⚠️ **MEDIDO, não escolhido**: dezanove das vinte formas ficam entre `92,8 %` e `100 %`, e o
/// defeito que o Enio viu lia `22 %` (prisma) e `79,9 %` (engrenagem). Ver o gate acima para as duas
/// réguas que foram medidas e **recusadas** antes desta.
const BARRA_DO_CHANFRO: f64 = 90.0;

/// ⛔ **As formas cujo vinco NÃO é uma junta** — o ápice de um cone não é uma aresta entre duas
/// peças, é a degenerescência da própria peça, e nenhum chanfro do aro lhe chega.
///
/// ⚠️ **A lista é a mesma do [`APEX_EXCEPTION`] na causa e OUTRA na grandeza** (aquela mede vinco
/// que sobra, esta mede vinco cortado), então os números não se copiam de uma para a outra —
/// *uma folga calibrada no instrumento errado descreve outra coisa*. Só o `solid_angle` precisa
/// dela: a `drop` lê `99,5 %` e a `pie` `99,6 %`, e as duas passam a barra normal.
const CHANFRO_APICE: [(&str, f64); 1] = [("solid_angle", 35.0)];

/// O piso desta forma, se ela for uma das do ápice.
fn chanfro_apice(key: &str) -> Option<f64> {
    CHANFRO_APICE
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| *v)
}

/// ⛔⛔ **A METADE QUE IMPEDE A CATRACA DE SUBIR** — cada entrada do [`CHANFRO_APICE`] tem de
/// **ainda** estar abaixo da barra normal (senão a causa foi curada e a entrada virou licença) e
/// **acima** do piso que declara (senão ela regrediu).
///
/// ⚠️ É a lei que esta casa mediu em 2026-08-30: *uma catraca sem censo de obsolescência não desce,
/// ela vira licença* — a lista de folgas de LOC por ficheiro não o tinha e acusou **três** entradas
/// obsoletas na primeira corrida que o teve.
#[test]
fn the_chamfer_apex_list_has_no_stale_entries() {
    for (nome, piso) in CHANFRO_APICE {
        let k = PrimitiveKind::ALL
            .iter()
            .find(|k| k.key() == nome)
            .unwrap_or_else(|| panic!("«{nome}» já não é uma forma — a entrada ficou órfã"));
        let p = representative(*k).unwrap_or_else(|| panic!("«{nome}» não tem representante"));
        let pct = fraccao_cortada(&p).unwrap_or_else(|| {
            panic!("«{nome}» deixou de ter chanfro — a entrada não descreve nada")
        });
        println!("  [apice-chanfro] {nome}: {pct:.1} % (piso declarado {piso:.1} %)");
        assert!(
            pct < BARRA_DO_CHANFRO,
            "«{nome}» já cumpre a barra normal ({pct:.1} %) — APAGUE a entrada, senão ela vira \
             licença para a próxima forma"
        );
        assert!(
            pct >= piso,
            "«{nome}» regrediu para {pct:.1} %, abaixo do piso declarado de {piso:.1} %"
        );
    }
}

/// A fracção dos pontos de vinco da forma VIVA que o chanfro a meia parede corta.
fn fraccao_cortada(p: &Primitive) -> Option<f64> {
    let chanfrada = with_pair(p, 0.5, 0.0)?;
    let (pontos, _, _) = traverse(p, 2048, 4);
    let vincos = only_creases(&pontos);
    if vincos.len() < 20 {
        return None;
    }
    let doc = FieldDoc::new(
        vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(chanfrada))],
        NodeId(0),
    )
    .ok()?;
    let f = ph2d_field_eval::Field::new(&doc);
    let cortados = vincos
        .iter()
        .filter(|(q, _)| f.at(q[0], q[1], q[2]) > 1.0e-4)
        .count();
    Some(100.0 * cortados as f64 / vincos.len() as f64)
}

/// **SONDA** — quanto vinco sobra em cada forma com o par ligado, a varrer as fracções.
#[test]
#[ignore = "sonda: imprime a tabela que escolhe a barra do gate irmao"]
fn measure_the_pair_over_every_shape() {
    println!("\n  forma            |  vivo  | so' chanfro | c=.5 r=.2 | c=.4 r=.4 | c=.3 r=.5");
    for k in PrimitiveKind::ALL {
        let Some(p) = representative(k) else { continue };
        if ph2d_field::round_limit(&p).is_none() {
            continue;
        }
        let (vivo, _, _) = probe_with(&p, 1024, 4);
        let f =
            |c: f32, r: f32| with_pair(&p, c, r).map_or(f64::NAN, |q| probe_with(&q, 1024, 4).0);
        println!(
            "  {:<16} | {vivo:>6.2} | {:>11.2} | {:>9.2} | {:>9.2} | {:>9.2}",
            k.key(),
            f(0.5, 0.0),
            f(0.5, 0.2),
            f(0.4, 0.4),
            f(0.3, 0.5)
        );
    }
}

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
        // ⚠️ A barra é `2 %` para toda forma, **menos** as que têm um ápice declarado — ver
        // [`APEX_EXCEPTION`], e o censo de obsolescência que a impede de virar licença.
        let barra = apex_slack(k.key()).unwrap_or(2.0);
        assert!(
            depois < barra,
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

/// ⭐⭐⭐ **O VALE DA ESTRELA ENCONTRA A TAMPA SEM VINCO** — o gate do 2.º smoke do Enio
/// (*«apenas a estrela tem resultado ruim»*, com meias-luas nos cinco vales).
///
/// # ⛔ Por que ele é DIRIGIDO, e não mais uma barra global
///
/// A cura — dar **folga** aos planos do sector, para que eles deixem de passar **pelo vale**, que é
/// um ponto da superfície — **suaviza** sem mudar quantos pontos passam a barra de vinco: a fração
/// fica em `1,0 %` com e sem ela, e **duas mutações sobreviveram** ao gate de contagem. *Uma barra de
/// contagem mede quantos sítios estão maus, e nunca quão mau é o pior.*
///
/// ⚠️ E uma barra global do **pior ângulo** não serve: ela é sensível à densidade da amostragem (o
/// cone lê `11,8°` na tabela e `23,9°` na finura do gate) e, no filete máximo, o cone é uma forma
/// **degenerada** — o filete come a tampa inteira e deixa um polo. ⇒ o gate aponta ao sítio que o
/// report nomeia, mede o **pior salto de normal** ali, e diz o mecanismo no nome.
///
/// Medido: **`13,6°`** com a folga, **`35,0°`** sem ela.
#[test]
fn the_valley_of_a_star_meets_the_cap_without_a_crease() {
    let (outer, inner, meia_altura) = (0.45_f64, 0.18, 0.25);
    let base = Primitive::Star {
        points: 5,
        outer: outer as f32,
        inner: inner as f32,
        half_height: meia_altura as f32,
        round: 0.0,
        chamfer: 0.0,
    };
    let p = with_round(&base, 0.999).expect("a estrela tem filete");
    let (pontos, _, _) = traverse(&p, 2048, 6);
    // O vértice de 3 vias: o vale, à altura da tampa. São cinco, e a simetria di-lo — basta medir a
    // vizinhança de **todos** eles de uma vez, por raio e altura.
    let beta = std::f64::consts::PI / 5.0;
    let (mut pior_no_vale, mut na_regiao) = (0.0_f64, 0_usize);
    for (c, ang) in &pontos {
        let raio = (c[0] * c[0] + c[1] * c[1]).sqrt();
        // A meia-lua vivia entre o raio do vale e um pouco além dele, junto à tampa.
        if raio < inner * 1.6 && c[2].abs() > meia_altura * 0.7 {
            na_regiao += 1;
            pior_no_vale = pior_no_vale.max(*ang);
        }
    }
    let _ = beta;
    // ⛔ **O CONTROLE, e ele é o que impede o gate de passar por não ter olhado.** Um balde que
    // ninguém enche lê-se como perfeito: se a amostragem deixar de cair na região do vale,
    // `pior_no_vale` fica em zero e a afirmação abaixo passa **sem medir nada**.
    assert!(
        na_regiao >= 20,
        "a sonda pôs {na_regiao} amostras na região do vale — poucas para afirmar o que se segue"
    );
    assert!(
        pior_no_vale < 20.0,
        "no encontro do vale com a tampa a normal salta {pior_no_vale:.1}° — as duas pipas chegam \
         ao vale cada uma com o vinco do seu plano de sector, e a união funde os dois vincos"
    );
}

/// ⭐⭐⭐ **O FILETE NÃO DEIXA UMA CRISTA DE CURVATURA** — o gate do 3.º smoke do Enio
/// (*«quase perfeito»*, com a seta numa ponta da estrela).
///
/// # ⚠️ Por que a barra de VINCO não bastava
///
/// A sonda de vinco mede o salto da **normal**: ela acha aresta viva. Um filete demasiado
/// **apertado** não tem aresta nenhuma — a normal é contínua —, mas a **curvatura** dispara, e é
/// isso que o olho lê como um risco no sombreado. A estrela lia `0,0 %` de vinco e a foto do Enio
/// mostrava a linha na ponta.
///
/// ⇒ a régua é a **segunda diferença da normal**, adimensionalizada pelo tamanho da peça, e é a
/// **média** que separa: uma crista larga levanta a média, um pico isolado não.
///
/// Medido no filete máximo: `0,04`–`0,47` em sete formas, e a estrela em **`1,19`** (era **`3,71`**
/// antes de a ponta dela ser compensada). A barra é `2,0` — abaixo do defeito curado com `1,9×` de
/// folga, e acima de toda forma boa com `4×`.
/// ⛔⛔ **A JUNÇÃO TANGENTE: lisa ao olho, DESCONTÍNUA na curvatura** (W106).
///
/// # O mecanismo, e porque ele não é um defeito por curar
///
/// A gota é uma bolha unida a duas **tangentes**. Uma junção tangente é **G1** — a normal é
/// contínua, e é por isso que a silhueta não tem quina — mas **não é G2**: a curvatura salta de
/// `1/r` (o círculo) para `0` (a recta), de um lado ao outro do ponto de tangência. ⭐ Esta régua
/// mede exactamente esse salto, e é o problema clássico de continuidade que todo CAD tem.
///
/// # ⛔⛔ E a cura óbvia foi MEDIDA e REJEITADA
///
/// A nota do código dizia *«arredondar ali abriria um sulco onde não há aresta»* — sem número. O
/// A/B (`min` cru contra [`crate::ops::union`] com [`crate::ops::Blended::Exact`]) diz que ela é
/// **pior ou igual em toda a faixa**:
///
/// | fracção do filete | `min` cru | união arredondada |
/// |---|---:|---:|
/// | 0,10 | **4,46** | 5,26 |
/// | 0,20 | **3,12** | 3,73 |
/// | 0,30 | **3,34** | 3,58 |
/// | 0,50 | **3,46** | 3,51 |
/// | 0,999 | **4,46** | 4,34 |
///
/// ⚠️ **E não é o teto do filete:** a sonda [`measure_drop_round_limit`] varreu de `0,1` a `0,999`
/// do limite e a quebra fica entre `3,12` e `4,46` em **toda** a faixa. *Um defeito que não se move
/// com o knob não é uma calibração.*
///
/// ⚠️ **A catraca SÓ ENCOLHE**, e o censo abaixo impede-a de virar licença.
const TANGENT_JOIN_EXCEPTION: [(&str, f64); 1] = [("drop", 4.6)];

fn tangent_join_slack(key: &str) -> Option<f64> {
    TANGENT_JOIN_EXCEPTION
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| *v)
}

/// ⛔⛔ **O censo de obsolescência da lista acima** — o irmão do
/// [`the_apex_exception_list_has_no_stale_entries`], e pela mesma razão.
#[test]
fn the_tangent_join_exception_list_has_no_stale_entries() {
    for (nome, folga) in TANGENT_JOIN_EXCEPTION {
        let k = PrimitiveKind::ALL
            .iter()
            .find(|k| k.key() == nome)
            .unwrap_or_else(|| panic!("«{nome}» já não é uma forma — a entrada ficou órfã"));
        let p = representative(*k).unwrap_or_else(|| panic!("«{nome}» não tem representante"));
        let q = with_max_round(&p).unwrap_or_else(|| {
            panic!("«{nome}» deixou de ter filete — a entrada não descreve nada")
        });
        let (_, media) = curvature_break(&q, 2048);
        println!("  [tangente] {nome}: {media:.2} (folga {folga:.2})");
        assert!(
            media >= 2.0,
            "«{nome}» já cumpre a barra normal ({media:.2} < 2,0) — APAGUE a entrada dele"
        );
        assert!(
            media < folga,
            "«{nome}» piorou para {media:.2}, acima da folga de {folga:.2} — a catraca SÓ ENCOLHE"
        );
    }
}

/// ⭐ **SONDA: onde é que o filete da GOTA deixa de ser um filete?**
///
/// ⚠️ Ela é a única forma cuja quebra de curvatura **PIORA** com mais filete (`3,46` a metade,
/// `4,46` no máximo), e isso é o sintoma de um limite generoso demais: acima de certo raio o
/// arredondamento deixa de acertar numa aresta e passa a **reformar a bolha**.
///
/// ⇒ o teto sai daqui, e não de um número escolhido.
#[test]
#[ignore = "sonda: escolhe o teto de filete da gota"]
fn measure_drop_round_limit() {
    let base = Primitive::Drop {
        radius: 0.22,
        height: 0.55,
        half_height: 0.12,
        round: 0.0,
        chamfer: 0.0,
    };
    let limite = ph2d_field::round_limit(&base).expect("a gota tem filete");
    println!(
        "\n  teto actual = {limite:.4}\n{:>10} {:>10} {:>14}",
        "fracao", "raio", "quebra media"
    );
    for f in [0.1_f32, 0.2, 0.3, 0.4, 0.5, 0.7, 0.999] {
        let Some(q) = with_round(&base, f) else {
            continue;
        };
        let (_, media) = curvature_break(&q, 2048);
        println!("{f:>10.3} {:>10.4} {media:>14.2}", limite * f);
    }
    println!();
}

#[test]
fn the_fillet_leaves_no_curvature_ridge() {
    let (mut medidas, mut maior) = (0, 0.0_f64);
    for k in PrimitiveKind::ALL {
        let Some(p) = representative(k) else {
            continue;
        };
        let Some(q) = with_round(&p, 0.999) else {
            continue;
        };
        let (_, media) = curvature_break(&q, 2048);
        maior = maior.max(media);
        // ⚠️ A barra é `2,0` para toda forma, **menos** a que tem uma junção TANGENTE declarada —
        // ver [`TANGENT_JOIN_EXCEPTION`], e o censo que a impede de virar licença.
        let barra = tangent_join_slack(k.key()).unwrap_or(2.0);
        assert!(
            media < barra,
            "«{}»: a quebra de curvatura média é {media:.2} — o filete deixa uma crista, e ela \
             vê-se como um risco no sombreado mesmo sem aresta nenhuma",
            k.key()
        );
        medidas += 1;
    }
    // ⛔ **O CONTROLE**: sem ele, um dia em que nenhuma forma tenha filete o gate passa vazio.
    assert!(
        medidas >= 6,
        "só {medidas} formas com filete foram medidas — o gate perdeu o sujeito"
    );
    // ⛔ **E o segundo CONTROLE: a sonda tem de estar a MEDIR alguma coisa.** Uma que devolvesse
    // sempre zero passaria a barra em toda a linha sem olhar para nada — e foi uma mutação a
    // substituí-la por `0,0` que mostrou o buraco. Um filete real deixa sempre alguma curvatura.
    assert!(
        maior > 0.1,
        "a maior quebra de curvatura medida foi {maior:.3} — a sonda deixou de medir"
    );
}

/// ⭐⭐⭐ **O CHANFRO NÃO PODE PIORAR UMA ARESTA** — o 3.º report do Enio sobre esta feature
/// (2026-08-30, com foto): *«algumas arestas não arredondam no prisma»*.
///
/// # A régua, e por que ela é uma RAZÃO
///
/// Pedir *«o filete arredonda»* em graus absolutos não funciona: uma ponta de estrela e uma quina de
/// caixa começam em sítios diferentes, e uma barra única ou branqueia a caixa ou reprova a estrela.
/// O que o artista diz é **«com chanfro fica pior do que sem»** — e isso é a razão entre o pior giro
/// da normal com os dois recuos e o **mesmo filete** sozinho.
///
/// ⚠️ **A régua é o GIRO DA NORMAL, e não o volume** — um chanfro deslocado tira o mesmo volume que
/// um arredondado. É a mesma lição que o 2.º report já tinha cobrado.
///
/// # ⛔⛔ A 1.ª versão media no ponto DEGENERADO, e a lista de toleradas era um artefacto disso
///
/// Ela punha `filete = chanfro`, que é onde o filete já consumiu a faceta inteira do chanfro e não
/// há arestas distintas para arredondar. Onze formas apareciam «pioradas» e **a maior parte era a
/// geometria do pedido, não um defeito**. A varredura da razão mostra-o:
///
/// | | `r=0,25c` | `r=0,5c` | `r=0,75c` | `r=c` |
/// |---|---:|---:|---:|---:|
/// | bando (as outras 18) | `0,92`–`3,37` | **`0,90`–`2,40`** | `0,92`–`2,36` | `0,85`–`2,41` |
/// | as 20, **depois** | `0,92`–`3,37` | **`0,90`–`2,40`** | `0,92`–`2,36` | `0,85`–`5,33` |
/// | cruz, **antes** | `16,16` | `15,95` | `11,18` | `9,31` |
/// | engrenagem, **antes** | `3,83` | `3,78` | `2,72` | `2,27` |
///
/// ⇒ a medição vale em **`r = 0,5c`**, e ali os dois destoantes eram estruturais (perfis feitos por
/// UNIÃO, cuja composta entrava inteira na mistura do aro) — curados, o catálogo cabe todo no bando
/// e a lista de toleradas ficou **VAZIA**.
///
/// # ⚠️ As duas barras, e de que elas são
///
/// `2,60` é o máximo MEDIDO do bando (`2,40`, o **ápice do cone** — uma feição que já mede `16,1°`
/// só com filete) mais `8 %`. `6,00` é o máximo medido na **saturação** (`5,33`, o arco de toro com
/// `r = c`) mais `13 %`. ⚠️ São barras de **corpus sobre uma lista FECHADA** (`PrimitiveKind::ALL`):
/// é exactamente o caso em que isso é a coisa certa — uma primitiva nova que as estoure é o que se
/// quer ver acusado.
#[test]
fn the_chamfer_never_makes_an_edge_worse_than_the_fillet_alone() {
    /// A folga sobre o filete sozinho, no ponto de trabalho `r = 0,5c`.
    const BARRA: f64 = 2.60;
    /// A mesma pergunta na SATURAÇÃO (`r = c`), onde o filete já comeu a faceta do chanfro.
    const BARRA_SATURADA: f64 = 6.00;
    /// Abaixo disto o giro é ruído de amostragem e a razão deixa de significar alguma coisa.
    const PISO_GRAUS: f64 = 3.0;
    let mut piores = Vec::new();
    let mut medidas = 0;
    for k in PrimitiveKind::ALL {
        let Some(base) = representative(k) else {
            continue;
        };
        let Some(limite) = ph2d_field::round_limit(&base) else {
            continue;
        };
        let c = limite * 0.5;
        let escreve = |p: &Primitive, chave: &str, v: f32| -> Option<Primitive> {
            let mut p = p.clone();
            let i = ph2d_field::dims(&p).iter().position(|d| d.key == chave)?;
            ph2d_field::set_dim(&mut p, 0, i, v).ok()?;
            Some(p)
        };
        let pior = |p: &Primitive| {
            traverse(p, 2048, 6)
                .0
                .iter()
                .map(|(_, a)| *a)
                .fold(0.0f64, f64::max)
        };
        let Some(so_filete) = escreve(&base, "field.dim.round", c) else {
            continue;
        };
        medidas += 1;
        let base_graus = pior(&so_filete);
        for (fracao, barra, onde) in [
            (0.5f32, BARRA, "no ponto de trabalho"),
            (1.0, BARRA_SATURADA, "na saturação"),
        ] {
            let Some(par) = escreve(&base, "field.dim.round", c * fracao)
                .and_then(|p| escreve(&p, "field.dim.chamfer", c))
            else {
                continue;
            };
            let b = pior(&par);
            let razao = b / base_graus.max(1.0e-9);
            if b > PISO_GRAUS && razao > barra {
                piores.push(format!(
                    "{k:?} {onde}: {base_graus:.1}° só com filete e {b:.1}° com chanfro \
                     ({razao:.2}x, barra {barra:.2}x)"
                ));
            }
        }
    }
    assert!(
        medidas >= 20,
        "só {medidas} formas com aresta — a lista derivada de `PrimitiveKind::ALL` partiu-se"
    );
    assert!(
        piores.is_empty(),
        "o chanfro piorou arestas que ele não pode piorar: {piores:?}"
    );
}
