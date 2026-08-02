//! Gates da subdivisão.
//!
//! ⚠️ **O oráculo mais forte desta suíte é o CANAL CONSTANTE**, e vale a pena
//! dizer por quê: uma tabela de pesos afim preserva um campo constante, e um
//! campo constante só é preservado se os pesos somam 1. Um `0,375` digitado
//! `0,357` em QUALQUER dos quatro ramos (Loop, Catmull-Clark, fronteira
//! tri/quad, borda) aparece ali — sem que o gate precise saber qual ramo cada
//! vértice tomou.

use super::*;
use crate::{shapes, shapes_open};

/// A promessa aritmética: `F × 4` faces e `V + E + Q` vértices.
#[test]
fn the_counts_are_the_arithmetic_the_module_promises() {
    for mesh in [
        shapes::cube(2.0),
        shapes::uv_sphere(8, 12, 1.0),
        shapes_open::open_tube3(),
    ] {
        let e = mesh.edges();
        let quads = mesh.faces().iter().filter(|f| !f.is_tri()).count();
        let out = subdivide(&mesh);
        assert_eq!(out.face_count(), mesh.face_count() * 4);
        assert_eq!(out.vert_count(), mesh.vert_count() + e.len() + quads);
    }
}

/// **A malha fechada continua FECHADA** — nenhuma aresta de saída com valência
/// diferente de 2.
///
/// É o gate que pega o defeito mais caro possível aqui: as duas faces de uma
/// aresta compartilhada nomearem vértices NOVOS diferentes. A malha rasgaria ao
/// longo de toda emenda, e nenhuma contagem acusaria.
#[test]
fn a_closed_mesh_stays_closed() {
    for mesh in [shapes::cube(2.0), shapes::uv_sphere(9, 13, 1.0)] {
        let out = subdivide(&mesh);
        let e = out.edges();
        assert!(!e.is_empty());
        for i in 0..e.len() as u32 {
            assert_eq!(e.valence(i), 2, "aresta {i} da saída");
        }
        // E Euler continua fechando.
        let (v, f) = (out.vert_count() as i64, out.face_count() as i64);
        assert_eq!(v - e.len() as i64 + f, 2);
    }
}

/// **A ORIENTAÇÃO sobrevive.** Numa malha fechada e convexa toda normal de face
/// aponta para longe do centro; se duas das quatro faces novas saíssem com o
/// winding trocado, o render mostraria buracos e nenhuma contagem mudaria.
#[test]
fn the_winding_of_every_new_face_follows_its_parent() {
    for mesh in [shapes::cube(2.0), shapes::uv_sphere(10, 14, 1.0)] {
        let out = subdivide(&mesh);
        for (f, face) in out.faces().iter().enumerate() {
            let p = out.positions();
            let v = face.verts();
            let c = [
                v.iter().map(|&i| p[i as usize][0]).sum::<f32>() / v.len() as f32,
                v.iter().map(|&i| p[i as usize][1]).sum::<f32>() / v.len() as f32,
                v.iter().map(|&i| p[i as usize][2]).sum::<f32>() / v.len() as f32,
            ];
            let n = out.face_normals()[f];
            let dot = c[0] * n[0] + c[1] * n[1] + c[2] * n[2];
            assert!(dot > 0.0, "face {f} aponta para dentro ({dot})");
        }
    }
}

/// ⚠️ **O gate que decide a wave: os pesos SOMAM 1.**
///
/// Um canal constante sobrevive a uma tabela afim, e só a ela. Como cada
/// vértice de saída toma um ramo diferente (Loop · Catmull-Clark · fronteira
/// tri/quad · borda · não-manifold), um único gate cobre os cinco — e um peso
/// errado em qualquer um sai como um vértice cujo valor não é mais a constante.
#[test]
fn a_constant_channel_survives_every_rule() {
    const C: f32 = 0.7;
    for mut mesh in [
        shapes::cube(2.0),
        shapes::uv_sphere(8, 12, 1.0),
        shapes_open::open_tube3(),
        shapes_open::pillow(),
    ] {
        let n = mesh.vert_count();
        mesh.masks_mut()[..n].fill(C);
        let out = subdivide(&mesh);
        let m = out.masks().expect("a máscara viaja");
        assert_eq!(m.len(), out.vert_count());
        let worst = m.iter().map(|&x| (x - C).abs()).fold(0.0f32, f32::max);
        assert!(
            worst < 1e-6,
            "um canal constante tem de sair constante, e desviou {worst}"
        );
    }
}

