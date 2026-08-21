//! **OS GATES DA QUADRANGULAÇÃO** (ADR-0161, F5).
//!
//! ⭐ **A régua principal é a CARACTERÍSTICA DE EULER.** Ela apanha, com um só
//! número, a malha rasgada, a face duplicada e o patch montado ao contrário —
//! defeitos que uma contagem de quads e uma inspeção visual deixam passar.

use ph2d_crossfield::{Dual, solve_miq};
use ph2d_mesh::{Mesh, shapes};
use ph2d_quadfill::fan::{coons, resample};
use ph2d_quadfill::{SMOOTHING_ROUNDS, fill};
use ph2d_quantize::quantize;
use ph2d_trace::trace_patches;

/// Corre a cadeia inteira e devolve a malha de quads e o relatório.
fn chain(mut mesh: Mesh, target_edge: f32) -> (Mesh, ph2d_quadfill::FillReport) {
    mesh.triangulate();
    let dual = Dual::build(&mesh);
    let (field, _) = solve_miq(&dual);
    let layout = trace_patches(&mesh, &dual, &field);
    let l = layout.to_layout(target_edge).expect("o layout fecha");
    let (q, _) = quantize(&l).expect("quantiza");
    fill(&mesh, &layout, &q, SMOOTHING_ROUNDS).expect("monta")
}

/// `V − E + F` da malha.
fn euler(mesh: &Mesh) -> i64 {
    let mut edges = std::collections::BTreeSet::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            edges.insert((a.min(b), a.max(b)));
        }
    }
    i64::try_from(mesh.vert_count()).unwrap() - i64::try_from(edges.len()).unwrap()
        + i64::try_from(mesh.face_count()).unwrap()
}

// ─────────────────────── a promessa da família ───────────────────────

#[test]
fn every_face_is_a_quad_and_the_mesh_is_watertight() {
    // ⭐ **É a promessa inteira desta família de algoritmos.** O motor local que
    // ela substitui entregava 65 a 83 % de quads; aqui é tudo ou é um defeito.
    for (name, mesh, edge) in [
        ("esfera 24x36", shapes::uv_sphere(24, 36, 1.0), 0.08),
        ("esfera 48x72", shapes::uv_sphere(48, 72, 1.0), 0.06),
        ("toro", shapes::torus(32, 16, 1.0, 0.35), 0.06),
    ] {
        let (_, r) = chain(mesh, edge);
        assert_eq!(r.non_quads, 0, "{name}: sairam faces que nao sao quads");
        assert!(r.quads > 0, "{name}: nao saiu quad nenhum");
        // ⚠️ Uma aresta com UMA face só é a assinatura da malha rasgada na divisa
        // entre patches — o defeito que a amostragem partilhada existe para
        // impedir, e que nenhum render mostra.
        assert_eq!(r.boundary_edges, 0, "{name}: a malha saiu aberta");
    }
}

#[test]
fn the_euler_characteristic_survives_the_remesh() {
    // ⭐ **Um número, três defeitos.** Rasgar uma divisa, duplicar uma face ou
    // montar um patch ao contrário mudam `V − E + F`, e nenhum deles se vê a
    // olho. A esfera tem de dar **2** e o toro **0**, e isso é topologia — não
    // depende de densidade, de alvo nem do solver.
    let (sphere, _) = chain(shapes::uv_sphere(48, 72, 1.0), 0.06);
    assert_eq!(euler(&sphere), 2, "a esfera tem de dar 2");
    let (torus, _) = chain(shapes::torus(32, 16, 1.0, 0.35), 0.06);
    assert_eq!(euler(&torus), 0, "o toro tem de dar 0");
}

