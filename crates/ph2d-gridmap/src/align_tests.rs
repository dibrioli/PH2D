use super::{Alignment, measure_alignment};
use crate::comb::Combed;
use crate::cut::{CutMesh, Seam, SeamSide};
use crate::solve::GridMap;

/// Monta o mínimo que a régua lê: quantos patches, e que costura liga quem com que
/// salto e que translação.
///
/// ⚠️ **A régua não toca em geometria nenhuma** — nem triângulos, nem `origin`, nem
/// direcções penteadas. *Uma fixtura que os construísse estaria a afirmar que eles
/// importam.*
fn setup(patches: usize, seams: &[(u32, u32, Option<i32>, [f32; 2])]) -> Alignment {
    let cut = CutMesh {
        origin: vec![Vec::new(); patches],
        tris: vec![Vec::new(); patches],
        tri_face: vec![Vec::new(); patches],
        seams: seams
            .iter()
            .map(|&(a, b, _, _)| Seam {
                arc: None,
                chain: Vec::new(),
                side: [
                    SeamSide {
                        patch: a,
                        local: Vec::new(),
                    },
                    SeamSide {
                        patch: b,
                        local: Vec::new(),
                    },
                ],
            })
            .collect(),
    };
    let combed = Combed {
        dir: vec![Vec::new(); patches],
        holonomy: vec![None; patches],
        jump: seams.iter().map(|&(_, _, k, _)| k).collect(),
    };
    let map = GridMap {
        uv: vec![Vec::new(); patches],
        shift: seams.iter().map(|&(_, _, _, t)| t).collect(),
    };
    measure_alignment(&cut, &combed, &map)
}

/// Uma árvore não tem ciclo nenhum, e uma peça sem ciclo não tem onde espiralar.
#[test]
fn a_single_seam_between_two_patches_makes_no_cycle() {
    let a = setup(2, &[(0, 1, Some(0), [7.0, -3.0])]);
    assert_eq!(a.cycles, 0);
    assert_eq!(a.flat_cycles, 0);
    assert_eq!(a.loose, 0);
    // ⚠️ Sem ciclo, a fracção é `1,0` — *não há linha nenhuma a espiralar*, e não `0`
    // por não haver denominador. Um zero ali leria como «tudo espirala».
    assert!((a.closed_fraction() - 1.0).abs() < 1e-6);
}

/// ⭐⭐⭐ **O TUBO E O PARAFUSO — o par que separa o COMPRIMENTO da volta da DERIVA.**
///
/// As duas peças têm a MESMA volta e o mesmo comprimento (`50` células); só a segunda
/// tem o passo de rosca. ⛔ *Uma régua que leia `|T|` dá `50` às duas e não distingue
/// coisa nenhuma* — foi o que a 1.ª redacção fez, e o corpus inteiro saiu bimodal.
#[test]
fn the_tube_closes_and_the_screw_does_not() {
    let tube = setup(
        2,
        &[(0, 1, Some(0), [0.0, 0.0]), (0, 1, Some(0), [50.0, 0.0])],
    );
    assert_eq!(tube.flat_cycles, 1);
    assert_eq!(tube.spiral_cycles, 0, "um tubo nao espirala");
    assert_eq!(
        tube.one_family_cycles, 1,
        "uma familia fecha, a outra da' a volta"
    );
    assert!(
        tube.drift_max <= super::INT_TOL,
        "deriva {}",
        tube.drift_max
    );
    assert!(
        (tube.span_max - 50.0).abs() < 1e-5,
        "comprimento {}",
        tube.span_max
    );

    let screw = setup(
        2,
        &[(0, 1, Some(0), [0.0, 0.0]), (0, 1, Some(0), [50.0, 1.0])],
    );
    assert_eq!(screw.flat_cycles, 1);
    assert_eq!(screw.spiral_cycles, 1, "⛔ o parafuso ESPIRALA");
    assert_eq!(screw.closed_cycles, 0);
    assert!(
        (screw.drift_max - 1.0).abs() < 1e-5,
        "deriva {}",
        screw.drift_max
    );
    // ⚠️ E o comprimento é o MESMO das duas — é isto que prova que a régua não o está a
    // ler como defeito.
    assert!((screw.span_max - 50.0).abs() < 1e-5);
}

