//! Os gates de CPU do primitivo de malha — a costura, a conversão e a marca.

use super::*;

fn v(i: u32) -> QuadVertex {
    #[expect(clippy::cast_precision_loss, reason = "indices pequenos de fixtura")]
    let f = i as f32;
    QuadVertex {
        pos: [f, f * 2.0 + 1.0],
        uv: [f * 0.25, 1.0 - f * 0.125],
    }
}

/// ⭐⭐ **`N` triângulos dão `5N − 2` vértices, e as janelas NÃO degeneradas são exactamente os
/// triângulos de entrada, pela ordem.**
///
/// ⚠️ É a lei que deixa a malha passar pelas pipelines `TriangleStrip` de sempre: um degenerado a
/// menos cola dois triângulos com um terceiro que atravessa a malha; um a mais desalinha a paridade
/// (inofensiva sem culling, mas o número deixa de ser `5N − 2`).
#[test]
fn stitching_n_triangles_gives_5n_minus_2_vertices_and_the_real_windows_are_the_triangles() {
    for n in [1_u32, 2, 5] {
        let verts: Vec<QuadVertex> = (0..3 * n).map(v).collect();
        let tris: Vec<[u32; 3]> = (0..n).map(|t| [3 * t, 3 * t + 1, 3 * t + 2]).collect();
        let mut out = Vec::new();
        let (a, b) = stitch(&mut out, &verts, &tris).expect("ha' triangulos validos");
        assert_eq!((a, b), (0, 5 * n - 2), "n = {n}");
        let reais: Vec<[QuadVertex; 3]> = out
            .windows(3)
            .filter(|w| w[0].pos != w[1].pos && w[1].pos != w[2].pos && w[0].pos != w[2].pos)
            .map(|w| [w[0], w[1], w[2]])
            .collect();
        assert_eq!(reais.len(), n as usize, "n = {n}: janelas reais");
        for (t, janela) in tris.iter().zip(&reais) {
            let esperado = [
                verts[t[0] as usize],
                verts[t[1] as usize],
                verts[t[2] as usize],
            ];
            assert_eq!(
                janela.map(|q| q.pos),
                esperado.map(|q| q.pos),
                "n = {n}: a janela nao e' o triangulo"
            );
        }
    }
}

/// Um triângulo com um índice fora da malha é SALTADO (nunca desenhado com lixo), e uma lista sem
/// nenhum válido não produz intervalo.
#[test]
fn a_triangle_with_an_index_out_of_range_is_skipped() {
    let verts: Vec<QuadVertex> = (0..3).map(v).collect();
    let mut out = Vec::new();
    assert_eq!(stitch(&mut out, &verts, &[[0, 1, 9]]), None);
    assert!(out.is_empty());
    let r = stitch(&mut out, &verts, &[[0, 1, 9], [0, 1, 2]]).expect("o 2.o e' valido");
    assert_eq!(
        r,
        (0, 3),
        "so' o triangulo valido entra, sem ligacao a um fantasma"
    );
}

/// ⭐ **A volta `local → quad_pos → anchor + quad_pos·size` devolve o `local`**, com a âncora
/// DESLOCADA (sprite não centrada / com offset) — a lei que o shader aplica a cada vértice.
#[test]
fn quad_pos_round_trips_through_the_shader_law_with_an_offset_anchor() {
    let (anchor, size) = ([0.75_f32, -1.25], [3.0_f32, 2.0]);
    for local in [[0.0_f32, 0.0], [2.25, -2.25], [-0.75, -0.25], [0.1, 0.3]] {
        let p = quad_pos(local, anchor, size).expect("size valido");
        let volta = [anchor[0] + p[0] * size[0], anchor[1] + p[1] * size[1]];
        assert!(
            (volta[0] - local[0]).abs() < 1e-5 && (volta[1] - local[1]).abs() < 1e-5,
            "{local:?} -> {p:?} -> {volta:?}"
        );
    }
    assert_eq!(quad_pos([0.0, 0.0], anchor, [0.0, 1.0]), None);
    assert_eq!(quad_pos([0.0, 0.0], anchor, [1.0, f32::NAN]), None);
}

