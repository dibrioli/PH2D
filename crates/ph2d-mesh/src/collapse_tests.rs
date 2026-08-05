//! **OS GATES DO COLAPSO.**
//!
//! O assunto é o mesmo dos gates da porta que CRESCE, do outro lado: *o que uma
//! mudança de topologia em EDIÇÃO tem de preservar em relação a uma
//! reconstrução*, mais as quatro recusas que são TOPOLOGIA e não zelo.

use super::*;
use crate::{Face, Mesh, dyntopo, shapes};

fn scratch() -> RegionScratch {
    RegionScratch::default()
}

fn tri_sphere(rings: usize, segs: usize) -> Mesh {
    let mut m = shapes::uv_sphere(rings, segs, 1.0);
    m.triangulate();
    m
}

/// O comprimento médio de aresta — a régua com que as fixtures escolhem um
/// limiar que de fato dispara.
fn mean_edge(m: &Mesh) -> f32 {
    let pos = m.positions();
    let mut sum = 0.0f32;
    let mut n = 0usize;
    for f in m.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (pos[v[k] as usize], pos[v[(k + 1) % v.len()] as usize]);
            let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            sum += (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            n += 1;
        }
    }
    sum / n.max(1) as f32
}

/// Quantas arestas do interior ficaram com valência ≠ 2 — **zero é o contrato**
/// de uma malha fechada. A mesma régua do gate irmão do refino.
fn cracks(m: &Mesh) -> usize {
    let e = m.edges();
    (0..e.len() as u32).filter(|&i| e.valence(i) != 2).count()
}

/// Toda face cita um vértice que existe? O modo de falha de uma renumeração
/// errada é um índice **válido** que é de outro vértice, então este é o piso e
/// não o teto — mas sem ele nem o piso está.
fn indices_in_range(m: &Mesh) -> bool {
    let n = m.vert_count() as u32;
    m.faces().iter().all(|f| f.verts().iter().all(|&v| v < n))
}

/// Uma grade `n × n` triangulada — a fixture aberta com INTERIOR de verdade.
///
/// ⚠️ **O `open_tube3` não serve, e o gate ficou verde-sobre-nada até esta
/// função existir:** ele tem três faces, todo vértice é de beira ou de valência
/// 3, e o colapso recusava por OUTRAS razões. Uma grade tem beira e miolo, e é
/// no miolo que o colapso de facto dispara — que é o que torna a recusa de beira
/// a única coisa a segurar o contorno.
fn grid(n: usize) -> Mesh {
    let mut pos = Vec::with_capacity((n + 1) * (n + 1));
    for j in 0..=n {
        for i in 0..=n {
            pos.push([i as f32 / n as f32, j as f32 / n as f32, 0.0]);
        }
    }
    let mut faces = Vec::with_capacity(n * n * 2);
    let at = |i: usize, j: usize| (j * (n + 1) + i) as u32;
    for j in 0..n {
        for i in 0..n {
            faces.push(Face::tri(at(i, j), at(i + 1, j), at(i + 1, j + 1)));
            faces.push(Face::tri(at(i, j), at(i + 1, j + 1), at(i, j + 1)));
        }
    }
    Mesh::from_parts(pos, faces).expect("índices válidos")
}

/// Duas faces com o MESMO conjunto de cantos — o que um colapso que ignora a
/// condição de elo produz.
fn duplicate_faces(m: &Mesh) -> usize {
    let mut keys: Vec<[u32; 3]> = m
        .faces()
        .iter()
        .map(|f| {
            let v = f.verts();
            let mut k = [v[0], v[1], v[2]];
            k.sort_unstable();
            k
        })
        .collect();
    keys.sort_unstable();
    let total = keys.len();
    keys.dedup();
    total - keys.len()
}

// ─────────────────────────── o oráculo ───────────────────────────

