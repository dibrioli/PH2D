//! ⭐ **Os estilos de cor no cartão** (D28/D30, maquete `docs/CartoesUI/maquete/estilos.png`): o ⠿
//! no cabeçalho do Fill/Stroke abre a lista «Color styles» (E); clicar aplica e LIGA a camada (F);
//! «+» guarda a cor da camada; o botão direito numa linha abre Apply · Edit style… · Rename ·
//! Duplicate · Delete (G). Quem guarda os estilos é a cena do Vetor: daqui saem PEDIDOS
//! ([`take_pedidos`]) e entra a VISTA ([`publica`]), as duas pelo mesmo thread do quadro.

use super::*;
use crate::interaction::{ContextMenuKind, InteractiveState, WidgetEvent};
use std::cell::{Cell, RefCell};

/// Um estilo como a lista o mostra.
#[derive(Clone, Debug, PartialEq)]
pub struct Estilo {
    pub id: u32,
    pub name: String,
    pub rgba: [u8; 4],
    /// Camadas vivas que o usam.
    pub usos: usize,
}

/// O que a lista pede à cena (a `camada` é a linha nas formas escolhidas: o índice da pilha, ou uma
/// tinta de base, `ph2d_vec_scene::LINHA_BASE_FILL`/`LINHA_BASE_STROKE`).
#[derive(Clone, Debug, PartialEq)]
pub enum Pedido {
    Aplicar { estilo: u32, camada: usize },
    Novo { camada: usize, nome: String },
    Renomear { estilo: u32, nome: String },
    Duplicar { estilo: u32, nome: String },
    Apagar { estilo: u32 },
    Cor { estilo: u32, rgba: [u8; 4] },
    Soltar { camada: usize },
}

thread_local! {
    static ESTILOS: RefCell<Vec<Estilo>> = const { RefCell::new(Vec::new()) };
    /// O estilo de cada linha da forma mostrada (camadas, depois as duas bases).
    static LIGADOS: RefCell<Vec<Option<u32>>> = const { RefCell::new(Vec::new()) };
    static PEDIDOS: RefCell<Vec<Pedido>> = const { RefCell::new(Vec::new()) };
    /// Os ⠿ declarados neste quadro: botão → (cartão, camada).
    static BOTOES: RefCell<Vec<(NodeId, NodeId, usize)>> = const { RefCell::new(Vec::new()) };
    /// A lista aberta: o botão que a abriu.
    static ABERTA: Cell<Option<NodeId>> = const { Cell::new(None) };
    /// O estilo no «Edit style»; `Some(0)` = o próximo que nascer (o «+»).
    static EDITA: Cell<Option<u32>> = const { Cell::new(None) };
    static MAIOR: Cell<u32> = const { Cell::new(0) };
}

use ph2d_tool_registry::hash_node_id;

const SAL_BOTAO: u64 = hash_node_id("card_styles_button").0;
const SAL_LINHA: u64 = hash_node_id("card_styles_row").0;
const SAL_SOLTA: u64 = hash_node_id("card_styles_detach").0;

/// O «+» da lista.
pub const MAIS: NodeId = hash_node_id("card_styles_add");
/// A caixa do nome no «Edit style».
pub const NOME: NodeId = hash_node_id("card_styles_name");
/// A caixa do código da cor no «Edit style».
pub const HEX: NodeId = hash_node_id("card_styles_hex");
/// A caixa «Search...» no topo da lista: filtra pelo nome, sem caixa alta/baixa.
pub const BUSCA: NodeId = hash_node_id("card_styles_search");
/// O fundo da lista e do editor: o clique nele não atravessa nem fecha.
const FUNDO: NodeId = hash_node_id("card_styles_backdrop");

/// O ⠿ do cartão `card`.
#[must_use]
pub const fn botao_de(card: NodeId) -> NodeId {
    NodeId(card.0 ^ SAL_BOTAO)
}

/// A linha do estilo `id` na lista.
#[must_use]
pub fn linha_de(id: u32) -> NodeId {
    NodeId(SAL_LINHA ^ u64::from(id).rotate_left(17))
}

/// O «soltar» (corrente partida) da camada `camada`.
#[must_use]
pub fn solta_de(camada: usize) -> NodeId {
    NodeId(SAL_SOLTA ^ (camada as u64).rotate_left(23))
}

