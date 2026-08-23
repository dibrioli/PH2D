//! ⭐⭐⭐ **OS GATES DO G2** — pentear, e o salto de período de cada costura.

use ph2d_mesh::{Mesh, shapes};

fn chain(mut mesh: Mesh) -> (Mesh, ph2d_trace::PatchLayout) {
    mesh.triangulate();
    ph2d_remesh_iso::remesh_isotropic(&mut mesh, ph2d_remesh_iso::ALPHA);
    mesh.triangulate();
    let dual = ph2d_crossfield::Dual::build(&mesh);
    let (field, _) = ph2d_crossfield::solve_miq(&dual);
    let layout = ph2d_trace::trace_patches(&mesh, &dual, &field);
    (mesh, layout)
}

/// ⭐ **TODO PATCH PENTEIA, E O SALTO LÊ-SE EM TODA COSTURA.**
///
/// ⚠️ As três colunas de recusa entram na asserção **cada uma por seu nome**: um patch
/// recusado pelo `comb`, um desalinhado, e uma costura sem salto são coisas diferentes
/// com curas diferentes. *«`0` penteados» sem elas não diz qual aconteceu.*
#[test]
fn every_patch_combs_and_every_seam_has_a_jump() {
    for (name, mesh) in [
        ("esfera lisa", shapes::uv_sphere(24, 36, 1.0)),
        ("toro", shapes::torus(32, 16, 1.0, 0.35)),
    ] {
        let (mesh, layout) = chain(mesh);
        let (cut, _) = crate::cut::cut_along_patches(&mesh, &layout);
        let (combed, rep) = super::comb_patches(&mesh, &layout, &cut);
        assert_eq!(
            rep.combed, rep.patches,
            "{name}: penteou {} de {} patches (recusados {}, desalinhados {})",
            rep.combed, rep.patches, rep.refused, rep.misaligned
        );
        assert_eq!(
            rep.jumps, rep.seams,
            "{name}: leu o salto em {} de {} costuras -- uma costura sem salto nao pode \
             ser acoplada, e um `0` no lugar dela acoplaria molduras desencontradas",
            rep.jumps, rep.seams
        );
        assert!(
            combed.jump.iter().all(Option::is_some),
            "{name}: ha' costura com salto `None`"
        );
    }
}

/// ⭐⭐⭐ **AS ARESTAS DE UMA COSTURA CONCORDAM SOBRE O SALTO.**
///
/// ⛔ O salto é uma propriedade da **costura inteira**, não de uma aresta. Discordar
/// significa que uma das duas molduras não é constante ao longo dela — e aí acoplar não
/// tem sentido.
///
/// ⚠️ **Foi este gate que apanhou o defeito de desenho:** a primeira versão escolhia as
/// duas faces pela ordem de armazenamento, e o **sinal** do salto depende de qual delas
/// é o lado `0`. As arestas da mesma costura saíam com sinais trocados ao acaso, e a
/// votação **escondia-o** em vez de o acusar.
///
/// # ⛔⛔ E a barra é sobre a POPULAÇÃO CERTA, não sobre todas
///
/// Um patch com singularidade **dentro** não é penteável — a moldura dele depende do
/// caminho — logo a costura dele **tem** de discordar, e nenhuma linha desta fase o pode
/// curar. Medido 2026-08-23:
///
/// | | patches sujos | costuras | inconsistentes |
/// |---|---|---|---|
/// | ⭐ esfera lisa | `0` | 42 | ⭐ **`0`** |
/// | toro | ⛔ `3` | 51 | `4` — **todas** tocam um sujo |
///
/// ⇒ a barra é `inconsistent_clean == 0`. ⚠️ *Uma barra sobre o total seria frouxa nos
/// dois sentidos: reprovaria sobre dívida do F3, e — baixada para o número do toro —
/// deixaria passar um defeito real na esfera.*
#[test]
fn the_edges_of_a_seam_agree_on_the_jump() {
    for (name, mesh) in [
        ("esfera lisa", shapes::uv_sphere(24, 36, 1.0)),
        ("toro", shapes::torus(32, 16, 1.0, 0.35)),
    ] {
        let (mesh, layout) = chain(mesh);
        let (cut, _) = crate::cut::cut_along_patches(&mesh, &layout);
        let (_, rep) = super::comb_patches(&mesh, &layout, &cut);
        assert_eq!(
            rep.inconsistent_clean, 0,
            "{name}: {} costuras com os DOIS lados limpos e arestas a discordar do salto \
             (de {} inconsistentes ao todo, com {} patches sujos)",
            rep.inconsistent_clean, rep.inconsistent, rep.dirty
        );
    }
}

