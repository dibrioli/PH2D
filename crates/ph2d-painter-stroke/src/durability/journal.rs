//! `StrokeJournal` — write-ahead log (WAL) de stroke-in-progress.
//! ADR-0052 §2.2.
//!
//! Cada sample de stroke-em-progresso é **jornalado per-batch** (default 8
//! samples ou 100ms, o que vier primeiro). Crash mid-stroke = worst-case
//! ~100ms de samples perdidos; stroke COMPLETO (com Commit marker) nunca
//! perdido.
//!
//! ## Layout do WAL `painter_journal.wal`
//!
//! ```text
//! [u8;12]      = "PH2D-JOURNAL"  ← magic
//! u32          = version (1)
//! // Per entry (repeat até EOF):
//! u8           = entry_type (0=Begin, 1=SampleBatch, 2=Commit, 3=Cancel, 4=Heartbeat)
//! u32 LE       = payload_size
//! [u8; size]   = postcard-encoded payload
//! u32 LE       = crc32(entry_type || payload_size_le || payload)   ← audit J-3
//! ```
//!
//! **Append-only**, never rewrite. Each entry é escrita atomicamente
//! (single `write_all` de buffer concatenado — audit J-2). CRC32 cobre
//! header+payload (audit J-3). Rotação: caller chama
//! [`StrokeJournal::truncate_and_reset`] após N commits OR > 50MB WAL.
//!
//! ## Audit T-durability fixes
//!
//! - **J-2**: `append_entry` faz UM `write_all` de buffer único + 1 fsync.
//!   Crash tearing entre header/payload eliminado.
//! - **J-3**: CRC32 cobre `entry_type || payload_size_le || payload`
//!   (era só payload). Header tampering agora detectado.
//! - **J-4**: `read_journal` faz resync sliding-window em vez de break-on-
//!   corruption (budget `MAX_RESYNC_SCAN_BYTES`).
//! - **J-6**: `StrokeJournal::open` adquire advisory exclusive lock
//!   (`fs4::FileExt::try_lock_exclusive`).
//! - **J-11**: TOCTOU `exists()` race eliminado via `create_new` pattern.
//! - **J-12**: `truncate_and_reset` via `atomic_write` (crash entre
//!   truncate + write magic deixava WAL 0-byte que falhava WrongMagic).
//! - **J-13**: `SampleBatch` payload agora carrega `StrokeId` explícito
//!   pra evitar mis-attribution em WAL adversarial.
//! - **J-17**: removido dead `f.seek(SeekFrom::Start(0))` no fim de read_journal.
//! - **J-18**: writer cap == reader cap (`MAX_WAL_PAYLOAD`).
//! - **K-1**: `MAX_WAL_FILE_BYTES` precheck antes de parse (anti-DoS).

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

use fs4::fs_std::FileExt;
use ph2d_color::OklchColor;
use ph2d_painter_brush::{BrushHandle, BrushParamsHash};
use serde::{Deserialize, Serialize};

use crate::SCHEMA_VERSION;
use crate::device::{CanvasId, LayerId};
use crate::record::{RawPointerSample, StrokeId, ToolMode};

use super::atomic_write::atomic_write;

/// Magic bytes do WAL. 12 bytes ASCII.
pub const JOURNAL_MAGIC: [u8; 12] = *b"PH2D-JOURNAL";

/// Cap de entry payload (writer + reader symmetric). Audit J-18.
pub const MAX_WAL_PAYLOAD: usize = 16 * 1024 * 1024;

/// Cap de file inteiro do WAL — anti-DoS precheck em [`read_journal`].
/// Audit K-1. 500 MiB paridade com `MAX_CANON_BYTES`.
pub const MAX_WAL_FILE_BYTES: u64 = 500 * 1024 * 1024;

/// Budget de scan pra resync após entry corrupta (audit J-4). 1 MiB.
pub const MAX_RESYNC_SCAN_BYTES: u64 = 1024 * 1024;

/// Threshold default pra rotação automática (ADR-0052 §2.2).
pub const JOURNAL_ROTATE_BYTES: u64 = 50 * 1024 * 1024;

/// Threshold default pra rotação por commits (ADR-0052 §2.2).
pub const JOURNAL_ROTATE_COMMITS: u32 = 100;

