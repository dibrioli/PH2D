//! ⭐ **O painel de uma FERRAMENTA de imagem em cartões** (`docs/CartoesUI/11_RECEITA`, bloco D3): o
//! que Padding, Upscale, Equalize Sizes, Background Removal e Color Equalization têm em comum.
//!
//! Os verbos (`Reset`, `Cancel`, `Apply`) moram numa faixa presa ao fim da lista, fora dos cartões:
//! não se dobram, não se reordenam, não saem para o canvas, e o `Apply` vê-se sem rolar.

use super::*;
use crate::widget::card_field::CardFieldSpec;
use crate::widget::key_mark::Rotulo;

thread_local! {
    static FORCADO: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
    static ADIA: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// A lista está a pintar-se por [`pinta_lista`]: ela é que pinta a lista suspensa, depois da faixa.
pub(super) fn adia_a_lista_suspensa() -> bool {
    ADIA.with(std::cell::Cell::get)
}

/// O painel pinta-se em cartões? `PH2D_CARDS=0` = o corpo de antes.
#[must_use]
pub fn cards_on() -> bool {
    FORCADO
        .with(std::cell::Cell::get)
        .unwrap_or_else(|| std::env::var("PH2D_CARDS").as_deref() != Ok("0"))
}

/// Força o modo NESTA thread: os testes de um processo correm em paralelo. `None` volta ao ambiente.
#[doc(hidden)]
pub fn force_cards_on_this_thread(on: Option<bool>) {
    FORCADO.with(|f| f.set(on));
}

/// Um número do painel: o chip, o passo da seta, o arrasto por px, a fábrica e o valor da ferramenta.
#[derive(Copy, Clone, Debug)]
pub struct Numero {
    pub chip: NodeId,
    pub step: f64,
    pub scrub: f64,
    pub fabrica: f64,
    pub valor: f64,
}

/// Marca cada campo e semeia-o com o valor da ferramenta; o que a mão arrasta não se semeia.
pub fn prepara_numeros(store: &mut WidgetStore, numeros: &[Numero]) {
    let arrastado = store.number_input_drag().map(|d| d.id);
    for n in numeros {
        store.mark_card_field(
            n.chip,
            CardFieldSpec {
                step: n.step,
                scrub: n.scrub,
                default: Some(n.fabrica),
            },
        );
        if arrastado != Some(n.chip) {
            store.set_number_value(n.chip, n.valor);
        }
    }
}

/// O campo de uma linha: o rótulo por cima e a letra da grandeza na pega. Ajuste de ferramenta: sem
/// marca de animação (D17).
#[must_use]
pub fn campo(id: NodeId, label: &str) -> Cell<'_> {
    Cell::new(Widget::Field {
        id,
        glyph: Glyph::Text(corpo::letra_da_grandeza(label)),
    })
    .label(label)
}

/// A caixa de marcar de um ajuste de ferramenta: o id continua `Button`, emite `Click`, e não leva
/// marca de animação (D17).
pub fn caixa(c: &mut CardCtx<'_>, id: NodeId, chave: &'static str, on: bool, y: f32) -> f32 {
    c.check(id, Rotulo::vista(crate::panel::TextKey::new(chave)), on, y)
}

/// Caixas de nome curto, duas por linha (T64): `(id, chave, ligada)`.
pub fn caixas_aos_pares(
    c: &mut CardCtx<'_>,
    caixas: &[(NodeId, &'static str, bool)],
    y: f32,
) -> f32 {
    let (x, w) = (c.x, c.w);
    let meia = ((w - m::FIELD_GAP) * 0.5).max(0.0);
    // A 1.ª linha do corpo sobe o vão, e uma linha de duas caixas sobe-o uma vez, nas duas.
    let mut topo = if c.first { y - m::FIELD_ROW_GAP } else { y };
    c.first = false;
    for par in caixas.chunks(2) {
        let mut fundo = topo;
        for (i, &(id, chave, on)) in par.iter().enumerate() {
            c.x = x + i as f32 * (meia + m::FIELD_GAP);
            c.w = meia;
            fundo = caixa(c, id, chave, on, topo);
        }
        topo = fundo;
    }
    c.x = x;
    c.w = w;
    topo
}

/// Duas células por linha, na ordem dada (T56).
pub fn aos_pares(c: &mut CardCtx<'_>, cells: &[Cell<'_>], y: f32) -> f32 {
    cells.chunks(2).fold(y, |y, par| c.row(par, y))
}

/// Os segmentos de uma escolha: `(id, texto)` e a posição acesa.
#[must_use]
pub fn opcoes<'a>(pares: &[(NodeId, &'a str)], aceso: usize) -> Vec<Seg<'a>> {
    pares
        .iter()
        .enumerate()
        .map(|(n, &(id, text))| Seg {
            id,
            icon: None,
            text,
            on: n == aceso,
        })
        .collect()
}

