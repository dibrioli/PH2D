//! [`AssetWatcher`] — off-thread file watcher producing
//! [`ReloadEvent`]s for the main thread to apply at the frame
//! boundary.
//!
//! Per HR-6: decode happens on the watcher thread (notify's internal
//! callback thread), so the main thread's apply step is purely
//! `BTreeMap` mutation under the write lock — never disk I/O, never
//! PNG decode. This keeps frame-time spikes invisible to the renderer
//! even when an asset reloads.
//!
//! Debouncing: notify fires multiple events for some editor save
//! patterns (atomic save = remove + create). We don't debounce
//! explicitly — apply_pending is idempotent under "last write wins",
//! and the cost of one extra decode + apply is sub-millisecond.

use crate::asset::Asset;
use crate::error::AssetError;
use crate::id::AssetId;
use crate::loader::decode_png_bytes;
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc;

/// Event delivered to the main thread for [`crate::AssetDb::apply_pending`].
///
/// `Changed` carries the fully-decoded `Asset` and pre-computed
/// `AssetId` — no work for the main thread beyond inserting into the
/// db's BTreeMaps.
#[derive(Debug)]
pub enum ReloadEvent {
    Changed {
        path: PathBuf,
        id: AssetId,
        asset: Asset,
    },
    Removed {
        path: PathBuf,
    },
    Error {
        path: PathBuf,
        error: AssetError,
    },
}

pub struct AssetWatcher {
    /// Holding the watcher alive keeps the OS-native watcher thread
    /// running. Dropping it stops watching.
    _watcher: RecommendedWatcher,
    rx: mpsc::Receiver<ReloadEvent>,
}

impl AssetWatcher {
    /// Watch `dir` recursively, decoding any `.png` file that changes
    /// on disk. Other extensions are ignored (M6 supports PNG only).
    pub fn watch_dir(dir: &Path) -> Result<Self, AssetError> {
        let (tx, rx) = mpsc::channel::<ReloadEvent>();
        let watcher_tx = tx;

        let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            let event = match res {
                Ok(e) => e,
                Err(e) => {
                    // notify-internal error — log via channel as a
                    // generic Watch error tied to no specific path.
                    let _ = watcher_tx.send(ReloadEvent::Error {
                        path: PathBuf::new(),
                        error: AssetError::Watch(e.to_string()),
                    });
                    return;
                }
            };
            handle_fs_event(event, &watcher_tx);
        })
        .map_err(|e| AssetError::Watch(e.to_string()))?;

        watcher
            .watch(dir, RecursiveMode::Recursive)
            .map_err(|e| AssetError::Watch(e.to_string()))?;

        Ok(Self {
            _watcher: watcher,
            rx,
        })
    }

    /// Drain every event currently queued without blocking. Call once
    /// per frame, then hand the result to [`crate::AssetDb::apply_pending`].
    pub fn drain(&self) -> Vec<ReloadEvent> {
        let mut out = Vec::new();
        while let Ok(ev) = self.rx.try_recv() {
            out.push(ev);
        }
        out
    }

    /// Block up to `timeout` waiting for the first event, then drain
    /// the rest non-blocking. Tests use this; production should call
    /// `drain` from the per-frame loop.
    pub fn drain_blocking(&self, timeout: std::time::Duration) -> Vec<ReloadEvent> {
        let mut out = Vec::new();
        if let Ok(first) = self.rx.recv_timeout(timeout) {
            out.push(first);
            while let Ok(ev) = self.rx.try_recv() {
                out.push(ev);
            }
        }
        out
    }
}

fn handle_fs_event(event: notify::Event, tx: &mpsc::Sender<ReloadEvent>) {
    match event.kind {
        EventKind::Create(_) | EventKind::Modify(_) => {
            for path in event.paths {
                if path.extension().and_then(|s| s.to_str()) != Some("png") {
                    continue;
                }
                let bytes = match std::fs::read(&path) {
                    Ok(b) => b,
                    Err(e) => {
                        let _ = tx.send(ReloadEvent::Error {
                            path: path.clone(),
                            error: AssetError::Io {
                                path: path.clone(),
                                source: e,
                            },
                        });
                        continue;
                    }
                };
                let id = AssetId::from_bytes(&bytes);
                match decode_png_bytes(&bytes, Some(&path)) {
                    Ok(asset) => {
                        let _ = tx.send(ReloadEvent::Changed { path, id, asset });
                    }
                    Err(e) => {
                        let _ = tx.send(ReloadEvent::Error { path, error: e });
                    }
                }
            }
        }
        EventKind::Remove(_) => {
            for path in event.paths {
                if path.extension().and_then(|s| s.to_str()) != Some("png") {
                    continue;
                }
                let _ = tx.send(ReloadEvent::Removed { path });
            }
        }
        _ => {}
    }
}
