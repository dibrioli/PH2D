//! I8: o registo local das explicações que um balão abriu (uma linha `<segundos>\t<classe>`, sem
//! rede), para reordenar o trabalho nas explicações depois.

use std::io::Write as _;
use std::path::PathBuf;

/// O ficheiro do registo por omissão: `PH2D_CODE_SEEN_FILE` · `$XDG_DATA_HOME/ph2d/…` ·
/// `$HOME/.local/share/ph2d/…` · `%APPDATA%\ph2d\…`.
fn default_file() -> Option<PathBuf> {
    const NAME: &str = "code-explanations-seen.tsv";
    let var = |k: &str| {
        std::env::var_os(k)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    };
    var("PH2D_CODE_SEEN_FILE")
        .or_else(|| var("XDG_DATA_HOME").map(|d| d.join("ph2d").join(NAME)))
        .or_else(|| var("HOME").map(|h| h.join(".local/share/ph2d").join(NAME)))
        .or_else(|| var("APPDATA").map(|d| d.join("ph2d").join(NAME)))
}

/// As explicações vistas nesta corrida, e onde se gravam.
#[derive(Debug)]
pub struct Seen {
    /// `None` = só na memória (os testes).
    pub file: Option<PathBuf>,
    pub kinds: Vec<&'static str>,
    /// A primeira escrita que falhou já foi dita no terminal.
    pub told: bool,
}

impl Default for Seen {
    fn default() -> Self {
        Self {
            file: default_file(),
            kinds: Vec::new(),
            told: false,
        }
    }
}

impl Seen {
    pub fn record(&mut self, kinds: &[&'static str]) {
        self.kinds.extend_from_slice(kinds);
        let Some(file) = self.file.as_ref() else {
            return;
        };
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        let lines: String = kinds.iter().map(|k| format!("{secs}\t{k}\n")).collect();
        let wrote = file
            .parent()
            .map_or(Ok(()), std::fs::create_dir_all)
            .and_then(|()| {
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(file)?
                    .write_all(lines.as_bytes())
            });
        if let Err(e) = wrote
            && !self.told
        {
            self.told = true;
            eprintln!("code explanations: could not write {}: {e}", file.display());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn o_registo_acrescenta_uma_linha_por_explicacao_e_cria_a_pasta() {
        let dir = std::env::temp_dir().join(format!("ph2d-seen-{}", std::process::id()));
        let file = dir.join("sub").join("seen.tsv");
        let mut s = Seen {
            file: Some(file.clone()),
            kinds: Vec::new(),
            told: false,
        };
        s.record(&["unknown_global"]);
        s.record(&["unclosed", "same_line"]);
        let text = std::fs::read_to_string(&file).expect("gravado");
        let kinds: Vec<&str> = text
            .lines()
            .map(|l| l.split('\t').nth(1).expect("<segundos>\t<classe>"))
            .collect();
        assert_eq!(kinds, ["unknown_global", "unclosed", "same_line"]);
        assert_eq!(s.kinds, kinds);
        let _ = std::fs::remove_dir_all(dir);
    }
}
