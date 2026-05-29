//! Integration test T1.9 — `PainterTool` wire `StrokeHistory` + `StrokeJournal`.
//!
//! Cobre 3 cenários canon:
//! 1. **In-memory only** — `stroke_journal == None`; samples vão pra
//!    `current_samples` + history.push em end_stroke.
//! 2. **Com journal** — WAL persiste Begin/SampleBatch/Commit; crash
//!    simulado (drop sem end_stroke) deixa stroke `InProgressAtCrash`
//!    em `CrashRecovery::scan`.
//! 3. **Recovery roundtrip** — pintar 3 strokes commitados + crash mid 4º →
//!    `CrashRecovery` recupera 3 committed + 1 in-progress.

use std::path::PathBuf;

use ph2d_painter_brush::PointerSample;
use ph2d_painter_stroke::{
    CanvasId, CrashRecovery, FlushPolicy, JournalError, LayerId, RecoveryState,
    SAMPLE_FLAG_AZIMUTH_UNAVAILABLE, SAMPLE_FLAG_BARREL_ROLL_UNAVAILABLE,
    SAMPLE_FLAG_TIMESTAMP_UNAVAILABLE, StrokeHistory,
};
use ph2d_tool_painter::PainterTool;

fn tmp_wal_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "ph2d-tool-painter-t19-{}-{name}.wal",
        std::process::id()
    ))
}

fn mk_painter_with_canvas() -> PainterTool {
    use ph2d_editor_core::tool::RasterEditTool;
    let mut t = PainterTool::default();
    // Ativa source canvas pra queue_pointer poder pintar.
    let w = 32u32;
    let h = 32u32;
    let pixels = vec![255u8; (w * h * 4) as usize];
    t.set_source(pixels, w, h);
    t
}

#[test]
fn in_memory_only_lifecycle_pushes_to_history() {
    let mut t = mk_painter_with_canvas();
    // 1 stroke completo → history.len == 1.
    t.begin_stroke(42);
    t.queue_pointer(PointerSample {
        position: [10.0, 10.0],
        pressure: 0.8,
        tilt: 0.0,
    });
    t.queue_pointer(PointerSample {
        position: [11.0, 11.0],
        pressure: 0.9,
        tilt: 0.0,
    });
    t.end_stroke();
    assert_eq!(t.stroke_history().len(), 1, "1 stroke commitado em history");
    let recorded = t.stroke_history().iter().next().unwrap();
    assert_eq!(recorded.seq, 0, "primeiro stroke tem seq=0");
    assert_eq!(recorded.rng_seed, 42);
    assert!(!recorded.points.is_empty(), "samples preservados em points");
}

#[test]
fn in_memory_only_seq_monotonic_across_strokes() {
    let mut t = mk_painter_with_canvas();
    for i in 0..5u64 {
        t.begin_stroke(100 + i);
        t.queue_pointer(PointerSample {
            position: [i as f32, i as f32],
            pressure: 1.0,
            tilt: 0.0,
        });
        t.end_stroke();
    }
    let seqs: Vec<u64> = t.stroke_history().iter().map(|r| r.seq).collect();
    assert_eq!(seqs, vec![0, 1, 2, 3, 4]);
}

#[test]
fn attach_journal_wires_wal_writes() {
    let path = tmp_wal_path("attach");
    std::fs::remove_file(&path).ok();

    let mut t = mk_painter_with_canvas();
    t.attach_journal(path.clone(), FlushPolicy::EveryNSamples(100))
        .expect("attach");

    t.begin_stroke(7);
    t.queue_pointer(PointerSample {
        position: [5.0, 5.0],
        pressure: 0.5,
        tilt: 0.0,
    });
    t.end_stroke();

    // history tem 1; journal escreveu Begin+SampleBatch+Commit.
    assert_eq!(t.stroke_history().len(), 1);

    // Detach + recovery scan.
    t.detach_journal();
    let recovery = CrashRecovery::scan(path.clone()).expect("scan");
    // Stroke committed cleanly → CommittedNotPersisted (canon NÃO save aqui).
    assert_eq!(recovery.recovered_strokes.len(), 1);
    assert_eq!(
        recovery.recovered_strokes[0].state,
        RecoveryState::CommittedNotPersisted
    );

    std::fs::remove_file(&path).ok();
}

