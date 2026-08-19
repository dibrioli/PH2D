//! **DOS CICLOS ÀS FACES** — a metade do [`super`] que fala de POLÍGONOS.
//!
//! Irmão (`#[path]`), e o corte é de ASSUNTO: lá mora *que células existem e
//! quais são vizinhas* (o grafo); aqui *que polígonos esse grafo delimita*. As
//! duas metades crescem por motivos diferentes — a de lá com cada lei de
//! agrupamento, esta com cada patologia que um passeio de faces pode encontrar.
//!
//! ⚠️ **E o corte foi FORÇADO pelo teto de LOC do workspace (700):** o pai
//! chegou a **839**. A cura de um teto é um corte para o IRMÃO, nunca uma
//! allowlist.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_mesh::Face;

use crate::orientation::OrientationField;

use super::{cross, dot, normalize_or, sub};

/// **A PODA DOS PENDENTES** — todo nó de uma grade fechada tem grau ≥ 3.
///
/// ⚠️ **Sem ela o sistema de rotação não descreve uma superfície, e a
/// característica de Euler o denuncia.** Um nó de grau 1 faz o passeio ir `a→b` e
/// voltar `b→a`: um ciclo de DOIS, que não delimita área. O ciclo é descartado
/// (uma face de dois lados não existe) mas as duas arestas dirigidas já foram
/// consumidas — e passam a não pertencer a face nenhuma. A soma dos comprimentos
/// das faces deixa de ser `2E`, e `χ = V − E + F` sai **12** numa esfera, onde o
/// máximo de uma superfície conexa é **2**.
///
/// ⚠️ **Iterativa, e é obrigatório:** podar um pendente pode deixar o vizinho com
/// grau 2, e esse com grau 1. Uma passada só deixaria a cauda pela metade.
pub(super) fn prune_dangling(graph: &mut [BTreeSet<u32>]) {
    let mut changed = true;
    while changed {
        changed = false;
        for c in 0..graph.len() {
            if graph[c].is_empty() || graph[c].len() >= 3 {
                continue;
            }
            let gone: Vec<u32> = graph[c].iter().copied().collect();
            graph[c].clear();
            for w in gone {
                graph[w as usize].remove(&(c as u32));
            }
            changed = true;
        }
    }
}

/// **AS FACES, pelo SISTEMA DE ROTAÇÃO.**
///
/// Em cada célula os vizinhos são ordenados por ângulo no plano tangente. Uma
/// face é o passeio que, ao chegar a `v` vindo de `u`, sai pelo vizinho
/// **seguinte** de `u` na ordem angular de `v` — sempre virando para o mesmo
/// lado. Numa superfície orientável fechada isso parte as `2E` arestas dirigidas
/// exatamente nas `F` faces.
///
/// ⚠️ **É por isso que a normal da célula é load-bearing**: ela dá o *sentido*
/// da ordem angular. Uma normal virada numa célula inverte o passeio ali, e a
/// face que passa por ela sai com o comprimento errado — não com o sinal
/// errado, o que a torna difícil de reconhecer.
pub(super) fn trace_faces(
    graph: &[BTreeSet<u32>],
    verts: &[[f32; 3]],
    normals: &[[f32; 3]],
    _orient: &OrientationField,
) -> Vec<Vec<u32>> {
    // A ordem angular de cada célula, e o índice inverso.
    let mut order: Vec<Vec<u32>> = Vec::with_capacity(graph.len());
    let mut index: Vec<BTreeMap<u32, usize>> = Vec::with_capacity(graph.len());
    for (c, ns) in graph.iter().enumerate() {
        let n = normals[c];
        let e1 = tangent_of(n);
        let e2 = cross(n, e1);
        let mut ns: Vec<u32> = ns.iter().copied().collect();
        ns.sort_by(|a, b| {
            let ang = |x: u32| {
                let d = sub(verts[x as usize], verts[c]);
                dot(d, e2).atan2(dot(d, e1))
            };
            // ⚠️ `total_cmp` e não `partial_cmp`: um `NaN` numa coordenada
            // tornaria a ordenação não-determinística (ou um pânico), e a saída
            // desta função tem de ser byte-reprodutível.
            ang(*a).total_cmp(&ang(*b))
        });
        let mut m = BTreeMap::new();
        for (i, &w) in ns.iter().enumerate() {
            m.insert(w, i);
        }
        order.push(ns);
        index.push(m);
    }

    let mut seen: BTreeSet<(u32, u32)> = BTreeSet::new();
    let mut faces = Vec::new();
    for u in 0..graph.len() as u32 {
        for &v in &order[u as usize] {
            if seen.contains(&(u, v)) {
                continue;
            }
            let mut cycle = Vec::new();
            let (mut a, mut b) = (u, v);
            loop {
                if !seen.insert((a, b)) {
                    break;
                }
                cycle.push(a);
                let deg = order[b as usize].len();
                let Some(&i) = index[b as usize].get(&a) else {
                    break;
                };
                let next = order[b as usize][(i + 1) % deg];
                (a, b) = (b, next);
                if (a, b) == (u, v) {
                    break;
                }
                // Uma face maior que o grafo é um passeio que não fecha — só
                // acontece com o sistema de rotação inconsistente, e parar aqui
                // impede o laço infinito de esconder isso.
                if cycle.len() > graph.len() {
                    break;
                }
            }
            if cycle.len() >= 3 {
                faces.push(cycle);
            }
        }
    }
    faces
}

