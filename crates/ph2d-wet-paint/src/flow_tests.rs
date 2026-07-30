//! Gates das portas da grade de FLUXO (filho de [`super`]).
//!
//! Os dois que carregam o peso: **`rf = 1` reduz LITERALMENTE** (a rede de
//! segurança de toda fase seguinte) e **as duas portas são INVERSAS** (o que
//! impede o campo de sair deslocado meia célula).

use super::*;

/// As razões varridas — inclui a 1 (o controle), primos e o teto.
const RATIOS: [usize; 8] = [1, 2, 3, 4, 5, 7, 8, 16];
/// Larguras que exercitam bloco EXATO e bloco PARCIAL de propósito.
const WIDTHS: [usize; 7] = [1, 2, 7, 8, 9, 64, 900];

#[test]
fn ratio_one_reduces_to_the_expression_the_engine_shipped() {
    // A identidade não se pede a um épsilon: `(x - 1) / 1 + 1` É `x`.
    for x in -3..=2000 {
        assert_eq!(
            fine_to_flow(x, 1),
            if x <= 0 { 0 } else { x },
            "fine_to_flow em rf=1 tem de ser a identidade"
        );
    }
    for c in 1..=2000 {
        assert_eq!(
            flow_probe(c, 4096, 1),
            c,
            "flow_probe em rf=1 é a identidade"
        );
    }
    // f64 BIT-exato, não aproximado.
    for k in 0..2000 {
        let xf = 1.0 + f64::from(k) * 0.37;
        assert!(
            flow_coord(xf, 1).to_bits() == xf.to_bits(),
            "flow_coord em rf=1 tem de devolver o proprio xf, bit a bit ({xf})"
        );
    }
}

#[test]
fn the_difference_scale_is_exactly_one_at_ratio_one() {
    // A regra de unidade da wave passa por AQUI, e ela é a rede de segurança de
    // toda fase seguinte: se `x * diff_scale(1)` não for `x` bit a bit, o
    // fingerprint do ADR-0134 quebra e ninguém sabe se foi a wave ou o fator.
    assert_eq!(diff_scale(1).to_bits(), 1.0f64.to_bits());
    for k in -5000..5000 {
        let x = f64::from(k) * 0.017_3;
        assert!(
            (x * diff_scale(1)).to_bits() == x.to_bits(),
            "diff_scale(1) tem de ser a identidade multiplicativa em f64 ({x})"
        );
    }
    // E ela É a discretização, não um número escolhido.
    assert!((diff_scale(4) - 0.25).abs() < f64::EPSILON);
}

#[test]
fn the_probe_index_at_ratio_one_is_the_fine_index() {
    let (w, h, s) = (900i32, 450i32, 902usize);
    for y in 1..=h {
        for x in [1, 2, 17, 449, w] {
            assert_eq!(
                probe_idx(x, y, w, h, s, 1),
                x as usize + y as usize * s,
                "probe_idx em rf=1 tem de ser o indice fino"
            );
        }
    }
}

#[test]
fn the_flow_geometry_at_ratio_one_equals_the_fine_grid() {
    for &w in &WIDTHS {
        for &h in &WIDTHS {
            let g = FlowGeom::new(w, h, 1);
            assert!(g.is_identity());
            assert_eq!((g.w, g.h), (w, h), "dims");
            assert_eq!(g.s, w + 2, "stride");
            assert_eq!(g.rows, h + 2, "rows");
            assert_eq!(g.cells, (w + 2) * (h + 2), "cells");
        }
    }
}

#[test]
fn the_two_doors_are_inverses() {
    for &rf in &RATIOS {
        for &w in &WIDTHS {
            let (fw, _) = flow_dims(w, w, rf);
            for c in 1..=fw as i32 {
                let x = flow_probe(c, w as i32, rf);
                assert!(
                    (1..=w as i32).contains(&x),
                    "flow_probe saiu da folha: rf {rf} w {w} c {c} -> {x}"
                );
                assert_eq!(
                    fine_to_flow(x, rf),
                    c,
                    "as portas discordam sobre o BLOCO: rf {rf} w {w} c {c} -> fino {x}"
                );
            }
        }
    }
}

#[test]
fn every_fine_cell_lands_in_a_flow_cell_that_exists() {
    for &rf in &RATIOS {
        for &w in &WIDTHS {
            let (fw, _) = flow_dims(w, w, rf);
            for x in 1..=w as i32 {
                let c = fine_to_flow(x, rf);
                assert!(
                    (1..=fw as i32).contains(&c),
                    "celula fina sem casa: rf {rf} w {w} x {x} -> {c} (fw {fw})"
                );
            }
        }
    }
}

#[test]
fn the_continuous_coord_agrees_with_the_probe() {
    // `flow_coord` é a inversa CONTÍNUA do `flow_probe`: no ponto que o probe
    // escolheu, ela tem de devolver o índice daquela célula. (O último bloco é
    // pulado: lá o probe é CLAMPADO à folha, então a concordância exata não
    // vale — e é por isso que o clamp está escrito no doc dele.)
    for &rf in &RATIOS {
        for &w in &WIDTHS {
            let (fw, _) = flow_dims(w, w, rf);
            for c in 1..fw as i32 {
                let x = flow_probe(c, w as i32, rf);
                let u = flow_coord(f64::from(x), rf);
                assert!(
                    (u - f64::from(c)).abs() < 1e-12,
                    "coord contínua discorda do probe: rf {rf} w {w} c {c} -> {u}"
                );
            }
        }
    }
}

#[test]
fn the_dims_never_lose_a_partial_block() {
    // 9 células com rf 4 são TRÊS blocos (4 + 4 + 1), não dois: perder o bloco
    // parcial seria perder a coluna em que a água corre.
    assert_eq!(flow_dims(9, 9, 4), (3, 3));
    assert_eq!(flow_dims(8, 8, 4), (2, 2));
    assert_eq!(flow_dims(1, 1, 16), (1, 1));
    for &rf in &RATIOS {
        for &w in &WIDTHS {
            let (fw, _) = flow_dims(w, w, rf);
            assert!(fw * rf >= w, "dims perdeu um bloco: rf {rf} w {w} fw {fw}");
            assert!(fw >= 1, "dims degenerou a zero: rf {rf} w {w}");
        }
    }
}

#[test]
fn the_ratio_is_clamped_at_the_door_not_at_the_caller() {
    assert_eq!(clamp_ratio(0), MIN_FLOW_RATIO);
    assert_eq!(clamp_ratio(999), MAX_FLOW_RATIO);
    // Uma razão fora da faixa não pode produzir geometria degenerada em NENHUMA
    // porta — é o que torna seguro o número vir de um slider.
    assert_eq!(flow_dims(64, 64, 0), flow_dims(64, 64, MIN_FLOW_RATIO));
    assert_eq!(fine_to_flow(7, 0), fine_to_flow(7, MIN_FLOW_RATIO));
    assert_eq!(flow_probe(3, 64, 999), flow_probe(3, 64, MAX_FLOW_RATIO));
}
