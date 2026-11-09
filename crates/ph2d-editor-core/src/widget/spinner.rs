//! [`Spinner`] — indeterminate progress indicator.
//!
//! Always rotating in the live UI; the static frame is rendered via
//! `IconId::Spinner`. Rotation animation is shell-side (timeline +
//! per-frame Affine::rotate) — out of scope for paint.

use crate::icons::IconId;
use crate::paint::paint_icon;
use crate::zones::Rect;
use ph2d_a11y::{Node, NodeBuilder, NodeId, Role};
use ph2d_tokens::{ColorToken, StrokeToken, Theme};
use ph2d_vector::VectorScene;

#[derive(Clone, Debug)]
pub struct Spinner {
    pub id: NodeId,
    pub label: String,
}

impl Spinner {
    pub fn new(id: NodeId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
        }
    }

    /// Build the AccessKit node. `Role::ProgressIndicator` without
    /// numeric value signals "indeterminate" to assistive tech.
    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        NodeBuilder::new(Role::ProgressIndicator)
            .label(&self.label)
            .bounds(x, y, w, h)
            .build()
    }
}

/// Static spinner glyph at the rect, tinted with `Accent`. The shell
/// applies an `Affine::rotate` around the rect center each frame to
/// animate.
pub fn paint_spinner(spinner: &Spinner, rect: Rect, scene: &mut VectorScene, theme: Theme) {
    let _ = spinner;
    let color = crate::paint::resolve(ColorToken::Accent, theme);
    paint_icon(
        scene,
        IconId::Spinner,
        rect,
        color,
        StrokeToken::Default.px(),
    );
}

/// ⭐ **Em que pé está um trabalho que acaba sozinho** (dono 09/11, a folha ao vivo do Painter no cartão Animation):
/// o anel GIRA enquanto não se sabe quando acaba, ENCHE quando a espera é conhecida, e acaba num check verde que some.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ProgressPhase {
    /// Sem prazo conhecido: um quarto de anel a rodar.
    Indeterminate,
    /// `0..=1` da espera conhecida, a encher a partir do topo, no sentido do relógio.
    Filling(f32),
    /// `0..=1` da vida do check: desenha-se da esquerda, fica, e desvanece.
    Done(f32),
}

/// Uma volta do anel indeterminado.
const SPIN_PERIOD_NS: u128 = 1_000_000_000;
/// O pedaço do anel que roda, sem prazo.
const SPIN_ARC: f32 = 0.25; // LITERAL-PX-OK: fração de volta
/// Até aqui da vida do check ele desenha-se; daqui em diante desvanece.
const CHECK_DRAWN: f32 = 0.3; // LITERAL-PX-OK: fração de tempo
const CHECK_FADES: f32 = 0.7; // LITERAL-PX-OK: fração de tempo
/// Lados do polígono de um anel inteiro (um arco usa a parte dele).
const RING_SIDES: f32 = 48.0; // LITERAL-PX-OK: contagem de segmentos

/// O arco de raio `r` em torno de `(cx, cy)`, de `from` a `from + sweep` voltas (`0` = o topo, sentido do relógio).
fn arc_points(cx: f32, cy: f32, r: f32, from: f32, sweep: f32) -> Vec<(f32, f32)> {
    let n = (RING_SIDES * sweep.clamp(0.0, 1.0)).ceil().max(1.0) as usize;
    (0..=n)
        .map(|i| {
            let turn = from + sweep * i as f32 / n as f32;
            let a = std::f32::consts::TAU * turn - std::f32::consts::FRAC_PI_2;
            (cx + r * a.cos(), cy + r * a.sin())
        })
        .collect()
}

/// **O selo de progresso** em `rect` (um quadrado): disco do cartão, anel, e o check verde no fim. `clock_ns` roda o
/// anel indeterminado ([`crate::interaction::WidgetStore::clock_ns`]).
pub fn paint_progress_badge(
    scene: &mut VectorScene,
    rect: Rect,
    theme: Theme,
    phase: ProgressPhase,
    clock_ns: u128,
) {
    let side = rect.w.min(rect.h);
    let (cx, cy) = (rect.x + rect.w / 2.0, rect.y + rect.h / 2.0);
    let color = |t| crate::paint::resolve(t, theme);
    let width = StrokeToken::Thick.px();
    let ring = side / 2.0 - crate::paint::icon_pad(side);
    let fade = match phase {
        ProgressPhase::Done(t) if t > CHECK_FADES => {
            (1.0 - (t - CHECK_FADES) / (1.0 - CHECK_FADES)).max(0.0)
        }
        _ => 1.0,
    };
    crate::paint::fill_circle(
        scene,
        cx,
        cy,
        side / 2.0,
        color(ColorToken::Bg1).multiply_alpha(fade),
    );
    let (from, sweep) = match phase {
        ProgressPhase::Indeterminate => {
            let turn = (clock_ns % SPIN_PERIOD_NS) as f32 / SPIN_PERIOD_NS as f32;
            (turn, SPIN_ARC)
        }
        ProgressPhase::Filling(f) => (0.0, f.clamp(0.0, 1.0)),
        ProgressPhase::Done(t) => {
            crate::paint::fill_circle(
                scene,
                cx,
                cy,
                side / 2.0,
                color(ColorToken::SuccessSoft).multiply_alpha(fade),
            );
            let shown = (t / CHECK_DRAWN).clamp(0.0, 1.0);
            let wipe = ph2d_vector::Rect::new(
                f64::from(rect.x),
                f64::from(rect.y),
                f64::from(rect.x + rect.w * shown),
                f64::from(rect.y + rect.h),
            );
            scene.push_clip(&wipe);
            let check = color(ColorToken::Success).multiply_alpha(fade);
            paint_icon(scene, IconId::Check, rect, check, width);
            scene.pop_layer();
            return;
        }
    };
    let track = arc_points(cx, cy, ring, 0.0, 1.0);
    crate::paint::stroke_polyline(scene, &track, width, color(ColorToken::Bg3));
    if sweep > 0.0 {
        let arc = arc_points(cx, cy, ring, from, sweep);
        crate::paint::stroke_polyline(scene, &arc, width, color(ColorToken::Accent));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_spec() {
        let s = Spinner::new(NodeId(1), "Loading");
        assert_eq!(s.id, NodeId(1));
        assert_eq!(s.label, "Loading");
    }

    #[test]
    fn a11y_role_is_progress_indicator() {
        let node = Spinner::new(NodeId(1), "Loading").build_a11y(0.0, 0.0, 24.0, 24.0);
        assert_eq!(node.role(), Role::ProgressIndicator);
        assert_eq!(node.label(), Some("Loading"));
    }

    #[test]
    fn paint_smoke() {
        let s = Spinner::new(NodeId(1), "Loading");
        let mut scene = VectorScene::new();
        paint_spinner(
            &s,
            Rect::new(0.0, 0.0, 24.0, 24.0),
            &mut scene,
            Theme::Forge,
        );
    }
}
