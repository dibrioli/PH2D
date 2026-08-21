//! **SONDA — de que ESPÉCIE são os 52 cantos do layout?**
//!
//! ```text
//! cd .../Worktrees/line-sculpt3d && cargo test -p ph2d-trace --release \
//!     --test corner_census -- --ignored --nocapture
//! ```
//!
//! ⭐ **A pergunta que a decomposição por origem do F5 levantou.** Dos 47 vértices
//! irregulares de uma esfera, **32 são cantos do layout** — e uma esfera admite
//! **8**. Antes de mexer no traçado é preciso saber de que espécie são os outros:
//!
//! | espécie | o que a cura seria |
//! |---|---|
//! | **singularidade** | nenhuma — são obrigatórias, e são 8 |
//! | **junção de paredes** (T ou X) | estrutura: fundir separatrizes (QuadWild §6) |
//! | ⚠️ **nem uma nem outra** | ⛔ artefacto do PASSEIO: a parede zigue-zagueia sobre arestas de malha e vira 90° sem que nada na estrutura tenha virado |
//!
//! *A terceira espécie não se cura com topologia nenhuma, e a primeira versão
//! deste plano ia gastar uma jornada a fundir separatrizes por causa dela.*
//!
//! ✅ **Respondido em 2026-08-21, e a resposta era a terceira linha:**
//!
//! | malha | cantos ANTES | artefactos | cantos DEPOIS | artefactos |
//! |---|---|---|---|---|
//! | esfera 96×144 + F1 | 52 | **27** | — | **0** |
//! | toro 64×32 + F1 | 72 | **37** | — | **0** |
//! | esfera 98 k + F1 | 68 | **37** | — | **0** |
//!
//! A cura é a porta estrutural em [`ph2d_trace::patches`] — *um canto só existe
//! onde a parede se ramifica*. ⚠️ **A sonda FICA**: ela é a única coisa que volta
//! a responder isto se alguém mexer no passeio, e a coluna dos artefactos é um
//! **zero que tem de continuar zero**.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_crossfield::{Dual, solve_miq};
use ph2d_mesh::{Mesh, shapes};
use ph2d_trace::{Walker, trace_patches};

fn census(name: &str, mut mesh: Mesh, iso: bool) {
    if iso {
        ph2d_remesh_iso::remesh_isotropic(&mut mesh, ph2d_remesh_iso::ALPHA);
    }
    mesh.triangulate();
    let dual = Dual::build(&mesh);
    let (field, _) = solve_miq(&dual);
    let walker = Walker::new(&mesh, &dual, &field);
    let (walls, _) = walker.trace_all();
    let singular: BTreeSet<u32> = walker.singular().into_iter().collect();
    let layout = trace_patches(&mesh, &dual, &field);

    // Os cantos são as PONTAS dos arcos — é exactamente o que o F5 marca como
    // `Provenance::Corner`, e é por isso que esta sonda é comparável com aquela.
    let mut corner: BTreeSet<u32> = BTreeSet::new();
    for chain in &layout.arc_chain {
        if let (Some(&a), Some(&b)) = (chain.first(), chain.last()) {
            corner.insert(a);
            corner.insert(b);
        }
    }

    // Quantos arcos chegam a cada canto — a valência dele no layout, que é a
    // valência que ele vai ter na malha de quads.
    let mut arcs_at: BTreeMap<u32, usize> = BTreeMap::new();
    for chain in &layout.arc_chain {
        if let (Some(&a), Some(&b)) = (chain.first(), chain.last()) {
            *arcs_at.entry(a).or_default() += 1;
            *arcs_at.entry(b).or_default() += 1;
        }
    }

    let mut kind = [0usize; 3]; // singularidade · junção de parede · artefacto
    let mut irregular = [0usize; 3];
    for &v in &corner {
        let deg = walls.degree.get(&v).copied().unwrap_or(0);
        // ⚠️ `degree` conta PONTAS de aresta de parede: 2 é o interior de uma
        // separatriz, 4 ou mais é uma junção. Um canto com grau 2 não tem
        // estrutura nenhuma por baixo — ele é só uma curva do passeio.
        let k = if singular.contains(&v) {
            0
        } else if deg > 2 {
            1
        } else {
            2
        };
        kind[k] += 1;
        if arcs_at.get(&v).copied().unwrap_or(0) != 4 {
            irregular[k] += 1;
        }
    }
    println!(
        "{name:<24} patches {:<4} arcos {:<5} CANTOS {:<4} = sing {} + juncao {} + \
         ⛔ARTEFACTO {}   | irregulares: sing {} juncao {} ARTEFACTO {}",
        layout.side_arcs.len(),
        layout.arc_length.len(),
        corner.len(),
        kind[0],
        kind[1],
        kind[2],
        irregular[0],
        irregular[1],
        irregular[2],
    );
}

#[test]
#[ignore = "sonda -- de que especie sao os cantos do layout"]
fn what_kind_of_corners_does_the_trace_make() {
    census("esfera 24x36", shapes::uv_sphere(24, 36, 1.0), false);
    census("esfera 48x72", shapes::uv_sphere(48, 72, 1.0), false);
    census("esfera 96x144 +F1", shapes::uv_sphere(96, 144, 1.0), true);
    census("toro 32x16", shapes::torus(32, 16, 1.0, 0.35), false);
    census("toro 64x32 +F1", shapes::torus(64, 32, 1.0, 0.35), true);
    census("esfera 98k +F1", shapes::sculpt_sphere(1.0), true);
}
