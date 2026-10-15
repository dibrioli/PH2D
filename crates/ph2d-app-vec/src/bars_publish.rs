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
        if hero.store.tool_bars().is_some() {
            hero.store.set_tool_bars(None);
            hero.store.set_area_commands(Vec::new(), Vec::new());
        }
        return;
    }
    let selection = hero
        .grid
        .view
        .and_then(|v| selection_on_screen(scene, pen, xforms, &v));
    let readout = readout(hero, snap_on, px_to_world);
    let bars = ph2d_panel_vector::bars::tool_bars(&hero.store, selection, readout);
    hero.store.set_tool_bars(Some(bars));
    // F16: o vetor já não publica pulldowns de área — a ② é a barra de controle dele (o encaixe
    // voltou ao painel; os verbos do caminho estão na ④).
    hero.store.set_area_commands(Vec::new(), Vec::new());
}
