//! **O FLIP DE ARESTA** — o operador que devolve a qualidade que o corte tirou.
//!
//! Irmão do [`crate::dyntopo`]: o corte e o flip são as duas metades de UM refino, e
//! separá-los em crates diferentes seria convidar alguém a rodar só o primeiro.
//!
//! # Por que ele existe, com o número ao lado
//!
//! Um refino que só PARTE não pode manter a forma dos triângulos, e a razão é
//! estrutural: para não deixar rachadura, a vizinha de uma face escolhida tem de
//! aprender o vértice novo — e uma face com **uma** aresta partida vira dois
//! triângulos com metade do ângulo daquele canto (o corte "verde"). Isso não é
//! um caso raro: é o ANEL inteiro em volta de tudo o que o pincel toca, a cada
//! dab.
//!
//! Medido num traço de 24 dabs sobre a esfera de smoke, pelo pior ângulo mínimo
//! de triângulo — as duas saídas que existem sem este operador:
//!
//! | esquema | pior ângulo | abaixo de 10° | vértices no hemisfério NÃO tocado |
//! |---|---|---|---|
//! | controle (sem refino) | 21,21° | 0,0% | 57 |
//! | escolha por ARESTA | 1,53° | 15,0% | 57 |
//! | escolha por FACE | 0,59° | 48,0% | 57 |
//! | + promover a vizinha a 1→4 | **20,47°** | 0,0% | **846** |
//!
//! ⚠️ **A última linha é a tentação, e ela foi MEDIDA e REJEITADA:** dar à
//! vizinha a mesma garantia da escolhida cascateia pela malha inteira — 846
//! vértices onde o artista nunca passou, ou seja o **oposto exato** da promessa
//! deste modo. Qualidade é global; a pegada não pode ser.
//!
//! O flip fecha a tensão porque ele é **local e não propaga**: ele não cria nem
//! remove vértice, só troca a diagonal de dois triângulos vizinhos quando isso
//! deixa o pior ângulo dos dois melhor. É o operador de sempre do remalhamento
//! incremental (Botsch–Kobbelt: *split · collapse · **flip** · smooth*), e a
//! metade `collapse` é a wave seguinte.
//!
//! # As três recusas, e as três são geometria
//!
//! 1. **Aresta de borda** (uma face só) não tem diagonal a trocar.
//! 2. **A diagonal nova já existe** — flipar criaria uma segunda aresta entre o
//!    mesmo par, e a malha deixaria de ser variedade. Acontece de verdade num
//!    tetraedro e em qualquer anel de valência 3.
//! 3. **A troca DOBRARIA a superfície** — se uma das faces novas aponta para o
//!    lado oposto do par antigo, o flip é uma dobra e não uma melhoria. Sem esta
//!    guarda um vinco afiado é "melhorado" para um bico invertido.

use crate::dyntopo::rebuild_with_faces;
use crate::face::Face;
use crate::mesh::Mesh;

/// Quantas rodadas de flip um refino pode gastar.
///
/// ⚠️ **É um teto de RECURSO, e o recurso é o quadro** — o mesmo argumento do
/// `MAX_PASSES` do corte. Medido, a segunda rodada já quase não acha o que
/// trocar (o critério é estritamente melhorar, então o processo drena); três é
/// folga para o caso em que um dab inteiro nasce de uma vez.
const MAX_ROUNDS: usize = 3;

/// O ganho mínimo, em cosseno do ângulo, para uma troca valer a pena.
///
/// ⚠️ **Um limiar estritamente positivo é o que garante que isto TERMINA**: cada
/// troca aceita aumenta o pior ângulo do par, e um empate não é aceito — então
/// não há ciclo de duas arestas trocando uma para a outra para sempre.
const MIN_GAIN: f32 = 1e-4;

/// **Relaxa a malha por troca de diagonal.** Devolve quantas arestas trocaram.
///
/// Não move vértice nenhum e não muda a contagem — só a ligação. É por isso que
/// ele pode rodar depois do corte sem falar com o traço em voo: os índices dos
/// vértices ficam exatamente onde estavam, que é a mesma premissa de que o
/// `SculptStroke::grow_with` depende.
pub(crate) fn relax(mesh: &mut Mesh) -> usize {
    let mut total = 0;
    for _ in 0..MAX_ROUNDS {
        let n = one_round(mesh);
        total += n;
        if n == 0 {
            break;
        }
    }
    total
}