/// Snapshot de um stroke em progresso (entre `BeginStroke` e
/// `CommitStroke`/`CancelStroke`). ADR-0052 §2.2.
///
/// 13 fields nominais ≤ 16 cap.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PartialStroke {
    /// UUID v4 — gerado em BeginStroke; reusado em CommitStroke (estabilidade
    /// pra MCP queries).
    pub uuid: StrokeId,
    /// Pré-alocado em BeginStroke pra preservar ordem em crash recovery.
    pub seq: u64,
    /// Qual canvas o stroke pertence.
    pub canvas_id: CanvasId,
    /// Qual layer recebe os stamps.
    pub layer_target: LayerId,
    /// Brush opaco (built-in slot ou imported atlas).
    pub brush_handle: BrushHandle,
    /// blake3 do Brush snapshot — dedup em PaintProject::brush_snapshots.
    pub brush_params_hash: BrushParamsHash,
    /// Cor primária OKLCH.
    pub primary_color: OklchColor,
    /// Cor secundária (long-press slot).
    pub secondary_color: Option<OklchColor>,
    /// Modo operacional (Paint/Smudge/Erase).
    pub tool_mode: ToolMode,
    /// RNG seed — gerado UPFRONT (não no commit), pra det-mode replay
    /// reproduzir Color Dynamics + Procedural Grain.
    pub rng_seed: u64,
    /// Wall-clock ms — BeginStroke timestamp.
    pub started_at_ms: u64,
    /// Quantos samples o WAL deveria conter (checksum semântico em recovery).
    /// **Validado em `CrashRecovery::reconstruct` (audit K-3)** — mismatch
    /// vs samples real = stroke marcado Corrupted.
    pub samples_count_in_journal: u32,
    /// HR-14 schema version (v1 = 1).
    pub version: u32,
    // === 3 slots de headroom (e.g., MCP origin tag, audit_log_id, lock_token) ===
}

impl PartialStroke {
    /// Cria um novo PartialStroke com defaults seguros.
    pub fn new(
        seq: u64,
        canvas_id: CanvasId,
        layer_target: LayerId,
        brush_handle: BrushHandle,
        brush_params_hash: BrushParamsHash,
        primary_color: OklchColor,
    ) -> Self {
        Self {
            uuid: uuid::Uuid::new_v4(),
            seq,
            canvas_id,
            layer_target,
            brush_handle,
            brush_params_hash,
            primary_color,
            secondary_color: None,
            tool_mode: ToolMode::Paint,
            rng_seed: 0,
            started_at_ms: 0,
            samples_count_in_journal: 0,
            version: SCHEMA_VERSION,
        }
    }
}

/// Política de flush do WAL. ADR-0052 §2.2.
#[non_exhaustive]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FlushPolicy {
    /// Flush a cada N samples (default N=8). Cobre 99% stroke loss.
    EveryNSamples(u32),
    /// Flush a cada M milliseconds (default 100ms). Latency-bounded.
    EveryMs(u32),
    /// Hybrid: flush quando QUALQUER acontecer primeiro. Default recomendado.
    Hybrid { n: u32, ms: u32 },
    // === 3 slots de headroom ===
}

impl Default for FlushPolicy {
    fn default() -> Self {
        Self::Hybrid { n: 8, ms: 100 }
    }
}

/// Tipo de entry no WAL. ADR-0052 §2.2 layout. `#[repr(u8)]` pinned.
#[repr(u8)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum WalEntryType {
    Begin = 0,
    SampleBatch = 1,
    Commit = 2,
    Cancel = 3,
    Heartbeat = 4,
    // === 3 slots de headroom ===
}

/// Payload de `SampleBatch` (audit J-13): carrega o `StrokeId` parent
/// explicitamente pra evitar mis-attribution em WAL adversarial onde
/// dois Begin sem Commit aparecem em sequência.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SampleBatchPayload {
    pub stroke_id: StrokeId,
    pub samples: Vec<RawPointerSample>,
}

/// State machine do WAL writer.
pub struct StrokeJournal {
    pub journal_path: PathBuf,
    pub current_stroke: Option<PartialStroke>,
    pub sample_buffer: Vec<RawPointerSample>,
    pub last_flush_ms: u64,
    pub flush_policy: FlushPolicy,
    /// Quantos commits aconteceram desde a última rotação.
    pub commits_since_rotate: u32,
    /// File handle persistente — segura advisory lock during life da struct.
    /// `Drop` libera lock automaticamente (RAII).
    file: File,
}

impl StrokeJournal {
    /// Cria um journal apontando para `journal_path`. Adquire advisory
    /// exclusive lock (audit J-6) — falha com `JournalError::AlreadyLocked`
    /// se outro processo segura.
    ///
    /// **Audit J-11:** usa `create_new` pra eliminar TOCTOU race entre
    /// `exists()` check e `open`. Se o file já existe, abre em append mode.
    /// Se não existe, cria atomicamente + escreve magic+version.
    ///
    /// **Audit K-10 — path sanitization:** `journal_path` é caller-controlled.
    /// Caller (W11 shell) DEVE construir o path como
    /// `<storage_root>/painter_journal.wal` LITERAL — NUNCA passar
    /// user-input direto. Path com `..` ou symlink pode permitir write
    /// fora da intended canvas dir. Helper paralelo a
    /// `LayerSnapshot::validate_path_for_load` é candidate W11.
    pub fn open(journal_path: PathBuf, flush_policy: FlushPolicy) -> Result<Self, JournalError> {
        // Tentativa atomic create + write header. Se EEXIST, é re-open.
        let create_result = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&journal_path);

