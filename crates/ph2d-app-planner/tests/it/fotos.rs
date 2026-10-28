//! 📷 As fotos da aba sobre a cena do smoke, claro e escuro (`#[ignore]`: a placa).
//! `bash scripts/cargo-test-narrow.sh ph2d-app-planner foto_ -- --ignored`

use ph2d_app_planner::{Hit, Mode, TAB};
use ph2d_editor_core::doc_surface::DocumentSurface;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::Theme;

use crate::Mesa;

const W: u32 = 1440;
const H: u32 = 900;

fn shot(
    gpu: &ph2d_gpu::GpuContext,
    m: &mut Mesa,
    ts: &mut ph2d_text::TextSystem,
    name: &str,
    theme: Theme,
) {
    let size = (W, H);
    #[allow(clippy::cast_precision_loss)]
    let rect = Rect::new(0.0, 0.0, W as f32, H as f32);
    let mut pass =
        ph2d_render::VelloPass::new(gpu, ph2d_gpu::TextureFormat::Rgba8Unorm, size).expect("pass");
    let mut scene = ph2d_vector::VectorScene::new();
    m.docs().paint(TAB, rect, &mut scene, ts, &theme);
    let mut scene = ph2d_vector::VectorScene::new();
    m.docs().paint(TAB, rect, &mut scene, ts, &theme);
    let px = pass
        .render_and_readback(gpu, scene.inner(), size)
        .expect("readback");
    let dir =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/TrelloClone/smoke/W9");
    std::fs::create_dir_all(&dir).expect("pasta");
    let path = dir.join(format!("{name}.png"));
    image::save_buffer(&path, &px, W, H, image::ExtendedColorType::Rgba8).expect("png");
    println!("foto: {}", path.display());
}

#[test]
#[ignore = "a placa"]
fn foto_planner_os_modos_nos_dois_temas() {
    let gpu =
        ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None).expect("gpu");
    let mut ts = ph2d_text::TextSystem::new();
    for (theme, tag) in [(Theme::Light, "claro"), (Theme::Dark, "escuro")] {
        let (mut m, _) = Mesa::cena();
        m.hero.theme = theme;
        shot(&gpu, &mut m, &mut ts, &format!("calendario_{tag}"), theme);
        m.clica_em(Hit::NewEvent);
        shot(&gpu, &mut m, &mut ts, &format!("evento_{tag}"), theme);
        m.docs().close_pop_for_photo();
        m.clica_em(Hit::Bell);
        shot(&gpu, &mut m, &mut ts, &format!("sino_{tag}"), theme);
        m.clica_em(Hit::Bell);
        m.clica_em(Hit::Mode(Mode::Planner));
        shot(&gpu, &mut m, &mut ts, &format!("planner_{tag}"), theme);
        m.clica_em(Hit::Mode(Mode::MyCards));
        shot(&gpu, &mut m, &mut ts, &format!("my_cards_{tag}"), theme);
    }
}

#[test]
#[ignore = "a placa"]
fn foto_destino_e_alcance_nos_dois_temas() {
    let gpu =
        ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None).expect("gpu");
    let mut ts = ph2d_text::TextSystem::new();
    for (theme, tag) in [(Theme::Light, "claro"), (Theme::Dark, "escuro")] {
        let (mut m, _) = Mesa::cena();
        m.hero.theme = theme;
        m.clica_em(Hit::Mode(Mode::Planner));
        let id = m
            .set()
            .agenda()
            .inbox
            .values()
            .find(|i| i.text == "Call the sound studio")
            .map(|i| i.id)
            .expect("item");
        m.clica_em(Hit::InboxToCard(id));
        shot(
            &gpu,
            &mut m,
            &mut ts,
            &format!("move_to_kanban_{tag}"),
            theme,
        );
        m.escreve("record");
        shot(
            &gpu,
            &mut m,
            &mut ts,
            &format!("move_to_kanban_pesquisa_{tag}"),
            theme,
        );
        m.docs().close_pop_for_photo();
        m.clica_em(Hit::Mode(Mode::Calendar));
        let e = m
            .set()
            .agenda()
            .events
            .values()
            .find(|e| e.repeat.is_some())
            .cloned()
            .expect("série");
        let r = *m
            .docs()
            .probe_calendar()
            .item_rects(ph2d_app_planner::source::event_key(e.id, e.start))
            .first()
            .expect("pintada");
        let z = ph2d_kanban_dates::Zone::utc();
        let alvo = ph2d_kanban_dates::day_of(e.start.0, &z).add_days(1);
        let para = crate::centro(m.docs().probe_calendar().cell_rect(alvo).expect("dia"));
        m.arrasta(crate::centro(r), para);
        shot(&gpu, &mut m, &mut ts, &format!("alcance_{tag}"), theme);
    }
}

/// 📷 Tutorial 9.8: o menu `File` aberto sobre o hero inteiro, com os quatro comandos do Planner.
#[test]
#[ignore = "a placa"]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn foto_menu_file_com_os_modos_nos_dois_temas() {
    use ph2d_editor_core::interaction::{ContextMenuKind, ContextMenuRequest};
    use ph2d_editor_core::screens::hero::{HERO_VIEWPORT_H, HERO_VIEWPORT_W, paint_hero_screen};
    let gpu =
        ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None).expect("gpu");
    let size = (HERO_VIEWPORT_W as u32, HERO_VIEWPORT_H as u32);
    for (theme, tag) in [(Theme::Light, "claro"), (Theme::Dark, "escuro")] {
        ph2d_editor_core::test_support::ensure_panel_registry();
        let (mut m, _) = Mesa::cena();
        m.hero.theme = theme;
        m.ts = ph2d_text::TextSystem::new();
        m.quadros(2);
        let vp = Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H);
        let mut ts = std::mem::replace(&mut m.ts, ph2d_text::TextSystem::without_system_fonts());
        paint_hero_screen(
            &mut m.hero,
            vp,
            &mut ph2d_vector::VectorScene::new(),
            &mut ts,
        );
        // Como o clique no título (`pointer_down_menus`): o menu ancorado por baixo de «File».
        let file = m
            .hero
            .hit_index
            .rect_for(ph2d_editor_core::ids::MENUBAR_FILE)
            .expect("o título File pintado");
        m.hero.store.open_context_menu(ContextMenuRequest {
            x: file.x,
            y: file.y + file.h,
            kind: ContextMenuKind::MenuBarFile,
        });
        assert_eq!(
            m.hero.store.context_menu().map(|r| r.kind),
            Some(ContextMenuKind::MenuBarFile),
            "o menu File não abriu"
        );
        let mut scene = ph2d_vector::VectorScene::new();
        paint_hero_screen(&mut m.hero, vp, &mut scene, &mut ts);
        let mut scene = ph2d_vector::VectorScene::new();
        paint_hero_screen(&mut m.hero, vp, &mut scene, &mut ts);
        let mut pass = ph2d_render::VelloPass::new(&gpu, ph2d_gpu::TextureFormat::Rgba8Unorm, size)
            .expect("pass");
        let px = pass
            .render_and_readback(&gpu, scene.inner(), size)
            .expect("readback");
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/TrelloClone/smoke/tutorial");
        std::fs::create_dir_all(&dir).expect("pasta");
        let path = dir.join(format!("9_8_{tag}.png"));
        image::save_buffer(&path, &px, size.0, size.1, image::ExtendedColorType::Rgba8)
            .expect("png");
        println!("foto: {}", path.display());
    }
}
