//! Typography scale tokens. Source: `docs/design/tokens.json` →
//! `typography.*`.
//!
//! Pixel sizes assume 1.0 device pixel ratio; widget code multiplies
//! by `dpr` at render time. OS text-scaling override (up to 200 %) is
//! honored at the same step.
//!
//! Canonical families: **Inter** (variable sans, with `Inter Display`
//! for titles) + **JetBrains Mono** (mono). Fallbacks below follow
//! the design stack (Apple → BlinkMacSystemFont → system-ui).

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TypeToken {
    /// `xxs` — 10 px (badges, micro-tooltips).
    Xxs,
    /// `xs` — 11 px (status bar, mini chips).
    Xs,
    /// `sm` — 12 px (sidebar labels, dense lists).
    Sm,
    /// `base` — 13 px (default body, inputs).
    Base,
    /// `md` — 15 px (panel section headers).
    Md,
    /// `lg` — 18 px (page subheaders).
    Lg,
    /// `xl` — 24 px (page headers).
    Xl,
    /// `xl2` (`2xl` in JSON) — 32 px (section heroes).
    Xl2,
    /// `xl3` (`3xl` in JSON) — 44 px (welcome screen hero).
    Xl3,
}

impl TypeToken {
    pub const fn px(self) -> f32 {
        match self {
            Self::Xxs => crate::generated::TYPOGRAPHY_SIZE_XXS,
            Self::Xs => crate::generated::TYPOGRAPHY_SIZE_XS,
            Self::Sm => crate::generated::TYPOGRAPHY_SIZE_SM,
            Self::Base => crate::generated::TYPOGRAPHY_SIZE_BASE,
            Self::Md => crate::generated::TYPOGRAPHY_SIZE_MD,
            Self::Lg => crate::generated::TYPOGRAPHY_SIZE_LG,
            Self::Xl => crate::generated::TYPOGRAPHY_SIZE_XL,
            Self::Xl2 => crate::generated::TYPOGRAPHY_SIZE_XL2,
            Self::Xl3 => crate::generated::TYPOGRAPHY_SIZE_XL3,
        }
    }

    /// Token id (matches JSON key).
    pub const fn id(self) -> &'static str {
        match self {
            Self::Xxs => "xxs",
            Self::Xs => "xs",
            Self::Sm => "sm",
            Self::Base => "base",
            Self::Md => "md",
            Self::Lg => "lg",
            Self::Xl => "xl",
            Self::Xl2 => "2xl",
            Self::Xl3 => "3xl",
        }
    }
}

/// Font weight tokens (matches JSON `weight.*`).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FontWeight {
    Regular,
    Medium,
    Semibold,
    Bold,
}

impl FontWeight {
    pub const fn value(self) -> u16 {
        match self {
            Self::Regular => crate::generated::TYPOGRAPHY_WEIGHT_REGULAR,
            Self::Medium => crate::generated::TYPOGRAPHY_WEIGHT_MEDIUM,
            Self::Semibold => crate::generated::TYPOGRAPHY_WEIGHT_SEMIBOLD,
            Self::Bold => crate::generated::TYPOGRAPHY_WEIGHT_BOLD,
        }
    }
}

/// Line-height multipliers (matches JSON `line.*`).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LineHeight {
    Tight,
    Snug,
    Normal,
    Loose,
}

impl LineHeight {
    pub const fn ratio(self) -> f32 {
        match self {
            Self::Tight => crate::generated::TYPOGRAPHY_LINE_TIGHT,
            Self::Snug => crate::generated::TYPOGRAPHY_LINE_SNUG,
            Self::Normal => crate::generated::TYPOGRAPHY_LINE_NORMAL,
            Self::Loose => crate::generated::TYPOGRAPHY_LINE_LOOSE,
        }
    }
}

/// Letter-spacing tokens (matches JSON `track.*`).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LetterSpacing {
    Tight,
    Normal,
    Wide,
    Caps,
}

