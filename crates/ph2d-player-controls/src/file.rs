//! O ficheiro do jogador: `<base>/players/<game_id em 16 hex>.txt`, fora do projecto (plano InputMap §4).
//! A shell passa `~/.ph2d` (a pasta das prefs); os testes, uma pasta temporária. Sem pasta, salta-se.

use std::path::{Path, PathBuf};

use ph2d_input::PlayerKeys;

/// O ficheiro das teclas do jogador para este jogo.
#[must_use]
pub fn player_file(base: &Path, game_id: u64) -> PathBuf {
    base.join("players").join(format!("{game_id:016x}.txt"))
}

/// O ficheiro deste jogo, se ele tem um. `game_id` 0 é o projecto gravado sem mundo: partilharia o
/// `0000000000000000.txt` entre projectos, por isso não tem ficheiro — a tela funciona em memória.
#[must_use]
pub fn player_file_of(base: Option<&Path>, game_id: u64) -> Option<PathBuf> {
    base.filter(|_| game_id != 0)
        .map(|b| player_file(b, game_id))
}

/// `~/.ph2d`, ou `None` com `$HOME` por definir (como as prefs).
#[must_use]
pub fn home_base() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".ph2d"))
}

/// As teclas gravadas; ficheiro ausente ou ilegível = nada trocado.
#[must_use]
pub fn load(path: &Path) -> PlayerKeys {
    std::fs::read_to_string(path).map_or_else(|_| PlayerKeys::new(), |t| PlayerKeys::from_text(&t))
}

/// Grava as teclas (cria a pasta). Nada trocado apaga o ficheiro.
///
/// # Errors
/// O erro do sistema de ficheiros.
pub fn save(path: &Path, keys: &PlayerKeys) -> std::io::Result<()> {
    if keys.is_empty() {
        return match std::fs::remove_file(path) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        };
    }
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(path, keys.to_text())
}
