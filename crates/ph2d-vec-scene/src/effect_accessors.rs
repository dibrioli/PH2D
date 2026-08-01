//! **A superfície de ACESSORES de [`PathEffect`]** (`as_trim`/`as_zigzag`/… + `label`) — `impl
//! PathEffect` continuado noutro arquivo pelo teto de LOC, irmão de [`crate::effect`] como o
//! [`super::params_surface`]. Os `match` são **exaustivos de propósito**: um efeito novo obriga
//! cada acessor a decidir o que responde.

use super::PathEffect;
use crate::fx_falloff::FalloffSpec;
use crate::fx_hatch::HatchSpec;
use crate::fx_knot::KnotSpec;
use crate::fx_sketch::SketchSpec;
use crate::fx_trim::TrimSpec;
use crate::fx_twist::TwistSpec;
use crate::fx_zigzag::ZigZagSpec;

impl PathEffect {
    /// Este efeito é um Trim? Devolve os parâmetros dele.
    ///
    /// ⚠️ O `match` é **exaustivo de propósito**: quando o 2º tipo de efeito entrar, isto
    /// deixa de compilar e quem o acrescentar TEM de decidir o que este acessor responde.
    /// Um `_ => None` hoje seria um silêncio que ninguém voltaria a ler.
    #[must_use]
    pub fn as_trim(&self) -> Option<&TrimSpec> {
        match self {
            Self::Trim(t) => Some(t),
            Self::ZigZag(_)
            | Self::Repeat(_)
            | Self::Bloat(_)
            | Self::Warp(_)
            | Self::Falloff(_)
            | Self::Twist(_)
            | Self::Knot(_)
            | Self::Sketch(_)
            | Self::Hatch(_) => None,
        }
    }

    /// O irmão mutável do [`Self::as_trim`] — para quem AJUSTA um parâmetro.
    pub fn as_trim_mut(&mut self) -> Option<&mut TrimSpec> {
        match self {
            Self::Trim(t) => Some(t),
            Self::ZigZag(_)
            | Self::Repeat(_)
            | Self::Bloat(_)
            | Self::Warp(_)
            | Self::Falloff(_)
            | Self::Twist(_)
            | Self::Knot(_)
            | Self::Sketch(_)
            | Self::Hatch(_) => None,
        }
    }

    /// O Zig Zag deste efeito, se for um — irmão do [`Self::as_trim`].
    #[must_use]
    pub fn as_zigzag(&self) -> Option<&ZigZagSpec> {
        match self {
            Self::ZigZag(z) => Some(z),
            Self::Trim(_)
            | Self::Repeat(_)
            | Self::Bloat(_)
            | Self::Warp(_)
            | Self::Falloff(_)
            | Self::Twist(_)
            | Self::Knot(_)
            | Self::Sketch(_)
            | Self::Hatch(_) => None,
        }
    }

    /// O irmão mutável do [`Self::as_zigzag`].
    pub fn as_zigzag_mut(&mut self) -> Option<&mut ZigZagSpec> {
        match self {
            Self::ZigZag(z) => Some(z),
            Self::Trim(_)
            | Self::Repeat(_)
            | Self::Bloat(_)
            | Self::Warp(_)
            | Self::Falloff(_)
            | Self::Twist(_)
            | Self::Knot(_)
            | Self::Sketch(_)
            | Self::Hatch(_) => None,
        }
    }

