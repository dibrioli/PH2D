//! Project-level settings exposed by the editor.
//!
//! Holds the configuration that applies to the whole open project /
//! scene rather than to a single entity or panel: the canonical
//! example is `pixels_per_meter`, the source-image-to-world-space
//! scale used by the import pipeline.
//!
//! Stored on [`crate::HeroScreen`] (`hero.project`); read by the
//! shell when processing image imports (M14.4c, M14.4d) so a
//! `512×512` PNG lands as a sprite of `512 / px_per_meter` world
//! meters per side instead of a hardcoded `[1.5, 1.5]`. UI surface
//! lives in the TopBar Settings cluster (gear icon).
//!
//! Per-asset overrides (Inspector slider on a selected sprite) are
//! deferred — added when there's a real case of mixing scales in a
//! single scene.

pub use ph2d_host::ImageFilterMode;
use ph2d_vector::ImageQuality;

/// Defaults — kept as `pub const` so tests + UI presets can reference
/// the canonical value without re-typing the literal.
pub const DEFAULT_PIXELS_PER_METER: f32 = 100.0;

/// Min / max accepted by the UI slider. Outside this range the import
/// math either overflows world coordinates (px/m too low → a single
/// sprite spans kilometers) or collapses sprites to sub-pixel sizes
/// (px/m too high → 4096-px sprite becomes 4 millimeters wide).
pub const MIN_PIXELS_PER_METER: f32 = 1.0;
pub const MAX_PIXELS_PER_METER: f32 = 4096.0;

/// M14.7 F: snap-to-grid step in world meters used by the gizmo's
/// Ctrl/Cmd-translate path. 0.16 m = 16 px @ default 100 px/m, a
/// reasonable pixel-art tile size. `0.0` disables snap.
pub const DEFAULT_SNAP_MOVE_METERS: f32 = 0.16;

/// M14.7 F: snap-to-angle step in degrees used by the gizmo's
/// Shift-rotate path. 15° matches Figma/Affinity convention (15°,
/// 30°, 45°, … as the user holds Shift while rotating). `0.0`
/// disables rotation snap.
pub const DEFAULT_SNAP_ROTATE_DEG: f32 = 15.0;

/// User-visible unit for length / position readouts. Sim values are
/// always stored in **meters** (the world-space canonical) — this
/// enum only changes how those values are FORMATTED for the user in
/// panels (Inspector, Grid Settings, Gizmo tooltip).
///
/// Conversion uses `ProjectSettings::pixels_per_meter`:
///   pixels = meters × pixels_per_meter
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayUnit {
    Meters,
    Pixels,
}

impl DisplayUnit {
    /// Convert a sim-stored meter value to the user-visible value in
    /// the active display unit.
    ///
    /// Delegates to [`Self::from_meters_f64`] so the RULE lives once.
    pub fn from_meters(self, meters: f32, pixels_per_meter: f32) -> f32 {
        self.from_meters_f64(f64::from(meters), pixels_per_meter) as f32
    }

    /// The same conversion at `f64` width — for callers that measure the
    /// world in `f64` (the ruler, the snap guides).
    ///
    /// ⚠️ **Two widths, ONE rule.** The narrow one delegates here rather
    /// than repeating the `match`: a second copy is what lets pixels and
    /// meters disagree in the day someone adds a third unit. And the
    /// width matters — a ruler coordinate of `1e6 m` in pixels is `1e8`,
    /// which `f32` cannot hold to the digit the label prints.
    pub fn from_meters_f64(self, meters: f64, pixels_per_meter: f32) -> f64 {
        match self {
            DisplayUnit::Meters => meters,
            DisplayUnit::Pixels => meters * f64::from(pixels_per_meter),
        }
    }

    /// Inverse of [`Self::from_meters`] — convert a value the user
    /// typed (in display unit) back to sim-space meters before
    /// writing into ECS.
    ///
    /// Delegates to [`Self::to_meters_f64`] so the RULE lives once.
    pub fn to_meters(self, value: f32, pixels_per_meter: f32) -> f32 {
        self.to_meters_f64(f64::from(value), pixels_per_meter) as f32
    }

    /// The same inverse at `f64` width — the mirror of [`Self::from_meters_f64`],
    /// and it exists for the same reason: a coordinate the artist types far from
    /// the origin (`1e8` px) does not survive a round trip through `f32`.
    ///
    /// ⚠️ **A guarda de `pixels_per_meter > 0.0` é a que impede um `inf` de entrar
    /// no documento** — a escala tem piso na UI (`MIN_PIXELS_PER_METER`), mas um
    /// arquivo de outra máquina não é obrigado a honrá-lo.
    pub fn to_meters_f64(self, value: f64, pixels_per_meter: f32) -> f64 {
        match self {
            DisplayUnit::Meters => value,
            DisplayUnit::Pixels => {
                if pixels_per_meter > 0.0 {
                    value / f64::from(pixels_per_meter)
                } else {
                    value
                }
            }
        }
    }

