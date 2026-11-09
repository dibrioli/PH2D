//! A sessão: uma pasta `<raiz>/<início>-<pid>/` com a tranca `session.lock` (o `pid`), a cópia
//! `project.ph2dproj` (os bytes do Save, nada mais) e o `project.meta` (`name=`, `path=`, `saved=`).
//!
//! Uma sessão está **viva** enquanto o processo dela segura a tranca do SO (`File::try_lock`) em
//! `session.lock` ([`is_held`]): o SO solta-a quando o processo morre, então não há `pid` reciclado
//! que esconda uma morta nem outro SO onde uma viva pareça morta. O fecho limpo apaga a pasta; o
//! fecho sem decisão do artista ([`Session::leave`]) e o fecho à força deixam-na para o arranque.

use std::path::{Path, PathBuf};

use crate::atomic::write_atomic;

const LOCK: &str = "session.lock";
const COPY: &str = "project.ph2dproj";
const META: &str = "project.meta";

/// A sessão deste processo.
#[derive(Debug)]
pub struct Session {
    dir: PathBuf,
    /// A tranca do SO, segura enquanto a sessão vive (solta-se ao cair o `File`).
    _lock: std::fs::File,
}

impl Session {
    /// Abre a sessão `<raiz>/<início>-<pid>` e escreve a tranca.
    ///
    /// # Errors
    /// O erro do sistema de ficheiros.
    pub fn start(root: &Path, started: u64, pid: u32) -> std::io::Result<Self> {
        let dir = root.join(format!("{started}-{pid}"));
        std::fs::create_dir_all(&dir)?;
        let mut lock = std::fs::File::create(dir.join(LOCK))?;
        lock.try_lock().map_err(std::io::Error::other)?;
        std::io::Write::write_all(&mut lock, format!("pid={pid}\n").as_bytes())?;
        Ok(Self { dir, _lock: lock })
    }

    /// A pasta da sessão.
    #[must_use]
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// O ficheiro da cópia desta sessão.
    #[must_use]
    pub fn copy_path(&self) -> PathBuf {
        self.dir.join(COPY)
    }

    /// O nome da pasta (o id da sessão).
    #[must_use]
    pub fn id(&self) -> String {
        self.dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    /// Grava a cópia (os bytes do Save) e o meta, ambos atómicos. O meta vai DEPOIS: um meta velho
    /// sobre uma cópia nova só erra na hora, nunca nos bytes.
    ///
    /// # Errors
    /// O erro do sistema de ficheiros.
    pub fn write_copy(&self, bytes: &[u8], meta: &Meta) -> std::io::Result<()> {
        write_copy_at(&self.dir, bytes, meta)
    }

    /// Apaga a cópia desta sessão (o Save e o Open: o disco passa a ser a verdade).
    pub fn clear_copy(&self) {
        let _ = std::fs::remove_file(self.dir.join(COPY));
        let _ = std::fs::remove_file(self.dir.join(META));
    }

    /// O fecho limpo (o artista decidiu, ou não havia nada por gravar): apaga a pasta inteira.
    pub fn close(self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }

    /// O fecho sem decisão do artista: solta a tranca e DEIXA a cópia (o próximo arranque oferece-a).
    pub fn leave(self) {}

    /// `Restore` (só DEPOIS de o projecto abrir): a cópia órfã passa a ser desta sessão, com a hora
    /// de agora (o próximo intervalo grava por cima dela, e ela já não é oferecida de novo), e a
    /// pasta morta sai.
    ///
    /// # Errors
    /// O erro do sistema de ficheiros (a órfã fica onde estava).
    pub fn adopt(&self, orphan: &Orphan, now: u64) -> std::io::Result<Vec<u8>> {
        let bytes = std::fs::read(&orphan.copy)?;
        let meta = Meta {
            saved: now,
            ..orphan.meta.clone()
        };
        self.write_copy(&bytes, &meta)?;
        let _ = std::fs::remove_dir_all(&orphan.dir);
        Ok(bytes)
    }
}

/// O que se sabe da cópia além dos bytes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Meta {
    /// O nome do projecto (o ficheiro sem pasta, ou `Untitled`).
    pub name: String,
    /// O caminho original (vazio = nunca gravado): o Save depois do `Restore` grava ali.
    pub path: String,
    /// Quando a cópia foi gravada (segundos desde 1970).
    pub saved: u64,
}

