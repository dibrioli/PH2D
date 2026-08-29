//! ⭐⭐⭐ **OS GATES DO G3** — e o controlo onde a resposta exacta se sabe de antemão.
//!
//! # Por que a fixtura é montada À MÃO
//!
//! ⚠️ Correr a cadeia inteira até aqui mede **tudo ao mesmo tempo**: o remalhado, o
//! campo, o traçado, o corte, o pentear e o solver. Se o número sair mau, não há como
//! saber de quem é. ⭐ **A fixtura desta folha é uma tira PLANA de duas metades**, com o
//! campo escrito à mão — e sobre ela a resposta certa é conhecida ao ponto flutuante:
//! `u = x/h`, `v = y/h`.
//!
//! ⛔ *Um solver que só se mede na esfera é um solver cujo erro nunca se separa do erro
//! das cinco fases antes dele.*

use ph2d_mesh::{Face, Mesh};

use crate::comb::Combed;
use crate::cut::{CutMesh, Seam, SeamSide};

/// Colunas e linhas de células da tira.
const NX: usize = 6;
const NY: usize = 4;
/// O passo da grade, na unidade da peça.
const H: f32 = 0.1;
/// Onde a tira se parte em dois patches.
const SPLIT: usize = 3;

/// A tira plana, e a decomposição à mão em duas metades.
///
/// `turned` roda o campo da metade da direita um quarto de volta — é o caso que **prova
/// o sinal do salto**, e com ele o `k` correcto é `3` (ver o doc de [`crate::solve`]).
fn strip(turned: bool) -> (Mesh, CutMesh, Combed) {
    let at = |i: usize, j: usize| (j * (NX + 1) + i) as u32;
    let mut pos = Vec::with_capacity((NX + 1) * (NY + 1));
    for j in 0..=NY {
        for i in 0..=NX {
            #[allow(clippy::cast_precision_loss)]
            pos.push([i as f32 * H, j as f32 * H, 0.0]);
        }
    }
    let mut faces = Vec::with_capacity(NX * NY * 2);
    // `cell[i][j]` gera duas faces consecutivas — a ordem é o que liga face a patch.
    let mut owner: Vec<usize> = Vec::with_capacity(NX * NY * 2);
    for j in 0..NY {
        for i in 0..NX {
            faces.push(Face::tri(at(i, j), at(i + 1, j), at(i + 1, j + 1)));
            faces.push(Face::tri(at(i, j), at(i + 1, j + 1), at(i, j + 1)));
            let p = usize::from(i >= SPLIT);
            owner.push(p);
            owner.push(p);
        }
    }
    let mesh = Mesh::from_parts(pos, faces).expect("a tira e' bem formada");

    // ── O corte à mão: cada metade com os seus próprios índices locais.
    let mut cut = CutMesh {
        origin: vec![Vec::new(), Vec::new()],
        tris: vec![Vec::new(), Vec::new()],
        tri_face: vec![Vec::new(), Vec::new()],
        seams: Vec::new(),
    };
    let mut slot: [std::collections::BTreeMap<u32, u32>; 2] = Default::default();
    for (f, &p) in owner.iter().enumerate() {
        let v = mesh.faces()[f].verts();
        let mut t = [0u32; 3];
        for (k, &g) in v.iter().enumerate() {
            let next = u32::try_from(cut.origin[p].len()).expect("cabe");
            let l = *slot[p].entry(g).or_insert_with(|| {
                cut.origin[p].push(g);
                next
            });
            t[k] = l;
        }
        cut.tris[p].push(t);
        cut.tri_face[p].push(u32::try_from(f).expect("cabe"));
    }

    // ── A costura: a coluna do meio, vista dos dois lados.
    let chain: Vec<u32> = (0..=NY).map(|j| at(SPLIT, j)).collect();
    let side = |p: usize| SeamSide {
        patch: u32::try_from(p).expect("cabe"),
        local: chain.iter().map(|g| slot[p].get(g).copied()).collect(),
    };
    let sides = [side(0), side(1)];
    cut.seams.push(Seam {
        arc: Some(0),
        chain,
        side: sides,
    });

    // ── O campo, escrito à mão.
    let dir_a = [1.0f32, 0.0, 0.0];
    let dir_b = if turned {
        [0.0f32, 1.0, 0.0]
    } else {
        [1.0f32, 0.0, 0.0]
    };
    // ⭐ `R^k X_b = X_a`. Com `X_a = (1,0)` e `X_b = (0,1)`, `R^3 (0,1) = (1,0)` ⇒ `k = 3`.
    let jump = if turned { 3 } else { 0 };
    let combed = Combed {
        dir: vec![
            vec![dir_a; cut.tris[0].len()],
            vec![dir_b; cut.tris[1].len()],
        ],
        holonomy: vec![Some(ph2d_crossfield::Holonomy::default()); 2],
        jump: vec![Some(jump)],
    };
    (mesh, cut, combed)
}