/// **O ORÁCULO DA EDIÇÃO QUE ENCOLHE** — depois de um colapso de verdade, a
/// malha tem de ser *exatamente* o que uma reconstrução produziria.
///
/// ⚠️ **A comparação inclui o ÚLTIMO BIT das normais**, o mesmo argumento do
/// gate gêmeo do corte: a normal de um vértice é a soma das normais das faces do
/// anel, e somar os mesmos números noutra ordem dá outro `f32`. Uma compactação
/// que "só" reordena um anel passa em todo gate de conteúdo e diverge aqui.
///
/// ⚠️ **E ele roda sobre um colapso que de fato APAGOU alguma coisa** — sem o
/// controle, uma malha que ninguém colapsou é trivialmente igual a si mesma.
#[test]
fn a_collapsed_mesh_is_exactly_what_a_rebuild_would_have_produced() {
    let mut m = tri_sphere(14, 20);
    let (v0, f0) = (m.vert_count(), m.face_count());
    let emin = 1.2 * mean_edge(&m);
    let mut remap = Remap::default();
    let r = collapse_in_sphere(
        &mut m,
        [0.0, 0.0, 1.0],
        0.9,
        emin,
        &mut remap,
        &mut scratch(),
    );
    assert!(
        matches!(r, Collapse::Done { .. }),
        "o controle: a fixture tem de colapsar de verdade, e não colapsou ({r:?})"
    );
    assert!(m.vert_count() < v0 && m.face_count() < f0);
    assert_eq!(
        m.vert_count(),
        remap.verts,
        "o remap mentiu sobre a contagem"
    );
    assert_eq!(m.face_count(), remap.faces);
    assert!(
        indices_in_range(&m),
        "uma face cita um vértice que não existe"
    );

    let rebuilt =
        Mesh::from_parts(m.positions().to_vec(), m.faces().to_vec()).expect("índices válidos");
    let (edited, fresh) = (m.adjacency(), rebuilt.adjacency());
    for v in 0..m.vert_count() {
        assert_eq!(
            edited.vert_faces.neighbours(v),
            fresh.vert_faces.neighbours(v),
            "o anel de faces de {v} divergiu de um rebuild"
        );
        assert_eq!(
            edited.vert_verts.neighbours(v),
            fresh.vert_verts.neighbours(v),
            "o anel de vértices de {v} divergiu de um rebuild"
        );
    }
    assert_eq!(m.face_normals(), rebuilt.face_normals());
    assert_eq!(
        m.normals(),
        rebuilt.normals(),
        "as normais de vértice divergiram — provavelmente a ORDEM de um anel"
    );
}

/// **A MALHA CONTINUA FECHADA.** Um colapso mal costurado deixa uma aresta com
/// uma face só, e o sintoma é um buraco que a luz mostra e o índice não.
#[test]
fn the_collapsed_mesh_has_no_cracks() {
    let mut m = tri_sphere(14, 20);
    assert_eq!(cracks(&m), 0, "o controle: a esfera nasce fechada");
    let emin = 1.2 * mean_edge(&m);
    let mut remap = Remap::default();
    let r = collapse_in_sphere(
        &mut m,
        [0.0, 0.0, 1.0],
        0.9,
        emin,
        &mut remap,
        &mut scratch(),
    );
    assert!(matches!(r, Collapse::Done { .. }), "o controle: {r:?}");
    assert_eq!(cracks(&m), 0, "o colapso abriu a malha");
}

/// **O ÍNDICE ESPACIAL NÃO ESQUECE GEOMETRIA** depois de perder faces.
///
/// ⚠️ **O oráculo é a RESPOSTA, não a árvore** — a mesma lei do gate gêmeo: a
/// partição decide a *forma* da árvore, a caixa frouxa decide a *resposta*. O que
/// tem de valer é que nenhuma face vive escondida do pincel.
#[test]
fn the_collapsed_octree_still_finds_every_face_it_should() {
    let mut m = tri_sphere(14, 20);
    let emin = 1.2 * mean_edge(&m);
    let mut remap = Remap::default();
    let r = collapse_in_sphere(
        &mut m,
        [0.0, 0.0, 1.0],
        0.9,
        emin,
        &mut remap,
        &mut scratch(),
    );
    assert!(matches!(r, Collapse::Done { .. }), "o controle: {r:?}");

    let mut found = Vec::new();
    m.octree()
        .faces_in_sphere([0.0, 0.0, 0.0], 10.0, &mut found);
    found.sort_unstable();
    found.dedup();
    let want: Vec<u32> = (0..m.face_count() as u32).collect();
    assert_eq!(found, want, "o octree perdeu ou duplicou faces");
}