/// A máscara PINTADA viaja: o que estava protegido continua protegido, e o
/// resto continua livre — a subdivisão não é uma forma de perder trabalho.
#[test]
fn a_painted_mask_travels_through_the_subdivision() {
    let mut mesh = shapes::uv_sphere(12, 18, 1.0);
    let n = mesh.vert_count();
    let up: Vec<bool> = (0..n).map(|i| mesh.positions()[i][1] > 0.5).collect();
    {
        let m = mesh.masks_mut();
        for i in 0..n {
            m[i] = if up[i] { 1.0 } else { 0.0 };
        }
    }
    let out = subdivide(&mesh);
    let m = out.masks().expect("viaja");
    // Os vértices ORIGINAIS bem dentro de cada zona mantêm o valor.
    let mut checked = 0;
    for (i, &got) in m.iter().take(n).enumerate() {
        let y = mesh.positions()[i][1];
        if y > 0.8 {
            assert!(got > 0.99, "vértice {i} protegido virou {got}");
            checked += 1;
        } else if y < 0.2 {
            assert!(got < 0.01, "vértice {i} livre virou {got}");
            checked += 1;
        }
    }
    assert!(
        checked > 20,
        "a fixture tem de ter as duas zonas: {checked}"
    );
    // E nada saiu da faixa: os pesos são afins, então não há clamp escondendo.
    assert!(m.iter().all(|&x| (-1e-6..=1.0 + 1e-6).contains(&x)));
}

/// **A regra do vértice é Catmull-Clark, conferida contra a fórmula PUBLICADA.**
///
/// É o gate que separa *subdividir* de *inserir pontos médios*: com a regra
/// LINEAR os oito vértices originais ficariam onde estão.
///
/// ⚠️ **O oráculo é `(F + 2R + (n−3)V)/n` computada AQUI**, dos anéis, e não os
/// três literais do kernel. Um gate que repetisse `0,5625 / 0,09375 / 0,015625`
/// seria um espelho — ele passaria com os três trocados por outros três
/// consistentes entre si. Assim ele passa a **derivar** o que espera de uma
/// fonte que o produto não conhece.
///
/// ⚠️ E a primeira versão deste gate cravava **7/9**, um número que eu escrevi
/// sem fazer a soma vetorial: a quina do cubo vai a `(15V + 6Σanel + Σdiag)/36`,
/// que com `V = (−1,−1,−1)`, `Σanel = (−1,−1,−1)` e `Σdiag = (1,1,1)` dá
/// **−5/9**, e é o que o produto media. *Um oráculo derivado não teria como
/// carregar esse erro.*
#[test]
fn the_even_rule_is_the_published_catmull_clark() {
    let mesh = shapes::cube(2.0);
    let out = subdivide(&mesh);
    let p = mesh.positions();
    let adj = mesh.adjacency();

    for v in 0..mesh.vert_count() {
        let ring = adj.vert_verts.neighbours(v);
        let faces = adj.vert_faces.neighbours(v);
        let n = ring.len() as f32;
        assert!(
            mesh.faces().iter().all(|f| !f.is_tri()),
            "esta fórmula é a de malha só-quad"
        );

        // F: a média dos pontos de FACE (o centroide de cada quad incidente).
        // R: a média dos pontos MÉDIOS das arestas incidentes.
        let mut f_avg = [0.0f32; 3];
        for &fi in faces {
            let q = mesh.faces()[fi as usize];
            for k in 0..3 {
                let c: f32 = q.verts().iter().map(|&i| p[i as usize][k]).sum::<f32>() / 4.0;
                f_avg[k] += c / faces.len() as f32;
            }
        }
        let mut r_avg = [0.0f32; 3];
        for &w in ring {
            for k in 0..3 {
                r_avg[k] += (p[v][k] + p[w as usize][k]) * 0.5 / n;
            }
        }

        for k in 0..3 {
            let want = (f_avg[k] + 2.0 * r_avg[k] + (n - 3.0) * p[v][k]) / n;
            let got = out.positions()[v][k];
            assert!(
                (want - got).abs() < 1e-5,
                "vértice {v} eixo {k}: a fórmula publicada pede {want}, o produto deu {got}"
            );
        }
    }

    // E a consequência VISÍVEL, com o número: a quina anda para dentro.
    let len = |q: [f32; 3]| (q[0] * q[0] + q[1] * q[1] + q[2] * q[2]).sqrt();
    let ratio = len(out.positions()[0]) / len(p[0]);
    assert!(
        (ratio - 5.0 / 9.0).abs() < 1e-4,
        "a quina foi a {ratio:.4} do raio; a álgebra pede 5/9"
    );
}