/// ⭐⭐⭐ **O CONTROLO EXACTO: uma tira plana com campo constante.**
///
/// A resposta certa é `u = x/h`, `v = y/h` — e o solver não tem condição de fronteira
/// nenhuma para lá chegar por acidente. ⇒ **os dois resíduos têm de ser ~`0`**.
///
/// ⚠️ *Sem isto, um erro no gradiente de base, na área, ou no sinal de `Y = n × X`
/// passaria despercebido e reapareceria como «a esfera está pouco pior».*
#[test]
fn a_flat_strip_with_a_constant_field_is_solved_exactly() {
    let (mesh, cut, combed) = strip(false);
    let (map, rep) = super::solve(&mesh, &cut, &combed, H);
    assert_eq!(rep.skipped, 0, "triangulos de fora: {rep:?}");
    assert!(rep.pairs > 0, "a costura nao acoplou par nenhum: {rep:?}");
    assert!(
        rep.align_p95 < 1.0e-3,
        "o gradiente nao segue o campo numa tira PLANA com campo CONSTANTE \
         (p50 {:.5}, p95 {:.5}) -- o erro e' do solver, nao da peca",
        rep.align_p50,
        rep.align_p95
    );
    assert!(
        rep.seam_max < 1.0e-2,
        "a costura nao fecha numa tira plana (p50 {:.5}, max {:.5})",
        rep.seam_p50,
        rep.seam_max
    );
    // ⭐ E o mapa **é** a coordenada, a menos de uma translação global (a energia só
    // conhece gradientes). Mede-se a DIFERENÇA entre dois vértices, que não tem gauge.
    let pos = mesh.positions();
    let (mut worst, mut n) = (0.0f32, 0usize);
    let base_g = cut.origin[0][0];
    let base_z = map.uv[0][0];
    for (l, &g) in cut.origin[0].iter().enumerate() {
        let want = [
            (pos[g as usize][0] - pos[base_g as usize][0]) / H,
            (pos[g as usize][1] - pos[base_g as usize][1]) / H,
        ];
        let got = [map.uv[0][l][0] - base_z[0], map.uv[0][l][1] - base_z[1]];
        worst = worst
            .max((want[0] - got[0]).abs())
            .max((want[1] - got[1]).abs());
        n += 1;
    }
    assert!(n > 10, "o gate mediu so' {n} vertices");
    assert!(
        worst < 1.0e-2,
        "o mapa nao e' a coordenada dividida pelo passo -- pior desvio {worst:.5} celulas"
    );
}

/// ⭐⭐⭐ **O CONTROLO DO SINAL: a metade da direita com o campo RODADO.**
///
/// ⛔⛔ **Este é o gate que o sinal do salto ou passa ou reprova.** Com o campo da
/// direita a um quarto de volta do da esquerda, `z_b = R^k z_a + t` só fecha com o `k`
/// certo; com o sinal trocado (`R^{−k}`) o resíduo da costura salta para a ordem de uma
/// célula inteira. *A dedução está no doc do módulo; isto é a medição dela.*
#[test]
fn the_seam_closes_when_the_two_sides_disagree_by_a_quarter_turn() {
    let (mesh, cut, combed) = strip(true);
    let (_, rep) = super::solve(&mesh, &cut, &combed, H);
    assert!(
        rep.align_p95 < 1.0e-3,
        "o alinhamento partiu-se com o campo rodado (p95 {:.5})",
        rep.align_p95
    );
    assert!(
        rep.seam_max < 1.0e-2,
        "a costura NAO fecha com um quarto de volta entre os lados \
         (p50 {:.5}, max {:.5}) -- o sinal de `z_b = R^k z_a + t` esta' trocado",
        rep.seam_p50,
        rep.seam_max
    );
}

