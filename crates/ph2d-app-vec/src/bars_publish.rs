//! ⭐ **As barras do vetor, publicadas em todo quadro** (D10 — `docs/UI_New_and_Simple/vetor_nova_ui/`).
//!
//! A TABELA (o que cada barra mostra em cada modo) é do painel ([`ph2d_panel_vector::bars`]); aqui
//! só se junta o que só este passo sabe: onde a seleção cai no ECRÃ (a âncora da ④) e as leituras
//! do rodapé ⑤. Corre no passo 5 do `vector_bridge::dispatch` («publicar»), depois de o painel ter
//! recebido o retrato deste quadro.
//!
//! ⚠️ **Em todo quadro** com a ferramenta na mão; e no quadro em que ela larga a mão, LIMPA o que
//! escreveu — uma coluna que ficasse pintada (e clicável) por cima de outro módulo seria um botão a
//! despachar para uma ferramenta que já não está na mão.

use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::zones::Rect;
use ph2d_vec_edit::PenTool;
use ph2d_vec_scene::{VecScene, VecXforms};

/// A seleção (união das caixas dos caminhos seleccionados) no ecrã, pela projeção das réguas.
#[must_use]
pub fn selection_on_screen(
    scene: &VecScene,
    pen: &PenTool,
    xforms: &VecXforms,
    view: &ph2d_editor_core::GridView,
) -> Option<Rect> {
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    let mut any = false;
    for &id in pen.selected_paths() {
        // A MESMA caixa que o gizmo desenha (o traço incluído — F11): uma pergunta, uma porta.
        if let Some((a, b)) = ph2d_vec_render::path_world_visual_bbox(scene, xforms, id) {
            any = true;
            lo = [lo[0].min(a[0]), lo[1].min(a[1])];
            hi = [hi[0].max(b[0]), hi[1].max(b[1])];
        }
    }
    if !any {
        return None;
    }
    // y do mundo para cima ⇒ o canto de CIMA no ecrã é o `hi` do mundo.
    let top_left = view.world_to_screen([lo[0], hi[1]]);
    let bottom_right = view.world_to_screen([hi[0], lo[1]]);
    Some(Rect::new(
        top_left[0],
        top_left[1],
        (bottom_right[0] - top_left[0]).max(0.0),
        (bottom_right[1] - top_left[1]).max(0.0),
    ))
}

/// ⭐ (dono 02/11, F50) **A âncora da ④ não pode ficar por cima do GIZMO**: a união da seleção com as
/// caixas que o gizmo desenha (a da forma, as extra e a GLOBAL da seleção múltipla, que se afasta
/// 32 px das formas) — sem isto a ④ tapava-lhe a borda e as alças.
#[must_use]
pub fn anchor_clear_of_gizmo(sel: Rect, gizmo: Option<Rect>) -> Rect {
    let Some(g) = gizmo else {
        return sel;
    };
    let (x0, y0) = (sel.x.min(g.x), sel.y.min(g.y));
    let (x1, y1) = (
        (sel.x + sel.w).max(g.x + g.w),
        (sel.y + sel.h).max(g.y + g.h),
    );
    Rect::new(x0, y0, x1 - x0, y1 - y0)
}

/// ⑤ as leituras: onde o ponteiro está (na unidade do projeto), o encaixe e o zoom.
fn readout(hero: &HeroScreen, snap_on: bool, px_to_world: f64) -> Vec<String> {
    let mut out = Vec::new();
    let display = ph2d_editor_core::LengthDisplay::of(&hero.project);
    if let (Some(view), Some(p)) = (hero.grid.view, hero.store.last_pointer())
        && view.canvas.contains(p[0], p[1])
    {
        let w = view.screen_to_world(p);
        let f = |v: f64| format!("{:.0} {}", display.value(v), display.suffix());
        out.push(ph2d_i18n::tr_with(
            "panel.vector.bar.readout.xy",
            &[("x", &f(w[0])), ("y", &f(w[1]))],
        ));
    }
    out.push(
        ph2d_i18n::tr(if snap_on {
            "panel.vector.bar.readout.snap_on"
        } else {
            "panel.vector.bar.readout.snap_off"
        })
        .to_owned(),
    );
    let ppm = f64::from(hero.project.pixels_per_meter);
    if px_to_world > 0.0 && ppm > 0.0 {
        let zoom = 100.0 / (px_to_world * ppm);
        out.push(ph2d_i18n::tr_with(
            "panel.vector.bar.readout.zoom",
            &[("z", &format!("{zoom:.0}"))],
        ));
    }
    out
}

