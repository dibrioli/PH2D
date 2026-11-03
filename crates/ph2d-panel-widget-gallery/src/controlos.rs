//! ⭐ **A TABELA dos controlos** (doc `CartoesUI/08` §1.1, «zero no-op silencioso»): cada botão, ícone,
//! segmento e opção da galeria é UMA linha daqui — o `populate` regista por ela, o `prepare` dá a
//! dica por ela, a pintura tira dela o ícone e o aceso, e o evento age por ela.

use crate::ids as g;
use crate::state::{Choices, Flag, Pick};
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::IconId;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::paint::fill_rounded_rect;
use ph2d_editor_core::panel::TextKey;
use ph2d_editor_core::panel::cards::{CardBody, CardCtx, CardDef, HeadAction, Seg, Widget, corpo};
use ph2d_editor_core::widget::card_field::Glyph;
use ph2d_editor_core::widget::key_mark::Rotulo;
use ph2d_editor_core::zones::Rect;
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
    /// O conta-gotas de um alvo: arma; o 2.º clique faz de clique no canvas (a galeria não tem canvas)
    /// e escreve o nome no campo de texto.
    PickTarget(Flag, NodeId),
    PixelSwap,
    /// Dispara a entrada Trigger `usize`.
    Fire(usize),
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
    /// A cor `u8` da paleta: vai para o alvo (Foreground ou Background).
    Palette(u8),
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
    (g::CARD_LIGHT, g::LT_SWATCH, ColorToken::Info),
    (g::CARD_GRADIENT, g::STOPS[0].swatch, ColorToken::Accent),
    (g::CARD_GRADIENT, g::STOPS[1].swatch, ColorToken::Danger),
    (g::CARD_GRADIENT, g::STOPS[2].swatch, ColorToken::Success),
];

/// A paleta indexada do cartão Pixel Art, em tokens (nunca hex).
pub const PALETTE_TOKENS: [ColorToken; 16] = [
    ColorToken::Text1,
    ColorToken::Text2,
    ColorToken::Text3,
    ColorToken::Accent,
    ColorToken::AccentPress,
    ColorToken::AccentSoft,
    ColorToken::Info,
    ColorToken::InfoSoft,
    ColorToken::Success,
    ColorToken::SuccessSoft,
    ColorToken::Danger,
    ColorToken::DangerSoft,
    ColorToken::Bg3,
    ColorToken::Bg2,
    ColorToken::Bg1,
    ColorToken::Bg0,
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
#[path = "controlos_reais.rs"]
mod reais;
#[path = "controlos_rive.rs"]
pub(crate) mod rive;
#[path = "controlos_tinta.rs"]
mod tinta;

/// ⭐ Todos os controlos da galeria.
pub fn controls() -> impl Iterator<Item = &'static Control> {
    forma::FORMA
        .iter()
        .chain(tinta::TINTA)
        .chain(reais::REAIS)
        .chain(reais::linhas())
        .chain(rive::RIVE)
        .chain(rive::linhas())
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
        Kind::Act(Act::PickTarget(f, _)) => ch.on(f),
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

/// O glifo de um campo sem ícone próprio: a LETRA da grandeza (T56). Ícone só onde ele quer dizer
/// aquela grandeza: ângulo, opacidade, traço, raio, mistura, desfoque, corrente.
pub(crate) fn letra(rotulo: &str) -> Glyph<'_> {
    Glyph::Text(ph2d_editor_core::panel::cards::corpo::letra_da_grandeza(
        rotulo,
    ))
}

/// Um campo com rótulo, a marca de animação na linha do rótulo (`$id` constante) e, como glifo, a
/// letra da grandeza ou o ícone que a quer dizer.
macro_rules! num {
    ($id:expr, $label:expr) => {{
        let rotulo = $label;
        Cell::new($crate::controlos::field(
            $id,
            $crate::controlos::letra(rotulo),
        ))
        .label(rotulo)
        .key(&[$id])
    }};
    ($id:expr, $icon:expr, $label:expr) => {
        Cell::new($crate::controlos::field(
            $id,
            ph2d_editor_core::widget::card_field::Glyph::Icon($icon),
        ))
        .label($label)
        .key(&[$id])
    };
}
pub(crate) use num;

/// Uma frase curta na tinta do rótulo (o que o estado de um cartão quer dizer). Devolve o fundo.
pub(crate) fn dica(c: &mut CardCtx<'_>, text: &str, y: f32) -> f32 {
    let y = y + m::LABEL_TOP;
    ph2d_editor_core::paint::paint_text_elided(
        c.text_system,
        c.scene,
        text,
        c.x,
        y,
        m::LABEL_PX,
        c.w,
        c.ink.color(ColorToken::CardLabel),
    );
    y + m::LABEL_PX
}

/// Uma caixa de marcar: o nome à direita e a marca de animação no fim da linha.
pub(crate) fn marca(c: &mut CardCtx<'_>, id: NodeId, key: &'static str, on: bool, y: f32) -> f32 {
    corpo::check(
        c.scene,
        c.text_system,
        c.store,
        c.hit_index,
        c.ink,
        c.x,
        c.w,
        y,
        corpo::Marca {
            id,
            label: Rotulo::from(TextKey::new(key)),
            on,
        },
    )
}

/// Uma caixa de marcar de estado da VISTA: a coluna da marca fica vazia (um ponto diria «isto anima»).
pub(crate) fn marca_vista(
    c: &mut CardCtx<'_>,
    id: NodeId,
    key: &'static str,
    on: bool,
    y: f32,
) -> f32 {
    corpo::check(
        c.scene,
        c.text_system,
        c.store,
        c.hit_index,
        c.ink,
        c.x,
        c.w,
        y,
        corpo::Marca {
            id,
            label: Rotulo::vista(TextKey::new(key)),
            on,
        },
    )
}

/// Uma barra fina com o trecho `de..ate` (`0..=1`) aceso (Health, Trim path). Devolve o fundo.
pub(crate) fn barra(c: &mut CardCtx<'_>, de: f32, ate: f32, y: f32) -> f32 {
    let y = y + m::FIELD_ROW_GAP;
    let h = m::FIELD_GAP;
    let trilha = Rect::new(c.x, y, c.w, h);
    fill_rounded_rect(c.scene, trilha, h * 0.5, c.ink.color(ColorToken::CardField));
    let (a, b) = (de.clamp(0.0, 1.0), ate.clamp(0.0, 1.0)); // CLAMP-OK: fração de um dado fictício
    if b > a {
        let cheio = Rect::new(c.x + c.w * a, y, c.w * (b - a), h);
        fill_rounded_rect(c.scene, cheio, h * 0.5, c.ink.color(ColorToken::Accent));
    }
    y + h
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