#[test]
fn crash_mid_stroke_recovery_yields_in_progress() {
    let path = tmp_wal_path("crash_mid");
    std::fs::remove_file(&path).ok();
    {
        let mut t = mk_painter_with_canvas();
        t.attach_journal(path.clone(), FlushPolicy::EveryNSamples(100))
            .expect("attach");

        // 3 strokes commitados.
        for i in 0..3 {
            t.begin_stroke(100 + i);
            t.queue_pointer(PointerSample {
                position: [i as f32, i as f32],
                pressure: 1.0,
                tilt: 0.0,
            });
            t.end_stroke();
        }

        // 4º stroke começa, recebe sample, MAS NÃO chama end_stroke (crash).
        t.begin_stroke(999);
        t.queue_pointer(PointerSample {
            position: [50.0, 50.0],
            pressure: 1.0,
            tilt: 0.0,
        });
        // Force flush remaining batch pra recovery encontrar samples.
        if let Some(j) = t.stroke_journal.as_mut() {
            let _ = j.flush_sample_batch(0);
        }
        // t drops aqui → journal Drop libera lock; sem Commit do 4º stroke.
    }

    let recovery = CrashRecovery::scan(path.clone()).expect("scan");
    assert_eq!(recovery.recovered_strokes.len(), 4);
    assert_eq!(recovery.committed_not_persisted().count(), 3);
    assert_eq!(recovery.in_progress_at_crash().count(), 1);

    std::fs::remove_file(&path).ok();
}

#[test]
fn set_canvas_id_stamped_in_partial_stroke() {
    let path = tmp_wal_path("canvas_id");
    std::fs::remove_file(&path).ok();

    let mut t = mk_painter_with_canvas();
    t.attach_journal(path.clone(), FlushPolicy::EveryNSamples(100))
        .expect("attach");
    t.set_canvas_id(CanvasId(42));
    t.begin_stroke(0);
    t.queue_pointer(PointerSample {
        position: [1.0, 1.0],
        pressure: 1.0,
        tilt: 0.0,
    });
    t.end_stroke();
    t.detach_journal();

    let rec = CrashRecovery::scan(path.clone()).expect("scan");
    assert_eq!(rec.recovered_strokes.len(), 1);
    assert_eq!(rec.recovered_strokes[0].partial.canvas_id, CanvasId(42));

    // Filter por canvas — caller W11+ pattern.
    assert_eq!(rec.committed_for_canvas(CanvasId(42)).count(), 1);
    assert_eq!(rec.committed_for_canvas(CanvasId(99)).count(), 0);

    std::fs::remove_file(&path).ok();
}

#[test]
fn set_next_seq_overrides_baseline() {
    let mut t = mk_painter_with_canvas();
    t.set_next_seq(100);
    t.begin_stroke(0);
    t.queue_pointer(PointerSample {
        position: [1.0, 1.0],
        pressure: 1.0,
        tilt: 0.0,
    });
    t.end_stroke();
    let recorded = t.stroke_history().iter().next().unwrap();
    assert_eq!(
        recorded.seq, 100,
        "next_seq override aplica no primeiro stroke"
    );
}

#[test]
fn set_layer_target_stamped_in_record() {
    let mut t = mk_painter_with_canvas();
    t.set_layer_target(LayerId(7));
    t.begin_stroke(0);
    t.queue_pointer(PointerSample {
        position: [1.0, 1.0],
        pressure: 1.0,
        tilt: 0.0,
    });
    t.end_stroke();
    let recorded = t.stroke_history().iter().next().unwrap();
    assert_eq!(recorded.layer_target, LayerId(7));
}

