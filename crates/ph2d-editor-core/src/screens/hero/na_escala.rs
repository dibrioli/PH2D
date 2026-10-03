//! ⭐⭐ **O ecrã hero NA ESCALA DA INTERFACE** (2026-10-02) — a porta que a shell chama em vez do
//! [`super::paint_hero_screen`] quando a janela é física. Spec:
//! `docs/UI_New_and_Simple/spec/05_a_escala_da_interface.md`.
//!
//! O chrome pinta-se numa cena LÓGICA (viewport = janela / `s`) e entra na cena do quadro sob
//! `Affine::scale(s)`, no MESMO ponto da ordem de pintura. ⚠️ A `100 %` é o caminho de sempre, byte
//! a byte: nenhuma cena intermédia, nenhum `append`.

use super::*;
use crate::ui_scale::UiScaleMap;

/// Pinta o hero na escala [`HeroScreen::escala`] (ecrã × `ui_scale`) para uma janela FÍSICA `viewport`; `depois` pinta, na
/// MESMA cena lógica e com o viewport lógico, o que é chrome e a shell desenha a seguir (a forma de
/// onda do Audio Editor).
pub fn paint_hero_screen_na_escala(
    hero: &mut HeroScreen,
    viewport: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    depois: impl FnOnce(&mut HeroScreen, Rect, &mut VectorScene, &mut TextSystem),
) {
    let mapa = hero.escala();
    let fisico = (!mapa.is_identity()).then(|| Fisico::para_o_logico(hero, mapa));
    crate::ui_scale::pintar_no_chrome(mapa, viewport, scene, |vp, chrome| {
        paint_hero_screen(hero, vp, chrome, text_system);
        depois(hero, vp, chrome, text_system);
    });
    if let Some(fisico) = fisico {
        fisico.de_volta(hero);
    }
}

/// ⭐⭐ **As portas FÍSICAS do chrome** — o que a shell usa para perguntar ao hero com um ponto da
/// JANELA. ⛔ A shell não chama `hit_index.hit` / `store.panel_at` / `handle_pointer*` /
/// `handle_wheel` com coordenadas cruas: o gate `a_shell_pergunta_ao_chrome_pelas_portas_fisicas`
/// recusa-o (um ponto físico num índice lógico acerta no vizinho a qualquer escala ≠ 100 %).
impl HeroScreen {
    /// O mapa da escala deste hero: o factor do ecrã × a preferência.
    #[must_use]
    pub fn escala(&self) -> UiScaleMap {
        UiScaleMap::no_ecra(self.ui_scale, self.escala_do_ecra)
    }

    /// O alvo do chrome sob um ponto FÍSICO da janela.
    #[must_use]
    pub fn chrome_hit(&self, x: f32, y: f32) -> Option<NodeId> {
        let (x, y) = self.escala().to_logical((x, y));
        self.hit_index.hit(x, y)
    }

    /// O painel sob um ponto FÍSICO da janela.
    #[must_use]
    pub fn chrome_panel_at(&self, x: f32, y: f32) -> Option<NodeId> {
        let (x, y) = self.escala().to_logical((x, y));
        self.store.panel_at(x, y)
    }

    /// O rect de um painel na JANELA (físico) — para o comparar com o ponteiro, ou para o usar
    /// num pass que pinta em píxeis da janela.
    #[must_use]
    pub fn panel_rect_fisico(&self, id: NodeId) -> Option<Rect> {
        self.store
            .panel_rect(id)
            .map(|r| self.escala().rect_to_physical(r))
    }

    /// [`Self::handle_pointer_with_text`] para um evento FÍSICO.
    pub fn handle_pointer_fisico<'frame>(
        &mut self,
        mut event: ph2d_host::PointerEvent,
        text_system: &mut TextSystem,
        arena: &'frame bumpalo::Bump,
    ) -> &'frame [crate::interaction::WidgetEvent] {
        (event.x, event.y) = self.escala().to_logical((event.x, event.y));
        self.handle_pointer_with_text(event, text_system, arena)
    }

    /// [`Self::handle_wheel`] para um evento FÍSICO — a posição E o passo (um passo de `n`
    /// píxeis físicos rola `n` píxeis no ecrã, a qualquer escala).
    pub fn handle_wheel_fisico<'frame>(
        &mut self,
        mut event: ph2d_host::WheelEvent,
        arena: &'frame bumpalo::Bump,
    ) -> &'frame [crate::interaction::WidgetEvent] {
        let m = self.escala();
        (event.x, event.y) = m.to_logical((event.x, event.y));
        (event.delta_x, event.delta_y) = m.to_logical((event.delta_x, event.delta_y));
        self.handle_wheel(event, arena)
    }
}