/// ⭐⭐ **`SpriteMesh::uv_at` dá a cada canto do QUAD a UV que o quad lhe dá** — com a âncora deslocada.
///
/// ⚠️ É o que faz uma malha em repouso ler os mesmos texels que o quad: a UV sai do ponto onde o
/// vértice está, e o gate confere-a contra a PRÓPRIA tabela do quad (`QuadVertex::QUAD_STRIP`), não
/// contra uma convenção de `v` escrita à mão aqui.
///
/// (Mutação: `0.5 + q[1]` no `uv_at` ⇒ RED nos quatro cantos.)
#[test]
fn uv_at_gives_every_quad_corner_the_uv_the_quad_gives_it() {
    let (anchor, size) = ([0.75_f32, -1.25], [3.0_f32, 2.0]);
    for canto in QuadVertex::QUAD_STRIP {
        let local = [
            anchor[0] + canto.pos[0] * size[0],
            anchor[1] + canto.pos[1] * size[1],
        ];
        let uv = SpriteMesh::uv_at(local, anchor, size).expect("size valido");
        assert!(
            (uv[0] - canto.uv[0]).abs() < 1e-6 && (uv[1] - canto.uv[1]).abs() < 1e-6,
            "o canto {:?} deu a UV {uv:?} e o quad usa {:?}",
            canto.pos,
            canto.uv
        );
    }
    assert_eq!(SpriteMesh::uv_at([0.0, 0.0], anchor, [0.0, 1.0]), None);
}

fn instancia() -> RenderInstance {
    let mut i: RenderInstance = bytemuck::Zeroable::zeroed();
    i.size = [2.0, 2.0];
    i.anchor = [0.5, 0.0];
    i
}