/// ⭐ Publica (ou limpa) as barras deste quadro.
pub fn publish(
    hero: &mut HeroScreen,
    scene: &VecScene,
    pen: &PenTool,
    xforms: &VecXforms,
    vector_active: bool,
    snap_on: bool,
    px_to_world: f64,
) {
    if !vector_active {
        // Só as SUAS: a coluna do pintor é do chrome e ele a limpa (`painter_column::publish`); a do
        // Pixel, a família dela.
        if hero
            .store
            .tool_bars()
            .is_some_and(|b| !b.is_family_column())
        {
            hero.store.set_tool_bars(None);
            hero.store.set_area_commands(Vec::new(), Vec::new());
        }
        return;
    }
    // As caixas do gizmo só contam se ele vai ser PINTADO: o mesmo predicado com que a
    // `fase_gizmo_suppression` da shell as zera depois (as tools de vértice `vector_*` não
    // coexistem com `vector_active`, e Deform/Pairs não são do vetor).
    let gizmo_painted = ph2d_editor_core::screens::hero::mode_drive::object_gizmo_shows(hero);
    let selection = hero
        .grid
        .view
        .and_then(|v| selection_on_screen(scene, pen, xforms, &v))
        .map(|sel| {
            let g = &hero.gizmo;
            g.view
                .iter()
                .chain(g.global_view.iter())
                .chain(g.extra_views.iter().map(|(_, v)| v))
                .filter(|_| gizmo_painted)
                .map(ph2d_editor_core::gizmo::screen_aabb)
                .fold(sel, |a, r| anchor_clear_of_gizmo(a, Some(r)))
        });
    let readout = readout(hero, snap_on, px_to_world);
    let bars = ph2d_panel_vector::bars::tool_bars(&hero.store, selection, readout);
    hero.store.set_tool_bars(Some(bars));
    // F16: o vetor já não publica pulldowns de área — a ② é a barra de controle dele (o encaixe
    // voltou ao painel; os verbos do caminho estão na ④).
    hero.store.set_area_commands(Vec::new(), Vec::new());
}

#[cfg(test)]
mod tests {
    use ph2d_editor_core::interaction::ToolBars;
    use ph2d_editor_core::screens::hero::HeroScreen;
    use ph2d_vec_scene::{VecScene, VecXforms};

    /// A ④ ancora-se na UNIÃO da seleção com a caixa do gizmo; sem gizmo, na seleção.
    #[test]
    fn the_anchor_clears_the_gizmo_box() {
        use ph2d_editor_core::zones::Rect;
        let sel = Rect::new(391.0, 183.0, 203.0, 170.0);
        let gizmo = Rect::new(359.0, 163.0, 266.0, 218.0);
        let a = super::anchor_clear_of_gizmo(sel, Some(gizmo));
        assert_eq!((a.x, a.y, a.w, a.h), (359.0, 163.0, 266.0, 218.0));
        let a = super::anchor_clear_of_gizmo(sel, None);
        assert_eq!((a.x, a.y, a.w, a.h), (sel.x, sel.y, sel.w, sel.h));
        // Uma caixa MENOR que a seleção não encolhe a âncora.
        let dentro = Rect::new(400.0, 200.0, 10.0, 10.0);
        let a = super::anchor_clear_of_gizmo(sel, Some(dentro));
        assert_eq!((a.x, a.y, a.w, a.h), (sel.x, sel.y, sel.w, sel.h));
    }

