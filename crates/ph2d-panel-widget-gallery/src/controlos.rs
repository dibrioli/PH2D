//! ⭐ **A TABELA dos controlos** (doc `CartoesUI/08` §1.1, «zero no-op silencioso»): cada botão, ícone,
//! segmento e opção da galeria é UMA linha daqui — o `populate` regista por ela, o `prepare` dá a
//! dica por ela, a pintura tira dela o ícone e o aceso, e o evento age por ela.

use crate::ids as g;
use crate::state::{Choices, Flag, Pick};
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::IconId;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::cards::{CardBody, CardCtx, CardDef, HeadAction, Seg, Widget};
use ph2d_editor_core::widget::card_field::Glyph;
use ph2d_i18n::tr;
use ph2d_tokens::ColorToken;

/// Uma ação de um clique sobre o objeto fictício ([`crate::valores`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Act {
    AlignLeft,
    AlignHCenter,
    AlignRight,
    AlignTop,
    AlignVCenter,
    AlignBottom,
    DistributeH,
    DistributeV,
    Flatten,
    TransformReset,
    Rotate90,
    FlipH,
    FlipV,
    Export,
    Apply,
    Restore,
}

/// O que um controlo É.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Interruptor que acende quando ligado.
    Toggle(Flag),
    /// O olho: acende (e fecha) quando DESLIGADO.
    Eye(Flag),
    /// Um dos valores de uma escolha.
    Pick(Pick, u8),
    /// A caixa de uma lista suspensa (o despacho dos cartões abre-a).
    Box,
    Act(Act),
    /// O «+» de uma lista: liga a primeira linha livre.
    Add([Flag; 3]),
    /// O «−» de uma linha: desliga-a e larga a amostra `usize` (se o seletor estava nela).
    Remove(Flag, usize),
    /// Liga ou desliga uma secção inteira.
    Set(Flag, bool),
    /// A amostra `usize` de [`SWATCHES`]: abre o seletor de cor.
    Swatch(usize),
    /// A opção do «+» de uma lista com tipos: liga a primeira linha livre, já com esse tipo.
    AddOf([Flag; 3], [Pick; 3], u8),
    /// A linha que abre o painel do lado: o 2.º clique fecha.
    Expand(Pick, u8),
    /// O ⠿ dos estilos: quem o abre é o despacho dos cartões (`panel::cards::estilos`).
    Styles,
}

/// Uma linha da tabela. `icon` = o de sempre; `lit_icon` = o do estado aceso, se é outro.
#[derive(Clone, Copy, Debug)]
pub struct Control {
    pub id: NodeId,
    pub kind: Kind,
    pub icon: Option<IconId>,
    pub lit_icon: Option<IconId>,
    /// A chave do texto do segmento ou da opção.
    pub text: Option<&'static str>,
    /// A chave da dica dos ícones sem texto.
    pub tip: Option<&'static str>,
}

/// `(cartão, amostra, token de fábrica)` de cada amostra de cor.
pub const SWATCHES: [(NodeId, NodeId, ColorToken); crate::state::N_SWATCHES] = [
    (g::CARD_FILL, g::FILLS[0].swatch, ColorToken::Accent),
    (g::CARD_FILL, g::FILLS[1].swatch, ColorToken::Accent),
    (g::CARD_FILL, g::FILLS[2].swatch, ColorToken::Accent),
    (g::CARD_STROKE, g::STROKES[0].swatch, ColorToken::Text1),
    (g::CARD_STROKE, g::STROKES[1].swatch, ColorToken::Text1),
    (g::CARD_STROKE, g::STROKES[2].swatch, ColorToken::Text1),
    (g::CARD_EFFECTS, g::EFFECTS[0].swatch, ColorToken::Text3),
    (g::CARD_EFFECTS, g::EFFECTS[1].swatch, ColorToken::Text3),
    (g::CARD_EFFECTS, g::EFFECTS[2].swatch, ColorToken::Text3),
];

pub(super) const fn icon(id: NodeId, kind: Kind, icon: IconId, tip: &'static str) -> Control {
    Control {
        id,
        kind,
        icon: Some(icon),
        lit_icon: None,
        text: None,
        tip: Some(tip),
    }
}

pub(super) const fn act(id: NodeId, a: Act, ic: IconId, tip: &'static str) -> Control {
    icon(id, Kind::Act(a), ic, tip)
}

pub(super) const fn seg(id: NodeId, p: Pick, i: u8, ic: IconId, tip: &'static str) -> Control {
    icon(id, Kind::Pick(p, i), ic, tip)
}

pub(super) const fn eye(id: NodeId, f: Flag, tip: &'static str) -> Control {
    Control {
        lit_icon: Some(IconId::EyeClosed),
        ..icon(id, Kind::Eye(f), IconId::Eye, tip)
    }
}

pub(super) const fn minus(id: NodeId, f: Flag, swatch: usize, tip: &'static str) -> Control {
    icon(id, Kind::Remove(f, swatch), IconId::Minus, tip)
}

pub(super) const fn plus(id: NodeId, rows: [Flag; 3], tip: &'static str) -> Control {
    icon(id, Kind::Add(rows), IconId::Add, tip)
}