/// ⭐⭐ **O que a shell escreve no hero em pixels FÍSICOS** — as vistas do mundo (janela, canvas,
/// cursor) e o arrasto. O hero pinta e regista os alvos delas no espaço LÓGICO, então durante a
/// pintura ele recebe cópias divididas por `s`; a shell continua a fazer as CONTAS (o arrasto, a
/// projecção do ponteiro) nas físicas, que voltam no fim. ⚠️ A projecção do mundo é LINEAR na
/// janela (`px/m = janela_h / altura_mundo`), logo o lógico × `s` cai no píxel físico da arte.
///
/// ⛔ Um campo de ecrã novo nestas vistas tem de entrar aqui — o gate
/// `as_vistas_do_mundo_caem_no_mesmo_pixel_em_toda_escala` projecta cada uma nas duas escalas.
struct Fisico {
    grid: Option<crate::grid::GridView>,
    gizmo: [Option<crate::gizmo::GizmoView>; 6],
    extras: Vec<(u64, crate::gizmo::GizmoView)>,
    point: Option<crate::gizmo::PointGizmoView>,
    drag: Option<crate::gizmo::GizmoDragState>,
    ficheiros: Option<(f32, f32)>,
}

impl Fisico {
    fn para_o_logico(hero: &mut HeroScreen, m: UiScaleMap) -> Self {
        let g = &mut hero.gizmo;
        let fisico = Self {
            grid: hero.grid.view,
            gizmo: [
                g.view,
                g.global_view,
                g.pose_view,
                g.selection_view,
                g.field_view,
                g.global_view_start,
            ],
            extras: g.extra_views.clone(),
            point: g.point_view.clone(),
            drag: g.drag,
            ficheiros: hero.dragging_files.as_ref().map(|(_, p)| *p),
        };
        hero.grid.view = hero.grid.view.map(|v| grid_logico(v, m));
        for v in [
            &mut g.view,
            &mut g.global_view,
            &mut g.pose_view,
            &mut g.selection_view,
            &mut g.field_view,
            &mut g.global_view_start,
        ] {
            *v = v.map(|v| gizmo_logico(v, m));
        }
        for (_, v) in &mut g.extra_views {
            *v = gizmo_logico(*v, m);
        }
        if let Some(p) = g.point_view.as_mut() {
            p.window_w /= m.factor();
            p.window_h /= m.factor();
            p.canvas = m.rect_to_logical(p.canvas);
        }
        if let Some(d) = g.drag.as_mut() {
            d.start_screen = m.to_logical(d.start_screen);
            d.cursor_screen = m.to_logical(d.cursor_screen);
        }
        if let Some((_, p)) = hero.dragging_files.as_mut() {
            *p = m.to_logical(*p);
        }
        fisico
    }

    fn de_volta(self, hero: &mut HeroScreen) {
        let g = &mut hero.gizmo;
        hero.grid.view = self.grid;
        [
            g.view,
            g.global_view,
            g.pose_view,
            g.selection_view,
            g.field_view,
            g.global_view_start,
        ] = self.gizmo;
        g.extra_views = self.extras;
        g.point_view = self.point;
        g.drag = self.drag;
        if let (Some((_, p)), Some(f)) = (hero.dragging_files.as_mut(), self.ficheiros) {
            *p = f;
        }
    }
}

fn grid_logico(v: crate::grid::GridView, m: UiScaleMap) -> crate::grid::GridView {
    crate::grid::GridView {
        window_w: v.window_w / m.factor(),
        window_h: v.window_h / m.factor(),
        canvas: m.rect_to_logical(v.canvas),
        ..v
    }
}

fn gizmo_logico(v: crate::gizmo::GizmoView, m: UiScaleMap) -> crate::gizmo::GizmoView {
    crate::gizmo::GizmoView {
        window_w: v.window_w / m.factor(),
        window_h: v.window_h / m.factor(),
        canvas: m.rect_to_logical(v.canvas),
        cursor_screen: v.cursor_screen.map(|p| m.to_logical(p)),
        ..v
    }
}

#[cfg(test)]
#[path = "na_escala_tests.rs"]
mod tests;
