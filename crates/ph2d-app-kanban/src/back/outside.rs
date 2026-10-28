//! ⭐ **O que o verso pede ao MUNDO** — abrir um link no navegador, abrir um anexo no programa do
//! sistema, escolher ficheiros no diálogo nativo, copiar o link do cartão. A vista só ENFILEIRA
//! (`Back::opened`, `Back::pick`, `Back::copy`); a shell serve a fila uma vez por imagem
//! (`ph2d_app_kanban::service`, ao lado do `io::service` do Quadro) por um [`World`] — o do sistema,
//! ou o de um teste, que responde no lugar do diálogo pela MESMA função.

use crate::docs::KanbanDocs;
use ph2d_i18n::{tr, tr_with};
use ph2d_kanban_model::FileId;
use std::path::{Path, PathBuf};

/// Um pedido de abrir fora do PH2D.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outside {
    /// Um link (de um anexo ou da descrição): o navegador do sistema.
    Link(String),
    /// Um anexo de ficheiro: uma cópia no temporário, aberta pelo programa do sistema.
    File(FileId, String),
}

/// O mundo de fora: o diálogo de ficheiros e o «abrir com o programa do sistema».
pub trait World {
    /// O diálogo nativo «escolher ficheiros» (`None` = cancelado).
    fn pick_files(&mut self) -> Option<Vec<PathBuf>>;
    /// Abre `target` (um link ou um caminho) no programa do sistema. `false` = não abriu.
    fn open(&mut self, target: &str) -> bool;
}

/// O mundo real: o `rfd` pela porta dos diálogos modais (que declara o congelamento) e o
/// `xdg-open`/`open`/`explorer`.
pub struct System;

impl World for System {
    fn pick_files(&mut self) -> Option<Vec<PathBuf>> {
        let dialog = rfd::FileDialog::new().set_title(tr("kanban.back.attach_computer"));
        ph2d_app_host::modal::pick_files(dialog)
    }

    fn open(&mut self, target: &str) -> bool {
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
}

/// O que a fila deu à shell: o texto a copiar e os avisos para o ecrã.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Served {
    pub copy: Option<String>,
    pub notices: Vec<String>,
}

/// O link que se abre no navegador: `http(s)` e `mailto` como estão; sem esquema («example.com»,
/// o que o Trello também aceita), `https://`. Outro esquema (`file:`, `javascript:`) não se abre.
#[must_use]
pub fn web_link(u: &str) -> Option<String> {
    let u = u.trim();
    let lower = u.to_ascii_lowercase();
    if lower.starts_with("https://") || lower.starts_with("http://") || lower.starts_with("mailto:")
    {
        return Some(u.to_owned());
    }
    let scheme = u.split_once(':').map(|(s, _)| s);
    let bare = scheme.is_none_or(|s| s.contains('.') || s.chars().all(|c| c.is_ascii_digit()));
    (bare && !u.is_empty() && !u.contains(char::is_whitespace)).then(|| format!("https://{u}"))
}

/// A cópia de um anexo no temporário (`<tmp>/ph2d-kanban/<sha>/<nome>`): o nome do anexo sem
/// pastas, para o programa do sistema reconhecer o tipo. Já lá estando (o mesmo conteúdo), fica.
fn temp_copy(id: FileId, name: &str, bytes: &[u8]) -> Option<PathBuf> {
    let leaf = Path::new(name)
        .file_name()
        .and_then(|n| n.to_str())
        .filter(|n| !n.is_empty() && *n != "..")
        .unwrap_or("attachment");
    let dir = std::env::temp_dir().join("ph2d-kanban").join(id.hex());
    std::fs::create_dir_all(&dir).ok()?;
    let path = dir.join(leaf);
    if std::fs::metadata(&path).is_ok_and(|m| m.len() == bytes.len() as u64) {
        return Some(path);
    }
    std::fs::write(&path, bytes).ok()?;
    Some(path)
}

impl KanbanDocs {
    /// ⭐ Serve a fila de TODAS as abas por `world`: o diálogo pedido anexa o que se escolheu ao
    /// cartão do verso, os links abrem-se, os anexos de ficheiro abrem-se de uma cópia temporária.
    pub fn serve(&mut self, world: &mut dyn World) -> Served {
        let mut out = Served {
            notices: std::mem::take(&mut self.notices),
            ..Served::default()
        };
        let ids: Vec<u64> = self.tabs.iter().map(crate::tab::Tab::id).collect();
        for id in ids {
            let Some(b) = self.tab_mut(id).and_then(|t| t.back.as_deref_mut()) else {
                continue;
            };
            let (card, pick, opened, copy) = (
                b.card,
                std::mem::take(&mut b.pick),
                std::mem::take(&mut b.opened),
                b.copy.take(),
            );
            if let Some(text) = copy {
                out.copy = Some(text);
                out.notices.push(tr("kanban.back.link_copied").to_owned());
            }
            if pick && let Some(paths) = world.pick_files() {
                super::attach::attach_paths(self, id, card, &paths);
                out.notices.append(&mut self.notices);
            }
            for o in opened {
                let (shown, target) = match &o {
                    Outside::Link(u) => (u.clone(), web_link(u)),
                    Outside::File(f, name) => {
                        let bytes = self.set.borrow().file(*f).map(|k| k.bytes.0.clone());
                        let path = bytes.and_then(|b| temp_copy(*f, name, &b));
                        (name.clone(), path.map(|p| p.to_string_lossy().into_owned()))
                    }
                };
                if !target.is_some_and(|t| world.open(&t)) {
                    out.notices
                        .push(tr_with("kanban.back.cannot_open", &[("t", &shown)]));
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::web_link;

    #[test]
    fn only_web_links_open_and_a_bare_host_gets_https() {
        assert_eq!(web_link("https://a.b/c").as_deref(), Some("https://a.b/c"));
        assert_eq!(web_link("mailto:x@y.z").as_deref(), Some("mailto:x@y.z"));
        assert_eq!(web_link("example.com/x").as_deref(), Some("https://example.com/x"));
        assert_eq!(web_link("localhost:8080").as_deref(), None);
        assert_eq!(web_link("javascript:alert(1)"), None);
        assert_eq!(web_link("file:///etc/passwd"), None);
        assert_eq!(web_link("two words"), None);
    }
}
