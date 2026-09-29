//! ⭐ **Pinta e, se o controlo estiver abaixo da dobra, ROLA até ele aparecer** — o que o artista
//! faria.
//!
//! ⚠️ Existe desde a porta `scroll_area` (2026-09-29): o `HitIndex` deste painel passou a ser
//! recortado pelo corpo, e o painel flutuante nasce com `640 px` — mais alto que o viewport não o
//! faz crescer. Um controlo rolado para fora **não se clica** (é a cura), logo um gate que quer
//! clicá-lo tem de rolar primeiro; crescer a janela não serve aqui.
//!
//! ⚠️ O passo é **meia** altura visível: todo controlo mais baixo que isso aparece inteiro em
//! algum dos passos, então o centro do rect registado é um ponto do próprio controlo.

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids::GS_PANEL;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_grid_snap::GridSnapPanel;
use ph2d_panel_grid_snap::state::GridSnapPanelState;
use ph2d_ui_testkit::MockPanelHost;

pub(crate) fn pinta_ate_ver(
    host: &mut MockPanelHost,
    state: &mut GridSnapPanelState,
    viewport: Rect,
    id: NodeId,
) -> Option<Rect> {
    let mut scroll = 0.0_f32;
    loop {
        host.set_panel_scroll(GS_PANEL, scroll);
        let painted = host.paint::<GridSnapPanel>(state, viewport);
        if let Some((_, r)) = painted.iter().rev().find(|(pid, _)| *pid == id) {
            return Some(*r);
        }
        let conteudo = host.store().panel_content_h(GS_PANEL)?;
        let visivel = host.store().panel_visible_h(GS_PANEL)?;
        let fim = (conteudo - visivel).max(0.0);
        if visivel <= 0.0 || scroll >= fim {
            return None;
        }
        scroll = (scroll + visivel * 0.5).min(fim);
    }
}