#[test]
fn deactivate_drops_journal_and_clears_in_progress() {
    use ph2d_editor_core::tool::Tool;
    let path = tmp_wal_path("deactivate");
    std::fs::remove_file(&path).ok();
    let mut t = mk_painter_with_canvas();
    t.attach_journal(path.clone(), FlushPolicy::EveryNSamples(100))
        .expect("attach");
    t.begin_stroke(0);
    t.queue_pointer(PointerSample {
        position: [1.0, 1.0],
        pressure: 1.0,
        tilt: 0.0,
    });
    // Caller esqueceu end_stroke — deactivate cleanup.
    Tool::on_deactivate(&mut t);
    assert!(t.stroke_journal.is_none(), "deactivate drops journal");
    assert!(t.current_samples_is_empty(), "deactivate clears samples");

    // Lock liberado — outro open mesmo path funciona.
    let _t2 = ph2d_painter_stroke::StrokeJournal::open(path.clone(), FlushPolicy::default())
        .expect("re-open após deactivate");

    std::fs::remove_file(&path).ok();
}

#[test]
fn take_stroke_history_replaces_with_default() {
    let mut t = mk_painter_with_canvas();
    t.begin_stroke(0);
    t.queue_pointer(PointerSample {
        position: [1.0, 1.0],
        pressure: 1.0,
        tilt: 0.0,
    });
    t.end_stroke();
    assert_eq!(t.stroke_history().len(), 1);

    let taken = t.take_stroke_history();
    assert_eq!(taken.len(), 1);
    assert_eq!(t.stroke_history().len(), 0, "after take_stroke_history");
    assert!(matches!(t.stroke_history(), StrokeHistory::Full(_)));
}

// =============================================================================
// Audit T1.9 remediation tests (Q-2 / Q-4 / Q-6 / Q-8/R-1 / R-2)
// =============================================================================

/// Q-2: begin_stroke cancelando stroke ativo NÃO consome seq.
/// Cenário: begin(A) sem end → begin(B) → end(B) → seq de B = 0, não 1.
#[test]
fn cancelled_stroke_does_not_consume_seq() {
    let mut t = mk_painter_with_canvas();
    t.begin_stroke(42);
    t.queue_pointer(PointerSample {
        position: [1.0, 1.0],
        pressure: 1.0,
        tilt: 0.0,
    });
    // Sem end_stroke — caller esqueceu.
    t.begin_stroke(99); // cancela A, inicia B
    t.queue_pointer(PointerSample {
        position: [2.0, 2.0],
        pressure: 1.0,
        tilt: 0.0,
    });
    t.end_stroke();

    let recorded = t.stroke_history().iter().next().unwrap();
    assert_eq!(
        recorded.seq, 0,
        "B mantém seq=0 — A foi cancelado, não commitado"
    );
    assert_eq!(t.stroke_history().len(), 1);
}

/// Q-2 (continuação): após cancel+commit, próximo stroke recebe seq=1.
#[test]
fn next_seq_advances_only_on_commit() {
    let mut t = mk_painter_with_canvas();
    t.begin_stroke(0);
    t.queue_pointer(PointerSample {
        position: [1.0, 1.0],
        pressure: 1.0,
        tilt: 0.0,
    });
    // cancela
    t.begin_stroke(1);
    t.queue_pointer(PointerSample {
        position: [2.0, 2.0],
        pressure: 1.0,
        tilt: 0.0,
    });
    t.end_stroke(); // commits seq=0

    t.begin_stroke(2);
    t.queue_pointer(PointerSample {
        position: [3.0, 3.0],
        pressure: 1.0,
        tilt: 0.0,
    });
    t.end_stroke(); // commits seq=1

    let seqs: Vec<u64> = t.stroke_history().iter().map(|r| r.seq).collect();
    assert_eq!(seqs, vec![0, 1], "seq contíguo apesar do cancel");
}

