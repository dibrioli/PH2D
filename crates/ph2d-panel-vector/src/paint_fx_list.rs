//! ⭐⭐ **As LISTAS de Effects e Filters no layout do Figma** (F43, dono 01/11: *«Nada de todos
//! botões à mostra. Mas sim dropdown, lista de efeitos, Propriedade dos efeitos abre na lateral
//! esquerda.»*).
//!
//! ```text
//! Effects                          +  ⠿
//! [▣ Zig Zag            ]  👁  −   ◇
//! [▣ Wave               ]  👁  −   ◇
//! ```
//!
//! O cabeçalho leva o **«+»** (um `Dropdown` cujas linhas são os tipos publicados pelo motor); cada
//! efeito aplicado é UMA linha — glifo, nome, olho, «−» e o losango do olho. A linha é também um
//! `Dropdown`: aberta, o painel de propriedades dela pinta-se à ESQUERDA do Inspector
//! ([`crate::fx_side`]). Os campos não moram aqui.
//!
//! As DUAS secções são o mesmo desenho, e a [`Pilha`] é o que as distingue: um id por papel.

use super::*;
use ph2d_a11y::NodeId;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_icon, stroke_rounded_rect};
use ph2d_editor_core::widget::{IconButtonStyle, IconGlyph, paint_icon_button};
use ph2d_tokens::{Radius, StrokeToken};
use ph2d_tool_vector::ids as tv;
use std::cell::RefCell;

/// Qual das duas listas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Pilha {
    /// Os deformadores vetoriais (ADR-0132).
    Efeitos,
    /// Os FX raster (plano 24).
    Filtros,
}

impl Pilha {
    pub(crate) const TODAS: [Self; 2] = [Self::Efeitos, Self::Filtros];

    const fn i(self) -> usize {
        match self {
            Self::Efeitos => 0,
            Self::Filtros => 1,
        }
    }

    /// O «+» do cabeçalho.
    pub(crate) fn add_menu(self) -> NodeId {
        match self {
            Self::Efeitos => tv::VECTOR_FX_ADD_MENU,
            Self::Filtros => ids::FILTER_ADD_MENU,
        }
    }

    /// O X do painel de propriedades.
    pub(crate) fn close(self) -> NodeId {
        match self {
            Self::Efeitos => tv::VECTOR_FX_PROPS_CLOSE,
            Self::Filtros => ids::FILTER_PROPS_CLOSE,
        }
    }

    pub(crate) fn max_rows(self) -> usize {
        match self {
            Self::Efeitos => tv::MAX_FX_ROWS,
            Self::Filtros => ids::MAX_FILTER_ROWS,
        }
    }

    pub(crate) fn max_kinds(self) -> usize {
        match self {
            Self::Efeitos => tv::MAX_FX_KINDS,
            Self::Filtros => ids::MAX_FILTER_KINDS,
        }
    }

    pub(crate) fn row(self, r: usize) -> NodeId {
        match self {
            Self::Efeitos => tv::vector_fx_row_id(r),
            Self::Filtros => ids::filter_row_id(r),
        }
    }

    pub(crate) fn add(self, kind: usize) -> NodeId {
        match self {
            Self::Efeitos => tv::vector_fx_add_id(kind),
            Self::Filtros => ids::filter_add_id(kind),
        }
    }

    pub(crate) fn hide(self, r: usize) -> NodeId {
        match self {
            Self::Efeitos => tv::vector_fx_hide_id(r),
            Self::Filtros => ids::filter_hide_id(r),
        }
    }

    pub(crate) fn remove(self, r: usize) -> NodeId {
        match self {
            Self::Efeitos => tv::vector_fx_remove_id(r),
            Self::Filtros => ids::filter_remove_id(r),
        }
    }

    pub(crate) fn up(self, r: usize) -> NodeId {
        match self {
            Self::Efeitos => tv::vector_fx_up_id(r),
            Self::Filtros => ids::filter_up_id(r),
        }
    }

    pub(crate) fn down(self, r: usize) -> NodeId {
        match self {
            Self::Efeitos => tv::vector_fx_down_id(r),
            Self::Filtros => ids::filter_down_id(r),
        }
    }

    /// Quantas linhas a pilha publicada tem agora.
    pub(crate) fn len(self) -> usize {
        match self {
            Self::Efeitos => state::stack().len(),
            Self::Filtros => state::filters::stack().len(),
        }
    }
}

/// O `Dropdown` `id` está aberto?
pub(crate) fn is_open(store: &WidgetStore, id: NodeId) -> bool {
    matches!(
        store.get(id),
        Some(InteractiveState::Dropdown { open: true, .. })
    )
}