/// ⭐⭐⭐ **O CALIBRE NÃO MOVE A DERIVA** — e é isto que separa esta régua de uma coluna
/// por costura.
///
/// Deslocar a origem do patch `1` soma a mesma constante às **duas** translações que
/// lhe chegam. ⛔ Uma régua por costura leria dois números diferentes; a holonomia do
/// ciclo lê a mesma deriva.
#[test]
fn the_gauge_does_not_move_the_drift() {
    let plain = setup(
        2,
        &[(0, 1, Some(0), [0.0, 0.0]), (0, 1, Some(0), [50.0, 1.0])],
    );
    let gauged = setup(
        2,
        &[(0, 1, Some(0), [5.0, -3.0]), (0, 1, Some(0), [55.0, -2.0])],
    );
    assert!((plain.drift_max - gauged.drift_max).abs() < 1e-5);
    assert_eq!(plain.spiral_cycles, gauged.spiral_cycles);
    assert_eq!(plain.flat_cycles, gauged.flat_cycles);
}

/// Translações iguais ⇒ a volta fecha nas DUAS famílias.
#[test]
fn equal_shifts_close_both_families() {
    let a = setup(
        2,
        &[(0, 1, Some(0), [4.0, 1.0]), (0, 1, Some(0), [4.0, 1.0])],
    );
    assert_eq!(a.flat_cycles, 1);
    assert_eq!(a.closed_cycles, 1);
    assert_eq!(a.spiral_cycles, 0);
    assert!(a.drift_max <= super::INT_TOL);
    assert!((a.closed_fraction() - 1.0).abs() < 1e-6);
}

/// ⚠️ **A ponte que abre um anel tem o MESMO patch dos dois lados** — ela é um laço, e
/// um laço é uma volta por si só. *Saltá-la perderia exactamente a volta que ela criou.*
#[test]
fn a_self_loop_seam_is_a_cycle_of_its_own() {
    let a = setup(1, &[(0, 0, Some(0), [3.0, 1.0])]);
    assert_eq!(a.cycles, 1);
    assert_eq!(a.flat_cycles, 1);
    assert_eq!(a.spiral_cycles, 1);
    assert!((a.drift_max - 1.0).abs() < 1e-5, "deriva {}", a.drift_max);
}

/// ⚠️ Um ciclo que RODA não entra na estatística: ali o salto é grandeza de calibre, e
/// o que é invariante é o ponto fixo.
#[test]
fn a_turning_cycle_is_counted_apart() {
    let a = setup(
        2,
        &[(0, 1, Some(0), [0.0, 0.0]), (0, 1, Some(1), [2.0, 0.0])],
    );
    assert_eq!(a.cycles, 1);
    assert_eq!(a.turning_cycles, 1);
    assert_eq!(a.flat_cycles, 0);
    assert_eq!(a.drift_max, 0.0);
}

/// ⛔ Uma costura sem salto lido é **contada** e não ligada — um `0` ali acoplaria duas
/// molduras que ninguém comparou.
#[test]
fn a_seam_without_a_jump_is_counted_not_assumed() {
    let a = setup(2, &[(0, 1, Some(0), [0.0, 0.0]), (0, 1, None, [2.0, 0.0])]);
    assert_eq!(a.loose, 1);
    assert_eq!(a.cycles, 0);
}