/// ⭐⭐⭐ **CONTROLO NEGATIVO: um salto errado custa o ALINHAMENTO, não a costura.**
///
/// ⛔⛔ **A primeira versão deste gate exigia que a costura ABRISSE, e a medição
/// desmentiu-a:** com o salto errado o resíduo da costura fica em fracções de célula,
/// enquanto o alinhamento salta de `0,00000` para `0,3`. *A penalização é forte, então o
/// solver paga o erro onde ela não o impede — no gradiente.*
///
/// Com o peso que shipa (`8`):
///
/// | salto | alinhamento p50 | p95 | costura max |
/// |---|---|---|---|
/// | ⭐ **`3` (o CERTO)** | **`0,00000`** | **`0,00000`** | **`0,00000`** |
/// | `0` | `0,21769` | `0,55782` | `0,08916` |
/// | `1` | `0,30785` | `0,78887` | `0,12609` |
/// | `2` | `0,21769` | `0,55782` | `0,08916` |
///
/// ⚠️ *A costura abre um pouco mais do que abria a `512` (`0,016`), como tem de ser —
/// mas continua uma ordem de grandeza abaixo do sinal do alinhamento.*
///
/// ⭐⭐⭐ **E a lição vale para lá deste gate: «as costuras fecharam» não é sinal de
/// saúde nenhum.** Quem olhar só para a coluna da costura vê `0,016` e conclui que
/// está tudo bem, com o mapa a errar **um terço** do gradiente que promete. *As duas
/// réguas têm de ser lidas juntas, e é por isso que o relatório traz as duas.*
///
/// ⚠️ A barra sai da tabela — `0,2` está a meio caminho entre `0,00000` e o pior
/// `0,22349`, e a separação são **cinco ordens de grandeza**.
#[test]
fn a_wrong_jump_is_paid_in_alignment_not_in_the_seam() {
    for k in [0, 1, 2] {
        let (mesh, cut, mut combed) = strip(true);
        combed.jump[0] = Some(k); // o certo é `3`
        let (_, rep) = super::solve(&mesh, &cut, &combed, H);
        assert!(
            rep.align_p50 > 0.2,
            "com o salto ERRADO ({k}) o alinhamento ficou em {:.5} -- \
             ou o acoplamento nao corre, ou a regua nao mede o que diz",
            rep.align_p50
        );
        assert!(
            rep.seam_max < 0.2,
            "salto {k}: a costura abriu {:.5} -- se ela abrir de vez, a barra deste gate \
             podia ser sobre ela, e a licao da tabela acima muda",
            rep.seam_max
        );
    }
}

/// ⭐ **SONDA — o que um salto errado de facto estraga.**
///
/// ```text
/// cd .../Worktrees/line-sculpt3d && cargo test -p ph2d-gridmap --release \
///     what_does_a_wrong_jump_cost -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda -- calibra a barra do controlo negativo"]
fn what_does_a_wrong_jump_cost() {
    for k in [3, 0, 1, 2] {
        let (mesh, cut, mut combed) = strip(true);
        combed.jump[0] = Some(k);
        let (_, rep) = super::solve(&mesh, &cut, &combed, H);
        eprintln!(
            "  salto {k}{}: alinhamento p50 {:.5} p95 {:.5} | costura p50 {:.5} max {:.5}",
            if k == 3 { " (o CERTO)" } else { "         " },
            rep.align_p50,
            rep.align_p95,
            rep.seam_p50,
            rep.seam_max
        );
    }
}

