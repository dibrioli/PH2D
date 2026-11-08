//! **O canal com a família**: abrir a janela, a tecla capturada e o mapa novo. O painel não vê a
//! família; ela chama [`pede_abrir`], manda o acorde a [`oferece`] antes dos atalhos e toma o mapa
//! em [`toma_keymap`].

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use ph2d_pixel_keys::{Chord, Keymap, user_file};

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
        Some(p) => user_file::load_from(&p).0,
        None => user_file::load_user().0,
    }
}

/// Grava o delta (atómico: um irmão temporário e `rename`).
pub(crate) fn grava(k: &Keymap) -> std::io::Result<()> {
    let outro = FICHEIRO
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    match outro {
        Some(p) => user_file::save_to(&p, k),
        None => user_file::save_user(k),
    }
}

/// O Import/Export pedido pela janela: a família abre o diálogo de ficheiro do app
/// (`ph2d_app_host::modal`, a porta cronometrada) e devolve o caminho por [`entrega_troca`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Troca {
    Import,
    Export,
}

static TROCA: Mutex<Option<Troca>> = Mutex::new(None);
static ESCOLHIDO: Mutex<Option<(Troca, std::path::PathBuf)>> = Mutex::new(None);

pub(crate) fn pede_troca(t: Troca) {
    *TROCA
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(t);
}

/// O Import/Export que a janela pediu, uma vez (a família abre o diálogo).
#[must_use]
pub fn toma_troca() -> Option<Troca> {
    TROCA
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take()
}

/// O caminho que o artista escolheu no diálogo (a janela grava ou lê no próximo quadro).
pub fn entrega_troca(t: Troca, p: std::path::PathBuf) {
    *ESCOLHIDO
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some((t, p));
}

pub(crate) fn take_escolhido() -> Option<(Troca, std::path::PathBuf)> {
    ESCOLHIDO
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take()
}