/// **A BORDA de uma malha aberta não é sugada para o miolo** — a mesma lei que
/// a W6.0 pôs no laplaciano, agora na subdivisão: um vértice de beira ouve só a
/// beira.
///
/// O oráculo é o EIXO do tubo, pelo mesmo motivo de lá: o raio da boca encolhe
/// de propósito (alisar um polígono o leva ao círculo inscrito), e medir o raio
/// mediria o alisamento em vez da regra.
///
/// ⚠️ **E a primeira versão deste gate era VERDE sobre a regra removida** — a
/// mutação achou. Ele media a extensão em `y` da malha INTEIRA, e os pontos de
/// ARESTA da beira nascem do ponto médio de dois vértices de mesma altura (a
/// regra ímpar de borda, que a mutação não tocava): eles PINAM o extremo em
/// `y = ±1` enquanto os vértices ORIGINAIS da boca descem para `±0,75`. O
/// resultado é uma beira em zigue-zague, visualmente pior que o encolhimento, e
/// o número que o gate lia não se movia um ulp.
///
/// Agora ele mede exatamente os vértices a que a regra se aplica: **os de
/// BORDA**, que é onde a pergunta vive.
#[test]
fn the_lip_of_an_open_mesh_is_not_pulled_toward_the_middle() {
    let mesh = shapes_open::open_tube3();
    let out = subdivide(&mesh);
    let mut checked = 0;
    let mut worst = 0.0f32;
    for v in 0..mesh.vert_count() {
        if !mesh.adjacency().is_border(v) {
            continue;
        }
        checked += 1;
        worst = worst.max((out.positions()[v][1] - mesh.positions()[v][1]).abs());
    }
    assert!(checked >= 12, "a fixture tem de ter beira: {checked}");
    assert!(
        worst < 1e-5,
        "um vértice de boca andou {worst} no eixo — ele está ouvindo o miolo"
    );
}

/// Malhas degeneradas não fazem a subdivisão panicar nem produzir `NaN`. As
/// fixtures são as duas que a W1 construiu para a face de área zero.
#[test]
fn a_degenerate_mesh_subdivides_without_panicking_or_producing_nan() {
    for mesh in [
        shapes_open::collapsed_tetra(),
        shapes_open::sliver_bipyramid(),
        shapes_open::pillow(),
    ] {
        let out = subdivide(&mesh);
        assert!(
            out.positions()
                .iter()
                .all(|p| p.iter().all(|x| x.is_finite())),
            "posição não-finita na saída"
        );
    }
}

/// A entrada fica INTACTA — `subdivide` é uma função, não um gesto sobre a
/// malha que recebeu.
#[test]
fn the_input_mesh_is_untouched() {
    let mesh = shapes::uv_sphere(8, 10, 1.0);
    let before: Vec<[f32; 3]> = mesh.positions().to_vec();
    let faces = mesh.faces().to_vec();
    let _ = subdivide(&mesh);
    assert_eq!(mesh.positions(), &before[..]);
    assert_eq!(mesh.faces(), &faces[..]);
}

