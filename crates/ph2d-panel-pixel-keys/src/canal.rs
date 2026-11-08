//! **O canal com a família**: abrir a janela, a tecla capturada e o mapa novo. O painel não vê a
//! família; ela chama [`pede_abrir`], manda o acorde a [`oferece`] antes dos atalhos e toma o mapa
//! em [`toma_keymap`].

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use ph2d_pixel_keys::{Chord, Keymap};

static ABRIR: AtomicBool = AtomicBool::new(false);
static A_CAPTURAR: AtomicBool = AtomicBool::new(false);
static TECLA: Mutex<Option<Chord>> = Mutex::new(None);
static NOVO: Mutex<Option<Keymap>> = Mutex::new(None);

/// Abre a janela no próximo quadro (menu Edit ▸ Keyboard Shortcuts, `Ctrl+Alt+Shift+K`).
pub fn pede_abrir() {
    ABRIR.store(true, Ordering::Relaxed);
}

pub(crate) fn take_abrir() -> bool {
    ABRIR.swap(false, Ordering::Relaxed)
}

/// A janela espera uma tecla: o acorde é dela, não dos atalhos.
#[must_use]
pub fn a_capturar() -> bool {
    A_CAPTURAR.load(Ordering::Relaxed)
}

pub(crate) fn publica_captura(on: bool) {
    A_CAPTURAR.store(on, Ordering::Relaxed);
}

/// A família oferece o acorde premido; `true` = a captura tomou-o (não o despache).
pub fn oferece(chord: Chord) -> bool {
    if !a_capturar() {
        return false;
    }
    *TECLA
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(chord);
    true
}

pub(crate) fn take_tecla() -> Option<Chord> {
    TECLA
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take()
}

pub(crate) fn publica_keymap(k: Keymap) {
    *NOVO
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(k);
}

/// O mapa depois da última mudança na janela, uma vez (a família põe-no em `PixelApp::keys`).
#[must_use]
pub fn toma_keymap() -> Option<Keymap> {
    NOVO.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take()
}

static FICHEIRO: Mutex<Option<std::path::PathBuf>> = Mutex::new(None);

/// Troca o ficheiro do utilizador (`None` = `~/.ph2d/pixel-keys.txt`): os testes nunca tocam o do
/// dono.
pub fn usa_ficheiro(p: Option<std::path::PathBuf>) {
    *FICHEIRO
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = p;
}

/// O mapa do ficheiro do utilizador (ou do que [`usa_ficheiro`] deu).
pub(crate) fn carrega() -> Keymap {
    let outro = FICHEIRO
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    match outro {
        Some(p) => Keymap::from_text(&std::fs::read_to_string(p).unwrap_or_default()).0,
        None => crate::shim::load_user().0,
    }
}

/// Grava o delta (atómico: um irmão temporário e `rename`).
pub(crate) fn grava(k: &Keymap) -> std::io::Result<()> {
    let outro = FICHEIRO
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    match outro {
        Some(p) => crate::shim::save_to(&p, k),
        None => crate::shim::save_user(k),
    }
}

/// O ficheiro do Import/Export: ao lado do do utilizador.
pub(crate) fn ficheiro_de_troca() -> Option<std::path::PathBuf> {
    let outro = FICHEIRO
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    outro
        .or_else(crate::shim::user_path)
        .map(|p| p.with_file_name("pixel-keys-export.txt"))
}
