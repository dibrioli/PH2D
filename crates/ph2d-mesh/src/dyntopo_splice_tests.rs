//! **OS GATES DA EDIÇÃO INCREMENTAL** — filho do [`super`], não irmão.
//!
//! Eles têm um assunto só: *o que a mudança de topologia em EDIÇÃO tem de
//! preservar em relação a uma reconstrução*. O pai julga a LEI do refino (a
//! ausência de rachadura, o alcance do pincel, o padrão do corte); aqui julga-se
//! a **rota** que a wave da estrutura mutável trocou por baixo dela.
//!
//! ⚠️ **FILHO e não irmão** para que `tri_sphere`, `scratch` e `mean_edge_of`
//! continuem sendo uma PORTA e não uma segunda cópia: uma fixture duplicada é
//! como dois gates passam a testar duas malhas diferentes com o mesmo nome.

use super::*;

/// **A FRENTE FECHA O MESMO CONJUNTO QUE A VARREDURA FECHAVA.**
///
/// O fecho de Rivara é um ponto FIXO, e trocar a forma de alcançá-lo não pode
/// mover o conjunto — só a ordem em que as marcas entram. ⚠️ **E uma frente que
/// esquecesse de empurrar uma vizinha falharia em SILÊNCIO:** o padrão de corte
/// fecha a rachadura sobre qualquer subconjunto de arestas partidas, então a
/// malha continuaria fechada e a única evidência seria um triângulo mais fino do
/// que precisava — invisível até alguém medir ângulo.
///
/// O oráculo é a varredura CONGELADA (`close_lepp_by_sweep`, o código que
/// shipava), e a fixture semeia as mesmas marcas iniciais nas duas rotas.
#[test]
fn the_front_closes_the_same_set_the_sweep_did() {
    let m = tri_sphere(28, 40);
    let (faces, adj, pos) = (m.faces(), m.adjacency(), m.positions());
    let ids = EdgeIds::build(adj);

    // Semente: as arestas longas de um punhado de faces espalhadas. Espalhadas
    // de propósito — um fecho que arranca de um bloco único é o caso fácil, e o
    // que separa as duas rotas é a cadeia alcançar longe.
    let mut seed: Vec<(u32, u32, u32)> = Vec::new();
    let mut pending_front = vec![false; ids.len()];
    let mut front: Vec<u32> = Vec::new();
    for fi in (0..faces.len()).step_by(41) {
        let v = faces[fi].verts();
        let Some((_, e, a, b)) = super::longest_edge(&ids, adj, v, pos) else {
            continue;
        };
        if !pending_front[e as usize] {
            pending_front[e as usize] = true;
            seed.push((e, a, b));
            super::faces_of_edge(adj, faces, a, b, &mut front);
        }
    }
    assert!(seed.len() > 20, "o controle: a semente tem de existir");
    let mut pending_sweep = pending_front.clone();

    let mut marked = seed.clone();
    super::close_lepp(
        &ids,
        adj,
        faces,
        pos,
        &mut pending_front,
        &mut marked,
        &mut front,
    );
    super::close_lepp_by_sweep(&ids, adj, faces, pos, &mut pending_sweep);

    assert!(
        marked.len() > seed.len(),
        "o controle: o fecho tem de ACRESCENTAR marcas ({} de semente)",
        seed.len()
    );
    let a: Vec<usize> = (0..ids.len()).filter(|&e| pending_front[e]).collect();
    let b: Vec<usize> = (0..ids.len()).filter(|&e| pending_sweep[e]).collect();
    assert_eq!(
        a,
        b,
        "a frente e a varredura fecharam conjuntos diferentes ({} contra {})",
        a.len(),
        b.len()
    );

    // E a FRENTE tem de conter toda face que toca uma marca — é ela que o emit
    // itera, então uma face de fora seria uma face que fica com o T-vértice.
    front.sort_unstable();
    front.dedup();
    for (fi, f) in faces.iter().enumerate() {
        let v = f.verts();
        let touches = (0..v.len()).any(|k| {
            ids.id_of(adj, v[k], v[(k + 1) % v.len()])
                .is_some_and(|e| pending_front[e as usize])
        });
        assert_eq!(
            touches,
            front.binary_search(&(fi as u32)).is_ok(),
            "a face {fi} toca uma marca mas nao esta' na frente (ou o contrario)"
        );
    }
}

