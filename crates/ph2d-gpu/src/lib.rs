//! TODO: spike S0 (bootstrap skeleton).
//!
//! NOTE: este crate é a única exceção a `#![forbid(unsafe_code)]` no core.
//! `ph2d-gpu::interop::{metal,vulkan,dx12}` precisa de `wgpu::hal` que é
//! API insegura por design. Cada bloco `unsafe` deve seguir HR-2 (justificativa
//! escrita) e ADR-0008 (invariantes documentadas).