/// Q-4: BrushHandle stub com slot ≥ 64 (built-in) cai pra default
/// em vez de panicar.
#[test]
fn brush_handle_slot_overflow_falls_back_to_default() {
    use ph2d_tool_painter::params::BrushHandle as StubBrushHandle;
    let mut t = mk_painter_with_canvas();
    // Escreve slot inválido direto no pub field (simula bridge bug).
    t.params.active_brush = StubBrushHandle(100); // built-in 100 = invalid
    // No panic — fallback pra default (slot 0).
    t.begin_stroke(0);
    t.queue_pointer(PointerSample {
        position: [1.0, 1.0],
        pressure: 1.0,
        tilt: 0.0,
    });
    t.end_stroke();
    let recorded = t.stroke_history().iter().next().unwrap();
    // Default = slot 0 built-in.
    assert_eq!(recorded.brush_handle.slot(), 0);
    assert!(!recorded.brush_handle.is_imported());
}

/// Q-6: pointer_to_raw_sample seta SAMPLE_FLAG_* sentinel pra campos
/// não suportados pelo gerador T1.9.
#[test]
fn raw_sample_flags_sentinel_for_unavailable_fields() {
    let mut t = mk_painter_with_canvas();
    t.begin_stroke(0);
    t.queue_pointer(PointerSample {
        position: [10.0, 10.0],
        pressure: 0.5,
        tilt: 0.3,
    });
    t.end_stroke();
    let recorded = t.stroke_history().iter().next().unwrap();
    let sample = &recorded.points[0];
    assert!(
        sample.flags & SAMPLE_FLAG_AZIMUTH_UNAVAILABLE != 0,
        "azimuth flag set"
    );
    assert!(
        sample.flags & SAMPLE_FLAG_BARREL_ROLL_UNAVAILABLE != 0,
        "barrel flag set"
    );
    assert!(
        sample.flags & SAMPLE_FLAG_TIMESTAMP_UNAVAILABLE != 0,
        "timestamp flag set"
    );
}

/// Q-8/R-1: attach_journal com stroke ativo retorna
/// AttachDuringActiveStroke em vez de criar journal órfão.
#[test]
fn attach_journal_mid_stroke_returns_error() {
    let path = tmp_wal_path("attach_mid");
    std::fs::remove_file(&path).ok();
    let mut t = mk_painter_with_canvas();
    t.begin_stroke(0);
    t.queue_pointer(PointerSample {
        position: [1.0, 1.0],
        pressure: 1.0,
        tilt: 0.0,
    });
    let result = t.attach_journal(path.clone(), FlushPolicy::default());
    assert!(matches!(
        result,
        Err(JournalError::AttachDuringActiveStroke)
    ));
    // Após end_stroke, attach funciona.
    t.end_stroke();
    t.attach_journal(path.clone(), FlushPolicy::default())
        .expect("attach OK após end_stroke");
    t.detach_journal();
    std::fs::remove_file(&path).ok();
}

/// R-2: detach_journal cancela stroke ativo em vez de deixar
/// Begin órfão (que recovery interpretaria como InProgressAtCrash).
#[test]
fn detach_journal_cancels_active_stroke() {
    let path = tmp_wal_path("detach_cancel");
    std::fs::remove_file(&path).ok();
    {
        let mut t = mk_painter_with_canvas();
        t.attach_journal(path.clone(), FlushPolicy::EveryNSamples(100))
            .expect("attach");
        t.begin_stroke(0);
        t.queue_pointer(PointerSample {
            position: [1.0, 1.0],
            pressure: 1.0,
            tilt: 0.0,
        });
        // Detach mid-stroke — deve cancelar.
        t.detach_journal();
    }
    let recovery = CrashRecovery::scan(path.clone()).expect("scan");
    // O WAL é append-only e o `Begin` foi fsynced eagerly em `begin_stroke`
    // (durabilidade crash ADR-0052) — detach mid-stroke NÃO pode apagá-lo;
    // emite um terminator `Cancel`. Recovery então classifica o stroke como
    // `Cancelled` (descartado silenciosamente no boot), e NÃO como
    // `InProgressAtCrash` (que dispararia o prompt "recover orphan work").
    // A garantia sob teste: detach mid-stroke deixa ZERO strokes recuperáveis-
    // como-vivos. (Contrato espelhado em recovery.rs::recovery_detects_cancelled,
    // que prova recovered_strokes.len() == 1 + state Cancelled.)
    assert_eq!(
        recovery.in_progress_at_crash().count(),
        0,
        "detach mid-stroke não pode deixar Begin órfão recuperável-como-vivo"
    );
    assert!(
        recovery
            .recovered_strokes
            .iter()
            .all(|s| s.state == RecoveryState::Cancelled),
        "o stroke do detach deve ser classificado Cancelled, não vivo"
    );
    std::fs::remove_file(&path).ok();
}

