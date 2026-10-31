//! Interruptores SÓ das sondas (override em execução, nunca `const`): desligar uma parcela do depósito de cada vez na
//! MESMA corrida (`sonda_relevo_oleo`, doc `docs/Painter/Arquitetura/RELEVO_oleo_ondas_e_crateras.md` §Mecanismo 2).
//! Todos a `false` = o produto, ao bit.

use std::sync::atomic::{AtomicBool, Ordering};

/// O tecto suave de 3000 do depósito (`transfer.rs`) desligado: a célula soma `old + v` sempre.
pub static SEM_TETO: AtomicBool = AtomicBool::new(false);

#[inline]
pub(super) fn sem_teto() -> bool {
    SEM_TETO.load(Ordering::Relaxed)
}

/// O piso do espalhar desligado (a lei de antes do Mecanismo 2: o 1.º toque podia cavar abaixo da camada de baixo).
pub static SEM_PISO: AtomicBool = AtomicBool::new(false);

#[inline]
pub(super) fn sem_piso() -> bool {
    SEM_PISO.load(Ordering::Relaxed)
}
