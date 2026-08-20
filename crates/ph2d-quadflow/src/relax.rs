//! **A RELAXAÇÃO — a grade fica REGULAR sem sair da forma** (ADR-0160 §5, Q3.6).
//!
//! A extração entrega os nós onde o campo de posição os pôs: a média das origens
//! que caíram em cada célula. Isso acerta *onde a linha da grade passa* e não diz
//! nada sobre *como os quads se distribuem ao longo dela* — e o resultado é uma
//! malha topologicamente correta com quads de tamanhos visivelmente diferentes,
//! que é a queixa **"a qualidade é em geral muito baixa"** (Enio, 2026-08-19).
//!
//! A cura é a padrão da literatura de remesh, e são **duas metades que não
//! funcionam separadas**:
//!
//! 1. **Laplaciano** — cada nó anda para a média dos vizinhos DELE na grade nova.
//!    Sozinho, ele encolhe a peça: uma esfera relaxada o suficiente vira um ponto.
//! 2. **Projeção** — o nó volta ao ponto mais próximo da superfície de ENTRADA.
//!    Sozinha, ela não faz nada (o nó já estava perto). É ela que transforma o
//!    encolhimento num deslize **ao longo** da superfície.
//!
//! ⚠️ **JACOBI e não Gauss-Seidel** (todos os alvos calculados antes de qualquer
//! escrita): com escrita no lugar, o resultado passa a depender da ordem dos
//! índices, e a A7 (determinismo byte-a-byte) deixaria de ser uma propriedade da
//! lei para ser uma propriedade da numeração das células.

use ph2d_mesh::Mesh;

use crate::extract::{dot, sub};

/// **Quantas passadas** — MEDIDO, não escolhido (`CLAUDE.md` §0.0).
///
/// Pela sonda `measure_the_relaxation`, a `3,0×` a aresta de entrada. As duas
/// réguas: o **desvio de aresta** (σ/média — a irregularidade que se vê) e o
/// **Hausdorff bilateral em unidades de quad** (a forma).
///
/// | passadas | 0 | 1 | **2** | 4 | 8 | 16 |
/// |---|---|---|---|---|---|---|
/// | esfera — desvio | 0,170 | 0,160 | **0,160** | 0,163 | 0,165 | 0,166 |
/// | esfera — forma | 0,091 | 0,099 | **0,098** | 0,096 | 0,093 | 0,100 |
/// | toro — desvio | 0,172 | 0,164 | **0,163** | 0,165 | 0,169 | 0,179 |
/// | toro — forma | 0,204 | 0,195 | **0,202** | 0,196 | 0,180 | 0,190 |
/// | amassada — desvio | 0,185 | 0,171 | **0,167** | 0,164 | 0,160 | 0,158 |
/// | amassada — forma | 0,507 | 0,493 | **0,465** | 0,443 | 0,419 | 0,413 |
/// | amassada — relógio | — | 12 ms | **24 ms** | 47 ms | 93 ms | 190 ms |
///
/// ⚠️ **Duas é o joelho, e ele é o mesmo nas três fixturas**: é onde a esfera e
/// o toro atingem o MELHOR desvio das seis colunas, e daí para cima os dois
/// **pioram** (0,160 → 0,166 e 0,163 → 0,179) — o Laplaciano continua a
/// redistribuir depois de a grade já estar tão regular quanto aquela topologia
/// permite, e passa a arrastar os nós à volta das singularidades.
///
/// ⚠️ **E o ganho é MODESTO, o que este doc diz de propósito:** −6 % de
/// irregularidade na esfera, −10 % na amassada, e −8 % de Hausdorff nela. Não é
/// o passo que faz a diferença entre uma grade feia e uma bonita — o que faz é o
/// piso do [`crate::scale::FLOOR_IN_INPUT_EDGES`] e o fecho sem leque. *Um
/// número honesto ao lado do knob impede a próxima wave de o subir esperando o
/// que ele não dá.*
pub const RELAX_PASSES: usize = 2;

/// **O passo do Laplaciano** — quanto de cada passada é obedecido.
///
/// ⚠️ **Meio passo, e a razão é a alternância par/ímpar.** Com `λ = 1` o nó salta
/// para a média dos vizinhos, e numa grade de quads isso faz os dois subgrafos do
/// tabuleiro de xadrez trocarem de lugar a cada passada — a malha oscila em vez
/// de convergir. Metade é o amortecimento que a torna monótona; é o mesmo `0,5`
/// da suavização dos campos, e pela mesma razão.
const LAMBDA: f32 = 0.5;