/// **A FOLHA QUE ENGORDA SE DIVIDE** — a árvore não degrada ao longo de um traço.
///
/// ⚠️ **É o preço escondido da inserção incremental, e ele não levanta erro
/// nenhum.** Uma face nova nasce na folha da mãe, então uma folha que só recebe
/// vira uma lista linear de centenas de faces — e a partir daí toda consulta
/// naquela região devolve todas elas, o `refit` percorre todas elas, e o sintoma
/// aparece como *"o pincel ficou lento onde eu mais trabalhei"*.
///
/// Medido sobre doze dabs cruzando a esfera (`measure_whether_the_octree_degrades`):
/// as faces vão de 48 768 a 61 526 (+26%), os nós de 2 633 a 3 313 (+26% — a
/// árvore cresce junto) e a folha mais cheia fica em **97-100**. Sem a divisão
/// local ela cresceria sem teto.
///
/// A barra é **2×** o teto e não o teto exato, e a folga tem mecanismo: a divisão
/// acontece DEPOIS de a folha receber a leva inteira do dab, então ela passa do
/// teto por um instante por construção; e uma folha na profundidade máxima não
/// pode se dividir de jeito nenhum.
#[test]
fn a_leaf_that_fattens_splits_instead_of_growing_forever() {
    let mut m = tri_sphere(24, 36);
    let radius = 0.4f32;
    let target = 0.35 * mean_edge_of(&m);
    let mut births = Vec::new();
    let (max0, _) = m.octree().leaf_occupancy();
    let f0 = m.face_count();
    let mut scr = scratch();
    for k in 0..8u8 {
        let x = f32::from(k) * 0.14 - 0.5;
        let centre = [x, (1.0 - x * x).max(0.0).sqrt(), 0.0];
        refine_in_sphere(&mut m, centre, radius, target, &mut births, &mut scr);
    }
    assert!(
        m.face_count() > f0 * 2,
        "o controle: o traco tem de ADENSAR de verdade ({f0} -> {})",
        m.face_count()
    );
    let (max, mean) = m.octree().leaf_occupancy();
    assert!(
        max <= max0 * 2,
        "a folha mais cheia passou de {max0} para {max} — a arvore esta' degradando \
         em listas lineares, e o custo reaparece na consulta sem ninguem ligar ao indice"
    );
    assert!(
        mean < max0 as f64,
        "a media ({mean:.1}) tambem tem de ficar sob o teto"
    );
}