/// ⭐⭐ **O PENTEADO É O CAMPO, um braço de cada vez.**
///
/// Cada direcção penteada tem de ser **unitária, tangente à face** e um dos quatro
/// braços da cruz original. ⚠️ *Um pentear que devolvesse outra direcção qualquer
/// passaria em todos os gates de contagem acima* — eles medem se ele correu, não se
/// respondeu o campo.
#[test]
fn the_combed_direction_is_still_the_field() {
    let (mesh, layout) = chain(shapes::uv_sphere(24, 36, 1.0));
    let (cut, _) = crate::cut::cut_along_patches(&mesh, &layout);
    let (combed, _) = super::comb_patches(&mesh, &layout, &cut);
    let pos = mesh.positions();
    let mut checked = 0usize;
    for (p, dirs) in combed.dir.iter().enumerate() {
        for (i, &d) in dirs.iter().enumerate() {
            let f = cut.tri_face[p][i];
            let v = mesh.faces()[f as usize].verts();
            let n = super::unit(super::cross(
                super::sub(pos[v[1] as usize], pos[v[0] as usize]),
                super::sub(pos[v[2] as usize], pos[v[0] as usize]),
            ))
            .expect("a face tem area");
            assert!(
                (super::dot(d, d) - 1.0).abs() < 1.0e-3,
                "patch {p} triangulo {i}: a direcao penteada nao e' unitaria"
            );
            assert!(
                super::dot(d, n).abs() < 1.0e-3,
                "patch {p} triangulo {i}: a direcao penteada saiu do plano da face"
            );
            // ⭐ Um dos quatro braços: o ângulo com a cruz original é múltiplo de 90°.
            let raw = super::tangent(layout.face_dir[f as usize], n).expect("a cruz e' tangente");
            let c = super::dot(d, raw).abs().min(1.0);
            let s = super::dot(super::cross(n, d), raw).abs().min(1.0);
            assert!(
                c < 1.0e-2 || s < 1.0e-2,
                "patch {p} triangulo {i}: a direcao penteada nao e' um braco da cruz \
                 (cos {c:.4}, sen {s:.4})"
            );
            checked += 1;
        }
    }
    assert!(checked > 1000, "o gate mediu so' {checked} triangulos");
}

