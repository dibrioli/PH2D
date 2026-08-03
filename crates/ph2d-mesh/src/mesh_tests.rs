//! Gates da malha.

use super::*;
use crate::shapes;

#[test]
fn a_cube_is_six_quads_and_twelve_triangles() {
    let m = shapes::cube(2.0);
    assert_eq!(m.vert_count(), 8);
    assert_eq!(m.face_count(), 6);
    assert_eq!(
        m.triangle_count(),
        12,
        "a contagem de TRIÂNGULOS não é a de faces quando há quads — é ela que os tetos por tier falam"
    );
    let mut tris = Vec::new();
    m.triangle_indices(&mut tris);
    assert_eq!(tris.len(), 12);
}

/// Num sólido convexo centrado na origem, a normal de cada vértice aponta para
/// fora ⇔ `dot(normal, posição) > 0`. Este é o gate da CONVENÇÃO de winding, e
/// ele falha se a fórmula de Newell inverter o sinal — o que a renderização
/// mostraria como um objeto iluminado por dentro.
///
/// ⚠️ **É aqui que uma fixture nova mal-enrolada é pega, e por isso a
/// `sliver_bipyramid` entra:** ela é convexa (os quatro arcos do anel são < 180°,
/// então o centro fica estritamente dentro), logo o oráculo se aplica a ela sem
/// emenda. As outras três fixtures malformadas **não** entram, e não é omissão:
/// o `open_tube3` é aberto, o `pillow` tem volume zero e o `collapsed_tetra`
/// colapsou num triângulo — nenhum é um sólido convexo, e enfiá-los aqui seria
/// um gate falhando por não ser sobre eles.
#[test]
fn the_normals_of_a_convex_solid_point_outward() {
    for m in [
        shapes::cube(2.0),
        shapes::uv_sphere(12, 16, 1.0),
        crate::shapes_open::sliver_bipyramid(),
    ] {
        for (v, n) in m.normals().iter().enumerate() {
            let p = m.positions()[v];
            let dot = p[0] * n[0] + p[1] * n[1] + p[2] * n[2];
            assert!(
                dot > 0.0,
                "a normal do vértice {v} aponta para dentro ({dot})"
            );
        }
        for (f, n) in m.face_normals().iter().enumerate() {
            let c = crate::normals::face_normal(m.positions(), m.faces()[f]);
            assert_eq!(*n, c, "a normal guardada divergiu da porta que a calcula");
        }
    }
}

/// Uma face sem ÁREA não vota na normal dos vértices dela.
///
/// ⚠️ **O vértice que discrimina não é o óbvio.** No `collapsed_tetra` as duas
/// faces boas são exatamente opostas e se cancelam, então em `v0`/`v1` o
/// resultado é `[0,1,0]` **com ou sem** a cura — um gate ancorado ali passaria
/// nos dois mundos. Quem separa é `v2`/`v3`, cujo anel tem UMA face com área e
/// duas degeneradas: a normal do vértice tem de ser a daquela face, e antes da
/// cura ela vinha **37,16° inclinada para `+Y`** pelos dois votos fabricados
/// (medido: `[0,214, 0,953, −0,214]` contra `[0,577, 0,577, −0,577]`).
///
/// A asserção é escrita como PROPRIEDADE — *"anel com exatamente uma face de
/// área ⇒ a normal do vértice é a dela"* — e não como o par de índices, para não
/// virar um espelho da fixture.
#[test]
fn a_zero_area_face_does_not_vote_on_its_vertices_normal() {
    let m = crate::shapes_open::collapsed_tetra();
    // ⚠️ **A ÁREA sai do Newell CRU, e nunca do `face_normal`.** Perguntar a
    // degenerescência à função sob teste é um oráculo auto-referente: com a cura
    // revertida ela devolve um unitário, o detector responde *"não há face
    // degenerada"*, e o gate reprova dizendo que a FIXTURE está errada — o
    // diagnóstico apontando para o lugar oposto ao do defeito. (Escrito de novo
    // aqui, e não importado do irmão `shapes_open_tests`: são módulos filhos de
    // pais diferentes, e a alternativa seria promover a área a API de produto
    // com um chamador de teste.)
    let area2 = |f: usize| {
        let vs = m.faces()[f].verts();
        let n = vs.len();
        let mut acc = [0.0f32; 3];
        for i in 0..n {
            let a = m.positions()[vs[i] as usize];
            let b = m.positions()[vs[(i + 1) % n] as usize];
            acc[0] += (a[1] - b[1]) * (a[2] + b[2]);
            acc[1] += (a[2] - b[2]) * (a[0] + b[0]);
            acc[2] += (a[0] - b[0]) * (a[1] + b[1]);
        }
        acc[0] * acc[0] + acc[1] * acc[1] + acc[2] * acc[2]
    };
    let dead: Vec<usize> = (0..m.face_count()).filter(|&f| area2(f) == 0.0).collect();
    assert_eq!(
        dead,
        vec![1, 3],
        "a fixture tinha de trazer exatamente duas faces sem área"
    );

    let mut checked = 0;
    for v in 0..m.vert_count() {
        let ring = m.adjacency().vert_faces.neighbours(v);
        let live: Vec<u32> = ring
            .iter()
            .copied()
            .filter(|&f| area2(f as usize) > 0.0)
            .collect();
        if live.len() != 1 {
            continue;
        }
        checked += 1;
        let want = crate::normals::face_normal(m.positions(), m.faces()[live[0] as usize]);
        let got = m.normals()[v];
        for k in 0..3 {
            assert!(
                (got[k] - want[k]).abs() < 1e-5,
                "vértice {v}: {got:?} contra a única face com área, {want:?} — \
                 as degeneradas votaram"
            );
        }
    }
    assert!(
        checked >= 2,
        "a fixture não contém o caso: nenhum vértice com uma só face de área"
    );

    // A outra metade, e ela é o motivo de o fallback ser do CHAMADOR: mesmo onde
    // o anel não soma direção nenhuma — em `v0`/`v1` as faces boas se cancelam —
    // o vértice ainda tem de sair com um vetor UNITÁRIO. Um zero aqui viraria
    // `NaN` na primeira normalização a jusante, e a tela não diria de onde veio.
    for (v, n) in m.normals().iter().enumerate() {
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        assert!(
            (len - 1.0).abs() < 1e-5,
            "a normal do vértice {v} não é unitária: {n:?}"
        );
    }
}

