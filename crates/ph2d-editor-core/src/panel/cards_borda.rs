//! ⭐ **A BORDA DE FORA de uma janela acessória** — a alça que a alarga, como a da coluna do
//! Inspector (ordem do dono, 08/11: *«O painel Slices e outros painéis acessórios devem ter a
//! alcinha de redimensionamento como o inspector»*).
//!
//! As janelas acessórias (Slices, Tilemap) flutuam à ESQUERDA da coluna, com a direita presa a ela;
//! a alça é a esquerda — a borda longe da coluna. A mesma faixa ([`DOCK_SEAM_PX`]), a mesma marca
//! ([`paint_seam_mark`]), a mesma seta ↔ (`board_view::pointer_look`). O estado é do store
//! (`state::accessory_width_ops`); aqui moram o piso, o tecto e o desenho.

use crate::interaction::{BlenderHitKind, InteractiveState, WidgetStore};
use crate::math::safe_clamp;
use crate::panel::PaintCtx;
use crate::panel::cards::banda::metrics as m;
use crate::screens::hero::dock_reopen::paint_seam_mark;
use crate::screens::layout::DOCK_SEAM_PX;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_tokens::{PANEL_MIN_W_PX, Spacing};

/// ⭐ **O tecto da largura** — metade do vão à esquerda da coluna, menos as três folgas.
///
/// O recurso é a ÁREA de desenho: as duas janelas acessórias (Tilemap e Slices) empilham-se lado a
/// lado à esquerda da coluna, e com metade cada uma as duas cabem juntas no máximo sem que a de
/// fora seja empurrada contra a beira da área (o `max(area.x)` da posição) e tape a vizinha. Na
/// bancada do dono (`1 930 px`) isso dá ~`600 px` por janela, quase três vezes o mínimo. O piso é
/// o [`PANEL_MIN_W_PX`]: abaixo dele o cabeçalho e uma linha deixam de caber juntos.
#[must_use]
pub fn tecto(area: Rect, coluna_x: f32) -> f32 {
    ((coluna_x - area.x - Spacing::Lg.px() * 3.0) * 0.5).max(PANEL_MIN_W_PX)
}

/// ⭐ **O fundo de uma janela acessória**: acima da faixa da pílula do rodapé (a dica da ferramenta),
/// que senão se pinta por cima da última linha (foto `P12_smoke_07`, dono 08/11).
#[must_use]
pub fn fundo(layout: &crate::screens::layout::HeroLayout) -> f32 {
    let area = layout.draw_area;
    (area.y + area.h).min(layout.bottom_hud.y) - Spacing::Sm.px()
}

/// A largura da janela `panel` neste quadro — a escolha do artista limitada ao piso e ao tecto; o
/// limitado volta ao store, para o próximo arrasto partir do que se vê (sem borracha).
pub fn largura(store: &mut WidgetStore, panel: NodeId, area: Rect, coluna_x: f32) -> f32 {
    largura_com_piso(store, panel, area, coluna_x, PANEL_MIN_W_PX)
}

/// [`largura`] com o piso que o CONTEÚDO da janela pede ([`piso_das_linhas`]); a largura de
/// fábrica continua o [`PANEL_MIN_W_PX`].
pub fn largura_com_piso(
    store: &mut WidgetStore,
    panel: NodeId,
    area: Rect,
    coluna_x: f32,
    piso: f32,
) -> f32 {
    let Some(escolha) = store.accessory_width(panel) else {
        return PANEL_MIN_W_PX;
    };
    let tecto = tecto(area, coluna_x);
    let w = escolha.clamp(piso.min(tecto), tecto);
    store.set_accessory_width(panel, w);
    w
}

/// O que a janela gasta à volta do texto de uma linha da lista (`ListRow`): as margens da lista,
/// o recuo dos cartões, o do corpo do cartão e o da linha, dos dois lados.
pub const MOLDURA_DA_LINHA: f32 =
    (m::LIST_MARGIN + m::LIST_PAD_X + m::CARD_PAD_X + m::SEG_INSET) * 2.0;

/// O cabeçalho mais estreito que ainda se usa: as margens, o «…» e o «×» ([`declara_fecho`]) e
/// uma caixa de ícone de título (o resto do nome elide-se).
pub const PISO_DO_CABECALHO: f32 =
    (m::LIST_MARGIN + m::LIST_PAD_X + m::LIST_HEAD_PAD_X) * 2.0 + m::HEAD_ICON_BOX * 3.0;

