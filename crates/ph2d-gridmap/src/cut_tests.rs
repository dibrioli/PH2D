//! ⭐⭐⭐ **OS GATES DO CORTE** — e o que cada um apanha que os outros não apanham.
//!
//! | gate | o erro que ele apanha |
//! |---|---|
//! | `χ = 1` por patch | sub-cortar (anel) **e** sobre-cortar (partido) |
//! | faces preservadas | um patch a menos, sem erro nenhum a acusar |
//! | cortar duplica | um corte que não corta |
//! | ⭐ os dois lados são cópias **distintas** | a construção `global → local`, que funde |
//! | sem órfãos na costura | um vértice de arco que nenhum lado alcança |
//!
//! ⚠️ **O quarto é o que vale a pena ler.** Os três primeiros passam com a construção
//! ingénua num patch normal; só a ponte de um anel os separa, e é por isso que o **toro**
//! está aqui.

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

/// ⭐⭐⭐ **CADA PATCH CORTADO É UM DISCO**, e a régua apanha os dois erros opostos.
///
/// ⛔ `χ ≤ 0` = sub-cortado (ainda é anel) · `χ ≥ 2` = sobre-cortado (partiu-se). *Uma
/// contagem de vértices não distingue nenhum dos dois de «está bem».*
#[test]
fn every_cut_patch_is_a_disc() {
    for (name, mesh) in [
        ("esfera lisa", shapes::uv_sphere(24, 36, 1.0)),
        // ⚠️ **O TORO está aqui de propósito:** é o género `1`, e é onde o traçado
        // precisa da ponte que abre um patch-anel. Sem ele, este ficheiro mediria só o
        // caso fácil.
        ("toro", shapes::torus(32, 16, 1.0, 0.35)),
    ] {
        let (mesh, layout) = chain(mesh);
        let (_, rep) = super::cut_along_patches(&mesh, &layout);
        assert_eq!(
            rep.discs,
            rep.patches,
            "{name}: {} de {} patches nao sao discos (aneis {}, partidos {}) -- \
             a fase seguinte resolveria um dominio que nao fecha",
            rep.patches - rep.discs,
            rep.patches,
            rep.not_discs[0],
            rep.not_discs[1],
        );
        assert!(rep.patches > 0, "{name}: o corte nao viu patch nenhum");
    }
}

/// ⭐ **NENHUMA FACE SE PERDE, e nenhuma cai fora por não ser triângulo.**
///
/// ⚠️ *Uma face perdida é um patch mais pequeno do que a peça, e mais pequeno lê-se
/// como geometria.*
#[test]
fn the_cut_keeps_every_face() {
    let (mesh, layout) = chain(shapes::uv_sphere(24, 36, 1.0));
    let (cut, rep) = super::cut_along_patches(&mesh, &layout);
    assert_eq!(rep.non_tris, 0, "a malha chegou com faces nao-triangulares");
    assert_eq!(
        rep.faces,
        mesh.faces().len(),
        "o corte perdeu faces: {} de {}",
        rep.faces,
        mesh.faces().len()
    );
    let total: usize = cut.tris.iter().map(Vec::len).sum();
    assert_eq!(
        total, rep.faces,
        "a contagem do relatorio nao bate com a malha cortada"
    );
}

/// ⭐ **CORTAR DUPLICA, NUNCA FUNDE.**
///
/// ⚠️ E o `>` é estrito de propósito: uma esfera decomposta tem costuras, logo tem de
/// haver vértices com mais de uma cópia. *Um corte que devolvesse exactamente `V` não
/// cortou nada.*
#[test]
fn cutting_only_ever_adds_vertices() {
    let (mesh, layout) = chain(shapes::uv_sphere(24, 36, 1.0));
    let (_, rep) = super::cut_along_patches(&mesh, &layout);
    assert!(
        rep.verts > mesh.vert_count(),
        "a malha cortada tem {} vertices e a original {} -- cortar tem de DUPLICAR",
        rep.verts,
        mesh.vert_count()
    );
}