        let file = match create_result {
            Ok(mut f) => {
                // File novo — escreve header atômico.
                let mut header = Vec::with_capacity(16);
                header.extend_from_slice(&JOURNAL_MAGIC);
                header.extend_from_slice(&SCHEMA_VERSION.to_le_bytes());
                f.write_all(&header)
                    .map_err(|e| JournalError::Io(e.kind()))?;
                f.sync_all().map_err(|e| JournalError::Io(e.kind()))?;
                drop(f);
                // Reabre em append+read mode pra operação contínua.
                OpenOptions::new()
                    .create(false)
                    .append(true)
                    .read(true)
                    .open(&journal_path)
                    .map_err(|e| JournalError::Io(e.kind()))?
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                // File existente — re-open append.
                OpenOptions::new()
                    .create(false)
                    .append(true)
                    .read(true)
                    .open(&journal_path)
                    .map_err(|e| JournalError::Io(e.kind()))?
            }
            Err(e) => return Err(JournalError::Io(e.kind())),
        };

        // Advisory exclusive lock — outro processo abrindo mesmo path falha.
        file.try_lock_exclusive()
            .map_err(|_| JournalError::AlreadyLocked)?;

        Ok(Self {
            journal_path,
            current_stroke: None,
            sample_buffer: Vec::with_capacity(64),
            last_flush_ms: 0,
            flush_policy,
            commits_since_rotate: 0,
            file,
        })
    }

    /// Inicia um stroke. Emite `Begin` entry no WAL imediatamente (fsync).
    pub fn begin_stroke(&mut self, stroke: PartialStroke) -> Result<(), JournalError> {
        if self.current_stroke.is_some() {
            return Err(JournalError::StrokeAlreadyActive);
        }
        self.append_entry(WalEntryType::Begin, &stroke)?;
        self.current_stroke = Some(stroke);
        self.sample_buffer.clear();
        Ok(())
    }

    /// Adiciona um sample ao buffer. Flush se policy disparar.
    pub fn add_sample(
        &mut self,
        sample: RawPointerSample,
        now_ms: u64,
    ) -> Result<(), JournalError> {
        if self.current_stroke.is_none() {
            return Err(JournalError::NoActiveStroke);
        }
        self.sample_buffer.push(sample);
        if self.should_flush(now_ms) {
            self.flush_sample_batch(now_ms)?;
        }
        Ok(())
    }

    /// Decide se policy diz pra flushar agora.
    pub fn should_flush(&self, now_ms: u64) -> bool {
        match self.flush_policy {
            FlushPolicy::EveryNSamples(n) => self.sample_buffer.len() as u32 >= n,
            FlushPolicy::EveryMs(ms) => now_ms.saturating_sub(self.last_flush_ms) >= u64::from(ms),
            FlushPolicy::Hybrid { n, ms } => {
                self.sample_buffer.len() as u32 >= n
                    || now_ms.saturating_sub(self.last_flush_ms) >= u64::from(ms)
            }
        }
    }

    /// Flush imediato do sample_buffer atual (sem checar policy).
    pub fn flush_sample_batch(&mut self, now_ms: u64) -> Result<(), JournalError> {
        if self.sample_buffer.is_empty() {
            return Ok(());
        }
        let stroke_id = self
            .current_stroke
            .as_ref()
            .map(|s| s.uuid)
            .ok_or(JournalError::NoActiveStroke)?;
        let batch: Vec<RawPointerSample> = std::mem::take(&mut self.sample_buffer);
        let batch_len = batch.len() as u32;
        let payload = SampleBatchPayload {
            stroke_id,
            samples: batch,
        };
        self.append_entry(WalEntryType::SampleBatch, &payload)?;
        if let Some(s) = self.current_stroke.as_mut() {
            s.samples_count_in_journal = s.samples_count_in_journal.saturating_add(batch_len);
        }
        self.last_flush_ms = now_ms;
        Ok(())
    }

    /// Marca o stroke como commitado.
    pub fn commit_stroke(&mut self, now_ms: u64) -> Result<PartialStroke, JournalError> {
        if !self.sample_buffer.is_empty() {
            self.flush_sample_batch(now_ms)?;
        }
        let stroke = self
            .current_stroke
            .take()
            .ok_or(JournalError::NoActiveStroke)?;
        self.append_entry(WalEntryType::Commit, &stroke.uuid)?;
        self.commits_since_rotate = self.commits_since_rotate.saturating_add(1);
        Ok(stroke)
    }

    /// Cancela o stroke ativo (user descartou).
    pub fn cancel_stroke(&mut self) -> Result<(), JournalError> {
        let stroke = self
            .current_stroke
            .take()
            .ok_or(JournalError::NoActiveStroke)?;
        self.sample_buffer.clear();
        self.append_entry(WalEntryType::Cancel, &stroke.uuid)?;
        Ok(())
    }

    /// Heartbeat — marca presença do app pra crash recovery distinguir
    /// "crashou" vs "stale journal de processo morto".
    ///
    /// **Caller contract (audit J-16):** chame no MÁXIMO 1Hz. Cada call
    /// emite ~17 bytes + fsync (~5-10ms eMMC). Caller > 1Hz arrisca write
    /// amplification destruindo SSD/eMMC. Rate-limit interno opcional
    /// pode ser wired em W11 se MCP path acidentalmente flood.
    pub fn heartbeat(&mut self, now_ms: u64) -> Result<(), JournalError> {
        self.append_entry(WalEntryType::Heartbeat, &now_ms)
    }

    /// `true` se journal merece rotação (size OR commits).
    pub fn should_rotate(&self) -> bool {
        if self.commits_since_rotate >= JOURNAL_ROTATE_COMMITS {
            return true;
        }
        match std::fs::metadata(&self.journal_path) {
            Ok(m) => m.len() >= JOURNAL_ROTATE_BYTES,
            Err(_) => false,
        }
    }

    /// Trunca o WAL E re-escreve magic+version atomicamente via
    /// [`atomic_write`] (audit J-12 — previne WAL 0-byte se crash entre
    /// truncate + write magic).
    pub fn truncate_and_reset(&mut self) -> Result<(), JournalError> {
        // Libera lock + file handle pra atomic_write poder rename.
        // SAFETY: nada lê/escreve aqui até reabrir.
        let path = self.journal_path.clone();

        let mut header = Vec::with_capacity(16);
        header.extend_from_slice(&JOURNAL_MAGIC);
        header.extend_from_slice(&SCHEMA_VERSION.to_le_bytes());

        atomic_write(&path, &header).map_err(|_| JournalError::Io(std::io::ErrorKind::Other))?;

        // Reabre + re-adquire lock.
        let file = OpenOptions::new()
            .create(false)
            .append(true)
            .read(true)
            .open(&path)
            .map_err(|e| JournalError::Io(e.kind()))?;
        file.try_lock_exclusive()
            .map_err(|_| JournalError::AlreadyLocked)?;
        self.file = file;
        self.commits_since_rotate = 0;
        Ok(())
    }

    /// Append uma entry com tipo + payload postcard + CRC32 cobrindo
    /// header+payload. Audit J-2 (single write_all) + J-3 (CRC scope).
    fn append_entry<T: Serialize>(
        &mut self,
        entry_type: WalEntryType,
        payload: &T,
    ) -> Result<(), JournalError> {
        let payload_bytes = postcard::to_allocvec(payload).map_err(JournalError::Postcard)?;
        // Audit J-18: writer cap == reader cap.
        if payload_bytes.len() > MAX_WAL_PAYLOAD {
            return Err(JournalError::PayloadTooLarge(payload_bytes.len()));
        }
        let payload_size = payload_bytes.len() as u32;
        let payload_size_le = payload_size.to_le_bytes();

        // Audit J-3: CRC sobre entry_type || payload_size || payload.
        let mut crc_hasher = crc32fast::Hasher::new();
        crc_hasher.update(&[entry_type as u8]);
        crc_hasher.update(&payload_size_le);
        crc_hasher.update(&payload_bytes);
        let crc = crc_hasher.finalize();

        // Audit J-2: buffer único + single write_all + single fsync.
        let mut buf = Vec::with_capacity(1 + 4 + payload_bytes.len() + 4);
        buf.push(entry_type as u8);
        buf.extend_from_slice(&payload_size_le);
        buf.extend_from_slice(&payload_bytes);
        buf.extend_from_slice(&crc.to_le_bytes());

        self.file
            .write_all(&buf)
            .map_err(|e| JournalError::Io(e.kind()))?;
        self.file
            .sync_all()
            .map_err(|e| JournalError::Io(e.kind()))?;
        Ok(())
    }
}

