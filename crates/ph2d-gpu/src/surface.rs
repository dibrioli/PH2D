//! SurfaceContext — owns wgpu Surface + ADR-0020 recovery protocol.
//!
//! `acquire_frame()` is the **only** public way to get a renderable
//! target. It encapsulates retry / reconfigure / cascade-to-Lost
//! logic so callers never see a raw [`wgpu::SurfaceTexture`]
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
            // Prefer a NON-BLOCKING present mode so `acquire_frame()`
            // never stalls multiple vsync intervals when the render loop
            // saturates the present queue (the M5 demo bouncing-motion
            // sim animates the scene every frame, so the loop is
            // effectively continuous). Under `Fifo` that stall is the
            // measured mouse-move stutter (worst over the Hierarchy, the
            // heaviest scene); `Mailbox` (non-blocking, no tearing —
            // drops stale frames) or `Immediate` (non-blocking, may
            // tear) keep acquire from piling up. Falls back to `Fifo`
            // when the backend exposes neither (so vsync correctness is
            // preserved on platforms without a non-blocking mode).
            present_mode: [
                wgpu::PresentMode::Mailbox,
                wgpu::PresentMode::Immediate,
                wgpu::PresentMode::Fifo,
            ]
            .into_iter()
            .find(|pref| surface_caps.present_modes.contains(pref))
            .unwrap_or(surface_caps.present_modes[0]),
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        // Boot-time record of the selected present mode — confirms
        // whether the backend gave us a non-blocking mode (Mailbox /
        // Immediate, the stutter fix) or fell back to Fifo.
        eprintln!("[ph2d-gpu] surface present mode: {:?}", config.present_mode);
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

    /// The surface's active present mode.
    pub fn present_mode(&self) -> wgpu::PresentMode {
        self.config.present_mode
    }

    /// Switch the present mode at runtime (Config → Display toggle:
    /// `Fifo` for smooth vsync vs `Immediate` for non-blocking / no
    /// mouse-stutter). Reconfigures the swap chain in place; no-op when
    /// the mode is unchanged. The caller is responsible for only passing
    /// a mode the backend supports (Fifo is always supported; Immediate
    /// where exposed).
    pub fn set_present_mode(&mut self, mode: wgpu::PresentMode) {
        if self.config.present_mode == mode {
            return;
        }
        self.config.present_mode = mode;
        self.surface.configure(&self.gpu.device, &self.config);
        eprintln!("[ph2d-gpu] surface present mode → {mode:?}");
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

        // wgpu 28 returns Result<SurfaceTexture, SurfaceError> where
        // suboptimal is a field on the success variant. The variant
        // mapping below preserves the wgpu 29 semantics we previously
        // had — see ADR-0020 for the recovery state machine.
        match self.surface.get_current_texture() {
            Ok(texture) => {
                if texture.suboptimal {
                    // Render this frame, reconfigure before next acquire.
                    self.state = SurfaceState::NeedsReconfigureNext;
                } else {
                    self.state = match self.state {
                        SurfaceState::TimingOut(_) => SurfaceState::Healthy,
                        other => other,
                    };
                }
                Ok(FrameTarget::new(
                    texture,
                    self.config.format,
                    self.gpu.clone(),
                ))
            }

            Err(wgpu::SurfaceError::Outdated) => {
                self.surface.configure(&self.gpu.device, &self.config);
                match self.surface.get_current_texture() {
                    Ok(texture) => {
                        self.state = SurfaceState::Healthy;
                        Ok(FrameTarget::new(
                            texture,
                            self.config.format,
                            self.gpu.clone(),
                        ))
                    }
                    Err(_) => {
                        self.state = SurfaceState::AwaitingReconfigure;
                        Err(AcquireError::AwaitingReconfigure)
                    }
                }
            }

            Err(wgpu::SurfaceError::Timeout) => {
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

            Err(wgpu::SurfaceError::Lost) => {
                self.transients.clear();
                self.state = SurfaceState::AwaitingReconfigure;
                Err(AcquireError::AwaitingReconfigure)
            }

            Err(wgpu::SurfaceError::OutOfMemory) => Err(AcquireError::Other(
                "wgpu out-of-memory during get_current_texture".into(),
            )),

            Err(wgpu::SurfaceError::Other) => Err(AcquireError::Other(
                "wgpu generic error inside get_current_texture (caught by error scope)".into(),
            )),
        }
    }
}
