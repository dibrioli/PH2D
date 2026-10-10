//! 2D orthographic camera.
//!
//! World space is Y-up (per SKILL §11.1). The view-proj matrix
//! includes the Y-flip so the shader output matches clip space
//! (which is Y-down in wgpu/WebGPU NDC). Single source of the flip
//! per §11.1: "O flip Y-up → Y-down é aplicado uma vez na projection
//! matrix".

use ph2d_core::Mat4;
use ph2d_host::WindowSize;

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

#[derive(Copy, Clone, Debug)]
pub struct Camera2d {
    /// Camera center in world units.
    pub center: [f32; 2],
    /// World-units height of the visible region. Width is derived
    /// from window aspect. Use small values (e.g., 10.0) for "zoomed
    /// in" feel; larger for overview.
    pub height_world: f32,
    /// Visibility-layer cull mask (Sprite Inspector v2 W3.T3.12). A
    /// sprite renders for this camera only when its
    /// `ph2d_ecs::VisibilityLayer` mask intersects `cull_mask`; absence
    /// of the component = visible. CPU-only (drives the extract cull),
    /// never uploaded to `CameraUniform`. Default `u32::MAX` = every
    /// layer visible (no culling).
    pub cull_mask: u32,
    /// ⭐ **A matriz de MODELO** `[a, b, c, d, tx, ty]` — local → mundo, aplicada ANTES da vista
    /// (doc ObjectsAndComponents 01 §13: os gizmos do Motion vêem o grafo no espaço do objecto Motion
    /// em edição). Identidade = a câmara de sempre, byte a byte. ⚠️ O passe de sprites nunca a põe.
    pub model: [f32; 6],
}

impl Camera2d {
    /// Minimum `height_world` allowed by [`Camera2d::zoom`]. 0.5 m =
    /// "zoomed in enough that a half-meter sprite fills the screen
    /// vertically" — anything tighter starts to lose floating-point
    /// precision in the projection matrix.
    pub const ZOOM_MIN_HEIGHT_WORLD: f32 = 0.5;
    /// Maximum `height_world` allowed by [`Camera2d::zoom`]. 100 m =
    /// "5 city blocks visible vertically" — anything wider and the
    /// grid LOD threshold suppresses minor lines anyway.
    pub const ZOOM_MAX_HEIGHT_WORLD: f32 = 100.0;

    pub fn new(center: [f32; 2], height_world: f32) -> Self {
        Self {
            center,
            height_world,
            cull_mask: u32::MAX,
            model: Self::MODEL_IDENTITY,
        }
    }