/// A cena publica os estilos e o estilo de cada camada da forma mostrada (todo quadro).
pub fn publica(estilos: Vec<Estilo>, ligados: Vec<Option<u32>>) {
    ESTILOS.with(|e| *e.borrow_mut() = estilos);
    LIGADOS.with(|l| *l.borrow_mut() = ligados);
}

#[must_use]
pub fn estilos() -> Vec<Estilo> {
    ESTILOS.with(|e| e.borrow().clone())
}

/// O estilo a que a camada `camada` da forma mostrada está ligada.
#[must_use]
pub fn ligado(camada: usize) -> Option<Estilo> {
    let id = LIGADOS.with(|l| l.borrow().get(camada).copied().flatten())?;
    ESTILOS.with(|e| e.borrow().iter().find(|x| x.id == id).cloned())
}

pub fn pede(p: Pedido) {
    PEDIDOS.with(|q| q.borrow_mut().push(p));
}

#[must_use]
pub fn take_pedidos() -> Vec<Pedido> {
    PEDIDOS.with(|q| std::mem::take(&mut *q.borrow_mut()))
}

/// O cartão declara o ⠿ dele (ao montar-se) e a camada a que a lista se aplica.
pub fn declare(card: NodeId, camada: usize) -> NodeId {
    let b = botao_de(card);
    BOTOES.with(|v| {
        let mut v = v.borrow_mut();
        v.retain(|(x, ..)| *x != b);
        v.push((b, card, camada));
    });
    b
}

/// A lista está aberta pelo ⠿ de `card`?
#[must_use]
pub fn aberta_em(card: NodeId) -> bool {
    ABERTA.with(Cell::get) == Some(botao_de(card))
}

/// A cena de smoke (`PH2D_VEC_CARTOES_SMOKE=4`) abre a lista do primeiro ⠿ declarado (o do Fill).
pub fn abre_o_primeiro() {
    if let Some(b) = BOTOES.with(|v| v.borrow().first().map(|(b, ..)| *b)) {
        ABERTA.with(|a| a.set(Some(b)));
    }
}

/// A cena `PH2D_VEC_CARTOES_SMOKE=5` (maquete G): a lista aberta e o «Edit style» do primeiro estilo.
pub fn abre_o_editor_do_primeiro() {
    abre_o_primeiro();
    if let Some(id) = ESTILOS.with(|e| e.borrow().first().map(|x| x.id)) {
        EDITA.with(|e| e.set(Some(id)));
    }
}

/// Os estilos que a busca deixa ver (a busca vazia mostra todos).
#[must_use]
pub fn filtrados(busca: &str) -> Vec<Estilo> {
    let q = busca.trim().to_lowercase();
    let mut v = estilos();
    v.retain(|e| e.name.to_lowercase().contains(&q));
    v
}

fn camada_aberta() -> Option<(NodeId, usize)> {
    let b = ABERTA.with(Cell::get)?;
    BOTOES.with(|v| {
        v.borrow()
            .iter()
            .find(|(x, ..)| *x == b)
            .map(|(_, c, i)| (*c, *i))
    })
}

/// O estilo cuja LINHA é `id` (o botão direito pergunta-o).
#[must_use]
pub fn estilo_da_linha(id: NodeId) -> Option<u32> {
    ABERTA.with(Cell::get)?;
    ESTILOS.with(|e| e.borrow().iter().map(|x| x.id).find(|x| linha_de(*x) == id))
}

fn fecha() {
    ABERTA.with(|a| a.set(None));
    EDITA.with(|e| e.set(None));
}

fn nome_de(id: u32) -> Option<String> {
    ESTILOS.with(|e| {
        e.borrow()
            .iter()
            .find(|x| x.id == id)
            .map(|x| x.name.clone())
    })
}

fn texto(store: &WidgetStore, id: NodeId) -> Option<String> {
    match store.get(id) {
        Some(InteractiveState::TextInput { text, .. }) => Some(text.clone()),
        _ => None,
    }
}

