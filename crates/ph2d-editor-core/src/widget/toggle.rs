//! [`Toggle`] — binary on/off switch (snap-to-grid, lock-axis, etc).
//!
//! Same pattern as [`crate::widget::Button`]: data + state enum +
//! token-resolved colors + AccessKit `Role::Switch` node + paint
//! helper colocated. Maps the boolean `on` to `Toggled::True/False`
//! so screen readers announce "on" / "off".

use crate::paint::{fill_rounded_rect, resolve, stroke_rounded_rect};
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role, Toggled};
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme};
use ph2d_vector::{Affine, Brush, Circle, Fill, Point, VectorScene};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum ToggleState {
    #[default]
    Normal,
    Hovered,
    Pressed,
    Focused,
    Disabled,
}

#[derive(Clone, Debug)]
pub struct Toggle {
    pub id: NodeId,
    pub label: String,
    pub on: bool,
    pub state: ToggleState,
    /// Quanto do hover está PRESENTE (`0`..=`1`); [`crate::motion::SETTLED`] = assente no estado.
    pub hover_t: f32,
}

impl Toggle {
    pub fn new(id: NodeId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            on: false,
            state: ToggleState::Normal,
            hover_t: crate::motion::SETTLED,
        }
    }

    pub fn on(mut self, on: bool) -> Self {
        self.on = on;
        self
    }

    /// **O par visual do store numa chamada** — o irmão exacto do [`super::Button::visual`]; o
    /// `.on(..)` continua a vir do MODELO.
    #[must_use]
    pub fn visual(self, v: (ToggleState, f32)) -> Self {
        self.state(v.0).hover_t(v.1)
    }

    /// Quanto do hover está presente. O neutro [`crate::motion::SETTLED`] pinta o traço DURO.
    #[must_use]
    pub fn hover_t(mut self, t: f32) -> Self {
        self.hover_t = t.clamp(0.0, 1.0);
        self
    }

    pub fn state(mut self, state: ToggleState) -> Self {
        self.state = state;
        self
    }

    /// Flip the current state. Convenience for click handlers that
    /// don't know or care about the prior value.
    pub fn toggle(&mut self) {
        self.on = !self.on;
    }

    /// Build the AccessKit node. `Role::Switch` is the AccessKit
    /// canonical for binary on/off (vs. CheckBox which is tri-state).
    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        NodeBuilder::new(Role::Switch)
            .label(&self.label)
            .bounds(x, y, w, h)
            .focusable(self.state != ToggleState::Disabled)
            .action(Action::Click)
            .toggled(if self.on {
                Toggled::True
            } else {
                Toggled::False
            })
            .build()
    }
}

/// Pill body + circular thumb (left=off, right=on). Focus ring is
/// **O disco do thumb** — centro e raio, e ele NUNCA sai do corpo.
///
/// ⚠️ **O curso de um toggle é exatamente `w − h`**, porque o thumb é dimensionado pela ALTURA e
/// posicionado pela LARGURA. Nada perguntava se sobrava largura, e o preço era medido: numa
/// moldura **quadrada** o curso é **zero** — ON e OFF pintam o disco no MESMO pixel, um controle
/// que não consegue mostrar o próprio estado; e numa moldura **em pé** (30×60) o curso é
/// **−30**, ou seja a semântica **INVERTE** e o disco cai **24 px para fora do corpo**, 80% do
/// diâmetro, nos dois estados.
///
/// ⚠️ **Isto deixou de ser hipotético quando a pele chegou:** o `widget_live::frame_of` entrega a
/// bbox da forma que o ARTISTA desenhou, e o único guard dela é `w > 1 && h > 1`.
///
/// A lei é a do `glyph_box`: a caixa mede pelo lado que a contém. O diâmetro sai de
/// `min(h, w)` e o centro é clampado ao corpo — e ⚠️ **para todo `w >= h` isto REDUZ à expressão
/// que shipava, termo a termo** (`min(h,w)` é `h`, e o clamp é no-op porque `pad >= 0`), que é o
/// que mantém todos os chamadores de hoje byte-idênticos.
///
/// Um toggle mais alto que largo continua **mudo**, e isso é honesto: o curso de um toggle é
/// horizontal, e ali não há largura para percorrer. O que ele deixa de fazer é pintar fora de si
/// mesmo e dizer o contrário do que vale.
#[must_use]
pub fn thumb_circle(rect: Rect, on: bool) -> (f32, f32, f32) {
    let pad = (rect.h * 0.15).clamp(2.0, 6.0); // LITERAL-PX-OK: toggle inner pad 15% of body height with min/max
    // ⚠️ O piso de `Xs` vem do desenho original (um disco não pode sumir num toggle normal), mas
    // ele **não pode vencer o corpo**: num corpo de 2 px ele sozinho põe 1 px de disco para fora.
    // O `.min(w)` é o que torna a contenção incondicional — e para `w >= h` ele é no-op, porque
    // `h - 2·pad < h <= w`.
    let diameter = (rect.h.min(rect.w) - pad * 2.0)
        .max(Spacing::Xs.px())
        .min(rect.w.max(0.0));
    let r = diameter * 0.5;
    let cy = rect.y + rect.h * 0.5;
    // ⚠️ **O CURSO é a lei, e ele pode ser negativo:** o `pad` sai da ALTURA e é subtraído da
    // LARGURA, então numa moldura estreita e alta os dois extremos **se cruzam** — ON à esquerda de
    // OFF, a semântica ao contrário. Sem largura para percorrer, os dois estados colapsam no
    // centro: mudo, mas nunca mentindo.
    let travel = rect.w - pad * 2.0 - diameter;
    let cx = if travel <= 0.0 {
        rect.x + rect.w * 0.5
    } else if on {
        rect.x + rect.w - pad - r
    } else {
        rect.x + pad + r
    };
    // O clamp é a GARANTIA, e não o cálculo: mesmo com um `pad` grande num corpo minúsculo, o
    // disco fica dentro. ⚠️ É o `safe_clamp` do repo, e não o da `std`, porque o `arch_safe_clamp`
    // recusa bounds variáveis — aqui o ramo de troca dele é **inalcançável** (o `.min(w)` do
    // diâmetro garante `r <= w/2`, logo `lo <= hi`), e o que ele acrescenta de facto é a recusa do
    // NaN, que numa bbox autorada não é hipótese ociosa.
    let cx = crate::math::safe_clamp(cx, rect.x + r, rect.x + rect.w - r);
    (cx, cy, r)
}