/// As normais de uma esfera unitária são a própria posição normalizada. É o
/// oráculo ANALÍTICO — independente do código, ao contrário de comparar a saída
/// com ela mesma.
#[test]
fn a_unit_spheres_normals_are_its_own_positions() {
    let m = shapes::uv_sphere(24, 32, 1.0);
    for (v, n) in m.normals().iter().enumerate() {
        let p = m.positions()[v];
        for k in 0..3 {
            assert!(
                (n[k] - p[k]).abs() < 0.02,
                "vértice {v}: normal {n:?} contra posição {p:?}"
            );
        }
    }
}

/// `rebuild` é idempotente — chamá-lo duas vezes dá o mesmo resultado. Sem
/// isso, um dab que reconstrói derivaria a cada movimento do mouse.
#[test]
fn rebuild_is_idempotent() {
    let mut m = shapes::uv_sphere(10, 14, 1.0);
    let n0 = m.normals().to_vec();
    let a0 = m.adjacency().clone();
    let nodes0 = m.octree().node_count();
    m.rebuild();
    assert_eq!(m.normals(), &n0[..]);
    assert_eq!(m.adjacency(), &a0);
    assert_eq!(m.octree().node_count(), nodes0);
}

/// ⚠️ **O oráculo do `refresh_region` é o `rebuild`.** O passe por-região é uma
/// otimização de um resultado que já tem uma resposta certa e cara; provar que
/// ele "atualiza alguma coisa" não diz nada. O que importa é que a malha
/// resultante seja **byte-idêntica** à que a reconstrução inteira produziria.
#[test]
fn a_region_refresh_gives_exactly_what_a_full_rebuild_would() {
    let mut m = shapes::uv_sphere(16, 22, 1.0);
    let mut scratch = RegionScratch::default();

    // Desloca uma calota — verts com y alto.
    let moved: Vec<u32> = (0..m.vert_count() as u32)
        .filter(|&v| m.positions()[v as usize][1] > 0.6)
        .collect();
    assert!(
        moved.len() > 10,
        "a fixture precisa mover um pedaço de verdade"
    );
    for &v in &moved {
        let n = m.normals()[v as usize];
        let p = &mut m.positions_mut()[v as usize];
        for k in 0..3 {
            p[k] += n[k] * 0.25;
        }
    }

    m.refresh_region(&moved, &mut scratch);
    let after_region = (m.normals().to_vec(), m.face_normals().to_vec());

    m.rebuild();
    assert_eq!(
        after_region.0,
        m.normals(),
        "as normais de vértice divergiram da reconstrução completa"
    );
    assert_eq!(after_region.1, m.face_normals());
}

