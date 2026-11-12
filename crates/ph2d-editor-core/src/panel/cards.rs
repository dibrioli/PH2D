//! ⭐⭐ **O painel como LISTA DE CARTÕES** (`docs/CartoesUI`, D3–D6, D16): a lista flutua sobre o
//! canvas (sem fundo de dock) e abraça o conteúdo; cada secção é um cartão do Kanban que se
//! arrasta pelo cabeçalho (o `ph2d_card_stack::Stack`: vão, voo inclinado, assentar com mola);
//! dentro do cartão o layout é o do Figma (rótulo com unidade acima, campo ícone + valor, marca de
//! animação na linha do rótulo, ações = ícones no cabeçalho).
//!
//! Tema, contorno e notas por secção (D6; o solto tem os da sua face, T63) pelo menu da pega — o
//! cabeçalho É a pega. A geometria vive em `ph2d_card_stack::metrics`. ⚠️ **As alturas
//! são as do quadro ANTERIOR**: cada cartão mede-se ao pintar e a mola leva a diferença ao seguinte.

use crate::icons::IconId;
use crate::interaction::{HitIndex, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_icon, resolve};
use crate::panel::PaintCtx;
use crate::widget::card_field::{self, Glyph};
use crate::widget::card_ink::CardInk;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_card_stack::{Item, reorder};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, StrokeToken, Theme};
use ph2d_vector::{Affine, VectorScene};

/// Um ícone de ação no cabeçalho do cartão (T2).
#[derive(Copy, Clone, Debug)]
pub struct HeadAction {
    pub id: NodeId,
    pub icon: IconId,
    /// Aceso (o 👁 de um limite visível, o alfinete preso).
    pub on: bool,
}

/// O corpo de um cartão: pinta a partir de `y` e devolve o fundo.
pub type CardBody<'a> = Box<dyn Fn(&mut CardCtx<'_>, f32) -> f32 + 'a>;

/// Um cartão (uma secção).
pub struct CardDef<'a> {
    pub section: NodeId,
    pub title: &'a str,
    pub actions: Vec<HeadAction>,
    /// Secção AUSENTE: título esmaecido e «+» (este id) que a cria (T3).
    pub add: Option<NodeId>,
    /// O resumo da secção dobrada, à direita do título (T4).
    pub summary: Option<String>,
    pub body: Option<CardBody<'a>>,
}

/// Um segmento de uma escolha: `(id, ícone, texto, aceso)`.
#[derive(Copy, Clone, Debug)]
pub struct Seg<'a> {
    pub id: NodeId,
    pub icon: Option<IconId>,
    pub text: &'a str,
    pub on: bool,
}

/// O que ocupa uma coluna de uma linha.
#[derive(Copy, Clone, Debug)]
pub enum Widget<'a> {
    /// O campo Figma (o painel marca-o com `WidgetStore::mark_card_field` ANTES de pintar).
    Field { id: NodeId, glyph: Glyph<'a> },
    /// Uma escolha segmentada que estica.
    Choice(&'a [Seg<'a>]),
    /// Uma escolha de ÍCONES de largura fixa (um quadrado por segmento).
    Icons(&'a [Seg<'a>]),
    /// Um botão; `primary` = o de destaque (azul).
    Button {
        id: NodeId,
        text: &'a str,
        primary: bool,
    },
    /// A cara de um campo que um clique ABRE (a opacidade de uma camada fechada): glifo e valor.
    FieldFace {
        id: NodeId,
        glyph: Glyph<'a>,
        text: &'a str,
    },
    /// Um campo de TEXTO (o nome do objeto): o `TextInput` de hoje dentro da caixa do cartão.
    Text { id: NodeId, placeholder: &'a str },
    /// Um campo de BUSCA: o `Text` com o «×» que o esvazia dentro da caixa, enquanto há texto ([`busca`]).
    Search { id: NodeId, placeholder: &'a str },
    /// Um ícone fixo numa caixa quadrada (o tipo do objeto) — não é clicável.
    Badge(IconId),
    /// Um botão APAGADO: o gesto não é possível agora, e sem clique (sem rect não há `Click`).
    ButtonOff { text: &'a str },
    /// Um sinal numa cor: o ícone sem caixa e sem clique (a marca de contraste de um token), com a
    /// largura de um ícone. O que ele avisa está escrito noutro sítio do cartão.
    Mark { icon: IconId, tint: ColorToken },
    /// Um FACTO só de leitura (a escala do projeto, quantos corpos há): o valor sem caixa, porque a
    /// caixa promete edição. Não regista clique.
    Readout { text: &'a str },
    /// Ícones soltos, sem caixa (👁 · 🔒, girar 90° · espelhar, a corrente) — um quadrado cada.
    Bare(&'a [Seg<'a>]),
    /// A amostra de cor + o código (Fill/Stroke, 02 §2.7): o clique na amostra abre o seletor; com
    /// `edit`, o código é uma caixa de texto (esse id) — digitar o hex é escolher a cor.
    Swatch {
        id: NodeId,
        rgba: Option<[u8; 4]>,
        text: &'a str,
        edit: Option<NodeId>,
        /// D30: a camada ligada a um estilo mostra o NOME dele e a corrente partida (este id) solta-a.
        detach: Option<NodeId>,
    },
    /// Uma caixa de marcar como célula (05 §4): a 2.ª coluna de um campo. `off` = parada.
    Check {
        id: NodeId,
        label: crate::widget::key_mark::Rotulo<'a>,
        value: crate::widget::CheckboxValue,
        off: bool,
    },
    /// Uma lista suspensa: a caixa com o valor e a seta; as opções abrem por baixo (por cima de tudo).
    Dropdown {
        id: NodeId,
        text: &'a str,
        options: &'a [Seg<'a>],
    },
}

impl Widget<'_> {
    /// O alvo do controlo (o primeiro segmento de uma escolha); `None` = só pinta.
    #[must_use]
    pub fn target(&self) -> Option<NodeId> {
        match *self {
            Widget::Field { id, .. }
            | Widget::Button { id, .. }
            | Widget::FieldFace { id, .. }
            | Widget::Text { id, .. }
            | Widget::Search { id, .. }
            | Widget::Swatch { id, .. }
            | Widget::Check { id, .. }
            | Widget::Dropdown { id, .. } => Some(id),
            Widget::Choice(segs) | Widget::Icons(segs) | Widget::Bare(segs) => {
                segs.first().map(|s| s.id)
            }
            _ => None,
        }
    }
}