/// ⭐⭐⭐ **OS DOIS LADOS DE UMA COSTURA SÃO CÓPIAS DISTINTAS** — a definição de corte.
///
/// ⛔⛔ **É este gate que separa a construção certa da ingénua.** Um `BTreeMap` de
/// vértice global para índice local dá **o mesmo** índice aos dois lados quando o
/// patch é o mesmo — que é exactamente a ponte que abre um anel — e o anel sairia
/// anel, sem nada a acusar.
///
/// ⚠️ Para lados em patches diferentes o par `(patch, local)` já difere pelo patch; o
/// caso que mede alguma coisa é `patch[0] == patch[1]`, e a asserção separa-o.
#[test]
fn the_two_sides_of_a_seam_are_distinct_copies() {
    for (name, mesh) in [
        ("esfera lisa", shapes::uv_sphere(24, 36, 1.0)),
        ("toro", shapes::torus(32, 16, 1.0, 0.35)),
    ] {
        let (mesh, layout) = chain(mesh);
        let (cut, rep) = super::cut_along_patches(&mesh, &layout);
        assert!(!cut.seams.is_empty(), "{name}: nenhuma costura");
        assert_eq!(
            rep.orphan_seam_vertices, 0,
            "{name}: {} posicoes de cadeia sem copia de um dos lados",
            rep.orphan_seam_vertices
        );
        let mut bridges = 0usize;
        for s in &cut.seams {
            let (a, b) = (&s.side[0], &s.side[1]);
            if a.patch != b.patch {
                continue;
            }
            bridges += 1;
            for (i, (x, y)) in a.local.iter().zip(&b.local).enumerate() {
                if let (Some(x), Some(y)) = (x, y) {
                    assert_ne!(
                        x, y,
                        "{name}: a costura {:?} e' percorrida pelo MESMO patch dos dois lados e \
                         a posicao {i} recebeu a MESMA copia -- o corte fundiu o que devia \
                         separar, e o patch continua um anel",
                        s.arc
                    );
                }
            }
        }
        eprintln!(
            "  {name}: {} costuras, {bridges} delas pontes",
            cut.seams.len()
        );
    }
}

/// ⭐ **A ORIGEM DE CADA CÓPIA É UM VÉRTICE REAL**, e os triângulos indexam dentro do
/// seu próprio patch.
///
/// ⚠️ *Um índice local fora de alcance só se manifestaria no solver da fase seguinte,
/// como um número errado em vez de um erro.*
#[test]
fn every_local_index_is_in_range_and_maps_to_a_real_vertex() {
    let (mesh, layout) = chain(shapes::uv_sphere(24, 36, 1.0));
    let (cut, _) = super::cut_along_patches(&mesh, &layout);
    for (p, origin) in cut.origin.iter().enumerate() {
        for &g in origin {
            assert!(
                (g as usize) < mesh.vert_count(),
                "patch {p}: uma copia aponta para o vertice {g}, que nao existe"
            );
        }
        for t in &cut.tris[p] {
            for &l in t {
                assert!(
                    (l as usize) < origin.len(),
                    "patch {p}: um triangulo indexa a copia {l} de {}",
                    origin.len()
                );
            }
        }
        assert_eq!(
            cut.tris[p].len(),
            cut.tri_face[p].len(),
            "patch {p}: a face de origem nao acompanha todos os triangulos"
        );
    }
}

