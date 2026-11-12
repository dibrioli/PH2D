//! **A memória de SESSÃO da paleta `Ctrl+K`**: os comandos escolhidos por último («Recent») e a
//! seleção que rola para ficar à vista. Vive no hero; gravar em disco é a onda 3.

use crate::widget::command_palette::{CMD_PALETTE_CARD, follow_scroll, item_ids, row_span, scoped};
use ph2d_a11y::NodeId;

/// Quantos recentes a sessão guarda — mais do que a lista mostra, porque um recente que o modelo
/// de agora não oferece não aparece. Escolha de desenho, não tecto de recurso.
const RECENT_KEPT: usize = 20;

/// Os recentes e o que a lista já seguiu.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PaletteSession {
    recent: Vec<NodeId>,
    /// O destaque para o qual a lista já rolou, e a busca que ele respondia.
    followed: Option<NodeId>,
    query: String,
}

impl PaletteSession {
    /// Os últimos comandos escolhidos, o mais novo primeiro, sem repetir.
    #[must_use]
    pub fn recent(&self) -> &[NodeId] {
        &self.recent
    }

    /// `id` foi escolhido: sobe ao topo.
    pub fn remember(&mut self, id: NodeId) {
        self.recent.retain(|r| *r != id);
        self.recent.insert(0, id);
        self.recent.truncate(RECENT_KEPT);
    }

    /// A paleta abriu: nada foi seguido ainda.
    pub(super) fn opened(&mut self) {
        self.followed = None;
        self.query.clear();
    }
}

/// ⭐ **A linha selecionada rola para ficar à vista** (uma vez por quadro, com a paleta em lista
/// aberta). Busca nova ⇒ o destaque volta à primeira linha. Só age quando o destaque MUDA: a roda
/// continua livre.
pub(super) fn follow(hero: &mut super::super::HeroScreen) {
    let store = &mut hero.store;
    let Some(model) = store.command_palette_model().filter(|m| m.is_list()) else {
        return;
    };
    let query = store.command_palette_query().to_owned();
    let view = scoped(model, &query, store.command_palette_scopes());
    let session = &mut hero.palette_session;
    let retyped = session.query != query;
    let first = item_ids(&view.model).first().copied();
    let chosen = if retyped {
        first
    } else {
        store.command_palette_highlight()
    };
    let span = chosen.and_then(|id| row_span(&view.model, id));
    drop(view);
    if retyped {
        session.query = query;
        if let Some(id) = first {
            store.highlight_command_palette(id);
        }
    }
    if session.followed == chosen {
        return;
    }
    session.followed = chosen;
    let (Some(span), Some(view_h)) = (span, store.panel_visible_h(CMD_PALETTE_CARD)) else {
        return;
    };
    let now = store.panel_scroll_target(CMD_PALETTE_CARD);
    let next = follow_scroll(span, now, view_h);
    if (next - now).abs() > f32::EPSILON {
        store.set_panel_scroll(CMD_PALETTE_CARD, next);
    }
}
