//! [`Tag`] (a.k.a. Chip) — small pill carrying a label and an
//! optional close affordance.
//!
//! Used for filters, attached labels, and active-search tokens.
//! `Removable` tags expose `Action::Click` and a Close icon at the
//! right edge; non-removable tags read as `Role::Label`.

use crate::icons::IconId;
use crate::paint::{fill_rounded_rect, paint_icon, paint_text_centered, resolve};
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum TagState {
    #[default]
    Normal,
    Hovered,
    Pressed,
    Disabled,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum TagTone {
    /// Default chip — `Bg2` background.
    #[default]
    Neutral,
    /// `AccentSoft` background — for active filters.
    Accent,
    /// `SuccessSoft` — for "ok" tokens.
    Success,
    /// `WarnSoft` — for "warning" tokens.
    Warn,
    /// `DangerSoft` — for "blocked / error" tokens.
    Danger,
}

#[derive(Clone, Debug)]
pub struct Tag {
    pub id: NodeId,
    pub label: String,
    pub state: TagState,
    pub tone: TagTone,
    pub removable: bool,
    /// ⚠️ **Campo com NEUTRO** (`1.0`), o molde do [`crate::widget::Button::hover_t`]: quem não o
    /// define não sabe que ele existe, e pinta o que pintava antes.
    pub hover_t: f32,
}

impl Tag {
    pub fn new(id: NodeId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            state: TagState::Normal,
            tone: TagTone::Neutral,
            removable: false,
            hover_t: crate::motion::SETTLED,
        }
    }

    pub fn state(mut self, state: TagState) -> Self {
        self.state = state;
        self
    }

    /// **As DUAS metades numa chamada** — o par que o store entrega, e o irmão exacto do
    /// [`crate::widget::Button::visual`].
    #[must_use]
    pub fn visual(self, v: (TagState, f32)) -> Self {
        self.state(v.0).hover_t(v.1)
    }

    /// **Quanto do hover está presente**, `0..1`. Neutro = [`crate::motion::SETTLED`].
    #[must_use]
    pub fn hover_t(mut self, t: f32) -> Self {
        self.hover_t = t.clamp(0.0, 1.0);
        self
    }

    pub fn tone(mut self, tone: TagTone) -> Self {
        self.tone = tone;
        self
    }

    pub fn removable(mut self, yes: bool) -> Self {
        self.removable = yes;
        self
    }

    /// Build the AccessKit node. Removable tags expose Click; static
    /// tags are pure `Role::Label`.
    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        let mut builder = NodeBuilder::new(Role::Label)
            .label(&self.label)
            .bounds(x, y, w, h);
        if self.removable {
            builder = builder
                .focusable(self.state != TagState::Disabled)
                .action(Action::Click);
        }
        builder.build()
    }

    /// Rect of the close `X` icon inside `host` (or `None` for
    /// non-removable tags). Hosts use this to register a dedicated
    /// hit zone for the close action so a click on the X removes
    /// the tag rather than activating the whole pill.
    pub fn close_rect(&self, host: Rect) -> Option<Rect> {
        if !self.removable {
            return None;
        }
        let pad_x = (host.h * 0.5).max(8.0); // LITERAL-PX-OK: tag pill horizontal pad scales with height (chrome geometry)
        let close_size = (host.h * 0.7).clamp(10.0, 16.0); // LITERAL-PX-OK: close icon scales 70% of pill height with min/max
        // ⚠️ O `X` mora numa ESQUINA e o host é variável: `pad_x + close_size` pode passar da
        // largura da pílula, e aí a borda esquerda do ícone cai FORA dela — por cima do rótulo,
        // que é o vizinho da esquerda. O piso é `host.x`; num host mais estreito que o próprio
        // ícone nada cabe, e transbordar pela DIREITA (na borda da pílula) é o menor dos males.
        Some(Rect::new(
            (host.x + host.w - pad_x - close_size).max(host.x),
            host.y + (host.h - close_size) * 0.5,
            close_size,
            close_size,
        ))
    }

    fn bg_token(&self) -> ColorToken {
        if self.state == TagState::Disabled {
            return ColorToken::Bg2;
        }
        match self.tone {
            TagTone::Neutral => ColorToken::Bg2,
            TagTone::Accent => ColorToken::AccentSoft,
            TagTone::Success => ColorToken::SuccessSoft,
            TagTone::Warn => ColorToken::WarnSoft,
            TagTone::Danger => ColorToken::DangerSoft,
        }
    }

    fn fg_token(&self) -> ColorToken {
        if self.state == TagState::Disabled {
            return ColorToken::TextDisabled;
        }
        match self.tone {
            TagTone::Neutral => ColorToken::Text2,
            TagTone::Accent => ColorToken::Accent,
            TagTone::Success => ColorToken::Success,
            TagTone::Warn => ColorToken::Warn,
            TagTone::Danger => ColorToken::Danger,
        }
    }

    /// **A cor do ANEL de hover** — `None` quando não há anel a desenhar.
    ///
    /// ⚠️ **O par é `rest = None` / `hot = fg`, e o primitivo já sabe o que isso significa:** o
    /// [`crate::motion::blend_token_color`] trata um lado ausente como **transparente**, não como
    /// a outra cor, *"para o hover EMERGIR do nada em vez de aparecer de repente"* — a lei que o
    /// fundo de um botão *ghost* já usa. Uma tag em repouso não tem anel; o anel dela é
    /// exactamente esse caso, e por isso esta wave **não escreve lei nova**.
    ///
    /// ⚠️ **`a == 0` devolve `None` de propósito:** um traço transparente ainda é um comando de
    /// cena, e devolvê-lo faria toda tag PARADA pagar um stroke por quadro.
    #[must_use]
    pub fn ring_color(&self, theme: Theme) -> Option<ph2d_tokens::Color> {
        let hot = self.fg_token().resolve(theme);
        let soft = matches!(self.state, TagState::Normal | TagState::Hovered);
        if let Some(c) = crate::motion::hover_axis(soft, self.hover_t, None, Some(hot)) {
            return (c.a > 0).then_some(c);
        }
        // O token DURO: assente, quem manda é o estado.
        (self.state == TagState::Hovered).then_some(hot)
    }
}

