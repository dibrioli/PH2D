//! [`Card`] — surface container with optional header, body and
//! footer rects. Pure layout primitive; the consumer paints children
//! into the slot rects we expose.

use crate::paint::{fill_rounded_rect, paint_text, resolve};
use crate::zones::Rect;
use ph2d_a11y::{Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ICON_BTN_SIZE_PX, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

fn header_h() -> f32 {
    Spacing::Xl3.px()
}
const FOOTER_H: f32 = ICON_BTN_SIZE_PX;

#[derive(Clone, Debug)]
pub struct Card {
    pub id: NodeId,
    pub title: Option<String>,
    pub footer: bool,
}

impl Card {
    pub fn new(id: NodeId) -> Self {
        Self {
            id,
            title: None,
            footer: false,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn footer(mut self, yes: bool) -> Self {
        self.footer = yes;
        self
    }

    /// **A altura que a cabeça de facto OCUPA em `host`** — nunca mais que o próprio `host`.
    ///
    /// ⚠️ Ela existe porque as três portas de layout deste widget devolviam retângulos **fora**
    /// da caixa que lhes foi dada: a cabeça mede `Spacing::Xl3` (32 px) e uma row deste app mede
    /// `ROW_H_PX` (28), e `WidgetKind::Card` **é um tipo de row** do painel autorado. Medido, com
    /// `host = y 100..128`: a cabeça saía em `100..132` e o divisor dela pousava em `y = 131`,
    /// **três píxeis por cima da row de baixo**; o rodapé pousava em `y = 92`, **oito acima do
    /// topo**; e o corpo nascia em `y = 144`, dezasseis abaixo do fundo.
    ///
    /// ⚠️ **A cabeça tem PRECEDÊNCIA sobre o rodapé, e é uma escolha:** é nela que está o título,
    /// que é o que identifica o cartão. Numa caixa que não comporta os dois, quem cede é o rodapé.
    fn header_h_in(&self, host: Rect) -> f32 {
        if self.title.is_some() {
            header_h().min(host.h.max(0.0))
        } else {
            0.0
        }
    }

    /// A altura que o rodapé ocupa — o que sobra depois da cabeça.
    fn footer_h_in(&self, host: Rect) -> f32 {
        if self.footer {
            FOOTER_H.min((host.h.max(0.0) - self.header_h_in(host)).max(0.0))
        } else {
            0.0
        }
    }

    pub fn header_rect(&self, host: Rect) -> Option<Rect> {
        if self.title.is_some() {
            Some(Rect::new(host.x, host.y, host.w, self.header_h_in(host)))
        } else {
            None
        }
    }

    pub fn footer_rect(&self, host: Rect) -> Option<Rect> {
        if self.footer {
            let h = self.footer_h_in(host);
            Some(Rect::new(host.x, host.y + host.h - h, host.w, h))
        } else {
            None
        }
    }

    /// Body slot (between header and footer if either is present).
    ///
    /// ⚠️ **O `.max(0.0)` da altura salvava o TAMANHO e não a POSIÇÃO:** com a cabeça a comer a
    /// caixa inteira o corpo nascia dezasseis píxeis abaixo do fundo, com altura zero — e o
    /// [`push_card_body_clip`] empurrava um recorte degenerado **fora** da caixa, onde tudo o que o
    /// consumidor pintasse era cortado para nada. Por isso o topo é limitado ao fundo disponível.
    pub fn body_rect(&self, host: Rect) -> Rect {
        let top = host.y + self.header_h_in(host);
        let bot = host.y + host.h - self.footer_h_in(host);
        let pad = Spacing::Lg.px();
        Rect::new(
            host.x + pad,
            (top + pad).min(bot),
            (host.w - pad * 2.0).max(0.0),
            (bot - top - pad * 2.0).max(0.0),
        )
    }

    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        let mut builder = NodeBuilder::new(Role::Group).bounds(x, y, w, h);
        if let Some(title) = &self.title {
            builder = builder.label(title);
        } else {
            builder = builder.label("card");
        }
        builder.build()
    }
}

pub fn paint_card(
    card: &Card,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    // ⭐ Raio e moldura pela porta do TEMA: o cartão de 12 px com contorno é do clássico; num
    //    tema moderno é uma superfície de raio 4 sem moldura (*«os cartões ainda desenham
    //    moldura»* — `pesquisa/07 §22.3`).
    let radius = crate::paint::frame_radius(theme, Radius::Lg.px());
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::Bg2, theme));
    crate::paint::stroke_frame(
        scene,
        rect,
        radius,
        theme,
        ph2d_tokens::visuals::Feel::Rest,
        1.0,
        resolve(ColorToken::Border, theme),
    );

    if let (Some(header), Some(title)) = (card.header_rect(rect), &card.title) {
        let pad = Spacing::Lg.px();
        let font = TypeToken::Md.px();
        let y = header.y + (header.h - font) * 0.5;
        paint_text(
            text_system,
            scene,
            title,
            header.x + pad,
            y,
            font,
            (header.w - pad * 2.0).max(0.0),
            resolve(ColorToken::Text1, theme),
        );
        // 1-px divider between header and body, inset so the line
        // doesn't run into the rounded corners and create the same
        // ear artifact tracked in `docs/UI_Bugs/README.md` §3.1.
        let pad_x = Spacing::Md.px();
        let div_rect = Rect::new(
            rect.x + pad_x,
            header.y + header.h - 1.0,
            rect.w - pad_x * 2.0,
            1.0,
        );
        fill_rounded_rect(scene, div_rect, 0.5, resolve(ColorToken::Border, theme));
    }
    if let Some(footer) = card.footer_rect(rect) {
        let pad_x = Spacing::Md.px();
        let div_rect = Rect::new(rect.x + pad_x, footer.y, rect.w - pad_x * 2.0, 1.0);
        fill_rounded_rect(scene, div_rect, 0.5, resolve(ColorToken::Border, theme));
    }
}