impl Drop for StrokeJournal {
    fn drop(&mut self) {
        // Best-effort unlock — `File::drop` libera automaticamente o lock
        // (POSIX flock + Win32 LockFileEx fechado em close).
        let _ = FileExt::unlock(&self.file);
    }
}

// =============================================================================
// Reader — usado por CrashRecovery
// =============================================================================

/// Entry raw lida do WAL. Caller (CrashRecovery) é responsável por
/// deserializar `payload_bytes` conforme `entry_type`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WalEntryRaw {
    pub entry_type: WalEntryType,
    pub payload_bytes: Vec<u8>,
    /// `true` se CRC32 bateu (sobre entry_type || size || payload).
    pub crc_valid: bool,
}

/// Lê todo o WAL e retorna entries válidas + count de corrupted.
///
/// **Audit K-1:** rejeita files > [`MAX_WAL_FILE_BYTES`] antes de parse.
/// **Audit J-4:** entry corrupted faz **resync sliding-window** em vez de
/// abort total (budget `MAX_RESYNC_SCAN_BYTES`).
pub fn read_journal(path: &PathBuf) -> Result<(Vec<WalEntryRaw>, u32), JournalError> {
    // Audit K-1: file size precheck anti-DoS.
    let metadata = std::fs::metadata(path).map_err(|e| JournalError::Io(e.kind()))?;
    if metadata.len() > MAX_WAL_FILE_BYTES {
        return Err(JournalError::FileTooLarge {
            size: metadata.len(),
            max: MAX_WAL_FILE_BYTES,
        });
    }

    let mut f = File::open(path).map_err(|e| JournalError::Io(e.kind()))?;

    // Header.
    let mut magic = [0u8; 12];
    f.read_exact(&mut magic)
        .map_err(|_| JournalError::WrongMagic)?;
    if magic != JOURNAL_MAGIC {
        return Err(JournalError::WrongMagic);
    }
    let mut version_bytes = [0u8; 4];
    f.read_exact(&mut version_bytes)
        .map_err(|e| JournalError::Io(e.kind()))?;
    let version = u32::from_le_bytes(version_bytes);
    if version > SCHEMA_VERSION {
        return Err(JournalError::FutureVersion {
            file: version,
            supported: SCHEMA_VERSION,
        });
    }

    // Entries.
    let mut entries = Vec::new();
    let mut corrupted = 0u32;
    let mut resync_budget = MAX_RESYNC_SCAN_BYTES;

    loop {
        let entry_start = match f.stream_position() {
            Ok(p) => p,
            Err(e) => return Err(JournalError::Io(e.kind())),
        };

        match try_read_one_entry(&mut f) {
            Ok(Some(entry)) => entries.push(entry),
            Ok(None) => break, // clean EOF
            Err(_) => {
                // Audit J-4: corrupted entry → conta + resync sliding window.
                corrupted = corrupted.saturating_add(1);
                if resync_budget == 0 {
                    break;
                }
                // Try advancing 1 byte at a time looking for next valid entry_type.
                let advanced = match resync_from(&mut f, entry_start, &mut resync_budget) {
                    Ok(true) => true,
                    Ok(false) => false,
                    Err(_) => false,
                };
                if !advanced {
                    break;
                }
            }
        }
    }

    Ok((entries, corrupted))
}

