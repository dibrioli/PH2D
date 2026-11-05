//! O plano de passadas de um passo depois do rebuild, nas duas leis da tinta.

use crate::{
    CAMPO_OLEO, PLANO_BORDAS, PLANO_CAMPO, PLANO_PROJETA, PLANO_SEDIMENTA, PLANO_SUAVE, PLANO_VMAX,
    T_BORDAS, T_COMMIT, T_CURA, T_SEDIMENTA, T_TRANSPORTE,
};

/// Passadas no máximo num passo depois do rebuild (campo 7 + sedimenta 2 + bordas 2 + projecção 5 + vmax 1), mais as da
/// lei `Pigmento` (cura 1 + transporte 2 + sedimenta 2 + bordas 1); folga até à `porta` (96 ÷ 3).
pub(super) const MAX_PLANO: usize = 32;

/// O plano do quadro `n`. A cura vai à cabeça: corre logo depois da secagem, como no `sim_step_stage`.
pub(super) fn do_passo(n: u64, sedimenta: bool, com_tinta: bool, cura: bool) -> Vec<usize> {
    let mut plano = Vec::with_capacity(MAX_PLANO);
    if cura {
        plano.push(T_CURA);
    }
    let base = if n.is_multiple_of(4) {
        &PLANO_CAMPO[..]
    } else {
        &PLANO_SUAVE[..]
    };
    for &k in base {
        match k {
            6 if com_tinta => plano.push(CAMPO_OLEO),
            3 if com_tinta => plano.extend([T_TRANSPORTE, 3, T_COMMIT]),
            _ => plano.push(k),
        }
    }
    if sedimenta {
        plano.extend(PLANO_SEDIMENTA);
        if com_tinta {
            plano.extend([T_SEDIMENTA, T_COMMIT]);
        }
    }
    plano.extend(PLANO_BORDAS);
    if com_tinta {
        plano.push(T_BORDAS);
    }
    if n.is_multiple_of(3) {
        plano.extend(PLANO_PROJETA);
    }
    plano.extend(PLANO_VMAX);
    plano
}
