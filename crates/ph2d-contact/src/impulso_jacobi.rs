//! doc 121 §9.19 (5) — **os impulsos por JACOBI COM MÉDIA**, a composição que um dispositivo corre em paralelo.
//!
//! Cada restrição corre o MESMO [`super::resolve_um`] (a normal, o atrito e o rolamento, pela ordem de sempre)
//! sobre a FOTOGRAFIA das velocidades da iteração, e o que ela muda nas duas peças acumula-se; no fim cada peça
//! aplica a MÉDIA dos seus contactos — a lei da separação ([`super::super::varredura`]) levada à velocidade.
//! O `λ` de cada restrição continua acumulado e preso como no Gauss–Seidel.

use super::{Leis, Movimento, Pecas, Restricao, resolve_um};

pub(super) fn uma_iteracao(
    restricoes: &mut [Restricao],
    mov: &mut Movimento<'_>,
    w: &mut [f32],
    pecas: &Pecas<'_>,
    leis: Leis,
) {
    let n = w.len();
    let (vel0, w0, giro0) = (mov.vel.to_vec(), w.to_vec(), mov.giro.to_vec());
    let mut dv = vec![[0.0_f32; 2]; n];
    let mut dw = vec![0.0_f32; n];
    let mut dg = vec![0.0_f32; n];
    let mut quantos = vec![0_u32; n];
    for r in restricoes.iter_mut() {
        let (lo, hi) = (r.lo, r.hi);
        resolve_um(r, mov, w, pecas, leis);
        for k in [lo, hi] {
            dv[k][0] += mov.vel[k][0] - vel0[k][0];
            dv[k][1] += mov.vel[k][1] - vel0[k][1];
            dw[k] += w[k] - w0[k];
            dg[k] += mov.giro[k] - giro0[k];
            quantos[k] += 1;
            // A fotografia volta para a restrição seguinte.
            mov.vel[k] = vel0[k];
            w[k] = w0[k];
            mov.giro[k] = giro0[k];
        }
    }
    for k in 0..n {
        if quantos[k] == 0 {
            continue;
        }
        #[expect(
            clippy::cast_precision_loss,
            reason = "contactos de uma peca, muito abaixo de 2^24"
        )]
        let inv = 1.0 / quantos[k] as f32;
        mov.vel[k] = [vel0[k][0] + dv[k][0] * inv, vel0[k][1] + dv[k][1] * inv];
        w[k] = w0[k] + dw[k] * inv;
        mov.giro[k] = giro0[k] + dg[k] * inv;
    }
}

/// O Gauss–Seidel por CORES: cada restrição toma a menor cor que nenhuma outra restrição das suas duas peças já
/// tem (gulosa, pela ordem de sempre), e a lista passa a ir cor a cor — dentro de uma cor nenhuma escreve o que a
/// outra lê, e correr a cor em série ou em paralelo dá o mesmo.
pub(super) fn por_cores(restricoes: &mut Vec<Restricao>, n: usize) {
    let mut usadas: Vec<u64> = vec![0; n];
    let mut cor = Vec::with_capacity(restricoes.len());
    for r in restricoes.iter() {
        let ocupadas = usadas[r.lo] | usadas[r.hi];
        let c = (!ocupadas).trailing_zeros().min(63);
        usadas[r.lo] |= 1 << c;
        usadas[r.hi] |= 1 << c;
        cor.push(c);
    }
    let mut com_cor: Vec<(u32, usize, Restricao)> = restricoes
        .drain(..)
        .enumerate()
        .map(|(i, r)| (cor[i], i, r))
        .collect();
    com_cor.sort_by_key(|(c, i, _)| (*c, *i));
    restricoes.extend(com_cor.into_iter().map(|(_, _, r)| r));
}
