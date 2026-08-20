//! **OS GATES DA HIERARQUIA** (ADR-0160, Q3.5).

use ph2d_mesh::shapes;

use super::{COARSEST, Hierarchy};

/// ⭐ **CADA NÍVEL REDUZ, E A PILHA TERMINA.**
///
/// ⚠️ **A terminação não é óbvia e o teto de níveis existe por isso:** um
/// emparelhamento que parasse de reduzir (uma componente de um vértice só)
/// deixaria o laço a construir níveis idênticos para sempre. O gate mede as duas
/// metades — que reduz, e que chega ao fim.
#[test]
fn every_level_is_strictly_smaller_and_the_stack_terminates() {
    for (name, mesh) in [
        ("esfera", shapes::uv_sphere(32, 48, 1.0)),
        ("toro", shapes::torus(48, 24, 1.0, 0.35)),
        ("cubo", shapes::cube(1.0)),
    ] {
        let h = Hierarchy::build(&mesh);
        assert!(h.depth() >= 1, "{name}: a pilha nasceu vazia");
        for l in 1..h.depth() {
            assert!(
                h.level(l).len() < h.level(l - 1).len(),
                "{name}: o nivel {l} ({}) nao e' menor que o {} ({})",
                h.level(l).len(),
                l - 1,
                h.level(l - 1).len()
            );
        }
        let top = h.level(h.depth() - 1).len();
        eprintln!(
            "[quadflow] {name}: {} niveis, {} -> {top} vertices",
            h.depth(),
            h.level(0).len()
        );
        assert!(
            top <= COARSEST || h.depth() == 1,
            "{name}: o topo ficou com {top} vertices e o alvo e' <= {COARSEST}"
        );
    }
}

/// **TODO VÉRTICE TEM PAI, E O PAI EXISTE.**
///
/// ⚠️ Um `parent` fora de alcance é um índice que a prolongação lê sem erro em
/// release — e o campo do filho passa a vir de um vértice que não é o dele, num
/// canto do modelo, em silêncio.
#[test]
fn every_vertex_has_a_parent_that_exists() {
    let mesh = shapes::uv_sphere(24, 32, 1.0);
    let h = Hierarchy::build(&mesh);
    for l in 0..h.depth() - 1 {
        let lv = h.level(l);
        assert_eq!(
            lv.parent.len(),
            lv.len(),
            "o nivel {l} nao tem um pai por vertice"
        );
        let up = h.level(l + 1).len();
        for (v, &p) in lv.parent.iter().enumerate() {
            assert!(
                (p as usize) < up,
                "o vertice {v} do nivel {l} aponta para o pai {p}, e o nivel de cima tem {up}"
            );
        }
    }
}

/// **A VIZINHANÇA INDUZIDA É SIMÉTRICA E SEM LAÇO PRÓPRIO.**
///
/// ⚠️ Uma aresta que só um lado conhece quebra a suavização: o acumulador de um
/// vértice vê o vizinho, o do vizinho não o vê de volta, e o campo fica com uma
/// assimetria que nenhuma medição de energia distingue de uma malha torta.
#[test]
fn the_induced_adjacency_is_symmetric() {
    let mesh = shapes::torus(48, 24, 1.0, 0.35);
    let h = Hierarchy::build(&mesh);
    for l in 0..h.depth() {
        let lv = h.level(l);
        for v in 0..lv.len() {
            for link in &lv.adjacency[v] {
                let w = link.id as usize;
                assert_ne!(w, v, "o nivel {l} tem um laco proprio no vertice {v}");
                assert!(
                    lv.adjacency[w].iter().any(|l| l.id == v as u32),
                    "o nivel {l}: {v} conhece {w}, e {w} nao conhece {v}"
                );
                // ⚠️ **E o PESO tem de ser o mesmo dos dois lados** — o
                // Laplaciano cotangente é simétrico por construção, e uma
                // agregação que somasse só de um lado partiria a simetria sem
                // que a contagem de vizinhos visse nada.
                let back = lv.adjacency[w]
                    .iter()
                    .find(|l| l.id == v as u32)
                    .expect("acabou de se verificar que existe");
                assert!(
                    (back.weight - link.weight).abs() <= 1.0e-4 * link.weight.abs().max(1.0),
                    "o nivel {l}: o peso de {v}->{w} e' {} e o de volta e' {}",
                    link.weight,
                    back.weight
                );
            }
        }
    }
}
