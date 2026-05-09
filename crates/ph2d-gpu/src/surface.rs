//! SurfaceContext — owns wgpu Surface + ADR-0020 recovery protocol.
//!
//! `acquire_frame()` is the **only** public way to get a renderable
//! target. It encapsulates retry / reconfigure / cascade-to-Lost
//! logic so callers never see a raw [`wgpu::CurrentSurfaceTexture`]
//! variant. See
//! [ADR-0020](../../../docs/architecture/decisions/0020-surface-lifecycle.md)
//! for the full per-variant protocol.

use crate::context::GpuContext;
use crate::frame::FrameTarget;
use crate::transient::TransientPool;
use ph2d_host::WindowSize;

/// Errors propagated to the caller of `acquire_frame()`. Variants
/// the protocol can recover from internally are NOT here.
#[derive(Debug)]
pub enum AcquireError {
    /// Surface in [`SurfaceState::AwaitingReconfigure`] — caller
    /// must call [`SurfaceContext::reconfigure_after_lost`] before
    /// the next acquire.
    AwaitingReconfigure,
    /// Window occluded (minimized / behind another). Caller skips
    /// frame and tries again.
    Occluded,
    /// Single-frame timeout (count < 3). Caller skips frame; counter
    /// at the SurfaceContext level cascades to Lost at 3.
    Timeout,
    /// Catch-all for unexpected wgpu errors. Caller logs + bails.
    Other(String),
}

impl std::fmt::Display for AcquireError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AwaitingReconfigure => write!(
                f,
                "surface awaiting reconfigure after Lost — call reconfigure_after_lost()"
            ),
            Self::Occluded => write!(f, "window occluded; skip frame and try again"),
            Self::Timeout => write!(f, "frame timed out; skip and try again"),
            Self::Other(s) => write!(f, "surface acquire failed: {s}"),
        }
    }
}

impl std::error::Error for AcquireError {}

/// Internal state machine for the recovery protocol. Public for
/// inspection and unit tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceState {
    Healthy,
    /// `Suboptimal` returned last frame; render this frame, then
    /// reconfigure before the next acquire.
    NeedsReconfigureNext,
    /// `Lost` returned — reconfigure required before any acquire.
    AwaitingReconfigure,
    /// `Timeout` keeps a counter; ≥ 3 cascades to `AwaitingReconfigure`.
    TimingOut(u32),
}

pub struct SurfaceContext {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    gpu: GpuContext,
    state: SurfaceState,
    transients: TransientPool,
}

impl SurfaceContext {
    pub fn new(
        gpu: GpuContext,
        surface: wgpu::Surface<'static>,
        size: WindowSize,
    ) -> Result<Self, String> {
        let surface_caps = surface.get_capabilities(&gpu.adapter);
        let format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or_else(|| surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: surface_caps
                .present_modes
                .iter()
                .copied()
                .find(|m| *m == wgpu::PresentMode::Fifo)
                .unwrap_or(surface_caps.present_modes[0]),
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&gpu.device, &config);

        Ok(Self {
            surface,
            config,
            gpu,
            state: SurfaceState::Healthy,
            transients: TransientPool::new(),
        })
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.config.format
    }
    pub fn size(&self) -> WindowSize {
        WindowSize::new(self.config.width, self.config.height)
    }
    pub fn state(&self) -> SurfaceState {
        self.state
    }
    pub fn gpu(&self) -> &GpuContext {
        &self.gpu
    }

    /// Apply a coalesced resize. Caller (shell) holds the latest size
    /// and calls this once per frame, not per OS event (per LLM1
    /// audit + plan: "resize não-coalescido" anti-pattern).
    pub fn resize(&mut self, size: WindowSize) {
        if size.width == self.config.width && size.height == self.config.height {
            return;
        }
        self.config.width = size.width.max(1);
        self.config.height = size.height.max(1);
        self.surface.configure(&self.gpu.device, &self.config);
    }

    /// After a `Lost` cascade, the caller must invoke this before the
    /// next `acquire_frame()`. Drops transients + reconfigures the
    /// surface (per ADR-0020).
    pub fn reconfigure_after_lost(&mut self) {
        self.transients.clear();
        self.surface.configure(&self.gpu.device, &self.config);
        self.state = SurfaceState::Healthy;
    }

    /// Proactive cleanup on `Lifecycle::Background` (mobile-specific).
    /// ADR-0020 § "Background→foreground (mobile-specific)".
    pub fn on_background(&mut self) {
        self.transients.clear();
        self.state = SurfaceState::AwaitingReconfigure;
    }

    /// Acquire a `FrameTarget` for the current frame. Encapsulates
    /// the entire ADR-0020 protocol.
    pub fn acquire_frame(&mut self) -> Result<FrameTarget, AcquireError> {
        if self.state == SurfaceState::AwaitingReconfigure {
            return Err(AcquireError::AwaitingReconfigure);
        }
        if matches!(self.state, SurfaceState::NeedsReconfigureNext) {
            self.surface.configure(&self.gpu.device, &self.config);
            self.state = SurfaceState::Healthy;
        }

        // wgpu 29 returns CurrentSurfaceTexture (an enum with
        // success+failure variants); see ADR-0020 for the full
        // recovery state machine.
        match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => {
                self.state = match self.state {
                    SurfaceState::TimingOut(_) => SurfaceState::Healthy,
                    other => other,
                };
                Ok(FrameTarget::new(
                    texture,
                    self.config.format,
                    self.gpu.clone(),
                ))
            }

            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => {
                // Render this frame, reconfigure before next acquire.
                self.state = SurfaceState::NeedsReconfigureNext;
                Ok(FrameTarget::new(
                    texture,
                    self.config.format,
                    self.gpu.clone(),
                ))
            }

            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.gpu.device, &self.config);
                match self.surface.get_current_texture() {
                    wgpu::CurrentSurfaceTexture::Success(texture) => {
                        self.state = SurfaceState::Healthy;
                        Ok(FrameTarget::new(
                            texture,
                            self.config.format,
                            self.gpu.clone(),
                        ))
                    }
                    _ => {
                        self.state = SurfaceState::AwaitingReconfigure;
                        Err(AcquireError::AwaitingReconfigure)
                    }
                }
            }

            wgpu::CurrentSurfaceTexture::Timeout => {
                let count = match self.state {
                    SurfaceState::TimingOut(n) => n + 1,
                    _ => 1,
                };
                if count >= 3 {
                    self.transients.clear();
                    self.state = SurfaceState::AwaitingReconfigure;
                    Err(AcquireError::AwaitingReconfigure)
                } else {
                    self.state = SurfaceState::TimingOut(count);
                    Err(AcquireError::Timeout)
                }
            }

            wgpu::CurrentSurfaceTexture::Occluded => Err(AcquireError::Occluded),

            wgpu::CurrentSurfaceTexture::Lost => {
                self.transients.clear();
                self.state = SurfaceState::AwaitingReconfigure;
                Err(AcquireError::AwaitingReconfigure)
            }

            wgpu::CurrentSurfaceTexture::Validation => Err(AcquireError::Other(
                "wgpu validation error inside get_current_texture (caught by error scope)".into(),
            )),
        }
    }
}