/// **RELAXA a malha extraída sobre a superfície da entrada.**
///
/// ⚠️ **A entrada é a malha ORIGINAL do artista**, não a extraída: é ela que
/// define *onde a superfície está*. Projetar sobre a própria saída seria pedir a
/// uma malha que se corrigisse contra si mesma, e o Laplaciano então encolhe sem
/// nada a segurá-lo.
pub fn relax(out: &mut Mesh, input: &Mesh, passes: usize) {
    if passes == 0 || out.vert_count() == 0 {
        return;
    }
    let n = out.vert_count();
    // A vizinhança na GRADE NOVA, uma vez só — ela não muda com o relaxamento.
    let neighbours: Vec<Vec<u32>> = {
        let adj = out.adjacency();
        (0..n)
            .map(|v| adj.vert_verts.neighbours(v).to_vec())
            .collect()
    };
    // O raio de busca da projeção: uma aresta da grade nova, com folga. Um raio
    // pequeno demais devolve zero faces e a projeção vira um no-op silencioso —
    // por isso a busca DOBRA até achar, em vez de desistir.
    let seed_radius = mean_edge_of(out).max(crate::scale::MIN_EDGE);

    let mut target = vec![[0.0f32; 3]; n];
    let mut normals = vec![[0.0f32; 3]; n];
    for _ in 0..passes {
        vertex_normals(out, &mut normals);
        let pos = out.positions();
        for v in 0..n {
            let ns = &neighbours[v];
            if ns.len() < 3 {
                // ⚠️ Um nó de grau baixo é borda ou resíduo: relaxá-lo puxa a
                // malha para dentro sem ninguém do outro lado a segurar.
                target[v] = pos[v];
                continue;
            }
            let mut sum = [0.0f32; 3];
            for &w in ns {
                let p = pos[w as usize];
                for i in 0..3 {
                    sum[i] += p[i];
                }
            }
            let inv = 1.0 / ns.len() as f32;
            let p = pos[v];
            let d = [
                sum[0].mul_add(inv, -p[0]),
                sum[1].mul_add(inv, -p[1]),
                sum[2].mul_add(inv, -p[2]),
            ];
            // ⚠️ **SÓ A COMPONENTE TANGENTE, e é a diferença entre relaxar e
            // encolher.** A média dos vizinhos de um nó sobre uma superfície
            // curva cai **para dentro** dela — é a corda, não o arco —, e é essa
            // componente normal que tira volume. Medido no toro (quad de `0,25`
            // sobre um raio menor de `0,35`, a grade mais grossa que aquele tubo
            // aceita): o Laplaciano cheio custava **4,5 %** do volume, com a
            // projeção já ligada. A componente tangente é a que redistribui os
            // nós ao longo da superfície, que é tudo o que se quer.
            let nv = normals[v];
            let along = dot(d, nv);
            let t = [
                along.mul_add(-nv[0], d[0]),
                along.mul_add(-nv[1], d[1]),
                along.mul_add(-nv[2], d[2]),
            ];
            target[v] = [
                LAMBDA.mul_add(t[0], p[0]),
                LAMBDA.mul_add(t[1], p[1]),
                LAMBDA.mul_add(t[2], p[2]),
            ];
        }
        for t in &mut target {
            *t = project_onto(input, *t, seed_radius);
        }
        out.positions_mut().copy_from_slice(&target);
    }
    // ⚠️ **O `rebuild` paga a dívida que o `positions_mut` nomeia**: sem ele a
    // caixa, o octree e as normais continuam a descrever a malha de antes — e é
    // a caixa que a câmera usa para enquadrar.
    out.rebuild();
}

/// **AS NORMAIS DE VÉRTICE a partir das posições ATUAIS.**
///
/// ⚠️ **Recalculadas a cada passada, e não lidas do [`Mesh`].** As do `Mesh`
/// descrevem a malha de antes do `positions_mut`, e usá-las seria projetar o
/// deslocamento no plano tangente **errado** — o erro cresceria a cada passada,
/// que é a forma mais silenciosa de um alisador voltar a encolher.
fn vertex_normals(mesh: &Mesh, out: &mut [[f32; 3]]) {
    for n in out.iter_mut() {
        *n = [0.0; 3];
    }
    let p = mesh.positions();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 1..v.len() - 1 {
            let (a, b, c) = (p[v[0] as usize], p[v[k] as usize], p[v[k + 1] as usize]);
            let (u, w) = (sub(b, a), sub(c, a));
            // ⚠️ **SEM normalizar: o produto vetorial já pesa pela ÁREA**, e é o
            // peso certo — uma face minúscula não pode ter o mesmo voto que a
            // face grande ao lado na direção da superfície ali.
            let cr = [
                u[1].mul_add(w[2], -(u[2] * w[1])),
                u[2].mul_add(w[0], -(u[0] * w[2])),
                u[0].mul_add(w[1], -(u[1] * w[0])),
            ];
            for &vi in &[v[0], v[k], v[k + 1]] {
                for i in 0..3 {
                    out[vi as usize][i] += cr[i];
                }
            }
        }
    }
    for n in out.iter_mut() {
        let len = dot(*n, *n).sqrt();
        if len > 1.0e-20 {
            *n = [n[0] / len, n[1] / len, n[2] / len];
        } else {
            *n = [0.0, 0.0, 1.0];
        }
    }
}

/// A aresta média de uma malha — o raio de partida da busca.
fn mean_edge_of(mesh: &Mesh) -> f32 {
    let p = mesh.positions();
    let (mut sum, mut count) = (0.0f64, 0usize);
    for f in mesh.faces() {
        let v = f.verts();
        for i in 0..v.len() {
            let d = sub(p[v[i] as usize], p[v[(i + 1) % v.len()] as usize]);
            sum += f64::from(dot(d, d).sqrt());
            count += 1;
        }
    }
    if count == 0 {
        0.0
    } else {
        (sum / count as f64) as f32
    }
}