/// **AS FAIXAS DAS FOLHAS CONTINUAM DISJUNTAS** depois de encolherem.
///
/// Sem este gate, duas folhas sobrepostas devolveriam faces reais e o sintoma
/// só apareceria numa face contada duas vezes, muito depois.
#[test]
fn the_leaf_ranges_stay_disjoint_after_faces_die() {
    let mut m = tri_sphere(14, 20);
    let emin = 1.2 * mean_edge(&m);
    let mut remap = Remap::default();
    let r = collapse_in_sphere(
        &mut m,
        [0.0, 0.0, 1.0],
        0.9,
        emin,
        &mut remap,
        &mut scratch(),
    );
    assert!(matches!(r, Collapse::Done { .. }), "o controle: {r:?}");

    let (mut spans, total) = m.octree().leaf_spans_for_gate();
    spans.sort_unstable();
    let mut prev = 0usize;
    for (s, e) in spans {
        assert!(s >= prev, "duas folhas se sobrepõem em {s}..{e}");
        assert!(e <= total, "uma folha aponta para fora de face_indices");
        prev = e;
    }
}

// ─────────────────────────── as recusas ───────────────────────────

/// **A BEIRA NUNCA É PUXADA PARA DENTRO.** Colapsar um vértice de contorno muda
/// a forma do buraco sozinho, e o artista não pediu isso.
///
/// ⚠️ **O controle é a metade que importa:** a fixture tem de ter beira E arestas
/// sob o limiar, senão o gate fica verde porque nada foi tentado.
#[test]
fn a_border_is_never_pulled_in() {
    let mut m = grid(6);
    let border: Vec<usize> = (0..m.vert_count())
        .filter(|&v| m.adjacency().is_border(v))
        .collect();
    assert!(!border.is_empty(), "o controle: a fixture tem de ter beira");
    // ⚠️ **O CONJUNTO, não a lista por índice** — o colapso do miolo renumera, e
    // uma comparação posicional acusaria a compactação em vez do contorno.
    let mut before: Vec<[u32; 3]> = border.iter().map(|&v| bits(m.positions()[v])).collect();
    before.sort_unstable();
    let f0 = m.face_count();

    // Um limiar generoso: TODA aresta desta malha está sob ele.
    let mut remap = Remap::default();
    let _ = collapse_in_sphere(
        &mut m,
        [0.0, 0.0, 0.0],
        100.0,
        100.0,
        &mut remap,
        &mut scratch(),
    );
    assert!(
        m.face_count() < f0,
        "o controle: o MIOLO tem de colapsar, senão o gate não testou nada"
    );
    let mut after: Vec<[u32; 3]> = (0..m.vert_count())
        .filter(|&v| m.adjacency().is_border(v))
        .map(|v| bits(m.positions()[v]))
        .collect();
    after.sort_unstable();
    assert_eq!(after, before, "o contorno mudou de forma ou de tamanho");
}

/// Os bits de uma posição — para ordenar e comparar sem `f32: Ord`.
fn bits(p: [f32; 3]) -> [u32; 3] {
    [p[0].to_bits(), p[1].to_bits(), p[2].to_bits()]
}

