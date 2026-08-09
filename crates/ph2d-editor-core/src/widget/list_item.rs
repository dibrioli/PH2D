//! [`ListItem`] — single row in a vertical list (asset browser,
//! hierarchy panel, context menus).
//!
//! Layout: optional left icon + label + optional right value text +
//! optional chevron. Selected rows fill with `AccentSoft`. Density
//! is consumer-driven (the host sets row height via `ph2d-tokens`
//! `ROW_H_PX[Density]`).

use crate::icons::IconId;
use crate::paint::{fill_rounded_rect, paint_icon, paint_text, resolve};
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum ListItemState {
    #[default]
    Normal,
    Hovered,
    Pressed,
    Focused,
    Disabled,
}

#[derive(Clone, Debug)]
pub struct ListItem {
    pub id: NodeId,
    pub label: String,
    /// Right-aligned secondary text (key shortcut, file size, etc).
    pub value: Option<String>,
    pub leading_icon: Option<IconId>,
    pub trailing_chevron: bool,
    pub selected: bool,
    pub state: ListItemState,
}

impl ListItem {
    pub fn new(id: NodeId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            value: None,
            leading_icon: None,
            trailing_chevron: false,
            selected: false,
            state: ListItemState::Normal,
        }
    }

    pub fn icon(mut self, icon: IconId) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = Some(v.into());
        self
    }

    pub fn chevron(mut self, yes: bool) -> Self {
        self.trailing_chevron = yes;
        self
    }

    pub fn selected(mut self, yes: bool) -> Self {
        self.selected = yes;
        self
    }

    pub fn state(mut self, state: ListItemState) -> Self {
        self.state = state;
        self
    }

    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        NodeBuilder::new(Role::ListItem)
            .label(&self.label)
            .bounds(x, y, w, h)
            .focusable(self.state != ListItemState::Disabled)
            .action(Action::Click)
            .build()
    }
}

pub fn paint_list_item(
    item: &ListItem,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let radius = Radius::Xs.px();
    let bg = match (item.selected, item.state) {
        (true, _) => Some(ColorToken::AccentSoft),
        (false, ListItemState::Hovered | ListItemState::Focused) => Some(ColorToken::Bg2),
        (false, ListItemState::Pressed) => Some(ColorToken::AccentPress),
        _ => None,
    };
    if let Some(token) = bg {
        fill_rounded_rect(scene, rect, radius, resolve(token, theme));
    }

    let pad_x = Spacing::Lg.px();
    let icon_w = (rect.h * 0.6).clamp(14.0, 20.0); // LITERAL-PX-OK: list icon sized 60% of row height with min/max
    let mut cursor_x = rect.x + pad_x;
    if let Some(icon) = item.leading_icon {
        let icon_rect = Rect::new(cursor_x, rect.y + (rect.h - icon_w) * 0.5, icon_w, icon_w);
        let icon_color = if item.state == ListItemState::Disabled {
            ColorToken::TextDisabled
        } else if item.selected {
            ColorToken::Accent
        } else {
            ColorToken::Text2
        };
        paint_icon(
            scene,
            icon,
            icon_rect,
            resolve(icon_color, theme),
            StrokeToken::Default.px(),
        );
        cursor_x += icon_w + Spacing::Md.px();
    }

    let chevron_w = if item.trailing_chevron { icon_w } else { 0.0 };
    // Real text measurement for the right-aligned value pill — the
    // `len * 7` heuristic underestimated wide glyphs and overlapped
    // the label on proportional fonts (`docs/UI_Bugs/README.md` §3.3).
    let value_font = TypeToken::Sm.px();
    // ⚠️ **A medição é ilimitada e a row NÃO é.** `layout(.., INFINITY)` devolve a largura que o
    // texto QUER, e o pilar é ancorado à direita (`rect.w - pad - chevron - value_w`), então um
    // valor comprido faz o `x` ficar NEGATIVO em relação à row: medido, 48 caracteres numa row de
    // 200 px começam **337 px à esquerda dela**, e um valor realista de 40 caracteres já começa 54
    // px fora — por cima de tudo o que estiver ao lado.
    //
    // O teto é o que sobra da row depois do ícone e do chevron; com ele o `paint_text` quebra o
    // texto na borda em vez de o deixar sair, e o rótulo à esquerda colapsa a zero (ele já tinha
    // `.max(0.0)`). Um valor que CABE é byte-idêntico — o teto só morde quando ele não caberia.
    let value_room = (rect.x + rect.w - pad_x - chevron_w - cursor_x).max(0.0);
    let value_w = value_pill_width_in(item, value_room, value_font, text_system);
    let label_w =
        (rect.x + rect.w - cursor_x - pad_x - chevron_w - value_w - Spacing::Lg.px()).max(0.0);
    let font_size = TypeToken::Base.px();
    let label_y = rect.y + (rect.h - font_size) * 0.5;
    let label_color = if item.state == ListItemState::Disabled {
        ColorToken::TextDisabled
    } else {
        ColorToken::Text1
    };
    // ⚠️ **O recorte é a outra metade, e a lição é do `TextInput`:** o `paint_text` QUEBRA o texto
    // no `max_width` em vez de o cortar, então um teto sozinho troca o transbordo horizontal por
    // uma segunda linha desenhada ABAIXO da row, sobre a row seguinte. Numa row cujo texto cabe
    // isto é no-op.
    scene.push_clip(&crate::paint::rect_to_vello(rect));
    paint_text(
        text_system,
        scene,
        &item.label,
        cursor_x,
        label_y,
        font_size,
        label_w,
        resolve(label_color, theme),
    );

    if let Some(value) = &item.value {
        let value_x = rect.x + rect.w - pad_x - chevron_w - value_w;
        paint_text(
            text_system,
            scene,
            value,
            value_x,
            label_y,
            value_font,
            value_w,
            resolve(ColorToken::Text2, theme),
        );
    }

    if item.trailing_chevron {
        let chev_rect = Rect::new(
            rect.x + rect.w - pad_x - icon_w,
            rect.y + (rect.h - icon_w) * 0.5,
            icon_w,
            icon_w,
        );
        paint_icon(
            scene,
            IconId::ChevronRight,
            chev_rect,
            resolve(ColorToken::Text3, theme),
            StrokeToken::Default.px(),
        );
    }
    // Fecha o recorte aberto antes do rótulo — o chevron entra DENTRO dele de propósito: ele é
    // desenhado na borda da row, e um chevron que saísse dela seria o mesmo defeito com outra
    // tinta.
    scene.pop_layer();
}