/// ⛔ Um salto que nem sequer é inteiro é um mapa que não está em grade naquele ciclo.
#[test]
fn a_fractional_jump_is_counted() {
    let a = setup(
        2,
        &[(0, 1, Some(0), [0.0, 0.0]), (0, 1, Some(0), [0.5, 0.0])],
    );
    assert_eq!(a.flat_cycles, 1);
    assert_eq!(a.fractional, 1);
    assert_eq!(a.closed_cycles, 0);
    // ⚠️ `b = 0` — a família de `v` fecha; o que não é inteiro é a de `u`.
    assert_eq!(a.one_family_cycles, 1);
}

/// ⭐⭐⭐ **O TORO LIMPO DEIXA AS DUAS FAMÍLIAS FECHAR** — e é o controlo que separa
/// «reticulado de ordem 2» de «defeito».
///
/// ⛔ Uma régua que chamasse defeito à ordem 2 reprovaria um toro perfeito.
#[test]
fn the_clean_torus_lets_both_families_close() {
    let a = setup(
        3,
        &[
            (0, 1, Some(0), [0.0, 0.0]),
            (0, 1, Some(0), [50.0, 0.0]),
            (0, 2, Some(0), [0.0, 0.0]),
            (0, 2, Some(0), [0.0, 30.0]),
        ],
    );
    assert_eq!(a.lattice_rank, 2);
    assert_eq!(a.u_period, 30, "as linhas de u fecham a cada 30 celulas");
    assert_eq!(a.v_period, 50, "as de v a cada 50");
}

/// ⛔ O parafuso: `L = ℤ·(N,1)` — nenhum múltiplo tem uma componente nula, logo
/// **nenhuma família fecha**. *É a assinatura exacta do espiral.*
#[test]
fn the_screw_closes_no_family() {
    let a = setup(
        2,
        &[(0, 1, Some(0), [0.0, 0.0]), (0, 1, Some(0), [50.0, 1.0])],
    );
    assert_eq!(a.lattice_rank, 1);
    assert_eq!(a.u_period, 0, "⛔ nenhuma volta deixa u onde estava");
    assert_eq!(a.v_period, 0);
}

/// ⭐⭐⭐ **A ÁRVORE NÃO MOVE O RETICULADO** — a volta DIAGONAL não muda resposta nenhuma.
///
/// ⛔ É este o gate que a contagem por ciclo **não** passa: acrescentar a diagonal faz
/// [`Alignment::spiral_cycles`] subir sem que a peça tenha mudado. *A base é minha; a
/// peça não.*
#[test]
fn a_diagonal_cycle_does_not_move_the_lattice() {
    let plain = setup(
        3,
        &[
            (0, 1, Some(0), [0.0, 0.0]),
            (0, 1, Some(0), [50.0, 0.0]),
            (0, 2, Some(0), [0.0, 0.0]),
            (0, 2, Some(0), [0.0, 30.0]),
        ],
    );
    let with_diagonal = setup(
        3,
        &[
            (0, 1, Some(0), [0.0, 0.0]),
            (0, 1, Some(0), [50.0, 0.0]),
            (0, 2, Some(0), [0.0, 0.0]),
            (0, 2, Some(0), [0.0, 30.0]),
            (1, 2, Some(0), [50.0, 30.0]),
        ],
    );
    assert_eq!(plain.lattice_rank, with_diagonal.lattice_rank);
    assert_eq!(plain.u_period, with_diagonal.u_period);
    assert_eq!(plain.v_period, with_diagonal.v_period);
    // ⚠️ E a contagem por ciclo MUDA — é a prova de que ela não é a régua.
    assert!(with_diagonal.spiral_cycles > plain.spiral_cycles);
}

/// Sem volta nenhuma o reticulado é trivial, e `0` ali significa **«não há volta»**, não
/// «não fecha». *Ler os dois zeros como o mesmo faria uma esfera sem costuras parecer o
/// pior caso.*
#[test]
fn no_cycle_means_a_trivial_lattice() {
    let a = setup(2, &[(0, 1, Some(0), [7.0, -3.0])]);
    assert_eq!(a.lattice_rank, 0);
    assert_eq!(a.u_period, 0);
    assert_eq!(a.v_period, 0);
}