/// **UM TETRAEDRO RECUSA**, e a recusa é a (2b) — não a condição de elo.
///
/// ⚠️ **Este gate existe porque a condição de elo PASSA aqui**, e foi ele que a
/// mostrou insuficiente: num tetraedro os anéis de dois vértices compartilham
/// exatamente os dois opostos, então (3) diz sim; o que impede o desastre é os
/// opostos terem valência 3, e um colapso os deixaria com DUAS faces — uma aba,
/// não uma superfície.
#[test]
fn a_tetrahedron_refuses_because_its_opposites_are_valence_three() {
    let mut m = shapes::octahedron(1.0);
    // O octaedro tem valência 4 em todo vértice — ele é o CONTROLE positivo:
    // aqui o colapso pode acontecer.
    let (v0, f0) = (m.vert_count(), m.face_count());
    let mut remap = Remap::default();
    let _ = collapse_in_sphere(
        &mut m,
        [0.0, 0.0, 0.0],
        100.0,
        100.0,
        &mut remap,
        &mut scratch(),
    );
    assert!(
        m.vert_count() < v0 && m.face_count() < f0,
        "o controle: o octaedro tem valência 4 e TEM de poder colapsar"
    );

    // O tetraedro: quatro vértices, todos de valência 3.
    let tetra = Mesh::from_parts(
        vec![
            [1.0, 1.0, 1.0],
            [-1.0, -1.0, 1.0],
            [-1.0, 1.0, -1.0],
            [1.0, -1.0, -1.0],
        ],
        vec![
            Face::tri(0, 1, 2),
            Face::tri(0, 3, 1),
            Face::tri(0, 2, 3),
            Face::tri(1, 3, 2),
        ],
    )
    .expect("índices válidos");
    let mut t = tetra.clone();
    let mut remap = Remap::default();
    let r = collapse_in_sphere(
        &mut t,
        [0.0, 0.0, 0.0],
        100.0,
        100.0,
        &mut remap,
        &mut scratch(),
    );
    assert_eq!(r, Collapse::Enough, "o tetraedro tinha de recusar");
    assert_eq!(t.vert_count(), 4);
    assert_eq!(t.face_count(), 4);
}

/// **A CONDIÇÃO DE ELO RECUSA UM TÚNEL ESTREITO.**
///
/// ⚠️ **A fixture teve de ser CAÇADA, e a caça é o gate.** Num toro `4×4` a
/// condição nunca dispara — os anéis de dois vizinhos compartilham exatamente os
/// dois opostos. Num toro `3×6` o túnel é estreito o bastante para o anel dar a
/// volta e encostar em si mesmo: **18 arestas** violam o elo, com os quatro
/// vértices de valência 6 (ou seja, as recusas de beira e de valência-3 não as
/// alcançam). Sem esta fixture a mutação *"apague a condição de elo"* sobrevivia
/// à suíte inteira.
///
/// O sintoma de ignorá-la não é um pânico: são **duas faces com os mesmos três
/// cantos**, uma malha que ainda desenha e que nenhuma operação seguinte
/// conserta.
#[test]
fn the_link_condition_refuses_a_narrow_tunnel() {
    // O CONTROLE: no toro largo a condição não dispara, e o colapso acontece.
    let mut wide = shapes::torus(4, 4, 1.0, 0.45);
    wide.triangulate();
    let f0 = wide.face_count();
    let mut remap = Remap::default();
    let _ = collapse_in_sphere(
        &mut wide,
        [0.0, 0.0, 0.0],
        100.0,
        100.0,
        &mut remap,
        &mut scratch(),
    );
    assert!(
        wide.face_count() < f0,
        "o controle: o toro largo TEM de colapsar"
    );
    assert_eq!(duplicate_faces(&wide), 0);
    assert_eq!(cracks(&wide), 0);

    // O estreito: o colapso pode acontecer ou não, mas a malha não pode
    // degenerar.
    let mut narrow = shapes::torus(3, 6, 1.0, 0.45);
    narrow.triangulate();
    assert_eq!(duplicate_faces(&narrow), 0, "o controle: ela nasce sã");
    let mut remap = Remap::default();
    let _ = collapse_in_sphere(
        &mut narrow,
        [0.0, 0.0, 0.0],
        100.0,
        100.0,
        &mut remap,
        &mut scratch(),
    );
    assert_eq!(
        duplicate_faces(&narrow),
        0,
        "o colapso costurou a malha em si mesma"
    );
    assert_eq!(cracks(&narrow), 0, "e abriu o túnel");
}

