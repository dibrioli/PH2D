//! Sonda (não afirma nada): onde acaba o corpo do Inspector e da Hierarquia contra o fundo da
//! janela, e o que a porta de rolagem publicou para eles. `--run-ignored all` para a correr.

use ph2d_editor_core::ids;
use ph2d_editor_core::screens::hero::{HeroScreen, paint_hero_screen};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;

#[test]
#[ignore = "diagnóstico: imprime, não afirma"]
fn diag_onde_acaba_o_corpo_das_colunas() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    for (w, h) in [(1930.0_f32, 1012.0_f32), (1366.0, 768.0)] {
        let vp = Rect::new(0.0, 0.0, w, h);
        let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
        let mut scene = ph2d_vector::VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        for _ in 0..3 {
            paint_hero_screen(&mut hero, vp, &mut scene, &mut text);
        }
        println!("== janela {w}x{h}");
        for (nome, id) in [
            ("inspector", ids::INSP_PANEL),
            ("hierarchy", ids::HIER_PANEL),
        ] {
            let r = hero.store.panel_rect(id);
            let c = hero.store.panel_content_h(id);
            let v = hero.store.panel_visible_h(id);
            println!("  {nome}: rect={r:?} content_h={c:?} visible_h={v:?}");
            if let Some(r) = r {
                println!("    fundo do rect = {:.1} (janela {h})", r.y + r.h);
            }
        }
        let mut outros: Vec<Rect> = hero.store.panel_rects().collect();
        outros.sort_by(|a, b| (a.y + a.h).total_cmp(&(b.y + b.h)));
        for r in outros {
            println!(
                "  rect publicado: x={:.0} y={:.0} w={:.0} h={:.0} fundo={:.0}",
                r.x,
                r.y,
                r.w,
                r.h,
                r.y + r.h
            );
        }
    }
}
