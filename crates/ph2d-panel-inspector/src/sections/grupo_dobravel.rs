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
    // No cartão as linhas não trazem folga em baixo: o grupo pede-a em cima, como um rótulo.
    let y = y + ph2d_editor_core::widget::card_ink::card_body().map_or(0.0, |_| Spacing::Sm.px());
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

/// O resumo de um grupo dobrado, à direita no cabeçalho dele: vem inteiro ou não vem. `corpo_y` é o
/// [`Grupo::y`]. O nome manda: sem folga para os dois, fica só o nome.
#[allow(clippy::too_many_arguments)]
pub(crate) fn resumo_do_grupo(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    corpo_y: f32,
    titulo: &str,
    resumo: &str,
) {
    use ph2d_editor_core::widget::group_header_h;
    let h = group_header_h();
    let font = TypeToken::Sm.px();
    let ocupado = h + text_system.prefix_width(titulo, font).ceil() + Spacing::Sm.px();
    let pede = text_system.prefix_width(resumo, font).ceil();
    if ocupado + pede > w {
        return;
    }
    let y = corpo_y - ph2d_tokens::control_gap_px() - h;
    ph2d_editor_core::paint::paint_text(
        text_system,
        scene,
        resumo,
        x + w - pede,
        y + ph2d_editor_core::paint::folga_do_texto(h, font),
        font,
        pede + 1.0, // LITERAL-PX-OK: folga de arredondamento da medida
        resolve(ColorToken::Text3, theme),
    );
}
