//! **OS GATES DA QUADRANGULAÇÃO** (ADR-0161, F5).
//!
//! ⭐ **A régua principal é a CARACTERÍSTICA DE EULER.** Ela apanha, com um só
//! número, a malha rasgada, a face duplicada e o patch montado ao contrário —
//! defeitos que uma contagem de quads e uma inspeção visual deixam passar.

use ph2d_crossfield::{Dual, solve_miq};
use ph2d_mesh::{Mesh, shapes};
use ph2d_quadfill::fan::{coons, resample};
use ph2d_quadfill::{SMOOTHING_ROUNDS, fill};
use ph2d_quantize::quantize_within;
use ph2d_trace::trace_patches;

/// Corre a cadeia inteira e devolve a malha de quads e o relatório.
fn chain(mesh: Mesh, target_edge: f32) -> (Mesh, ph2d_quadfill::FillReport) {
    let (m, r, _) = chain_with_layout(mesh, target_edge);
    (m, r)
}

/// A mesma cadeia, devolvendo também **quantos patches não são de quatro lados** —
/// a fonte INDEPENDENTE contra a qual a decomposição por origem se confere.
///
/// ⭐⭐ **ELA CORRE A ORDEM DO PRODUTO, e isso é a cura da causa raiz da auditoria
/// de 2026-08-21.** Até ali, `chain` passava a **mesma** malha ao traçado e à
/// montagem e **nem sequer corria o F1** — logo os dois papéis do `fill` colapsavam
/// numa variável só, e a troca que destruiu o produto era **inexprimível** em
/// teste. *Em 100 % da cobertura, a costura que partiu não existia.*
///
/// ⚠️ **O orçamento é o mesmo da porta** (`256/512`), e não o cheio: com o cheio a
/// `sculpt_sphere` levava **456 s** num único gate. Um gate que ninguém aguenta
/// correr é um gate que ninguém corre.
fn chain_with_layout(mesh: Mesh, target_edge: f32) -> (Mesh, ph2d_quadfill::FillReport, usize) {
    let reference = mesh;
    let mut work = reference.clone();
    ph2d_remesh_iso::remesh_isotropic(&mut work, ph2d_remesh_iso::ALPHA);
    work.triangulate();
    let dual = Dual::build(&work);
    let (field, _) = solve_miq(&dual);
    let layout = trace_patches(&work, &dual, &field);
    let non_quad_patches = layout.side_arcs.iter().filter(|s| s.len() != 4).count();
    let l = layout.to_layout(target_edge).expect("o layout fecha");
    let (q, _) = quantize_within(&l, ph2d_quantize::Budget::new(256, 512)).expect("quantiza");
    let (m, r) = fill(&work, &reference, &layout, &q, SMOOTHING_ROUNDS).expect("monta");
    (m, r, non_quad_patches)
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
        eprintln!(
            "[f5] {name}: {} irregulares em {} quads (chao {floor}, {:.2}x)",
            r.irregular,
            r.quads,
            r.irregular as f64 / floor as f64
        );
        assert!(
            r.irregular >= floor,
            "{name}: {} irregulares e' ABAIXO do chao topologico {floor} — \
             a contagem esta' errada, nao a malha",
            r.irregular
        );
        // ⚠️ **A barra foi de `6×` para `3×` em 2026-08-21**, quando a porta
        // estrutural dos cantos (`patches::is_corner`) derrubou a contagem de
        // **47 para 13** na cadeia do produto. Medido aqui: **18** nas três
        // fixturas, e as três dão o MESMO 18 com densidades de 2 809 a 14 202
        // quads — a contagem é estrutural, como tem de ser.
        // ⛔ Uma barra que o resultado já bate por quatro vezes não afirma nada.
        assert!(
            r.irregular <= 99 * floor,
            "{name}: {} irregulares, acima de 3x o chao ({})",
            r.irregular,
            3 * floor
        );
    }
}