impl Meta {
    fn to_text(&self) -> String {
        let clean = |s: &str| s.replace(['\n', '\r'], " ");
        format!(
            "name={}\npath={}\nsaved={}\n",
            clean(&self.name),
            clean(&self.path),
            self.saved
        )
    }

    fn parse(text: &str) -> Self {
        let mut m = Self::default();
        for line in text.lines() {
            match line.split_once('=') {
                Some(("name", v)) => v.clone_into(&mut m.name),
                Some(("path", v)) => v.clone_into(&mut m.path),
                Some(("saved", v)) => m.saved = v.trim().parse().unwrap_or(0),
                _ => {}
            }
        }
        m
    }
}

pub(crate) fn write_copy_at(dir: &Path, bytes: &[u8], meta: &Meta) -> std::io::Result<()> {
    write_atomic(&dir.join(COPY), bytes)?;
    write_atomic(&dir.join(META), meta.to_text().as_bytes())
}

/// Uma cópia de uma sessão morta, à espera de `Restore` / `Discard` / `Later`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Orphan {
    /// A pasta da sessão morta.
    pub dir: PathBuf,
    /// O ficheiro da cópia.
    pub copy: PathBuf,
    /// Nome, caminho original e hora.
    pub meta: Meta,
    /// O tamanho da cópia em bytes.
    pub size: u64,
}

impl Orphan {
    /// `Discard`: a pasta morta sai.
    pub fn discard(&self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// A sessão da pasta `dir` vive? = alguém segura a tranca dela. Sem tranca (pasta antiga ou meia
/// criada) = morta; erro que não seja «ocupada» = viva (na dúvida nada se apaga).
#[must_use]
pub fn is_held(dir: &Path) -> bool {
    let Ok(f) = std::fs::File::open(dir.join(LOCK)) else {
        return false;
    };
    match f.try_lock() {
        Ok(()) => false,
        Err(std::fs::TryLockError::WouldBlock) => true,
        Err(std::fs::TryLockError::Error(_)) => true,
    }
}

/// A hora da cópia: o `saved=` do meta, ou (meta ausente/ilegível) o `mtime` da cópia; `None` = não
/// se sabe (e então ela nunca se poda).
fn copy_time(meta: &Meta, copy: &Path) -> Option<u64> {
    if meta.saved > 0 {
        return Some(meta.saved);
    }
    let m = std::fs::metadata(copy).and_then(|m| m.modified()).ok()?;
    m.duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs())
}

/// A varredura ao arrancar: poda as sessões mortas com mais de [`crate::KEEP_SECS`] (e as mortas
/// sem cópia, e os `.tmp` de escritas mortas) e devolve as cópias que se oferecem, a mais nova
/// primeiro. `except` é a pasta da sessão viva deste processo; as vivas ([`is_held`]) não se tocam.
#[must_use]
pub fn scan(root: &Path, now: u64, except: Option<&Path>) -> Vec<Orphan> {
    let Ok(entries) = std::fs::read_dir(root) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for e in entries.flatten() {
        let dir = e.path();
        if !dir.is_dir() || except.is_some_and(|x| x == dir) {
            continue;
        }
        if is_held(&dir) {
            continue;
        }
        let copy = dir.join(COPY);
        let mut meta = std::fs::read_to_string(dir.join(META))
            .map_or_else(|_| Meta::default(), |t| Meta::parse(&t));
        let size = std::fs::metadata(&copy).map_or(0, |m| m.len());
        let time = copy_time(&meta, &copy);
        meta.saved = time.unwrap_or(0);
        let old = time.is_some_and(|t| now.saturating_sub(t) > crate::KEEP_SECS);
        if !copy.is_file() || old {
            let _ = std::fs::remove_dir_all(&dir);
            continue;
        }
        let _ = std::fs::remove_file(crate::atomic::tmp_of(&copy));
        let _ = std::fs::remove_file(crate::atomic::tmp_of(&dir.join(META)));
        out.push(Orphan {
            dir,
            copy,
            meta,
            size,
        });
    }
    out.sort_by_key(|s| std::cmp::Reverse(s.meta.saved));
    out
}