/// O comprimento médio de aresta — a régua que faz um alvo refinar em qualquer
/// densidade, em vez de um número fixo que a malha fina já satisfaz.
fn mean_edge_of(m: &Mesh) -> f32 {
    let pos = m.positions();
    let mut tris = Vec::new();
    m.triangle_indices(&mut tris);
    let mut sum = 0.0f32;
    for t in &tris {
        for k in 0..3 {
            let (a, b) = (pos[t[k] as usize], pos[t[(k + 1) % 3] as usize]);
            let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            sum += (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        }
    }
    sum / (tris.len() * 3).max(1) as f32
}

/// **O ORÁCULO DA EDIÇÃO INCREMENTAL** — depois de um refino de verdade, a malha
/// tem de ser *exatamente* o que uma reconstrução produziria.
///
/// ⚠️ **A comparação inclui o ÚLTIMO BIT das normais**, e não é zelo: a normal de
/// um vértice é a soma das normais das faces do anel dele, e somar os mesmos
/// números noutra ordem dá outro resultado em `f32`. Um incremental que "só"
/// reordena um anel passa em todo gate de conteúdo e diverge aqui — que é o
/// único lugar onde a diferença é observável antes de virar uma costura na luz.
///
/// ⚠️ **E ele roda sobre um refino que de fato PARTIU alguma coisa** — sem o
/// controle, uma malha que ninguém refinou é trivialmente igual a si mesma e o
/// gate ficaria verde sobre uma porta que nunca correu.
#[test]
fn a_spliced_mesh_is_exactly_what_a_rebuild_would_have_produced() {
    let mut m = tri_sphere(12, 18);
    let (v0, f0) = (m.vert_count(), m.face_count());
    let mut births = Vec::new();
    let r = refine_in_sphere(
        &mut m,
        [0.0, 0.0, 1.0],
        0.6,
        edge_target(0.6, 1.0),
        &mut births,
        &mut scratch(),
    );
    assert!(
        matches!(r, Refine::Done { .. }),
        "o controle: a fixture tem de refinar de verdade, e nao refinou ({r:?})"
    );
    assert!(m.vert_count() > v0 && m.face_count() > f0);

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
    assert_eq!(
        m.face_normals(),
        rebuilt.face_normals(),
        "as normais de face divergiram"
    );
    assert_eq!(
        m.normals(),
        rebuilt.normals(),
        "as normais de vértice divergiram — provavelmente a ORDEM de um anel"
    );
    assert_eq!(m.bounds(), rebuilt.bounds(), "a caixa do mundo divergiu");
}

/// **O ÍNDICE ESPACIAL NÃO ESQUECE GEOMETRIA** depois de absorver faces novas.
///
/// ⚠️ **O oráculo é a RESPOSTA, não a árvore.** Duas partições diferentes são as
/// duas corretas (o doc do octree explica: a partição decide a *forma* da
/// árvore, a caixa frouxa decide a *resposta*), então comparar nós contra um
/// build seria pinar uma heurística. O que tem de valer é que toda face cujo
/// vértice cai na esfera está na lista que a consulta devolve — porque a face
/// que sumir dela vira um BURACO no traço, sem erro e sem aviso.
#[test]
fn the_spliced_octree_still_finds_every_face_it_should() {
    let mut m = tri_sphere(12, 18);
    let mut births = Vec::new();
    let r = refine_in_sphere(
        &mut m,
        [0.0, 0.0, 1.0],
        0.6,
        edge_target(0.6, 1.0),
        &mut births,
        &mut scratch(),
    );
    assert!(matches!(r, Refine::Done { .. }), "o controle: {r:?}");

    let mut hits = Vec::new();
    let mut probes = 0;
    for probe in [
        ([0.0, 1.0, 0.0], 0.30f32),
        ([0.0, 1.0, 0.0], 0.60),
        ([0.7, 0.7, 0.0], 0.25),
        ([0.0, -1.0, 0.0], 0.40),
        ([0.0, 0.0, 1.0], 0.35),
    ] {
        let (c, rad) = probe;
        m.octree().faces_in_sphere(c, rad, &mut hits);
        let found: std::collections::BTreeSet<u32> = hits.iter().copied().collect();
        let r2 = rad * rad;
        for (fi, f) in m.faces().iter().enumerate() {
            let inside = f.verts().iter().any(|&v| {
                let p = m.positions()[v as usize];
                let d = [p[0] - c[0], p[1] - c[1], p[2] - c[2]];
                d[0] * d[0] + d[1] * d[1] + d[2] * d[2] <= r2
            });
            if inside {
                probes += 1;
                assert!(
                    found.contains(&(fi as u32)),
                    "a face {fi} esta' na esfera {c:?}/{rad} e o octree nao a devolveu"
                );
            }
        }
    }
    assert!(probes > 100, "o controle: as sondas tem de achar geometria");
}

/// **AS FAIXAS DAS FOLHAS CONTINUAM DISJUNTAS** depois de realocadas — é isso que
/// torna a leitura segura quando o vetor de índices deixa de ser uma permutação
/// arrumada.
///
/// Irmão exato do `rows_are_disjoint` do CSR, e pela mesma razão: o invariante
/// que sobrevive à edição não é *"as faixas ladrilham o vetor"* (que a
/// realocação quebra de propósito), é *"nenhuma pisa na outra"*.
#[test]
fn the_leaf_ranges_stay_disjoint_after_absorbing_new_faces() {
    let mut m = tri_sphere(12, 18);
    let mut births = Vec::new();
    for k in 0..3u8 {
        let centre = [0.0, 0.0, 1.0 - 0.1 * f32::from(k)];
        refine_in_sphere(
            &mut m,
            centre,
            0.6,
            edge_target(0.6, 1.0),
            &mut births,
            &mut scratch(),
        );
    }
    let (spans, indices) = m.octree().leaf_spans_for_gate();
    let mut sorted_spans = spans.clone();
    sorted_spans.sort_unstable();
    for w in sorted_spans.windows(2) {
        assert!(
            w[0].1 <= w[1].0,
            "duas folhas se sobrepoem em face_indices: {:?} e {:?}",
            w[0],
            w[1]
        );
    }
    for &(_, e) in &spans {
        assert!(e <= indices, "uma folha sai de face_indices");
    }
    // E TODA face está em exatamente uma folha — o que a disjunção sozinha não
    // diz (faixas disjuntas e um buraco entre elas também são disjuntas).
    let total: usize = spans.iter().map(|&(a, b)| b - a).sum();
    assert_eq!(
        total,
        m.face_count(),
        "as folhas juntas tem de conter cada face uma vez"
    );
}
