//! ⭐ **As abas da Timeline são abas de PASTA na borda esquerda** (dono 13/11,
//! `docs/Timeline/fotos/18_pedido_abas_laterais_esboco.png`): Keys · Containers · (um por container
//! aberto) · Arrange, empilhadas de cima para baixo, coladas à moldura, por fora, texto a ler de baixo
//! para cima. A ATIVA inteira, à frente, na tinta da lista (painel e aba leem-se como um objecto); as
//! INATIVAS por trás da moldura, apagadas, sobrepostas à vizinha, com só o começo do nome à vista.
//!
//! A lista de abas, a escolhida e os ids são os de [`crate::transport_tabs`] — o clique é o de sempre.
//! O cartão reserva a calha delas à esquerda ([`sem_a_calha`]): o painel pode estar encostado à borda
//! da janela e as abas não saem dele.

use ph2d_editor_core::paint;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::banda::{self, metrics as m};
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::TimelineViewSnapshot;
use ph2d_tokens::{ColorToken, Theme};

use crate::tab::Tab;

/// A grossura de uma aba (a largura da calha): a altura de um campo da barra.
pub(crate) const ESPESSURA: f32 = m::FIELD_H;

/// O comprimento à vista de uma aba INATIVA: o começo do nome (dono 13/11).
const ESPIA: f32 = m::FIELD_H * 1.5;

/// O pedaço de uma inativa que fica escondido debaixo da vizinha da frente (a sobreposição).
const SOB: f32 = m::FIELD_RADIUS;

/// O tamanho do nome numa aba: o das abas da barra de antes.
const TEXTO_PX: f32 = m::SEG_TEXT_PX;

/// O rect do cartão SEM a calha das abas: é dele que a moldura, a barra e a folha se medem.
pub(crate) fn sem_a_calha(rect: Rect) -> Rect {
    Rect::new(
        rect.x + ESPESSURA,
        rect.y,
        (rect.w - ESPESSURA).max(0.0),
        rect.h,
    )
}

/// Uma aba posta: o alvo (a parte À VISTA), o desenho (o que se pinta, com o pedaço escondido) e o
/// nome que cabe à vista.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Aba {
    pub(crate) id: ph2d_a11y::NodeId,
    pub(crate) nome: String,
    /// O nome inteiro (a dica da aba: a inativa só mostra o começo).
    pub(crate) inteiro: String,
    pub(crate) alvo: Rect,
    pub(crate) desenho: Rect,
    pub(crate) ativa: bool,
    /// A distância (em abas) à ativa: as mais longe pintam-se primeiro, por trás.
    pub(crate) fundo: usize,
}

/// O começo de `nome` que cabe em `w` (por carácter, sem reticências: a aba cobre o resto).
fn comeco(nome: &str, w: f32, largura: &mut dyn FnMut(&str) -> f32) -> String {
    if largura(nome) <= w {
        return nome.to_owned();
    }
    let mut fim = 0;
    for (i, c) in nome.char_indices() {
        let j = i + c.len_utf8();
        if largura(&nome[..j]) > w {
            break;
        }
        fim = j;
    }
    nome[..fim].to_owned()
}

/// ⭐ **A pilha**: cada aba na ordem de [`crate::transport_tabs::items`], de cima para baixo a partir
/// do fim da quina da lista. A ativa mede o nome inteiro; as outras, [`ESPIA`]. Pura (a largura do
/// texto entra por fora), para o gate medir sem pintar.
pub(crate) fn pilha(
    lista: Rect,
    snap: &TimelineViewSnapshot,
    tab: Tab,
    largura: &mut dyn FnMut(&str) -> f32,
) -> Vec<Aba> {
    let celulas = crate::transport_tabs::items(snap);
    let escolhida = crate::transport_tabs::selected(tab, snap);
    let n = celulas.len();
    let x = lista.x - ESPESSURA;
    let topo = lista.y + m::LIST_RADIUS;
    // A ativa cabe no que sobra da altura da lista depois das espias (painel baixo: o nome corta).
    let cabe = (lista.h - m::LIST_RADIUS * 2.0 - ESPIA * n.saturating_sub(1) as f32).max(ESPIA);
    let pad = m::SEG_INSET;
    let mut y = topo;
    let mut out = Vec::with_capacity(n);
    for (i, (id, nome)) in celulas.into_iter().enumerate() {
        let ativa = i == escolhida;
        let h = if ativa {
            // O nome, a folga de cada ponta e o raio da quina de fora.
            (largura(&nome) + pad * 2.0 + m::FIELD_RADIUS).clamp(ESPIA, cabe.max(ESPIA)) // CLAMP-OK: cabe ≥ ESPIA
        } else {
            ESPIA
        };
        let alvo = Rect::new(x, y, ESPESSURA, h);
        // A ativa entra um pouco na moldura (a junta não se vê); a inativa estende-se para baixo da
        // vizinha do lado da ativa.
        let desenho = if ativa {
            Rect::new(x, y, ESPESSURA + m::SEG_INSET, h)
        } else if i < escolhida {
            Rect::new(x, y, ESPESSURA + m::LIST_RADIUS, h + SOB)
        } else {
            Rect::new(x, y - SOB, ESPESSURA + m::LIST_RADIUS, h + SOB)
        };
        out.push(Aba {
            id,
            nome: comeco(&nome, h - pad * 2.0, largura),
            inteiro: nome,
            alvo,
            desenho,
            ativa,
            fundo: i.abs_diff(escolhida),
        });
        y += h;
    }
    out
}

