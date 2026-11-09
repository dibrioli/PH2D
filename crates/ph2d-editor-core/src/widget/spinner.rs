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
    /// `0..=1` da espera conhecida: o mesmo arco, sem parar de rodar, cresce do quarto até fechar o anel (smoke
    /// 09/11: recomeçar vazio no topo fazia o selo piscar um quadro na passagem).
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
    let (from, sweep) = match ring_arc(phase, clock_ns) {
        Some(arc) => arc,
        None => {
            let ProgressPhase::Done(t) = phase else {
                return;
            };
            crate::paint::fill_circle(
                scene,
                cx,
                cy,
                side / 2.0,
                color(ColorToken::SuccessSoft).multiply_alpha(fade),
            );
            // A cortina corre só sobre o DESENHO do check (smoke 09/11: sobre a caixa toda, o disco ficava uns quadros
            // vazio enquanto ela atravessava o respiro e o vazio à esquerda do glifo).
            let (x0, x1) = check_span(side);
            let shown = (t / CHECK_DRAWN).clamp(0.0, 1.0);
            let wipe = ph2d_vector::Rect::new(
                f64::from(rect.x),
                f64::from(rect.y),
                f64::from(rect.x + x0 + (x1 - x0) * shown),
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
    let arc = arc_points(cx, cy, ring, from, sweep);
    crate::paint::stroke_polyline(scene, &arc, width, color(ColorToken::Accent));
}

/// Onde o traço do check começa e acaba na horizontal, em px a partir da borda esquerda de um quadrado de lado `side`:
/// a caixa do caminho do glifo na viewbox 24×24, posta no quadrado como o `paint_icon` a põe (o respiro, depois a escala).
fn check_span(side: f32) -> (f32, f32) {
    use ph2d_vector::Shape;
    const VIEWBOX: f64 = 24.0; // LITERAL-PX-OK: a viewbox dos ícones
    let mut path = ph2d_vector::BezPath::new();
    for cmd in IconId::Check.cmds() {
        path.extend(crate::icons::cmd_to_path(*cmd));
    }
    let caixa = path.bounding_box();
    let pad = crate::paint::icon_pad(side);
    let glyph = f64::from(side - 2.0 * pad) / VIEWBOX;
    let em = |x: f64| pad + (x * glyph) as f32;
    (em(caixa.x0), em(caixa.x1))
}

/// O arco aceso do anel, `(início, comprimento)` em voltas; `None` no check. Gira SEMPRE pelo relógio; a espera conhecida
/// alonga-o do quarto até à volta inteira.
fn ring_arc(phase: ProgressPhase, clock_ns: u128) -> Option<(f32, f32)> {
    let turn = (clock_ns % SPIN_PERIOD_NS) as f32 / SPIN_PERIOD_NS as f32;
    match phase {
        ProgressPhase::Indeterminate => Some((turn, SPIN_ARC)),
        ProgressPhase::Filling(f) => Some((turn, SPIN_ARC + f.clamp(0.0, 1.0) * (1.0 - SPIN_ARC))),
        ProgressPhase::Done(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ **Girar → encher sem saltar** (smoke 09/11: o 1.º quadro «enchendo 0 %» não tinha arco e o seguinte recomeçava
    /// no topo — o selo piscava). Em qualquer relógio, encher 0 é o arco de girar, e encher 1 fecha o anel.
    /// *Mutação: encher a partir do topo com comprimento `f` ⇒ difere em todo relógio ≠ 0.*
    #[test]
    fn encher_comeca_no_arco_de_girar_e_acaba_no_anel_inteiro() {
        for clock in [0, 123_456_789, 700_000_000, 1_999_999_999] {
            let gira = ring_arc(ProgressPhase::Indeterminate, clock);
            assert_eq!(
                ring_arc(ProgressPhase::Filling(0.0), clock),
                gira,
                "relógio {clock}"
            );
            let cheio = ring_arc(ProgressPhase::Filling(1.0), clock).map(|a| a.1);
            assert_eq!(cheio, Some(1.0));
        }
        assert_eq!(ring_arc(ProgressPhase::Done(0.5), 0), None);
    }

    /// A cortina do check começa onde o traço dele começa: dentro do quadrado, à direita do respiro, e acaba antes da
    /// borda direita. *Mutação: a cortina sobre a caixa toda (`0..side`) ⇒ começa no 0.*
    #[test]
    fn a_cortina_do_check_corre_so_sobre_o_desenho() {
        let side = 22.0;
        let (x0, x1) = check_span(side);
        let pad = crate::paint::icon_pad(side);
        assert!(
            x0 > pad && x1 > x0 && x1 < side - pad,
            "{x0}..{x1} num lado {side}"
        );
    }

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