/// O que o corpo deixou para o passe diferido: o «+» aberto e a linha aberta, por pilha.
/// ⚠️ Sobrescreve (não acumula): o corpo pode pintar-se mais de uma vez por quadro.
#[derive(Clone, Copy, Default)]
pub(crate) struct Pendentes {
    pub menu: [Option<Rect>; 2],
    pub props: [Option<(usize, Rect)>; 2],
}

thread_local! {
    static PENDENTES: RefCell<Pendentes> = RefCell::new(Pendentes::default());
}

/// Lê e limpa o que o corpo deixou neste quadro.
pub(crate) fn take_pendentes() -> Pendentes {
    PENDENTES.with(|p| std::mem::take(&mut *p.borrow_mut()))
}

impl BodyCtx<'_> {
    /// O cabeçalho da secção com o «+». `live` = há onde pôr mais um (o «+» apaga-se cheio).
    pub(crate) fn fx_list_header(
        &mut self,
        p: Pilha,
        section: NodeId,
        label: &str,
        live: bool,
        y: f32,
    ) -> (f32, bool) {
        let open = live && is_open(self.store, p.add_menu());
        let r = self.with_rows(|rc| {
            rc.section_header_with_action(
                section,
                label,
                y,
                Some((p.add_menu(), IconId::Plus, open, live)),
            )
        });
        if open && let Some(chip) = self.hit_index.rect_for(p.add_menu()) {
            PENDENTES.with(|s| s.borrow_mut().menu[p.i()] = Some(chip));
        }
        r
    }

    /// Uma linha da lista: `[glifo nome]  👁  −  ◇`. A caixa do nome é o que ABRE as propriedades.
    pub(crate) fn fx_list_row(
        &mut self,
        p: Pilha,
        row: usize,
        glyph: IconId,
        label: &str,
        enabled: bool,
        y: f32,
    ) -> f32 {
        let h = self.row_h;
        let gap = Spacing::Xs.px();
        // ⭐ Na coluna do decorator, o losango do OLHO (ligado/desligado é o canal da linha).
        let (w, dot) = ph2d_editor_core::widget::form_row_columns(self.inner_x, self.inner_w, y, h);
        self.decorator(dot, &[p.hide(row)]);
        let eye = if enabled {
            IconId::Eye
        } else {
            IconId::EyeClosed
        };
        let mut x = self.inner_x + w;
        for (id, icon) in [(p.remove(row), IconId::Minus), (p.hide(row), eye)] {
            x -= h;
            let r = Rect::new(x, y, h, h);
            paint_icon_button(
                r,
                IconGlyph::Builtin(icon),
                IconButtonStyle::Plain,
                self.store.button_visual(id),
                self.scene,
                self.theme,
            );
            self.hit_index.register(id, r);
            x -= gap;
        }
        let field = Rect::new(self.inner_x, y, (x - self.inner_x).max(1.0), h);
        let open = is_open(self.store, p.row(row));
        let radius = Radius::Sm.px();
        let (bg, edge) = if open {
            (ColorToken::AccentSoft, ColorToken::Accent)
        } else {
            (ColorToken::Bg1, ColorToken::Border)
        };
        fill_rounded_rect(self.scene, field, radius, resolve(bg, self.theme));
        stroke_rounded_rect(
            self.scene,
            field,
            radius,
            StrokeToken::Thin.px(),
            resolve(edge, self.theme),
        );
        let ink = resolve(
            if enabled {
                ColorToken::Text1
            } else {
                ColorToken::TextDisabled
            },
            self.theme,
        );
        let pad = Spacing::Sm.px();
        // O desenho do glifo na altura do texto que o segue (o quadrado leva o respiro).
        let g = ph2d_editor_core::paint::icon_side_for_glyph(self.font).min(h);
        let gr = Rect::new(field.x + pad, y + (h - g) * 0.5, g, g);
        paint_icon(self.scene, glyph, gr, ink, StrokeToken::Default.px());
        let tx = gr.x + gr.w + ph2d_tokens::icon_label_gap_px();
        paint_text(
            self.text_system,
            self.scene,
            label,
            tx,
            y + (h - self.font) * 0.5,
            self.font,
            (field.x + field.w - pad - tx).max(1.0),
            ink,
        );
        self.hit_index.register(p.row(row), field);
        if open {
            PENDENTES.with(|s| s.borrow_mut().props[p.i()] = Some((row, field)));
        }
        y + h + self.row_gap
    }
}
