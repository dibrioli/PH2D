//! ⭐ **A BORDA DE FORA de uma janela acessória** — a alça que a alarga, como a da coluna do
//! Inspector (ordem do dono, 08/11: *«O painel Slices e outros painéis acessórios devem ter a
//! alcinha de redimensionamento como o inspector»*).
//!
//! As janelas acessórias (Slices, Tilemap) flutuam à ESQUERDA da coluna, com a direita presa a ela;
//! a alça é a esquerda — a borda longe da coluna. A mesma faixa ([`DOCK_SEAM_PX`]), a mesma marca
//! ([`paint_seam_mark`]), a mesma seta ↔ (`board_view::pointer_look`). O estado é do store
//! (`state::accessory_width_ops`); aqui moram o piso, o tecto e o desenho.

use crate::interaction::{BlenderHitKind, InteractiveState, WidgetStore};
use crate::panel::PaintCtx;
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
    let Some(escolha) = store.accessory_width(panel) else {
        return PANEL_MIN_W_PX;
    };
    let w = escolha.clamp(PANEL_MIN_W_PX, tecto(area, coluna_x));
    store.set_accessory_width(panel, w);
    w
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