/// Pill body + label + optional close icon. Label is centered when
/// `!removable`; left-aligned with the close icon at right when
/// removable.
pub fn paint_tag(
    tag: &Tag,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    // ⭐ A pílula é do CLÁSSICO: num tema moderno a etiqueta é um rectângulo de raio 4, como no
    //    Godot (e como o dono pediu — *«etiquetas ainda são pílulas»* estava na lista do que
    //    destoava, `pesquisa/07 §22.3`).
    let radius = crate::paint::frame_radius(theme, Radius::Full.px());
    let bg = resolve(tag.bg_token(), theme);
    let fg = resolve(tag.fg_token(), theme);
    fill_rounded_rect(scene, rect, radius, bg);
    // ⚠️ **Este comentário dizia o oposto, e foi a MINHA wave anterior que o falsificou:** ele
    // afirmava que *"o `skin::paint_widget_skin_with` recebe `live: Option<&InteractiveState>` e
    // não tem store — não há de onde tirar um `t`"*, e o passo que fez a pele receber o PAR
    // apagou esse bloqueio sem ninguém reconferir a nota. É o §0 a morder em casa: *quem move o
    // número que tornava algo inalcançável tem de reconferir a nota.*
    if let Some(ring) = tag.ring_color(theme) {
        let ring = crate::paint::token_to_vello(ring);
        crate::paint::stroke_frame(
            scene,
            rect,
            radius,
            theme,
            ph2d_tokens::visuals::Feel::Hovered,
            1.0, // LITERAL-PX-OK: tag hover ring stroke (geometry 1px)
            ring,
        );
    }

    let pad_x = (rect.h * 0.5).max(8.0); // LITERAL-PX-OK: pill horizontal pad scales with height (geometry)
    if let Some(close_rect) = tag.close_rect(rect) {
        let label_rect = Rect::new(
            rect.x + pad_x,
            rect.y,
            (close_rect.x - rect.x - pad_x * 1.5).max(0.0), // LITERAL-PX-OK: label width budget composite (mirrors close-rect math)
            rect.h,
        );
        paint_text_centered(
            text_system,
            scene,
            &tag.label,
            label_rect,
            TypeToken::Xs.px(),
            fg,
        );
        paint_icon(
            scene,
            IconId::Close,
            close_rect,
            fg,
            StrokeToken::Default.px(),
        );
    } else {
        paint_text_centered(text_system, scene, &tag.label, rect, TypeToken::Xs.px(), fg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ **O NEUTRO é o mundo pré-substrato, ao bit** — e é a metade do gate que impede a wave de
    /// ter custo onde ninguém a pediu.
    ///
    /// Uma tag construída sem tocar no `hover_t` chega aqui com [`crate::motion::SETTLED`], e a
    /// guarda do `hover_axis` manda-a para o token DURO: em repouso não há anel, em hover há um
    /// anel cheio em `fg`. É exactamente o que o `if tag.state == Hovered` fazia antes.
    ///
    /// ⚠️ **A tag da HIERARQUIA é o consumidor que isto protege:** ela é pintada com `NodeId(0)` —
    /// um crachá decorativo que nunca entra no store —, então ela nunca terá `t` nenhum, e tem de
    /// continuar a pintar o que pintava.
    ///
    /// *Mutação que deve sangrar:* tirar a guarda `t >= SETTLED` do `hover_axis` (a tag em repouso
    /// passa a pintar um anel cheio, porque `blend(None, fg, 1.0)` é `fg`).
    #[test]
    fn a_tag_that_never_met_the_clock_paints_the_hard_token() {
        let theme = Theme::Forge;
        let rest = Tag::new(NodeId(1), "x");
        assert!(
            rest.ring_color(theme).is_none(),
            "uma tag em repouso ganhou anel — o neutro deixou de ser byte-identico"
        );
        let hot = Tag::new(NodeId(1), "x").state(TagState::Hovered);
        assert_eq!(
            hot.ring_color(theme),
            Some(hot.fg_token().resolve(theme)),
            "o anel assente tem de ser o `fg` CHEIO, como antes do substrato"
        );
        for hard in [TagState::Pressed, TagState::Disabled] {
            assert!(
                Tag::new(NodeId(1), "x")
                    .state(hard)
                    .hover_t(0.5)
                    .ring_color(theme)
                    .is_none(),
                "{hard:?} nao e uma FRACCAO de nada — nao pode entrar no eixo"
            );
        }
    }

    /// **E meio caminho é meio anel** — o eixo existe e emerge do NADA.
    ///
    /// ⚠️ O par é `rest = None` / `hot = fg`, então o primitivo devolve `fade(fg, t)`: a mesma cor,
    /// com alfa. Um anel que nascesse na cor da pílula deixaria meia-espessura visível sobre o
    /// painel (o traço é centrado na fronteira), que é a razão de a resposta certa ser a que o
    /// `blend_token_color` já dá a um botão *ghost*.
    #[test]
    fn half_a_hover_is_half_a_ring() {
        let theme = Theme::Forge;
        let full = Tag::new(NodeId(1), "x")
            .state(TagState::Hovered)
            .ring_color(theme)
            .expect("assente no hover ha anel");
        let mid = Tag::new(NodeId(1), "x")
            .state(TagState::Hovered)
            .hover_t(0.5)
            .ring_color(theme)
            .expect("a meio caminho ha anel");
        assert_eq!(
            (mid.r, mid.g, mid.b),
            (full.r, full.g, full.b),
            "o anel mudou de COR a meio caminho — ele so devia estar a emergir"
        );
        assert!(
            mid.a > 0 && mid.a < full.a,
            "alfa {} nao esta entre 0 e {} — o `t` foi deitado fora",
            mid.a,
            full.a
        );
        assert!(
            Tag::new(NodeId(1), "x")
                .state(TagState::Normal)
                .hover_t(0.0)
                .ring_color(theme)
                .is_none(),
            "frio e frio: um traco transparente e um comando de cena que ninguem ve"
        );
    }

    #[test]
    fn defaults_match_spec() {
        let t = Tag::new(NodeId(1), "filter");
        assert_eq!(t.state, TagState::Normal);
        assert_eq!(t.tone, TagTone::Neutral);
        assert!(!t.removable);
    }

    #[test]
    fn a11y_static_tag_has_label_role() {
        let node = Tag::new(NodeId(1), "x").build_a11y(0.0, 0.0, 60.0, 24.0);
        assert_eq!(node.role(), Role::Label);
    }

    #[test]
    fn close_rect_only_for_removable() {
        let host = Rect::new(0.0, 0.0, 80.0, 22.0);
        assert!(Tag::new(NodeId(1), "x").close_rect(host).is_none());
        let r = Tag::new(NodeId(1), "x")
            .removable(true)
            .close_rect(host)
            .unwrap();
        // close X must sit at the right edge of the pill.
        assert!(r.x > host.x + host.w * 0.5);
    }

    #[test]
    fn a11y_removable_tag_supports_click() {
        let node = Tag::new(NodeId(1), "x")
            .removable(true)
            .build_a11y(0.0, 0.0, 60.0, 24.0);
        assert!(node.supports_action(Action::Click));
    }

    fn smoke(tag: Tag, theme: Theme) {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        paint_tag(
            &tag,
            Rect::new(0.0, 0.0, 80.0, 22.0),
            &mut scene,
            &mut text,
            theme,
        );
    }

    #[test]
    fn paint_smoke_neutral() {
        smoke(Tag::new(NodeId(1), "x"), Theme::Forge);
    }

    #[test]
    fn paint_smoke_accent_removable() {
        smoke(
            Tag::new(NodeId(1), "x")
                .tone(TagTone::Accent)
                .removable(true),
            Theme::Sunstone,
        );
    }

    #[test]
    fn paint_smoke_warn_hovered() {
        smoke(
            Tag::new(NodeId(1), "x")
                .tone(TagTone::Warn)
                .state(TagState::Hovered),
            Theme::Blueprint,
        );
    }

    #[test]
    fn paint_smoke_danger_pressed() {
        smoke(
            Tag::new(NodeId(1), "x")
                .tone(TagTone::Danger)
                .state(TagState::Pressed),
            Theme::Workshop,
        );
    }

    #[test]
    fn paint_smoke_disabled() {
        smoke(
            Tag::new(NodeId(1), "x").state(TagState::Disabled),
            Theme::Forge,
        );
    }

    /// **O `X` de uma tag nunca começa à esquerda da pílula.**
    ///
    /// ⚠️ O ícone mora numa ESQUINA e o host é variável: `pad_x + close_size` cresce com a ALTURA
    /// da pílula, não com a largura, então uma tag baixa e estreita punha a borda esquerda do `X`
    /// fora — por cima do rótulo, que é o vizinho da esquerda. O oráculo é a contenção do lado
    /// que colide, não a fórmula.
    #[test]
    fn the_close_icon_never_starts_left_of_the_pill() {
        let t = Tag::new(NodeId(1), "L").removable(true);
        for (hw, hh) in [
            (80.0_f32, 22.0_f32),
            (30.0, 22.0),
            (16.0, 22.0),
            (4.0, 40.0),
        ] {
            let host = Rect::new(10.0, 5.0, hw, hh);
            let r = t.close_rect(host).expect("removivel tem X");
            assert!(
                r.x >= host.x - 1e-3,
                "host {hw}x{hh}: o X comeca em {}, a esquerda da pilula ({})",
                r.x,
                host.x
            );
        }
    }
}