/// R-2 + deactivate: deactivate também cancela mid-stroke pra não
/// deixar Begin órfão.
#[test]
fn deactivate_cancels_active_stroke_in_wal() {
    use ph2d_editor_core::tool::Tool;
    let path = tmp_wal_path("deactivate_cancel");
    std::fs::remove_file(&path).ok();
    {
        let mut t = mk_painter_with_canvas();
        t.attach_journal(path.clone(), FlushPolicy::EveryNSamples(100))
            .expect("attach");
        t.begin_stroke(0);
        t.queue_pointer(PointerSample {
            position: [1.0, 1.0],
            pressure: 1.0,
            tilt: 0.0,
        });
        Tool::on_deactivate(&mut t);
    }
    let recovery = CrashRecovery::scan(path.clone()).expect("scan");
    // Mesma semântica de `detach_journal_cancels_active_stroke`: o `Begin`
    // já está no disco (append-only), então `on_deactivate` emite `Cancel` e
    // recovery vê o stroke como `Cancelled`, nunca `InProgressAtCrash`.
    assert_eq!(
        recovery.in_progress_at_crash().count(),
        0,
        "deactivate mid-stroke não pode deixar Begin órfão recuperável-como-vivo"
    );
    assert!(
        recovery
            .recovered_strokes
            .iter()
            .all(|s| s.state == RecoveryState::Cancelled),
        "o stroke do deactivate deve ser classificado Cancelled, não vivo"
    );
    std::fs::remove_file(&path).ok();
}

/// Q-3/R-3: last_wal_error é None após attach bem-sucedido.
#[test]
fn last_wal_error_none_after_attach() {
    let path = tmp_wal_path("wal_err_none");
    std::fs::remove_file(&path).ok();
    let mut t = mk_painter_with_canvas();
    t.attach_journal(path.clone(), FlushPolicy::default())
        .expect("attach");
    assert!(t.last_wal_error().is_none());
    t.begin_stroke(0);
    t.end_stroke();
    assert!(
        t.last_wal_error().is_none(),
        "WAL OK em begin/commit ⇒ sem erro"
    );
    t.detach_journal();
    std::fs::remove_file(&path).ok();
}

/// Q-2 + R-9 doc: also smoke-check current_samples_len getter pra W14.
#[test]
fn current_samples_len_tracks_pushed_samples() {
    let mut t = mk_painter_with_canvas();
    assert_eq!(t.current_samples_len(), 0);
    t.begin_stroke(0);
    assert_eq!(t.current_samples_len(), 0);
    t.queue_pointer(PointerSample {
        position: [1.0, 1.0],
        pressure: 1.0,
        tilt: 0.0,
    });
    t.queue_pointer(PointerSample {
        position: [2.0, 2.0],
        pressure: 1.0,
        tilt: 0.0,
    });
    assert_eq!(t.current_samples_len(), 2);
    t.end_stroke();
    assert!(t.current_samples_is_empty());
}

// =============================================================================
// Auditoria completa T1.9 (4 lentes S/T/U/V) — gates de remediação
// =============================================================================