/// Uma coluna: rótulo opcional (com a marca de animação dos `key` quando há), e o widget.
#[derive(Copy, Clone, Debug)]
pub struct Cell<'a> {
    pub label: Option<&'a str>,
    pub key: Option<&'a [NodeId]>,
    pub widget: Widget<'a>,
}

impl<'a> Cell<'a> {
    #[must_use]
    pub fn new(widget: Widget<'a>) -> Self {
        Self {
            label: None,
            key: None,
            widget,
        }
    }
    #[must_use]
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }
    /// Uma propriedade animável: a marca sobre a borda direita da caixa (D17).
    #[must_use]
    pub fn key(mut self, fields: &'a [NodeId]) -> Self {
        self.key = Some(fields);
        self
    }
}

/// O contexto de pintura de DENTRO de um cartão.
pub struct CardCtx<'s> {
    pub scene: &'s mut VectorScene,
    pub text_system: &'s mut TextSystem,
    pub store: &'s WidgetStore,
    pub hit_index: &'s mut HitIndex,
    /// A tinta do cartão (o tema dele, ou o Kanban da lista).
    pub ink: CardInk,
    pub x: f32,
    pub w: f32,
    first: bool,
}

/// ⭐ A moldura e os controlos em fila de um painel-banda (Timeline).
#[path = "cards_banda.rs"]
pub mod banda;
/// ⭐ A borda de fora de uma janela acessória — a alça que a alarga.
#[path = "cards_borda.rs"]
pub mod borda;
#[path = "cards_busca.rs"]
pub mod busca;

#[path = "cards_cartao.rs"]
mod cartao;
/// A letra que distingue dois campos vizinhos.
#[path = "cards_letras.rs"]
pub mod letras;
pub use crate::widget::card_censo as censo;
#[path = "cards_colunas.rs"]
mod colunas;
#[path = "cards_compacta.rs"]
pub mod compacta;
#[path = "cards_corpo.rs"]
pub mod corpo;
/// ⭐ A lista suspensa de um cartão: abre, cabe na janela, rola; ver o módulo.
#[path = "cards_dropdown.rs"]
pub mod dropdown;
#[path = "cards_entra_sai.rs"]
pub mod entra_sai;
#[path = "cards_estilos.rs"]
pub mod estilos;
#[path = "cards_ferramenta.rs"]
pub mod ferramenta;
#[path = "cards_linhas.rs"]
mod linhas;
#[path = "cards_linhas_extra.rs"]
pub mod linhas_extra;
#[path = "cards_lista.rs"]
mod lista;
#[path = "cards_menu.rs"]
mod menu;
#[path = "cards_rolagem.rs"]
mod rolagem;
#[path = "cards_rotulos.rs"]
mod rotulos;
#[path = "cards_solto.rs"]
mod solto;
#[path = "cards_solto_some.rs"]
pub mod solto_some;
/// ⭐ O cartão User Data (cor, texto e a tabela de propriedades) de um dono do Pixel.
#[path = "cards_user_data.rs"]
pub mod user_data;
/// ⭐ Os botões baixos de verbo, lado a lado em dois degraus.
#[path = "cards_verbos.rs"]
pub mod verbos;
use cartao::{flight_shadow, paint_card, paint_card_with, shadow};
pub use lista::{ListTop, paint_card_list_pinned, paint_card_list_top, scroll_at_edge};
pub use menu::{apply_list_menu_click, list_menu_of};
pub use solto_some::pinta_as_cordas;