    /// One-character suffix to show alongside a formatted value
    /// (`"m"` / `"px"`). Used by panel painters for "1.50 m" /
    /// "150 px" style readouts.
    pub fn suffix(self) -> &'static str {
        match self {
            DisplayUnit::Meters => "m",
            DisplayUnit::Pixels => "px",
        }
    }
}

/// User-visible unit for **angle** readouts — the sibling of [`DisplayUnit`],
/// and it exists for the same reason and obeys the same law.
///
/// ⚠️ **Sim values are always stored in RADIANS** (`ph2d_ecs::Transform`:
/// *"rotation (radians, CCW from +X)"*). This enum only changes how an angle is
/// FORMATTED for the artist and PARSED back from what they typed. ⛔ Storing
/// degrees would put a conversion inside every trigonometric call and let
/// rounding accumulate in the document.
///
/// # Por que ela existe (Enio, 2026-08-30)
///
/// *"Devemos ter ambas as opções no app (px e metros, graus e radianos)."*
///
/// Metade já existia: o [`DisplayUnit`] fazia isto para COMPRIMENTO desde o doc 88.
/// A outra metade estava a um campo — o `Unit { …, Degrees, Radians, … }` do
/// `numeric_input_with_unit` já sabia mostrar e ler as duas, e ⛔ **`Unit::Radians`
/// tinha zero consumidores fora do próprio ficheiro**: o app aceitava `"2.25rad"`
/// digitado e nunca escrevia um ângulo em radianos. *Meio caminho ligado.*
///
/// # ⛔ O que ela NÃO alcança
///
/// **Só os ângulos que o artista AUTORA.** Não muda a fase de um oscilador, o ângulo
/// interno de um gradiente, nem qualquer `to_degrees()` que seja passo de geometria —
/// esses são matemática, não leitura. *Deixar a unidade escorregar para lá poria um
/// selector sobre números que não são do artista.*
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayAngle {
    Degrees,
    Radians,
}

impl DisplayAngle {
    /// Convert a sim-stored **radian** value to the user-visible value in the
    /// active display angle.
    ///
    /// ⚠️⚠️ **Aqui o estreito NÃO delega no largo, e isso DIVERGE do
    /// [`DisplayUnit`] de propósito.** Ver a nota de [`Self::to_radians`] — a
    /// divergência foi medida, não escolhida.
    #[must_use]
    pub fn from_radians(self, radians: f32) -> f32 {
        match self {
            DisplayAngle::Degrees => radians.to_degrees(),
            DisplayAngle::Radians => radians,
        }
    }

    /// The same conversion at `f64` width — para quem já mede em `f64` (o
    /// `WidgetStore` guarda os valores das caixas em `f64`).
    #[must_use]
    pub fn from_radians_f64(self, radians: f64) -> f64 {
        match self {
            DisplayAngle::Degrees => radians.to_degrees(),
            DisplayAngle::Radians => radians,
        }
    }

    /// Inverse of [`Self::from_radians`] — convert a value the user typed (in the
    /// display angle) back to sim-space radians before writing into ECS.
    ///
    /// # ⚠️⚠️ Por que este par NÃO delega, ao contrário do [`DisplayUnit`]
    ///
    /// O irmão do comprimento faz o estreito chamar o largo, e a razão dele é boa:
    /// a regra envolve um **parâmetro externo** (`pixels_per_meter`) que tem de ser
    /// alargado do mesmo modo nas duas larguras, e a magnitude é o perigo
    /// (`1e8 px` não cabe em `f32`).
    ///
    /// ⛔ **Num ângulo, delegar PIORA** — e o número é medido: a `std` já dá a
    /// regra nas duas larguras (`f32::to_radians` e `f64::to_radians`), então
    /// passar pelo `f64` no caminho estreito acrescenta uma **segunda
    /// arredondagem**. Medido sobre 11 ângulos, ida-e-volta `rad → mostrado → rad`:
    ///
    /// | volta | pior erro |
    /// |---|---|
    /// | pelo `f64` (delegando) | **1 ULP** (`-2.7182817` → `-155.74608` → falha) |
    /// | em `f32` (directa) | **0 ULP** |
    ///
    /// *Arredondar duas vezes não é arredondar melhor.* ⭐ **E o gate que apanhou
    /// isto foi escrito contra a minha própria afirmação** — a 1ª redacção deste
    /// doc dizia que o `f64` no meio era o que fazia o round-trip fechar.
    #[must_use]
    pub fn to_radians(self, value: f32) -> f32 {
        match self {
            DisplayAngle::Degrees => value.to_radians(),
            DisplayAngle::Radians => value,
        }
    }