/// **QUAD RECUSA** — a mesma lei do refino, e pelo mesmo motivo: a operação é
/// definida sobre triângulos, e o motor nunca depende de quem chamou ter
/// lembrado de triangular.
#[test]
fn a_quad_mesh_refuses() {
    let mut m = shapes::cube(1.0);
    let mut remap = Remap::default();
    let r = collapse_in_sphere(
        &mut m,
        [0.0, 0.0, 0.0],
        100.0,
        100.0,
        &mut remap,
        &mut scratch(),
    );
    assert_eq!(r, Collapse::NotTriangles);
    assert_eq!(m.face_count(), 6, "a malha não pode ter sido tocada");
}

/// **UM LIMIAR NÃO-POSITIVO É NO-OP**, e não um laço infinito. O irmão exato da
/// recusa do refino.
#[test]
fn a_non_positive_threshold_does_nothing() {
    let mut m = tri_sphere(8, 10);
    let f0 = m.face_count();
    let mut remap = Remap::default();
    for bad in [0.0, -1.0, f32::NAN] {
        let r = collapse_in_sphere(
            &mut m,
            [0.0, 0.0, 1.0],
            0.5,
            bad,
            &mut remap,
            &mut scratch(),
        );
        assert_eq!(r, Collapse::Enough, "limiar {bad} devia ser no-op");
    }
    assert_eq!(m.face_count(), f0);
}

// ─────────────────────────── a lei ───────────────────────────

/// **O PAR REFINO+COLAPSO ASSENTA — E ABAIXO DE DOIS ELE NÃO ASSENTA.**
///
/// ⚠️ **Este gate nasceu afirmando outra coisa, e a medição o corrigiu.** A
/// primeira versão exigia que o colapso removesse ZERO logo depois de um refino,
/// e ela reprovou: o corte 1→2 cria uma MEDIANA, e a mediana de um triângulo
/// fino é curta — a histerese protege a filha de uma aresta partida ao meio, não
/// todo lado que o padrão produz. A pergunta certa não é *"remove zero?"* e sim
/// ***o par tem ponto fixo?***
///
/// Medido (`measure_whether_refine_and_collapse_settle`), contagem de vértices
/// ao longo de 12 ciclos de (refino, colapso) no MESMO lugar:
///
/// | razão | ciclos | assenta? |
/// |---|---|---|
/// | 1,80 | 710 712 700 698 696 700 698 699 695 … | **NÃO** |
/// | 2,00 | 716 721 714 714 714 714 … | sim |
/// | **2,05** | 716 720 716 716 716 716 … | sim |
/// | 2,50 | 721 722 722 722 … | sim |
///
/// O joelho está entre 1,8 e 2,0, e o `2,05` da referência senta logo acima
/// dele. ⚠️ **A metade de baixo é o que torna a de cima não-vazia:** sem ela o
/// gate ficaria verde com qualquer limiar suficientemente pequeno para não
/// colapsar nada.
#[test]
fn the_pair_of_refine_and_collapse_settles_and_below_two_it_does_not() {
    assert!(
        settles(collapse_target(1.0)),
        "o par não assentou com a histerese que SHIPA"
    );
    assert!(
        !settles(1.0 / 1.8),
        "o controle: com 1,8 o par TEM de moer — se ele assenta, a fixture \
         deixou de conter o fenômeno e a metade de cima não prova nada"
    );
}

