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
    /// Um ícone fixo numa caixa quadrada (o tipo do objeto) — não é clicável.
    Badge(IconId),
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
    /// Uma lista suspensa: a caixa com o valor e a seta; as opções abrem por baixo (por cima de tudo).
    Dropdown {
        id: NodeId,
        text: &'a str,
        options: &'a [Seg<'a>],
    },
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

#[path = "cards_cartao.rs"]
mod cartao;
#[path = "cards_colunas.rs"]
mod colunas;
#[path = "cards_corpo.rs"]
pub mod corpo;
#[path = "cards_entra_sai.rs"]
pub mod entra_sai;
#[path = "cards_estilos.rs"]
pub mod estilos;
#[path = "cards_linhas.rs"]
mod linhas;
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
use cartao::{flight_shadow, paint_card, paint_card_with, shadow};
pub use lista::paint_card_list_pinned;
pub use menu::{apply_list_menu_click, list_menu_of};

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
    dropdown::ao_premir(hit_index, event.x, event.y);
    estilos::ao_premir(store, hit_index, event.x, event.y);
}

/// ⭐ **A lista suspensa de um cartão** — qual está aberta (um clique na caixa abre/fecha; qualquer
/// outro clique fecha) e as opções do quadro, pintadas por CIMA de tudo no fim da lista. Cada opção
/// leva o id do painel dono: o clique nela é o `Click` que o `apply_event` dele já trata.
pub mod dropdown {
    use super::*;
    use std::cell::RefCell;

    type Opcoes = (Rect, CardInk, Vec<(NodeId, String, bool)>);

    thread_local! {
        static ABERTA: RefCell<Option<NodeId>> = const { RefCell::new(None) };
        static PENDENTE: RefCell<Option<Opcoes>> = const { RefCell::new(None) };
        static CAIXAS: RefCell<Vec<NodeId>> = const { RefCell::new(Vec::new()) };
        /// As opções pintadas no último quadro: a pressão nelas não é «fora».
        static OPCOES: RefCell<Vec<NodeId>> = const { RefCell::new(Vec::new()) };
    }

    /// A pressão fora da caixa e das opções fecha a lista (a caixa decide no clique dela).
    pub(super) fn ao_premir(hit_index: &HitIndex, x: f32, y: f32) {
        let dela = hit_index.hit_with_rect(x, y).is_some_and(|(id, _)| {
            CAIXAS.with(|c| c.borrow().contains(&id)) || OPCOES.with(|o| o.borrow().contains(&id))
        });
        if !dela {
            ABERTA.with(|a| *a.borrow_mut() = None);
        }
    }

    #[must_use]
    pub fn open() -> Option<NodeId> {
        ABERTA.with(|a| *a.borrow())
    }

    pub fn stash(r: Rect, ink: CardInk, options: Vec<(NodeId, String, bool)>) {
        PENDENTE.with(|p| *p.borrow_mut() = Some((r, ink, options)));
    }

    /// Declara `id` como caixa de lista suspensa (o painel, ao montar os cartões).
    pub fn declare(id: NodeId) {
        CAIXAS.with(|c| {
            let mut c = c.borrow_mut();
            if !c.contains(&id) {
                c.push(id);
            }
        });
    }

    /// O clique: na caixa abre/fecha (consome); noutro sítio fecha e deixa o clique seguir.
    pub fn on_click(id: NodeId) -> bool {
        if CAIXAS.with(|c| c.borrow().contains(&id)) {
            ABERTA.with(|a| {
                let mut a = a.borrow_mut();
                *a = if *a == Some(id) { None } else { Some(id) };
            });
            return true;
        }
        ABERTA.with(|a| *a.borrow_mut() = None);
        false
    }

    /// Pinta as opções da lista aberta por baixo da caixa, por cima de tudo.
    pub fn paint_pending(
        scene: &mut VectorScene,
        text_system: &mut TextSystem,
        hit_index: &mut HitIndex,
        store: &WidgetStore,
    ) {
        let Some((r, ink, opts)) = PENDENTE.with(|p| p.borrow_mut().take()) else {
            return;
        };
        OPCOES.with(|o| *o.borrow_mut() = opts.iter().map(|(id, ..)| *id).collect());
        let h = m::FIELD_H * opts.len() as f32 + m::SEG_PAD * 2.0;
        let caixa = Rect::new(r.x, r.y + r.h + m::SEG_PAD, r.w, h);
        shadow(
            scene,
            caixa,
            m::FIELD_RADIUS,
            ink.color(ColorToken::KanbanListShadow),
            m::LIST_SHADOW_FAR,
        );
        fill_rounded_rect(
            scene,
            caixa,
            m::FIELD_RADIUS,
            ink.color(ColorToken::KanbanCardRaised),
        );
        for (i, (id, text, on)) in opts.iter().enumerate() {
            let lr = Rect::new(
                caixa.x + m::SEG_PAD,
                caixa.y + m::SEG_PAD + m::FIELD_H * i as f32,
                caixa.w - m::SEG_PAD * 2.0,
                m::FIELD_H,
            );
            hit_index.register(*id, lr);
            if *on || store.hover_live(*id) > 0.0 {
                fill_rounded_rect(
                    scene,
                    lr,
                    m::SEG_RADIUS,
                    ink.color(ColorToken::CardFieldHover),
                );
            }
            crate::paint::paint_text_elided(
                text_system,
                scene,
                text,
                lr.x + m::FIELD_PAD_R,
                lr.y + (lr.h - m::VALUE_PX) * 0.5,
                m::VALUE_PX,
                lr.w - m::FIELD_PAD_R * 2.0,
                ink.color(ColorToken::Text1),
            );
        }
    }
}
