//! Os gates do arredondamento inteiro, mais a sonda que justifica as constantes.

use super::{Relaxer, RoundOptions, round_to_integers};
use crate::solve::{SolveReport, assemble, solve_with};

/// A cadeia até ao corte e ao pente, sobre uma peça de verdade.
///
/// ⚠️ **`pub(crate)` de propósito:** a sonda da soldadura precisa da MESMA cadeia, e
/// uma segunda cópia dela divergiria desta sem ninguém dar por isso.
pub(crate) fn chain(
    mesh: &mut ph2d_mesh::Mesh,
) -> (crate::cut::CutMesh, crate::comb::Combed, f32, Vec<u32>) {
    mesh.triangulate();
    ph2d_remesh_iso::remesh_isotropic(mesh, ph2d_remesh_iso::ALPHA);
    mesh.triangulate();
    let dual = ph2d_crossfield::Dual::build(mesh);
    let (field, _) = ph2d_crossfield::solve_miq(&dual);
    let layout = ph2d_trace::trace_patches(mesh, &dual, &field);
    let (cut, _) = crate::cut::cut_along_patches(mesh, &layout);
    let (combed, _) = crate::comb::comb_patches(mesh, &layout, &cut);
    let pos = mesh.positions();
    let mut e: Vec<f32> = Vec::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (pos[v[k] as usize], pos[v[(k + 1) % v.len()] as usize]);
            let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
            e.push(d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt());
        }
    }
    e.sort_by(f32::total_cmp);
    let h = e[e.len() / 2];
    // ⭐ As singularidades saem do CAMPO — o índice por-vértice é um facto dele, e
    // pedir à `ph2d-gridmap` que o re-derive seria reconstruir o que já existe.
    let singular: Vec<u32> = ph2d_crossfield::vertex_index(mesh, &dual, &field)
        .into_iter()
        .enumerate()
        .filter(|(_, k)| *k != 0)
        .filter_map(|(v, _)| u32::try_from(v).ok())
        .collect();
    (cut, combed, h, singular)
}

/// ⭐⭐⭐ **A SONDA QUE JUSTIFICA AS CONSTANTES** (`CLAUDE.md` §0.0).
///
/// ```text
/// cargo test -p ph2d-gridmap --release -- --ignored the_rounding_ladder --nocapture
/// ```
///
/// ⛔ **O que ela mede é a FRACÇÃO que fica no degrau 1.** Se ela for baixa, o tecto
/// ou a tolerância estão mal escolhidos e o custo vai para o degrau caro — que é
/// exactamente o que a escada adaptativa existe para evitar.
#[test]
#[ignore = "sonda -- a escada do arredondamento inteiro"]
fn the_rounding_ladder_sweeps_its_two_constants() {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(24, 36, 1.0);
    let (cut, combed, h, singular) = chain(&mut mesh);
    eprintln!("esfera remalhada: {} faces, h = {h:.5}", mesh.face_count());
    for tol in [1.0e-2f32, 1.0e-3, 1.0e-4] {
        for cap in [2_000usize, 20_000, 200_000] {
            let t = std::time::Instant::now();
            let (_, r) = round_to_integers(
                &mesh,
                &cut,
                &combed,
                crate::solve::Step::uniform(h),
                RoundOptions {
                    local_tol: tol,
                    local_cap: cap,
                    ..RoundOptions::default()
                },
                &singular,
            );
            #[allow(clippy::cast_precision_loss)]
            let frac = 100.0 * r.level1 as f64 / r.pinned.max(1) as f64;
            eprintln!(
                "  tol {tol:>8.0e} tecto {cap:>7}: ⭐degrau1 {frac:>5.1}% ({}/{}) | {:>9} visitas \
                 | passo pior {:.4} soma {:>6.2} | costura p50 {:.4} max {:.4} | angulo {:.1}° | {:.1}s",
                r.level1,
                r.pinned,
                r.visits,
                r.worst_step,
                r.sum_step,
                r.seam_after.0,
                r.seam_after.1,
                r.solve.angle_p50,
                t.elapsed().as_secs_f64()
            );
        }
    }
}

