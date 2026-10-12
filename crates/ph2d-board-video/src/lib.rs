//! W18 ([ADR-0185]) — **o vídeo do quadro, o lado que fala com o MUNDO**: a miniatura pela rede
//! ([`thumb::fetch`]) e o tocador oficial no navegador da família Chrome que a pessoa já tem ([`Player`]),
//! conduzido pelo CDP num tubo e projectado no canvas. Sem navegador ([`PlayerState::NoBrowser`]): a
//! miniatura e «Abrir no navegador». Nada aqui corre no fio da janela.
//!
//! [ADR-0185]: ../../../docs/architecture/decisions/0185-o-video-do-quadro-toca-no-navegador-da-pessoa-projectado-no-canvas.md

mod browser;
mod playback;
mod player;
pub mod thumb;

pub use browser::find as find_browser;
pub use playback::{Phase, Playback};
pub use player::{Button, Frame, Mouse, Player, PlayerState};

/// «Abrir no navegador»: o `url` (só `https://`) no navegador por omissão do sistema. `false` = não abriu.
#[must_use]
pub fn open_in_browser(url: &str) -> bool {
    if !url.starts_with("https://") {
        return false;
    }
    let opener = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(windows) {
        "explorer"
    } else {
        "xdg-open"
    };
    let child = std::process::Command::new(opener)
        .arg(url)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
    // Esperado num fio: um filho largado sem `wait` fica zombie até o PH2D sair.
    child.is_ok_and(|mut c| {
        std::thread::spawn(move || {
            let _ = c.wait();
        });
        true
    })
}

/// Os JPEG (a miniatura, os quadros do tocador) → RGBA.
#[must_use]
pub fn decode_jpeg(bytes: &[u8]) -> Option<Frame> {
    player::decode_jpeg(bytes)
}

#[cfg(test)]
mod tests;
