//! Atomic write protocol — `write-temp + fsync + rename + dir-fsync` per
//! ADR-0052 §2.4. Essencial em storage cheia / power-off / suspend mobile:
//! o file destino NUNCA fica num estado parcial.
//!
//! Padrão POSIX clássico (PostgreSQL, SQLite, git):
//!
//! 1. Pre-write check: espaço disponível ≥ 2× tamanho do payload (anti-disk-full).
//! 2. `File::create(tmp)` em mesmo diretório (rename é atomic só dentro do FS).
//! 3. `write_all(content)`.
//! 4. `f.sync_all()` — força flush para storage device (não só page cache).
//! 5. `fs::rename(tmp, path)` — atomic per POSIX.
//! 6. `dir.sync_all()` — POSIX exige isso pra rename persistir cross-crash.
//!
//! ## Audit T-durability fixes aplicados
//!
//! - **J-9**: dir-fsync em `#[cfg(unix)]` propaga erro (não swallows); Windows
//!   mantém swallow (NTFS journal absorve).
//! - **J-10**: `has_enough_storage` usa `fs4::available_space` real (era stub).
//! - **K-5**: TODAS as failure paths fazem `cleanup_tmp` (era só `RenameFailed`).

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

/// Margem de segurança para storage check: o file destino deve caber 2×
/// pra acomodar tmp + final simultâneos. ADR-0052 §2.4.
pub const STORAGE_SAFETY_MULTIPLIER: u64 = 2;

/// Escreve `content` em `path` atomicamente. Garante que `path` NUNCA fica
/// num estado parcial — ou tem o conteúdo antigo, ou tem o conteúdo novo
/// completo. Mesmo sob crash / power-off / disk-full mid-write.
///
/// **Ordem das operações** (ADR-0052 §2.4):
///
/// 1. Storage check (≥ 2× `content.len()` livre).
/// 2. Escreve `<path>.tmp` no mesmo diretório.
/// 3. `fsync` do `<path>.tmp`.
/// 4. `rename(<path>.tmp, <path>)` — atomic per POSIX.
/// 5. `fsync` do diretório pai (POSIX exige pra rename persistir).
///
/// Retorna `AtomicWriteError` em falha; o file destino é deixado intacto
/// (estado anterior preservado). **Em qualquer Err, o `<path>.tmp` é
/// limpo** (audit K-5).
pub fn atomic_write(path: &Path, content: &[u8]) -> Result<(), AtomicWriteError> {
    // 1. Storage check (anti-disk-full UX).
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .ok_or(AtomicWriteError::InvalidPath)?;

    let needed = (content.len() as u64).saturating_mul(STORAGE_SAFETY_MULTIPLIER);
    if !has_enough_storage(parent, needed) {
        return Err(AtomicWriteError::InsufficientStorage {
            needed_bytes: needed,
        });
    }

    // 2. Compute tmp path (same dir; same FS guarantee for rename).
    let tmp = tmp_path(path);

    // 3. Write + fsync. Audit K-5: cleanup em qualquer Err.
    let write_result = {
        let mut f = match File::create(&tmp) {
            Ok(f) => f,
            Err(e) => return Err(AtomicWriteError::CreateFailed(e.kind())),
        };
        if let Err(e) = f.write_all(content) {
            cleanup_tmp(&tmp);
            return Err(AtomicWriteError::WriteFailed(e.kind()));
        }
        if let Err(e) = f.sync_all() {
            cleanup_tmp(&tmp);
            return Err(AtomicWriteError::FsyncFailed(e.kind()));
        }
        Ok(())
    };
    write_result?;

    // 4. Atomic rename.
    if let Err(e) = fs::rename(&tmp, path) {
        cleanup_tmp(&tmp);
        return Err(AtomicWriteError::RenameFailed(e.kind()));
    }

    // 5. Dir fsync — POSIX exige pra rename ser persisted cross-crash.
    // Audit J-9: em unix, falha = AtomicWriteError::FsyncFailed propagated;
    // em windows, no-op (NTFS journal absorve).
    #[cfg(unix)]
    {
        let dir = File::open(parent).map_err(|e| AtomicWriteError::FsyncFailed(e.kind()))?;
        dir.sync_all()
            .map_err(|e| AtomicWriteError::FsyncFailed(e.kind()))?;
    }
    #[cfg(not(unix))]
    {
        // Windows: dir fsync não é meaningful; NTFS journal cuida do rename.
        if let Ok(dir) = File::open(parent) {
            let _ = dir.sync_all();
        }
    }

    Ok(())
}

/// Helper interno: tenta remover o tmp file órfão. Ignora erro (best-effort).
fn cleanup_tmp(tmp: &Path) {
    let _ = fs::remove_file(tmp);
}