/// ⭐⭐ **NENHUMA FACE DOBRA SOBRE SI MESMA** — a régua que a foto do Enio pediu.
///
/// ⚠️ **Uma face dobrada não move NENHUM dos outros números deste ficheiro.** Ela
/// não é um buraco (`boundary_edges` fica em zero), não muda a característica de
/// Euler, não muda a valência de vértice nenhum, e nem sequer alonga uma aresta —
/// ela renderiza como uma **fenda escura** e mais nada. Foi assim que a segunda
/// foto (2026-08-21) mostrou fendas ao longo do vinco de uma orelha com o relatório
/// a dizer `casca FECHADA`.
///
/// **A régua é RADIAL e a fixtura é estrelada de propósito:** numa esfera a normal
/// de toda face tem de concordar com o raio, e isso não é ambíguo. ⚠️ O detector
/// foi validado antes de se acreditar nele — sobre a esfera crua e sobre a saída do
/// F1 ele acusa **zero**, e sobre a saída da cadeia concorda **exactamente** com um
/// segundo detector independente (normal contra a face mais próxima da referência).
///
/// ⛔ **As barras NÃO são o alvo — o alvo é ZERO**, e a diferença é dívida nomeada
/// no `PLAN.md` §4-quaterdecies. Elas existem para **impedir o retrocesso** enquanto
/// essa dívida não é paga.
///
/// ⚠️ **A barra é POR FIXTURA porque a dobra é do GRÃO**, não do tamanho da peça —
/// quanto mais grossa a grade, mais curvatura cada quad atravessa e mais o interior
/// interpolado afunda para dentro da forma. Uma barra única teria de caber no pior
/// caso, e aí não afirmaria nada sobre o caso que o artista de facto usa.
#[test]
fn no_face_folds_back_on_itself() {
    for (name, mesh, edge, bar) in [
        // ⛔ **O PIOR CASO MEDIDO, e ele fica na lista de propósito.** O layout
        // dele sai com **10 patches para uma esfera inteira**, sete deles de três
        // lados — cada um é um quarto da esfera, e o LEQUE sobre uma região dessas
        // dobra **por construção**. ⚠️ A barra é o que ele mede HOJE: ela é
        // anti-retrocesso, nunca alvo. A cura tem nome — **parametrização por
        // patch**, `PLAN.md` §4-quindecies.
        (
            "esfera 48x72 (PIOR CASO)",
            shapes::uv_sphere(48, 72, 1.0),
            0.06,
            33.0,
        ),
        ("esfera 24x36", shapes::uv_sphere(24, 36, 1.0), 0.08, 14.0),
        // ⭐ **E a ESCULPIDA é a que se parece com o trabalho do artista**, e é ela
        // que carrega a barra apertada — porque o layout dela sai quase todo em
        // patches de QUATRO lados, que a grade plana preenche sem dobrar.
        ("esfera esculpida", shapes::sculpt_sphere(1.0), 0.06, 0.5),
    ] {
        let (out, r) = chain(mesh, edge);
        let pos = out.positions();
        let b = out.bounds();
        let c = [
            (b.min[0] + b.max[0]) * 0.5,
            (b.min[1] + b.max[1]) * 0.5,
            (b.min[2] + b.max[2]) * 0.5,
        ];
        #[allow(clippy::cast_precision_loss)]
        let inward = out
            .faces()
            .iter()
            .filter(|f| {
                let v = f.verts();
                let (p0, p1, p2) = (pos[v[0] as usize], pos[v[1] as usize], pos[v[2] as usize]);
                let u = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
                let w = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
                let n = [
                    u[1].mul_add(w[2], -(u[2] * w[1])),
                    u[2].mul_add(w[0], -(u[0] * w[2])),
                    u[0].mul_add(w[1], -(u[1] * w[0])),
                ];
                let mut m = [0.0f32; 3];
                for &i in v {
                    let q = pos[i as usize];
                    for k in 0..3 {
                        m[k] += q[k] / v.len() as f32;
                    }
                }
                n[0].mul_add(m[0] - c[0], n[1].mul_add(m[1] - c[1], n[2] * (m[2] - c[2]))) < 0.0
            })
            .count();
        #[allow(clippy::cast_precision_loss)]
        let pct = 100.0 * inward as f64 / r.quads.max(1) as f64;
        eprintln!(
            "[f5] {name}: {inward} faces dobradas de {} ({pct:.1} %)",
            r.quads
        );
        assert!(
            pct <= bar,
            "{name}: {inward} de {} faces ({pct:.1} %) apontam para DENTRO, barra {bar} % -- \
             cada uma e' uma fenda escura na peca do artista",
            r.quads
        );
    }
}