impl LetterSpacing {
    /// Em units (matches CSS `letter-spacing: Xem`).
    pub const fn em(self) -> f32 {
        match self {
            Self::Tight => crate::generated::TYPOGRAPHY_TRACK_TIGHT,
            Self::Normal => crate::generated::TYPOGRAPHY_TRACK_NORMAL,
            Self::Wide => crate::generated::TYPOGRAPHY_TRACK_WIDE,
            Self::Caps => crate::generated::TYPOGRAPHY_TRACK_CAPS,
        }
    }
}

/// Font family chains. Strings constantes para uso direto em parley
/// font stack lookups (parley aceita comma-separated fallback chain).
pub const FONT_SANS: &str =
    "Inter, -apple-system, BlinkMacSystemFont, 'SF Pro Text', system-ui, sans-serif";
pub const FONT_DISPLAY: &str = "'Inter Display', Inter, system-ui, sans-serif";
pub const FONT_MONO: &str = "'JetBrains Mono', ui-monospace, 'SF Mono', Menlo, monospace";

/// Text-rendering strategy. Orthogonal to [`crate::Theme`] — qualquer
/// combinação Theme × TextRendering é válida. Lida via thread-local
/// `paint::text_rendering()` dentro de `paint_text*` em
/// `ph2d-editor-core`.
///
/// Três presets:
///
/// - `Default` — pipeline histórico: snap-Y only, sem snap-X, hint=true,
///   sem boost.
/// - `CrispHeavy` — snap-X integral + boost +300/+200/+150
///   (Medium 500 → ExtraBold 800 a 11 px) **com hint OFF**: deixa o
///   eixo variable wght fluir sem quantização → letras visivelmente
///   mais cheias, qualidade "Linear/Notion-pro".
/// - `CrispHeavyPlus` — variação experimental de CrispHeavy (2026-05-25
///   "tempero final"): mesmo boost + hint=off de CrispHeavy, mas com
///   **half-pixel snap-X** (preserva ~50 % do kerning), **letter-spacing
///   -0.01 em em corpos ≤16 px** (aperta a densidade que ExtraBold abre)
///   e **MSAA16 no Vello pass** (em vez de AaConfig::Area). Pensada
///   para A/B test contra `CrispHeavy` — se ficar perceptualmente
///   melhor, promove pra default e descarta o canonical `CrispHeavy`.
///
/// Adicionar preset = (a) novo variant aqui, (b) caso novo em
/// [`Self::params`] / [`Self::id`] / [`Self::display_name`] / [`Self::next`].
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum TextRendering {
    /// Pipeline histórico — Vello AA analítico + snap-Y, sem boost,
    /// sem snap-X.
    #[default]
    Default,
    /// Snap-X + hint **OFF** + boost +300/+200/+150 → ExtraBold @ 11 px,
    /// axis variable flowing sem quantização do autohinter.
    CrispHeavy,
    /// Experimental: CrispHeavy + half-pixel snap-X + letter-spacing
    /// -0.01em em corpos ≤16 px + MSAA16 no Vello pass. A/B vs CrispHeavy.
    CrispHeavyPlus,
}

/// Estratégia de snap horizontal do glyph origin. Trade-off entre
/// preservar kerning subpixel (None) e alinhar stems ao pixel grid (Full).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SnapX {
    /// Sem snap — `g.x` fica fracionário. Preserva 100 % do kerning.
    None,
    /// Snap a 0.5 px — preserva ~50 % do kerning, ganha ~80 % do snap.
    Half,
    /// Snap a 1 px (inteiro) — perde kerning subpixel, stems alinhados.
    Full,
}