/// ⚠️ **O gate que pega o erro tentador:** atualizar só os vértices que se
/// MOVERAM. O vizinho parado ao lado de uma face que girou tem a normal
/// mudada, e esquecê-lo deixa uma costura na borda exata do pincel.
#[test]
fn a_region_refresh_also_fixes_the_neighbours_that_did_not_move() {
    let mut m = shapes::uv_sphere(14, 18, 1.0);
    let mut scratch = RegionScratch::default();
    let moved = vec![0u32]; // só o polo norte
    let before = m.normals().to_vec();

    let n = m.normals()[0];
    let p = &mut m.positions_mut()[0];
    for k in 0..3 {
        p[k] += n[k] * 0.4;
    }
    m.refresh_region(&moved, &mut scratch);

    let ring: Vec<u32> = m.adjacency().vert_verts.neighbours(0).to_vec();
    assert!(!ring.is_empty());
    let changed = ring
        .iter()
        .filter(|&&v| m.normals()[v as usize] != before[v as usize])
        .count();
    assert_eq!(
        changed,
        ring.len(),
        "todo vizinho do vértice movido devia ter a normal atualizada"
    );
}

/// O scratch é reusado entre dabs, e limpar só o que sujou é o que mantém o
/// passe limitado pela pegada. Duas passagens seguidas dão o mesmo resultado.
#[test]
fn the_region_scratch_is_clean_between_dabs() {
    let mut m = shapes::uv_sphere(12, 16, 1.0);
    let mut scratch = RegionScratch::default();
    let a = vec![0u32];
    let b = vec![m.vert_count() as u32 - 1];
    m.refresh_region(&a, &mut scratch);
    let once = m.normals().to_vec();
    m.refresh_region(&b, &mut scratch);
    m.refresh_region(&a, &mut scratch);
    assert_eq!(m.normals(), &once[..], "o scratch vazou entre passagens");
    assert!(scratch.capacity_bytes() > 0);
}

/// A consulta de esfera contra a **força bruta**, que é um oráculo
/// independente: ela não sabe que existe um octree.
#[test]
fn the_sphere_query_agrees_with_brute_force() {
    let m = shapes::uv_sphere(18, 24, 1.0);
    let mut scratch = QueryScratch::default();
    let mut got = Vec::new();
    for (center, radius) in [
        ([0.0, 1.0, 0.0], 0.5),  // o polo
        ([1.0, 0.0, 0.0], 0.3),  // o equador
        ([0.7, 0.7, 0.0], 0.25), // entre os dois
        ([0.0, 0.0, 0.0], 2.0),  // engole tudo
        ([5.0, 5.0, 5.0], 0.1),  // não pega nada
    ] {
        m.verts_in_sphere(center, radius, &mut scratch, &mut got);
        got.sort_unstable();

        let r2 = radius * radius;
        let mut want: Vec<u32> = (0..m.vert_count() as u32)
            .filter(|&v| {
                let p = m.positions()[v as usize];
                let d = [p[0] - center[0], p[1] - center[1], p[2] - center[2]];
                d[0] * d[0] + d[1] * d[1] + d[2] * d[2] <= r2
            })
            .collect();
        want.sort_unstable();

        assert_eq!(got, want, "consulta em {center:?} r={radius}");
    }
}

/// O scratch é reusado entre consultas, e a época tem de isolar uma da outra.
/// Uma época que não avança faz a segunda consulta devolver vazio.
#[test]
fn a_reused_scratch_does_not_leak_between_queries() {
    let m = shapes::uv_sphere(10, 12, 1.0);
    let mut scratch = QueryScratch::default();
    let mut a = Vec::new();
    let mut b = Vec::new();
    m.verts_in_sphere([0.0, 1.0, 0.0], 0.6, &mut scratch, &mut a);
    m.verts_in_sphere([0.0, 1.0, 0.0], 0.6, &mut scratch, &mut b);
    assert!(!a.is_empty());
    assert_eq!(a, b, "a mesma consulta duas vezes tem de dar o mesmo");
    assert!(scratch.capacity_bytes() > 0);
}

