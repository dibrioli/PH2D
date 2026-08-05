//! **O COLAPSO** — a outra metade da topologia dinâmica: o pincel REMOVE detalhe
//! onde ele deixou de ser preciso.
//!
//! Adaptado de `reference/sculptgl/src/mesh/dynamic/Decimation.js`, MIT — ver
//! `LICENSES/sculptgl-MIT.txt`.
//!
//! # Por que ele existe, e não é *"a contagem cresce sem parar"*
//!
//! Sem colapso o refino já CONVERGE: quando toda aresta da região está sob o
//! alvo, ele para. O que falta é o caminho de volta, e ele aparece em três
//! gestos reais — **comprimir** a superfície (um Grab ou um Crease aproxima
//! vértices, e triângulos que eram do tamanho certo viram lascas), **alisar**
//! (o detalhe que justificava a densidade some, a densidade fica) e **engrossar
//! o pincel** (o alvo é uma fração do raio, então o mesmo lugar passa a pedir
//! menos). Nos três a malha carrega detalhe que ninguém pediu.
//!
//! # A HISTERESE, e por que ela é 2,05 e não 2
//!
//! `d2Min = d2Max / 4.2025` no `SculptBase.js` — e `4.2025` é `2,05²`. A ideia é
//! que partir uma aresta ao meio deixa duas de `alvo/2`, e um limiar de colapso
//! em `alvo/2` as apagaria no dab seguinte, para o refino as recriar no outro;
//! com `alvo/2,05` a filha recém-nascida fica acima do limiar por 2,5%.
//!
//! ⚠️ **Mas essa frase sozinha estaria vendendo mais do que a margem entrega, e
//! foi um gate que a corrigiu.** Ela protege a filha de uma bissecção e **não**
//! todo lado que o padrão produz: o corte 1→2 cria uma MEDIANA, e a mediana de
//! um triângulo fino é curta. A pergunta que decide o número não é *"o colapso
//! remove zero logo depois do refino?"* (não remove) e sim ***o par tem ponto
//! fixo?***
//!
//! Medido (`measure_whether_refine_and_collapse_settle`, contagem de vértices em
//! 12 ciclos de refino+colapso no MESMO lugar):
//!
//! | razão | ciclos | assenta? |
//! |---|---|---|
//! | 1,80 | 710 712 700 698 696 700 698 699 695 … | **NÃO** |
//! | 2,00 | 716 721 714 714 714 714 … | sim |
//! | **2,05** | 716 720 716 716 716 716 … | sim |
//! | 2,50 | 721 722 722 722 … | sim |
//!
//! **O joelho está entre 1,8 e 2,0**, e o `2,05` da referência senta logo acima
//! dele — é margem, não arredondamento. Abaixo do joelho a malha treme para
//! sempre e o custo é um dab inteiro de topologia por movimento do mouse.
//!
//! # As quatro recusas, e todas são TOPOLOGIA
//!
//! Nenhuma delas é zelo — cada uma nomeia uma malha que o colapso quebraria:
//!
//! 1. **A aresta não tem exatamente duas faces.** Uma é beira, três ou mais é
//!    não-manifold; colapsar qualquer das duas costura a malha em si mesma.
//! 2. **Algum dos quatro vértices está na BEIRA.** Colapsar puxa o contorno
//!    aberto para dentro, e o buraco muda de forma sozinho.
//! 3. **Os dois anéis compartilham mais que os dois vértices opostos** — a
//!    *condição de elo*. Quando um terceiro vizinho é comum, juntar `a` e `b`
//!    cria duas faces com os mesmos três cantos: uma malha que ainda desenha e
//!    que nenhuma operação seguinte consegue consertar.
//! 4. **Um vizinho já foi tocado nesta rodada** — ver [`one_pass`].
//!
//! ⚠️ **O SculptGL responde (3) com uma TROCA DE DIAGONAL em vez de recusar.** É
//! uma reparação a mais, não uma correção: o Blender (`pbvh_bmesh_collapse_edge`)
//! recusa, e nós já rodamos o [`crate::dyntopo_flip`] no mesmo dab. Duas respostas
//! para *"como esta região melhora de forma"* divergiriam, e a que a wave escolhe
//! é a que já está gateada.

use crate::face::Face;
use crate::mesh::{Mesh, RegionScratch, VertexMerge};
use crate::remap::Remap;

/// A razão entre o alvo do refino e o limiar do colapso. Ver o cabeçalho.
const HYSTERESIS: f32 = 2.05;

/// Quantas rodadas um único dab pode gastar. Espelha o `MAX_PASSES` do refino, e
/// pelo mesmo motivo: cada rodada é uma edição de topologia, e o recurso é o
/// quadro. O comum termina em **uma**.
const MAX_PASSES: usize = 3;

