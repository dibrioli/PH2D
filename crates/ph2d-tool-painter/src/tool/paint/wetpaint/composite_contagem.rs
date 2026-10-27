//! O que cada composite renderiza (só sob `cfg(test)`, para `sonda_custo_oleo`): `(chamadas, células do dirty do
//! motor, células renderizadas)` — com o Óleo Real a segunda cresce pelo [`ph2d_wet_paint::oleo_real::alcance`] —
//! e os ms do campo de altura, do pigmento e da sombra no papel.
use std::cell::Cell;

thread_local! {
    static C: Cell<(u64, u64, u64)> = const { Cell::new((0, 0, 0)) };
}

pub(in crate::tool::paint) fn nota(motor: usize, render: usize) {
    C.with(|c| {
        let (n, m, r) = c.get();
        c.set((n + 1, m + motor as u64, r + render as u64));
    });
}

pub(in crate::tool::paint) fn take() -> (u64, u64, u64) {
    C.with(Cell::take)
}

thread_local! {
    static T: Cell<(f64, f64)> = const { Cell::new((0.0, 0.0)) };
}

pub(in crate::tool::paint) fn tempos(pig: std::time::Duration, som: std::time::Duration) {
    T.with(|c| {
        let (a, b) = c.get();
        c.set((a + pig.as_secs_f64() * 1e3, b + som.as_secs_f64() * 1e3));
    });
}

thread_local! {
    static K: Cell<f64> = const { Cell::new(0.0) };
}

pub(in crate::tool::paint) fn campo(d: std::time::Duration) {
    K.with(|c| c.set(c.get() + d.as_secs_f64() * 1e3));
}

/// Os ms do `CampoAltura::de` (o campo de altura do Óleo Real, numa só thread).
pub(in crate::tool::paint) fn take_campo() -> f64 {
    K.with(Cell::take)
}

/// `(ms do pigmento, ms da sombra no papel)`.
pub(in crate::tool::paint) fn take_tempos() -> (f64, f64) {
    T.with(Cell::take)
}