/// Uma rodada. Devolve quantas trocas aconteceram.
fn one_round(mesh: &mut Mesh) -> usize {
    let edges = mesh.edges();
    let pos = mesh.positions();
    if !mesh.faces().iter().all(Face::is_tri) {
        return 0;
    }

    // Aresta → as (no máximo duas) faces que a dividem. É a adjacência que o
    // `Edges` não guarda, e construí-la aqui custa uma varredura das faces — a
    // mesma ordem do resto do passe.
    let mut side = vec![[u32::MAX; 2]; edges.len()];
    for (fi, face) in mesh.faces().iter().enumerate() {
        for k in 0..face.verts().len() {
            let Some(e) = edges.face_edge(fi, k) else {
                continue;
            };
            let slot = &mut side[e as usize];
            if slot[0] == u32::MAX {
                slot[0] = fi as u32;
            } else if slot[1] == u32::MAX {
                slot[1] = fi as u32;
            }
        }
    }

    let mut faces: Vec<Face> = mesh.faces().to_vec();
    // Uma face só pode entrar numa troca por rodada: as duas trocas leriam a
    // mesma face pela versão ANTIGA e a segunda escreveria por cima da primeira.
    let mut spent = vec![false; faces.len()];
    let mut flips = 0;

    for &[f0, f1] in &side {
        if f1 == u32::MAX {
            continue;
        }
        let (i0, i1) = (f0 as usize, f1 as usize);
        if spent[i0] || spent[i1] {
            continue;
        }
        let Some((a, b, c, d)) = quad(&faces, i0, i1) else {
            continue;
        };
        // A diagonal nova já existe em outro lugar da malha? Ver a recusa 2.
        if edges.id_of(mesh.adjacency(), c, d).is_some() {
            continue;
        }
        let p = |v: u32| pos[v as usize];
        let old = worst_angle(p(a), p(b), p(c)).min(worst_angle(p(b), p(a), p(d)));
        let new = worst_angle(p(a), p(d), p(c)).min(worst_angle(p(d), p(b), p(c)));
        if new <= old + MIN_GAIN {
            continue;
        }
        if folds(p(a), p(b), p(c), p(d)) {
            continue;
        }
        faces[i0] = Face::tri(a, d, c);
        faces[i1] = Face::tri(d, b, c);
        spent[i0] = true;
        spent[i1] = true;
        flips += 1;
    }

    if flips == 0 {
        return 0;
    }
    rebuild_with_faces(mesh, faces);
    flips
}

/// Os quatro cantos do quadrilátero que duas faces vizinhas formam:
/// `(a, b, c, d)` com a diagonal atual em `a—b`, `c` oposto na primeira face e
/// `d` na segunda. O contorno é `a → d → b → c`.
fn quad(faces: &[Face], i0: usize, i1: usize) -> Option<(u32, u32, u32, u32)> {
    let (t0, t1) = (faces[i0].verts(), faces[i1].verts());
    // O canto de `t0` que não está em `t1` é o `c`; os outros dois são a aresta.
    let k = (0..3).find(|&k| !t1.contains(&t0[k]))?;
    let c = t0[k];
    let (a, b) = (t0[(k + 1) % 3], t0[(k + 2) % 3]);
    // ⚠️ A ordem importa: em `t0` o percurso é `c → a → b`, então a aresta
    // compartilhada corre `a → b` aqui e `b → a` na vizinha, e é isso que dá o
    // contorno `a → d → b → c` do quadrilátero.
    let d = *t1.iter().find(|v| **v != a && **v != b)?;
    Some((a, b, c, d))
}

/// O menor ângulo do triângulo, em COSSENO invertido — devolvemos o cosseno do
/// maior ângulo negado para comparar sem `acos`, que é transcendental e não
/// muda a ordem. Maior valor = triângulo melhor.
fn worst_angle(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> f32 {
    let pts = [p0, p1, p2];
    let mut worst = f32::INFINITY;
    for k in 0..3 {
        let (o, u, v) = (pts[k], pts[(k + 1) % 3], pts[(k + 2) % 3]);
        let a = sub(u, o);
        let b = sub(v, o);
        let (la, lb) = (norm(a), norm(b));
        if la < 1e-12 || lb < 1e-12 {
            return -1.0;
        }
        // O ângulo cresce quando o cosseno cai, então o PIOR canto é o de menor
        // `-cos` — é a mesma ordem de `min(ângulo)`, sem chamar `acos`.
        worst = worst.min(-(dot(a, b) / (la * lb)));
    }
    worst
}

/// A troca dobraria a superfície? Ver a recusa 3.
fn folds(a: [f32; 3], b: [f32; 3], c: [f32; 3], d: [f32; 3]) -> bool {
    let before = add(tri_normal(a, b, c), tri_normal(b, a, d));
    let (n0, n1) = (tri_normal(a, d, c), tri_normal(d, b, c));
    dot(n0, before) <= 0.0 || dot(n1, before) <= 0.0
}

fn tri_normal(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> [f32; 3] {
    cross(sub(p1, p0), sub(p2, p0))
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn norm(a: [f32; 3]) -> f32 {
    dot(a, a).sqrt()
}