/// Doze ciclos de (refino, colapso) no mesmo ponto; `true` se os três últimos
/// concordam. `min_over_max` é o limiar de colapso em fração do alvo.
fn settles(min_over_max: f32) -> bool {
    let mut m = tri_sphere(12, 18);
    let (centre, radius) = ([0.0, 0.0, 1.0], 0.6);
    let emax = dyntopo::edge_target(radius, 1.0);
    let mut births = Vec::new();
    let mut remap = Remap::default();
    let mut counts = Vec::new();
    for _ in 0..12 {
        let _ =
            dyntopo::refine_in_sphere(&mut m, centre, radius, emax, &mut births, &mut scratch());
        let _ = collapse_in_sphere(
            &mut m,
            centre,
            radius,
            emax * min_over_max,
            &mut remap,
            &mut scratch(),
        );
        counts.push(m.vert_count());
    }
    let tail = &counts[counts.len() - 3..];
    tail.iter().all(|&c| c == tail[0])
}

/// **AS ARESTAS CURTAS SOMEM E AS LONGAS FICAM** — a lei, medida onde ela vale.
#[test]
fn the_short_edges_go_and_the_long_ones_stay() {
    let mut m = tri_sphere(14, 20);
    let emin = 1.2 * mean_edge(&m);
    let (centre, radius) = ([0.0, 0.0, 1.0], 0.9);
    let short_before = short_edges_in(&m, centre, radius, emin);
    let long_before = long_edges_in(&m, centre, radius, emin);
    assert!(short_before > 0, "o controle: a fixture tem arestas curtas");
    assert!(long_before > 0, "o controle: e também tem longas");

    let mut remap = Remap::default();
    let r = collapse_in_sphere(&mut m, centre, radius, emin, &mut remap, &mut scratch());
    assert!(matches!(r, Collapse::Done { .. }), "o controle: {r:?}");

    let short_after = short_edges_in(&m, centre, radius, emin);
    assert!(
        short_after * 2 <= short_before,
        "as curtas mal encolheram: {short_before} → {short_after}"
    );

    // ⚠️ **A barra é *pelo menos metade num dab*, e o número é MEDIDO** — não é
    // frouxidão. A trava torna cada rodada um lote INDEPENDENTE, então um dab
    // colhe um conjunto esparso; a cascata que a referência faz num laço só, aqui
    // sai da repetição. Medido (`measure_how_much_one_collapse_dab_removes`):
    // 60 → 30 → 8 → 0 curtas em quatro dabs, com a contagem parando em 227
    // vértices. Um artista dá sessenta dabs por segundo.
    for _ in 0..3 {
        let _ = collapse_in_sphere(&mut m, centre, radius, emin, &mut remap, &mut scratch());
    }
    assert_eq!(
        short_edges_in(&m, centre, radius, emin),
        0,
        "quatro dabs no mesmo lugar têm de limpar a região"
    );
    assert!(
        long_edges_in(&m, centre, radius, emin) > 0,
        "e as longas continuam lá — o colapso não é um alisador"
    );
}

/// Quantas arestas sob o limiar têm o ponto médio na esfera.
fn short_edges_in(m: &Mesh, c: [f32; 3], r: f32, emin: f32) -> usize {
    edges_in(m, c, r, |l| l < emin)
}

/// Quantas arestas ACIMA do limiar têm o ponto médio na esfera.
fn long_edges_in(m: &Mesh, c: [f32; 3], r: f32, emin: f32) -> usize {
    edges_in(m, c, r, |l| l >= emin)
}

fn edges_in(m: &Mesh, c: [f32; 3], r: f32, keep: impl Fn(f32) -> bool) -> usize {
    let pos = m.positions();
    let mut n = 0;
    for f in m.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            if a > b {
                continue;
            }
            let (pa, pb) = (pos[a as usize], pos[b as usize]);
            let mid = [
                (pa[0] + pb[0]) * 0.5,
                (pa[1] + pb[1]) * 0.5,
                (pa[2] + pb[2]) * 0.5,
            ];
            let d = [mid[0] - c[0], mid[1] - c[1], mid[2] - c[2]];
            if d[0] * d[0] + d[1] * d[1] + d[2] * d[2] > r * r {
                continue;
            }
            let e = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
            if keep((e[0] * e[0] + e[1] * e[1] + e[2] * e[2]).sqrt()) {
                n += 1;
            }
        }
    }
    n
}