/// ⭐ Os cliques, o `Submit` das duas caixas e as linhas do menu — ANTES dos painéis (o Inspector
/// consome todo `Click`). Devolve se consumiu.
pub fn on_event(store: &mut WidgetStore, event: WidgetEvent) -> bool {
    match event {
        WidgetEvent::Click(id) => clique(store, id),
        WidgetEvent::Submit(id) if id == NOME => {
            if let (Some(e), Some(t)) = (EDITA.with(Cell::get), texto(store, NOME))
                && !t.trim().is_empty()
            {
                pede(Pedido::Renomear {
                    estilo: e,
                    nome: t.trim().to_owned(),
                });
            }
            store.set_focus(None);
            true
        }
        WidgetEvent::Submit(id) if id == BUSCA => {
            store.set_focus(None);
            true
        }
        WidgetEvent::Submit(id) if id == HEX => {
            if let (Some(e), Some(c)) = (
                EDITA.with(Cell::get),
                texto(store, HEX)
                    .and_then(|t| crate::widget::parse_hex(&t))
                    .map(|v| v.rgba),
            ) {
                pede(Pedido::Cor { estilo: e, rgba: c });
            }
            store.set_focus(None);
            true
        }
        _ => false,
    }
}

fn clique(store: &mut WidgetStore, id: NodeId) -> bool {
    if menu(store, id) {
        return true;
    }
    if BOTOES.with(|v| v.borrow().iter().any(|(b, ..)| *b == id)) {
        let ja = ABERTA.with(Cell::get) == Some(id);
        fecha();
        // A busca recomeça vazia sempre que a lista abre ou fecha.
        if store.focus_id() == Some(BUSCA) {
            store.set_focus(None);
        }
        if let Some(InteractiveState::TextInput { text, caret, .. }) = store.get_mut(BUSCA) {
            text.clear();
            *caret = 0;
        }
        if !ja {
            ABERTA.with(|a| a.set(Some(id)));
        }
        return true;
    }
    if id == FUNDO {
        return true;
    }
    if let Some(i) = (0..ph2d_vec_scene::LINHAS_DA_PILHA).find(|i| solta_de(*i) == id) {
        pede(Pedido::Soltar { camada: i });
        return true;
    }
    if id == NOME || id == HEX || id == BUSCA {
        return false;
    }
    let Some((_, camada)) = camada_aberta() else {
        if EDITA.with(Cell::get).is_some() {
            fecha();
        }
        return false;
    };
    if id == MAIS {
        let n = ESTILOS.with(|e| e.borrow().len()) + 1;
        MAIOR.with(|m| m.set(ESTILOS.with(|e| e.borrow().iter().map(|x| x.id).max().unwrap_or(0))));
        pede(Pedido::Novo {
            camada,
            nome: format!("{} {n}", ph2d_i18n::tr("card.styles.new_name")),
        });
        EDITA.with(|e| e.set(Some(0)));
        return true;
    }
    if let Some(e) = ESTILOS.with(|e| e.borrow().iter().map(|x| x.id).find(|x| linha_de(*x) == id))
    {
        pede(Pedido::Aplicar { estilo: e, camada });
        fecha();
        return true;
    }
    fecha();
    false
}

/// É da lista ou do «Edit style»? A pressão aí não é «fora».
fn da_lista(id: NodeId) -> bool {
    [FUNDO, MAIS, BUSCA, NOME, HEX].contains(&id)
        || ESTILOS.with(|e| e.borrow().iter().any(|x| linha_de(x.id) == id))
}

/// ⭐ A pressão FORA fecha a lista e o «Edit style»; o que estava escrito sem Enter grava-se. O ⠿ que
/// a abriu fica de fora (o clique dele é que a fecha) e um menu aberto é quem decide a pressão.
pub(super) fn ao_premir(store: &mut WidgetStore, hit_index: &HitIndex, x: f32, y: f32) {
    let aberta = ABERTA.with(Cell::get);
    if (aberta.is_none() && EDITA.with(Cell::get).is_none()) || store.context_menu().is_some() {
        return;
    }
    let alvo = hit_index.hit_with_rect(x, y).map(|(id, _)| id);
    if alvo.is_some_and(|id| Some(id) == aberta || da_lista(id)) {
        return;
    }
    if let Some(id) = store.focus_id().filter(|f| *f == NOME || *f == HEX) {
        on_event(store, WidgetEvent::Submit(id));
    }
    fecha();
}

