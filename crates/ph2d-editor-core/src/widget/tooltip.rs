//! [`Tooltip`] — small floating popover with a single-line message.
//!
//! Lives in `Layer::Overlay`. v1 paints unconditionally — the host
//! decides when (hover dwell, focus + key) to render it. AccessKit
//! `Role::Tooltip` so screen readers can announce it as auxiliary
//! help text.

use super::lista_flutuante::Superficie;
use crate::paint::{paint_text_centered, paint_text_elided};
use crate::zones::Rect;
use ph2d_a11y::{Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

#[derive(Clone, Debug)]
pub struct Tooltip {
    pub id: NodeId,
    pub message: String,
}

impl Tooltip {
    pub fn new(id: NodeId, message: impl Into<String>) -> Self {
        Self {
            id,
            message: message.into(),
        }
    }

    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        NodeBuilder::new(Role::Tooltip)
            .label(&self.message)
            .bounds(x, y, w, h)
            .build()
    }
}

pub fn paint_tooltip(
    tip: &Tooltip,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let superficie = Superficie::de(theme);
    superficie.moldura_da_dica(scene, rect);
    if superficie.tinta().is_some() {
        // O `paint_text_centered` reserva respiro e cortava «Add entity» em «Add ent…» (foto m1_dica).
        let px = TypeToken::Sm.px();
        let w = text_system.prefix_width(&tip.message, px).min(rect.w);
        return paint_text_elided(
            text_system,
            scene,
            &tip.message,
            rect.x + (rect.w - w) * 0.5,
            rect.y + (rect.h - px) * 0.5,
            px,
            rect.w,
            superficie.cor(ColorToken::Text1),
        );
    }
    paint_text_centered(
        text_system,
        scene,
        &tip.message,
        rect,
        TypeToken::Sm.px(),
        superficie.cor(ColorToken::Text1),
    );
}

/// ⭐ O tecto de uma linha de dica: acima dele a dica QUEBRA em linhas, nunca corta (lei da casa: nunca elidir).
pub const LARGURA_MAXIMA_DA_DICA: f32 = 320.0; // LITERAL-PX-OK: tecto de leitura de uma dica (o «rich tooltip» do Material)

/// Uma dica medida no peso e no modo de texto em que se pinta: as linhas e o tamanho da pílula.
#[derive(Clone, Debug, PartialEq)]
pub struct DicaMedida {
    pub linhas: Vec<String>,
    pub w: f32,
    pub h: f32,
}

/// O passo entre linhas de uma dica.
fn passo_da_linha(text_system: &mut TextSystem, px: f32) -> f32 {
    text_system.layout("Ag", px, f32::INFINITY).height().max(px)
}

/// ⭐ **Mede a dica** com o MESMO `prefix_width` (peso Medium, modo de texto activo) com que [`paint_dica`] a pinta —
/// medir com o `layout` e pintar com outro peso cortava-a em «…» (smoke do dono 10/11). Quebra por palavras
/// ([`super::showcase::linhas_visuais`]) no tecto [`LARGURA_MAXIMA_DA_DICA`] ou na janela `viewport_w`, o menor.
#[must_use]
pub fn mede_a_dica(text_system: &mut TextSystem, text: &str, viewport_w: f32) -> DicaMedida {
    let px = TypeToken::Sm.px();
    let (pad, gap) = (ph2d_tokens::Spacing::Xl.px(), ph2d_tokens::Spacing::Sm.px());
    let budget = LARGURA_MAXIMA_DA_DICA.min(viewport_w - pad - gap * 2.0);
    let mede = |ts: &mut TextSystem, s: &str| ts.prefix_width(s.trim_end(), px);
    let linhas: Vec<String> =
        super::showcase::linhas_visuais(text, budget, |s| mede(text_system, s))
            .into_iter()
            .map(|(a, b)| text[a..b].trim_end().to_owned())
            .collect();
    let maior = linhas
        .iter()
        .map(|l| mede(text_system, l))
        .fold(0.0, f32::max);
    let extra = passo_da_linha(text_system, px) * linhas.len().saturating_sub(1) as f32;
    DicaMedida {
        w: (maior + pad).max(60.0), // LITERAL-PX-OK: tooltip pill min width (chrome-specific)
        h: (px + 10.0 + extra).max(22.0), // LITERAL-PX-OK: tooltip pill height composite + min (chrome-specific)
        linhas,
    }
}

/// Pinta uma dica medida em `rect` (o de [`mede_a_dica`], já colocado): uma linha pelo [`paint_tooltip`] de sempre,
/// várias empilhadas e centradas, cada uma no orçamento da pílula — nenhuma corta.
pub fn paint_dica(
    dica: &DicaMedida,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    if let [uma] = dica.linhas.as_slice() {
        return paint_tooltip(
            &Tooltip::new(NodeId(0), uma),
            rect,
            scene,
            text_system,
            theme,
        );
    }
    let superficie = Superficie::de(theme);
    superficie.moldura_da_dica(scene, rect);
    let px = TypeToken::Sm.px();
    let passo = passo_da_linha(text_system, px);
    let bloco = px + passo * dica.linhas.len().saturating_sub(1) as f32;
    let y0 = rect.y + (rect.h - bloco) * 0.5;
    for (i, linha) in dica.linhas.iter().enumerate() {
        let w = text_system.prefix_width(linha, px).min(rect.w);
        paint_text_elided(
            text_system,
            scene,
            linha,
            rect.x + (rect.w - w) * 0.5,
            y0 + passo * i as f32,
            px,
            rect.w,
            superficie.cor(ColorToken::Text1),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_spec() {
        let t = Tooltip::new(NodeId(1), "Save (Ctrl+S)");
        assert_eq!(t.message, "Save (Ctrl+S)");
    }

    #[test]
    fn a11y_role_is_tooltip() {
        let node = Tooltip::new(NodeId(1), "x").build_a11y(0.0, 0.0, 100.0, 24.0);
        assert_eq!(node.role(), Role::Tooltip);
    }

    #[test]
    fn paint_smoke() {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        paint_tooltip(
            &Tooltip::new(NodeId(1), "Save (Ctrl+S)"),
            Rect::new(0.0, 0.0, 120.0, 24.0),
            &mut scene,
            &mut text,
            Theme::Forge,
        );
    }
}