/// drawn as a true stroked outer rounded rect — no more "re-fill
/// inside the ring" inversion hack.
pub fn paint_toggle(toggle: &Toggle, rect: Rect, scene: &mut VectorScene, theme: Theme) {
    let radius = Radius::Full.px();
    let body_token = match (toggle.on, toggle.state) {
        // Disabled + on stays on a muted accent so users can still
        // read the current value at a glance; disabled + off keeps
        // the neutral Border fill. Previously both collapsed to
        // Border which read as "off, locked" regardless of the
        // actual `on` flag.
        (true, ToggleState::Disabled) => ColorToken::AccentSoft,
        (false, ToggleState::Disabled) => ColorToken::Border,
        (true, _) => ColorToken::Accent,
        (false, _) => ColorToken::Bg2,
    };
    fill_rounded_rect(scene, rect, radius, resolve(body_token, theme));

    // Subtle border emphasis on Hover/Focus so the off-state pill
    // doesn't look static when the cursor is over it.
    // ⚠️ **Aqui o eixo é uma PRESENÇA, não um par de cores:** em repouso não há traço nenhum, e o
    //    do hover tem de EMERGIR do nada — é o caso `(None, Some(hot))` da `blend_token_color`, o
    //    mesmo que o botão `Default` já usava. O `Focused` fica de fora (estado duro, traço de
    //    2 px), e o neutro `SETTLED` devolve o traço cheio de sempre.
    let soft = matches!(toggle.state, ToggleState::Normal | ToggleState::Hovered);
    let emph = crate::motion::hover_axis(
        soft,
        toggle.hover_t,
        None,
        Some(ColorToken::BorderEmph.resolve(theme)),
    );
    if let Some(c) = emph {
        stroke_rounded_rect(scene, rect, radius, 1.0, crate::paint::token_to_vello(c));
    } else if matches!(toggle.state, ToggleState::Hovered | ToggleState::Focused) {
        let stroke_w = if toggle.state == ToggleState::Focused {
            2.0
        } else {
            1.0
        };
        stroke_rounded_rect(
            scene,
            rect,
            radius,
            stroke_w,
            resolve(ColorToken::BorderEmph, theme),
        );
    }

    // Circular thumb. Diameter = body height - 2*pad. Off → left,
    // On → right. Disabled tints to Text3 so it stays visible but mute.
    let (cx, cy, r) = thumb_circle(rect, toggle.on);
    let thumb_token = if toggle.state == ToggleState::Disabled {
        ColorToken::Text3
    } else if toggle.on {
        ColorToken::AccentFg
    } else {
        ColorToken::Text2
    };
    let thumb = Circle::new(Point::new(cx as f64, cy as f64), r as f64);
    scene.inner_mut().fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(resolve(thumb_token, theme)),
        None,
        &thumb,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_spec() {
        let t = Toggle::new(NodeId(1), "Snap");
        assert_eq!(t.id, NodeId(1));
        assert_eq!(t.label, "Snap");
        assert!(!t.on);
        assert_eq!(t.state, ToggleState::Normal);
    }

    #[test]
    fn toggle_flips_state() {
        let mut t = Toggle::new(NodeId(1), "x");
        assert!(!t.on);
        t.toggle();
        assert!(t.on);
        t.toggle();
        assert!(!t.on);
    }

    #[test]
    fn builder_chain_sets_on() {
        let t = Toggle::new(NodeId(1), "x").on(true);
        assert!(t.on);
    }

    #[test]
    fn a11y_node_has_switch_role_and_toggled_off() {
        let t = Toggle::new(NodeId(1), "Snap");
        let node = t.build_a11y(0.0, 0.0, 60.0, 24.0);
        assert_eq!(node.role(), Role::Switch);
        assert_eq!(node.label(), Some("Snap"));
        assert_eq!(node.toggled(), Some(Toggled::False));
    }

    #[test]
    fn a11y_node_toggled_true_when_on() {
        let t = Toggle::new(NodeId(1), "Snap").on(true);
        let node = t.build_a11y(0.0, 0.0, 60.0, 24.0);
        assert_eq!(node.toggled(), Some(Toggled::True));
    }

    fn smoke(t: Toggle, theme: Theme) {
        let mut scene = VectorScene::new();
        paint_toggle(&t, Rect::new(0.0, 0.0, 60.0, 24.0), &mut scene, theme);
    }

    #[test]
    fn paint_smoke_off() {
        smoke(Toggle::new(NodeId(1), "x"), Theme::Forge);
    }

    #[test]
    fn paint_smoke_on() {
        smoke(Toggle::new(NodeId(1), "x").on(true), Theme::Forge);
    }

    #[test]
    fn paint_smoke_hovered_on() {
        smoke(
            Toggle::new(NodeId(1), "x")
                .on(true)
                .state(ToggleState::Hovered),
            Theme::Sunstone,
        );
    }

    #[test]
    fn paint_smoke_focused() {
        smoke(
            Toggle::new(NodeId(1), "x").state(ToggleState::Focused),
            Theme::Sunstone,
        );
    }

    #[test]
    fn paint_smoke_pressed() {
        smoke(
            Toggle::new(NodeId(1), "x").state(ToggleState::Pressed),
            Theme::Blueprint,
        );
    }

    #[test]
    fn paint_smoke_disabled() {
        smoke(
            Toggle::new(NodeId(1), "x").state(ToggleState::Disabled),
            Theme::Forge,
        );
    }
}