    /// O nome que o painel mostra. Mora aqui (e não numa tabela no painel) porque uma
    /// segunda lista dos efeitos divergiria da primeira assim que alguém acrescentasse um.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Trim(_) => "Trim Path",
            Self::ZigZag(z) => {
                if z.rough_seed.is_some() {
                    "Roughen"
                } else {
                    "Zig Zag"
                }
            }
            Self::Repeat(_) => "Repeater",
            Self::Bloat(_) => "Pucker & Bloat",
            Self::Warp(w) => w.style.label(),
            Self::Falloff(f) => f.shape.label(),
            Self::Twist(_) => "Twist",
            Self::Knot(_) => "Knot",
            Self::Sketch(_) => "Sketch",
            Self::Hatch(_) => "Hatch",
        }
    }

    /// O Warp deste efeito, se for um — irmão do [`Self::as_trim`].
    #[must_use]
    pub fn as_warp(&self) -> Option<&crate::fx_warp_presets::WarpSpec> {
        match self {
            Self::Warp(w) => Some(w),
            Self::Trim(_)
            | Self::ZigZag(_)
            | Self::Repeat(_)
            | Self::Bloat(_)
            | Self::Falloff(_)
            | Self::Twist(_)
            | Self::Knot(_)
            | Self::Sketch(_)
            | Self::Hatch(_) => None,
        }
    }

    /// O irmão mutável do [`Self::as_warp`].
    pub fn as_warp_mut(&mut self) -> Option<&mut crate::fx_warp_presets::WarpSpec> {
        match self {
            Self::Warp(w) => Some(w),
            Self::Trim(_)
            | Self::ZigZag(_)
            | Self::Repeat(_)
            | Self::Bloat(_)
            | Self::Falloff(_)
            | Self::Twist(_)
            | Self::Knot(_)
            | Self::Sketch(_)
            | Self::Hatch(_) => None,
        }
    }

    /// O Falloff deste efeito, se for um — irmão do [`Self::as_trim`].
    #[must_use]
    pub fn as_falloff(&self) -> Option<&FalloffSpec> {
        match self {
            Self::Falloff(f) => Some(f),
            Self::Trim(_)
            | Self::ZigZag(_)
            | Self::Repeat(_)
            | Self::Bloat(_)
            | Self::Warp(_)
            | Self::Twist(_)
            | Self::Knot(_)
            | Self::Sketch(_)
            | Self::Hatch(_) => None,
        }
    }

    /// O irmão mutável do [`Self::as_falloff`].
    pub fn as_falloff_mut(&mut self) -> Option<&mut FalloffSpec> {
        match self {
            Self::Falloff(f) => Some(f),
            Self::Trim(_)
            | Self::ZigZag(_)
            | Self::Repeat(_)
            | Self::Bloat(_)
            | Self::Warp(_)
            | Self::Twist(_)
            | Self::Knot(_)
            | Self::Sketch(_)
            | Self::Hatch(_) => None,
        }
    }

    /// O Twist deste efeito, se for um — irmão do [`Self::as_trim`].
    #[must_use]
    pub fn as_twist(&self) -> Option<&TwistSpec> {
        match self {
            Self::Twist(t) => Some(t),
            Self::Trim(_)
            | Self::ZigZag(_)
            | Self::Repeat(_)
            | Self::Bloat(_)
            | Self::Warp(_)
            | Self::Falloff(_)
            | Self::Knot(_)
            | Self::Sketch(_)
            | Self::Hatch(_) => None,
        }
    }

    /// O irmão mutável do [`Self::as_twist`].
    pub fn as_twist_mut(&mut self) -> Option<&mut TwistSpec> {
        match self {
            Self::Twist(t) => Some(t),
            Self::Trim(_)
            | Self::ZigZag(_)
            | Self::Repeat(_)
            | Self::Bloat(_)
            | Self::Warp(_)
            | Self::Falloff(_)
            | Self::Knot(_)
            | Self::Sketch(_)
            | Self::Hatch(_) => None,
        }
    }

    /// O Knot deste efeito, se for um — irmão do [`Self::as_trim`].
    #[must_use]
    pub fn as_knot(&self) -> Option<&KnotSpec> {
        match self {
            Self::Knot(k) => Some(k),
            Self::Trim(_)
            | Self::ZigZag(_)
            | Self::Repeat(_)
            | Self::Bloat(_)
            | Self::Warp(_)
            | Self::Falloff(_)
            | Self::Twist(_)
            | Self::Sketch(_)
            | Self::Hatch(_) => None,
        }
    }

    /// O irmão mutável do [`Self::as_knot`].
    pub fn as_knot_mut(&mut self) -> Option<&mut KnotSpec> {
        match self {
            Self::Knot(k) => Some(k),
            Self::Trim(_)
            | Self::ZigZag(_)
            | Self::Repeat(_)
            | Self::Bloat(_)
            | Self::Warp(_)
            | Self::Falloff(_)
            | Self::Twist(_)
            | Self::Sketch(_)
            | Self::Hatch(_) => None,
        }
    }

    /// O Sketch deste efeito, se for um — irmão do [`Self::as_trim`].
    #[must_use]
    pub fn as_sketch(&self) -> Option<&SketchSpec> {
        match self {
            Self::Sketch(s) => Some(s),
            Self::Trim(_)
            | Self::ZigZag(_)
            | Self::Repeat(_)
            | Self::Bloat(_)
            | Self::Warp(_)
            | Self::Falloff(_)
            | Self::Twist(_)
            | Self::Knot(_)
            | Self::Hatch(_) => None,
        }
    }

    /// O irmão mutável do [`Self::as_sketch`].
    pub fn as_sketch_mut(&mut self) -> Option<&mut SketchSpec> {
        match self {
            Self::Sketch(s) => Some(s),
            Self::Trim(_)
            | Self::ZigZag(_)
            | Self::Repeat(_)
            | Self::Bloat(_)
            | Self::Warp(_)
            | Self::Falloff(_)
            | Self::Twist(_)
            | Self::Knot(_)
            | Self::Hatch(_) => None,
        }
    }

    /// O Hatch deste efeito, se for um — irmão do [`Self::as_trim`].
    #[must_use]
    pub fn as_hatch(&self) -> Option<&HatchSpec> {
        match self {
            Self::Hatch(h) => Some(h),
            Self::Trim(_)
            | Self::ZigZag(_)
            | Self::Repeat(_)
            | Self::Bloat(_)
            | Self::Warp(_)
            | Self::Falloff(_)
            | Self::Twist(_)
            | Self::Knot(_)
            | Self::Sketch(_) => None,
        }
    }

    /// O irmão mutável do [`Self::as_hatch`].
    pub fn as_hatch_mut(&mut self) -> Option<&mut HatchSpec> {
        match self {
            Self::Hatch(h) => Some(h),
            Self::Trim(_)
            | Self::ZigZag(_)
            | Self::Repeat(_)
            | Self::Bloat(_)
            | Self::Warp(_)
            | Self::Falloff(_)
            | Self::Twist(_)
            | Self::Knot(_)
            | Self::Sketch(_) => None,
        }
    }
}
