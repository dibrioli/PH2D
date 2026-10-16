//! ⛔ **Nunca se perde texto em silêncio** — abrir outro projecto esquece todo o estado por
//! `ScriptId` (os ids recomeçam em 1 em cada projecto), e os textos POR GRAVAR do projecto anterior
//! (abas abertas e fechadas com ●) viram ficheiros de recuperação, um por script, com um aviso no
//! ecrã a dizer onde. A pasta: `PH2D_SCRIPT_RECOVERY_DIR` · `$XDG_DATA_HOME/ph2d/recovered-scripts` ·
//! `$HOME/.local/share/ph2d/recovered-scripts` · `%APPDATA%\ph2d\recovered-scripts`.

use ph2d_editor_core::Toast;
use ph2d_i18n::tr_with;
use std::path::{Path, PathBuf};

const SUB: &str = "recovered-scripts";

/// A pasta por omissão (`None` = o sistema não dá nenhuma).
pub(crate) fn default_dir() -> Option<PathBuf> {
    let var = |k: &str| std::env::var_os(k).filter(|v| !v.is_empty()).map(PathBuf::from);
    var("PH2D_SCRIPT_RECOVERY_DIR")
        .or_else(|| var("XDG_DATA_HOME").map(|d| d.join("ph2d").join(SUB)))
        .or_else(|| var("HOME").map(|h| h.join(".local/share/ph2d").join(SUB)))
        .or_else(|| var("APPDATA").map(|d| d.join("ph2d").join(SUB)))
}

/// Um nome de ficheiro a partir do nome do script (sem separadores de caminho).
fn file_name(stamp: u64, n: usize, name: &str) -> String {
    let clean: String = name
        .chars()
        .map(|c| if matches!(c, '/' | '\\' | ':' | '\0') { '_' } else { c })
        .collect();
    if n == 0 {
        format!("{stamp}-{clean}")
    } else {
        format!("{stamp}-{n}-{clean}")
    }
}

fn write_all(dir: &Path, unsaved: &[(String, String)]) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    for (name, text) in unsaved {
        let path = (0..)
            .map(|n| dir.join(file_name(stamp, n, name)))
            .find(|p| !p.exists())
            .unwrap_or_else(|| dir.join(file_name(stamp, 0, name)));
        std::fs::write(path, text)?;
    }
    Ok(())
}

/// Grava os textos `(nome, texto)` e devolve o aviso do ecrã. Se não houver onde gravar, o aviso é
/// um erro e o texto vai, inteiro, para o terminal (a última rede).
pub(crate) fn keep(dir: Option<&Path>, unsaved: &[(String, String)]) -> Toast {
    let count = unsaved.len();
    let written = match dir {
        Some(d) => write_all(d, unsaved).map(|()| d.display().to_string()),
        None => Err(std::io::Error::other("no data directory")),
    };
    match written {
        Ok(dir) => {
            let key = if count == 1 {
                "code.recovery.kept_one"
            } else {
                "code.recovery.kept"
            };
            Toast::warning(tr_with(key, &[("count", &count), ("dir", &dir)]))
        }
        Err(e) => {
            for (name, text) in unsaved {
                eprintln!("[code] texto por gravar de {name} (não se gravou: {e}):\n{text}");
            }
            let err = e.to_string();
            Toast::error(tr_with(
                "code.recovery.failed",
                &[("count", &count), ("err", &err)],
            ))
        }
    }
}