/// **O limiar de colapso**, derivado do alvo do refino.
///
/// ⚠️ **Derivado e não autorado, de propósito.** Um segundo slider daria ao
/// artista dois números que precisam concordar para a malha não tremer, e a
/// relação entre eles é a histerese — uma propriedade do par, não uma
/// preferência. O SculptGL expõe os dois e ship o de colapso em ZERO; o Blender
/// deriva (`bm_min_edge_len = bm_max_edge_len × 0,4`) e é o que shipa ligado.
#[must_use]
pub fn collapse_target(edge_max: f32) -> f32 {
    edge_max / HYSTERESIS
}

/// O que o colapso fez, ou por que não fez nada.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Collapse {
    /// Colapsou. Os números são o que **sumiu**, não o total.
    Done {
        verts_removed: usize,
        faces_removed: usize,
        passes: usize,
    },
    /// Nenhuma aresta na esfera está sob o limiar, ou nenhuma das que estão pode
    /// ser colapsada. É o desfecho NORMAL do meio de um traço.
    Enough,
    /// A malha tem quads — a mesma recusa do refino, e pela mesma razão.
    NotTriangles,
}

/// **Colapsa as arestas curtas dentro da esfera.**
///
/// ⚠️ **`remap` é LIMPO e preenchido, e é o contrato desta porta.** Um colapso
/// renumera; quem guarda índice de vértice ou de face entre chamadas — o traço em
/// voo, e é o caso real — tem de aplicar a sequência, na ordem. Ver [`Remap`].
///
/// ⚠️ **É parâmetro obrigatório e não um `Option`**, a mesma lei do `births` do
/// refino: um canal opcional é o que chega a duas das sete rotas e faz a feature
/// simplesmente não acontecer nas outras cinco, em silêncio. Aqui o preço de
/// esquecer é um traço lendo o `pre` de outro vértice.
pub fn collapse_in_sphere(
    mesh: &mut Mesh,
    center: [f32; 3],
    radius: f32,
    edge_min: f32,
    remap: &mut Remap,
    scratch: &mut RegionScratch,
) -> Collapse {
    *remap = Remap {
        faces: mesh.face_count(),
        verts: mesh.vert_count(),
        ..Remap::default()
    };
    if !mesh.faces().iter().all(Face::is_tri) {
        return Collapse::NotTriangles;
    }
    if edge_min <= 0.0 || radius <= 0.0 || !edge_min.is_finite() {
        return Collapse::Enough;
    }
    let (v0, f0) = (mesh.vert_count(), mesh.face_count());
    let mut passes = 0;
    while passes < MAX_PASSES {
        let Some(step) = one_pass(mesh, center, radius, edge_min, scratch) else {
            break;
        };
        remap.then(step);
        passes += 1;
    }
    if passes == 0 {
        return Collapse::Enough;
    }
    Collapse::Done {
        verts_removed: v0 - mesh.vert_count(),
        faces_removed: f0 - mesh.face_count(),
        passes,
    }
}

/// Um colapso decidido, ainda não aplicado.
struct Planned {
    keep: u32,
    gone: u32,
    /// As duas faces que a aresta divide — as que somem.
    pair: [u32; 2],
    /// Onde o sobrevivente pousa.
    at: [f32; 3],
}