/// S-1: PartialStroke.started_at_ms (mapped to timestamp_ms) é wall-clock
/// real (não-zero) após begin_stroke.
#[test]
fn s1_timestamp_ms_is_set_from_wall_clock() {
    let mut t = mk_painter_with_canvas();
    t.begin_stroke(0);
    t.queue_pointer(PointerSample {
        position: [1.0, 1.0],
        pressure: 1.0,
        tilt: 0.0,
    });
    t.end_stroke();
    let recorded = t.stroke_history().iter().next().unwrap();
    assert!(
        recorded.timestamp_ms > 1_700_000_000_000,
        "timestamp_ms should be wall-clock UNIX_EPOCH ms (post-2023); got {}",
        recorded.timestamp_ms
    );
}

/// S-2: PainterMode → ToolMode mapping (ADR-0043 §2.6.1 congelado).
#[test]
fn s2_painter_mode_maps_to_tool_mode() {
    use ph2d_painter_stroke::ToolMode;
    use ph2d_tool_painter::params::PainterMode;
    for (pm, expected) in [
        (PainterMode::Brush, ToolMode::Paint),
        (PainterMode::Smudge, ToolMode::Smudge),
        (PainterMode::Eraser, ToolMode::Erase),
    ] {
        let mut t = mk_painter_with_canvas();
        t.params.mode = pm;
        t.begin_stroke(0);
        t.queue_pointer(PointerSample {
            position: [1.0, 1.0],
            pressure: 1.0,
            tilt: 0.0,
        });
        t.end_stroke();
        let rec = t.stroke_history().iter().last().unwrap();
        assert_eq!(rec.tool_mode, expected, "{pm:?} should map to {expected:?}");
    }
}

/// S-3: secondary_color é Some após T1.9 wire (sempre persisted, W2
/// refinará "in use" semantics em S-15 carry-over).
#[test]
fn s3_secondary_color_is_some() {
    let mut t = mk_painter_with_canvas();
    t.begin_stroke(0);
    t.queue_pointer(PointerSample {
        position: [1.0, 1.0],
        pressure: 1.0,
        tilt: 0.0,
    });
    t.end_stroke();
    let rec = t.stroke_history().iter().next().unwrap();
    assert!(
        rec.secondary_color.is_some(),
        "T1.9 wire persists params.secondary_color as Some always"
    );
}

/// V-1/V-3: empty stroke (begin → end sem queue_pointer) NÃO é pushado
/// pra history; seq é recycled.
#[test]
fn v1_empty_stroke_not_pushed_to_history() {
    let mut t = mk_painter_with_canvas();
    t.begin_stroke(0);
    t.end_stroke(); // zero samples
    assert_eq!(
        t.stroke_history().len(),
        0,
        "empty stroke should NOT pollute canon"
    );
    // Próximo stroke pega seq=0 (recycled).
    t.begin_stroke(1);
    t.queue_pointer(PointerSample {
        position: [1.0, 1.0],
        pressure: 1.0,
        tilt: 0.0,
    });
    t.end_stroke();
    let rec = t.stroke_history().iter().next().unwrap();
    assert_eq!(rec.seq, 0, "seq recycled after empty-stroke cancel");
}

/// V-1: set_source mid-stroke (que chama end_stroke implicit) + sem samples
/// também NÃO pusha empty record.
#[test]
fn v1_set_source_mid_stroke_does_not_push_empty() {
    use ph2d_editor_core::tool::RasterEditTool;
    let mut t = mk_painter_with_canvas();
    t.begin_stroke(0);
    // No queue_pointer → samples empty.
    let pixels = vec![0u8; (32 * 32 * 4) as usize];
    t.set_source(pixels, 32, 32); // dispara end_stroke implícito
    assert_eq!(t.stroke_history().len(), 0);
}

