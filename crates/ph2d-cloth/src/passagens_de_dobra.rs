//! **Quantas passagens de Jacobi a dobra faz por varredura** — `PH2D_DOBRA_N`, omissão `1`.
//!
//! ⛔ É um knob de INVESTIGAÇÃO: não tem preço medido, e na malha grossa destrói o caimento
//! (`ph2d-panel-sculpt3d/src/rows_cloth_filter.rs`).
//!
//! ⚠️ Morava como `std::env::var` DENTRO do laço do solver — uma leitura do ambiente e uma `String`
//! por varredura — e a sonda `sonda_da_onda_da_prega` escrevia no ambiente com
//! `unsafe { std::env::set_var(..) }` para o varrer. Hoje o ambiente lê-se **uma vez por processo**,
//! e a sonda fixa o valor por [`fixar`], sem `unsafe` (auditoria de arquitectura 2026-09-12, A2: a
//! workspace proíbe `unsafe` em todo alvo).

use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

/// `0` = nada fixado: a palavra é do ambiente.
static FIXADO: AtomicU32 = AtomicU32::new(0);

/// As passagens em vigor: o valor [`fixar`]-do, senão o do ambiente (lido uma vez), senão `1`.
pub fn passagens() -> u32 {
    let f = FIXADO.load(Ordering::Relaxed);
    if f != 0 {
        return f;
    }
    static DO_AMBIENTE: OnceLock<u32> = OnceLock::new();
    *DO_AMBIENTE.get_or_init(|| {
        std::env::var("PH2D_DOBRA_N")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1)
    })
}

/// Fixa [`passagens`] para o PROCESSO inteiro (`0` devolve a palavra ao ambiente). Só para sondas.
pub fn fixar(n: u32) {
    FIXADO.store(n, Ordering::Relaxed);
}
