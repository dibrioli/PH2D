//! ⭐⭐ **Fill & Stroke numa ÚNICA lista** (F45, dono 01/11: *«uma única seção para fill e stroke e
//! que a fila de sobreposição funcione na ordem em que são colocadas […] clicar e arrastar o ícone de
//! config deve ser usado para mudar a ordem»*).
//!
//! As linhas saem na ordem da pilha da forma (`Appearance::stack`, o topo em cima): a de cima pinta
//! por cima, a tinta de base incluída. Arrastar pelo ⚙ (família [`PanelRowFamily::VectorPaint`]) é o
//! gesto do cartão do Kanban: a linha VOA sob o cursor (`paint_flight`), as outras abrem o vão onde
//! ela vai cair e uma linha de acento marca-o. ⭐ A queda é função SÓ do lugar do cursor (a ranhura
//! sob ele), e é a MESMA conta que pinta o vão e que o largar aplica ([`take_landing`]).

use super::*;
use crate::paint_sections::fx_list::is_open;
use ph2d_a11y::NodeId;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::PanelRowFamily;
use ph2d_editor_core::paint::fill_rounded_rect;
use ph2d_editor_core::widget::card_ink::card_body;
use ph2d_tokens::{Radius, StrokeToken};
use ph2d_vec_scene::{Place, StackItem};
use std::cell::{Cell, RefCell};

/// O que a lista deixou para o passe diferido ([`deferred`]).
#[derive(Default)]
struct Pendente {
    /// Os ⚙ arrastáveis deste quadro.
    gears: Vec<NodeId>,
    /// O «+» aberto (a caixa dele).
    menu: Option<Rect>,
    /// A linha que voa: o desenho dela (na origem) e onde pô-lo.
    voo: Option<(ph2d_vector::VectorScene, Rect)>,
}

thread_local! {
    static PENDENTE: RefCell<Pendente> = RefCell::new(Pendente::default());
    /// A queda que o vão mostra AGORA — o largar aplica esta, e não outra conta.
    static LANDING: Cell<Option<(StackItem, Place)>> = const { Cell::new(None) };
}

/// A queda prevista do arrasto em curso (uma vez) — o `PanelRowReparent` da família aplica-a.
pub(crate) fn take_landing() -> Option<(StackItem, Place)> {
    LANDING.with(Cell::take)
}

/// ⭐ **Para onde ↑/↓ leva a camada `i`** na pilha à vista (a vizinha pode ser a tinta de base):
/// `None` = não há vizinha desse lado; senão o índice que ela passa a ter no documento (passar
/// por uma base não o muda; trocar com uma camada dá o dela).
pub(crate) fn shifted(i: usize, up: bool) -> Option<usize> {
    let a = state::current_appearance()?;
    // A pilha publicada já é a à vista; o filtro cobre a neutra (sem pilha publicada).
    let vistas: Vec<StackItem> = a
        .stack()
        .into_iter()
        .filter(|it| match *it {
            StackItem::BaseFill => state::base_fill() != Some(state::BasePaint::Absent),
            StackItem::BaseStroke => state::stroke_present() != Some(false),
            StackItem::Entry(_) => true,
        })
        .collect();
    let k = vistas.iter().position(|x| *x == StackItem::Entry(i))?;
    let v = if up {
        k.checked_add(1)?
    } else {
        k.checked_sub(1)?
    };
    match *vistas.get(v)? {
        StackItem::Entry(j) => Some(j),
        _ => Some(i),
    }
}

/// O ⚙ de uma linha — o que se arrasta e o que abre os ajustes dela.
pub(crate) fn gear_of(it: StackItem) -> NodeId {
    match it {
        StackItem::BaseFill => ids::VECTOR_FILL_ROW,
        StackItem::BaseStroke => ids::VECTOR_STROKE_SETTINGS,
        StackItem::Entry(i) => ids::vector_paint_row_id(i),
    }
}