/// Parâmetros derivados de [`TextRendering`] consumidos pelo painter
/// de texto e pelo `effective_weight` em `ph2d-text`. Cada preset
/// expõe o mesmo formato → caller é independente do número de variants.
// PartialEq only (not Eq) because `letter_spacing_em_dense: f32` is
// not `Eq` (float NaN semantics). Compare via `==` is fine for our
// purposes; we never use TextRenderingParams as a HashMap key.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TextRenderingParams {
    /// Weight bump (CSS units) aplicado a corpos ≤12 px.
    pub weight_boost_body: u16,
    /// Weight bump aplicado em 12 < size ≤ 16 px (dense lists).
    pub weight_boost_dense: u16,
    /// Weight bump aplicado em 16 < size ≤ 20 px (mid headings).
    pub weight_boost_mid: u16,
    /// Estratégia de snap horizontal da origem do glyph.
    pub snap_x: SnapX,
    /// Quando `true`, passa `hint(true)` para o Vello — autohinter
    /// skrifa snapa stems ao pixel grid (mais "crisp" mas QUANTIZA
    /// diferenças de wght a 11-12 px: presets que diferem por <1 px
    /// de massa de stem ficam visualmente idênticos pós-hint).
    /// Quando `false`, deixa o eixo wght variable fluir livremente
    /// — strokes ficam ligeiramente mais "soft" mas a variação entre
    /// presets é visível.
    pub hint: bool,
    /// Letter-spacing (em ems) aplicado a corpos ≤16 px. Negativo
    /// aperta a densidade horizontal — útil em presets com ExtraBold
    /// para compensar o "abrir" natural do weight. `0.0` = sem ajuste.
    pub letter_spacing_em_dense: f32,
    /// Quando `true`, o Vello pass usa `AaConfig::Msaa16` em vez de
    /// `AaConfig::Area`. Trade-off: mais amostras por pixel (edges
    /// de glyph com hint=off ficam mais suaves) mas pode stipplar
    /// strokes vetoriais finos. Read pelo shell antes de chamar
    /// `vello_pass.render_to_intermediate`.
    pub prefer_msaa: bool,
}

/// Tier limits para [`crisp_weight_boost_for`] (px). Discrete tiers
/// evitam micro-shifts visíveis no Inter variable weight axis.
const CRISP_BOOST_TIER_BODY_MAX: f32 = 12.0;
const CRISP_BOOST_TIER_DENSE_MAX: f32 = 16.0;
const CRISP_BOOST_TIER_MID_MAX: f32 = 20.0;

impl TextRendering {
    /// Cycle entre as opções (toggle do menu).
    pub fn next(self) -> Self {
        match self {
            Self::Default => Self::CrispHeavy,
            Self::CrispHeavy => Self::CrispHeavyPlus,
            Self::CrispHeavyPlus => Self::Default,
        }
    }

    /// Stable identifier (matches future tokens.json key).
    pub fn id(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::CrispHeavy => "crisp_heavy",
            Self::CrispHeavyPlus => "crisp_heavy_plus",
        }
    }

    /// Human-readable display name (menu items).
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::CrispHeavy => "Crisp Heavy",
            Self::CrispHeavyPlus => "Crisp Heavy +",
        }
    }

    /// Parâmetros do preset (boost tiers + snap-X flag). Único lugar
    /// onde cada preset declara seu shape — `effective_weight` e
    /// `paint_text*` consomem isso sem conhecer os variants
    /// individualmente.
    pub const fn params(self) -> TextRenderingParams {
        match self {
            Self::Default => TextRenderingParams {
                weight_boost_body: 0,
                weight_boost_dense: 0,
                weight_boost_mid: 0,
                snap_x: SnapX::None,
                hint: true,
                letter_spacing_em_dense: 0.0,
                prefer_msaa: false,
            },
            Self::CrispHeavy => TextRenderingParams {
                weight_boost_body: 300,
                weight_boost_dense: 200,
                weight_boost_mid: 150,
                snap_x: SnapX::Full,
                // hint=false aqui é deliberado: a 11-12 px o autohinter
                // do skrifa colapsa diferenças de wght <1 px ao mesmo
                // pixel grid; desligar libera o eixo variable a fluir
                // → CrispHeavy fica visualmente "pro" (ExtraBold real).
                hint: false,
                letter_spacing_em_dense: 0.0,
                prefer_msaa: false,
            },
            Self::CrispHeavyPlus => TextRenderingParams {
                // Mesmo boost de CrispHeavy.
                weight_boost_body: 300,
                weight_boost_dense: 200,
                weight_boost_mid: 150,
                // Half-pixel snap — preserva ~50 % do kerning vs Full.
                snap_x: SnapX::Half,
                hint: false,
                // Aperta densidade dos corpos pequenos compensando o
                // "abrir" do ExtraBold.
                letter_spacing_em_dense: -0.01,
                // MSAA16 no Vello pass — mais amostras por pixel,
                // edges de glyph (com hint=off) mais suaves.
                prefer_msaa: true,
            },
        }
    }
}

