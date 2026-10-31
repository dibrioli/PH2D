//! Caminhantes de linhas da SONDA — espelho mínimo de `ph2d_wet_paint::par` (commit base
//! `341341b233`), só `Serial`/`Parallel` (sem a rota `Medida`). Os da crate são `pub(crate)`;
//! aqui são `#[inline(always)]` para que a cópia sob `#[target_feature]` os absorva.

use rayon::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Serial,
    Parallel,
}

#[inline(always)]
pub fn walk_rows<T: Send, F>(mode: Mode, a: &mut [T], stride: usize, f: F)
where
    F: Fn(usize, &mut [T]) + Send + Sync,
{
    match mode {
        Mode::Serial => a.chunks_mut(stride).enumerate().for_each(|(r, ra)| f(r, ra)),
        Mode::Parallel => a
            .par_chunks_mut(stride)
            .enumerate()
            .for_each(|(r, ra)| f(r, ra)),
    }
}

#[inline(always)]
pub fn walk_rows_reduce<T: Send, R, F, G>(
    mode: Mode,
    a: &mut [T],
    stride: usize,
    id: R,
    f: F,
    red: G,
) -> R
where
    R: Copy + Send + Sync,
    F: Fn(usize, &mut [T]) -> R + Send + Sync,
    G: Fn(R, R) -> R + Send + Sync,
{
    match mode {
        Mode::Serial => a
            .chunks_mut(stride)
            .enumerate()
            .fold(id, |acc, (r, ra)| red(acc, f(r, ra))),
        Mode::Parallel => a
            .par_chunks_mut(stride)
            .enumerate()
            .map(|(r, ra)| f(r, ra))
            .reduce(|| id, &red),
    }
}

#[inline(always)]
pub fn walk_rows2<A: Send, B: Send, F>(mode: Mode, a: &mut [A], b: &mut [B], stride: usize, f: F)
where
    F: Fn(usize, &mut [A], &mut [B]) + Send + Sync,
{
    match mode {
        Mode::Serial => a
            .chunks_mut(stride)
            .zip(b.chunks_mut(stride))
            .enumerate()
            .for_each(|(r, (ra, rb))| f(r, ra, rb)),
        Mode::Parallel => a
            .par_chunks_mut(stride)
            .zip(b.par_chunks_mut(stride))
            .enumerate()
            .for_each(|(r, (ra, rb))| f(r, ra, rb)),
    }
}

#[inline(always)]
pub fn walk_rows3<A: Send, B: Send, C: Send, F>(
    mode: Mode,
    a: &mut [A],
    b: &mut [B],
    c: &mut [C],
    stride: usize,
    f: F,
) where
    F: Fn(usize, &mut [A], &mut [B], &mut [C]) + Send + Sync,
{
    match mode {
        Mode::Serial => a
            .chunks_mut(stride)
            .zip(b.chunks_mut(stride))
            .zip(c.chunks_mut(stride))
            .enumerate()
            .for_each(|(r, ((ra, rb), rc))| f(r, ra, rb, rc)),
        Mode::Parallel => a
            .par_chunks_mut(stride)
            .zip(b.par_chunks_mut(stride))
            .zip(c.par_chunks_mut(stride))
            .enumerate()
            .for_each(|(r, ((ra, rb), rc))| f(r, ra, rb, rc)),
    }
}

#[inline(always)]
#[allow(clippy::too_many_arguments)]
pub fn walk_rows5<A: Send, B: Send, C: Send, D: Send, E: Send, F>(
    mode: Mode,
    a: &mut [A],
    b: &mut [B],
    c: &mut [C],
    d: &mut [D],
    e: &mut [E],
    stride: usize,
    f: F,
) where
    F: Fn(usize, &mut [A], &mut [B], &mut [C], &mut [D], &mut [E]) + Send + Sync,
{
    match mode {
        Mode::Serial => a
            .chunks_mut(stride)
            .zip(b.chunks_mut(stride))
            .zip(c.chunks_mut(stride))
            .zip(d.chunks_mut(stride))
            .zip(e.chunks_mut(stride))
            .enumerate()
            .for_each(|(r, ((((ra, rb), rc), rd), re))| f(r, ra, rb, rc, rd, re)),
        Mode::Parallel => a
            .par_chunks_mut(stride)
            .zip(b.par_chunks_mut(stride))
            .zip(c.par_chunks_mut(stride))
            .zip(d.par_chunks_mut(stride))
            .zip(e.par_chunks_mut(stride))
            .enumerate()
            .for_each(|(r, ((((ra, rb), rc), rd), re))| f(r, ra, rb, rc, rd, re)),
    }
}