/// As cinco linhas do menu do botão direito numa linha de estilo.
fn menu(store: &mut WidgetStore, id: NodeId) -> bool {
    use crate::ids;
    let todas = [
        ids::CTX_MENU_STYLE_APPLY,
        ids::CTX_MENU_STYLE_EDIT,
        ids::CTX_MENU_STYLE_RENAME,
        ids::CTX_MENU_STYLE_DUPLICATE,
        ids::CTX_MENU_STYLE_DELETE,
    ];
    if !todas.contains(&id) {
        return false;
    }
    let Some(req) = store.consume_last_context_menu() else {
        return true;
    };
    let ContextMenuKind::ColorStyleRow { style } = req.kind else {
        return true;
    };
    if id == ids::CTX_MENU_STYLE_APPLY {
        if let Some((_, camada)) = camada_aberta() {
            pede(Pedido::Aplicar {
                estilo: style,
                camada,
            });
        }
        fecha();
    } else if id == ids::CTX_MENU_STYLE_EDIT || id == ids::CTX_MENU_STYLE_RENAME {
        EDITA.with(|e| e.set(Some(style)));
        if id == ids::CTX_MENU_STYLE_RENAME {
            store.set_focus(Some(NOME));
        }
    } else if id == ids::CTX_MENU_STYLE_DUPLICATE {
        let nome = nome_de(style).unwrap_or_default();
        pede(Pedido::Duplicar {
            estilo: style,
            nome: format!("{nome} {}", ph2d_i18n::tr("card.styles.copy")),
        });
    } else {
        pede(Pedido::Apagar { estilo: style });
        if EDITA.with(Cell::get) == Some(style) {
            EDITA.with(|e| e.set(None));
        }
    }
    true
}

/// Antes da pintura: as caixas do «Edit style» e as dicas (a pintura só lê o store).
pub(super) fn prepara(store: &mut WidgetStore) {
    // O botão direito numa linha abre o menu dela: o despacho lê-as do store (o DAG da fundação).
    let aberta = ABERTA.with(Cell::get).is_some();
    let linhas: Vec<(NodeId, u32)> = if aberta {
        ESTILOS.with(|e| e.borrow().iter().map(|x| (linha_de(x.id), x.id)).collect())
    } else {
        Vec::new()
    };
    store.set_color_style_rows(linhas);
    let botoes: Vec<NodeId> = BOTOES
        .with(|v| v.borrow().iter().map(|(b, ..)| *b).collect::<Vec<_>>())
        .into_iter()
        .chain([MAIS, FUNDO])
        .chain(ESTILOS.with(|e| {
            e.borrow()
                .iter()
                .map(|x| linha_de(x.id))
                .collect::<Vec<_>>()
        }))
        .chain((0..ph2d_vec_scene::LINHAS_DA_PILHA).map(solta_de))
        .collect();
    for id in botoes {
        store.register_if_absent(
            id,
            InteractiveState::Button {
                state: crate::widget::ButtonState::Normal,
            },
        );
    }
    caixa_de_texto(store, BUSCA);
    // O «+» abriu o editor do próximo que nascer: ele chegou.
    if EDITA.with(Cell::get) == Some(0) {
        let antes = MAIOR.with(Cell::get);
        if let Some(novo) = ESTILOS
            .with(|e| e.borrow().iter().map(|x| x.id).max())
            .filter(|m| *m > antes)
        {
            EDITA.with(|e| e.set(Some(novo)));
            store.set_focus(Some(NOME));
        }
    }
    let Some(e) = EDITA
        .with(Cell::get)
        .and_then(|id| ESTILOS.with(|v| v.borrow().iter().find(|x| x.id == id).cloned()))
    else {
        return;
    };
    let foco = store.focus_id();
    for (id, valor) in [(NOME, e.name.clone()), (HEX, hex(e.rgba))] {
        caixa_de_texto(store, id);
        if foco != Some(id)
            && let Some(InteractiveState::TextInput {
                text,
                caret,
                selection_anchor,
                ..
            }) = store.get_mut(id)
            && *text != valor
        {
            *caret = valor.len();
            *text = valor;
            *selection_anchor = None;
        }
    }
    if store.tooltip_for(MAIS).is_none() {
        store.set_tooltip(MAIS, ph2d_i18n::tr("card.styles.add"));
    }
}