/// Uma rodada. `None` se nada pôde ser colapsado.
///
/// ⚠️ **Os colapsos de uma rodada são INDEPENDENTES por construção, e é isso que
/// torna a aplicação um lote só.** Cada um TRAVA `{a, b} ∪ anel(a) ∪ anel(b)`, e
/// um candidato cujo próprio conjunto encoste em algo travado é adiado para a
/// rodada seguinte. Como toda face de `a` tem os três cantos em `{a} ∪ anel(a)`,
/// conjuntos de trava disjuntos implicam conjuntos de FACE disjuntos — nenhuma
/// edição deste lote pode ler o que outra escreveu.
///
/// ⚠️ **A alternativa é a do SculptGL: aplicar em cascata, remendando os anéis a
/// cada colapso e adiando só a remoção.** Ela colapsa mais por rodada e paga com
/// um estado intermediário que nenhum gate consegue afirmar — a malha fica
/// meio-editada durante o laço. Aqui cada rodada leva a malha de um estado
/// íntegro a outro, e o teto de rodadas devolve a cascata.
fn one_pass(
    mesh: &mut Mesh,
    center: [f32; 3],
    radius: f32,
    edge_min: f32,
    scratch: &mut RegionScratch,
) -> Option<Remap> {
    let r2 = radius * radius;
    let emin2 = edge_min * edge_min;

    let mut hits = Vec::new();
    mesh.octree().faces_in_sphere(center, radius, &mut hits);
    if hits.is_empty() {
        return None;
    }
    // ⚠️ **A ordem é ORDENADA, e aqui isso é correção e não estética.** No refino
    // a escolha é comutativa (toda aresta longa de toda face escolhida é
    // marcada); aqui ela decide QUEM ganha a trava, logo qual colapso acontece.
    // A ordem que o octree devolve é função do formato da árvore, que muda com o
    // histórico de inserções — duas malhas idênticas construídas por caminhos
    // diferentes colapsariam diferente.
    hits.sort_unstable();
    hits.dedup();

    let mut locked = vec![false; mesh.vert_count()];
    let mut planned: Vec<Planned> = Vec::new();
    for &f in &hits {
        let Some(face) = mesh.faces().get(f as usize) else {
            continue;
        };
        let v = face.verts();
        // O MESMO teste do refino — *esta face está no dab?* é uma pergunta só, e
        // duas respostas dariam uma coroa onde uma metade adensa e a outra não.
        if !centroid_in_sphere(mesh.positions(), v, center, r2) {
            continue;
        }
        let Some((a, b)) = shortest_edge_under(mesh.positions(), v, emin2) else {
            continue;
        };
        let Some(p) = plan(mesh, a, b) else {
            continue;
        };
        if touches_locked(mesh, &locked, p.keep, p.gone) {
            continue;
        }
        lock_around(mesh, &mut locked, p.keep, p.gone);
        planned.push(p);
    }
    if planned.is_empty() {
        return None;
    }

    let mut edits: Vec<(u32, Face)> = Vec::new();
    let mut dead_faces: Vec<u32> = Vec::new();
    let mut dead_verts: Vec<u32> = Vec::with_capacity(planned.len());
    let mut merges: Vec<VertexMerge> = Vec::with_capacity(planned.len());
    for p in &planned {
        for &fi in mesh.adjacency().vert_faces.neighbours(p.gone as usize) {
            if p.pair.contains(&fi) {
                continue;
            }
            let mut face = mesh.faces()[fi as usize];
            face.rename_vert(p.gone, p.keep);
            edits.push((fi, face));
        }
        dead_faces.extend_from_slice(&p.pair);
        dead_verts.push(p.gone);
        merges.push(VertexMerge {
            keep: p.keep,
            gone: p.gone,
            at: p.at,
        });
    }
    dead_faces.sort_unstable();
    dead_verts.sort_unstable();
    Some(mesh.shrink_topology(&edits, &dead_faces, &dead_verts, &merges, scratch))
}

/// O centroide da face está na esfera? A mesma lei do refino.
fn centroid_in_sphere(pos: &[[f32; 3]], v: &[u32], center: [f32; 3], r2: f32) -> bool {
    let inv = 1.0 / v.len() as f32;
    let mut c = [0.0f32; 3];
    for &i in v {
        let p = pos[i as usize];
        for (acc, q) in c.iter_mut().zip(p) {
            *acc += q * inv;
        }
    }
    let d = [c[0] - center[0], c[1] - center[1], c[2] - center[2]];
    d[0] * d[0] + d[1] * d[1] + d[2] * d[2] <= r2
}

/// A aresta mais CURTA da face, se ela estiver sob o limiar.
///
/// ⚠️ **A mais curta e não *qualquer uma* sob o limiar**, e o motivo é o espelho
/// exato do `longest_edge` do refino: colapsar uma aresta que não é a mais curta
/// deixa a mais curta ainda lá, e a rodada seguinte a apaga de qualquer jeito —
/// mas depois de o vizinho ter mudado de forma por nada.
fn shortest_edge_under(pos: &[[f32; 3]], v: &[u32], emin2: f32) -> Option<(u32, u32)> {
    let mut best: Option<(f32, u32, u32)> = None;
    for k in 0..v.len() {
        let (a, b) = (v[k], v[(k + 1) % v.len()]);
        let (pa, pb) = (pos[a as usize], pos[b as usize]);
        let d = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
        let l2 = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
        if best.is_none_or(|(w, ..)| l2 < w) {
            best = Some((l2, a, b));
        }
    }
    best.filter(|&(l2, ..)| l2 < emin2).map(|(_, a, b)| (a, b))
}

