//! GpuContext — owns wgpu Instance/Adapter/Device/Queue.
//!
//! Created once at app startup. Cloned/borrowed by the SurfaceContext
//! and (later) by render subsystems. Construction is the only place
//! we hit async wgpu APIs; we use `pollster::block_on` to keep the
//! rest of the core sync (per SKILL §10.1: "async morre na fronteira
//! da shell"; pollster is the approved sync runtime).

use std::sync::Arc;

#[derive(Debug)]
pub enum GpuError {
    NoAdapter(String),
    DeviceRequest(String),
}

impl std::fmt::Display for GpuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoAdapter(s) => write!(f, "no compatible GPU adapter found: {s}"),
            Self::DeviceRequest(s) => write!(f, "wgpu device request failed: {s}"),
        }
    }
}

impl std::error::Error for GpuError {}

/// Holder for the long-lived wgpu objects. Cheap to clone (`Arc`
/// internally on Device + Queue; Instance and Adapter are owned).
#[derive(Clone)]
pub struct GpuContext {
    pub instance: wgpu::Instance,
    pub adapter: Arc<wgpu::Adapter>,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
}

impl GpuContext {
    /// Build a `GpuContext` compatible with the provided surface
    /// target. Must be called after the OS window exists (the surface
    /// target's window handle is needed during adapter selection so
    /// we pick an adapter that can render to it).
    pub fn new(
        instance: wgpu::Instance,
        compatible_surface: Option<&wgpu::Surface<'_>>,
    ) -> Result<Self, GpuError> {
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface,
            force_fallback_adapter: false,
        }))
        .map_err(|e| GpuError::NoAdapter(format!("{e:?}")))?;

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("ph2d-gpu device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::downlevel_defaults(),
            experimental_features: wgpu::ExperimentalFeatures::default(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
        }))
        .map_err(|e| GpuError::DeviceRequest(format!("{e:?}")))?;

        Ok(Self {
            instance,
            adapter: Arc::new(adapter),
            device: Arc::new(device),
            queue: Arc::new(queue),
        })
    }

    /// Default Instance with PRIMARY backends (Metal/Vulkan/D3D12/WebGPU).
    pub fn default_instance() -> wgpu::Instance {
        wgpu::Instance::default()
    }
}
