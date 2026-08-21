//! **OS GATES DO FLIP DE ARESTA** — filho do [`super`], não irmão.
//!
//! Eles têm um assunto só: *a troca de diagonal*, cujo operador vive no seu
//! próprio módulo ([`crate::dyntopo_flip`]). O pai julga a LEI do corte (a
//! ausência de rachadura, o alcance do pincel, o padrão do corte); aqui julgam-se
//! as **quatro recusas** do flip e o que ele existe para entregar.
//!
//! ⚠️ **FILHO e não irmão** para que `tri_sphere`, `cracks` e `scratch`
//! continuem sendo uma PORTA e não uma segunda cópia — a mesma razão do
//! [`super::splice`]. *Uma fixture duplicada é como dois gates passam a testar
//! duas malhas diferentes com o mesmo nome.*
//!
//! ⚠️ **Nasceram no ficheiro do pai e mudaram-se em 2026-08-21**, quando a quarta
//! recusa levou o `dyntopo_tests.rs` a **740 LOC** contra o teto de 700. ⛔ A cura
//! de um teto de ficheiro é o corte para o IRMÃO, nunca uma entrada na lista de
//! exceções.

use super::*;

/// **O FLIP PERGUNTA SÓ PELO QUE O CORTE MEXEU.**
///
/// ⚠️ **A fixture é construída para CONTER o fenômeno, e sem isso o gate seria
/// vazio:** uma esfera UV já é estável a flip fora da região do dab (medido — um
/// dab a 28k altera 3648 faces, todas dentro da esfera do pincel e nenhuma
/// fora), então sobre ela um flip global e um flip local dão o MESMO resultado e
/// nenhum oráculo os separa. Aqui um par de faces é deliberadamente virado para
/// a pior diagonal, longe de tudo: uma varredura global o encontraria e o
/// consertaria, uma varredura da região não.
///
/// As duas metades são independentes e as duas são precisas:
///
/// 1. **Sem sementes ele não sai à procura** — é o escopo.
/// 2. **Apontado para o estrago ele repara** — é o controle positivo, e sem ele
///    a primeira metade passaria com um operador que simplesmente não funciona.
#[test]
fn the_flip_asks_only_about_the_faces_the_cut_touched() {
    let (mut m, pair) = wreck_the_worst_pair(&tri_sphere(16, 24));
    assert!(
        has_pair(&m, pair),
        "o controle: a fixture tem de conter o estrago"
    );

    crate::dyntopo_flip::relax(&mut m, &[], &mut scratch());
    assert!(
        has_pair(&m, pair),
        "o flip varreu a malha atrás de trabalho que ninguém pediu"
    );

    let seed = face_of(&m, pair[0]).expect("a face estragada está na malha");
    crate::dyntopo_flip::relax(&mut m, &[seed], &mut scratch());
    assert!(
        !has_pair(&m, pair),
        "o operador tem de QUERER reparar isto — sem esta metade, a de cima é vazia"
    );
}

/// ⭐ **A RECUSA 4: duas trocas da mesma rodada não podem criar a MESMA
/// diagonal.**
///
/// ⚠️ **A recusa 2 não alcança isto, e a razão é o instante em que ela pergunta.**
/// Ela consulta o anel de `c` na adjacência de ENTRADA; duas trocas sobre pares
/// de faces disjuntos — logo invisíveis ao `spent`, que só protege a face —
/// produzem `c—d` sem que nenhuma das duas veja a outra. A malha sai com **duas
/// arestas entre o mesmo par**, o que aqui aparece como uma aresta de valência
/// **4** (a assinatura de *criada duas vezes*: uma criada por cima de uma que já
/// existia teria 3).
///
/// **A fixture é a MENOR que contém o fenómeno**, e isso foi medido, não
/// escolhido — uma rodada de `relax_valence`, com a recusa 4 desligada:
///
/// | fixture | vértices | trocas | arestas de valência ≠ 2 |
/// |---|---|---|---|
/// | `uv_sphere(*)` (lisa, qualquer tamanho) | — | 0 | 0 — *não flipa, não prova nada* |
/// | `uv_sphere_shuffled(48,72)` | 3 386 | 2 390 | **0** |
/// | ⭐ `uv_sphere_noisy(24,36)` | **830** | 457 | **7** |
/// | `uv_sphere_noisy(96,144)` | 13 682 | 8 590 | **185** |
/// | `uv_sphere_shuffled(96,144)` | 13 682 | 9 968 | 1 |
///
/// ⚠️ **A esfera LISA é o controle negativo que quase enganou:** ela não aceita
/// troca nenhuma (0 flips), então passaria com o operador inteiro apagado. É o
/// **ruído** que dá ao par `c,d` a valência alta de que a colisão precisa.
#[test]
fn a_round_of_flips_never_creates_the_same_diagonal_twice() {
    let mut m = shapes::uv_sphere_noisy(24, 36, 1.0, 0.02);
    m.triangulate();
    assert_eq!(cracks(&m), 0, "o controle: a fixture nasce variedade");

    let flips = crate::dyntopo_flip::relax_valence(&mut m, &mut scratch());
    // ⚠️ O controle POSITIVO: sem trocas nenhumas a asserção de baixo é vazia, e
    // é exatamente assim que uma esfera lisa passaria com o operador desligado.
    assert!(
        flips > 100,
        "a fixture tem de FLIPAR para que este gate afirme algo — só {flips}"
    );
    assert_eq!(
        cracks(&m),
        0,
        "duas trocas da mesma rodada criaram a mesma diagonal: a malha deixou de ser variedade"
    );
}

