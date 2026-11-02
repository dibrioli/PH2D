//! Ferramenta de MEDIÇÃO (J11, FPS com o Real Oil): quem marca o sujo do motor — chamadas e células por chamador
//! de [`super::Engine::merge_dirty`]/[`super::Engine::mark_dirty_full`]. Desligada, custa uma leitura atómica.

use std::sync::atomic::{AtomicBool, Ordering::Relaxed};

/// `true` = conta (por fio).
#[doc(hidden)]
pub static CONTA_SUJO: AtomicBool = AtomicBool::new(false);

thread_local! {
    static TABELA: std::cell::RefCell<std::collections::BTreeMap<String, (u64, u64)>> =
        const { std::cell::RefCell::new(std::collections::BTreeMap::new()) };
}

#[track_caller]
pub(super) fn regista(celulas: u64) {
    if !CONTA_SUJO.load(Relaxed) {
        return;
    }
    let l = std::panic::Location::caller();
    let k = format!("{}:{}", l.file(), l.line());
    TABELA.with(|t| {
        let mut t = t.borrow_mut();
        let e = t.entry(k).or_default();
        e.0 += 1;
        e.1 += celulas;
    });
}

/// A tabela deste fio desde a última leitura: `(chamador, chamadas, células)`.
#[doc(hidden)]
#[must_use]
pub fn tira() -> Vec<(String, u64, u64)> {
    TABELA.with(|t| {
        std::mem::take(&mut *t.borrow_mut())
            .into_iter()
            .map(|(k, (n, c))| (k, n, c))
            .collect()
    })
}