/// ⭐ **Onde a linha `k` (de `n`, do topo para o chão) cai com o cursor na ranhura `t`** — logo acima
/// da que fica por baixo dela, ou, no fundo, logo abaixo da que fica por cima. `None` = não se move.
#[must_use]
pub fn landing(rows: &[StackItem], k: usize, t: usize) -> Option<(StackItem, Place)> {
    if t == k || k >= rows.len() || t >= rows.len() {
        return None;
    }
    let outras: Vec<StackItem> = rows
        .iter()
        .enumerate()
        .filter(|(j, _)| *j != k)
        .map(|(_, it)| *it)
        .collect();
    let lugar = match outras.get(t) {
        Some(&abaixo) => Place::Above(abaixo),
        None => Place::Below(*outras.get(t.checked_sub(1)?)?),
    };
    Some((rows[k], lugar))
}

impl BodyCtx<'_> {
    /// ⭐ **A secção FILL & STROKE**: o «+» (um menu Fill / Stroke) e as linhas, na ordem da pilha.
    pub(crate) fn fill_stroke_section(&mut self, snap: &VectorStyleSnapshot, y: f32) -> f32 {
        let a = state::current_appearance();
        let menu = ids::VECTOR_PAINT_ADD_MENU;
        // Há sujeito: a pilha publicada, ou a resposta «tem traço?» (só com UMA forma selecionada).
        let live = a.is_some() || state::stroke_present().is_some();
        let open = live && is_open(self.store, menu);
        let (y, collapsed) = self.section_header_with_action(
            ph2d_tool_vector::ids::VECTOR_SECTION_FILL,
            tr("panel.vector.section.fill_stroke"),
            y,
            (menu, IconId::Plus, open, live),
        );
        if open && let Some(chip) = self.hit_index.rect_for(menu) {
            PENDENTE.with(|p| p.borrow_mut().menu = Some(chip));
        }
        if collapsed {
            return y;
        }
        self.fill_stroke_rows(snap, y)
    }

    /// As linhas da secção, sem o cabeçalho: o cartão (G2) tem o dele, com o «+» lá dentro.
    pub(crate) fn fill_stroke_rows(&mut self, snap: &VectorStyleSnapshot, y: f32) -> f32 {
        let a = state::current_appearance();
        let live = a.is_some() || state::stroke_present().is_some();
        let mut y = y;
        // F16: com o painel do traço aberto por baixo da barra ②, as linhas de traço moram LÁ neste
        // quadro — um id pintado em dois sítios faria o arrasto seguir o rect do outro.
        let na_barra = crate::bars::stroke_popover_open(self.store);
        let differ = a.as_ref().is_some_and(|a| a.layers_differ);
        let pilha = a.as_ref().filter(|a| !a.layers_differ).map_or_else(
            || vec![StackItem::BaseFill, StackItem::BaseStroke],
            state::Appearance::stack,
        );
        let rows: Vec<StackItem> = pilha
            .into_iter()
            .rev()
            .filter(|it| match *it {
                StackItem::BaseFill => state::base_fill() != Some(state::BasePaint::Absent),
                StackItem::BaseStroke => !na_barra && state::stroke_present() != Some(false),
                StackItem::Entry(i) => {
                    !na_barra
                        || a.as_ref()
                            .and_then(|a| a.layers.get(i))
                            .is_some_and(|r| r.is_fill)
                }
            })
            .collect();
        if differ {
            y = self.dica_da_lista(tr("panel.vector.appearance.mixed_layers"), y);
        }
        y = self.paint_rows(snap, &rows, live && !differ, y);
        if na_barra {
            y = self.dica_da_lista(tr("panel.vector.stroke.in_bar"), y);
        }
        y
    }

    /// Uma frase da lista: a dica do cartão, ou a de antes.
    fn dica_da_lista(&mut self, texto: &str, y: f32) -> f32 {
        let Some(ink) = card_body() else {
            return self.hint_line(texto, y);
        };
        let fim = ph2d_editor_core::panel::cards::CardCtx::flutuante(
            &mut *self.scene,
            &mut *self.text_system,
            self.store,
            &mut *self.hit_index,
            ink,
            self.inner_x,
            self.inner_w,
        )
        .hint(texto, y);
        fim + self.row_gap
    }

    /// As linhas, com o arrasto activo pintado: as outras abrem o vão e a agarrada voa.
    fn paint_rows(
        &mut self,
        snap: &VectorStyleSnapshot,
        rows: &[StackItem],
        arrastavel: bool,
        y0: f32,
    ) -> f32 {
        // ⚠️ As linhas têm altura PRÓPRIA (uma fila, ou duas quando o Weight não cabe): a ranhura,
        // o vão e a linha que voa leem os topos medidos, nunca um passo fixo.
        let alturas: Vec<f32> = rows
            .iter()
            .map(|it| self.altura_do_item(snap, *it))
            .collect();
        let topos: Vec<f32> = alturas
            .iter()
            .scan(y0, |y, h| {
                let topo = *y;
                *y += h + self.row_gap;
                Some(topo)
            })
            .collect();
        let drag = self
            .store
            .panel_row_drag()
            .filter(|(f, d)| *f == PanelRowFamily::VectorPaint && d.active)
            .map(|(_, d)| d);
        let agarrada = drag.and_then(|d| rows.iter().position(|it| gear_of(*it) == d.dragged));
        let n = rows.len();
        let ranhura = |cy: f32| {
            topos
                .iter()
                .rposition(|&t| t <= cy)
                .unwrap_or(0)
                .min(n.saturating_sub(1))
        };
        let (vao, alvo) = match (drag, agarrada) {
            (Some(d), Some(k)) => {
                let t = ranhura(d.cursor_y);
                (Some((k, t)), landing(rows, k, t))
            }
            _ => (None, None),
        };
        LANDING.with(|c| c.set(alvo));
        if arrastavel {
            PENDENTE.with(|p| p.borrow_mut().gears = rows.iter().map(|it| gear_of(*it)).collect());
        }
        let mut outras = rows
            .iter()
            .enumerate()
            .filter(|(j, _)| vao.is_none_or(|(k, _)| *j != k))
            .map(|(_, it)| *it);
        let mut y = y0;
        for s in 0..n {
            if let Some((k, t)) = vao
                && s == t
            {
                self.drop_mark(y, alturas[k]);
                y += alturas[k] + self.row_gap;
                continue;
            }
            if let Some(it) = outras.next() {
                y = self.stack_row(snap, it, y);
            }
        }
        if let (Some(d), Some((k, _))) = (drag, vao) {
            self.fly_row(
                snap,
                rows[k],
                alturas[k],
                // O ponto onde a mão agarrou fica sob o cursor, nos DOIS eixos (o cartão do Kanban).
                (
                    self.inner_x + d.cursor_x - d.down_x,
                    d.cursor_y - (d.down_y - topos[k]),
                ),
            );
        }
        y
    }

    /// A altura da linha `it` — pelo MESMO Weight que ela pinta (`peso_base`/`peso_camada`).
    fn altura_do_item(&mut self, snap: &VectorStyleSnapshot, it: StackItem) -> f32 {
        let peso = match it {
            StackItem::BaseFill => None,
            StackItem::BaseStroke => Some(self.peso_base(snap)),
            StackItem::Entry(i) => state::current_appearance()
                .and_then(|a| a.layers.get(i).copied())
                .filter(|r| !r.is_fill)
                .map(|r| self.peso_camada(i, &r)),
        };
        self.linha_h(peso)
    }

    /// Uma linha da lista: a tinta de base ou a camada `i`.
    fn stack_row(&mut self, snap: &VectorStyleSnapshot, it: StackItem, y: f32) -> f32 {
        match it {
            StackItem::BaseFill => self.base_fill_line(snap, y),
            StackItem::BaseStroke => self.base_stroke_line(snap, y, false),
            StackItem::Entry(i) => {
                match state::current_appearance().and_then(|a| a.layers.get(i).copied()) {
                    Some(row) => self.layer_line(i, &row, y),
                    None => y,
                }
            }
        }
    }

    /// O vão onde a linha vai cair: a ranhura funda (o lugar do cartão no Kanban) e a linha de
    /// acento na borda de CIMA dela — fora da linha que voa por cima do vão, sempre à vista.
    fn drop_mark(&mut self, y: f32, lh: f32) {
        let raio = ph2d_editor_core::paint::frame_radius(self.theme, Radius::Sm.px());
        let ink = card_body();
        fill_rounded_rect(
            self.scene,
            Rect::new(self.inner_x, y, self.inner_w, lh),
            raio,
            ink.map_or_else(
                || resolve(ColorToken::Bg3, self.theme),
                |i| i.color(ColorToken::CardField),
            ),
        );
        let h = StrokeToken::Thick.px();
        fill_rounded_rect(
            self.scene,
            Rect::new(self.inner_x, y - (self.row_gap + h) * 0.5, self.inner_w, h),
            raio,
            ink.map_or_else(
                || resolve(ColorToken::Accent, self.theme),
                |i| i.color(ColorToken::KanbanBlueBold),
            ),
        );
    }

    /// Pinta a linha agarrada à parte (num cartão) para o passe diferido a pôr sob o cursor, por cima
    /// de tudo, inclinada e com a sombra de voo.
    fn fly_row(
        &mut self,
        snap: &VectorStyleSnapshot,
        it: StackItem,
        lh: f32,
        (left, top): (f32, f32),
    ) {
        let mut local = ph2d_vector::VectorScene::new();
        let mut cliques = ph2d_editor_core::interaction::HitIndex::default();
        let caixa = Rect::new(0.0, 0.0, self.inner_w, lh);
        fill_rounded_rect(
            &mut local,
            caixa,
            ph2d_editor_core::paint::frame_radius(self.theme, Radius::Sm.px()),
            card_body().map_or_else(
                || resolve(ColorToken::Bg2, self.theme),
                |i| i.color(ColorToken::KanbanCardRaised),
            ),
        );
        let mut b = BodyCtx {
            scene: &mut local,
            text_system: &mut *self.text_system,
            store: self.store,
            hit_index: &mut cliques,
            theme: self.theme,
            inner_x: 0.0,
            inner_w: self.inner_w,
            row_h: self.row_h,
            row_gap: self.row_gap,
            chip_w: self.chip_w,
            font: self.font,
            open_fold: None,
        };
        super::fill_stroke::VOANDO.with(|v| v.set(true));
        b.stack_row(snap, it, 0.0);
        super::fill_stroke::VOANDO.with(|v| v.set(false));
        let onde = Rect::new(left, top, self.inner_w, lh);
        PENDENTE.with(|p| p.borrow_mut().voo = Some((local, onde)));
    }
}

