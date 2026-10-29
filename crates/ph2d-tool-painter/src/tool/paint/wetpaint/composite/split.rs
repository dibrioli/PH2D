use std::cell::Cell;
use std::time::Duration;

thread_local! {
    static ENGINE_US: Cell<u64> = const { Cell::new(0) };
    static PIXEL_US: Cell<u64> = const { Cell::new(0) };
}

pub(in crate::tool::paint) fn note_engine(d: Duration) {
    ENGINE_US.with(|c| c.set(c.get() + d.as_micros() as u64));
}

pub(in crate::tool::paint) fn note_pixel(d: Duration) {
    PIXEL_US.with(|c| c.set(c.get() + d.as_micros() as u64));
}

/// Drena os dois baldes em milissegundos — `(precisa do motor, não precisa)`.
pub(in crate::tool::paint) fn take() -> (f64, f64) {
    let e = ENGINE_US.with(Cell::take);
    let p = PIXEL_US.with(Cell::take);
    (e as f64 / 1000.0, p as f64 / 1000.0)
}
