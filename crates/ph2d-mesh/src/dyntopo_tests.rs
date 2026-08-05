//! Os gates da topologia dinâmica.
//!
//! ⚠️ **O oráculo central é a AUSÊNCIA DE RACHADURA**, e ele não é sobre
//! contagem: uma malha fechada refinada continua fechada, e isso se verifica
//! contando as faces de cada aresta — `2` no interior. Um refino que esquecesse
//! a vizinha deixaria arestas de valência `1` no meio da superfície, que é
//! precisamente o T-vértice, e a contagem de vértices estaria *certa*.

use super::*;
use crate::{Face, Mesh, shapes};

/// Uma esfera triangulada — a fixture do produto (o `uv_sphere` nasce em quads,
/// e é por isso que entrar em dyntopo triangula).
fn tri_sphere(rings: usize, segs: usize) -> Mesh {
    let mut m = shapes::uv_sphere(rings, segs, 1.0);
    m.triangulate();
    m
}

/// Quantas arestas do interior ficaram com valência ≠ 2 — **zero é o contrato**.
fn cracks(m: &Mesh) -> usize {
    let e = m.edges();
    (0..e.len() as u32).filter(|&i| e.valence(i) != 2).count()
}

#[test]
fn the_refined_mesh_has_no_cracks() {
    let mut m = tri_sphere(12, 18);
    assert_eq!(cracks(&m), 0, "o controle: a esfera nasce fechada");

    let target = edge_target(0.6, 1.0);
    let r = refine_in_sphere(&mut m, [0.0, 0.0, 1.0], 0.6, target, &mut Vec::new());
    assert!(
        matches!(r, Refine::Done { .. }),
        "algo tem de ser partido: {r:?}"
    );
    assert_eq!(
        cracks(&m),
        0,
        "toda face que toca uma aresta partida aprendeu o vértice do meio"
    );
}

/// **O PADRÃO É TOTAL** — e este é o gate que de fato prova a ausência de
/// rachadura, depois de a mutação ter mostrado que a varredura não era ela.
///
/// Para cada um dos **oito** subconjuntos de arestas partidas, os triângulos
/// emitidos têm de formar exatamente o mesmo pedaço de superfície: toda aresta
/// interior aparece **duas vezes, em sentidos opostos** (é isso que uma malha
/// sem rachadura É, localmente) e a fronteira é o perímetro original com os
/// meios inseridos. Uma face girada errado inverte um par e o multiconjunto
/// acusa.
#[test]
fn every_split_pattern_tiles_the_triangle_without_a_crack() {
    let v = [0u32, 1, 2];
    let mids = [3u32, 4, 5]; // meio de (0,1), de (1,2), de (2,0)
    for bits in 0u8..8 {
        let mid = [
            (bits & 1 != 0).then_some(mids[0]),
            (bits & 2 != 0).then_some(mids[1]),
            (bits & 4 != 0).then_some(mids[2]),
        ];
        let mut out = Vec::new();
        emit_triangle(&mut out, v, mid);
        let n = mid.iter().filter(|m| m.is_some()).count();
        assert_eq!(
            out.len(),
            [1, 2, 3, 4][n],
            "padrão {bits:03b}: número de faces"
        );

        // O multiconjunto de arestas DIRIGIDAS.
        let mut dir: Vec<(u32, u32)> = Vec::new();
        for f in &out {
            assert!(f.is_tri(), "padrão {bits:03b}: só triângulos");
            let t = f.verts();
            dir.push((t[0], t[1]));
            dir.push((t[1], t[2]));
            dir.push((t[2], t[0]));
        }
        // Fronteira esperada: o perímetro 0→1→2→0 com os meios inseridos.
        let mut border: Vec<(u32, u32)> = Vec::new();
        for k in 0..3 {
            let (a, b) = (v[k], v[(k + 1) % 3]);
            match mid[k] {
                Some(m) => border.extend([(a, m), (m, b)]),
                None => border.push((a, b)),
            }
        }
        for e in &border {
            assert!(
                dir.contains(e),
                "padrão {bits:03b}: a fronteira {e:?} tem de estar na saída"
            );
        }
        // Toda aresta que não é da fronteira aparece nos DOIS sentidos.
        for &(a, b) in &dir {
            if border.contains(&(a, b)) {
                continue;
            }
            assert!(
                dir.contains(&(b, a)),
                "padrão {bits:03b}: a aresta interior ({a},{b}) não tem par — \
                 é exatamente a forma de uma rachadura, ou de uma face invertida"
            );
        }
        // E nenhum meio dado fica de fora: um padrão que ignore um meio deixa o
        // T-vértice de volta, com a contagem de faces certa.
        for m in mid.into_iter().flatten() {
            assert!(
                dir.iter().any(|&(a, b)| a == m || b == m),
                "padrão {bits:03b}: o meio {m} não foi usado"
            );
        }
    }
}