pub(super) fn tangent_of(n: [f32; 3]) -> [f32; 3] {
    let sign = 1.0f32.copysign(n[2]);
    let a = -1.0 / (sign + n[2]);
    let b = n[0] * n[1] * a;
    normalize_or(
        [sign.mul_add(n[0] * n[0] * a, 1.0), sign * b, -sign * n[0]],
        [1.0, 0.0, 0.0],
    )
}

/// **PARTE UM CICLO QUE SE PINÇA** — um passeio que volta ao mesmo vértice.
///
/// ⚠️ **Um ciclo que visita `X` duas vezes NÃO é um polígono**, e leque-triangulá-lo
/// produz a mesma aresta em mais de duas faces — a saída deixa de ser manifold
/// pela porta dos fundos. Medido na esfera: **6** arestas com três faces, e a
/// soma dos lados a passar `2E` em 20.
///
/// O pinçamento é a assinatura de um **vértice não-manifold** no grafo de
/// células: duas folhas da superfície a encontrarem-se num nó. Partir o ciclo no
/// vértice repetido é a leitura correta — cada folha fica com o seu polígono.
///
/// ⚠️ **A pilha, e não uma varredura:** um ciclo pode pinçar-se várias vezes e
/// aninhado (`A X B X C Y D Y`), e só uma pilha separa os laços na ordem certa.
pub(super) fn split_pinches(cycle: &[u32]) -> Vec<Vec<u32>> {
    let mut out = Vec::new();
    let mut stack: Vec<u32> = Vec::new();
    for &v in cycle {
        if let Some(at) = stack.iter().position(|&x| x == v) {
            // O laço fechado entre a visita anterior e esta.
            let loop_part: Vec<u32> = stack.split_off(at);
            if loop_part.len() >= 3 {
                out.push(loop_part);
            }
        }
        stack.push(v);
    }
    if stack.len() >= 3 {
        out.push(stack);
    }
    out
}

/// **JUNTA PARES DE TRIÂNGULOS VIZINHOS NUM QUAD.** Devolve `(faces, quantos
/// pares)`.
///
/// ⚠️ **Guloso e por ordem de face — determinístico.** Um emparelhamento ÓTIMO
/// (o casamento de peso máximo no grafo dual) daria alguns quads a mais e um
/// solver a justificar; o guloso resolve a esmagadora maioria porque os
/// triângulos aqui vêm **aos pares**, das mesmas células.
///
/// ⚠️ **A recusa por NORMAL é a guarda que impede um quad dobrado:** dois
/// triângulos que se encontram em ângulo agudo formam um quadrilátero não-planar
/// que a subdivisão e a iluminação leem como um vinco. O limiar é o cosseno de
/// 45°, e ele diz de que recurso é: a planaridade que um quad promete a quem o
/// subdivide.
pub(super) fn pair_triangles(faces: &[Face], verts: &[[f32; 3]]) -> (Vec<Face>, usize) {
    // Aresta → as faces triangulares que a usam.
    let mut owner: BTreeMap<(u32, u32), Vec<usize>> = BTreeMap::new();
    for (i, f) in faces.iter().enumerate() {
        let v = f.verts();
        if v.len() != 3 {
            continue;
        }
        for k in 0..3 {
            let (a, b) = (v[k], v[(k + 1) % 3]);
            let key = if a < b { (a, b) } else { (b, a) };
            owner.entry(key).or_default().push(i);
        }
    }

    let mut taken = vec![false; faces.len()];
    let mut out: Vec<Face> = Vec::with_capacity(faces.len());
    let mut merged = 0usize;
    let mut replacement: BTreeMap<usize, Face> = BTreeMap::new();

    for (&(a, b), who) in &owner {
        if who.len() != 2 {
            continue;
        }
        let (i, j) = (who[0], who[1]);
        if taken[i] || taken[j] {
            continue;
        }
        // Os dois vértices opostos à aresta partilhada.
        let opp = |f: &Face| -> u32 {
            *f.verts()
                .iter()
                .find(|v| **v != a && **v != b)
                .expect("um triangulo tem um vertice fora da aresta")
        };
        let (c, d) = (opp(&faces[i]), opp(&faces[j]));
        if c == d {
            continue;
        }
        // ⚠️ As normais têm de concordar: um par em ângulo agudo daria um quad
        // dobrado.
        let n0 = tri_normal(verts[a as usize], verts[b as usize], verts[c as usize]);
        let n1 = tri_normal(verts[b as usize], verts[a as usize], verts[d as usize]);
        if dot(n0, n1) < core::f32::consts::FRAC_1_SQRT_2 {
            continue;
        }
        taken[i] = true;
        taken[j] = true;
        merged += 1;
        // A ordem `c, a, d, b` percorre o quad: o oposto de um lado, a aresta, o
        // oposto do outro, e de volta.
        replacement.insert(i, Face::quad(c, a, d, b));
    }

    for (i, f) in faces.iter().enumerate() {
        if let Some(q) = replacement.get(&i) {
            out.push(*q);
        } else if !taken[i] {
            out.push(*f);
        }
    }
    (out, merged)
}

fn tri_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let ab = sub(b, a);
    let ac = sub(c, a);
    normalize_or(cross(ab, ac), [0.0, 0.0, 1.0])
}