/// ⚠️ **Limite MEDIDO e documentado, não um bug:** a consulta acha vértices
/// através das FACES, então um vértice sem face nenhuma é invisível para ela.
/// É a resposta certa para escultura (não há superfície a mover), e está aqui
/// para ninguém "consertar" isso e pagar uma varredura linear por dab.
#[test]
fn a_faceless_vertex_is_invisible_to_the_sphere_query() {
    let m = Mesh::from_parts(
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.1, 0.1, 0.0],
        ],
        vec![Face::tri(0, 1, 2)],
    )
    .unwrap();
    let mut scratch = QueryScratch::default();
    let mut out = Vec::new();
    m.verts_in_sphere([0.1, 0.1, 0.0], 0.05, &mut scratch, &mut out);
    assert!(
        out.is_empty(),
        "o vértice solto 3 não devia aparecer: {out:?}"
    );
}

/// Cor e máscara são preguiçosas: não existem até alguém escrever nelas.
#[test]
fn colour_and_mask_are_not_allocated_until_touched() {
    let mut m = shapes::cube(1.0);
    assert!(m.colors().is_none());
    assert!(m.masks().is_none());
    m.colors_mut()[0] = [1.0, 0.0, 0.0];
    assert_eq!(m.colors().unwrap().len(), m.vert_count());
    assert!(
        m.masks().is_none(),
        "tocar a cor não pode materializar a máscara"
    );
    m.masks_mut()[1] = 1.0;
    assert_eq!(m.masks().unwrap()[1], 1.0);
    assert_eq!(m.masks().unwrap()[0], DEFAULT_MASK);
}

/// Um índice fora de alcance é recusado na porta. Sem isto, ele vira leitura
/// errada em cada kernel, e o sintoma aparece a três waves de distância.
#[test]
fn an_out_of_range_index_is_refused_at_the_door() {
    let e = Mesh::from_parts(
        vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        vec![Face::tri(0, 1, 9)],
    )
    .unwrap_err();
    assert_eq!(
        e,
        MeshError::VertexOutOfRange {
            face: 0,
            vertex: 9,
            vert_count: 3
        }
    );
}

/// O sentinela de triângulo NÃO conta como índice fora de alcance — se
/// contasse, nenhuma malha de triângulos poderia ser construída.
#[test]
fn the_triangle_sentinel_is_not_mistaken_for_an_index() {
    let m = Mesh::from_parts(
        vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        vec![Face::tri(0, 1, 2)],
    );
    assert!(m.is_ok());
}

/// Deforma a malha empurrando um bloco de vértices para longe, sem pedir
/// permissão ao índice — é o que um dab forte faz.
fn shove(mesh: &mut Mesh, center: [f32; 3], radius: f32, push: f32) -> Vec<u32> {
    let mut moved = Vec::new();
    for v in 0..mesh.vert_count() {
        let p = mesh.positions()[v];
        let d = [p[0] - center[0], p[1] - center[1], p[2] - center[2]];
        if d[0] * d[0] + d[1] * d[1] + d[2] * d[2] <= radius * radius {
            let n = mesh.normals()[v];
            let out = &mut mesh.positions_mut()[v];
            for k in 0..3 {
                out[k] += n[k] * push;
            }
            moved.push(v as u32);
        }
    }
    moved
}

#[test]
fn the_query_still_finds_everything_after_the_surface_moved() {
    // O preço da frase que a W1 escreveu (*"o octree descreve as posições
    // anteriores; enquanto o dab move menos que a folga isso é invisível"*): um
    // empurrão FORTE tira o vértice da caixa da folha dele, ele some da consulta
    // e o pincel deixa um BURACO — sem erro, sem aviso, e sem nada que ligue o
    // sintoma ao índice. A fixture empurra o suficiente para sair.
    let mut mesh = shapes::uv_sphere(40, 56, 1.0);
    let center = [0.0, 0.0, 1.0];
    let moved = shove(&mut mesh, center, 0.35, 0.45);
    assert!(moved.len() > 20, "a fixture mal empurrou ({})", moved.len());
    let mut region = RegionScratch::default();
    mesh.refresh_region(&moved, &mut region);

    // Consulta na superfície NOVA, contra a força bruta — que não sabe que
    // existe um octree.
    let probe = [0.0, 0.0, 1.40];
    let radius = 0.25;
    let mut scratch = QueryScratch::default();
    let mut got = Vec::new();
    mesh.verts_in_sphere(probe, radius, &mut scratch, &mut got);
    got.sort_unstable();

    let mut want: Vec<u32> = (0..mesh.vert_count() as u32)
        .filter(|&v| {
            let p = mesh.positions()[v as usize];
            let d = [p[0] - probe[0], p[1] - probe[1], p[2] - probe[2]];
            d[0] * d[0] + d[1] * d[1] + d[2] * d[2] <= radius * radius
        })
        .collect();
    want.sort_unstable();
    assert!(want.len() > 10, "a sonda não pegou nada ({})", want.len());
    assert_eq!(got, want, "o índice perdeu geometria que se moveu");
}