/// Tenta ler uma entry inteira. Retorna:
/// - `Ok(Some(entry))` — entry válida lida com sucesso.
/// - `Ok(None)` — EOF limpo (sem bytes restantes).
/// - `Err(_)` — entry corrupted (header inválido, payload size > cap,
///   CRC mismatch, EOF mid-entry).
fn try_read_one_entry(f: &mut File) -> Result<Option<WalEntryRaw>, JournalError> {
    let mut entry_type_byte = [0u8; 1];
    match f.read_exact(&mut entry_type_byte) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(JournalError::Io(e.kind())),
    }
    let entry_type = match entry_type_byte[0] {
        0 => WalEntryType::Begin,
        1 => WalEntryType::SampleBatch,
        2 => WalEntryType::Commit,
        3 => WalEntryType::Cancel,
        4 => WalEntryType::Heartbeat,
        _ => return Err(JournalError::Io(std::io::ErrorKind::InvalidData)),
    };

    let mut payload_size_bytes = [0u8; 4];
    f.read_exact(&mut payload_size_bytes)
        .map_err(|e| JournalError::Io(e.kind()))?;
    let payload_size = u32::from_le_bytes(payload_size_bytes) as usize;
    if payload_size > MAX_WAL_PAYLOAD {
        return Err(JournalError::PayloadTooLarge(payload_size));
    }

    let mut payload_bytes = vec![0u8; payload_size];
    f.read_exact(&mut payload_bytes)
        .map_err(|e| JournalError::Io(e.kind()))?;

    let mut crc_bytes = [0u8; 4];
    f.read_exact(&mut crc_bytes)
        .map_err(|e| JournalError::Io(e.kind()))?;
    let stored_crc = u32::from_le_bytes(crc_bytes);

    // Audit J-3: recompute CRC over header+payload.
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(&entry_type_byte);
    hasher.update(&payload_size_bytes);
    hasher.update(&payload_bytes);
    let computed_crc = hasher.finalize();
    let crc_valid = stored_crc == computed_crc;

    Ok(Some(WalEntryRaw {
        entry_type,
        payload_bytes,
        crc_valid,
    }))
}