    /// A matriz de modelo identidade.
    pub const MODEL_IDENTITY: [f32; 6] = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];

    /// ⭐ Esta câmara vista do espaço LOCAL de `model` (local → mundo): um ponto local `p` cai no ecrã
    /// onde `model·p` cairia nesta câmara.
    #[must_use]
    pub fn with_model(mut self, model: [f32; 6]) -> Self {
        self.model = model;
        self
    }

    /// `model·p`.
    fn model_point(&self, p: [f32; 2]) -> [f32; 2] {
        let m = &self.model;
        [
            m[0] * p[0] + m[2] * p[1] + m[4],
            m[1] * p[0] + m[3] * p[1] + m[5],
        ]
    }

    /// `model⁻¹·w` — o ponto local que cai em `w`; um modelo degenerado (escala 0) devolve `w`.
    fn unmodel_point(&self, w: [f32; 2]) -> [f32; 2] {
        let m = &self.model;
        let det = m[0] * m[3] - m[2] * m[1];
        if det.abs() < f32::EPSILON {
            return w;
        }
        let (x, y) = (w[0] - m[4], w[1] - m[5]);
        [(m[3] * x - m[2] * y) / det, (-m[1] * x + m[0] * y) / det]
    }

    /// A matriz de modelo como `Mat4` (para a projecção).
    fn model_mat4(&self) -> Mat4 {
        let m = &self.model;
        Mat4::from_cols_array(&[
            m[0], m[1], 0.0, 0.0, m[2], m[3], 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, m[4], m[5], 0.0, 1.0,
        ])
    }

    /// Multiply `height_world` by `factor` and clamp to
    /// `[ZOOM_MIN_HEIGHT_WORLD, ZOOM_MAX_HEIGHT_WORLD]`.
    ///
    /// `factor < 1.0` zooms in (world appears bigger); `> 1.0` zooms
    /// out (world appears smaller). For wheel input the canonical
    /// formula is `factor = 0.9.powf(wheel_dy / 16.0)` — a small
    /// per-notch step that compounds nicely under fast scrolling.
    ///
    /// `factor == 1.0` is a no-op. Non-finite factors are also no-ops
    /// (defensive against NaN coming from misconfigured trackpads).
    pub fn zoom(&mut self, factor: f32) {
        self.height_world = Self::zoomed(self.height_world, factor);
    }

    /// **A LEI do zoom, sem a câmera** — `height_world` vezes `factor`, cravado no intervalo.
    ///
    /// ⚠️ Existe porque o zoom passou a ter **dois tempos**: a roda escreve um destino e o quadro
    /// publica o vivo (`shells/desktop/src/canvas_zoom.rs`). Quem compõe o destino não tem uma
    /// `Camera2d` para mutar, e uma segunda cópia do clamp seria a segunda resposta a *"até onde
    /// se pode aproximar?"* — divergindo no dia em que um dos dois limites se mover. A
    /// [`Self::zoom`] delega, então continua a existir **uma** lei.
    #[must_use]
    pub fn zoomed(height_world: f32, factor: f32) -> f32 {
        if !factor.is_finite() || factor <= 0.0 {
            return height_world;
        }
        Self::clamp_height(height_world * factor)
    }

    /// Um entalhe da roda em pixels (`16 px` por linha).
    pub const WHEEL_NOTCH_PX: f32 = 16.0;
    /// O que um entalhe faz ao `height_world`: `0,9` (a roda para cima aproxima).
    pub const WHEEL_ZOOM_PER_NOTCH: f32 = 0.9;

    /// **A LEI da roda**: `dy_px` de roda (positivo = para cima) vira o factor do zoom.
    #[must_use]
    pub fn wheel_factor(dy_px: f32) -> f32 {
        Self::WHEEL_ZOOM_PER_NOTCH.powf(dy_px / Self::WHEEL_NOTCH_PX)
    }

    /// A inversa de [`Self::wheel_factor`]: os `dy_px` de roda que dão `factor` — por onde a pinça
    /// de dois dedos (F6) entra pela MESMA porta que a roda. Factor inválido ⇒ `0`.
    #[must_use]
    pub fn wheel_px_for(factor: f32) -> f32 {
        if !factor.is_finite() || factor <= 0.0 {
            return 0.0;
        }
        factor.ln() / Self::WHEEL_ZOOM_PER_NOTCH.ln() * Self::WHEEL_NOTCH_PX
    }

    /// **O arrasto (px) que mantém o mundo sob `at` parado quando o `height_world` vai de
    /// `h_from` a `h_to`** — o zoom à volta de um ponto, dito como um pan pela porta do pan.
    ///
    /// `centre` é o centro da janela da cena em px e `h_live` o `height_world` com que o pan será
    /// aplicado (o zoom pode ainda estar em voo: o destino muda já, o vivo chega depois). Sem
    /// voo, `h_live == h_from`. Devolve o `(dx, dy)` para [`Self::pan_screen_delta`]; a conta é a
    /// mesma nos dois eixos porque o pan já carrega o flip do Y.
    #[must_use]
    pub fn anchor_pan_px(
        at: [f32; 2],
        centre: [f32; 2],
        h_from: f32,
        h_to: f32,
        h_live: f32,
    ) -> [f32; 2] {
        if !(h_live.is_finite() && h_live > 0.0 && h_from.is_finite() && h_to.is_finite()) {
            return [0.0, 0.0];
        }
        let k = (h_to - h_from) / h_live;
        [(at[0] - centre[0]) * k, (at[1] - centre[1]) * k]
    }

    /// Pan the camera by a screen-pixel delta — interpreted as
    /// "user dragged the cursor by `(dx_px, dy_px)` and the world
    /// under the cursor should move with it" (i.e. world moves
    /// **opposite** to the cursor drag direction; the camera center
    /// moves the same direction as the cursor).
    ///
    /// The Y-flip applied in [`Camera2d::view_proj`] is honored: a
    /// drag of `(0, +10)` (cursor moves down on Y-down screen)
    /// translates the camera **down** in world (Y-up); `center.y`
    /// decreases.
    ///
    /// `window_w` / `window_h` are the current swapchain dimensions
    /// in pixels — the function needs them to derive
    /// `pixels_per_meter`.
    pub fn pan_screen_delta(&mut self, dx_px: f32, dy_px: f32, window_w: f32, window_h: f32) {
        if window_h <= 0.0 || window_w <= 0.0 {
            return;
        }
        if !dx_px.is_finite() || !dy_px.is_finite() {
            return;
        }
        let pixels_per_meter = window_h / self.height_world.max(f32::EPSILON);
        let dx_world = dx_px / pixels_per_meter;
        let dy_world = dy_px / pixels_per_meter;
        // Pan semantics: world content under the cursor follows the
        // cursor drag direction (Photoshop/Figma "hand" pan). With
        // the projection's Y-flip (`view_proj` swaps bottom/top so
        // world Y-up maps to screen Y-down NDC), the perceived
        // screen-down drag corresponds to camera UP in world coords.
        //
        // Empirical result the user validated: cursor down → camera
        // center[1] increases. X is straight (cursor right → camera
        // left in world).
        //
        // History: M14.4e shipped with `center[1] -= dy_world`
        // assuming a different projection convention; the user
        // reported Y still inverted in two consecutive builds, which
        // pointed at the actual Y-flip direction being opposite to
        // the textbook expectation. Sign reverted to `+=` here.
        self.center[0] -= dx_world;
        self.center[1] += dy_world;
    }

    /// Project a screen-pixel cursor position to a world-space point
    /// in meters. Inverse of the orthographic projection used by
    /// [`Camera2d::view_proj`]; honors the Y-flip so a cursor at the
    /// **top** of the window maps to the **largest** world Y under
    /// the camera (world is Y-up, screen is Y-down).
    ///
    /// `cursor_px` is `(x, y)` in pixels with origin at the
    /// window's top-left corner. `window` carries the current
    /// swapchain dimensions. Returns `(world_x, world_y)` in meters.
    ///
    /// Used by:
    /// - Drag-and-drop image import (M14.4e) to spawn at the drop
    ///   point instead of always at the camera center.
    /// - Sprite picking (M14.5+) to hit-test sprites under the
    ///   cursor for selection.
    /// - Future tools (eyedropper, gizmo handles) needing world-
    ///   space coords from a click event.
    pub fn screen_to_world(&self, cursor_px: (f32, f32), window: WindowSize) -> [f32; 2] {
        let w = window.width.max(1) as f32;
        let h = window.height.max(1) as f32;
        let (cx_px, cy_px) = cursor_px;
        // Cursor coords are Y-down (top of window = cy_px=0). World
        // is Y-up. Cursor at screen TOP must map to world's HIGH Y
        // (top of camera view) — so we subtract `ny * half_h` to
        // invert the Y direction during the projection inverse.
        let nx = (cx_px / w) * 2.0 - 1.0;
        let ny = (cy_px / h) * 2.0 - 1.0;
        let aspect = w / h;
        let half_h = self.height_world * 0.5;
        let half_w = half_h * aspect;
        let world_x = self.center[0] + nx * half_w;
        let world_y = self.center[1] - ny * half_h;
        if self.model == Self::MODEL_IDENTITY {
            return [world_x, world_y];
        }
        self.unmodel_point([world_x, world_y])
    }

    /// Project a world-space point (meters) to a screen-pixel position
    /// (origin at the window's top-left, Y-down). Exact inverse of
    /// [`Camera2d::screen_to_world`] — world Y-up maps so the largest
    /// world Y lands at the smallest screen Y (top of the window).
    ///
    /// Used by overlays that must align to a sprite's on-canvas
    /// footprint without a GPU readback — e.g. the Background-Removal
    /// live preview blit.
    pub fn world_to_screen(&self, world: [f32; 2], window: WindowSize) -> (f32, f32) {
        let world = if self.model == Self::MODEL_IDENTITY {
            world
        } else {
            self.model_point(world)
        };
        let w = window.width.max(1) as f32;
        let h = window.height.max(1) as f32;
        let aspect = w / h;
        let half_h = self.height_world * 0.5;
        let half_w = half_h * aspect;
        // Inverse of `screen_to_world`'s nx/ny derivation.
        let nx = (world[0] - self.center[0]) / half_w;
        let ny = (self.center[1] - world[1]) / half_h;
        let cx_px = (nx + 1.0) * 0.5 * w;
        let cy_px = (ny + 1.0) * 0.5 * h;
        (cx_px, cy_px)
    }

    /// World-meters → screen-pixel [`vello::kurbo::Affine`] — the matrix
    /// form of [`Camera2d::world_to_screen`], for callers that hand an
    /// `Affine` to a renderer (e.g. the Vector Pen bridge feeding
    /// `vello::Scene::stroke`/`fill`) instead of projecting points one at
    /// a time.
    ///
    /// Single source of the projection: derived from the SAME uniform
    /// scale `k = window.height / height_world` and Y-flip as
    /// `world_to_screen`/`screen_to_world`, so a shell can no longer carry
    /// a hand-rolled copy that silently diverges (Vector audit H5/M3). The
    /// returned affine maps `(world_x, world_y)` to `(screen_x, screen_y)`
    /// with origin at the window's top-left, Y-down.
    ///
    /// Assumes square pixels (uniform `k`); if `Camera2d` ever becomes
    /// anisotropic, this and `world_to_screen` must change together.
    pub fn world_to_screen_affine(&self, window: WindowSize) -> vello::kurbo::Affine {
        use vello::kurbo::Affine;
        let w = window.width.max(1) as f64;
        let h = window.height.max(1) as f64;
        let k = h / (self.height_world as f64).max(1e-6);
        let view = Affine::translate((w * 0.5, h * 0.5))
            * Affine::scale_non_uniform(k, -k)
            * Affine::translate((-(self.center[0] as f64), -(self.center[1] as f64)));
        if self.model == Self::MODEL_IDENTITY {
            return view;
        }
        view * Affine::new(self.model.map(f64::from))
    }

    /// Build the view-projection matrix for the given window.
    /// Standard right-handed orthographic — world Y-up maps to wgpu
    /// clip space Y-up (per WebGPU spec §3.4), which means world top
    /// renders at screen top with no extra inversion.
    ///
    /// History (M14.4e v2): an earlier swap of `bottom`/`top` here
    /// produced a Y-flip ("world Y-up → clip Y-down NDC") that
    /// caused sprites to render mirrored relative to the grid + the
    /// drag cursor — the user reported "mouse e grid descem enquanto
    /// sprites sobem". Removing the swap aligns ALL Y consumers
    /// (sprites, grid painter, screen_to_world inverse, pan delta)
    /// to the same direction, and the QUAD_STRIP UV in
    /// `crates/ph2d-render/src/sprite.rs` is restored to its
    /// pre-M14.4d mapping (world-up → tex-top, V=0) so imported
    /// images still render upright.
    pub fn view_proj(&self, window: WindowSize) -> Mat4 {
        let aspect = window.width.max(1) as f32 / window.height.max(1) as f32;
        let half_h = self.height_world * 0.5;
        let half_w = half_h * aspect;
        let cx = self.center[0];
        let cy = self.center[1];
        let proj = Mat4::orthographic_rh(
            cx - half_w,
            cx + half_w,
            cy - half_h,
            cy + half_h,
            -1.0,
            1.0,
        );
        if self.model == Self::MODEL_IDENTITY {
            return proj;
        }
        proj * self.model_mat4()
    }

    pub fn uniform(&self, window: WindowSize) -> CameraUniform {
        let m = self.view_proj(window);
        CameraUniform {
            view_proj: m.to_cols_array_2d(),
        }
    }

    /// View-projection for rendering into a **sub-rect** of the target (Motion
    /// Nodes M0.T13 — the split viewport⟂graph). The aspect comes from the
    /// sub-rect's pixel dimensions (not the full window), so when the caller
    /// pairs this with `set_viewport`/`set_scissor_rect` the scene fills the
    /// sub-rect undistorted instead of being squashed by the window aspect.
    /// Center + `height_world` are unchanged (same framing, smaller area).
    /// Passing the full window dims yields exactly [`Self::view_proj`].
    pub fn view_proj_for_subrect(&self, subrect_w_px: f32, subrect_h_px: f32) -> Mat4 {
        let aspect = subrect_w_px.max(1.0) / subrect_h_px.max(1.0);
        let half_h = self.height_world * 0.5;
        let half_w = half_h * aspect;
        let cx = self.center[0];
        let cy = self.center[1];
        let proj = Mat4::orthographic_rh(
            cx - half_w,
            cx + half_w,
            cy - half_h,
            cy + half_h,
            -1.0,
            1.0,
        );
        if self.model == Self::MODEL_IDENTITY {
            return proj;
        }
        proj * self.model_mat4()
    }

    /// [`CameraUniform`] form of [`Self::view_proj_for_subrect`].
    pub fn uniform_for_subrect(&self, subrect_w_px: f32, subrect_h_px: f32) -> CameraUniform {
        CameraUniform {
            view_proj: self
                .view_proj_for_subrect(subrect_w_px, subrect_h_px)
                .to_cols_array_2d(),
        }
    }
}

impl Default for Camera2d {
    fn default() -> Self {
        Self {
            center: [0.0, 0.0],
            height_world: 10.0,
            cull_mask: u32::MAX,
            model: Self::MODEL_IDENTITY,
        }
    }
}

#[cfg(test)]
#[path = "camera_tests.rs"]
mod tests;