/// ⭐ **O passe diferido da lista**: regista os ⚙ arrastáveis deste quadro, pinta o menu do «+» e a
/// linha que voa — por cima de todas as secções.
pub(crate) fn deferred(ctx: &mut ph2d_editor_core::panel::PaintCtx, panel: Rect, theme: Theme) {
    let p = PENDENTE.with(|p| std::mem::take(&mut *p.borrow_mut()));
    ctx.host
        .store_mut()
        .set_panel_row_ids(PanelRowFamily::VectorPaint, p.gears.into_iter().collect());
    let menu = if crate::cards::cards_on() {
        ancora_no_cartao(ctx, panel, theme)
    } else {
        p.menu
    };
    if let Some(chip) = menu {
        paint_add_menu(ctx, chip, theme);
    }
    if let Some((local, r)) = p.voo {
        let radius = ph2d_editor_core::paint::frame_radius(theme, Radius::Sm.px());
        let scene = &mut *ctx.scene;
        ph2d_editor_core::paint_flight::flight_shadow(scene, theme, r, radius);
        ph2d_editor_core::paint_flight::fly(
            scene,
            &local,
            r,
            ph2d_editor_core::paint_flight::FLIGHT_TILT_DEG,
        );
    }
}

/// O «+» aberto no cabeçalho do cartão: a lista cresce para DENTRO, com a borda direita na dele, e leva
/// a tinta do cartão. Lido aqui, não no corpo: um cartão dobrado não corre o corpo, e o «+» vive.
fn ancora_no_cartao(
    ctx: &mut ph2d_editor_core::panel::PaintCtx,
    panel: Rect,
    theme: Theme,
) -> Option<Rect> {
    let menu = ids::VECTOR_PAINT_ADD_MENU;
    let live = state::current_appearance().is_some() || state::stroke_present().is_some();
    if !live || !is_open(ctx.host.store(), menu) {
        return None;
    }
    let chip = ctx.host.hit_index_mut().rect_for(menu)?;
    let tema = ctx
        .host
        .store()
        .section_theme(ph2d_tool_vector::ids::VECTOR_SECTION_FILL);
    let tinta = ph2d_editor_core::widget::card_ink::CardInk::of_card(tema, theme);
    let _ = ph2d_editor_core::widget::card_ink::troca_popover(Some((menu, tinta)));
    let w = panel.w * 0.5;
    Some(Rect::new(chip.x + chip.w - w, chip.y, w, chip.h))
}

