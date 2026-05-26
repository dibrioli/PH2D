# ADR-0052 — Tear-resistant stroke commit + crash recovery (journal + auto-save + suspend handling)

**Status:** Proposed (2026-05-26)
**Decisor(es):** Enio + Claude (Coord-A, cascata W0 rework regra perfeição).
**Pré-requisitos:** [ADR-0046 — Stroke Vector History](0046-stroke-vector-history.md).
**Motivação:** sense-check veterano paint stack 2026-05-26 (gap #4 "tear-resistant stroke commit / app suspend mid-stroke"). Procreate sangrou esse exato bug em 2014-2017.
**Tags:** painter, wave-0, contract, durability, crash-recovery, journal, suspend-handling

---

## 1. Contexto

Pintura profissional dura horas. Eventos que **interrompem mid-stroke** destroem trabalho se não tratados:

| Evento | Frequência | Impacto sem tratamento |
|---|---|---|
| **App suspend mobile** (call/notification iOS, screen-off Android, low battery) | Várias/hora em sessões longas | Stroke em progresso perdido + canvas state perdido se save não rolou |
| **Power off súbito (mobile/desktop)** | Raro mas catastrófico | Toda sessão pós-último-save perdida |
| **Crash do app** (driver GPU bug, OOM, panic) | Raro com QA bom | Toda sessão pós-último-save perdida |
| **Kill explícito** (force quit, task manager) | Comum em desenvolvedor | Idem |
| **Tablet plug/unplug mid-stroke** | Wacom, XP-Pen | Stroke fica "open" — driver não envia release |
| **Hibernação OS** | Comum em laptop | Recuperação OS-dependent; estado memória pode ser corrompido |
| **Storage cheia mid-save** | Possível em iPad / cloud sync | Save truncado = arquivo corrompido |

Histórico de mercado:

- **Procreate 2014-2017** sangrou auto-save bugs. Perdia strokes em iPad multitasking. Anos para estabilizar.
- **Photoshop iPad 2019** lançou sem auto-save robusto; reviews terríveis.
- **Krita** tem auto-save mas crash mid-stroke perde o stroke em progresso (não journal).
- **Adobe Fresco** tem cloud auto-save (network dependent).

Sem ADR-0052:

1. **Stroke-in-progress não jornalado** — qualquer interrupção mid-stroke = stroke perdido.
2. **Auto-save policy implícita** — "save when convenient" varia por implementação.
3. **Suspend handling não-spec** — iOS background tasks API tem timing apertado (~5s); shell deve drainar o que dá.
4. **Crash recovery não-spec** — boot do app pós-crash não tem journal para replay.
5. **Storage-full mid-save** — sem atomic write, arquivo fica corrompido.

Regra "perfeição desde início, sem adiamentos" obriga endereçar agora.

---

## 2. Decisão

### 2.1 `ph2d-painter-stroke` (existente, ADR-0046) ganha módulos novos

```
crates/ph2d-painter-stroke/src/
  ... (existente ADR-0046)
  durability/
    mod.rs            # API top-level: StrokeJournal + AutoSave + CrashRecovery + SuspendHandler
    journal.rs        # StrokeJournal — append-only WAL de stroke-in-progress
    autosave.rs       # AutoSave policy + scheduler
    recovery.rs       # CrashRecovery — boot scan + replay
    suspend.rs        # SuspendHandler — iOS/Android lifecycle integration
    atomic_write.rs   # AtomicWriter — write-temp + fsync + rename
```

LOC budget `ph2d-painter-stroke` cresce ~800 LOC. Aceito; é foundation safety crítica.

### 2.2 `StrokeJournal` — write-ahead log de stroke-in-progress

Stroke-in-progress (entre `BeginStroke` e `CommitStroke` ou `CancelStroke`) é **jornalado per-sample**:

```rust
pub struct StrokeJournal {
    pub journal_path: PathBuf,               // <storage_root>/painter_journal.wal
    pub current_stroke: Option<PartialStroke>,
    pub sample_buffer: Vec<RawPointerSample>,
    pub last_flush_ms: u64,
    pub flush_policy: FlushPolicy,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PartialStroke {
    pub uuid: StrokeId,                      // gerado em BeginStroke (commit-final ou cancel-discard)
    pub seq: u64,                            // pre-allocated para preserve order em crash recovery
    pub canvas_id: CanvasId,
    pub layer_target: LayerId,
    pub brush_handle: BrushHandle,
    pub brush_params_hash: BrushParamsHash,
    pub primary_color: OklchColor,
    pub secondary_color: Option<OklchColor>,
    pub tool_mode: ToolMode,
    pub rng_seed: u64,                       // gerado upfront, NÃO no commit
    pub started_at_ms: u64,
    pub samples_count_in_journal: u32,       // checksum: file should have this many samples
    pub version: u32,                        // HR-14 v1 = 1
    // === 2 slots de headroom ===
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum FlushPolicy {
    /// Flush a cada N samples (default N=8). Cobre 99% stroke loss.
    EveryNSamples(u32),
    /// Flush a cada M milliseconds (default 100ms). Latency garantida.
    EveryMs(u32),
    /// Both — flush quando QUALQUER acontecer primeiro.
    Hybrid { n: u32, ms: u32 },
    // === 1 slot de headroom ===
}
```

**Default:** `FlushPolicy::Hybrid { n: 8, ms: 100 }`. Trade-off: ~100ms worst-case stroke loss em crash; disk I/O ~10 flushes/s em stroke rápido (negligível).

**Layout do WAL `painter_journal.wal`:**

```
[u8;12] = "PH2D-JOURNAL"
u32 = version (1)
// Per stroke entry:
[u8] = entry_type (0=Begin, 1=SampleBatch, 2=Commit, 3=Cancel, 4=Heartbeat)
u32 = payload_size
[u8; payload_size] = postcard-encoded payload
u32 = crc32 of payload
```

**Append-only**, never rewrite. Rotação: após commit-success em main history, **WAL entry rewritten as `Commit` marker**; periodic `truncate` reclama disk space (a cada 100 commits OR > 50MB WAL).

### 2.3 `CrashRecovery` — boot scan + replay

```rust
pub struct CrashRecovery {
    pub journal_path: PathBuf,
    pub recovered_strokes: Vec<RecoveredStroke>,
    pub corrupted_entries: u32,              // count, não erro fatal
}

pub struct RecoveredStroke {
    pub partial: PartialStroke,
    pub samples: Vec<RawPointerSample>,
    pub state: RecoveryState,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum RecoveryState {
    /// WAL tem Begin + samples + Commit marker → stroke completou disco mas não foi gravado em main history.
    /// Action: replay para history; mark committed.
    CommittedNotPersisted,
    /// WAL tem Begin + samples mas NO Commit → stroke estava em progresso quando crashou.
    /// Action: UX dialog "Recover stroke in progress?" → commit ou discard.
    InProgressAtCrash,
    /// WAL tem Begin + Cancel → stroke foi cancelado pelo user; descartar.
    Cancelled,
    /// CRC fail / parse error em entry → corrupted; pular + log.
    Corrupted,
}
```

**Boot flow:**

1. App boota; check `painter_journal.wal` existe.
2. Scan completo WAL (rápido, ~5ms para 50MB).
3. Para cada `RecoveredStroke`:
   - `CommittedNotPersisted` → silently re-add para history + flush para `.ph2d-painter`.
   - `InProgressAtCrash` → coleta para UX prompt.
   - `Cancelled` / `Corrupted` → descarte silencioso (Corrupted logged).
4. Se houver `InProgressAtCrash`, dialog único: "Recover N strokes from last session?" → user click commit-all / discard-all / review-individually.
5. Após resolução, truncate WAL para 0 + start fresh journal.

### 2.4 `AutoSave` policy

`.ph2d-painter` (canon, ADR-0046 §2.7.1) é salvo periodicamente:

```rust
pub struct AutoSave {
    pub canvas_id: CanvasId,
    pub path: PathBuf,
    pub policy: AutoSavePolicy,
    pub last_save_ms: u64,
    pub strokes_since_last: u32,
    pub state: AutoSaveState,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AutoSavePolicy {
    /// Salva após N strokes commitados (default 25). Mobile-friendly.
    EveryNStrokes(u32),
    /// Salva a cada M minutos de wall-clock (default 5min).
    EveryMinutes(u32),
    /// Hybrid (recomendado): N strokes OR M minutos primeiro.
    Hybrid { n: u32, minutes: u32 },
    /// Manual only — power user.
    Manual,
    // === 1 slot de headroom ===
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AutoSaveState {
    Idle,
    Pending,                                 // dirty mas dentro da janela de policy
    SavingInBackground,                      // I/O em worker thread
    SavedAt(u64),                            // ms timestamp
    Failed(AutoSaveError),                   // diagnostic; UX toast
}
```

**Default:** `Hybrid { n: 25, minutes: 5 }`.

**Atomic write protocol (essencial em storage cheia / power-off):**

```rust
fn atomic_write(path: &Path, content: &[u8]) -> Result<(), io::Error> {
    let tmp = path.with_extension("ph2d-painter.tmp");
    {
        let mut f = File::create(&tmp)?;
        f.write_all(content)?;
        f.sync_all()?;                       // fsync — força flush para storage
    }
    fs::rename(tmp, path)?;                  // atomic rename — POSIX guarantee
    // No final, sync_all do diretório (POSIX para garantir rename persistido)
    let dir = path.parent().ok_or(io::ErrorKind::Other)?;
    File::open(dir)?.sync_all()?;
    Ok(())
}
```

**Storage-full handling:** `AtomicWriter` checa espaço disponível ANTES de gravar; se < 2× `.ph2d-painter` size, falha early com `AutoSaveError::InsufficientStorage`. UX toast: "Espaço de armazenamento insuficiente para auto-save. Libere espaço ou salve manualmente em outro local."

### 2.5 `SuspendHandler` — iOS / Android lifecycle integration

iOS background task API dá **~5s** para gracefully shutdown antes de força-kill. Android idem (com variações). Shell deve drenar:

```rust
pub struct SuspendHandler {
    pub state: SuspendState,
    pub drain_deadline_ms: u64,              // calculated from platform deadline
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SuspendState {
    Active,
    SuspendImminent { remaining_ms: u32 },
    Resuming,
}
```

**Drain protocol (chamado pela shell via `PlatformHost::on_suspend_imminent()` — extension):**

1. **t=0:** suspend imminent event recebido. `SuspendState::SuspendImminent { remaining_ms: 5000 }`.
2. **t<2000ms:** flush WAL (fsync) — stroke-in-progress preservado.
3. **t<3500ms:** atomic write `.ph2d-painter` se dirty.
4. **t<4500ms:** flush `.ph2d-painter-cache` se dirty (best-effort; sidecar regenerável).
5. **t<4900ms:** mark `SuspendState::SuspendImminent { remaining_ms: 100 }`; final fsync.
6. **t=5000ms:** force-kill pelo OS (esperado).

**Gate `suspend_drain_completes_under_5s`:** simulated suspend → WAL + canvas state persisted em ≤ 5s p99.

### 2.6 `PlatformHost::on_suspend_imminent()` — trait extension

ADR-0051 já amenda `ph2d-host::PlatformHost` (gyroscope). Esta ADR adiciona método paralelo:

```rust
pub trait PlatformHost {
    // ... existentes + gyroscope (ADR-0049) ...

    /// Called by the platform when app is about to suspend / be killed.
    /// `deadline_ms` is the OS-imposed remaining time to gracefully shutdown.
    /// Default impl: log warning + no-op (desktop fallback).
    fn on_suspend_imminent(&mut self, deadline_ms: u32) {
        // default — desktop platforms sem suspend semantics
    }

    /// Called when app resumes from suspend. Tool may need to refresh
    /// device state (Apple Pencil reconnect, Wacom driver re-init).
    fn on_resume(&mut self) {}
}
```

**Coord-A only** (mexer em `ph2d-host` é trait foundational — autorizado por ADR-0043 §2.5 ceded list, adicionado nesta cascata rework).

### 2.7 Stroke-in-progress: lifecycle congelado

```
[idle] ──BeginStroke()──> [stroke_active]
                              │
                              ├─AddSample()──► WAL flush per FlushPolicy
                              │                  (sample append disco)
                              │
                              ├─CommitStroke()──► WAL entry Commit
                              │                  ↓
                              │              History append (in-memory)
                              │                  ↓
                              │              AutoSave triggers per policy
                              │                  ↓
                              │              [idle]
                              │
                              └─CancelStroke()──► WAL entry Cancel
                                                  ↓
                                              [idle]
[crashed mid-stroke] ──boot──► CrashRecovery.recover()
                              │
                              ├─CommittedNotPersisted → silent replay
                              ├─InProgressAtCrash → UX prompt
                              └─Cancelled / Corrupted → discard
```

### 2.8 Caps numéricos

| Tipo | Cap |
|---|---|
| `PartialStroke` | ≤ 16 fields (v1 = 12) |
| `FlushPolicy` | ≤ 6 variants (v1 = 3) |
| `AutoSavePolicy` | ≤ 8 variants (v1 = 4) |
| `AutoSaveState` | ≤ 6 variants (v1 = 5) |
| `RecoveryState` | ≤ 6 variants (v1 = 4) |
| `SuspendState` | ≤ 4 variants (v1 = 3) |
| `AutoSaveError` | ≤ 8 variants (v1 = 4: IoError, InsufficientStorage, AtomicWriteFailed, FsyncFailed) |
| WAL flush interval | 100ms OR 8 samples (Hybrid default) |
| Drain deadline | 5000ms (iOS) / configurable per platform |

### 2.9 Arch-gate `painter_contract_surface::durability`

```rust
mod durability {
    #[test] fn partial_stroke_field_count_is_capped()           { /* ≤ 16 */ }
    #[test] fn flush_policy_variant_count_is_capped()           { /* ≤ 6 */ }
    #[test] fn auto_save_policy_variant_count_is_capped()       { /* ≤ 8 */ }
    #[test] fn recovery_state_variant_count_is_capped()         { /* ≤ 6 */ }
    #[test] fn suspend_state_variant_count_is_capped()          { /* ≤ 4 */ }
    #[test] fn atomic_write_uses_fsync_rename_pattern()         { /* grep textual */ }
    #[test] fn wal_entries_are_crc32_verified()                 { /* §2.2 */ }
}
```

### 2.10 Gates de comportamento

| Gate | Crate | Valida |
|---|---|---|
| `stroke_journal_flush_per_8_samples` | ph2d-painter-stroke | WAL flush a cada 8 samples (default). |
| `stroke_journal_flush_per_100ms` | idem | WAL flush a cada 100ms (default). |
| `crash_recovery_replays_committed_not_persisted` | idem | Sim crash entre Commit WAL + save .ph2d-painter → boot recupera silently. |
| `crash_recovery_prompts_in_progress_at_crash` | idem | Crash mid-stroke → UX dialog "Recover N strokes?". |
| `crash_recovery_skips_corrupted_entries` | idem | CRC fail → skip + log, sem panic. |
| `auto_save_atomic_write_no_corruption` | idem | Simulated kill durante write → .ph2d-painter NUNCA corrompido (tmp + rename). |
| `auto_save_fails_on_insufficient_storage` | idem | Disk full → AutoSaveError::InsufficientStorage + UX toast; sem corrupt write. |
| `suspend_drain_completes_under_5s` | idem (cross com ph2d-host) | Simulated suspend → state persisted ≤ 5s p99. |
| `resume_refreshes_device_state` | idem | After on_resume(), tool/device state válido (Apple Pencil reconnect etc.). |
| `tablet_unplug_mid_stroke_finalizes` | idem (cross com ADR-0050) | Driver release events ausentes (unplug) → tool detect via timeout (1s) e auto-commit stroke. |

---

## 3. Consequências

### Positivas

- **Stroke perdido é IMPOSSÍVEL.** WAL + flush 100ms = worst-case 100ms de samples perdidos. Stroke completo NUNCA perdido.
- **Crash recovery silent para committed-not-persisted strokes.** User experience: app fecha mid-session, abre app, canvas está IGUAL.
- **Storage-full nunca corrompe save.** Atomic write + fsync + check-before-write.
- **iOS background suspend handled.** 5s drain protocol explícito.
- **Tablet unplug mid-stroke não trava.** Timeout 1s → auto-commit ou descarte conforme user choice.

### Negativas / Custos

- **WAL I/O ~10 flushes/s durante stroke.** ~80 bytes/sample × 8 samples = 640 bytes/flush. SSD: imperceptível. eMMC mobile: ainda imperceptível. iPad: APFS handles fine. Aceito.
- **Atomic write 2× disk usage temporariamente.** `.ph2d-painter` 50MB + `.tmp` 50MB durante write. Storage check evita disco-full.
- **Boot scan WAL ~5ms.** Imperceptível.
- **fsync custa latência.** Mobile fsync ~5-10ms typical. AutoSave em worker thread; UI nunca bloqueada.

### Neutras

- **`ph2d-painter-stroke` crate cresce +800 LOC.** Aceito; safety crítica vale.
- **`PlatformHost::on_suspend_imminent` é 2º amend foundational** desta cascata W0 (1º foi `gyroscope()`). Trait cresce ~2-3 métodos total — ainda dentro de "small surface".

---

## 4. Alternativas consideradas

### 4.1 Auto-save sem WAL (só periodic full save)

**Rejeitada.** Crash mid-stroke = stroke perdido. Procreate-style; precisamente o gap apontado.

### 4.2 WAL append-only sem flush policy (lazy)

**Rejeitada.** "Lazy" + crash = arbitrary loss. Flush 100ms é trade-off industry-standard (PostgreSQL WAL fsync, SQLite WAL).

### 4.3 Cloud-only auto-save (Adobe Fresco style)

**Rejeitada.** Network dependent; offline workflow morre. Local-first canon.

### 4.4 Suspend drain best-effort sem deadline awareness

**Rejeitada.** iOS força-kill em 5s. Sem priorização (WAL > .ph2d-painter > cache), ordem errada perde stroke.

### 4.5 Crash recovery silent SEM UX prompt para in-progress

**Rejeitada.** Auto-commit stroke incompleto pode "completar" um stroke que user ia cancelar (pinta acidente, crash, app abre com acidente commited). UX prompt é honesto.

---

## 5. Verificação

```sh
cargo test -p ph2d-painter-stroke
# Durability tests + simulated crash + atomic write.

cargo test -p ph2d-painter-contracts --test architecture_painter_contract_surface
# Caps cumulativos.
```

**Manual smoke obrigatório em W1 T-durability:**
1. Pintar 50 strokes; force-quit app mid-stroke; reabrir → recovery dialog mostra "1 stroke in progress" + canvas com 49 commits.
2. iPad multitasking suspend during stroke → resume → stroke completa.
3. Disk full simulado → AutoSave falha gracefully + UX toast.

---

## 6. Tracking

- Plano operacional: integra em W1 T-durability (paralelo a T-input/T-color).
- Spec normativa: integra com `08_performance_memory.md` (I/O budget).
- Próxima ADR na cascata W0 rework: [ADR-0053 — Cross-platform tier policy](0053-cross-platform-tier.md).