#[test]
fn the_refinement_stays_inside_the_brush() {
    let mut m = tri_sphere(12, 18);
    let before = m.vert_count();
    // O dab no polo NORTE não pode adensar o polo sul.
    let centre = [0.0, 0.0, 1.0];
    let radius = 0.5f32;
    refine_in_sphere(
        &mut m,
        centre,
        radius,
        edge_target(radius, 1.0),
        &mut Vec::new(),
    );

    // ⚠️ O oráculo é a POSIÇÃO dos vértices novos, não a contagem: contar
    // provaria que cresceu, e o que está em julgamento é ONDE.
    //
    // ⚠️ **A barra é 2× o raio e não o raio, e o motivo é o FECHO de aresta mais
    // longa** (a *LEPP* de Rivara, `dyntopo.rs`): marcar uma aresta obriga a
    // vizinha a partir pela MAIS LONGA dela, o que pode obrigar a seguinte — é
    // essa cadeia que compra a qualidade do triângulo, e ela não para exatamente
    // no pincel. Medido sobre quatro densidades de esfera (sonda
    // `measure_how_far_the_propagation_reaches`): **1,66× · 1,31× · 1,38× ·
    // 0,93×** — ela ENCOLHE quando a malha já é fina, porque a cadeia é curta
    // quando as arestas já são pequenas.
    //
    // O que a barra protege continua sendo a promessa do modo: um dab no polo
    // NORTE não pode adensar o polo sul, que numa esfera unitária está a 4× este
    // raio.
    let far = m.positions()[before..]
        .iter()
        .filter(|p| {
            let d = [p[0] - centre[0], p[1] - centre[1], p[2] - centre[2]];
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() > radius * 2.0
        })
        .count();
    assert_eq!(far, 0, "nenhum vértice novo nasceu longe do pincel");
    assert!(m.vert_count() > before, "e alguma coisa nasceu");
}

#[test]
fn refining_twice_is_deterministic() {
    let run = || {
        let mut m = tri_sphere(10, 14);
        refine_in_sphere(
            &mut m,
            [0.3, 0.2, 0.9],
            0.7,
            edge_target(0.7, 0.8),
            &mut Vec::new(),
        );
        m
    };
    let (a, b) = (run(), run());
    assert_eq!(a.vert_count(), b.vert_count());
    assert_eq!(
        a.faces(),
        b.faces(),
        "os índices são os MESMOS, não só a forma"
    );
    assert_eq!(a.positions(), b.positions());
}

#[test]
fn the_edge_target_is_the_references_formula() {
    // Os dois extremos, conferidos contra a aritmética do `SculptBase.js`.
    let r = 2.0f32;
    assert!((edge_target(r, 0.0) - r * 0.22f32.sqrt()).abs() < 1e-6);
    assert!((edge_target(r, 1.0) - r * 0.02f32.sqrt()).abs() < 1e-6);
    // Mais detalhe ⇒ aresta MENOR. A direção é metade do contrato: invertê-la
    // deixa o slider vivo e o significado ao contrário.
    assert!(edge_target(r, 1.0) < edge_target(r, 0.5));
    assert!(edge_target(r, 0.5) < edge_target(r, 0.0));
    // Fora da faixa não estoura — o clamp é da porta, não do chamador.
    assert_eq!(edge_target(r, 2.0), edge_target(r, 1.0));
    assert_eq!(edge_target(r, -1.0), edge_target(r, 0.0));
}

#[test]
fn refining_shortens_the_edges_it_was_asked_about() {
    let mut m = tri_sphere(8, 12);
    let (centre, radius) = ([0.0, 0.0, 1.0], 0.6f32);
    let target = edge_target(radius, 1.0);

    let long_before = long_edges_near(&m, centre, radius, target);
    assert!(long_before > 0, "a fixture TEM de conter o fenômeno");

    refine_in_sphere(&mut m, centre, radius, target, &mut Vec::new());
    let long_after = long_edges_near(&m, centre, radius, target);
    assert!(
        long_after < long_before,
        "o refino encurtou as arestas do dab: {long_before} -> {long_after}"
    );
}