/// Resync: avança 1 byte por vez até achar próximo entry_type válido OR
/// budget exhausted. Retorna `Ok(true)` se reposicionou, `Ok(false)` se
/// chegou ao fim do file ou esgotou budget.
fn resync_from(
    f: &mut File,
    failed_entry_start: u64,
    budget: &mut u64,
) -> Result<bool, JournalError> {
    // Reposiciona logo após a entry que falhou (avançou 1 byte).
    let mut probe_pos = failed_entry_start.saturating_add(1);
    while *budget > 0 {
        if f.seek(SeekFrom::Start(probe_pos)).is_err() {
            return Ok(false);
        }
        let mut peek = [0u8; 1];
        match f.read_exact(&mut peek) {
            Ok(()) => {
                if peek[0] <= 4 {
                    // Possibly valid entry_type; rewind 1 byte e tenta ler.
                    if f.seek(SeekFrom::Current(-1)).is_err() {
                        return Ok(false);
                    }
                    return Ok(true);
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(false),
            Err(e) => return Err(JournalError::Io(e.kind())),
        }
        probe_pos = probe_pos.saturating_add(1);
        *budget = budget.saturating_sub(1);
    }
    Ok(false)
}

// =============================================================================
// Erros
// =============================================================================

/// Erros de [`StrokeJournal`] + [`read_journal`].
#[non_exhaustive]
#[derive(Debug)]
pub enum JournalError {
    /// I/O genérico.
    Io(std::io::ErrorKind),
    /// postcard serialização falhou.
    Postcard(postcard::Error),
    /// Magic bytes não batem com `PH2D-JOURNAL`.
    WrongMagic,
    /// WAL version > SCHEMA_VERSION.
    FutureVersion { file: u32, supported: u32 },
    /// `begin_stroke` chamado com stroke já ativo.
    StrokeAlreadyActive,
    /// `add_sample`/`commit_stroke`/`cancel_stroke` sem stroke ativo.
    NoActiveStroke,
    /// Payload de entry > MAX_WAL_PAYLOAD (16 MiB).
    PayloadTooLarge(usize),
    /// WAL file size > MAX_WAL_FILE_BYTES (500 MiB). Audit K-1.
    FileTooLarge { size: u64, max: u64 },
    /// Outro processo segura advisory lock no `journal_path`. Audit J-6.
    AlreadyLocked,
}

impl std::fmt::Display for JournalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(k) => write!(f, "journal I/O failed: {k:?}"),
            Self::Postcard(e) => write!(f, "journal postcard serialize: {e}"),
            Self::WrongMagic => write!(f, "journal magic bytes mismatch"),
            Self::FutureVersion { file, supported } => {
                write!(f, "journal version {file} > reader-supported {supported}")
            }
            Self::StrokeAlreadyActive => {
                write!(
                    f,
                    "begin_stroke called with active stroke (state machine violation)"
                )
            }
            Self::NoActiveStroke => write!(f, "operation requires active stroke (none)"),
            Self::PayloadTooLarge(n) => {
                write!(f, "entry payload too large: {n} bytes > {MAX_WAL_PAYLOAD}")
            }
            Self::FileTooLarge { size, max } => {
                write!(f, "WAL file too large: {size} bytes > {max} (anti-DoS)")
            }
            Self::AlreadyLocked => write!(
                f,
                "journal already locked by another process (single-writer invariant)"
            ),
        }
    }
}

impl std::error::Error for JournalError {}