/// **O PONTO MAIS PRÓXIMO da superfície de `mesh`.**
///
/// ⚠️ **A busca DOBRA o raio até achar face, e é obrigatório.** Um raio fixo
/// devolve zero faces sobre uma parte esparsa do modelo, e a projeção vira um
/// no-op **silencioso** — o Laplaciano então roda sem freio e a peça encolhe.
/// O teto de dobras é a diagonal da caixa: além dela não há malha nenhuma.
pub(super) fn project_onto(mesh: &Mesh, p: [f32; 3], seed_radius: f32) -> [f32; 3] {
    let b = mesh.bounds();
    let diag = {
        let d = sub(b.max, b.min);
        dot(d, d).sqrt()
    };
    let mut radius = seed_radius.max(1.0e-6);
    let mut hits: Vec<u32> = Vec::new();
    loop {
        hits.clear();
        mesh.octree().faces_in_sphere(p, radius, &mut hits);
        if !hits.is_empty() || radius > diag {
            break;
        }
        radius *= 2.0;
    }
    if hits.is_empty() {
        return p;
    }

    let (verts, faces) = (mesh.positions(), mesh.faces());
    let (mut best, mut best_p) = (f32::INFINITY, p);
    for &f in &hits {
        let v = faces[f as usize].verts();
        for k in 1..v.len() - 1 {
            let q = closest_on_triangle(
                p,
                verts[v[0] as usize],
                verts[v[k] as usize],
                verts[v[k + 1] as usize],
            );
            let d = sub(q, p);
            let dist = dot(d, d);
            if dist < best {
                best = dist;
                best_p = q;
            }
        }
    }
    best_p
}

/// O ponto do triângulo mais próximo de `p` — as sete regiões de Voronoi.
fn closest_on_triangle(p: [f32; 3], a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let ab = sub(b, a);
    let ac = sub(c, a);
    let ap = sub(p, a);
    let (d1, d2) = (dot(ab, ap), dot(ac, ap));
    if d1 <= 0.0 && d2 <= 0.0 {
        return a;
    }
    let bp = sub(p, b);
    let (d3, d4) = (dot(ab, bp), dot(ac, bp));
    if d3 >= 0.0 && d4 <= d3 {
        return b;
    }
    let vc = d1.mul_add(d4, -(d3 * d2));
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return [
            v.mul_add(ab[0], a[0]),
            v.mul_add(ab[1], a[1]),
            v.mul_add(ab[2], a[2]),
        ];
    }
    let cp = sub(p, c);
    let (d5, d6) = (dot(ab, cp), dot(ac, cp));
    if d6 >= 0.0 && d5 <= d6 {
        return c;
    }
    let vb = d5.mul_add(d2, -(d1 * d6));
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return [
            w.mul_add(ac[0], a[0]),
            w.mul_add(ac[1], a[1]),
            w.mul_add(ac[2], a[2]),
        ];
    }
    let va = d3.mul_add(d6, -(d5 * d4));
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        let bc = sub(c, b);
        return [
            w.mul_add(bc[0], b[0]),
            w.mul_add(bc[1], b[1]),
            w.mul_add(bc[2], b[2]),
        ];
    }
    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    [
        w.mul_add(ac[0], v.mul_add(ab[0], a[0])),
        w.mul_add(ac[1], v.mul_add(ab[1], a[1])),
        w.mul_add(ac[2], v.mul_add(ab[2], a[2])),
    ]
}

/// **O DESVIO RELATIVO DO COMPRIMENTO DE ARESTA** (`σ / média`) — a régua da
/// regularidade.
///
/// ⚠️ **É a grandeza que o artista de facto vê**, e nenhuma das asserções A1..A8
/// falava dela: uma malha pode ser 100 % quads, manifold, com `χ` certo e a forma
/// intacta, e ainda assim ter quads de tamanhos visivelmente diferentes. *Uma
/// grade regular é uma afirmação sobre a DISTRIBUIÇÃO, não sobre a contagem.*
#[must_use]
pub fn edge_length_spread(mesh: &Mesh) -> f32 {
    let p = mesh.positions();
    let mut lens: Vec<f32> = Vec::new();
    for f in mesh.faces() {
        let v = f.verts();
        for i in 0..v.len() {
            let d = sub(p[v[i] as usize], p[v[(i + 1) % v.len()] as usize]);
            lens.push(dot(d, d).sqrt());
        }
    }
    if lens.is_empty() {
        return 0.0;
    }
    let mean = lens.iter().sum::<f32>() / lens.len() as f32;
    if mean <= 0.0 {
        return 0.0;
    }
    let var = lens.iter().map(|l| (l - mean) * (l - mean)).sum::<f32>() / lens.len() as f32;
    var.sqrt() / mean
}

#[cfg(test)]
#[path = "relax_tests.rs"]
mod tests;