/// Calcula `<path>.tmp` reservando o mesmo diretório. ADR-0052 §2.4:
/// `path.with_extension("ph2d-painter.tmp")` (sufixo customizado pra
/// `.ph2d-painter`). Esta versão é generalizada: adiciona `.tmp` como
/// extensão final, preservando extensão original como parte do stem.
fn tmp_path(path: &Path) -> std::path::PathBuf {
    let mut p = path.to_path_buf();
    // E.g., `/a/b/canvas.ph2d-painter` → `/a/b/canvas.ph2d-painter.tmp`.
    let new_ext = match path.extension() {
        Some(ext) => {
            let mut s = ext.to_os_string();
            s.push(".tmp");
            s
        }
        None => std::ffi::OsString::from("tmp"),
    };
    p.set_extension(new_ext);
    p
}

/// `true` se o filesystem em `dir` tem `≥ needed_bytes` livres. Audit J-10:
/// usa `fs4::available_space` real (era stub `true` always).
///
/// Em platforms que `fs4` não suporta, retorna `true` (otimismo seguro —
/// `AtomicWriter` ainda preserva safety via write-temp + rename).
fn has_enough_storage(dir: &Path, needed_bytes: u64) -> bool {
    match fs4::available_space(dir) {
        Ok(available) => available >= needed_bytes,
        // fs4 falhou (FS sem suporte, path inválido, etc.) — otimismo:
        // write subsequente vai falhar com ENOSPC concreto se realmente cheio.
        Err(_) => true,
    }
}

/// Erros de [`atomic_write`].
///
/// `#[non_exhaustive]` permite variants futuros (Permissions, ReadonlyFs,
/// QuotaExceeded) sem semver break — pattern T1.8 audit L4-H10.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AtomicWriteError {
    /// Path inválido (e.g., sem parent directory).
    InvalidPath,
    /// Storage check falhou: espaço disponível < `needed_bytes`.
    /// Mensagem UX: "Espaço de armazenamento insuficiente para auto-save.
    /// Libere espaço ou salve manualmente em outro local."
    InsufficientStorage { needed_bytes: u64 },
    /// `File::create(<tmp>)` falhou.
    CreateFailed(io::ErrorKind),
    /// `write_all` falhou.
    WriteFailed(io::ErrorKind),
    /// `f.sync_all()` falhou — pode ser tmp file fsync OU dir fsync (unix).
    /// Sem fsync, conteúdo do tmp file pode estar só em page cache;
    /// crash pós-rename revela file zerado.
    FsyncFailed(io::ErrorKind),
    /// `fs::rename(<tmp>, <path>)` falhou. tmp limpo automaticamente.
    RenameFailed(io::ErrorKind),
}

impl std::fmt::Display for AtomicWriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPath => write!(f, "atomic_write: path has no parent directory"),
            Self::InsufficientStorage { needed_bytes } => write!(
                f,
                "atomic_write: insufficient storage (need ≥ {needed_bytes} bytes free)"
            ),
            Self::CreateFailed(k) => write!(f, "atomic_write: create tmp failed: {k:?}"),
            Self::WriteFailed(k) => write!(f, "atomic_write: write tmp failed: {k:?}"),
            Self::FsyncFailed(k) => write!(f, "atomic_write: fsync failed: {k:?}"),
            Self::RenameFailed(k) => write!(f, "atomic_write: rename tmp→path failed: {k:?}"),
        }
    }
}

impl std::error::Error for AtomicWriteError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tmp_path_appends_tmp_extension() {
        let p = std::path::Path::new("/tmp/canvas.ph2d-painter");
        let t = tmp_path(p);
        assert_eq!(t.to_string_lossy(), "/tmp/canvas.ph2d-painter.tmp");
    }

    #[test]
    fn tmp_path_handles_no_extension() {
        let p = std::path::Path::new("/tmp/canvas");
        let t = tmp_path(p);
        assert_eq!(t.to_string_lossy(), "/tmp/canvas.tmp");
    }

    #[test]
    fn atomic_write_creates_target_file() {
        let dir =
            std::env::temp_dir().join(format!("ph2d-painter-stroke-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("hello.bin");
        let content = b"hello atomic world";
        atomic_write(&path, content).expect("atomic_write should succeed");
        let read = std::fs::read(&path).unwrap();
        assert_eq!(read, content);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn atomic_write_overwrites_existing_atomically() {
        let dir = std::env::temp_dir().join(format!(
            "ph2d-painter-stroke-overwrite-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("data.bin");
        std::fs::write(&path, b"OLD CONTENT").unwrap();
        atomic_write(&path, b"NEW CONTENT").expect("ok");
        let read = std::fs::read(&path).unwrap();
        assert_eq!(read, b"NEW CONTENT");
        // Tmp file não deve permanecer.
        let tmp = tmp_path(&path);
        assert!(!tmp.exists(), "tmp file deve ser renamed (não permanecer)");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn atomic_write_rejects_path_without_parent() {
        // Path relativo sem componentes válidos.
        let err = atomic_write(std::path::Path::new(""), b"x").unwrap_err();
        assert_eq!(err, AtomicWriteError::InvalidPath);
    }

    /// J-10: storage check real (não stub).
    #[test]
    fn has_enough_storage_returns_true_for_modest_request() {
        // Pedir 1 KB em /tmp — deve passar em qualquer dev machine.
        assert!(has_enough_storage(&std::env::temp_dir(), 1024));
    }
}