/// Arestas mais longas que `target` cujo meio cai na esfera — a mesma pergunta
/// que o motor faz, para o gate medir o que ele decide.
fn long_edges_near(m: &Mesh, c: [f32; 3], r: f32, target: f32) -> usize {
    let mut n = 0;
    for (fi, f) in m.faces().iter().enumerate() {
        let v = f.verts();
        for k in 0..v.len() {
            let Some(e) = m.edges().face_edge(fi, k) else {
                continue;
            };
            // Cada aresta é vista por duas faces; conta só uma vez.
            let _ = e;
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            if a > b {
                continue;
            }
            let (pa, pb) = (m.positions()[a as usize], m.positions()[b as usize]);
            let d = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
            let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            let mid = [
                (pa[0] + pb[0]) * 0.5,
                (pa[1] + pb[1]) * 0.5,
                (pa[2] + pb[2]) * 0.5,
            ];
            let dm = [mid[0] - c[0], mid[1] - c[1], mid[2] - c[2]];
            if len > target && (dm[0] * dm[0] + dm[1] * dm[1] + dm[2] * dm[2]).sqrt() <= r {
                n += 1;
            }
        }
    }
    n
}

#[test]
fn a_quad_mesh_is_refused_instead_of_mangled() {
    let mut m = shapes::uv_sphere(8, 12, 1.0);
    assert!(
        m.faces().iter().any(|f| !f.is_tri()),
        "o controle: há quads"
    );
    let r = refine_in_sphere(&mut m, [0.0, 0.0, 1.0], 0.6, 0.05, &mut Vec::new());
    assert_eq!(r, Refine::NotTriangles);
    assert_eq!(m.face_count(), 8 * 12, "e a malha não foi tocada");
}

#[test]
fn triangulating_is_idempotent_and_moves_no_vertex() {
    let mut m = shapes::uv_sphere(6, 8, 1.0);
    let pos = m.positions().to_vec();
    let added = m.triangulate();
    assert!(added > 0);
    assert_eq!(m.positions(), &pos[..], "triangular não move um vértice");
    assert!(m.faces().iter().all(Face::is_tri));
    assert_eq!(m.triangulate(), 0, "a segunda chamada é um no-op");
}

#[test]
fn the_new_vertices_carry_colour_and_mask() {
    let mut m = tri_sphere(8, 12);
    // Pinta o hemisfério norte e mascara-o pela metade: os canais têm de
    // interpolar, não de nascer no default.
    let n = m.vert_count();
    m.put_masks(vec![0.25; n]);
    for (i, c) in m.colors_mut().iter_mut().enumerate() {
        *c = if m_pos_z(i) {
            [1.0, 0.0, 0.0]
        } else {
            [0.0, 0.0, 1.0]
        };
    }
    let before = m.vert_count();
    refine_in_sphere(
        &mut m,
        [0.0, 0.0, 1.0],
        0.6,
        edge_target(0.6, 1.0),
        &mut Vec::new(),
    );
    assert!(m.vert_count() > before);

    let masks = m.masks().expect("o plano sobrevive ao refino");
    assert_eq!(masks.len(), m.vert_count(), "e mede a malha nova");
    assert!(
        masks[before..].iter().all(|&v| (v - 0.25).abs() < 1e-5),
        "o meio de dois 0,25 é 0,25 — um vértice novo em zero apagaria a máscara ali"
    );
    let colors = m.colors().expect("a cor também");
    assert_eq!(colors.len(), m.vert_count());
}

/// Um `i` par para a fixture de cor — só precisa ser determinístico.
fn m_pos_z(i: usize) -> bool {
    i.is_multiple_of(2)
}

#[test]
fn an_empty_dab_changes_nothing() {
    let mut m = tri_sphere(8, 12);
    let before = (m.vert_count(), m.face_count());
    // Longe da malha.
    assert_eq!(
        refine_in_sphere(&mut m, [10.0, 10.0, 10.0], 0.5, 0.01, &mut Vec::new()),
        Refine::Enough
    );
    // Alvo maior que qualquer aresta.
    assert_eq!(
        refine_in_sphere(&mut m, [0.0, 0.0, 1.0], 0.5, 100.0, &mut Vec::new()),
        Refine::Enough
    );
    // Alvo degenerado: recusa em vez de pedir infinitos vértices.
    assert_eq!(
        refine_in_sphere(&mut m, [0.0, 0.0, 1.0], 0.5, 0.0, &mut Vec::new()),
        Refine::Enough
    );
    assert_eq!((m.vert_count(), m.face_count()), before);
}