/// Push a clip layer matching `card.body_rect(host)` so consumer
/// content painted between this call and [`pop_card_body_clip`] is
/// kept inside the body slot and can't leak past the rounded chrome.
pub fn push_card_body_clip(card: &Card, host: Rect, scene: &mut VectorScene) {
    let body = card.body_rect(host);
    let clip = ph2d_vector::Rect::new(
        body.x as f64,
        body.y as f64,
        (body.x + body.w) as f64,
        (body.y + body.h) as f64,
    );
    scene.push_clip(&clip);
}

/// Pop the clip layer pushed by [`push_card_body_clip`].
pub fn pop_card_body_clip(scene: &mut VectorScene) {
    scene.pop_layer();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Nada que o cartao PUBLICA sai da caixa que lhe foi dada.**
    ///
    /// ⚠️ Isto e' alcancavel pelo artista HOJE: `WidgetKind::Card` e' um tipo de row do painel
    /// autorado, uma row mede `ROW_H_PX` (28) e a cabeca mede `Spacing::Xl3` (32). Medido antes da
    /// cura, com `host = y 100..128`: a cabeca saia em `100..132` e o divisor dela pousava em
    /// `y = 131` -- tres pixeis por cima da row de baixo -- o rodape em `y = 92`, oito acima do
    /// topo, e o corpo em `y = 144`, dezasseis abaixo do fundo.
    ///
    /// ⚠️ **A varredura de alturas e' o gate**, e nao uma caixa escolhida: o joelho de cada porta
    /// esta' num sitio diferente (a cabeca cede a 32, o rodape a 36, o corpo ao par mais o
    /// enchimento), entao uma altura so' mediria uma delas e ficaria verde sobre as outras duas.
    /// A caixa GRANDE e' o controle -- ela ja' passava, e e' sobre ela que o defeito nao aparecia.
    #[test]
    fn every_slot_the_card_publishes_stays_inside_the_box() {
        for h in [0.0_f32, 1.0, 12.0, 28.0, 32.0, 36.0, 60.0, 80.0, 200.0] {
            let host = Rect::new(10.0, 100.0, 200.0, h);
            for card in [
                Card::new(NodeId(1)).title("Card"),
                Card::new(NodeId(1)).footer(true),
                Card::new(NodeId(1)).title("Card").footer(true),
            ] {
                let dentro = |r: Rect, nome: &str| {
                    assert!(
                        r.y >= host.y - 0.001
                            && r.y + r.h <= host.y + host.h + 0.001
                            && r.x >= host.x - 0.001
                            && r.x + r.w <= host.x + host.w + 0.001,
                        "h={h}: o {nome} ({:.1}..{:.1}) sai da caixa ({:.1}..{:.1})",
                        r.y,
                        r.y + r.h,
                        host.y,
                        host.y + host.h
                    );
                };
                if let Some(r) = card.header_rect(host) {
                    dentro(r, "cabecalho");
                }
                if let Some(r) = card.footer_rect(host) {
                    dentro(r, "rodape");
                }
                dentro(card.body_rect(host), "corpo");
            }
        }
    }

    /// **A cabeca e o rodape nao se sobrepoem** -- numa caixa que nao comporta os dois, cede o
    /// rodape.
    ///
    /// ⚠️ Sem ele, clampar cada um ao `host` isoladamente ficaria verde no gate acima e desenharia
    /// os dois divisores um sobre o outro: cada retangulo caberia na caixa, e ainda assim o
    /// desenho estaria errado. E a PRECEDENCIA e' uma escolha -- o titulo e' o que identifica o
    /// cartao --, entao ela e' afirmada e nao deixada ao acaso da ordem das linhas.
    #[test]
    fn the_header_wins_the_room_and_the_footer_takes_what_is_left() {
        let host = Rect::new(0.0, 0.0, 200.0, 40.0);
        let card = Card::new(NodeId(1)).title("Card").footer(true);
        let h = card.header_rect(host).expect("com titulo");
        let f = card.footer_rect(host).expect("com rodape");
        assert!(
            (h.y + h.h) <= f.y + 0.001,
            "a cabeca ({:.1}..{:.1}) invade o rodape ({:.1}..{:.1})",
            h.y,
            h.y + h.h,
            f.y,
            f.y + f.h
        );
        assert!(
            h.h > f.h,
            "numa caixa apertada quem cede tem de ser o rodape: cabeca {:.1}, rodape {:.1}",
            h.h,
            f.h
        );
    }

    #[test]
    fn header_rect_only_with_title() {
        let host = Rect::new(0.0, 0.0, 200.0, 200.0);
        assert!(Card::new(NodeId(1)).header_rect(host).is_none());
        assert!(Card::new(NodeId(1)).title("x").header_rect(host).is_some());
    }

    #[test]
    fn footer_rect_only_when_enabled() {
        let host = Rect::new(0.0, 0.0, 200.0, 200.0);
        assert!(Card::new(NodeId(1)).footer_rect(host).is_none());
        assert!(
            Card::new(NodeId(1))
                .footer(true)
                .footer_rect(host)
                .is_some()
        );
    }

    #[test]
    fn body_rect_inset_for_padding() {
        let host = Rect::new(0.0, 0.0, 200.0, 200.0);
        let body = Card::new(NodeId(1)).body_rect(host);
        assert!(body.w < host.w);
        assert!(body.h < host.h);
    }

    #[test]
    fn body_rect_excludes_header_and_footer() {
        let host = Rect::new(0.0, 0.0, 200.0, 200.0);
        let card = Card::new(NodeId(1)).title("x").footer(true);
        let body = card.body_rect(host);
        assert!(body.y > host.y + header_h() * 0.5);
        assert!(body.y + body.h < host.y + host.h - FOOTER_H * 0.5);
    }

    #[test]
    fn a11y_role_is_group() {
        let node = Card::new(NodeId(1))
            .title("Inspector")
            .build_a11y(0.0, 0.0, 200.0, 200.0);
        assert_eq!(node.role(), Role::Group);
    }

    fn smoke(card: Card, theme: Theme) {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        paint_card(
            &card,
            Rect::new(0.0, 0.0, 240.0, 200.0),
            &mut scene,
            &mut text,
            theme,
        );
    }

    #[test]
    fn paint_smoke_plain() {
        smoke(Card::new(NodeId(1)), Theme::Forge);
    }

    #[test]
    fn paint_smoke_with_header() {
        smoke(Card::new(NodeId(1)).title("Inspector"), Theme::Sunstone);
    }

    #[test]
    fn paint_smoke_with_footer() {
        smoke(
            Card::new(NodeId(1)).title("Project").footer(true),
            Theme::Blueprint,
        );
    }
}
