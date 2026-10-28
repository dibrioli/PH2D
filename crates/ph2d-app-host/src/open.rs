//! ⭐ **Abrir FORA do PH2D** — um link no navegador, um ficheiro no programa do sistema
//! (`xdg-open`/`open`/`explorer`). Uma só porta para todas as famílias (o vídeo do Quadro, o verso do
//! Kanban), com uma só política: `http(s)`/`mailto` sim; `file:`, `javascript:` e o resto não. Um
//! ficheiro só se abre se mora no temporário do sistema (a cópia que a família lá gravou).

use std::path::Path;

/// O link pode sair para o navegador: `http://`, `https://` ou `mailto:` (sem diferença de caixa).
#[must_use]
pub fn is_web_link(url: &str) -> bool {
    let lower = url.trim_start().to_ascii_lowercase();
    ["https://", "http://", "mailto:"]
        .iter()
        .any(|s| lower.starts_with(s))
}

/// Abre `url` no navegador do sistema, se [`is_web_link`]. `false` = recusado ou não abriu.
#[must_use]
pub fn open_link(url: &str) -> bool {
    is_web_link(url) && spawn(url.trim().as_ref())
}

/// Abre `path` no programa do sistema, só se o ficheiro existe DENTRO do temporário do sistema.
#[must_use]
pub fn open_temp_file(path: &Path) -> bool {
    in_temp(path) && spawn(path.as_os_str())
}

fn in_temp(path: &Path) -> bool {
    let (Ok(p), Ok(t)) = (path.canonicalize(), std::env::temp_dir().canonicalize()) else {
        return false;
    };
    p.is_file() && p.starts_with(t)
}

fn spawn(target: &std::ffi::OsStr) -> bool {
    let opener = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(windows) {
        "explorer"
    } else {
        "xdg-open"
    };
    let child = std::process::Command::new(opener)
        .arg(target)
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

#[cfg(test)]
mod tests {
    use super::{in_temp, is_web_link};
    use std::path::Path;

    #[test]
    fn only_web_links_and_files_in_the_temp_dir_leave_the_app() {
        assert!(is_web_link("https://youtu.be/x") && is_web_link("HTTP://a.b"));
        assert!(is_web_link("mailto:x@y.z"));
        assert!(!is_web_link("javascript:alert(1)") && !is_web_link("file:///etc/passwd"));
        assert!(!is_web_link("example.com"));
        let tmp = std::env::temp_dir().join(format!("ph2d_open_{}.txt", std::process::id()));
        std::fs::write(&tmp, b"x").expect("grava");
        let ok = in_temp(&tmp);
        let _ = std::fs::remove_file(&tmp);
        assert!(ok, "a cópia no temporário abre-se");
        assert!(
            !in_temp(Path::new("/etc/passwd")),
            "fora do temporário, não"
        );
        assert!(
            !in_temp(&std::env::temp_dir()),
            "uma pasta não é um ficheiro"
        );
    }
}