#[cfg(test)]
mod thumb_tests {
    use super::*;

    /// A expressão que SHIPAVA, congelada — o oráculo da redução.
    ///
    /// ⚠️ Ela é `cfg(test)` de propósito: um `pub(super)` sem chamador seria uma **segunda
    /// resposta** esperando alguém chamá-la, e o doc deste gate (*"é o código que shipava"*)
    /// passaria a mentir no dia em que alguém a usasse.
    fn shipped(rect: Rect, on: bool) -> (f32, f32, f32) {
        let pad = (rect.h * 0.15).clamp(2.0, 6.0);
        let diameter = (rect.h - pad * 2.0).max(Spacing::Xs.px());
        let r = diameter * 0.5;
        let cy = rect.y + rect.h * 0.5;
        let cx = if on {
            rect.x + rect.w - pad - r
        } else {
            rect.x + pad + r
        };
        (cx, cy, r)
    }

    /// **Toda moldura mais larga que alta é BYTE-IDÊNTICA ao que shipava.**
    ///
    /// ⚠️ Esta é a metade que torna a cura segura: é ela que diz que nenhum dos chamadores de hoje
    /// se move um ulp. Sem ela, "clampar" seria uma mudança de aparência disfarçada de correção.
    #[test]
    fn a_wide_toggle_is_byte_identical_to_what_shipped() {
        for (w, h) in [(160.0, 36.0), (60.0, 24.0), (44.0, 22.0), (28.0, 28.0)] {
            let rect = Rect::new(10.0, 20.0, w, h);
            for on in [false, true] {
                assert_eq!(
                    thumb_circle(rect, on),
                    shipped(rect, on),
                    "a geometria mudou para {w}x{h} (on={on}) — os chamadores de hoje NAO podem se mover"
                );
            }
        }
    }

    /// **O disco nunca sai do corpo, e ON nunca fica à ESQUERDA de OFF.**
    ///
    /// ⚠️ Gate red-first sobre as duas metades medidas: numa moldura em pé o curso era **−30**
    /// (semântica invertida) e o disco caía **24 px para fora**. A varredura inclui o quadrado e o
    /// degenerado, porque é ali que a aritmética antiga produzia o absurdo.
    #[test]
    fn the_thumb_never_leaves_the_body_and_never_runs_backwards() {
        for (w, h) in [
            (160.0, 36.0),
            (48.0, 48.0),
            (30.0, 60.0),
            (10.0, 80.0),
            (6.0, 6.0),
            (2.0, 200.0),
        ] {
            let rect = Rect::new(10.0, 20.0, w, h);
            let (cx_off, _, r) = thumb_circle(rect, false);
            let (cx_on, _, _) = thumb_circle(rect, true);
            for (cx, name) in [(cx_off, "off"), (cx_on, "on")] {
                assert!(
                    cx - r >= rect.x - 0.001 && cx + r <= rect.x + rect.w + 0.001,
                    "o thumb {name} saiu do corpo em {w}x{h}: centro {cx}, raio {r}, corpo \
                     [{}, {}]",
                    rect.x,
                    rect.x + rect.w
                );
            }
            assert!(
                cx_on >= cx_off - 0.001,
                "a semantica INVERTEU em {w}x{h}: ON ({cx_on}) ficou a' esquerda de OFF ({cx_off})"
            );
        }
    }
}