/// Decide se a aresta `a—b` pode colapsar, e para onde. Ver as quatro recusas no
/// cabeçalho do módulo.
fn plan(mesh: &Mesh, a: u32, b: u32) -> Option<Planned> {
    let adj = mesh.adjacency();
    let faces = mesh.faces();
    // (1) exatamente duas faces dividem a aresta.
    let mut pair = [u32::MAX; 2];
    let mut n = 0usize;
    for &fi in adj.vert_faces.neighbours(a as usize) {
        if faces[fi as usize].verts().contains(&b) {
            if n < 2 {
                pair[n] = fi;
            }
            n += 1;
        }
    }
    if n != 2 {
        return None;
    }
    let o1 = third(faces[pair[0] as usize], a, b)?;
    let o2 = third(faces[pair[1] as usize], a, b)?;
    // (2) nenhum dos quatro está na beira.
    for v in [a, b, o1, o2] {
        if adj.is_border(v as usize) {
            return None;
        }
    }
    // (2b) os dois opostos PERDEM uma face cada, e um vértice interior com duas
    // faces não é superfície — é uma aba. Sem esta recusa um tetraedro passa
    // pela condição de elo (os anéis dele compartilham exatamente os dois
    // opostos) e colapsa em duas faces com os MESMOS três cantos.
    if adj.valence(o1 as usize) < 4 || adj.valence(o2 as usize) < 4 {
        return None;
    }
    // (3) a condição de elo: os anéis compartilham EXATAMENTE `o1` e `o2`.
    let ring_b = adj.vert_verts.neighbours(b as usize);
    let shared = adj
        .vert_verts
        .neighbours(a as usize)
        .iter()
        .filter(|w| ring_b.contains(w))
        .count();
    if shared != 2 {
        return None;
    }
    // ⚠️ O sobrevivente é o de MENOR índice, e a escolha é por determinismo: a
    // aresta chega aqui pela face que a propôs, e a ordem dos cantos dela não
    // pode decidir quem sobrevive — duas faces vizinhas proporiam a mesma aresta
    // com os extremos trocados.
    let (keep, gone) = if a < b { (a, b) } else { (b, a) };
    Some(Planned {
        keep,
        gone,
        pair,
        at: landing(mesh, keep, gone),
    })
}

/// Onde o sobrevivente pousa: o **centroide do anel fundido, projetado de volta
/// no plano tangente**.
///
/// ⚠️ **Ele NÃO é o meio da aresta, e a diferença é a forma.** O meio é o que o
/// Blender usa e é mais simples; o que a referência faz — e o que esta função
/// porta — é deslizar o vértice para o centro do anel **sem o mover ao longo da
/// normal**. O resultado é um triângulo melhor sem a superfície andar: a
/// componente que mudaria a silhueta é exatamente a que a projeção remove.
fn landing(mesh: &Mesh, keep: u32, gone: u32) -> [f32; 3] {
    let adj = mesh.adjacency();
    let pos = mesh.positions();
    let mut sum = [0.0f32; 3];
    let mut n = 0.0f32;
    for &w in adj.vert_verts.neighbours(keep as usize) {
        if w == gone {
            continue;
        }
        for (acc, q) in sum.iter_mut().zip(pos[w as usize]) {
            *acc += q;
        }
        n += 1.0;
    }
    for &w in adj.vert_verts.neighbours(gone as usize) {
        if w == keep || adj.vert_verts.neighbours(keep as usize).contains(&w) {
            continue;
        }
        for (acc, q) in sum.iter_mut().zip(pos[w as usize]) {
            *acc += q;
        }
        n += 1.0;
    }
    let here = pos[keep as usize];
    if n < 1.0 {
        return here;
    }
    let mean = [sum[0] / n, sum[1] / n, sum[2] / n];
    let nk = mesh.normals()[keep as usize];
    let ng = mesh.normals()[gone as usize];
    let nrm = unit([nk[0] + ng[0], nk[1] + ng[1], nk[2] + ng[2]]);
    let d = [mean[0] - here[0], mean[1] - here[1], mean[2] - here[2]];
    let along = nrm[0] * d[0] + nrm[1] * d[1] + nrm[2] * d[2];
    [
        mean[0] - nrm[0] * along,
        mean[1] - nrm[1] * along,
        mean[2] - nrm[2] * along,
    ]
}

fn unit(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l > 1e-12 {
        [v[0] / l, v[1] / l, v[2] / l]
    } else {
        [0.0, 0.0, 1.0]
    }
}

/// O terceiro canto de um triângulo, dados dois.
fn third(f: Face, a: u32, b: u32) -> Option<u32> {
    f.verts().iter().copied().find(|&v| v != a && v != b)
}

/// Algum vértice do conjunto deste colapso já foi travado nesta rodada?
fn touches_locked(mesh: &Mesh, locked: &[bool], keep: u32, gone: u32) -> bool {
    let adj = mesh.adjacency();
    for v in [keep, gone] {
        if locked[v as usize] {
            return true;
        }
        for &w in adj.vert_verts.neighbours(v as usize) {
            if locked[w as usize] {
                return true;
            }
        }
    }
    false
}

fn lock_around(mesh: &Mesh, locked: &mut [bool], keep: u32, gone: u32) {
    let adj = mesh.adjacency();
    for v in [keep, gone] {
        locked[v as usize] = true;
        for &w in adj.vert_verts.neighbours(v as usize) {
            locked[w as usize] = true;
        }
    }
}

#[cfg(test)]
#[path = "collapse_tests.rs"]
mod tests;
