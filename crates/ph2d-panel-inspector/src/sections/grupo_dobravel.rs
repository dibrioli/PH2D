//! O GRUPO dobrável de um cartão composto — irmão de [`super::rows`] pelo tecto de 600 LOC dos painéis.

use super::*;

/// ⭐ **Um GRUPO dobrável de um cartão composto** (HUD, paralaxe, câmera): o cabeçalho tem o NOME
/// do componente dono das linhas, que é o prefixo com que um script as lê
/// (`camera_follow.damping`). Lei e oráculo: `docs/CodeEditor/12_cartoes_compostos.md`.
///
/// O chamador pinta o corpo em `x`/`w` a partir de `y`, só se [`Grupo::aberto`], e
/// devolve o `y` final pelo [`Grupo::fecha`]. ⚠️ O id tem de estar marcado
/// `mark_collapsible_section` no `populate` — senão o clique no cabeçalho morre.
pub(crate) struct Grupo {
    fold: Option<ph2d_editor_core::widget::SectionFold>,
    pub(crate) x: f32,
    pub(crate) w: f32,
    pub(crate) y: f32,
}

impl Grupo {
    pub(crate) fn aberto(&self) -> bool {
        self.fold.is_some()
    }

    pub(crate) fn fecha(
        self,
        store: &WidgetStore,
        scene: &mut VectorScene,
        hit_index: &mut HitIndex,
        cur_y: f32,
    ) -> f32 {
        match self.fold {
            Some(fold) => fold.finish(store, scene, hit_index, cur_y),
            None => self.y,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn grupo_dobravel(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    id: NodeId,
    texto: &str,
) -> Grupo {
    use ph2d_editor_core::widget::{SectionHeader, group_header_h, paint_group_header};
    let header = SectionHeader::new(id, texto)
        .collapsible(!store.is_collapsed(id))
        .open_t(store.section_open_live(id));
    let rect = Rect::new(x, y, w, group_header_h());
    paint_group_header(&header, rect, scene, text_system, theme);
    hit_index.register(id, rect);
    let body_top = y + rect.h + ph2d_tokens::control_gap_px();
    // ⛔ Sem recuo do corpo (o Godot recua): os nossos rótulos são à DIREITA, o recuo não se via
    // e só tirava o valor da coluna única do tronco (`onde_comeca_o_valor`, o HUD a 142 contra 111).
    Grupo {
        fold: ph2d_editor_core::widget::SectionFold::begin(
            store, id, x, w, body_top, scene, hit_index,
        ),
        x,
        w,
        y: body_top,
    }
}