    /// Num modo sem gizmo do objecto (Draw) a caixa do gizmo não é pintada, logo a ④ não se
    /// desvia dela: a âncora é só a seleção.
    #[test]
    fn the_anchor_ignores_a_gizmo_that_will_not_be_painted() {
        use ph2d_editor_core::zones::Rect;
        let canvas = Rect::new(0.0, 0.0, 1000.0, 1000.0);
        let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
        hero.grid.view = Some(ph2d_editor_core::GridView {
            camera_center: [0.0, 0.0],
            camera_height_world: 10.0,
            window_w: 1000.0,
            window_h: 1000.0,
            canvas,
        });
        hero.gizmo.view = Some(ph2d_editor_core::gizmo::GizmoView {
            bbox_min_world: [-3.0, -3.0],
            bbox_max_world: [3.0, 3.0],
            rotation: 0.0,
            camera_center: [0.0, 0.0],
            camera_height_world: 10.0,
            window_w: 1000.0,
            window_h: 1000.0,
            canvas,
            cursor_screen: None,
            pivot_world: [0.0, 0.0],
            pivot_tool_active: false,
            model: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        });
        let mut scene = VecScene::default();
        let id = scene.push_path(ph2d_vec_scene::rectangle([0.0, 0.0], [1.0, 1.0]));
        let mut pen = ph2d_vec_edit::PenTool::new();
        pen.select_many(&[id]);
        let xf = VecXforms::default();
        let anchor = |hero: &mut HeroScreen| {
            super::publish(hero, &scene, &pen, &xf, true, false, 1.0);
            hero.store
                .tool_bars()
                .and_then(|b| b.selection)
                .expect("âncora")
        };
        let com = anchor(&mut hero);
        hero.gizmo
            .mode
            .enter(7, ph2d_editor_core::object_mode::ObjectMode::Draw);
        let sem = anchor(&mut hero);
        assert!(
            com.w > sem.w + 1.0,
            "controlo: em Object a âncora abraça o gizmo"
        );
        let sel = super::selection_on_screen(&scene, &pen, &xf, &hero.grid.view.unwrap()).unwrap();
        assert_eq!((sem.x, sem.y, sem.w, sem.h), (sel.x, sel.y, sel.w, sel.h));
    }

    /// O vetor inactivo limpa as SUAS barras e deixa a coluna do pintor (que o chrome publica) em paz.
    #[test]
    fn the_idle_vector_clears_its_bars_and_never_the_painters_column() {
        let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
        let (scene, pen, xf) = (
            VecScene::default(),
            ph2d_vec_edit::PenTool::new(),
            VecXforms::default(),
        );
        hero.store.set_tool_bars(Some(ToolBars {
            painter_rail: true,
            ..ToolBars::default()
        }));
        super::publish(&mut hero, &scene, &pen, &xf, false, false, 1.0);
        assert!(
            hero.store.tool_bars().is_some(),
            "o vetor inactivo apagou a coluna do pintor"
        );
        // A coluna do Pixel Sprite ▸ Draw (ADR-0196) também é de família, não do vetor.
        hero.store.set_tool_bars(Some(ToolBars {
            pixel_rail: true,
            ..ToolBars::default()
        }));
        super::publish(&mut hero, &scene, &pen, &xf, false, false, 1.0);
        assert!(
            hero.store.tool_bars().is_some_and(|b| b.pixel_rail),
            "o vetor inactivo apagou a coluna do Pixel"
        );
        hero.store.set_tool_bars(Some(ToolBars::default()));
        super::publish(&mut hero, &scene, &pen, &xf, false, false, 1.0);
        assert!(
            hero.store.tool_bars().is_none(),
            "controlo: o vetor inactivo deixou as suas barras"
        );
    }
}