/// **O SOBREVIVENTE DESLIZA, NÃO AFUNDA** — a projeção no plano tangente.
///
/// ⚠️ **É o que separa este colapso do ponto-médio do Blender**, e a diferença é
/// a SILHUETA: o centroide do anel de uma superfície convexa cai *para dentro*,
/// e um colapso que o adotasse cru encolheria a forma um pouco por vértice
/// apagado. A componente ao longo da normal é exatamente a que a projeção tira.
///
/// A régua é o raio: numa esfera unitária, todo vértice tem de continuar a ~1 do
/// centro. O oráculo compara contra o que o ponto médio faria.
#[test]
fn the_survivor_slides_along_the_surface_instead_of_sinking() {
    let mut m = tri_sphere(16, 24);
    let emin = 1.2 * mean_edge(&m);
    let mut remap = Remap::default();
    let r = collapse_in_sphere(
        &mut m,
        [0.0, 0.0, 1.0],
        0.9,
        emin,
        &mut remap,
        &mut scratch(),
    );
    assert!(matches!(r, Collapse::Done { .. }), "o controle: {r:?}");

    let mut worst = 0.0f32;
    for p in m.positions() {
        let radius = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
        worst = worst.max((radius - 1.0).abs());
    }
    assert!(
        worst < 0.02,
        "a superfície afundou {worst:.4} — o pouso está a mover o vértice ao \
         longo da normal"
    );
}

/// **NADA NA ESFERA, NADA ACONTECE.** O desfecho normal do meio de um traço, e o
/// caminho que tem de custar nada.
#[test]
fn a_dab_far_from_the_mesh_is_a_no_op() {
    let mut m = tri_sphere(8, 10);
    let (v0, f0) = (m.vert_count(), m.face_count());
    let mut remap = Remap::default();
    let r = collapse_in_sphere(
        &mut m,
        [50.0, 0.0, 0.0],
        1.0,
        1.0,
        &mut remap,
        &mut scratch(),
    );
    assert_eq!(r, Collapse::Enough);
    assert_eq!((m.vert_count(), m.face_count()), (v0, f0));
    assert!(remap.moves_nothing());
}

/// **A ORDEM NÃO PODE DEPENDER DA FORMA DA ÁRVORE.** A mesma malha, construída
/// por caminhos diferentes, tem de colapsar igual.
///
/// ⚠️ **Este é o gate da linha `hits.sort_unstable()`**, e ela não é estética:
/// no colapso a ordem decide QUEM ganha a trava da rodada, logo quais arestas
/// somem. A do octree é função do histórico de inserções.
#[test]
fn the_same_mesh_collapses_the_same_way_however_it_was_built() {
    let base = tri_sphere(12, 18);
    let mut direct = base.clone();
    // O mesmo conteúdo, com o octree construído depois de uma reconstrução.
    let mut rebuilt =
        Mesh::from_parts(base.positions().to_vec(), base.faces().to_vec()).expect("válida");
    rebuilt.rebuild();

    let emin = 1.2 * mean_edge(&base);
    let mut ra = Remap::default();
    let mut rb = Remap::default();
    let a = collapse_in_sphere(
        &mut direct,
        [0.0, 0.0, 1.0],
        0.9,
        emin,
        &mut ra,
        &mut scratch(),
    );
    let b = collapse_in_sphere(
        &mut rebuilt,
        [0.0, 0.0, 1.0],
        0.9,
        emin,
        &mut rb,
        &mut scratch(),
    );
    assert!(matches!(a, Collapse::Done { .. }), "o controle: {a:?}");
    assert_eq!(a, b, "as duas rotas colapsaram quantidades diferentes");
    assert_eq!(direct.faces(), rebuilt.faces(), "as faces divergiram");
    assert_eq!(ra, rb, "os remaps divergiram");
}