/// ⭐⭐⭐ **SONDA — o mapa global numa ESFERA a sério, e a varredura dos dois números.**
///
/// ```text
/// cd .../Worktrees/line-sculpt3d && cargo test -p ph2d-gridmap --release \
///     the_global_map_on_a_real_sphere -- --ignored --nocapture
/// ```
///
/// ⚠️ **É aqui que as duas constantes se justificam** (`CLAUDE.md` §0.0): o peso da
/// costura e o número de rondas não podem ser o palpite de quem os escreveu.
#[test]
#[ignore = "sonda -- o mapa global na cadeia inteira"]
fn the_global_map_on_a_real_sphere() {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(24, 36, 1.0);
    mesh.triangulate();
    ph2d_remesh_iso::remesh_isotropic(&mut mesh, ph2d_remesh_iso::ALPHA);
    mesh.triangulate();
    let dual = ph2d_crossfield::Dual::build(&mesh);
    let (field, _) = ph2d_crossfield::solve_miq(&dual);
    let layout = ph2d_trace::trace_patches(&mesh, &dual, &field);
    let (cut, cr) = crate::cut::cut_along_patches(&mesh, &layout);
    let (combed, comb_rep) = crate::comb::comb_patches(&mesh, &layout, &cut);
    eprintln!(
        "  {} patches ({} discos, {} sujos) · {} costuras ({} com salto)",
        cr.patches, cr.discs, comb_rep.dirty, comb_rep.seams, comb_rep.jumps
    );
    // ⭐ O passo alvo: a aresta mediana da malha remalhada, que é o que o F1 entregou.
    let pos = mesh.positions();
    let mut edges: Vec<f32> = Vec::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (pos[v[k] as usize], pos[v[(k + 1) % v.len()] as usize]);
            let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
            edges.push(d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt());
        }
    }
    edges.sort_by(f32::total_cmp);
    let h = edges[edges.len() / 2];
    eprintln!("  passo alvo h = {h:.5} (aresta mediana da malha)");

    for weight in [1.0f32, 8.0, 64.0, 512.0] {
        for rounds in [2_000usize, 10_000, 40_000] {
            let (_, rep) = super::solve_with(
                &mesh,
                &cut,
                &combed,
                crate::solve::Step::uniform(h),
                weight,
                rounds,
            );
            eprintln!(
                "  peso {weight:>6.0} rondas {rounds:>6}: ⭐angulo p50 {:>5.1}° p95 {:>5.1}° \
                 | escala p50 {:.3} p95 {:.3} | costura p50 {:.4} max {:.4} | (misto {:.3})",
                rep.angle_p50,
                rep.angle_p95,
                rep.scale_p50,
                rep.scale_p95,
                rep.seam_p50,
                rep.seam_max,
                rep.align_p50
            );
        }
    }
}

/// ⭐⭐⭐ **SONDA — o sistema rígido é um COMPROMISSO ou só um solver lento?**
///
/// ```text
/// cd .../Worktrees/line-sculpt3d && cargo test -p ph2d-gridmap --release \
///     does_the_stiff_system_just_need_more_rounds -- --ignored --nocapture
/// ```
///
/// ⚠️ A varredura anterior mostrou que subir o peso fecha as costuras e estraga o
/// ângulo — **mas com a assinatura de não-convergência**: a `w = 64` o ângulo cai de
/// `38°` para `11,9°` entre `2 000` e `40 000` rondas, ainda depressa. *Um compromisso
/// verdadeiro não melhora com rondas; um solver lento melhora.* Esta sonda separa os
/// dois, que é a mesma lei do ponto fixo do `regraduate`: **uma afirmação sobre
/// convergência precisa de mais de um termo.*
#[test]
#[ignore = "sonda LENTA -- centenas de milhares de rondas"]
fn does_the_stiff_system_just_need_more_rounds() {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(24, 36, 1.0);
    mesh.triangulate();
    ph2d_remesh_iso::remesh_isotropic(&mut mesh, ph2d_remesh_iso::ALPHA);
    mesh.triangulate();
    let dual = ph2d_crossfield::Dual::build(&mesh);
    let (field, _) = ph2d_crossfield::solve_miq(&dual);
    let layout = ph2d_trace::trace_patches(&mesh, &dual, &field);
    let (cut, _) = crate::cut::cut_along_patches(&mesh, &layout);
    let (combed, _) = crate::comb::comb_patches(&mesh, &layout, &cut);
    let pos = mesh.positions();
    let mut edges: Vec<f32> = Vec::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (pos[v[k] as usize], pos[v[(k + 1) % v.len()] as usize]);
            let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
            edges.push(d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt());
        }
    }
    edges.sort_by(f32::total_cmp);
    let h = edges[edges.len() / 2];
    for weight in [64.0f32, 512.0] {
        for rounds in [40_000usize, 160_000, 640_000] {
            let (_, rep) = super::solve_with(
                &mesh,
                &cut,
                &combed,
                crate::solve::Step::uniform(h),
                weight,
                rounds,
            );
            eprintln!(
                "  peso {weight:>5.0} rondas {rounds:>7}: ⭐angulo p50 {:>5.1}° p95 {:>5.1}° \
                 | escala p50 {:.3} | costura p50 {:.4} max {:.4}",
                rep.angle_p50, rep.angle_p95, rep.scale_p50, rep.seam_p50, rep.seam_max
            );
        }
    }
}

