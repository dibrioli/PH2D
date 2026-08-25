//! ⛔⛔ **O VETO DA CADEIA** — ela corre, e só troca a malha se a troca for uma melhoria.
//!
//! ⚠️ **A fixtura é o ponto todo deste gate.** As três peças pedem **três vereditos diferentes**, e
//! uma fixtura de uma peça só não distingue *"a regra funciona"* de *"a regra devolve sempre a mesma
//! coisa"*.

use ph2d_mesh::{Face, Mesh};
use ph2d_quadchain::{Verdict, quads_or_keep};

/// Uma esfera em quads, pela parametrização UV — o caso **orgânico**, onde a cadeia ganha.
fn uv_sphere(nu: usize, nv: usize, r: f32) -> Mesh {
    let mut pos = Vec::new();
    for j in 0..=nv {
        let v = std::f32::consts::PI * j as f32 / nv as f32;
        for i in 0..nu {
            let u = std::f32::consts::TAU * i as f32 / nu as f32;
            pos.push([r * v.sin() * u.cos(), r * v.cos(), r * v.sin() * u.sin()]);
        }
    }
    let idx = |i: usize, j: usize| (j * nu + i % nu) as u32;
    let mut faces = Vec::new();
    for j in 0..nv {
        for i in 0..nu {
            faces.push(Face([
                idx(i, j),
                idx(i + 1, j),
                idx(i + 1, j + 1),
                idx(i, j + 1),
            ]));
        }
    }
    Mesh::from_parts(pos, faces).expect("a esfera")
}

/// Um cubo em quads, subdividido — o caso **duro**, onde a grade já é a resposta certa.
fn subdivided_cube(n: usize, half: f32) -> Mesh {
    let mut pos: Vec<[f32; 3]> = Vec::new();
    let mut faces = Vec::new();
    let mut index = std::collections::BTreeMap::new();
    let key = |p: [f32; 3]| {
        (
            (p[0] * 1.0e5) as i64,
            (p[1] * 1.0e5) as i64,
            (p[2] * 1.0e5) as i64,
        )
    };
    let mut vid = |p: [f32; 3], pos: &mut Vec<[f32; 3]>| -> u32 {
        *index.entry(key(p)).or_insert_with(|| {
            pos.push(p);
            (pos.len() - 1) as u32
        })
    };
    for axis in 0..3usize {
        for side in [-1.0f32, 1.0] {
            for a in 0..n {
                for b in 0..n {
                    let mut quad = [0u32; 4];
                    for (k, (da, db)) in [(0, 0), (1, 0), (1, 1), (0, 1)].into_iter().enumerate() {
                        let (ua, ub) = (
                            -half + 2.0 * half * (a + da) as f32 / n as f32,
                            -half + 2.0 * half * (b + db) as f32 / n as f32,
                        );
                        let mut p = [0.0f32; 3];
                        p[axis] = side * half;
                        p[(axis + 1) % 3] = ua;
                        p[(axis + 2) % 3] = ub;
                        quad[k] = vid(p, &mut pos);
                    }
                    faces.push(Face(quad));
                }
            }
        }
    }
    Mesh::from_parts(pos, faces).expect("o cubo")
}

/// ⛔⛔ **UM ESTOURO A JUSANTE NÃO DERRUBA QUEM PEDIU UMA MELHORIA** (W61b).
///
/// ⛔ **Medido:** um cubo subdividido — **fechado, manifold, 100 % quads** — faz o `ph2d-gridmap`
/// entrar em `index out of bounds: the len is 129 but the index is 157` (`solve.rs:336`). ⚠️ Não é
/// uma pré-condição que se possa conferir à porta: a malha satisfaz tudo o que se sabe exigir.
///
/// ⭐ Esta porta oferece uma **melhoria opcional**, e o veto já diz *"fica com a entrada a menos que
/// a saída seja melhor"* — um estouro é só mais uma forma de não ser melhor. ⛔ **Isto não é a
/// cura**: o defeito é do `ph2d-gridmap`, e a linha dele está viva sobre aquele arquivo.
#[test]
fn a_panic_downstream_does_not_take_down_the_caller() {
    let cube = subdivided_cube(12, 0.35);
    let target = ph2d_remesh_iso::target_edge(&cube, ph2d_remesh_iso::ALPHA);
    // ⚠️ O `panic` a jusante imprime o rasto dele; o que este gate afirma é que ele **volta como
    // veredito**.
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let (kept, v) = quads_or_keep(&cube, target);
    std::panic::set_hook(hook);
    assert!(
        !matches!(v, Verdict::Adopted(_)),
        "a cadeia foi adoptada numa peça de faces PLANAS — {v:?}"
    );
    assert_eq!(
        kept.face_count(),
        cube.face_count(),
        "a malha de entrada não voltou intacta depois do estouro"
    );
}

/// ⭐⭐ **O VETO DURO É O BURACO, e ele vem primeiro.**
///
/// ⚠️ Nenhum ganho de forma paga uma peça aberta — e é por isso que este ramo é testado **sozinho**:
/// numa fixtura só, «rejeitado por buraco» e «rejeitado por não melhorar» leem-se igual.
#[test]
fn a_hole_vetoes_before_any_shape_gain_is_considered() {
    // Uma calote: a esfera **sem** as últimas fileiras — ela já entra com bordo.
    let full = uv_sphere(40, 24, 0.4);
    let keep: Vec<Face> = full
        .faces()
        .iter()
        .copied()
        .take(full.face_count() * 3 / 4)
        .collect();
    let open = Mesh::from_parts(full.positions().to_vec(), keep).expect("a calote");
    let before = ph2d_quadchain::boundary_edges(&open);
    assert!(
        before > 0,
        "a fixtura não tem bordo — ela não contém o caso"
    );
    let target = ph2d_remesh_iso::target_edge(&open, ph2d_remesh_iso::ALPHA);
    let (kept, v) = quads_or_keep(&open, target);
    if let Verdict::Adopted(r) = &v {
        assert!(
            r.boundary_edges <= before,
            "a cadeia foi adoptada tendo AUMENTADO o bordo de {before} para {} — o veto duro não \
             mordeu",
            r.boundary_edges
        );
    } else {
        // Rejeitada ou sem ganho: a malha tem de voltar intacta.
        assert_eq!(kept.face_count(), open.face_count());
    }
}