/// As duas linhas do «+»: um preenchimento e um traço — a tinta de base, se a forma não a tem, senão
/// uma camada nova no topo (enquanto houver lugar).
fn add_options() -> Vec<ph2d_editor_core::widget::DropdownOption<usize>> {
    use ph2d_editor_core::widget::DropdownOption;
    let lugar = BodyCtx::room_for_layer();
    let fill = if state::base_fill() == Some(state::BasePaint::Absent) {
        Some(ids::VECTOR_FILL_PRESENT)
    } else {
        lugar.then_some(ids::VECTOR_PAINT_ADD_FILL)
    };
    let stroke = if state::stroke_present() == Some(false) {
        Some(ids::VECTOR_STROKE_PRESENT)
    } else {
        lugar.then_some(ids::VECTOR_PAINT_ADD_STROKE)
    };
    [
        (fill, "panel.vector.paint.fill"),
        (stroke, "panel.vector.paint.stroke"),
    ]
    .into_iter()
    .enumerate()
    .filter_map(|(k, (id, rotulo))| id.map(|id| DropdownOption::new(id, k, tr(rotulo))))
    .collect()
}

/// Os ids que as linhas do «+» podem ter — um clique num deles fecha o menu.
pub(crate) fn is_add_option(id: NodeId) -> bool {
    [
        ids::VECTOR_FILL_PRESENT,
        ids::VECTOR_PAINT_ADD_FILL,
        ids::VECTOR_STROKE_PRESENT,
        ids::VECTOR_PAINT_ADD_STROKE,
    ]
    .contains(&id)
}

fn paint_add_menu(ctx: &mut ph2d_editor_core::panel::PaintCtx, chip: Rect, theme: Theme) {
    let dd = ph2d_editor_core::widget::Dropdown::new(ids::VECTOR_PAINT_ADD_MENU, "", add_options())
        .open(true);
    let region = ctx.layout.popover_region();
    ph2d_editor_core::panel::paint_dropdown_list(ctx, &dd, chip, region, theme, None);
}

#[cfg(test)]
#[path = "paint_fill_stroke_list_tests.rs"]
mod tests;
