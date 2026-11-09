//! ⭐ **A recuperação do PROJECTO INTEIRO depois de um fecho à força** (P12 item S, dono 08/11).
//!
//! A lei é nossa (`docs/PixelArt/P12_plano.md` §4.3; o Trial do Aseprite não grava recuperação, O16):
//!
//! - a cada [`Period`] (de fábrica 2 min, O15) grava-se uma cópia **só se os bytes mudaram** desde a
//!   última ([`Autosave`]); os bytes são os do Save, byte a byte, e quem os produz é a shell;
//! - a cópia mora fora do projecto, em `<dados>/ph2d/recovery/<sessão>/` ([`data_dir`]), e é escrita
//!   de forma atómica ([`atomic::write_atomic`]: temporário + `fsync` + `rename` + `fsync` da pasta);
//! - a sessão segura uma tranca do SO enquanto vive ([`Session`]); o fecho normal e o Save apagam as cópias;
//! - ao arrancar, [`scan`] poda o que tem mais de uma semana e devolve as cópias de sessões mortas,
//!   que a janela em cartões ([`card`], estado [`Prompt`]) oferece: `Restore` · `Discard` · `Later`.
//!
//! O `PH2D_SCRIPT_RECOVERY_DIR` do editor de scripts (`ph2d-app-code::recovery`) usa a MESMA raiz
//! ([`data_dir`]), noutra subpasta.

mod app;
mod ask;
pub mod atomic;
mod autosave;
pub mod card;
mod prompt;
mod session;

pub use app::{IDLE, Quit, Recovery, meta_of};
pub use ask::{ago, body, size};
pub use autosave::{Autosave, Offer, Period, fold, print_of};
pub use prompt::{Choice, Prompt};
pub use session::{Meta, Orphan, Session, is_held, scan};

use std::path::PathBuf;

/// Quanto uma cópia de uma sessão morta fica guardada (O15: «Keep edited sprite data for 1 Week»).
pub const KEEP_SECS: u64 = 7 * 24 * 60 * 60;

/// A raiz de dados do PH2D com a subpasta `sub`: `$XDG_DATA_HOME/ph2d/<sub>` ·
/// `$HOME/.local/share/ph2d/<sub>` · `%APPDATA%\ph2d\<sub>`; `override_var`, se definida e não
/// vazia, ganha a todos. `None` = o sistema não dá nenhuma.
#[must_use]
pub fn data_dir(override_var: &str, sub: &str) -> Option<PathBuf> {
    let var = |k: &str| {
        std::env::var_os(k)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    };
    var(override_var)
        .or_else(|| var("XDG_DATA_HOME").map(|d| d.join("ph2d").join(sub)))
        .or_else(|| var("HOME").map(|h| h.join(".local/share/ph2d").join(sub)))
        .or_else(|| var("APPDATA").map(|d| d.join("ph2d").join(sub)))
}

/// A pasta da recuperação do projecto (`PH2D_RECOVERY_DIR` ganha).
#[must_use]
pub fn default_root() -> Option<PathBuf> {
    data_dir("PH2D_RECOVERY_DIR", "recovery")
}

/// Segundos desde 1970 (0 se o relógio estiver antes disso).
#[must_use]
pub fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}