/// ⭐ **O piso medido no conteúdo**: a linha mais larga de `textos` (no corpo das linhas da lista)
/// inteira, mais a moldura — e nunca abaixo do cabeçalho. Abaixo disto um nome de linha corta-se.
#[must_use]
pub fn piso_das_linhas<'a>(
    ts: &mut ph2d_text::TextSystem,
    textos: impl IntoIterator<Item = &'a str>,
) -> f32 {
    let maior = textos
        .into_iter()
        .map(|t| ts.prefix_width(t, m::VALUE_PX))
        .fold(0.0_f32, f32::max);
    (maior.ceil() + MOLDURA_DA_LINHA).max(PISO_DO_CABECALHO)
}

/// ⭐ **O rect de uma janela acessória à esquerda da coluna** (Slices, Tilemap, Undo History, Keys,
/// Preferences): de omissão `base_x` (a direita colada à coluna), abaixo de `teto`, até `fundo`;
/// o arrasto do título soma-se, limitado a `[esquerda, base_x] × [teto, fundo − cabeçalho]` (o
/// limitado volta ao store, sem borracha) e, no quadro em que abre, [`assenta_ao_abrir`].
pub fn coloca(
    store: &mut WidgetStore,
    panel: NodeId,
    w: f32,
    x: (f32, f32),
    y: (f32, f32),
) -> Rect {
    let ((esquerda, base_x), (teto, fundo)) = (x, y);
    let base_x = base_x.max(esquerda);
    let piso_y = (fundo - m::LIST_HEAD_H).max(teto);
    let (dx, dy) = store.blender_picker_offset(panel);
    let px = safe_clamp(base_x + dx, esquerda, base_x);
    let py = safe_clamp(teto + dy, teto, piso_y);
    if (px - base_x, py - teto) != (dx, dy) {
        store.set_blender_picker_offset(panel, px - base_x, py - teto);
    }
    let r = Rect::new(px, py, w, (fundo - py).max(0.0));
    assenta_ao_abrir(
        store,
        panel,
        r,
        Rect::new(esquerda, teto, base_x + w - esquerda, fundo - teto),
    )
}

/// ⭐ **A janela que ABRE não cai sobre outra** (smoke do dono 08/11: a Undo History reaberta caiu
/// sobre a Slices). `r` é o rect de omissão da janela `panel`, já com o arrasto; `dentro` é onde ela
/// pode ficar (o topo dela em `[dentro.y, fundo − cabeçalho]`, o fundo dela é o de `dentro`). Só no
/// quadro em que ela abre (ainda sem rect publicado): fora dele quem manda é o arrasto do artista.
/// Os obstáculos são TODOS os rects publicados neste ecrã — janelas acessórias, a Preview, o Export,
/// o seletor de cor, cartões soltos, os painéis encaixados. Os candidatos são o próprio sítio e os
/// colados a cada obstáculo nos quatro lados que caem em `dentro`; ganha a menor área sobreposta e,
/// no empate, o mais perto do sítio de omissão. O desvio escolhido vai para o arrasto da janela.
pub fn assenta_ao_abrir(store: &mut WidgetStore, panel: NodeId, r: Rect, dentro: Rect) -> Rect {
    if store.panel_rect(panel).is_some() {
        return r;
    }
    let fundo = (r.y + r.h).max(dentro.y + dentro.h);
    let outras: Vec<Rect> = store
        .panel_rects_by_id()
        .filter(|&(id, _)| id != panel)
        .map(|(_, o)| o)
        .collect();
    let em = |(x, y): (f32, f32)| Rect::new(x, y, r.w, (fundo - y).max(0.0));
    let sobra = |p: (f32, f32)| -> f32 {
        let a = em(p);
        outras
            .iter()
            .map(|o| {
                let w = (a.x + a.w).min(o.x + o.w) - a.x.max(o.x);
                let h = (a.y + a.h).min(o.y + o.h) - a.y.max(o.y);
                w.max(0.0) * h.max(0.0)
            })
            .sum()
    };
    let folga = Spacing::Lg.px();
    let (x0, x1) = (dentro.x, (dentro.x + dentro.w - r.w).max(dentro.x));
    let (y0, y1) = (dentro.y, (fundo - m::LIST_HEAD_H).max(dentro.y));
    let xs = std::iter::once(r.x)
        .chain(outras.iter().map(|o| o.x - folga - r.w))
        .chain(outras.iter().map(|o| o.x + o.w + folga));
    let ys: Vec<f32> = std::iter::once(r.y)
        .chain(outras.iter().map(|o| o.y + o.h + folga))
        .collect();
    let cabe =
        |&(x, y): &(f32, f32)| x >= x0 - 0.5 && x <= x1 + 0.5 && y >= y0 - 0.5 && y <= y1 + 0.5;
    let longe = |(x, y): (f32, f32)| (x - r.x).abs() + (y - r.y).abs();
    let p = xs
        .flat_map(|x| ys.iter().map(move |&y| (x, y)))
        .filter(cabe)
        .map(|p| (sobra(p), longe(p), p))
        .min_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)))
        .map_or((r.x, r.y), |(_, _, p)| p);
    if p != (r.x, r.y) {
        let (dx, dy) = store.blender_picker_offset(panel);
        store.set_blender_picker_offset(panel, dx + p.0 - r.x, dy + p.1 - r.y);
    }
    em(p)
}

