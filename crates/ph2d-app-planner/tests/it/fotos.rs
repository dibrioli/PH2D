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