fn pilha_pintada(
    ctx: &mut PaintCtx,
    painel: Rect,
    snap: &TimelineViewSnapshot,
    tab: Tab,
) -> Vec<Aba> {
    let lista = banda::geometria(painel).lista;
    let ts = &mut *ctx.text_system;
    pilha(lista, snap, tab, &mut |t| ts.prefix_width(t, TEXTO_PX))
}

fn pinta_uma(ctx: &mut PaintCtx, theme: Theme, a: &Aba) {
    let tons = crate::transport::cartoes::tons_da_barra(theme);
    let fundo = paint::resolve(ColorToken::KanbanList, theme);
    let hover = ctx.host.store().hover_live(a.id) > 0.0;
    let (cor, texto) = if a.ativa {
        (fundo, tons.ativa_texto)
    } else if hover {
        (fundo, tons.inativa_texto)
    } else {
        let alfa = ph2d_tokens::chrome::TIMELINE_ABA_INATIVA_ALPHA;
        (
            fundo.multiply_alpha(alfa),
            tons.inativa_texto.multiply_alpha(alfa),
        )
    };
    let r = m::FIELD_RADIUS;
    paint::fill_rounded_rect_radii(ctx.scene, a.desenho, (r, 0.0, 0.0, r), cor);
    let tx = a.alvo.x + paint::folga_do_texto(a.alvo.w, TEXTO_PX);
    let ty = a.alvo.y + a.alvo.h - m::SEG_INSET;
    paint::paint_text_rotated_ccw(
        ctx.text_system,
        ctx.scene,
        &a.nome,
        tx,
        ty,
        TEXTO_PX,
        a.alvo.h,
        texto,
    );
}

/// As INATIVAS, por trás: pintam-se ANTES da moldura, que lhes cobre o pedaço de dentro.
pub(crate) fn pinta_de_tras(
    ctx: &mut PaintCtx,
    theme: Theme,
    painel: Rect,
    (snap, tab): (&TimelineViewSnapshot, Tab),
) {
    let mut abas = pilha_pintada(ctx, painel, snap, tab);
    abas.retain(|a| !a.ativa);
    // A mais longe da ativa primeiro: cada uma fica por baixo da vizinha mais perto da frente.
    abas.sort_by_key(|a| std::cmp::Reverse(a.fundo));
    for a in &abas {
        pinta_uma(ctx, theme, a);
    }
}

/// A ATIVA, à frente (depois da moldura), e os alvos de todas: o da ativa é ela inteira, o de cada
/// inativa a parte à vista.
pub(crate) fn pinta_da_frente(
    ctx: &mut PaintCtx,
    theme: Theme,
    painel: Rect,
    (snap, tab): (&TimelineViewSnapshot, Tab),
) {
    let abas = pilha_pintada(ctx, painel, snap, tab);
    ph2d_editor_core::widget::composto::grupo(abas.iter().map(|a| a.id));
    if let Some(a) = abas.iter().find(|a| a.ativa) {
        pinta_uma(ctx, theme, a);
    }
    for a in &abas {
        ctx.host.hit_index_mut().register(a.id, a.alvo);
        let store = ctx.host.store_mut();
        if store.tooltip_for(a.id) != Some(a.inteiro.as_str()) {
            store.set_tooltip(a.id, a.inteiro.clone());
        }
    }
}

#[cfg(test)]
#[path = "abas_laterais_tests.rs"]
mod tests;