#[test]
fn the_irregular_vertices_stay_near_the_topological_floor() {
    // ⭐ **A grandeza que o pivô existiu para derrubar.** Uma grade numa esfera
    // admite **oito** vértices irregulares — é topologia, não gosto. O motor
    // local entregava 21 a 49 % de TODOS os vértices (milhares); o oráculo fica
    // perto do chão.
    //
    // ⚠️ **A barra é a CONTAGEM, nunca a percentagem.** A mesma malha com o dobro
    // da densidade tem os mesmos irregulares e metade da percentagem — medir em
    // % faz o número melhorar sozinho quando nada melhorou.
    for (name, mesh, edge, floor) in [
        ("esfera 24x36", shapes::uv_sphere(24, 36, 1.0), 0.08, 8),
        ("esfera 48x72", shapes::uv_sphere(48, 72, 1.0), 0.06, 8),
        ("esfera 48x72 fina", shapes::uv_sphere(48, 72, 1.0), 0.03, 8),
    ] {
        let (_, r) = chain(mesh, edge);
        assert!(
            r.irregular >= floor,
            "{name}: {} irregulares e' ABAIXO do chao topologico {floor} — \
             a contagem esta' errada, nao a malha",
            r.irregular
        );
        assert!(
            r.irregular <= 6 * floor,
            "{name}: {} irregulares, acima de 6x o chao ({})",
            r.irregular,
            6 * floor
        );
    }
}

#[test]
fn a_finer_target_adds_quads_without_adding_irregulars() {
    // ⭐ **A prova de que a estrutura é da TOPOLOGIA e não da densidade.** Pedir
    // uma grade duas vezes mais fina tem de dar muito mais quads e o **mesmo**
    // número de vértices irregulares — eles vivem nos cantos dos patches, que não
    // mudam com o alvo. Se subirem junto, o que se está a contar é ruído.
    let (_, coarse) = chain(shapes::uv_sphere(48, 72, 1.0), 0.06);
    let (_, fine) = chain(shapes::uv_sphere(48, 72, 1.0), 0.03);
    assert!(
        fine.quads > coarse.quads * 2,
        "o alvo fino ({}) tinha de dar bem mais quads que o grosso ({})",
        fine.quads,
        coarse.quads
    );
    assert_eq!(
        fine.irregular, coarse.irregular,
        "os irregulares sao dos CANTOS dos patches, e o alvo nao os move"
    );
}

// ───────────────────────── as peças ─────────────────────────

#[test]
fn coons_reproduces_its_four_borders_exactly() {
    // ⚠️ **Um Coons que não devolve os bordos que recebeu não costura nada**: os
    // pontos da grade deixariam de coincidir com os do patch vizinho, e a divisa
    // abriria — sem erro, com um passo minúsculo.
    let bottom = [[0.0, 0.0, 0.0], [1.0, 0.2, 0.0], [2.0, 0.0, 0.0]];
    let top = [[0.0, 0.0, 1.0], [1.0, -0.3, 1.0], [2.0, 0.0, 1.0]];
    let left = [[0.0, 0.0, 0.0], [0.0, 0.5, 0.5], [0.0, 0.0, 1.0]];
    let right = [[2.0, 0.0, 0.0], [2.0, -0.5, 0.5], [2.0, 0.0, 1.0]];
    let g = coons(&bottom, &top, &left, &right);
    for k in 0..3 {
        for c in 0..3 {
            assert!((g[k][0][c] - bottom[k][c]).abs() < 1e-5, "bottom {k}");
            assert!((g[k][2][c] - top[k][c]).abs() < 1e-5, "top {k}");
            assert!((g[0][k][c] - left[k][c]).abs() < 1e-5, "left {k}");
            assert!((g[2][k][c] - right[k][c]).abs() < 1e-5, "right {k}");
        }
    }
}

#[test]
fn resample_splits_by_arc_length_not_by_vertex_count() {
    // ⚠️ **A cadeia de malha tem arestas de tamanhos diferentes.** Dividir por
    // CONTAGEM põe os pontos onde a triangulação por acaso é densa, e a grade
    // herda a densidade da malha de entrada em vez da do alvo — que é exatamente
    // o defeito que o F1 existe para não ter.
    // Aqui: quatro vértices, mas o primeiro segmento vale 90 % do comprimento.
    let chain = [
        [0.0, 0.0, 0.0],
        [9.0, 0.0, 0.0],
        [9.5, 0.0, 0.0],
        [10.0, 0.0, 0.0],
    ];
    let out = resample(&chain, 2);
    assert_eq!(out.len(), 3);
    assert!(
        (out[1][0] - 5.0).abs() < 1e-4,
        "o meio tem de cair em 5,0 (metade do COMPRIMENTO), caiu em {}",
        out[1][0]
    );
    assert!((out[0][0]).abs() < 1e-6 && (out[2][0] - 10.0).abs() < 1e-6);
}
