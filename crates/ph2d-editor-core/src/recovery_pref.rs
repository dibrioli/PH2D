//! ⭐ Settings ▸ Recovery: o intervalo da cópia de recuperação (P12 S, O15), em segundos (`0` = Off).
//!
//! ⚠️ Preferência da MÁQUINA do artista, não do projecto: a shell semeia-a de `~/.ph2d/prefs.txt`
//! (`recovery.minutes`), lê-a a cada quadro (vale já) e grava-a quando muda. Mora num atómico porque
//! a marca do menu (`menu_row_mark`) é pintada sem acesso ao `HeroScreen`.

use std::sync::atomic::{AtomicU64, Ordering};

/// As opções, pela ordem do submenu: Off e as oito do Aseprite (O15).
pub const OPTIONS_SECS: [u64; 9] = [0, 10, 30, 60, 120, 300, 600, 900, 1800];

/// A de fábrica (O15): 2 minutos.
pub const DEFAULT_SECS: u64 = 120;

static SECS: AtomicU64 = AtomicU64::new(DEFAULT_SECS);

/// O intervalo escolhido, em segundos (`0` = Off).
#[must_use]
pub fn secs() -> u64 {
    SECS.load(Ordering::Relaxed)
}

/// Escolhe o intervalo (o menu, ou a shell ao arrancar).
pub fn set_secs(secs: u64) {
    SECS.store(secs, Ordering::Relaxed);
}