pub(super) const fn opt(id: NodeId, p: Pick, i: u8, text: &'static str) -> Control {
    Control {
        id,
        kind: Kind::Pick(p, i),
        icon: None,
        lit_icon: None,
        text: Some(text),
        tip: None,
    }
}

/// A opção `v` de um «+» com tipos.
pub(super) const fn add_of(
    id: NodeId,
    rows: [Flag; 3],
    picks: [Pick; 3],
    v: u8,
    text: &'static str,
) -> Control {
    Control {
        id,
        kind: Kind::AddOf(rows, picks, v),
        icon: None,
        lit_icon: None,
        text: Some(text),
        tip: None,
    }
}

/// Um controlo sem ícone nem texto próprios: a caixa, o botão, a caixa de marcar, a amostra.
pub(super) const fn plain(id: NodeId, kind: Kind) -> Control {
    Control {
        id,
        kind,
        icon: None,
        lit_icon: None,
        text: None,
        tip: None,
    }
}

#[path = "controlos_forma.rs"]
mod forma;
#[path = "controlos_tinta.rs"]
mod tinta;

/// ⭐ Todos os controlos da galeria.
pub fn controls() -> impl Iterator<Item = &'static Control> {
    forma::FORMA.iter().chain(tinta::TINTA)
}

/// A linha de `id`, se é um controlo da galeria.
pub(crate) fn find(id: NodeId) -> Option<&'static Control> {
    controls().find(|c| c.id == id)
}

/// `id` está aceso? Os espelhos acendem pelo SINAL do Scale: o número é a verdade (o arrasto, o ↺
/// e o Cancel apagam-nos sem ninguém se lembrar deles).
#[must_use]
pub fn lit(c: &Control, ch: &Choices, store: &WidgetStore) -> bool {
    let negativo = |id| store.number_value(id).is_some_and(|v| v < 0.0);
    match c.kind {
        Kind::Toggle(f) => ch.on(f),
        Kind::Eye(f) => !ch.on(f),
        Kind::Pick(p, i) | Kind::Expand(p, i) => ch.pick(p) == i,
        Kind::Act(Act::FlipH) => negativo(g::SCALE_X),
        Kind::Act(Act::FlipV) => negativo(g::SCALE_Y),
        _ => false,
    }
}

fn row_of(id: NodeId) -> &'static Control {
    find(id).unwrap_or_else(|| panic!("{id:?} não está em CONTROLS"))
}

/// O segmento de `id`, pela tabela.
pub(crate) fn seg_of(id: NodeId, ch: &Choices, store: &WidgetStore) -> Seg<'static> {
    let c = row_of(id);
    let on = lit(c, ch, store);
    Seg {
        id,
        icon: if on { c.lit_icon.or(c.icon) } else { c.icon },
        text: c.text.map_or("", tr),
        on,
    }
}

/// Vários segmentos de uma vez.
pub(crate) fn segs<const N: usize>(
    ids: [NodeId; N],
    ch: &Choices,
    store: &WidgetStore,
) -> [Seg<'static>; N] {
    ids.map(|id| seg_of(id, ch, store))
}

/// A ação de cabeçalho de `id`, pela tabela.
pub(crate) fn head(id: NodeId, ch: &Choices, store: &WidgetStore) -> HeadAction {
    let s = seg_of(id, ch, store);
    HeadAction {
        id,
        icon: s.icon.unwrap_or(IconId::Add),
        on: s.on,
    }
}

/// A largura que uma caixa pede para mostrar `text` inteiro: a célula do ícone (a amostra, a seta)
/// + o texto + a folga. As linhas de várias colunas partem-se quando ela não cabe (à Figma: nada se
/// corta na coluna estreita, censo `nenhum_rotulo`).
pub(crate) fn caixa_para(c: &mut CardCtx<'_>, text: &str) -> f32 {
    m::ICON_CELL + c.text_system.prefix_width(text, m::VALUE_PX).ceil() + m::FIELD_PAD_R
}

/// A largura mínima de um campo numérico com rótulo e marca na linha dele.
pub(crate) const CAMPO_MIN: f32 = m::FIELD_VALUE_MIN_W + m::KEY_MARK_BOX;

/// A lista suspensa `id`, mostrando a opção acesa.
pub(crate) fn dropdown<'a>(id: NodeId, options: &'a [Seg<'a>]) -> Widget<'a> {
    Widget::Dropdown {
        id,
        text: options.iter().find(|s| s.on).map_or("", |s| s.text),
        options,
    }
}

pub(crate) fn field(id: NodeId, glyph: Glyph<'_>) -> Widget<'_> {
    Widget::Field { id, glyph }
}

/// Um cartão presente: título (a chave), ações de cabeçalho e corpo.
pub(crate) fn card(
    section: NodeId,
    title: &'static str,
    actions: Vec<HeadAction>,
    body: CardBody<'static>,
) -> CardDef<'static> {
    CardDef {
        section,
        title,
        actions,
        add: None,
        summary: None,
        body: Some(body),
    }
}