/// ⭐⭐⭐ **A REGRA DE OURO: uma-a-uma bate o LOTE** — o §10 tornado executável.
///
/// ⛔ `solve::rounded_shifts` arredonda **todas** as translações de uma vez. O erro
/// soma, porque arredondar todas ao mesmo tempo desloca todas as outras ao mesmo
/// tempo; actualizar depois de cada uma deixa o sistema absorver o deslocamento nas
/// que ainda estão livres.
#[test]
#[ignore = "sonda -- uma-a-uma contra o lote"]
fn one_at_a_time_beats_the_batch() {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(24, 36, 1.0);
    let (cut, combed, h, singular) = chain(&mut mesh);
    let opts = RoundOptions::default();

    let (map, batch_rep) = solve_with(
        &mesh,
        &cut,
        &combed,
        crate::solve::Step::uniform(h),
        opts.weight,
        opts.rounds,
    );
    let (g, _) = crate::gauge::fix(&cut, &combed, &map);
    let batch: f32 = g
        .cycle
        .iter()
        .map(|&(_, t)| (t[0] - t[0].round()).abs() + (t[1] - t[1].round()).abs())
        .sum();

    let (_, r) = round_to_integers(
        &mesh,
        &cut,
        &combed,
        crate::solve::Step::uniform(h),
        opts,
        &singular,
    );
    eprintln!(
        "  LOTE: soma dos passos {batch:.3} (costura p50 {:.4} max {:.4})",
        batch_rep.seam_p50, batch_rep.seam_max
    );
    eprintln!(
        "  UMA-A-UMA: soma dos passos {:.3} (costura p50 {:.4} max {:.4}) | degrau1 {}/{}",
        r.sum_step, r.seam_after.0, r.seam_after.1, r.level1, r.pinned
    );
}

#[test]
fn as_translacoes_ficam_todas_inteiras() {
    // ⛔ **É a pre-condicao que a extraccao ASSUME e mede** — sem ela, a grade de uma
    // carta nao casa com a da vizinha e o saneamento apenas arredonda o erro para
    // dentro.
    let mut mesh = ph2d_mesh::shapes::uv_sphere(12, 18, 1.0);
    let (cut, combed, h, singular) = chain(&mut mesh);
    let (map, r) = round_to_integers(
        &mesh,
        &cut,
        &combed,
        crate::solve::Step::uniform(h),
        RoundOptions::default(),
        &singular,
    );
    assert_eq!(
        r.shift_frac_max, 0.0,
        "sobrou uma translacao nao-inteira: {:.3e}",
        r.shift_frac_max
    );
    for t in &map.shift {
        assert_eq!(t[0], t[0].round(), "shift u nao inteiro: {}", t[0]);
        assert_eq!(t[1], t[1].round(), "shift v nao inteiro: {}", t[1]);
    }
    assert!(
        r.pinned > 0,
        "nao havia nada para pregar — a peca nao contem o fenomeno"
    );
    assert!(
        r.singular_pinned > 0,
        "a peca nao tinha singularidade nenhuma, e a modalidade nao foi exercitada"
    );
    assert!(
        r.switched_to_seams,
        "⚠️ o CASO DE CANTO: as singularidades esgotaram-se e sobravam costuras. \
         Terminar ali deixaria o mapa *quase* inteiro, que e' pior que continuo"
    );
    // ⭐ E as imagens dos vertices singulares sao pontos INTEIROS — e' o que faz a
    // grade fechar a' volta deles (o ponto fixo da holonomia passa a ser o proprio
    // vertice, em vez de um meio-inteiro).
    let mut integer_singular = 0usize;
    for (p, uv) in map.uv.iter().enumerate() {
        for (l, z) in uv.iter().enumerate() {
            if singular.contains(&cut.origin[p][l]) && z[0] == z[0].round() && z[1] == z[1].round()
            {
                integer_singular += 1;
            }
        }
    }
    assert!(
        integer_singular >= r.singular_pinned,
        "so' {integer_singular} copias de singularidade cairam em pontos inteiros, \
         para {} vertices pregados",
        r.singular_pinned
    );
    assert_eq!(
        r.pinned,
        2 * (r.cycle_seams + r.singular_pinned),
        "sao DUAS componentes por costura de CICLO e duas por singularidade — e mais nada"
    );
}