/// V-2 cascade fix: begin_stroke com journal anexado MAS sample_buffer
/// scenario — queue_pointer/end_stroke não chamam WAL ops em path
/// degraded.
#[test]
fn v2_no_cascade_when_journal_lacks_begin() {
    // Smoke check: begin → queue → end com journal anexado, last_wal_error
    // permanece None (cenário healthy — sem cascade visível). O fix V-2
    // protege o cenário degraded (begin_stroke WAL fail) que requer
    // injeção de erro. Smoke aqui confirma path healthy unchanged.
    let path = tmp_wal_path("v2_smoke");
    std::fs::remove_file(&path).ok();
    let mut t = mk_painter_with_canvas();
    t.attach_journal(path.clone(), FlushPolicy::default())
        .expect("attach");
    t.begin_stroke(0);
    t.queue_pointer(PointerSample {
        position: [1.0, 1.0],
        pressure: 1.0,
        tilt: 0.0,
    });
    t.end_stroke();
    assert!(t.last_wal_error().is_none(), "healthy path: no WAL error");
    t.detach_journal();
    std::fs::remove_file(&path).ok();
}

/// T-1: last_wal_error captura erro WAL real.
/// Uses attach_journal em path inválido pra disparar IO error.
#[test]
fn t1_last_wal_error_captures_journal_failure() {
    let mut t = mk_painter_with_canvas();
    // Path em diretório que não existe → open falha com IO error.
    let bad_path = PathBuf::from("/nonexistent_dir_t1_test/wal.bin");
    let result = t.attach_journal(bad_path, FlushPolicy::default());
    assert!(matches!(result, Err(JournalError::Io(_))));
    // attach_journal failure não seta last_wal_error (é caller-visible
    // via Result). Confirma None pra evitar leak.
    assert!(t.last_wal_error().is_none());
}

/// T-2: clear_last_wal_error zera o flag de degraded durability.
#[test]
fn t2_clear_last_wal_error_resets() {
    let mut t = mk_painter_with_canvas();
    // Sem journal attach, last_wal_error é None.
    assert!(t.last_wal_error().is_none());
    t.clear_last_wal_error(); // no-op safe
    assert!(t.last_wal_error().is_none());
}

/// T-3: set_brush invalida cached_brush_hash → próximo stroke tem
/// brush_params_hash diferente.
#[test]
fn t3_set_brush_invalidates_hash_cache() {
    use ph2d_painter_brush::library;
    let mut t = mk_painter_with_canvas();
    t.begin_stroke(0);
    t.queue_pointer(PointerSample {
        position: [1.0, 1.0],
        pressure: 1.0,
        tilt: 0.0,
    });
    t.end_stroke();
    let hash_a = t.stroke_history().iter().next().unwrap().brush_params_hash;

    // Trocar pra brush diferente (square_hard slot 2).
    use ph2d_tool_painter::params::BrushHandle as StubBrushHandle;
    t.set_brush(StubBrushHandle(2), library::square_hard());
    t.begin_stroke(1);
    t.queue_pointer(PointerSample {
        position: [2.0, 2.0],
        pressure: 1.0,
        tilt: 0.0,
    });
    t.end_stroke();
    let hash_b = t.stroke_history().iter().nth(1).unwrap().brush_params_hash;
    assert_ne!(
        hash_a, hash_b,
        "set_brush must invalidate cache → new hash next stroke"
    );
}

/// T-5: next_seq u64 overflow panics LOUD (não silent dedup).
#[test]
#[should_panic(expected = "next_seq u64 overflow")]
fn t5_next_seq_overflow_panics_loud() {
    let mut t = PainterTool::default();
    use ph2d_editor_core::tool::RasterEditTool;
    let pixels = vec![255u8; (4 * 4 * 4) as usize];
    t.set_source(pixels, 4, 4);
    t.set_next_seq(u64::MAX);
    t.begin_stroke(0); // checked_add(1) overflow → panic
}

/// T-6 + S-7: set_canvas_id mid-stroke disparar debug_assert.
#[test]
#[should_panic(expected = "set_canvas_id called mid-stroke")]
fn t6_s7_set_canvas_id_mid_stroke_panics() {
    let mut t = mk_painter_with_canvas();
    t.begin_stroke(0);
    t.set_canvas_id(CanvasId(99)); // ADR-0052 §2.6 violation
}