/// **A largura que o pilar do valor ocupa** — o querido pelo texto, aparado pelo que sobra.
///
/// ⚠️ Porta própria para o gate poder PERGUNTAR em vez de re-derivar a aritmética do pintor: um
/// oráculo que recomputa a fórmula concorda com ela estando as duas erradas.
fn value_pill_width_in(item: &ListItem, room: f32, font: f32, text_system: &mut TextSystem) -> f32 {
    item.value
        .as_ref()
        .map(|v| text_system.layout(v, font, f32::INFINITY).width().min(room))
        .unwrap_or(0.0)
}

/// A folga que sobra para o valor numa row — o que vem depois do ícone e antes do chevron.
#[cfg(test)]
fn value_pill_room(item: &ListItem, rect: Rect) -> f32 {
    let pad_x = Spacing::Lg.px();
    // A MESMA aritmética do pintor para o que vem ANTES do valor — ela não é o que o gate mede
    // (o gate mede a borda), é só a fixture chegar ao mesmo `room`.
    let icon_w = (rect.h * 0.6).clamp(14.0, 20.0); // LITERAL-PX-OK: espelha o pintor, linha 106
    let chevron_w = if item.trailing_chevron { icon_w } else { 0.0 };
    let cursor_x = rect.x
        + pad_x
        + if item.leading_icon.is_some() {
            icon_w + Spacing::Md.px()
        } else {
            0.0
        };
    (rect.x + rect.w - pad_x - chevron_w - cursor_x).max(0.0)
}

