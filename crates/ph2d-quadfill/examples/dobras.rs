//! ⭐⭐⭐ **AS DOBRAS DE UMA MALHA, pela lei da casa** — o instrumento que faltava fora da sonda.
//!
//! ```bash
//! cargo run -p ph2d-quadfill --example dobras -- <malha.obj> [outra.obj …]
//! ```
//!
//! Imprime, por ficheiro: faces que apontam **contra a vizinhança**
//! ([`ph2d_quadfill::folded_by_neighbours`]), o maior **grupo** delas (é o grupo que se vê como
//! fenda, não a contagem), as gravatas, e a forma mediana.
//!
//! ⛔ **Por que ele existe:** em 2026-09-03 o dono fotografou uma fenda no flanco de um espinho.
//! A malha estava topologicamente perfeita (`χ = 2`, zero bordo, zero não-manifold) e o defeito
//! era **cinco dobras de até 180°** no mesmo ponto. As duas réguas de dobra existem nesta crate
//! desde sempre — e o selector do botão **nunca as consultou**. *Uma régua na prateleira não
//! protege ninguém* (`CLAUDE.md` §5.0).

use std::collections::BTreeMap;

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    // ⭐ `--curar <superficie.obj>` aplica o [`ph2d_quadfill::untangle_bowties`] a cada malha e
    // imprime o antes e o depois — o instrumento que responde *«por que este reparo nao pegou?»*
    // sem pagar uma corrida inteira do botao.
    let mut superficie: Option<ph2d_mesh::Mesh> = None;
    if let Some(i) = args.iter().position(|a| a == "--curar") {
        let caminho = args.get(i + 1).cloned().unwrap_or_default();
        args.drain(i..=i + 1);
        let texto = std::fs::read_to_string(&caminho).expect("a superficie");
        superficie = ph2d_mesh::import_obj(&texto)
            .ok()
            .and_then(|v| v.into_iter().next())
            .map(|p| p.mesh);
    }
    if args.is_empty() {
        eprintln!("uso: cargo run -p ph2d-quadfill --example dobras -- <malha.obj> [outra.obj]");
        return;
    }
    for path in &args {
        let Ok(text) = std::fs::read_to_string(path) else {
            eprintln!("{path}: nao consegui ler");
            continue;
        };
        let Ok(pecas) = ph2d_mesh::import_obj(&text) else {
            eprintln!("{path}: nao e' um OBJ que eu leia");
            continue;
        };
        let Some(peca) = pecas.into_iter().next() else {
            eprintln!("{path}: sem peca");
            continue;
        };
        let mut mesh = peca.mesh;
        if let Some(sup) = superficie.as_ref() {
            let antes_d = ph2d_quadfill::quality::folded_faces_by_neighbours(&mesh).len();
            let antes_g = ph2d_quadfill::local_shape(&mesh).0.bowties;
            let curadas = ph2d_quadfill::untangle_bowties(
                &mut mesh,
                sup,
                ph2d_quadfill::untangle::UNTANGLE_TRAVEL,
            ) + ph2d_quadfill::untangle::remove_flaps(&mut mesh, sup);
            println!(
                "   CURAR: {curadas} face(s) reparada(s) · gravatas {antes_g} -> {} · dobras {antes_d} -> {}",
                ph2d_quadfill::local_shape(&mesh).0.bowties,
                ph2d_quadfill::quality::folded_faces_by_neighbours(&mesh).len(),
            );
        }
        let dobradas = ph2d_quadfill::quality::folded_faces_by_neighbours(&mesh);
        let (forma, _) = ph2d_quadfill::local_shape(&mesh);
        let shape = ph2d_quadfill::quad_shape(&mesh);
        let nome = path.rsplit('/').next().unwrap_or(path);
        println!(
            "{nome}: {} faces | DOBRAS {} (maior grupo {}) | gravatas {} | enviesamento p50 {:.1} | aspecto p50 {:.2}",
            mesh.face_count(),
            dobradas.len(),
            maior_grupo(&mesh, &dobradas),
            forma.bowties,
            shape.skew_p50,
            shape.aspect_p50,
        );
    }
}

/// ⭐⭐⭐ **O MAIOR GRUPO de faces dobradas, e não a contagem** — e a distinção é a que o report
/// do dono exige: uma dobra isolada num vinco real da escultura não se vê; **cinco no mesmo
/// ponto** são a fenda que ele fotografou.
fn maior_grupo(mesh: &ph2d_mesh::Mesh, dobradas: &[u32]) -> usize {
    if dobradas.is_empty() {
        return 0;
    }
    let alvo: std::collections::BTreeSet<u32> = dobradas.iter().copied().collect();
    let mut por_aresta: BTreeMap<(u32, u32), Vec<u32>> = BTreeMap::new();
    for (i, f) in mesh.faces().iter().enumerate() {
        let i = u32::try_from(i).unwrap_or(0);
        if !alvo.contains(&i) {
            continue;
        }
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            por_aresta.entry((a.min(b), a.max(b))).or_default().push(i);
        }
    }
    let mut viz: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    for quem in por_aresta.values() {
        for &i in quem {
            for &j in quem {
                if i != j {
                    viz.entry(i).or_default().push(j);
                }
            }
        }
    }
    let mut vistos: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
    let mut maior = 0usize;
    for &s in &alvo {
        if vistos.contains(&s) {
            continue;
        }
        let mut n = 0usize;
        let mut pilha = vec![s];
        vistos.insert(s);
        while let Some(u) = pilha.pop() {
            n += 1;
            for &w in viz.get(&u).map(Vec::as_slice).unwrap_or(&[]) {
                if vistos.insert(w) {
                    pilha.push(w);
                }
            }
        }
        maior = maior.max(n);
    }
    maior
}