/// O id do «×» da janela `panel` (derivado dele: uma janela, um fecho).
#[must_use]
pub fn fecho_de(panel: NodeId) -> NodeId {
    const SAL_DO_FECHO: u64 = 0x2C6F_E1A9_0B37_D45E;
    NodeId(panel.0 ^ SAL_DO_FECHO)
}

/// ⭐ Declara o «×» da janela `panel` (antes do `paint_card_list`): o cabeçalho pinta-o no mesmo
/// sítio que o dos painéis encaixados (`cards_menu::pinta_fecho`).
pub fn declara_fecho(store: &mut WidgetStore, panel: NodeId) {
    let f = fecho_de(panel);
    store.register_if_absent(
        f,
        InteractiveState::Button {
            state: crate::widget::ButtonState::Normal,
        },
    );
    store.mark_card_list_close(panel, f);
}

/// O clique no «×» da janela `panel` esconde-a (`id` é o `Panel::ID`). `true` se era ele.
pub fn fechou(
    host: &mut dyn crate::panel::PanelHostInternal,
    ev: &crate::interaction::WidgetEvent,
    panel: NodeId,
    id: &'static str,
) -> bool {
    let hit = matches!(ev, crate::interaction::WidgetEvent::Click(c) if *c == fecho_de(panel));
    if hit {
        host.set_panel_visible(id, false);
    }
    hit
}

/// A reserva da alça do título: o «…» e o «×» à direita não são alça.
#[must_use]
pub fn reserva_do_titulo() -> f32 {
    (m::HEAD_ICON_BOX + m::LIST_HEAD_PAD_X) * 3.0
}

/// Regista o widget da alça `id` da janela `panel` (no `populate` do painel).
pub fn regista(store: &mut WidgetStore, id: NodeId, panel: NodeId) {
    store.register(
        id,
        InteractiveState::BlenderHit {
            parent: panel,
            kind: BlenderHitKind::WidthEdge,
        },
    );
}

/// A faixa de agarre: os primeiros [`DOCK_SEAM_PX`] da janela, dentro dela (logo fora do canvas).
#[must_use]
pub fn faixa(janela: Rect) -> Rect {
    Rect::new(janela.x, janela.y, DOCK_SEAM_PX.min(janela.w), janela.h)
}

/// Pinta a marca e regista a faixa — DEPOIS da lista, para ganhar dos cartões por baixo.
pub fn pega(ctx: &mut PaintCtx, id: NodeId, janela: Rect) {
    let r = faixa(janela);
    if r.w <= 0.0 || r.h <= 0.0 {
        return;
    }
    let theme = ctx.host.theme();
    paint_seam_mark(ctx.scene, r, theme);
    ctx.host.hit_index_mut().register(id, r);
}

/// `true` se `id` é uma alça de largura — a pergunta da seta ↔.
#[must_use]
pub fn e_alca(store: &WidgetStore, id: NodeId) -> bool {
    matches!(
        store.get(id),
        Some(InteractiveState::BlenderHit {
            kind: BlenderHitKind::WidthEdge,
            ..
        })
    )
}
