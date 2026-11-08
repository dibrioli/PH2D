//! A escrita atómica: quem reabre vê a cópia anterior inteira ou a nova inteira, nunca meia.
//!
//! Temporário `<nome>.tmp` → todos os bytes → `fsync` do ficheiro → `rename` sobre o nome final →
//! `fsync` da pasta (sem ele o `rename` pode não sobreviver a um corte de luz). O leitor só abre o
//! nome final; um `.tmp` que ficou é lixo de uma escrita morta e a varredura apaga-o.

use std::io::Write;
use std::path::{Path, PathBuf};

/// Onde a escrita «morre» — só os testes da propriedade o usam.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Escreve até ao fim.
    None,
    /// Morre depois de `k` bytes do temporário (o processo cai a meio do `write`).
    StopAfter(usize),
    /// Escreveu e fez `fsync` do temporário, morre antes do `rename`.
    BeforeRename,
}

/// O temporário de `path`.
#[must_use]
pub fn tmp_of(path: &Path) -> PathBuf {
    let mut s = path.as_os_str().to_owned();
    s.push(".tmp");
    PathBuf::from(s)
}

/// Grava `bytes` em `path` de forma atómica.
///
/// # Errors
/// O erro do sistema de ficheiros (o `path` anterior fica como estava).
pub fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    write_atomic_with(path, bytes, Fault::None)
}

/// [`write_atomic`] com um ponto de morte (`Fault`) — devolve erro ao «morrer», deixando no disco
/// exactamente o que um processo morto ali deixaria.
///
/// # Errors
/// O erro do sistema de ficheiros, ou o da morte simulada.
pub fn write_atomic_with(path: &Path, bytes: &[u8], fault: Fault) -> std::io::Result<()> {
    let tmp = tmp_of(path);
    let mut f = std::fs::File::create(&tmp)?;
    if let Fault::StopAfter(k) = fault {
        f.write_all(&bytes[..k.min(bytes.len())])?;
        return Err(std::io::Error::other("killed mid-write"));
    }
    f.write_all(bytes)?;
    f.sync_all()?;
    drop(f);
    if fault == Fault::BeforeRename {
        return Err(std::io::Error::other("killed before rename"));
    }
    std::fs::rename(&tmp, path)?;
    if let Some(dir) = path.parent() {
        // A pasta não abre como ficheiro em todo o SO (Windows); lá o `rename` já é durável.
        if let Ok(d) = std::fs::File::open(dir) {
            let _ = d.sync_all();
        }
    }
    Ok(())
}