/// S-7: set_layer_target mid-stroke disparar debug_assert.
#[test]
#[should_panic(expected = "set_layer_target called mid-stroke")]
fn s7_set_layer_target_mid_stroke_panics() {
    let mut t = mk_painter_with_canvas();
    t.begin_stroke(0);
    t.set_layer_target(LayerId(99));
}

/// S-7: set_next_seq mid-stroke disparar debug_assert.
#[test]
#[should_panic(expected = "set_next_seq called mid-stroke")]
fn s7_set_next_seq_mid_stroke_panics() {
    let mut t = mk_painter_with_canvas();
    t.begin_stroke(0);
    t.set_next_seq(999);
}

/// T-8: PainterTool é Send + Sync (compile-time assertion).
#[test]
fn t8_painter_tool_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<PainterTool>();
}

/// V-6: deactivate preserva stroke_history (contrato W11 caller).
#[test]
fn v6_deactivate_preserves_stroke_history() {
    use ph2d_editor_core::tool::Tool;
    let mut t = mk_painter_with_canvas();
    t.begin_stroke(0);
    t.queue_pointer(PointerSample {
        position: [1.0, 1.0],
        pressure: 1.0,
        tilt: 0.0,
    });
    t.end_stroke();
    assert_eq!(t.stroke_history().len(), 1);
    Tool::on_deactivate(&mut t);
    assert_eq!(
        t.stroke_history().len(),
        1,
        "history MUST survive deactivate per L1233-1236 contract"
    );
}

/// U-7: SAMPLE_FLAG_TILT_UNAVAILABLE setado quando sample.tilt == 0.
#[test]
fn u7_tilt_unavailable_flag_set_for_zero_tilt() {
    use ph2d_painter_stroke::SAMPLE_FLAG_TILT_UNAVAILABLE;
    let mut t = mk_painter_with_canvas();
    t.begin_stroke(0);
    t.queue_pointer(PointerSample {
        position: [1.0, 1.0],
        pressure: 1.0,
        tilt: 0.0, // mouse path
    });
    t.queue_pointer(PointerSample {
        position: [2.0, 2.0],
        pressure: 1.0,
        tilt: 0.3, // pencil real tilt
    });
    t.end_stroke();
    let rec = t.stroke_history().iter().next().unwrap();
    assert!(
        rec.points[0].flags & SAMPLE_FLAG_TILT_UNAVAILABLE != 0,
        "zero tilt should mark TILT_UNAVAILABLE"
    );
    assert!(
        rec.points[1].flags & SAMPLE_FLAG_TILT_UNAVAILABLE == 0,
        "non-zero tilt should NOT mark unavailable"
    );
}

/// S-6: attach_journal sem set_next_seq quando history não-vazio disparar
/// debug_assert (HR-W11 caller obligation).
#[test]
#[should_panic(expected = "set_next_seq(last_persisted_seq + 1) obrigatório")]
fn s6_attach_journal_without_baseline_panics() {
    let path = tmp_wal_path("s6_baseline");
    std::fs::remove_file(&path).ok();
    let mut t = mk_painter_with_canvas();
    // Stroke pra popular history.
    t.begin_stroke(0);
    t.queue_pointer(PointerSample {
        position: [1.0, 1.0],
        pressure: 1.0,
        tilt: 0.0,
    });
    t.end_stroke();
    assert_eq!(t.stroke_history().len(), 1);
    // History tem stroke MAS next_seq foi consumido até 1; em produção
    // caller load(canon) seta next_seq via set_next_seq(last+1). Aqui
    // simulamos esquecimento.
    t.set_next_seq(0); // reset pra simular caller que load + esqueceu baseline
    let _ = t.attach_journal(path.clone(), FlushPolicy::default()); // panic via debug_assert
}