/// ⭐⭐⭐ **SONDA — as translações das costuras dão para ARREDONDAR?**
///
/// ```text
/// cd .../Worktrees/line-sculpt3d && cargo test -p ph2d-gridmap --release \
///     can_the_seam_shifts_be_rounded -- --ignored --nocapture
/// ```
///
/// ⛔ **A extracção precisa delas INTEIRAS** — só assim as isolinhas de `(u, v)` casam
/// dos dois lados. ⚠️ *Medir o preço antes de o pagar:* se as translações já caírem
/// perto de inteiros, arredondar é de graça; se caírem a meio caminho (`0,5`), o
/// arredondamento é uma mudança grande e o resíduo da costura vai subir.
#[test]
#[ignore = "sonda -- o preco de arredondar as costuras"]
fn can_the_seam_shifts_be_rounded() {
    for (name, mesh) in [
        ("ESFERA FINA", ph2d_mesh::shapes::uv_sphere(96, 144, 1.0)),
        ("ESFERA LISA", ph2d_mesh::shapes::uv_sphere(24, 36, 1.0)),
    ] {
        let mut mesh = mesh;
        mesh.triangulate();
        ph2d_remesh_iso::remesh_isotropic(&mut mesh, ph2d_remesh_iso::ALPHA);
        mesh.triangulate();
        let dual = ph2d_crossfield::Dual::build(&mesh);
        let (field, _) = ph2d_crossfield::solve_miq(&dual);
        let layout = ph2d_trace::trace_patches(&mesh, &dual, &field);
        let (cut, _) = crate::cut::cut_along_patches(&mesh, &layout);
        let (combed, _) = crate::comb::comb_patches(&mesh, &layout, &cut);
        let pos = mesh.positions();
        let mut edges: Vec<f32> = Vec::new();
        for f in mesh.faces() {
            let v = f.verts();
            for k in 0..v.len() {
                let (a, b) = (pos[v[k] as usize], pos[v[(k + 1) % v.len()] as usize]);
                let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
                edges.push(d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt());
            }
        }
        edges.sort_by(f32::total_cmp);
        let h = edges[edges.len() / 2];

        let (map, free) = super::solve(&mesh, &cut, &combed, h);
        eprintln!(
            "── {name}: LIVRE angulo p50 {:>4.1}° | costura p50 {:.4} max {:.4} \
             | ⭐distancia a inteiro p50 {:.3} max {:.3}",
            free.angle_p50, free.seam_p50, free.seam_max, free.shift_frac_p50, free.shift_frac_max
        );
        let shift = super::rounded_shifts(&map);
        let (_, pin) = super::solve_pinned(
            &mesh,
            &cut,
            &combed,
            crate::solve::Step::uniform(h),
            super::SEAM_WEIGHT,
            super::ROUNDS,
            &shift,
        );
        eprintln!(
            "   {name}: ⭐INTEIRA angulo p50 {:>4.1}° | costura p50 {:.4} max {:.4}",
            pin.angle_p50, pin.seam_p50, pin.seam_max
        );
    }
}