fn quadrado() -> SpriteMesh {
    SpriteMesh {
        local: vec![[-0.5, -1.0], [1.5, -1.0], [1.5, 1.0], [-0.5, 1.0]],
        uv: vec![[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
        tris: vec![[0, 1, 2], [0, 2, 3]],
    }
}

/// ⭐⭐ **A recolha marca a instância que tem malha, deixa a outra no quad, e LIMPA a marca de uma
/// instância que chega de fora** (`extra`) — que indexaria a malha de outra chamada de render.
///
/// ⭐⭐⭐ **E uma de fora QUE TRAZ MALHA é marcada** (plano `docs/Skeleton/03`, W7): é assim que um
/// fantasma do onion de uma imagem presa desenha a arte DEFORMADA em `t ± k` em vez do quad de
/// repouso. ⚠️ A marca dela indexa o MESMO `MeshFrame` da cena, com o índice deslocado — por isso o
/// gate conta **duas** faixas e lê a marca `2` na de fora.
#[test]
fn the_collect_tags_the_meshed_instance_and_clears_a_tag_from_outside() {
    let mut present = ph2d_ecs::PresentWorld::new();
    let mut simples = instancia();
    simples.z_order = 0;
    present.world_mut().spawn(simples);
    let mut com_malha = instancia();
    com_malha.z_order = 1;
    present.world_mut().spawn((com_malha, quadrado()));

    let mut de_fora = instancia();
    de_fora.z_order = 2;
    de_fora.flip_uv = RenderInstance::FLIP_X_BIT | (7 << RenderInstance::MESH_SHIFT);
    let mut fantasma = instancia();
    fantasma.z_order = 3;

    let mut extra = crate::LiftedInstances::default();
    extra.push(de_fora, None);
    extra.push(fantasma, Some(&quadrado()));

    let mut scratch = Vec::new();
    let mut malhas = MeshFrame::default();
    crate::sprite_collect::collect_sorted_instances(
        &mut scratch,
        &mut malhas,
        &mut present,
        &extra,
        None,
        None,
    );
    let marcas: Vec<u32> = scratch
        .iter()
        .map(|i| RenderInstance::unpack_mesh(i.flip_uv))
        .collect();
    assert_eq!(
        marcas,
        vec![0, 1, 0, 2],
        "quad · malha da cena · a de fora LIMPA · a de fora COM malha"
    );
    assert_eq!(
        scratch[2].flip_uv,
        RenderInstance::FLIP_X_BIT,
        "limpar a marca nao pode tocar nos outros bits"
    );
    assert_eq!(
        malhas.ranges,
        vec![(0, 5 * 2 - 2), (5 * 2 - 2, 2 * (5 * 2 - 2))],
        "as duas metades do quadro partilham a costura, em sequencia"
    );
    // A malha em repouso cobre o quad: os cantos voltam aos cantos do QUAD_STRIP.
    let cantos: Vec<[f32; 2]> = malhas.vertices.iter().map(|q| q.pos).collect();
    for canto in [[-0.5_f32, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]] {
        assert!(
            cantos.contains(&canto),
            "o canto {canto:?} do quad nao esta' na malha"
        );
    }
}

/// Uma malha que não pode ser desenhada deixa a instância no QUAD e não deixa vértices para trás.
#[test]
fn a_mesh_that_cannot_be_drawn_leaves_the_quad_and_no_vertices() {
    let mut malhas = MeshFrame::default();
    let mut torta = quadrado();
    torta.uv.pop();
    assert_eq!(
        malhas.push(&torta, [0.0, 0.0], [1.0, 1.0]),
        0,
        "comprimentos diferentes"
    );
    assert_eq!(
        malhas.push(&quadrado(), [0.0, 0.0], [0.0, 1.0]),
        0,
        "size nulo"
    );
    let mut sem_triangulo = quadrado();
    sem_triangulo.tris = vec![[0, 1, 99]];
    assert_eq!(
        malhas.push(&sem_triangulo, [0.0, 0.0], [1.0, 1.0]),
        0,
        "nenhum valido"
    );
    assert!(malhas.vertices.is_empty() && malhas.ranges.is_empty());
}

fn triangulo(local: Vec<[f32; 2]>) -> SpriteMesh {
    SpriteMesh {
        local,
        uv: vec![[0.0, 1.0], [1.0, 1.0], [0.0, 0.0]],
        tris: vec![[0, 1, 2]],
    }
}

/// ⭐⭐⭐ **Uma instância COPIADA leva a malha que tinha, e um conjunto REUSADO nunca devolve a malha
/// do quadro anterior.**
///
/// ⛔ O defeito que isto fecha (plano 03, W3): o vidro do prefab e o emissivo copiavam só o
/// `RenderInstance` e desenhavam o quad de repouso de uma imagem presa ao esqueleto. ⚠️ E a metade do
/// reuso: o conjunto guarda as malhas em buffers que sobrevivem ao `clear`, e um slot velho lido
/// depois de um `clear` desenharia a pose de um quadro que já passou.
///
/// (Mutações: o `push` a ignorar a malha ⇒ RED; o `clear` sem `n_meshes = 0` ⇒ RED na metade do
/// reuso.)
#[test]
fn a_lifted_instance_carries_its_mesh_and_reuse_never_returns_last_frames_mesh() {
    let mut conjunto = LiftedInstances::default();
    let velha = triangulo(vec![[0.0, 0.0], [9.0, 0.0], [0.0, 9.0]]);
    conjunto.push(instancia(), Some(&velha));
    conjunto.clear();
    let nova = triangulo(vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]);
    conjunto.push(instancia(), None);
    conjunto.push(instancia(), Some(&nova));
    assert_eq!(conjunto.len(), 2);
    assert_eq!(
        conjunto.mesh_of(0),
        None,
        "a instancia sem malha herdou a malha do quadro anterior"
    );
    assert_eq!(
        conjunto.mesh_of(1),
        Some(&nova),
        "a instancia com malha nao a levou, ou levou a do quadro anterior"
    );
    assert_eq!(conjunto.meshes().len(), 1);
}

/// ⭐⭐ **A porta dos consumidores recolhe o que `keep` aceita — COM a malha, e com a cópia alterada.**
///
/// ⚠️ É a porta por onde o vidro do prefab e o emissivo passam: um consumidor que a use não pode
/// esquecer a malha, e a alteração que ele faz (o emissivo multiplica a tinta) chega à cópia.
#[test]
fn collect_from_lifts_what_keep_accepts_with_its_mesh() {
    let mut sim = ph2d_ecs::SimWorld::default();
    let sobe = sim.world_mut().spawn_empty().id();
    let fica = sim.world_mut().spawn_empty().id();
    let mut present = ph2d_ecs::PresentWorld::new();
    let malha = quadrado();
    present
        .world_mut()
        .spawn((ph2d_ecs::SimRef(sobe), instancia(), malha.clone()));
    present
        .world_mut()
        .spawn((ph2d_ecs::SimRef(fica), instancia(), quadrado()));
    let mut conjunto = LiftedInstances::default();
    conjunto.push(instancia(), None);
    conjunto.collect_from(&mut present, |e, inst| {
        inst.opacity = 0.25;
        e == sobe
    });
    assert_eq!(
        conjunto.len(),
        1,
        "so' a entidade aceite sobe, e o lixo do quadro anterior sai"
    );
    assert_eq!(
        conjunto.instances()[0].opacity,
        0.25,
        "a alteracao do keep nao chegou a' copia"
    );
    assert_eq!(
        conjunto.mesh_of(0),
        Some(&malha),
        "a instancia levantada perdeu a malha"
    );
}

/// ⭐⭐ **Uma fatia levantada desenha-se com as malhas dela; uma fatia CRUA limpa toda marca.**
///
/// (Mutação: o `tag_lifted` sem o laço das malhas ⇒ RED; sem o laço que limpa ⇒ RED na marca herdada.)
#[test]
fn tag_lifted_marks_each_lifted_mesh_and_a_raw_slice_carries_none() {
    let mut conjunto = LiftedInstances::default();
    conjunto.push(instancia(), None);
    conjunto.push(instancia(), Some(&quadrado()));
    let mut scratch = conjunto.instances().to_vec();
    scratch[0].flip_uv |= 5 << RenderInstance::MESH_SHIFT;
    let mut frame = MeshFrame::default();
    tag_lifted(&mut scratch, &mut frame, conjunto.meshes());
    let marcas: Vec<u32> = scratch
        .iter()
        .map(|i| RenderInstance::unpack_mesh(i.flip_uv))
        .collect();
    assert_eq!(
        marcas,
        vec![0, 1],
        "sem malha · com a malha dela (a marca herdada saiu)"
    );
    assert_eq!(frame.ranges, vec![(0, 5 * 2 - 2)]);
    tag_lifted(&mut scratch, &mut frame, &[]);
    assert!(
        scratch
            .iter()
            .all(|i| RenderInstance::unpack_mesh(i.flip_uv) == 0)
            && frame.ranges.is_empty(),
        "uma fatia crua levou uma marca de malha"
    );
}

/// ⭐⭐⭐ **O ponto sobre a malha POSADA lê a UV de REPOUSO debaixo dele; fora dela, nada — e a malha
/// que o passe recusaria não é malha para ninguém.**
///
/// ⚠️ A malha da fixtura está POSADA para fora do quad de repouso (um triângulo em `x = 3..5` de um
/// quad `2×2` centrado): é o caso em que o quad e a malha discordam, e o único que distingue quem lê
/// qual.
///
/// (Mutação: o `uv_under` a devolver a UV do 1.º vértice em vez da interpolada ⇒ RED.)
#[test]
fn a_point_on_the_posed_mesh_reads_the_rest_uv_under_it() {
    let m = triangulo(vec![[3.0, 0.0], [5.0, 0.0], [3.0, 2.0]]);
    let uv = uv_under(&m, [4.0, 0.0]).expect("o meio da base esta' na malha");
    assert!(
        (uv[0] - 0.5).abs() < 1e-6 && (uv[1] - 1.0).abs() < 1e-6,
        "o meio da base leu {uv:?} — a UV de repouso ali e' (0,5 · 1)"
    );
    assert!(
        covers(&m, [3.0, 2.0]),
        "um vertice pertence a' malha (a borda conta)"
    );
    assert!(
        !covers(&m, [0.0, 0.0]) && uv_under(&m, [0.0, 0.0]).is_none(),
        "o centro do quad de REPOUSO nao e' malha posada"
    );
    assert!(drawn_mesh(Some(&m), [2.0, 2.0]).is_some());
    let mut torta = m.clone();
    torta.uv.pop();
    assert!(
        drawn_mesh(Some(&torta), [2.0, 2.0]).is_none(),
        "uma malha que o passe recusa nao pode ser apontavel"
    );
    assert!(
        drawn_mesh(Some(&m), [0.0, 2.0]).is_none(),
        "com size nulo o passe desenha o quad, nao a malha"
    );
}