/// FontWeight boost (CSS units) dado os params do preset + tamanho
/// renderizado. Returns 0 acima de 20 px (boost não traz ganho
/// perceptual nesse range, independente do preset).
///
/// Caller soma o resultado ao [`FontWeight`] nominal e clampa a
/// `[100, 900]` (skrifa também enforça downstream).
pub const fn crisp_weight_boost_for(params: TextRenderingParams, font_size_px: f32) -> u16 {
    if font_size_px <= CRISP_BOOST_TIER_BODY_MAX {
        params.weight_boost_body
    } else if font_size_px <= CRISP_BOOST_TIER_DENSE_MAX {
        params.weight_boost_dense
    } else if font_size_px <= CRISP_BOOST_TIER_MID_MAX {
        params.weight_boost_mid
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_scale_strictly_increasing() {
        let scale = [
            TypeToken::Xxs,
            TypeToken::Xs,
            TypeToken::Sm,
            TypeToken::Base,
            TypeToken::Md,
            TypeToken::Lg,
            TypeToken::Xl,
            TypeToken::Xl2,
            TypeToken::Xl3,
        ];
        for w in scale.windows(2) {
            assert!(w[0].px() < w[1].px(), "{:?} → {:?}", w[0], w[1]);
        }
    }

    #[test]
    fn weight_values_match_css_standard() {
        assert_eq!(FontWeight::Regular.value(), 400);
        assert_eq!(FontWeight::Bold.value(), 700);
    }

    #[test]
    fn line_height_ratios_in_typography_norm() {
        assert!(LineHeight::Tight.ratio() < LineHeight::Normal.ratio());
        assert!(LineHeight::Normal.ratio() < LineHeight::Loose.ratio());
    }

    #[test]
    fn text_rendering_default_is_default_variant() {
        assert_eq!(TextRendering::default(), TextRendering::Default);
    }

    #[test]
    fn text_rendering_cycles_three_states() {
        let a = TextRendering::Default;
        let b = a.next();
        let c = b.next();
        let d = c.next();
        assert_eq!(b, TextRendering::CrispHeavy);
        assert_eq!(c, TextRendering::CrispHeavyPlus);
        assert_eq!(d, TextRendering::Default);
    }

    #[test]
    fn text_rendering_ids_stable() {
        assert_eq!(TextRendering::Default.id(), "default");
        assert_eq!(TextRendering::CrispHeavy.id(), "crisp_heavy");
        assert_eq!(TextRendering::CrispHeavyPlus.id(), "crisp_heavy_plus");
    }

    #[test]
    fn text_rendering_display_names() {
        assert_eq!(TextRendering::Default.display_name(), "Default");
        assert_eq!(TextRendering::CrispHeavy.display_name(), "Crisp Heavy");
        assert_eq!(
            TextRendering::CrispHeavyPlus.display_name(),
            "Crisp Heavy +"
        );
    }

    #[test]
    fn text_rendering_default_params_are_identity() {
        let p = TextRendering::Default.params();
        assert_eq!(p.weight_boost_body, 0);
        assert_eq!(p.weight_boost_dense, 0);
        assert_eq!(p.weight_boost_mid, 0);
        assert_eq!(p.snap_x, SnapX::None);
        assert!(p.hint);
        assert_eq!(p.letter_spacing_em_dense, 0.0);
        assert!(!p.prefer_msaa);
    }

    #[test]
    fn crisp_heavy_params_are_pro_quality() {
        let p = TextRendering::CrispHeavy.params();
        assert_eq!(p.weight_boost_body, 300);
        assert_eq!(p.weight_boost_dense, 200);
        assert_eq!(p.weight_boost_mid, 150);
        assert_eq!(p.snap_x, SnapX::Full);
        assert!(
            !p.hint,
            "CrispHeavy MUST disable hint — see typography docstring"
        );
        assert!(!p.prefer_msaa, "CrispHeavy stays on AaConfig::Area");
    }

    #[test]
    fn crisp_heavy_plus_differs_from_heavy_in_three_axes() {
        // The whole point of CrispHeavyPlus: same boost as CrispHeavy
        // but 3 axes flipped to A/B-test the "tempero final".
        let h = TextRendering::CrispHeavy.params();
        let p = TextRendering::CrispHeavyPlus.params();
        // Same boost (so the A/B isolates the 3 changes, not the weight).
        assert_eq!(p.weight_boost_body, h.weight_boost_body);
        assert_eq!(p.weight_boost_dense, h.weight_boost_dense);
        assert_eq!(p.weight_boost_mid, h.weight_boost_mid);
        // 3 axes flipped.
        assert_ne!(p.snap_x, h.snap_x, "Plus must use Half snap, not Full");
        assert_eq!(p.snap_x, SnapX::Half);
        assert!(p.letter_spacing_em_dense < 0.0, "Plus tightens body");
        assert!(p.prefer_msaa, "Plus must enable MSAA");
    }

    #[test]
    fn crisp_weight_boost_monotonically_decreases_in_size() {
        // For all presets, boost is non-increasing as size grows.
        let sizes = [10.0_f32, 11.0, 12.0, 13.0, 15.0, 17.0, 19.0, 22.0, 32.0];
        for preset in [
            TextRendering::Default,
            TextRendering::CrispHeavy,
            TextRendering::CrispHeavyPlus,
        ] {
            let params = preset.params();
            let mut prev = u16::MAX;
            for s in sizes {
                let b = crisp_weight_boost_for(params, s);
                assert!(b <= prev, "preset={preset:?} size={s} prev={prev} curr={b}");
                prev = b;
            }
        }
    }

    #[test]
    fn crisp_weight_boost_typetoken_coverage_for_heavy() {
        // Anchor the tiers against the real TypeToken sizes for the
        // canonical CrispHeavy preset.
        let p = TextRendering::CrispHeavy.params();
        assert_eq!(crisp_weight_boost_for(p, TypeToken::Xxs.px()), 300);
        assert_eq!(crisp_weight_boost_for(p, TypeToken::Xs.px()), 300);
        assert_eq!(crisp_weight_boost_for(p, TypeToken::Sm.px()), 300);
        assert_eq!(crisp_weight_boost_for(p, TypeToken::Base.px()), 200);
        assert_eq!(crisp_weight_boost_for(p, TypeToken::Md.px()), 200);
        assert_eq!(crisp_weight_boost_for(p, TypeToken::Lg.px()), 150);
        assert_eq!(crisp_weight_boost_for(p, TypeToken::Xl.px()), 0);
        assert_eq!(crisp_weight_boost_for(p, TypeToken::Xl2.px()), 0);
        assert_eq!(crisp_weight_boost_for(p, TypeToken::Xl3.px()), 0);
    }

    #[test]
    fn crisp_weight_boost_default_preset_always_zero() {
        // Default's params produce 0 boost at every size — the preset
        // is a true identity.
        let p = TextRendering::Default.params();
        for s in [10.0_f32, 11.0, 15.0, 20.0, 32.0, 44.0] {
            assert_eq!(crisp_weight_boost_for(p, s), 0);
        }
    }
}
