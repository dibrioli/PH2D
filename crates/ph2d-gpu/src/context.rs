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

        // Request the texture-compression families this adapter actually
        // advertises (KTX2 Fase 2 / W2.T4). wgpu only reports a feature in
        // `device.features()` if it was REQUESTED *and* granted — an
        // unrequested feature is invisible even on a capable adapter. So the
        // cooked-texture loader's `CompressionFeatureSet::best_tier()` (which
        // reads `device.features()`) would always fall to the uncompressed
        // RGBA8 `Constrained` tier unless we enable these here. Intersecting
        // the adapter's advertised features with the mask means we only ever
        // request what the adapter supports, so `request_device` cannot fail
        // on them (BC on desktop D3D12/Vulkan/Intel-Metal, ASTC/ETC2 on Apple
        // Silicon + mobile, none on a bare WebGPU adapter → RGBA8 floor).
        // Mirrors `CompressionFeatureSet::relevant_mask`.
        // TIMESTAMP_QUERY rides along when the adapter has it (it always does on
        // Metal/Vulkan/D3D12 desktops): the `pass_profiler` (PH2D_FLUID_PROFILE)
        // needs it to time GPU pass execution; granted-but-unused it costs nothing.
        let compression_features = adapter.features()
            & (wgpu::Features::TEXTURE_COMPRESSION_BC
                | wgpu::Features::TEXTURE_COMPRESSION_ASTC
                | wgpu::Features::TEXTURE_COMPRESSION_ETC2
                | wgpu::Features::TEXTURE_FORMAT_16BIT_NORM
                | wgpu::Features::TIMESTAMP_QUERY);

        // Limits::default() (desktop tier) is required by Vello's
        // compute pipelines (M11 widget paint) — downlevel_defaults
        // caps storage texture bindings and workgroup sizes too low
        // for Vello to even compile its shaders. iPad/web shells will
        // need to revisit if/when they pick a downlevel adapter.
        //
        // **ADR-0083 — 4K full-res watercolor residency.** The 32-channel wet-field
        // (`ph2d_painter_brush` `PIG_CH`, ADR-0080/0081) is 128 B/cell, so a full-res 4K grid
        // pigment buffer is ~1.06 GB — past the desktop-default `max_storage_buffer_binding_size`
        // (128 MiB). Raise the storage-buffer + total buffer-size caps to the ADAPTER's advertised
        // max (always ≥ the default → a safe superset; `request_device` can't fail on it and
        // nothing that worked breaks — it only ALLOWS bigger buffers). The resident field then
        // allocates at full-res 4K where the hardware has the VRAM (Apple Silicon unified memory,
        // modern dGPUs). Smaller devices advertise less → the field stays low-res grid (canvas/4,
        // the production default) + the perf bench skips oversized configs (no silent cap).
        let adapter_limits = adapter.limits();
        let mut required_limits = wgpu::Limits::default();
        required_limits.max_storage_buffer_binding_size = required_limits
            .max_storage_buffer_binding_size
            .max(adapter_limits.max_storage_buffer_binding_size);
        required_limits.max_buffer_size = required_limits
            .max_buffer_size
            .max(adapter_limits.max_buffer_size);
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("ph2d-gpu device"),
            required_features: compression_features,
            required_limits,
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression guard (W2.T4): every texture-compression family the adapter
    /// advertises MUST be granted on the created device, or the cooked-texture
    /// loader's `best_tier()` silently falls to the RGBA8 floor on capable
    /// hardware. `required_features: Features::empty()` (the pre-W2.T4 default)
    /// fails this. #[ignore] — needs a real adapter (no GPU on CI).
    #[test]
    #[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
    fn device_enables_every_adapter_advertised_compression_feature() {
        let mask = wgpu::Features::TEXTURE_COMPRESSION_BC
            | wgpu::Features::TEXTURE_COMPRESSION_ASTC
            | wgpu::Features::TEXTURE_COMPRESSION_ETC2
            | wgpu::Features::TEXTURE_FORMAT_16BIT_NORM;
        let Ok(gpu) = GpuContext::new(GpuContext::default_instance(), None) else {
            return; // no adapter on this machine — nothing to assert.
        };
        let adapter_caps = gpu.adapter.features() & mask;
        let device_caps = gpu.device.features() & mask;
        assert_eq!(
            device_caps, adapter_caps,
            "device must grant every adapter-advertised compression family \
             (adapter={adapter_caps:?}, device={device_caps:?}) — else best_tier() \
             silently picks RGBA8 on capable hardware"
        );
        // On any real desktop / Apple-Silicon adapter at least ONE compression
        // family is present; a bare WebGPU adapter legitimately has none.
        eprintln!("compression features granted on this device: {device_caps:?}");
    }
}