fn caixa_de_texto(store: &mut WidgetStore, id: NodeId) {
    store.register_if_absent(
        id,
        InteractiveState::TextInput {
            state: crate::widget::TextInputState::Normal,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
    store.mark_cancel_on_escape(id);
}

fn hex(c: [u8; 4]) -> String {
    format!("{:02X}{:02X}{:02X}", c[0], c[1], c[2])
}

thread_local! {
    static PENDENTE: RefCell<Option<(Rect, CardInk)>> = const { RefCell::new(None) };
}

/// Se o ⠿ aberto é de um cartão desta lista, a lista de estilos nasce por baixo do cabeçalho dele,
/// na largura dele, no tema dele.
pub(super) fn stash_aberta(
    store: &WidgetStore,
    hit_index: &HitIndex,
    theme: Theme,
    cards: &[CardDef<'_>],
) {
    let Some((card, _)) = camada_aberta() else {
        return;
    };
    if !cards.iter().any(|c| c.section == card) {
        return;
    }
    if let Some(r) = hit_index.rect_for(crate::ids::grip_de(card)) {
        let ink = CardInk::of_card(store.section_theme(card), theme);
        PENDENTE.with(|p| *p.borrow_mut() = Some((r, ink)));
    }
}

/// ⭐ A lista (E) e o «Edit style» (G), por cima de tudo — depois de todos os cartões.
pub(super) fn paint_pending(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    let Some((card, ink)) = PENDENTE.with(|p| p.borrow_mut().take()) else {
        return;
    };
    let total = ESTILOS.with(|e| e.borrow().len());
    let lista = filtrados(&texto(store, BUSCA).unwrap_or_default());
    let x = card.x + m::CARD_PAD_X;
    let w = (card.w - m::CARD_PAD_X * 2.0).max(0.0);
    let linhas = lista.len().max(1) as f32;
    let h = m::CARD_HEAD_H
        + m::FIELD_H
        + m::FIELD_ROW_GAP
        + m::LABEL_PX
        + m::LABEL_BOTTOM
        + m::FIELD_H * linhas
        + m::SEG_PAD * 2.0;
    // `card` é a pega = o cabeçalho inteiro: a lista nasce por BAIXO dele (maquete E), nunca sobre o título.
    let edita = EDITA
        .with(Cell::get)
        .and_then(|id| estilos().into_iter().find(|x| x.id == id));
    let pilha = h + edita
        .as_ref()
        .map_or(0.0, |_| m::CARD_GAP + editor::altura());
    let caixa = Rect::new(
        x,
        sobe_para_caber(card.y + card.h + m::LABEL_TOP, pilha),
        w,
        h,
    );
    fundo(scene, caixa, &ink);
    hit_index.register(FUNDO, caixa);
    titulo(
        scene,
        text_system,
        &ink,
        caixa,
        ph2d_i18n::tr("card.styles.title"),
    );
    let mais = Rect::new(
        caixa.x + caixa.w - m::HEAD_ICON_BOX - m::SEG_PAD,
        caixa.y + (m::CARD_HEAD_H - m::HEAD_ICON_BOX) * 0.5,
        m::HEAD_ICON_BOX,
        m::HEAD_ICON_BOX,
    );
    icone(scene, hit_index, store, &ink, mais, MAIS, IconId::Add);
    let mut y = caixa.y + m::CARD_HEAD_H;
    let busca = Rect::new(
        caixa.x + m::SEG_PAD,
        y,
        caixa.w - m::SEG_PAD * 2.0,
        m::FIELD_H,
    );
    editor::campo(scene, text_system, hit_index, store, &ink, busca, BUSCA);
    y += m::FIELD_H + m::FIELD_ROW_GAP;
    crate::paint::paint_text(
        text_system,
        scene,
        &format!("{} \u{b7} {}", ph2d_i18n::tr("card.styles.project"), total),
        caixa.x + m::CARD_PAD_X,
        y,
        m::LABEL_PX,
        caixa.w,
        ink.color(ColorToken::Text3),
    );
    y += m::LABEL_PX + m::LABEL_BOTTOM;
    if lista.is_empty() {
        let vazio = if total == 0 {
            "card.styles.empty"
        } else {
            "card.styles.no_match"
        };
        crate::paint::paint_text_elided(
            text_system,
            scene,
            ph2d_i18n::tr(vazio),
            caixa.x + m::CARD_PAD_X,
            y + (m::FIELD_H - m::VALUE_PX) * 0.5,
            m::VALUE_PX,
            caixa.w - m::CARD_PAD_X * 2.0,
            ink.color(ColorToken::Text3),
        );
    }
    let ligado =
        camada_aberta().and_then(|(_, i)| LIGADOS.with(|l| l.borrow().get(i).copied().flatten()));
    // A coluna mede os hex que ELA pinta — a família inteira, nunca uma sonda escrita à mão.
    let hexes: Vec<String> = lista.iter().map(|e| hex(e.rgba)).collect();
    let col_hex = crate::paint::label_column_width(
        text_system,
        m::LABEL_PX,
        hexes.iter().map(String::as_str),
    );
    for e in &lista {
        let lr = Rect::new(
            caixa.x + m::SEG_PAD,
            y,
            caixa.w - m::SEG_PAD * 2.0,
            m::FIELD_H,
        );
        let id = linha_de(e.id);
        hit_index.register(id, lr);
        if ligado == Some(e.id) || store.hover_live(id) > 0.0 {
            fill_rounded_rect(
                scene,
                lr,
                m::SEG_RADIUS,
                ink.color(ColorToken::CardFieldHover),
            );
        }
        let g = m::ICON_GLYPH;
        let a = Rect::new(
            lr.x + (m::ICON_CELL - g) * 0.5,
            lr.y + (lr.h - g) * 0.5,
            g,
            g,
        );
        amostra(scene, a, e.rgba, &ink);
        let tx = lr.x + m::ICON_CELL;
        crate::paint::paint_text_elided(
            text_system,
            scene,
            &e.name,
            tx,
            lr.y + (lr.h - m::VALUE_PX) * 0.5,
            m::VALUE_PX,
            (lr.w - m::ICON_CELL - col_hex - m::FIELD_PAD_R * 2.0).max(0.0),
            ink.color(ColorToken::Text1),
        );
        crate::paint::paint_text(
            text_system,
            scene,
            &hex(e.rgba),
            lr.x + lr.w - m::FIELD_PAD_R - col_hex,
            lr.y + (lr.h - m::LABEL_PX) * 0.5,
            m::LABEL_PX,
            col_hex,
            ink.color(ColorToken::Text3),
        );
        y += m::FIELD_H;
    }
    if let Some(e) = edita {
        editor::editor(scene, text_system, hit_index, store, &ink, caixa, &e);
    }
}

fn fundo(scene: &mut VectorScene, r: Rect, ink: &CardInk) {
    shadow(
        scene,
        r,
        m::CARD_RADIUS,
        ink.color(ColorToken::KanbanListShadow),
        m::LIST_SHADOW_FAR,
    );
    fill_rounded_rect(
        scene,
        r,
        m::CARD_RADIUS,
        ink.color(ColorToken::KanbanCardRaised),
    );
}

fn titulo(scene: &mut VectorScene, text_system: &mut TextSystem, ink: &CardInk, r: Rect, t: &str) {
    crate::paint::paint_text_title_elided(
        text_system,
        scene,
        t,
        r.x + m::CARD_PAD_X,
        r.y + (m::CARD_HEAD_H - m::CARD_TITLE_PX) * 0.5,
        m::CARD_TITLE_PX,
        r.w - m::CARD_PAD_X * 2.0 - m::HEAD_ICON_BOX,
        ink.color(ColorToken::Text1),
    );
}

fn amostra(scene: &mut VectorScene, r: Rect, c: [u8; 4], ink: &CardInk) {
    fill_rounded_rect(
        scene,
        r,
        m::SEG_RADIUS,
        ph2d_vector::Color::from_rgba8(c[0], c[1], c[2], c[3]),
    );
    // FRAME-RAW-OK: o contorno da AMOSTRA de cor é a borda que a separa do cartão — a cor dela é a mensagem
    crate::paint::stroke_rounded_rect(
        scene,
        r,
        m::SEG_RADIUS,
        m::FIELD_BORDER,
        ink.color(ColorToken::Border),
    );
}

#[allow(clippy::too_many_arguments)]
fn icone(
    scene: &mut VectorScene,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    ink: &CardInk,
    b: Rect,
    id: NodeId,
    icon: IconId,
) {
    hit_index.register(id, b);
    if store.hover_live(id) > 0.0 {
        fill_rounded_rect(
            scene,
            b,
            m::HEAD_ICON_RADIUS,
            ink.color(ColorToken::CardFieldHover),
        );
    }
    let g = m::HEAD_ICON_GLYPH;
    paint_icon(
        scene,
        icon,
        Rect::new(b.x + (b.w - g) * 0.5, b.y + (b.h - g) * 0.5, g, g),
        ink.color(ColorToken::Text2),
        StrokeToken::Default.px(),
    );
}

#[path = "cards_estilos_editor.rs"]
mod editor;