/// ⭐⭐⭐ **SONDA — de quem é a dívida do patch que fica anel?**
///
/// ```text
/// cd .../Worktrees/line-sculpt3d && cargo test -p ph2d-gridmap --release \
///     who_owns_the_ring -- --ignored --nocapture
/// ```
///
/// ⚠️ **Um `χ ≠ 1` pode ser meu (a união-busca não cortou) ou do F3 (nunca construiu a
/// ponte).** As duas leem igual no gate. O que as separa é perguntar se **algum arco é
/// percorrido DUAS vezes** pelo patch em causa: se sim, a ponte existe e quem falhou
/// fui eu; se não, não há por onde cortar e a dívida é do traçado.
#[test]
#[ignore = "sonda -- diagnostico do patch-anel do toro"]
fn who_owns_the_ring() {
    let (mesh, layout) = chain(shapes::torus(32, 16, 1.0, 0.35));
    let (cut, rep) = super::cut_along_patches(&mesh, &layout);
    eprintln!(
        "  {} patches, {} discos, aneis {}, partidos {}",
        rep.patches, rep.discs, rep.not_discs[0], rep.not_discs[1]
    );
    for (p, tris) in cut.tris.iter().enumerate() {
        let mut edges = std::collections::BTreeSet::new();
        for t in tris {
            for k in 0..3 {
                edges.insert(super::key(t[k], t[(k + 1) % 3]));
            }
        }
        let chi = cut.origin[p].len() as i64 - edges.len() as i64 + tris.len() as i64;
        if chi == 1 {
            continue;
        }
        // Quantas vezes cada arco aparece nos lados deste patch.
        let mut seen: std::collections::BTreeMap<u32, usize> = std::collections::BTreeMap::new();
        for side in &layout.side_arcs[p] {
            for &(a, _) in side {
                *seen.entry(a).or_default() += 1;
            }
        }
        let twice: Vec<u32> = seen
            .iter()
            .filter(|&(_, &n)| n >= 2)
            .map(|(&a, _)| a)
            .collect();
        eprintln!(
            "  ⛔ patch {p}: chi={chi} · {} faces · {} lados · {} arcos · arcos repetidos {:?}",
            tris.len(),
            layout.side_arcs[p].len(),
            seen.len(),
            twice
        );
        eprintln!(
            "     => {}",
            if twice.is_empty() {
                "NENHUM arco repetido: o F3 nunca construiu a ponte -- divida do TRACADO"
            } else {
                "ha' arco repetido: a ponte existe e o corte falhou -- divida MINHA"
            }
        );
    }
}

/// ⭐⭐⭐ **O ABRIDOR DE ANÉIS DISPARA, e conta-se.**
///
/// ⛔⛔ **Sem este gate, `18 discos de 18` no toro não distingue duas coisas muito
/// diferentes:** *o traçado entregou tudo em discos* e *entregou um anel e esta fase
/// abriu-o*. São o mesmo número, e só a segunda é verdade.
///
/// ⚠️ Medido 2026-08-23: o toro `32×16` entrega **um** patch com `χ = 0` — 666 faces,
/// 16 arcos, **nenhum arco repetido**, ou seja o F3 nunca lhe construiu ponte. *A
/// dívida é do traçado; esta fase abre-o porque o contrato dela é entregar discos.*
///
/// ⭐ **Provado por mutação:** com `OPEN_TRIES = 0` o toro volta a `1 de 18` anéis e a
/// esfera fica verde — o gate mede o abridor e mais nada.
#[test]
fn the_ring_opener_fires_on_the_torus_and_never_on_the_sphere() {
    let (mesh, layout) = chain(shapes::uv_sphere(24, 36, 1.0));
    let (_, rep) = super::cut_along_patches(&mesh, &layout);
    assert_eq!(
        rep.opened, 0,
        "a esfera nao tem anel nenhum e mesmo assim {} patches foram abertos -- \
         o abridor esta' a cortar por engano",
        rep.opened
    );

    let (mesh, layout) = chain(shapes::torus(32, 16, 1.0, 0.35));
    let (cut, rep) = super::cut_along_patches(&mesh, &layout);
    assert_eq!(
        rep.opened, 1,
        "o toro entrega UM patch-anel e o abridor devia ter aberto exactamente 1, abriu {}",
        rep.opened
    );
    assert_eq!(rep.unopened, 0, "sobrou patch por abrir");
    // ⭐ E o corte tem de aparecer na tabela de costuras: um corte que a fase seguinte
    // não veja deixa o patch RASGADO, que é pior do que anel.
    let internal = cut.seams.iter().filter(|s| s.arc.is_none()).count();
    assert_eq!(
        internal, 1,
        "o corte interno nao chegou a' tabela de costuras ({internal} entradas sem arco) -- \
         a fase seguinte deixaria o patch rasgado ao longo dele"
    );
}