#[test]
fn the_midpoint_follows_the_curve_instead_of_flattening_it() {
    // Numa esfera de raio 1 o meio geométrico de uma aresta cai DENTRO; o
    // deslocamento pela normal o traz de volta para perto da superfície.
    let mut m = tri_sphere(8, 12);
    let before = m.vert_count();
    refine_in_sphere(
        &mut m,
        [0.0, 0.0, 1.0],
        0.8,
        edge_target(0.8, 1.0),
        &mut Vec::new(),
    );

    let radii: Vec<f32> = m.positions()[before..]
        .iter()
        .map(|p| (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt())
        .collect();
    assert!(!radii.is_empty());
    let worst = radii.iter().fold(1.0f32, |acc, &r| acc.min(r));
    assert!(
        worst > 0.985,
        "o pior vértice novo ficou a {worst} do centro — o refino está achatando a esfera"
    );
}

/// ⚠️ **O GATE DESTA WAVE.** Um refino que só PARTE não consegue manter a forma
/// dos triângulos: a vizinha de uma face escolhida tem de aprender o vértice
/// novo, e isso a corta pela metade do ângulo. Com a esfera do dab a ANDAR, o
/// mesmo anel é cortado outra vez a cada passo — o pior ângulo mínimo desabava
/// de **21,21° para 0,59°** e **48% da malha** ficava abaixo de 10°.
///
/// ⚠️ **E nenhuma métrica de POSIÇÃO via isso.** Uma lasca não desloca vértice
/// nenhum; o desvio de guarda-chuva media o MESMO com e sem o conserto do `pre`
/// (0,7131 contra 0,7158). O que a luz desenha como agulha é a normal
/// por-vértice de um triângulo fino, que não aponta para lado nenhum. Foi o
/// smoke de 2026-08-04 que o viu, e é o ÂNGULO que o mede.
///
/// As duas peças que o seguram estão as duas neste caminho, e cada uma tem o
/// seu número na ablação: o FECHO de aresta mais longa (0,59° → 2,43°) e o
/// FLIP (2,43° → 16,85°).
#[test]
fn a_moving_dab_does_not_shred_the_triangles() {
    let mut m = tri_sphere(10, 14);
    let before = worst_min_angle(&m);
    assert!(
        before > 20.0,
        "a fixture começa com triângulos sãos: {before}"
    );

    let radius = 0.30f32;
    let target = edge_target(radius, 0.5);
    let mut births = Vec::new();
    for k in 0..24 {
        let t = f64::from(k) / 23.0;
        let x = (-0.6 + 1.2 * t) as f32;
        let y = (1.0 - x * x).max(0.0).sqrt();
        refine_in_sphere(&mut m, [x, y, 0.0], radius, target, &mut births);
    }

    let after = worst_min_angle(&m);
    // A barra é MEDIDA (15,55° no dia em que isto foi escrito) e diz o que
    // importa: o refino não pode devolver um triângulo pior do que a malha que
    // ele recebeu deixaria alguém desenhar. Dez graus é onde a lasca começa a
    // ser visível na luz.
    assert!(
        after > 10.0,
        "o refino de um traço inteiro não pode picar a malha: {before} -> {after}"
    );
    assert!(m.vert_count() > 128, "e alguma coisa foi de fato refinada");
}

/// O menor ângulo de triângulo da malha inteira, em GRAUS.
fn worst_min_angle(m: &Mesh) -> f32 {
    let pos = m.positions();
    let mut tris = Vec::new();
    m.triangle_indices(&mut tris);
    let mut worst = 180.0f32;
    for t in &tris {
        for k in 0..3 {
            let (o, u, v) = (
                pos[t[k] as usize],
                pos[t[(k + 1) % 3] as usize],
                pos[t[(k + 2) % 3] as usize],
            );
            let a = [u[0] - o[0], u[1] - o[1], u[2] - o[2]];
            let b = [v[0] - o[0], v[1] - o[1], v[2] - o[2]];
            let la = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
            let lb = (b[0] * b[0] + b[1] * b[1] + b[2] * b[2]).sqrt();
            if la < 1e-12 || lb < 1e-12 {
                return 0.0;
            }
            let c = ((a[0] * b[0] + a[1] * b[1] + a[2] * b[2]) / (la * lb)).clamp(-1.0, 1.0);
            worst = worst.min(c.acos().to_degrees());
        }
    }
    worst
}

/// A parentela é COMPLETA e vem em ORDEM — as duas metades de que o consumidor
/// depende. Sem a contagem, um vértice fica sem `pre` derivável; sem a ordem, um
/// filho do passe 2 perguntaria a um pai que ainda não foi resolvido.
#[test]
fn every_new_vertex_declares_where_it_came_from_in_order() {
    let mut m = tri_sphere(10, 14);
    let before = m.vert_count();
    let mut births = Vec::new();
    let r = 0.5f32;
    refine_in_sphere(&mut m, [0.0, 0.0, 1.0], r, edge_target(r, 1.0), &mut births);

    assert_eq!(
        births.len(),
        m.vert_count() - before,
        "um nascimento por vértice novo"
    );
    for (i, b) in births.iter().enumerate() {
        assert_eq!(
            b.vert as usize,
            before + i,
            "os índices novos saem em sequência"
        );
        // ⚠️ Um pai SEMPRE precede o filho — é isto que torna a travessia para a
        // frente suficiente, e é o que um passe 2 quebraria se nascesse antes.
        assert!(b.a < b.vert && b.b < b.vert, "o pai precede o filho");
        assert_ne!(b.a, b.b, "uma aresta tem dois extremos distintos");
    }
}

/// O buffer é LIMPO por quem escreve. Uma chamada que não parte nada não pode
/// deixar o chamador a olhar para os nascimentos da anterior — ele semearia
/// vértices que já foram semeados, com pais que já se moveram.
#[test]
fn a_refusal_leaves_no_stale_parentage_behind() {
    let mut m = tri_sphere(10, 14);
    let mut births = Vec::new();
    let r = 0.5f32;
    refine_in_sphere(&mut m, [0.0, 0.0, 1.0], r, edge_target(r, 1.0), &mut births);
    assert!(!births.is_empty(), "a fixture TEM de conter o fenômeno");

    // Longe de tudo: nada a partir.
    refine_in_sphere(&mut m, [10.0, 10.0, 10.0], 0.1, 0.05, &mut births);
    assert!(births.is_empty(), "o buffer é do refino, não do chamador");
}

/// **O FLIP PERGUNTA SÓ PELO QUE O CORTE MEXEU.**
///
/// ⚠️ **A fixture é construída para CONTER o fenômeno, e sem isso o gate seria
/// vazio:** uma esfera UV já é estável a flip fora da região do dab (medido — um
/// dab a 28k altera 3648 faces, todas dentro da esfera do pincel e nenhuma
/// fora), então sobre ela um flip global e um flip local dão o MESMO resultado e
/// nenhum oráculo os separa. Aqui um par de faces é deliberadamente virado para
/// a pior diagonal, longe de tudo: uma varredura global o encontraria e o
/// consertaria, uma varredura da região não.
///
/// As duas metades são independentes e as duas são precisas:
///
/// 1. **Sem sementes ele não sai à procura** — é o escopo.
/// 2. **Apontado para o estrago ele repara** — é o controle positivo, e sem ele
///    a primeira metade passaria com um operador que simplesmente não funciona.
#[test]
fn the_flip_asks_only_about_the_faces_the_cut_touched() {
    let (mut m, pair) = wreck_the_worst_pair(&tri_sphere(16, 24));
    assert!(
        has_pair(&m, pair),
        "o controle: a fixture tem de conter o estrago"
    );

    crate::dyntopo_flip::relax(&mut m, &[]);
    assert!(
        has_pair(&m, pair),
        "o flip varreu a malha atrás de trabalho que ninguém pediu"
    );

    let seed = face_of(&m, pair[0]).expect("a face estragada está na malha");
    crate::dyntopo_flip::relax(&mut m, &[seed]);
    assert!(
        !has_pair(&m, pair),
        "o operador tem de QUERER reparar isto — sem esta metade, a de cima é vazia"
    );
}

/// **Troca a diagonal do par vizinho cuja troca mais PIORA a qualidade** — o
/// estrago que o operador vai querer desfazer. Devolve a malha e as duas faces
/// novas por conjunto de vértices (que sobrevive a renumeração).
fn wreck_the_worst_pair(m: &Mesh) -> (Mesh, [[u32; 3]; 2]) {
    let pos = m.positions();
    let adj = m.adjacency();
    let src = m.faces();
    let mut best: Option<(f32, usize, usize, [u32; 4])> = None;
    for (i0, f0) in src.iter().enumerate() {
        if !f0.is_tri() {
            continue;
        }
        let v0 = f0.verts();
        for k in 0..3 {
            let (ea, eb) = (v0[k], v0[(k + 1) % 3]);
            let Some(i1) = adj
                .vert_faces
                .neighbours(ea as usize)
                .iter()
                .copied()
                .find(|&j| j as usize != i0 && src[j as usize].verts().contains(&eb))
                .map(|j| j as usize)
            else {
                continue;
            };
            if !src[i1].is_tri() {
                continue;
            }
            let Some((a, b, c, d)) = quad_of(src, i0, i1) else {
                continue;
            };
            let p = |v: u32| pos[v as usize];
            let old = min_angle(p(a), p(b), p(c)).min(min_angle(p(b), p(a), p(d)));
            let new = min_angle(p(a), p(d), p(c)).min(min_angle(p(d), p(b), p(c)));
            let loss = old - new;
            if best.is_none_or(|(l, ..)| loss > l) {
                best = Some((loss, i0, i1, [a, b, c, d]));
            }
        }
    }
    let (loss, i0, i1, [a, b, c, d]) = best.expect("a esfera tem pares vizinhos");
    assert!(
        loss > 1.0,
        "a fixture precisa de um par cuja troca piore de verdade: {loss} grau(s)"
    );
    let (n0, n1) = (Face::tri(a, d, c), Face::tri(d, b, c));
    let mut faces = src.to_vec();
    faces[i0] = n0;
    faces[i1] = n1;
    let wrecked = Mesh::from_parts(pos.to_vec(), faces).expect("a troca não inventa índice");
    (wrecked, [sorted(n0), sorted(n1)])
}

/// Os quatro cantos do quadrilátero de duas faces vizinhas — o mesmo desenho do
/// `dyntopo_flip::quad`, escrito aqui porque uma FIXTURE constrói um estado; ela
/// não pode chamar a função sob teste para decidir o que espera.
fn quad_of(faces: &[Face], i0: usize, i1: usize) -> Option<(u32, u32, u32, u32)> {
    let (t0, t1) = (faces[i0].verts(), faces[i1].verts());
    let k = (0..3).find(|&k| !t1.contains(&t0[k]))?;
    let c = t0[k];
    let (a, b) = (t0[(k + 1) % 3], t0[(k + 2) % 3]);
    let d = *t1.iter().find(|v| **v != a && **v != b)?;
    Some((a, b, c, d))
}

fn min_angle(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> f32 {
    let pts = [p0, p1, p2];
    let mut worst = 180.0f32;
    for k in 0..3 {
        let (o, u, v) = (pts[k], pts[(k + 1) % 3], pts[(k + 2) % 3]);
        let a = [u[0] - o[0], u[1] - o[1], u[2] - o[2]];
        let b = [v[0] - o[0], v[1] - o[1], v[2] - o[2]];
        let la = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
        let lb = (b[0] * b[0] + b[1] * b[1] + b[2] * b[2]).sqrt();
        if la < 1e-12 || lb < 1e-12 {
            return 0.0;
        }
        let c = ((a[0] * b[0] + a[1] * b[1] + a[2] * b[2]) / (la * lb)).clamp(-1.0, 1.0);
        worst = worst.min(c.acos().to_degrees());
    }
    worst
}

fn sorted(f: Face) -> [u32; 3] {
    let v = f.verts();
    let mut k = [v[0], v[1], v[2]];
    k.sort_unstable();
    k
}

fn face_of(m: &Mesh, key: [u32; 3]) -> Option<u32> {
    m.faces()
        .iter()
        .position(|f| f.is_tri() && sorted(*f) == key)
        .map(|i| u32::try_from(i).unwrap_or(u32::MAX))
}

fn has_pair(m: &Mesh, pair: [[u32; 3]; 2]) -> bool {
    pair.iter().all(|k| face_of(m, *k).is_some())
}