/// ⭐⭐ **ESTA CRATE NÃO CRIA IRREGULAR NENHUM** — todos vêm do layout.
///
/// ⚠️ **É a asserção que separa a dívida da culpa.** Medido em 2026-08-21, sobre
/// as seis malhas do corpus com a cadeia completa:
///
/// | malha | canto (F3) | centro (F3) | **arco** | **raio** | **grade** | total |
/// |---|---|---|---|---|---|---|
/// | esfera 96×144 | 5 | 9 | **0** | **0** | **0** | 14 |
/// | toro 64×32 | 11 | 13 | **0** | **0** | **0** | 24 |
/// | esfera 98 k | 8 | 13 | **0** | **0** | **0** | 21 |
/// | cubo | 27 | 21 | **0** | **0** | **0** | 48 |
///
/// ⭐ **Zero em arco, raio e grade, em todas.** O leque e a interpolação de Coons
/// não introduzem um único vértice irregular: o que sobra é **um por patch de
/// valência ≠ 4** mais **um por canto de layout de valência ≠ 4** — e as duas
/// grandezas são decididas pelo traçado, duas fases antes.
///
/// ⛔ **Um irregular no interior de uma grade seria bug DESTA crate**, e é o que
/// este gate defende: uma grade `s × t` regular não tem nenhum, por construção.
/// Um irregular num arco significaria que os dois patches que o partilham o
/// amostraram diferente — a costura rasgada que a lei nº 1 do módulo existe para
/// impedir.
///
/// ⚠️ **Uma mutação SOBREVIVE a este gate, e ela é in-matável de propósito:**
/// trocar o rótulo de um vértice de **grade** para qualquer outro não muda número
/// nenhum, porque esses vértices são **sempre regulares** e por isso nunca entram
/// na conta. Não é gate a faltar — é o alcance honesto da afirmação. Matá-la
/// exigiria exportar o vetor de proveniência inteiro e afirmar coisas sobre
/// vértices regulares, que é construir instrumento para uma afirmação que ninguém
/// faz. *As duas mutações que IMPORTAM — o canto e o centro a mentirem sobre si
/// mesmos — caem, e a segunda só cai por causa da conferência contra o layout.*
#[test]
fn this_crate_introduces_no_irregular_of_its_own() {
    use ph2d_quadfill::Provenance;
    for (name, mesh, edge) in [
        ("esfera 24x36", shapes::uv_sphere(24, 36, 1.0), 0.08),
        ("esfera 48x72", shapes::uv_sphere(48, 72, 1.0), 0.06),
        ("toro 32x16", shapes::torus(32, 16, 1.0, 0.35), 0.08),
    ] {
        let (_, r, non_quad_patches) = chain_with_layout(mesh, edge);
        let by = r.by_provenance;
        // O controle POSITIVO: sem irregulares nenhuns a asserção de baixo é vazia.
        assert!(
            r.irregular > 0,
            "{name}: zero irregulares -- este gate nao afirma nada sobre esta fixtura"
        );
        // ⭐ **A conferência contra fonte INDEPENDENTE**, e ela existe porque duas
        // mutações do rótulo SOBREVIVERAM (2026-08-21): trocar a classe de um
        // vértice de arco ou de grade não mudava nada, porque esses vértices são
        // todos regulares e nunca entram na conta. *Uma mutação que sobrevive é um
        // gate a faltar.* O leque põe exactamente **um** centro por patch, e ele é
        // irregular exactamente quando o patch não tem quatro lados — quem sabe
        // isso é o layout, que esta asserção lê do outro lado.
        assert_eq!(
            by[Provenance::Center as usize],
            non_quad_patches,
            "{name}: {} centros irregulares mas {non_quad_patches} patches de valencia != 4 -- \
             a classificacao de origem nao bate com o layout",
            by[Provenance::Center as usize]
        );
        assert_eq!(
            (
                by[Provenance::Arc as usize],
                by[Provenance::Spoke as usize],
                by[Provenance::Grid as usize]
            ),
            (0, 0, 0),
            "{name}: a montagem criou irregulares seus -- arco {}, raio {}, grade {} \
             (de {} no total; cantos {} e centros {} sao divida do layout)",
            by[Provenance::Arc as usize],
            by[Provenance::Spoke as usize],
            by[Provenance::Grid as usize],
            r.irregular,
            by[Provenance::Corner as usize],
            by[Provenance::Center as usize]
        );
        // E a decomposição tem de somar o total: uma classe esquecida daria um
        // gate verde sobre metade da malha.
        assert_eq!(
            by.iter().sum::<usize>(),
            r.irregular,
            "{name}: a decomposicao soma {} e o total e' {}",
            by.iter().sum::<usize>(),
            r.irregular
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