/// O id da pega (o cabeçalho) de um cartão — o mesmo das secções de hoje.
#[must_use]
pub fn grip_of(section: NodeId) -> NodeId {
    crate::ids::grip_de(section)
}

/// ⭐ C5: a linha-fantasma (traz o solto à vista) e «Return to panel» — ANTES dos painéis, que
/// consomem todo `Click` (o Bones manda-os à ferramenta). Devolve se consumiu.
pub fn apply_loose_click(store: &mut WidgetStore, id: NodeId) -> bool {
    solto::clique(store, id)
}

/// ⭐⭐ **Pinta o painel como lista de cartões** no `ctx.slot`. Devolve o rect da lista.
pub fn paint_card_list(
    ctx: &mut PaintCtx,
    panel: NodeId,
    title: &str,
    cards: &[CardDef<'_>],
) -> Rect {
    paint_card_list_with(ctx, panel, title, cards, None)
}

/// O pé da lista: `(id, texto)` — o «+ Add component» do Inspector (T5).
#[derive(Copy, Clone, Debug)]
pub struct Footer<'a> {
    pub id: NodeId,
    pub text: &'a str,
    /// D25: repetir o «+» no cabeçalho da lista (o Inspector só em modo Object, D25b).
    pub at_top: bool,
}

/// [`paint_card_list`] com o pé da lista.
pub fn paint_card_list_with(
    ctx: &mut PaintCtx,
    panel: NodeId,
    title: &str,
    cards: &[CardDef<'_>],
    footer: Option<Footer<'_>>,
) -> Rect {
    paint_card_list_pinned(ctx, panel, title, cards, footer, &[])
}

/// ⭐ **As dicas dos cartões**: todo ícone sem texto pintado num cartão regista-se aqui (o censo
/// `every_icon_in_a_card_has_a_tooltip` exige-lhe uma dica), e o campo numérico ganha a dica do
/// arrasto com o rótulo dele (maquete B: «Length — drag the icon · Shift ×10 · Alt fine»).
#[path = "cards_dicas.rs"]
pub mod dicas;

/// O id da barra de rolagem da lista de `panel`.
#[must_use]
pub fn scroll_bar_of(panel: NodeId) -> NodeId {
    const SAL_DA_BARRA: u64 = 0x5B1D_7E09_C3A4_6F28;
    NodeId(panel.0 ^ SAL_DA_BARRA)
}

/// O que o cartão SOLTO pinta a mais (C5/T7): ações no fim do cabeçalho (alfinete, «Return to
/// panel»), o subtítulo «de quem» e o esmaecido do «Not available».
#[derive(Default)]
struct Extra<'a> {
    acoes: Vec<HeadAction>,
    sub: Option<&'a str>,
    /// T42: o sujeito preso não é a seleção — o nome em destaque, com esta opacidade (o piscar).
    destaque: Option<f32>,
    /// T57: o corpo do solto rola por dentro, até ao fundo do canvas.
    rola: Option<solto::rola::Rola>,
}

/// ⭐ Premir fora fecha o que um cartão abriu (a lista suspensa, os estilos de cor). O botão do meio
/// é a mão que move a vista: não fecha.
pub fn ao_premir(store: &mut WidgetStore, hit_index: &HitIndex, event: ph2d_host::PointerEvent) {
    if event.kind != ph2d_host::PointerKind::Down
        || event.button == ph2d_host::PointerButton::Middle
    {
        return;
    }
    dropdown::ao_premir(store, hit_index, event.x, event.y);
    estilos::ao_premir(store, hit_index, event.x, event.y);
}

thread_local! {
    /// O topo e o fundo do canvas visível: as janelinhas dos cartões não passam deles.
    static LIMITE_VERTICAL: std::cell::Cell<(f32, f32)> =
        const { std::cell::Cell::new((f32::MIN, f32::MAX)) };
}

fn limita_janelinhas(canvas: Rect) {
    LIMITE_VERTICAL.with(|l| l.set((canvas.y, canvas.y + canvas.h)));
}

/// A altura que uma janelinha tem para caber: a do canvas visível.
fn altura_das_janelinhas() -> f32 {
    let (topo, fundo) = LIMITE_VERTICAL.with(std::cell::Cell::get);
    fundo - topo
}

/// O `y` de uma janelinha de altura `h` que quer nascer em `y`: sobe o que faltar para caber.
fn sobe_para_caber(y: f32, h: f32) -> f32 {
    let (topo, fundo) = LIMITE_VERTICAL.with(std::cell::Cell::get);
    if y + h <= fundo {
        y
    } else {
        (fundo - h).max(topo)
    }
}
