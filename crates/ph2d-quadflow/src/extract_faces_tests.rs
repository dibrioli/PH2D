//! **OS GATES DO FECHO SEM LEQUE** — o que a foto do Enio cobrou (2026-08-19).
//!
//! ⚠️ **O leque não era um bug de contagem, e é por isso que nenhum gate o via.**
//! A malha saía manifold, orientada, com `χ` certo e com a fração de quads
//! honesta — e ainda assim o artista via uma peça **espetada**, porque `n − 2`
//! triângulos ancorados no mesmo vértice são agulhas. *Nenhuma das asserções
//! A1..A8 fala sobre a FORMA de uma face.*
//!
//! ⚠️ **E a PROVA DE MUTAÇÃO desta folha diz a mesma coisa outra vez.** Repondo
//! o leque no [`super::fan_free_closure`], destes quatro gates **dois ficam
//! verdes**: o leque *também* particiona o polígono e *também* preserva `χ` — ele
//! é uma triangulação legítima. Quem o vê é o que mede **forma**
//! ([`no_face_is_a_needle`]) e o que conta a lei ([`the_closure_emits_quads_and_at_most_one_triangle`]).
//! *Um gate estrutural não substitui um gate de forma; ele não está a olhar.*

use std::collections::BTreeMap;

use ph2d_mesh::Face;

use super::fan_free_closure;

/// Um ciclo de `n` lados sobre um círculo unitário — a fixtura mais simples que
/// contém o fenômeno.
fn ring(n: usize) -> (Vec<[f32; 3]>, Vec<u32>) {
    let verts: Vec<[f32; 3]> = (0..n)
        .map(|i| {
            let a = core::f32::consts::TAU * i as f32 / n as f32;
            [a.cos(), a.sin(), 0.0]
        })
        .collect();
    let cycle: Vec<u32> = (0..n as u32).collect();
    (verts, cycle)
}

/// ⭐ **TODA ARESTA DO CICLO É USADA UMA VEZ, E NUMA SÓ FACE.**
///
/// É a asserção que faz do fecho uma **partição** do n-gon: se uma aresta da
/// fronteira fosse usada duas vezes a saída teria uma face a mais por cima; se
/// nenhuma vez, um buraco.
#[test]
fn the_closure_partitions_the_polygon() {
    for n in 5..=12usize {
        let (mut verts, cycle) = ring(n);
        let mut faces: Vec<Face> = Vec::new();
        fan_free_closure(&cycle, &mut verts, &mut faces);

        let mut dir: BTreeMap<(u32, u32), usize> = BTreeMap::new();
        for f in &faces {
            let v = f.verts();
            for i in 0..v.len() {
                *dir.entry((v[i], v[(i + 1) % v.len()])).or_insert(0) += 1;
            }
        }
        for i in 0..n as u32 {
            let e = (i, (i + 1) % n as u32);
            assert_eq!(
                dir.get(&e).copied().unwrap_or(0),
                1,
                "n={n}: a aresta da fronteira {e:?} foi percorrida {:?} vezes, e tem de ser UMA",
                dir.get(&e)
            );
        }
        assert!(
            dir.values().all(|c| *c == 1),
            "n={n}: alguma aresta dirigida aparece duas vezes -- duas faces a percorrem no mesmo \
             sentido, e as normais delas apontam para lados opostos"
        );
    }
}

/// ⭐ **A CARACTERÍSTICA DE EULER ATRAVESSA O FECHO.**
///
/// Um n-gon vale `V = n`, `E = n`, `F = 1`. Depois do fecho tem de valer o mesmo
/// `χ` — senão a A3 (o gênero sobrevive) passaria a medir o fecho em vez da
/// topologia da entrada.
#[test]
fn the_closure_preserves_the_euler_characteristic() {
    for n in 5..=12usize {
        let (mut verts, cycle) = ring(n);
        let mut faces: Vec<Face> = Vec::new();
        fan_free_closure(&cycle, &mut verts, &mut faces);

        let mut edges = std::collections::BTreeSet::new();
        for f in &faces {
            let v = f.verts();
            for i in 0..v.len() {
                let (a, b) = (v[i], v[(i + 1) % v.len()]);
                edges.insert(if a < b { (a, b) } else { (b, a) });
            }
        }
        let chi = verts.len() as i64 - edges.len() as i64 + faces.len() as i64;
        assert_eq!(
            chi, 1,
            "n={n}: o fecho devolveu chi={chi} sobre um disco, que vale 1 -- ele criou ou fundiu \
             topologia"
        );
    }
}

/// ⭐ **O LEQUE MORREU: no máximo UM triângulo, e ele só existe se `n` é ímpar.**
///
/// ⚠️ **É O GATE QUE A FOTO PEDIU.** O leque devolvia `n − 2` triângulos —
/// **dez** num ciclo de 12, **quarenta e dois** num de 44 —, todos ancorados no
/// vértice `0`. Aqui a conta é `⌈n/2⌉` faces, e o número de triângulos é `n % 2`.
#[test]
fn the_closure_emits_quads_and_at_most_one_triangle() {
    for n in 5..=12usize {
        let (mut verts, cycle) = ring(n);
        let mut faces: Vec<Face> = Vec::new();
        fan_free_closure(&cycle, &mut verts, &mut faces);

        let tris = faces.iter().filter(|f| f.verts().len() == 3).count();
        let quads = faces.iter().filter(|f| f.verts().len() == 4).count();
        assert_eq!(
            tris,
            n % 2,
            "n={n}: sairam {tris} triangulos, e a lei do fecho da' {} -- se sao muitos, o LEQUE \
             voltou (ele dava n-2 = {})",
            n % 2,
            n - 2
        );
        assert_eq!(
            quads + tris,
            n.div_ceil(2),
            "n={n}: sairam {} faces e a lei do fecho da' {}",
            quads + tris,
            n.div_ceil(2)
        );
    }
}

/// ⭐ **NENHUMA FACE É UMA AGULHA** — a razão entre o maior e o menor lado.
///
/// ⚠️ **É a propriedade que o artista de facto vê**, e a única aqui que fala de
/// GEOMETRIA e não de contagem. Um leque sobre um ciclo de 12 lados devolve
/// triângulos cuja razão de lados passa de **3,7**; o fecho pelo centro mantém
/// todas as faces abaixo de **2**, porque cada uma cobre dois lados do polígono e
/// dois raios.
#[test]
fn no_face_is_a_needle() {
    for n in 5..=12usize {
        let (mut verts, cycle) = ring(n);
        let mut faces: Vec<Face> = Vec::new();
        fan_free_closure(&cycle, &mut verts, &mut faces);

        let mut worst = 0.0f32;
        for f in &faces {
            let v = f.verts();
            let (mut lo, mut hi) = (f32::MAX, 0.0f32);
            for i in 0..v.len() {
                let (a, b) = (verts[v[i] as usize], verts[v[(i + 1) % v.len()] as usize]);
                let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
                let len = d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt();
                lo = lo.min(len);
                hi = hi.max(len);
            }
            worst = worst.max(hi / lo.max(1.0e-6));
        }
        assert!(
            worst < 2.5,
            "n={n}: a pior face tem razao de lados {worst:.2} -- ela e' uma AGULHA, e um leque \
             de agulhas e' o objeto espetado da foto (MEDIDO com o fecho: abaixo de 2,0)"
        );
    }
}