/// ⭐⭐⭐ **CONTROLO POSITIVO: rodar a moldura de um patch move TODOS os saltos dele.**
///
/// ⛔⛔ **Sem isto, `inconsistent == 0` e `jumps == seams` passam com um salto sempre
/// `0`** — que é exactamente o que um bug de sinal ou um transporte esquecido dariam.
/// *Um número que nunca muda não é um número medido.*
///
/// Rodar artificialmente a direcção penteada de um patch por um quarto de volta tem de
/// mudar o salto de **todas** as costuras que lhe tocam, e de mais nenhuma.
#[test]
fn turning_one_patch_moves_exactly_its_own_seams() {
    let (mesh, layout) = chain(shapes::uv_sphere(24, 36, 1.0));
    let (cut, _) = crate::cut::cut_along_patches(&mesh, &layout);
    let (before, _) = super::comb_patches(&mesh, &layout, &cut);

    // Roda a moldura do patch `0` — um quarto de volta em torno da normal da face.
    let victim = 0usize;
    let mut turned = before.clone();
    let pos = mesh.positions();
    for (i, d) in turned.dir[victim].iter_mut().enumerate() {
        let f = cut.tri_face[victim][i];
        let v = mesh.faces()[f as usize].verts();
        let n = super::unit(super::cross(
            super::sub(pos[v[1] as usize], pos[v[0] as usize]),
            super::sub(pos[v[2] as usize], pos[v[0] as usize]),
        ))
        .expect("a face tem area");
        *d = super::cross(n, *d);
    }
    let after = super::jumps_only(&mesh, &cut, &turned);

    let (mut moved, mut still) = (0usize, 0usize);
    for (s, seam) in cut.seams.iter().enumerate() {
        let touches =
            seam.side[0].patch as usize == victim || seam.side[1].patch as usize == victim;
        let (Some(a), Some(b)) = (before.jump[s], after[s]) else {
            continue;
        };
        if touches {
            assert_ne!(
                a, b,
                "a costura {s} toca o patch rodado e o salto nao mexeu ({a}) -- \
                 o salto nao esta' a ler a moldura"
            );
            moved += 1;
        } else {
            assert_eq!(
                a, b,
                "a costura {s} NAO toca o patch rodado e o salto mudou ({a} -> {b})"
            );
            still += 1;
        }
    }
    assert!(moved > 0, "o patch rodado nao toca costura nenhuma");
    assert!(
        still > 0,
        "todas as costuras tocam o patch rodado -- o controlo nao separa nada"
    );
    eprintln!("  {moved} costuras mexeram, {still} ficaram");
}

/// ⭐⭐⭐ **SONDA — quais costuras discordam, e o que têm em comum?**
///
/// ```text
/// cd .../Worktrees/line-sculpt3d && cargo test -p ph2d-gridmap --release \
///     which_seams_disagree -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda -- diagnostico das costuras inconsistentes"]
fn which_seams_disagree() {
    for (name, mesh) in [
        ("esfera lisa", shapes::uv_sphere(24, 36, 1.0)),
        ("toro", shapes::torus(32, 16, 1.0, 0.35)),
    ] {
        let (mesh, layout) = chain(mesh);
        let (cut, cr) = crate::cut::cut_along_patches(&mesh, &layout);
        let (combed, rep) = super::comb_patches(&mesh, &layout, &cut);
        eprintln!(
            "── {name}: {} patches ({} sujos), {} costuras, {} inconsistentes, {} abertos",
            rep.patches, rep.dirty, rep.seams, rep.inconsistent, cr.opened
        );
        for (s, seam) in cut.seams.iter().enumerate() {
            let (pa, pb) = (seam.side[0].patch, seam.side[1].patch);
            let da = combed.holonomy[pa as usize].map_or(9, |h| h.defects);
            let db = combed.holonomy[pb as usize].map_or(9, |h| h.defects);
            // Re-lê os votos desta costura, um por aresta.
            let mut per_edge: Vec<i32> = Vec::new();
            for one in 0..seam.chain.len().saturating_sub(1) {
                let mut sub = seam.clone();
                sub.chain = seam.chain[one..=one + 1].to_vec();
                sub.side[0].local = seam.side[0].local[one..=one + 1].to_vec();
                sub.side[1].local = seam.side[1].local[one..=one + 1].to_vec();
                let mut lone = cut.clone();
                lone.seams = vec![sub];
                if let Some(k) = super::jumps_only(&mesh, &lone, &combed)[0] {
                    per_edge.push(k);
                }
            }
            let mut uniq: Vec<i32> = per_edge.clone();
            uniq.sort_unstable();
            uniq.dedup();
            if uniq.len() > 1 {
                eprintln!(
                    "  ⛔ costura {s} (arco {:?}, patches {pa}/{pb}, defeitos {da}/{db}, \
                     {} arestas): votos {uniq:?}",
                    seam.arc,
                    seam.chain.len() - 1,
                );
            }
        }
    }
}
