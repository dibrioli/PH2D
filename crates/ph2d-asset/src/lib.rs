#![forbid(unsafe_code)]
//! ph2d-asset — content-addressed asset cache + off-thread hot reload (M6, HR-6).
//!
//! Three layers:
//! - [`AssetId`] is the blake3 hash of the raw asset bytes. Renaming
//!   the file on disk does NOT invalidate the id (path is metadata,
//!   not identity).
//! - [`AssetDb`] is the in-process cache. Sync API. Internally guarded
//!   by an `RwLock` so the watcher thread can deliver swaps without
//!   the renderer ever blocking on disk I/O.
//! - [`AssetWatcher`] runs the file watcher on its own thread (via
//!   `notify`), reads + decodes changed PNG files there, and posts
//!   [`ReloadEvent`]s into an `mpsc` channel. The render thread drains
//!   the channel and applies the batch atomically at the frame
//!   boundary via [`AssetDb::apply_pending`].
//!
//! Decoding is intentionally on the watcher thread (HR-6 "swap atomic
//! em <frame> boundary"): the main thread's apply step is just a few
//! `BTreeMap` inserts under the write lock — sub-microsecond, never
//! touches the disk, never decodes a PNG.
//!
//! M6 explicitly does NOT include: LRU eviction (HR-13 budget
//! enforcement lands when M10 physics + M11 vector start producing
//! real memory pressure), streaming decode, mipmap generation,
//! JPEG/WebP support.

pub mod asset;
pub mod db;
pub mod error;
pub mod id;
mod loader;
pub mod watcher;

pub use asset::Asset;
pub use db::AssetDb;
pub use error::AssetError;
pub use id::AssetId;
pub use watcher::{AssetWatcher, ReloadEvent};