    /// The same inverse at `f64` width — the mirror of [`Self::from_radians_f64`].
    #[must_use]
    pub fn to_radians_f64(self, value: f64) -> f64 {
        match self {
            DisplayAngle::Degrees => value.to_radians(),
            DisplayAngle::Radians => value,
        }
    }

    /// Suffix to show alongside a formatted angle (`"°"` / `" rad"`).
    ///
    /// ⚠️ **O grau NÃO leva espaço e o radiano leva** — é a convenção tipográfica
    /// (SI: o `°` é o único símbolo de unidade que se cola ao número), e é a que o
    /// Blender, o Godot e o Illustrator seguem. O sufixo do parser é outro (`"deg"`
    /// / `"rad"`, em [`crate::widget::numeric_input_with_unit::Unit`]): *o que se
    /// escreve e o que se lê não têm de ser a mesma string.*
    #[must_use]
    pub fn suffix(self) -> &'static str {
        match self {
            DisplayAngle::Degrees => "°",
            DisplayAngle::Radians => " rad",
        }
    }

    /// The matching parser/painter unit of the numeric input widget.
    ///
    /// ⭐ **Uma porta, e não um `match` em cada caixa** — é o que impede uma caixa
    /// de ângulo de mostrar graus enquanto outra mostra radianos.
    #[must_use]
    pub fn widget_unit(self) -> crate::widget::Unit {
        use crate::widget::Unit;
        match self {
            DisplayAngle::Degrees => Unit::Degrees,
            DisplayAngle::Radians => Unit::Radians,
        }
    }
}