/// **Troca a diagonal do par vizinho cuja troca mais PIORA a qualidade** — o
/// estrago que o operador vai querer desfazer. Devolve a malha e as duas faces
/// novas por conjunto de vértices (que sobrevive a renumeração).
fn wreck_the_worst_pair(m: &Mesh) -> (Mesh, [[u32; 3]; 2]) {
    let pos = m.positions();
    let adj = m.adjacency();
    let src = m.faces();
    let mut best: Option<(f32, usize, usize, [u32; 4])> = None;
    for (i0, f0) in src.iter().enumerate() {
        if !f0.is_tri() {
            continue;
        }
        let v0 = f0.verts();
        for k in 0..3 {
            let (ea, eb) = (v0[k], v0[(k + 1) % 3]);
            let Some(i1) = adj
                .vert_faces
                .neighbours(ea as usize)
                .iter()
                .copied()
                .find(|&j| j as usize != i0 && src[j as usize].verts().contains(&eb))
                .map(|j| j as usize)
            else {
                continue;
            };
            if !src[i1].is_tri() {
                continue;
            }
            let Some((a, b, c, d)) = quad_of(src, i0, i1) else {
                continue;
            };
            let p = |v: u32| pos[v as usize];
            let old = min_angle(p(a), p(b), p(c)).min(min_angle(p(b), p(a), p(d)));
            let new = min_angle(p(a), p(d), p(c)).min(min_angle(p(d), p(b), p(c)));
            let loss = old - new;
            if best.is_none_or(|(l, ..)| loss > l) {
                best = Some((loss, i0, i1, [a, b, c, d]));
            }
        }
    }
    let (loss, i0, i1, [a, b, c, d]) = best.expect("a esfera tem pares vizinhos");
    assert!(
        loss > 1.0,
        "a fixture precisa de um par cuja troca piore de verdade: {loss} grau(s)"
    );
    let (n0, n1) = (Face::tri(a, d, c), Face::tri(d, b, c));
    let mut faces = src.to_vec();
    faces[i0] = n0;
    faces[i1] = n1;
    let wrecked = Mesh::from_parts(pos.to_vec(), faces).expect("a troca não inventa índice");
    (wrecked, [sorted(n0), sorted(n1)])
}

/// Os quatro cantos do quadrilátero de duas faces vizinhas — o mesmo desenho do
/// `dyntopo_flip::quad`, escrito aqui porque uma FIXTURE constrói um estado; ela
/// não pode chamar a função sob teste para decidir o que espera.
fn quad_of(faces: &[Face], i0: usize, i1: usize) -> Option<(u32, u32, u32, u32)> {
    let (t0, t1) = (faces[i0].verts(), faces[i1].verts());
    let k = (0..3).find(|&k| !t1.contains(&t0[k]))?;
    let c = t0[k];
    let (a, b) = (t0[(k + 1) % 3], t0[(k + 2) % 3]);
    let d = *t1.iter().find(|v| **v != a && **v != b)?;
    Some((a, b, c, d))
}

fn min_angle(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> f32 {
    let pts = [p0, p1, p2];
    let mut worst = 180.0f32;
    for k in 0..3 {
        let (o, u, v) = (pts[k], pts[(k + 1) % 3], pts[(k + 2) % 3]);
        let a = [u[0] - o[0], u[1] - o[1], u[2] - o[2]];
        let b = [v[0] - o[0], v[1] - o[1], v[2] - o[2]];
        let la = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
        let lb = (b[0] * b[0] + b[1] * b[1] + b[2] * b[2]).sqrt();
        if la < 1e-12 || lb < 1e-12 {
            return 0.0;
        }
        let c = ((a[0] * b[0] + a[1] * b[1] + a[2] * b[2]) / (la * lb)).clamp(-1.0, 1.0);
        worst = worst.min(c.acos().to_degrees());
    }
    worst
}

fn sorted(f: Face) -> [u32; 3] {
    let v = f.verts();
    let mut k = [v[0], v[1], v[2]];
    k.sort_unstable();
    k
}

fn face_of(m: &Mesh, key: [u32; 3]) -> Option<u32> {
    m.faces()
        .iter()
        .position(|f| f.is_tri() && sorted(*f) == key)
        .map(|i| u32::try_from(i).unwrap_or(u32::MAX))
}

fn has_pair(m: &Mesh, pair: [[u32; 3]; 2]) -> bool {
    pair.iter().all(|k| face_of(m, *k).is_some())
}