/// Um cartão sem ações no cabeçalho.
#[must_use]
pub fn cartao(
    section: NodeId,
    title: &'static str,
    summary: Option<String>,
    body: impl Fn(&mut CardCtx<'_>, f32) -> f32 + 'static,
) -> CardDef<'static> {
    CardDef {
        section,
        title,
        actions: Vec::new(),
        add: None,
        summary,
        body: Some(Box::new(body)),
    }
}

/// Os três verbos, `(id, texto)`.
#[derive(Copy, Clone, Debug)]
pub struct Verbos<'a> {
    pub reset: (NodeId, &'a str),
    pub cancel: (NodeId, &'a str),
    pub apply: (NodeId, &'a str),
}

/// A altura da faixa: duas linhas de botão. O `Reset` não fica ao lado do `Apply`.
fn altura_da_faixa() -> f32 {
    m::LIST_PAD_BOTTOM * 2.0 + m::FIELD_H * 2.0 + m::FIELD_ROW_GAP
}

/// ⭐⭐ **A lista e, logo abaixo, a faixa dos verbos.** O «×» da lista é o `cancel`. Devolve o rect da
/// lista mais a faixa: é também o rect flutuante do painel, senão o clique na faixa iria ao canvas.
pub fn pinta_lista(
    ctx: &mut PaintCtx,
    panel: NodeId,
    title: &str,
    cards: &[CardDef<'_>],
    verbos: &Verbos<'_>,
) -> Rect {
    ctx.host
        .store_mut()
        .mark_card_list_close(panel, verbos.cancel.0);
    let h = altura_da_faixa();
    let slot = ctx.slot;
    ctx.slot.h = (slot.h - h - m::LIST_MARGIN).max(0.0);
    // A faixa pinta-se depois da lista, e a lista suspensa aberta tem de ficar por cima da faixa.
    ADIA.with(|a| a.set(true));
    let lista = paint_card_list(ctx, panel, title, cards);
    ADIA.with(|a| a.set(false));
    ctx.slot = slot;
    if lista.w <= 0.0 {
        dropdown::paint_pending(ctx);
        return lista;
    }
    let theme = ctx.host.theme();
    let r = Rect::new(lista.x, lista.y + lista.h + m::LIST_MARGIN, lista.w, h);
    shadow(
        ctx.scene,
        r,
        m::LIST_RADIUS,
        resolve(ColorToken::KanbanListShadow, theme),
        m::LIST_SHADOW_FAR,
    );
    fill_rounded_rect(
        ctx.scene,
        r,
        m::LIST_RADIUS,
        resolve(ColorToken::KanbanList, theme),
    );
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let mut c = CardCtx::flutuante(
        ctx.scene,
        ctx.text_system,
        store,
        hit_index,
        CardInk::of_list(theme),
        r.x + m::LIST_PAD_X,
        (r.w - m::LIST_PAD_X * 2.0).max(0.0),
    );
    let y = c.row(&[botao(verbos.reset, false)], r.y + m::LIST_PAD_BOTTOM);
    c.row(&[botao(verbos.cancel, false), botao(verbos.apply, true)], y);
    dropdown::paint_pending(ctx);
    let tudo = Rect::new(lista.x, lista.y, lista.w, r.y + r.h - lista.y);
    ctx.host.store_mut().set_floating_panel_rect(panel, tudo);
    tudo
}

fn botao((id, text): (NodeId, &str), primary: bool) -> Cell<'_> {
    Cell::new(Widget::Button { id, text, primary })
}

/// As alturas que o painel publica para os seus censos: `(conteúdo, visível)`.
#[must_use]
pub fn alturas(store: &WidgetStore, panel: NodeId) -> (f32, f32) {
    (
        store.panel_content_h(panel).unwrap_or(0.0),
        store.panel_visible_h(panel).unwrap_or(0.0),
    )
}
