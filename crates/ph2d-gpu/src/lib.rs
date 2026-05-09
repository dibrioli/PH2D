#![forbid(unsafe_code)]
//! ph2d-gpu — wgpu wrapper + safe surface lifecycle (M3, ADR-0020).
//!
//! This crate currently uses **only** the safe wgpu API. The
//! `interop::{metal,vulkan,dx12}` modules promised by §10.4 of the
//! SKILL (wgpu-hal interop for shells delivering native textures)
//! land in M5+; when they do, this `forbid(unsafe_code)` will be
//! replaced with `#![deny(unsafe_code)]` and each `unsafe` block
//! will follow HR-2 + ADR-0008 invariants.
//!
//! M3 surface:
//! - [`GpuContext`]: owns Instance/Adapter/Device/Queue.
//! - [`SurfaceContext`]: owns the wgpu Surface + ADR-0020 recovery
//!   protocol (`acquire_frame()` is the only path to a renderable
//!   target).
//! - [`FrameTarget`]: RAII handle for a single frame; auto-presents on
//!   drop unless the caller calls `present()` explicitly.
//! - [`TransientPool`]: skeleton for transient texture cache (depth,
//!   MSAA, intermediate render targets). M3 implements `clear()`
//!   only; full pool lands in M5 when render graph needs it.

pub mod context;
pub mod frame;
pub mod surface;
pub mod transient;

pub use context::{GpuContext, GpuError};
pub use frame::FrameTarget;
pub use surface::{AcquireError, SurfaceContext};
pub use transient::TransientPool;

/// Re-export the wgpu types that callers will need to interact with
/// the API surface here. **Don't** re-export wgpu wholesale — that
/// would force every downstream crate to track wgpu version churn
/// (Comfy post-mortem). Add types here as needed.
pub use wgpu::{Color, TextureFormat};