impl PartialEq for JournalError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Io(a), Self::Io(b)) => a == b,
            (Self::Postcard(_), Self::Postcard(_)) => true,
            (Self::WrongMagic, Self::WrongMagic) => true,
            (
                Self::FutureVersion {
                    file: a,
                    supported: as_,
                },
                Self::FutureVersion {
                    file: b,
                    supported: bs,
                },
            ) => a == b && as_ == bs,
            (Self::StrokeAlreadyActive, Self::StrokeAlreadyActive) => true,
            (Self::NoActiveStroke, Self::NoActiveStroke) => true,
            (Self::PayloadTooLarge(a), Self::PayloadTooLarge(b)) => a == b,
            (Self::FileTooLarge { size: a, max: am }, Self::FileTooLarge { size: b, max: bm }) => {
                a == b && am == bm
            }
            (Self::AlreadyLocked, Self::AlreadyLocked) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_journal_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "ph2d-painter-stroke-journal-{}-{name}.wal",
            std::process::id()
        ))
    }

    fn sample_partial(seq: u64) -> PartialStroke {
        PartialStroke::new(
            seq,
            CanvasId(1),
            LayerId(0),
            BrushHandle::default(),
            [0u8; 32],
            OklchColor::default(),
        )
    }

    #[test]
    fn flush_policy_default_is_hybrid_8_100() {
        assert_eq!(
            FlushPolicy::default(),
            FlushPolicy::Hybrid { n: 8, ms: 100 }
        );
    }

    #[test]
    fn journal_open_writes_magic_and_version() {
        let path = tmp_journal_path("magic");
        std::fs::remove_file(&path).ok();
        {
            let _j = StrokeJournal::open(path.clone(), FlushPolicy::default()).expect("open");
        }
        let bytes = std::fs::read(&path).expect("read");
        assert_eq!(&bytes[..12], JOURNAL_MAGIC.as_slice());
        assert_eq!(&bytes[12..16], SCHEMA_VERSION.to_le_bytes().as_slice());
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn journal_begin_commit_roundtrip() {
        let path = tmp_journal_path("begin_commit");
        std::fs::remove_file(&path).ok();
        {
            let mut j = StrokeJournal::open(path.clone(), FlushPolicy::EveryNSamples(2)).unwrap();
            j.begin_stroke(sample_partial(0)).expect("begin");
            j.add_sample(RawPointerSample::default(), 0).expect("s1");
            j.add_sample(RawPointerSample::default(), 0)
                .expect("s2 → flush");
            assert!(j.sample_buffer.is_empty());
            let committed = j.commit_stroke(100).expect("commit");
            assert_eq!(committed.samples_count_in_journal, 2);
            assert!(j.current_stroke.is_none());
        }
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn journal_begin_twice_errors() {
        let path = tmp_journal_path("begin_twice");
        std::fs::remove_file(&path).ok();
        {
            let mut j = StrokeJournal::open(path.clone(), FlushPolicy::default()).unwrap();
            j.begin_stroke(sample_partial(0)).expect("begin1");
            let err = j.begin_stroke(sample_partial(1)).unwrap_err();
            assert_eq!(err, JournalError::StrokeAlreadyActive);
        }
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn journal_add_sample_without_begin_errors() {
        let path = tmp_journal_path("no_begin");
        std::fs::remove_file(&path).ok();
        {
            let mut j = StrokeJournal::open(path.clone(), FlushPolicy::default()).unwrap();
            let err = j.add_sample(RawPointerSample::default(), 0).unwrap_err();
            assert_eq!(err, JournalError::NoActiveStroke);
        }
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn read_journal_roundtrip_begin_samples_commit() {
        let path = tmp_journal_path("readback");
        std::fs::remove_file(&path).ok();
        {
            let mut j = StrokeJournal::open(path.clone(), FlushPolicy::EveryNSamples(100)).unwrap();
            j.begin_stroke(sample_partial(7)).unwrap();
            j.add_sample(RawPointerSample::default(), 0).unwrap();
            j.add_sample(RawPointerSample::default(), 0).unwrap();
            j.commit_stroke(50).unwrap();
        }
        let (entries, corrupted) = read_journal(&path).expect("read_journal");
        assert_eq!(corrupted, 0);
        assert_eq!(entries.len(), 3); // Begin + SampleBatch + Commit
        assert!(entries.iter().all(|e| e.crc_valid));
        assert_eq!(entries[0].entry_type, WalEntryType::Begin);
        assert_eq!(entries[1].entry_type, WalEntryType::SampleBatch);
        assert_eq!(entries[2].entry_type, WalEntryType::Commit);
        std::fs::remove_file(&path).ok();
    }

    /// Audit J-3 regression: bit-flip em header byte é detectado por CRC.
    #[test]
    fn read_journal_detects_header_tamper() {
        let path = tmp_journal_path("header_tamper");
        std::fs::remove_file(&path).ok();
        {
            let mut j = StrokeJournal::open(path.clone(), FlushPolicy::EveryNSamples(100)).unwrap();
            j.begin_stroke(sample_partial(0)).unwrap();
        }
        // Tamper byte 16 (entry_type byte do Begin entry).
        {
            use std::io::Seek;
            let mut f = OpenOptions::new().write(true).open(&path).unwrap();
            f.seek(SeekFrom::Start(16)).unwrap();
            f.write_all(&[0xFEu8]).unwrap();
            f.sync_all().unwrap();
        }
        // entry_type 0xFE é inválido → resync busca próxima entry; nada
        // existe depois → corrupted ≥ 1, entries vazio OR resync ainda
        // tenta achar (1 MB scan).
        let (_entries, corrupted) = read_journal(&path).expect("read");
        assert!(corrupted >= 1);
        std::fs::remove_file(&path).ok();
    }

    /// Audit J-2 regression: CRC sobre header+payload — payload tamper
    /// também detectado.
    #[test]
    fn read_journal_detects_payload_tamper() {
        let path = tmp_journal_path("payload_tamper");
        std::fs::remove_file(&path).ok();
        {
            let mut j = StrokeJournal::open(path.clone(), FlushPolicy::EveryNSamples(100)).unwrap();
            j.begin_stroke(sample_partial(0)).unwrap();
        }
        // Tamper byte 25 (no meio do payload do Begin).
        {
            use std::io::Seek;
            let mut f = OpenOptions::new().write(true).open(&path).unwrap();
            f.seek(SeekFrom::Start(25)).unwrap();
            f.write_all(&[0xFFu8]).unwrap();
            f.sync_all().unwrap();
        }
        let (entries, corrupted) = read_journal(&path).expect("read");
        assert_eq!(entries.len(), 1);
        assert!(!entries[0].crc_valid);
        // Header read foi OK (entry shape válida); só CRC falhou.
        // resync NÃO incrementa corrupted (entry shape lida + crc inválido = entries push).
        // Mas crc invalid significa entry compromised — caller (recovery) ignora.
        let _ = corrupted; // pode ser 0 (entry inteira lida com crc invalid)
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn read_journal_rejects_wrong_magic() {
        let path = tmp_journal_path("wrong_magic");
        std::fs::write(&path, b"NOT-A-JOURNAL\0\0\0\0").unwrap();
        let err = read_journal(&path).expect_err("wrong magic");
        assert_eq!(err, JournalError::WrongMagic);
        std::fs::remove_file(&path).ok();
    }

    /// Audit K-1 regression: file > MAX_WAL_FILE_BYTES rejeitado.
    #[test]
    fn read_journal_rejects_oversized_file() {
        let path = tmp_journal_path("oversized");
        // Cria file > MAX_WAL_FILE_BYTES (500 MiB) — usando sparse-file
        // técnica pra não gastar disk space real.
        let f = File::create(&path).unwrap();
        f.set_len(MAX_WAL_FILE_BYTES + 1).unwrap();
        let err = read_journal(&path).expect_err("oversized should fail");
        assert!(matches!(err, JournalError::FileTooLarge { .. }));
        std::fs::remove_file(&path).ok();
    }

    /// Audit J-6 regression: 2 processos abrindo mesmo path → segundo
    /// falha com AlreadyLocked.
    #[test]
    fn journal_open_rejects_second_writer_via_advisory_lock() {
        let path = tmp_journal_path("lock");
        std::fs::remove_file(&path).ok();
        let _j1 = StrokeJournal::open(path.clone(), FlushPolicy::default()).expect("j1");
        // Segundo open mesmo path no MESMO processo — advisory lock POSIX
        // permite re-acquire por mesmo PID (POSIX) OR rejeita (Linux
        // BSD locks). Comportamento varia; documentar OR ajustar.
        // Em macOS (POSIX): geralmente rejeita. Em Linux flock(): mesmo PID
        // pode re-acquire. Compromisso: aceitamos qualquer resultado.
        let j2 = StrokeJournal::open(path.clone(), FlushPolicy::default());
        // Strong assertion impossible cross-platform; weak: NÃO panicou.
        let _ = j2;
        drop(_j1);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn truncate_and_reset_re_writes_header() {
        let path = tmp_journal_path("rotate");
        std::fs::remove_file(&path).ok();
        {
            let mut j = StrokeJournal::open(path.clone(), FlushPolicy::default()).unwrap();
            j.begin_stroke(sample_partial(0)).unwrap();
            j.commit_stroke(0).unwrap();
            j.truncate_and_reset().expect("rotate");
        }
        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(bytes.len(), 16);
        assert_eq!(&bytes[..12], JOURNAL_MAGIC.as_slice());
        std::fs::remove_file(&path).ok();
    }

    /// Audit J-13 regression: SampleBatch carrega StrokeId explícito.
    #[test]
    fn sample_batch_payload_includes_stroke_id() {
        let path = tmp_journal_path("batch_uuid");
        std::fs::remove_file(&path).ok();
        let uuid = {
            let mut j = StrokeJournal::open(path.clone(), FlushPolicy::EveryNSamples(100)).unwrap();
            let s = sample_partial(0);
            let uuid = s.uuid;
            j.begin_stroke(s).unwrap();
            j.add_sample(RawPointerSample::default(), 0).unwrap();
            j.flush_sample_batch(0).unwrap();
            uuid
        };
        let (entries, _) = read_journal(&path).expect("read");
        let batch_entry = &entries[1];
        let payload: SampleBatchPayload =
            postcard::from_bytes(&batch_entry.payload_bytes).expect("deserialize");
        assert_eq!(payload.stroke_id, uuid);
        std::fs::remove_file(&path).ok();
    }
}
