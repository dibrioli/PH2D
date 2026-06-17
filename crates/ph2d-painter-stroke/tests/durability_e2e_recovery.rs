//! Integration test: lifecycle end-to-end de WAL + crash recovery + autosave.
//! Simula crash mid-stroke + recovery boot scan + UX flow decisions.

use std::path::PathBuf;

use ph2d_color::OklchColor;
use ph2d_painter_brush::BrushHandle;
use ph2d_painter_stroke::{
    AutoSave, AutoSavePolicy, CanvasId, CrashRecovery, FlushPolicy, PartialStroke, PersistLayerId,
    RawPointerSample, RecoveryState, StrokeJournal,
};

fn unique_tmp(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "ph2d-painter-stroke-e2e-{}-{name}.wal",
        std::process::id()
    ))
}

fn mk_partial(seq: u64, canvas: u64) -> PartialStroke {
    PartialStroke::new(
        seq,
        CanvasId(canvas),
        PersistLayerId(0),
        BrushHandle::default(),
        [0u8; 32],
        OklchColor::default(),
    )
}

#[test]
fn e2e_crash_mid_stroke_recovery_yields_in_progress_and_committed() {
    let path = unique_tmp("crash_mid");
    std::fs::remove_file(&path).ok();

    // Simula sessão: 3 strokes commitados + 1 in-progress quando crashou.
    {
        let mut j = StrokeJournal::open(path.clone(), FlushPolicy::EveryNSamples(2)).unwrap();
        for seq in 0..3u64 {
            j.begin_stroke(mk_partial(seq, 1)).unwrap();
            j.add_sample(RawPointerSample::default(), 0).unwrap();
            j.add_sample(RawPointerSample::default(), 0).unwrap();
            j.commit_stroke(0).unwrap();
        }
        // Stroke 4 em progresso — sem commit (simula crash).
        j.begin_stroke(mk_partial(3, 1)).unwrap();
        j.add_sample(RawPointerSample::default(), 0).unwrap();
        j.flush_sample_batch(0).unwrap();
        // J drop sem commit — simula process kill mid-stroke.
    }

    // Boot scan.
    let recovery = CrashRecovery::scan(path.clone()).expect("recovery scan");
    assert_eq!(recovery.corrupted_entries, 0);
    assert_eq!(recovery.recovered_strokes.len(), 4);

    // 3 committed-not-persisted (porque .ph2d-painter NÃO foi salvo entre commits).
    let committed: Vec<_> = recovery.committed_not_persisted().collect();
    assert_eq!(committed.len(), 3);
    assert_eq!(committed[0].partial.seq, 0);
    assert_eq!(committed[1].partial.seq, 1);
    assert_eq!(committed[2].partial.seq, 2);

    // 1 in-progress (crash mid-stroke).
    let in_progress: Vec<_> = recovery.in_progress_at_crash().collect();
    assert_eq!(in_progress.len(), 1);
    assert_eq!(in_progress[0].partial.seq, 3);
    assert_eq!(in_progress[0].samples.len(), 1);

    std::fs::remove_file(&path).ok();
}

#[test]
fn e2e_cancelled_stroke_is_not_recovered_as_in_progress() {
    let path = unique_tmp("cancelled");
    std::fs::remove_file(&path).ok();

    {
        let mut j = StrokeJournal::open(path.clone(), FlushPolicy::default()).unwrap();
        j.begin_stroke(mk_partial(0, 1)).unwrap();
        j.add_sample(RawPointerSample::default(), 0).unwrap();
        j.cancel_stroke().unwrap();
    }

    let recovery = CrashRecovery::scan(path.clone()).expect("scan");
    assert_eq!(recovery.recovered_strokes.len(), 1);
    assert_eq!(
        recovery.recovered_strokes[0].state,
        RecoveryState::Cancelled
    );
    assert_eq!(recovery.committed_not_persisted().count(), 0);
    assert_eq!(recovery.in_progress_at_crash().count(), 0);

    std::fs::remove_file(&path).ok();
}

#[test]
fn e2e_autosave_triggers_after_threshold() {
    // AutoSave NÃO faz I/O sozinho; aqui validamos state machine + decisão
    // de triggering.
    let mut autosave = AutoSave::new(
        CanvasId(1),
        PathBuf::from("/tmp/never-used.ph2d-painter"),
        AutoSavePolicy::EveryNStrokes(3),
    );
    assert!(!autosave.on_stroke_committed(0));
    assert!(!autosave.on_stroke_committed(0));
    assert!(autosave.on_stroke_committed(0), "3rd dispara");
    // Simula save bem-sucedido.
    autosave.mark_saving();
    autosave.mark_saved(100);
    // Counter resetado, próximos 2 não disparam.
    assert!(!autosave.on_stroke_committed(100));
    assert!(!autosave.on_stroke_committed(100));
    assert!(autosave.on_stroke_committed(100));
}

#[test]
fn e2e_journal_rotation_threshold_detected() {
    let path = unique_tmp("rotate");
    std::fs::remove_file(&path).ok();
    let mut j = StrokeJournal::open(path.clone(), FlushPolicy::default()).unwrap();
    // Não atingiu cap.
    assert!(!j.should_rotate());
    // Após 100+ commits, deve disparar.
    j.commits_since_rotate = 105;
    assert!(j.should_rotate());
    // Após reset.
    j.truncate_and_reset().unwrap();
    assert_eq!(j.commits_since_rotate, 0);
    assert!(!j.should_rotate());
    std::fs::remove_file(&path).ok();
}