/// A largura que o pilar ocupa nessa folga — o que o gate compara.
#[cfg(test)]
fn value_pill_width(item: &ListItem, rect: Rect, text_system: &mut TextSystem) -> f32 {
    let room = value_pill_room(item, rect);
    value_pill_width_in(item, room, TypeToken::Sm.px(), text_system)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> ListItem {
        ListItem::new(NodeId(1), "Brush")
    }

    #[test]
    fn defaults_match_spec() {
        let i = fixture();
        assert!(i.value.is_none());
        assert!(i.leading_icon.is_none());
        assert!(!i.selected);
        assert!(!i.trailing_chevron);
    }

    #[test]
    fn builder_chain_sets_fields() {
        let i = fixture()
            .icon(IconId::Sprite)
            .value("Cmd+B")
            .chevron(true)
            .selected(true);
        assert_eq!(i.leading_icon, Some(IconId::Sprite));
        assert_eq!(i.value.as_deref(), Some("Cmd+B"));
        assert!(i.trailing_chevron);
        assert!(i.selected);
    }

    #[test]
    fn a11y_role_is_list_item() {
        let node = fixture().build_a11y(0.0, 0.0, 200.0, 26.0);
        assert_eq!(node.role(), Role::ListItem);
    }

    fn smoke(item: ListItem, theme: Theme) {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        paint_list_item(
            &item,
            Rect::new(0.0, 0.0, 240.0, 26.0),
            &mut scene,
            &mut text,
            theme,
        );
    }

    #[test]
    fn paint_smoke_plain() {
        smoke(fixture(), Theme::Forge);
    }

    #[test]
    fn paint_smoke_full_decoration() {
        smoke(
            fixture()
                .icon(IconId::Sprite)
                .value("Cmd+B")
                .chevron(true)
                .selected(true),
            Theme::Sunstone,
        );
    }

    #[test]
    fn paint_smoke_hovered() {
        smoke(fixture().state(ListItemState::Hovered), Theme::Blueprint);
    }

    #[test]
    fn paint_smoke_disabled() {
        smoke(fixture().state(ListItemState::Disabled), Theme::Workshop);
    }

    /// **O pilar do valor nunca começa à esquerda da row** — e um valor que CABE não se move.
    ///
    /// ⚠️ A medição é ilimitada (`layout(.., INFINITY)`) e a âncora é a borda DIREITA, então o
    /// `x` do valor é `rect.w − pad − chevron − largura_medida`: quanto mais comprido o texto,
    /// mais à ESQUERDA ele começa. Medido antes do teto, numa row de 200 px: 40 caracteres
    /// começavam 54 px fora, 48 caracteres começavam **337 px fora**.
    ///
    /// ⚠️ **O oráculo é a GEOMETRIA que a função devolve, não a aritmética dela** — `value_x` é
    /// re-derivado aqui a partir da largura que o pintor de facto usa, e a asserção é sobre a
    /// borda. E a segunda metade é o CONTROLE: um valor curto tem de medir exactamente a largura
    /// que o texto quer, senão o teto estaria a apertar quem cabe.
    #[test]
    fn the_value_pill_never_starts_left_of_the_row() {
        let mut ts = TextSystem::without_system_fonts();
        let font = TypeToken::Sm.px();
        let pad_x = Spacing::Lg.px();
        let rect = Rect::new(10.0, 0.0, 200.0, 28.0);

        // ⚠️ **A fixture TEM de conter o fenômeno, e a asserção abaixo prova que contém.** A
        // primeira versão deste gate levava só `"opaque"` (43 px) como valor curto, e a mutação
        // *"o teto aperta também quem cabe"* (`room * 0.5`) **SOBREVIVEU** — 43 px passa por
        // baixo de meio teto sem ser tocado. O controle só tem dentes com um valor que caiba na
        // row **e** exceda uma folga apertada.
        let mut saw_a_snug_value = false;
        for value in [
            "opaque",
            "um valor de tamanho medio",
            "um valor bastante comprido para uma row",
            "MMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMM",
        ] {
            let item = ListItem::new(NodeId(1), "Layer").value(value);
            let wanted = ts.layout(value, font, f32::INFINITY).width();
            let used = value_pill_width(&item, rect, &mut ts);
            let x = rect.x + rect.w - pad_x - used;
            assert!(
                x >= rect.x - 1e-3,
                "{value:?}: o valor comeca em {x}, a esquerda da row ({})",
                rect.x
            );
            let room = value_pill_room(&item, rect);
            if wanted <= room {
                assert!(
                    (used - wanted).abs() < 1e-3,
                    "{value:?}: cabia em {wanted} px (folga {room}) e o teto apertou-o para {used}"
                );
                if wanted > room * 0.5 {
                    saw_a_snug_value = true;
                }
            }
        }
        assert!(
            saw_a_snug_value,
            "nenhum valor da fixture cabe na row E ocupa mais de metade dela — o controle passaria \
             por baixo de um teto pela METADE, que foi a mutacao que sobreviveu a' 1a versao"
        );
    }
}