/// Map the app-wide [`ImageFilterMode`] (defined in `ph2d-host`) onto
/// the peniko [`ImageQuality`] used by the Vello image preview
/// (`VectorScene::draw_image_rgba`). Companion of
/// `ph2d_render::wgpu_filter`, which maps the SAME enum onto the wgpu
/// sprite sampler — keeping the on-canvas BG-Removal preview consistent
/// with the baked sprite.
///
/// - `PixelArt` → [`ImageQuality::Low`]  (nearest-neighbor, crisp)
/// - `Smooth`   → [`ImageQuality::High`] (bicubic, smooth)
pub fn image_quality_for(mode: ImageFilterMode) -> ImageQuality {
    match mode {
        ImageFilterMode::PixelArt => ImageQuality::Low,
        ImageFilterMode::Smooth => ImageQuality::High,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProjectSettings {
    /// Source-image pixels per world meter. Default `100.0` matches
    /// Godot's convention: a `64×64` PNG becomes a `0.64 × 0.64 m`
    /// sprite in world space. Increase for HD 2D (256+ for hand-drawn
    /// 1080p assets); decrease for pixel art with large tile-to-meter
    /// ratios (16–32 for classic tile-based games).
    pub pixels_per_meter: f32,
    /// M14.7 F: snap step in world meters for Ctrl/Cmd-modified
    /// gizmo translate. Setting to `0.0` disables the snap (the
    /// modifier becomes a no-op). Configurable per-project so a
    /// pixel-art project can pin to 16/100 = 0.16 while a HD scene
    /// can crank it to 1.0 (= 1 m grid).
    pub snap_move_meters: f32,
    /// M14.7 F: snap step in degrees for Shift-modified gizmo
    /// rotate. `0.0` disables the snap.
    pub snap_rotate_deg: f32,
    /// User-visible unit for length / position readouts. Sim storage
    /// is always meters; this flips the FORMAT in the Inspector, the Grid
    /// Settings, the gizmo readouts and — since doc 88 — every `Length`
    /// param row of the Motion panel, between "m" and "px".
    ///
    /// Default **`Pixels`**: the app ends up all in pixels (Enio), and the
    /// artist's ruler is the one the app answers in. The store stays metric
    /// either way, which is what keeps the cook's fingerprint out of this
    /// setting's reach.
    pub display_unit: DisplayUnit,
    /// User-visible unit for **angle** readouts (Settings → "Angle unit"). Sim
    /// storage is always radians; this flips the FORMAT of the angles the artist
    /// authors — a rotação e o skew do Inspector, e as linhas de ângulo dos
    /// painéis.
    ///
    /// Default **`Degrees`**: é o que o Unity, o Godot e o Blender mostram por
    /// omissão, e era o que este app fazia com a conversão escrita à mão em cada
    /// sítio. ⭐ **O default preserva o comportamento anterior ao bit** — quem
    /// nunca abrir o menu não vê diferença nenhuma.
    pub display_angle: DisplayAngle,
    /// App-wide image sampling mode (Config → "Image filter"). The
    /// editor stores it so the Settings submenu can show the active
    /// pick with a checkmark; the SHELL is the source of truth for the
    /// GPU sampler state and drives it via `EditorAction::SetImageFilter`
    /// then `SpriteRenderer::set_filter_mode`. Default **`Smooth`**.
    pub image_filter: ImageFilterMode,
}

impl ProjectSettings {
    /// Replace `pixels_per_meter`, clamping to the supported range so
    /// the UI can wire a NumberInput without re-implementing bounds
    /// checks. Returns the clamped value the field actually took.
    pub fn set_pixels_per_meter(&mut self, value: f32) -> f32 {
        let clamped = value.clamp(MIN_PIXELS_PER_METER, MAX_PIXELS_PER_METER);
        self.pixels_per_meter = clamped;
        clamped
    }
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            pixels_per_meter: DEFAULT_PIXELS_PER_METER,
            snap_move_meters: DEFAULT_SNAP_MOVE_METERS,
            snap_rotate_deg: DEFAULT_SNAP_ROTATE_DEG,
            display_unit: DisplayUnit::Pixels,
            display_angle: DisplayAngle::Degrees,
            image_filter: ImageFilterMode::Smooth,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_godot_convention() {
        assert_eq!(ProjectSettings::default().pixels_per_meter, 100.0);
    }

    #[test]
    fn set_pixels_per_meter_clamps_below_min() {
        let mut s = ProjectSettings::default();
        let returned = s.set_pixels_per_meter(0.0);
        assert_eq!(returned, MIN_PIXELS_PER_METER);
        assert_eq!(s.pixels_per_meter, MIN_PIXELS_PER_METER);
    }

    #[test]
    fn set_pixels_per_meter_clamps_above_max() {
        let mut s = ProjectSettings::default();
        let returned = s.set_pixels_per_meter(10_000.0);
        assert_eq!(returned, MAX_PIXELS_PER_METER);
        assert_eq!(s.pixels_per_meter, MAX_PIXELS_PER_METER);
    }

    #[test]
    fn set_pixels_per_meter_passes_through_valid_values() {
        let mut s = ProjectSettings::default();
        for v in [1.0, 16.0, 32.0, 100.0, 256.0, 1024.0] {
            assert_eq!(s.set_pixels_per_meter(v), v);
            assert_eq!(s.pixels_per_meter, v);
        }
    }

    #[test]
    fn display_unit_round_trips_through_meters() {
        // Sim values are meters → to_meters(from_meters(m)) == m for
        // either unit, so the user can flip the toggle without losing
        // precision.
        for unit in [DisplayUnit::Meters, DisplayUnit::Pixels] {
            for m in [0.0, 0.16, 1.5, 64.0] {
                let displayed = unit.from_meters(m, 100.0);
                let back = unit.to_meters(displayed, 100.0);
                assert!(
                    (m - back).abs() < 1e-4,
                    "round trip failed for {unit:?} @ m={m}: displayed={displayed}, back={back}"
                );
            }
        }
    }

    #[test]
    fn display_unit_pixels_conversion_matches_pixels_per_meter() {
        // 1 m at 100 px/m → 100 px.
        assert_eq!(DisplayUnit::Pixels.from_meters(1.0, 100.0), 100.0);
        // 1 m at 32 px/m → 32 px.
        assert_eq!(DisplayUnit::Pixels.from_meters(1.0, 32.0), 32.0);
        // Meters mode ignores the px_per_m parameter.
        assert_eq!(DisplayUnit::Meters.from_meters(1.5, 999.0), 1.5);
    }

    #[test]
    fn display_unit_suffix() {
        assert_eq!(DisplayUnit::Meters.suffix(), "m");
        assert_eq!(DisplayUnit::Pixels.suffix(), "px");
    }

    #[test]
    fn default_display_unit_is_pixels() {
        assert_eq!(ProjectSettings::default().display_unit, DisplayUnit::Pixels);
    }

    #[test]
    fn default_image_filter_is_smooth() {
        assert_eq!(
            ProjectSettings::default().image_filter,
            ImageFilterMode::Smooth
        );
    }
}