/// Subdividir de novo continua valendo — a saída é uma malha de primeira
/// classe, com adjacência, octree e normais próprias.
#[test]
fn the_output_is_a_first_class_mesh_that_subdivides_again() {
    let mesh = shapes::cube(2.0);
    let once = subdivide(&mesh);
    let twice = subdivide(&once);
    assert_eq!(twice.face_count(), mesh.face_count() * 16);
    // O octree respondeu (a malha nova foi de fato reconstruída).
    let ray = crate::Ray::new([0.0, 0.0, 5.0], [0.0, 0.0, -1.0]);
    assert!(twice.raycast(&ray).is_some(), "o índice espacial da saída");
}

/// **A regra da ARESTA, conferida contra as duas fórmulas publicadas.**
///
/// ⚠️ **Este gate nasceu de um buraco medido:** trocar o ponto de aresta pelo
/// ponto MÉDIO (a subdivisão LINEAR) sobrevivia a todos os outros gates desta
/// suíte — a contagem não muda, a malha continua fechada, o winding continua
/// certo, e o ponto médio é afim, então o canal constante também passa. A
/// diferença entre *subdividir* e *inserir pontos médios* estava aferida só no
/// lado PAR.
///
/// Os dois oráculos vêm de fora do produto:
/// - **Catmull-Clark** (só-quad): o ponto de aresta é `(v1 + v2 + f1 + f2)/4`,
///   onde `f` são os centroides das duas faces;
/// - **Loop** (só-triângulo): é `3/8·(v1 + v2) + 1/8·(o1 + o2)`, onde `o` são os
///   dois vértices opostos.
#[test]
fn the_odd_rule_is_the_published_catmull_clark_and_loop() {
    // — o lado QUAD —
    let mesh = shapes::cube(2.0);
    let out = subdivide(&mesh);
    let e = mesh.edges();
    let p = mesh.positions();
    for id in 0..e.len() as u32 {
        // As duas faces desta aresta, e os dois vértices que ela liga.
        let mut faces = Vec::new();
        let mut ends = None;
        for (f, face) in mesh.faces().iter().enumerate() {
            let v = face.verts();
            for k in 0..v.len() {
                if e.face_edge(f, k) == Some(id) {
                    faces.push(f);
                    ends = Some((v[k], v[(k + 1) % v.len()]));
                }
            }
        }
        assert_eq!(faces.len(), 2, "o cubo é fechado");
        let (a, b) = ends.expect("a aresta tem pontas");
        for (k, &pa) in p[a as usize].iter().enumerate() {
            let centroids: f32 = faces
                .iter()
                .map(|&f| {
                    let q = mesh.faces()[f].verts();
                    q.iter().map(|&i| p[i as usize][k]).sum::<f32>() / 4.0
                })
                .sum();
            let want = (pa + p[b as usize][k] + centroids) / 4.0;
            let got = out.positions()[mesh.vert_count() + id as usize][k];
            assert!(
                (want - got).abs() < 1e-5,
                "aresta {id} eixo {k}: Catmull-Clark pede {want}, o produto deu {got}"
            );
        }
    }

    // — o lado TRIÂNGULO —
    let mesh = shapes::octahedron(1.0);
    assert!(
        mesh.faces().iter().all(|f| f.is_tri()),
        "esta metade é a de malha só-triângulo"
    );
    let out = subdivide(&mesh);
    let e = mesh.edges();
    let p = mesh.positions();
    for id in 0..e.len() as u32 {
        let mut opposites = Vec::new();
        let mut ends = None;
        for (f, face) in mesh.faces().iter().enumerate() {
            let v = face.verts();
            for k in 0..3 {
                if e.face_edge(f, k) == Some(id) {
                    opposites.push(v[(k + 2) % 3]);
                    ends = Some((v[k], v[(k + 1) % 3]));
                }
            }
        }
        assert_eq!(opposites.len(), 2);
        let (a, b) = ends.expect("pontas");
        for (k, &pa) in p[a as usize].iter().enumerate() {
            let want = 0.375 * (pa + p[b as usize][k])
                + 0.125 * opposites.iter().map(|&o| p[o as usize][k]).sum::<f32>();
            let got = out.positions()[mesh.vert_count() + id as usize][k];
            assert!(
                (want - got).abs() < 1e-5,
                "aresta {id} eixo {k}: Loop pede {want}, o produto deu {got}"
            );
        }
    }
}