#[test]
fn a_refitted_tree_answers_exactly_what_a_rebuilt_one_answers() {
    // O refit não é só "não perder": ele deixa as caixas TÃO justas quanto uma
    // reconstrução deixaria. Uma caixa grande demais é conservadora — responde
    // certo e visita à toa —, e sem este gate a diferença entre as duas ficaria
    // invisível até alguém cronometrar a consulta numa malha grande.
    let mut refitted = shapes::uv_sphere(20, 28, 1.0);
    let moved = shove(&mut refitted, [0.4, 0.0, 0.9], 0.5, 0.3);
    let mut region = RegionScratch::default();
    refitted.refresh_region(&moved, &mut region);

    let mut rebuilt = refitted.clone();
    rebuilt.rebuild();

    let mut sa = QueryScratch::default();
    let mut sb = QueryScratch::default();
    let (mut a, mut b) = (Vec::new(), Vec::new());
    let mut nonempty = 0;
    for k in 0..40 {
        let t = k as f32 / 39.0;
        let probe = [-1.2 + 2.6 * t, (t * 6.0).sin() * 0.6, 0.4 + t * 0.9];
        refitted.verts_in_sphere(probe, 0.3, &mut sa, &mut a);
        rebuilt.verts_in_sphere(probe, 0.3, &mut sb, &mut b);
        a.sort_unstable();
        b.sort_unstable();
        assert_eq!(a, b, "sonda {k} em {probe:?}");
        if !a.is_empty() {
            nonempty += 1;
        }
    }
    assert!(nonempty > 10, "só {nonempty} sondas acertaram a malha");
    assert_eq!(
        refitted.bounds(),
        rebuilt.bounds(),
        "a caixa do mundo divergiu do que uma reconstrução daria"
    );
}

/// **A origem local vai para o centro da caixa** — e o oráculo é o ESPELHO, que
/// é o mecanismo que exige isto.
///
/// ⚠️ Um gate que só medisse `bounds().center() == 0` estaria testando a
/// aritmética. O que quebra sem `recenter` é o gesto: uma malha a dez unidades
/// do zero reflete em torno de um plano que não passa por ela, e a cópia
/// espelhada sai longe do modelo.
#[test]
fn recentering_puts_the_mirror_plane_through_the_model() {
    // Um triângulo inteiramente à direita do zero — o caso do arquivo que o
    // autor modelou fora da origem.
    let mut m = Mesh::from_parts(
        vec![[10.0, 0.0, 0.0], [12.0, 0.0, 0.0], [10.0, 2.0, 0.0]],
        vec![Face::tri(0, 1, 2)],
    )
    .expect("malha");
    let far = m.bounds().center()[0];
    assert!(far > 5.0, "a premissa: o modelo está longe do zero ({far})");

    let removed = m.recenter();

    assert_eq!(
        removed,
        [far, 1.0, 0.0],
        "o deslocamento retirado é devolvido"
    );
    let c = m.bounds().center();
    assert!(
        c[0].abs() < 1e-6 && c[1].abs() < 1e-6,
        "o centro tem de ficar na origem, e ficou em {c:?}"
    );
    // O que o espelho vê: a cópia refletida cai SOBRE o modelo, não a 20
    // unidades dele.
    let x: Vec<f32> = m.positions().iter().map(|p| p[0]).collect();
    let span =
        x.iter().cloned().fold(f32::MIN, f32::max) - x.iter().cloned().fold(f32::MAX, f32::min);
    let mirrored_gap = x.iter().cloned().fold(f32::MAX, f32::min).abs() * 2.0;
    assert!(
        mirrored_gap <= span,
        "a metade espelhada tem de encostar no modelo: vão {mirrored_gap}, largura {span}"
    );
}

/// **Uma malha já centrada não é reescrita** — `recenter` devolve zero e não
/// toca um vértice.
#[test]
fn recentering_a_centred_mesh_is_a_no_op() {
    let mut m = Mesh::from_parts(
        vec![[-1.0, -0.5, 0.0], [1.0, -0.5, 0.0], [0.0, 0.5, 0.0]],
        vec![Face::tri(0, 1, 2)],
    )
    .expect("malha");
    let before = m.positions().to_vec();
    assert_eq!(m.recenter(), [0.0, 0.0, 0.0]);
    assert_eq!(m.positions(), before.as_slice());
}