/// ⭐⭐⭐ **A PARIDADE ENTRE OS DOIS CALENDÁRIOS.**
///
/// ⚠️ **A barra não é o bit, e a razão é estrutural:** a varredura do
/// [`crate::solve`] calcula os numeradores de um patch inteiro antes de aplicar
/// (Jacobi por patch) e o [`Relaxer`] aplica vértice a vértice (Gauss–Seidel). São
/// dois **calendários** sobre a **mesma** equação.
///
/// ⇒ a afirmação forte que resta, e que uma lei diferente quebraria de imediato: **o
/// mapa convergido do solver é um PONTO FIXO do relaxador**. Se as duas equações
/// divergissem, relaxar o mapa convergido movê-lo-ia muito.
#[test]
fn o_mapa_convergido_do_solver_e_ponto_fixo_do_relaxador() {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(12, 18, 1.0);
    // ⚠️ **Sem arredondamento nenhum**: o que se cobra aqui é a EQUAÇÃO, e pregar um
    // inteiro no meio mediria a equação mais o arredondamento.
    let (cut, combed, h, _singular) = chain(&mut mesh);
    let (mut map, _) = solve_with(
        &mesh,
        &cut,
        &combed,
        crate::solve::Step::uniform(h),
        crate::solve::SEAM_WEIGHT,
        40_000,
    );
    let scale: f32 = map
        .uv
        .iter()
        .flatten()
        .map(|z| z[0].abs().max(z[1].abs()))
        .fold(0.0, f32::max);
    assert!(
        scale > 1.0,
        "o mapa esta' vazio, e o gate mediria zero contra zero"
    );

    let mut rep = SolveReport::default();
    let a = assemble(
        &mesh,
        &cut,
        &combed,
        crate::solve::Step::uniform(h),
        &mut rep,
    );
    let r = Relaxer::new(&a, &cut, &combed, crate::solve::SEAM_WEIGHT);
    let moved = r.sweep(&mut map);
    assert!(
        moved < scale * 1.0e-3,
        "uma varredura do relaxador moveu {moved:.3e} num mapa de escala {scale:.1} — \
         as duas equacoes divergiram"
    );
}

/// ⭐⭐⭐ **A ESCADA FICA NO DEGRAU BARATO** — a régua que o próprio método manda medir.
///
/// ⛔ *Se a fracção que fica no degrau 1 for baixa, o tecto ou a tolerância estão mal
/// escolhidos, e o custo vai para o degrau caro — que é o que a escada adaptativa
/// existe para evitar.* A varredura que escolheu as duas constantes está no doc de
/// [`super::LOCAL_TOL`].
#[test]
fn a_escada_fica_no_degrau_barato() {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(12, 18, 1.0);
    let (cut, combed, h, singular) = chain(&mut mesh);
    let (_, r) = round_to_integers(
        &mesh,
        &cut,
        &combed,
        crate::solve::Step::uniform(h),
        RoundOptions::default(),
        &singular,
    );
    assert_eq!(
        r.level1 + r.level2,
        r.pinned,
        "todo arredondamento sobe exactamente um degrau da escada"
    );
    assert!(
        r.level1 * 10 >= r.pinned * 9,
        "so' {}/{} arredondamentos ficaram no degrau 1 — o tecto ou a tolerancia estao \
         mal escolhidos, e o custo foi para o degrau caro",
        r.level1,
        r.pinned
    );
    // ⚠️ **O degrau 3 (factorizacao esparsa directa) NAO existe**, e esta e' a coluna
    // que o justifica: enquanto o degrau 2 nao for preciso, o 3 nao teria consumidor
    // nenhum e nada o mediria.
    assert_eq!(
        r.level2, 0,
        "o degrau 2 foi preciso {} vezes — se isto passar a acontecer, o degrau 3 \
         (factorizacao directa) deixa de ser um diferimento sem consumidor",
        r.level2
    );
}
