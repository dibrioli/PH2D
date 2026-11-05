//! A foto do Inspector armado, todos os cartões abertos, em fatias: `PH2D_FOTO_DIR=<pasta>`, `--ignored`,
//! por nome `q_foto`. Serve para OLHAR cartão a cartão (`docs/CartoesUI/18_REVISAO_DE_CAPRICHO.md`).

use super::a_marca_tem_a_altura_da_linha::abre_as_gavetas;
use ph2d_editor_core::panel::{PaintCtx, PanelHostInternal};
use ph2d_editor_core::screens::layout::HeroLayout;
use ph2d_editor_core::screens::slot::SlotSet;
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_ui_testkit::MockPanelHost;
use ph2d_vector::VectorScene;

/// A janela alta: a lista abraça o conteúdo e nenhum cartão fica abaixo da dobra.
const JANELA: Rect = Rect::new(0.0, 0.0, 1600.0, 30000.0);
/// A altura de cada fatia: duas vezes isto, depois de ampliada, ainda se lê numa foto só.
const FATIA: u32 = 900;
const AMPLIA: u32 = 2;

#[test]
#[ignore = "fotos: PH2D_FOTO_DIR=<pasta> e --ignored"]
fn q_foto() {
    let Ok(dir) = std::env::var("PH2D_FOTO_DIR") else {
        return;
    };
    let _ = ph2d_panel_registry_init::register_all_panels();
    ph2d_panel_inspector::set_cards_on(true);
    let gpu = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None)
        .expect("a foto precisa da placa");
    let layout = HeroLayout::for_viewport(JANELA);
    let mut text = TextSystem::new();
    let mut host = MockPanelHost::new().in_theme(ph2d_tokens::Theme::Dark);
    let mut cena = VectorScene::new();
    let mut coluna = Rect::new(0.0, 0.0, 0.0, 0.0);
    ph2d_editor_core::panel::with_registry(|reg| {
        let painel = reg
            .panels_mut()
            .iter_mut()
            .find(|p| p.manifest.id == "inspector")
            .expect("o inspector");
        super::o_inspector_armado::arma_tudo();
        painel.populate(host.store_mut());
        host.set_panel_visible("inspector", true);
        let slot_id = painel.manifest.default_slot;
        coluna = layout.slot_rects(SlotSet::of(slot_id)).get(slot_id);
        for _ in 0..4 {
            abre_as_gavetas(host.store_mut());
            cena = VectorScene::new();
            host.hit_index_mut().clear_for_frame();
            let mut ctx = PaintCtx {
                host: &mut host,
                layout: &layout,
                slot: coluna,
                viewport: JANELA,
                scene: &mut cena,
                text_system: &mut text,
            };
            ph2d_editor_core::aquecimento::com_passo_fixo(0.1, || painel.paint(&mut ctx));
        }
        super::o_inspector_armado::desarma_tudo();
    });
    let (w, h) = (coluna.w as u32, FATIA);
    let (x0, mut y0, mut n) = (coluna.x, 0_u32, 0_u32);
    let mut pass =
        ph2d_render::VelloPass::new(&gpu, ph2d_gpu::TextureFormat::Rgba8Unorm, (w, h)).unwrap();
    while y0 < JANELA.h as u32 {
        let mut fatia = VectorScene::new();
        fatia.append_transformed(
            &cena,
            ph2d_vector::Affine::translate((-f64::from(x0), -f64::from(y0))),
        );
        let px = pass
            .render_and_readback(&gpu, fatia.inner(), (w, h))
            .unwrap();
        if px.chunks(4).any(|p| p[3] != 0) {
            let img = image::RgbaImage::from_raw(w, h, px).unwrap();
            let img = image::imageops::resize(
                &img,
                w * AMPLIA,
                h * AMPLIA,
                image::imageops::FilterType::Nearest,
            );
            img.save(format!("{dir}/f{n:02}.png")).unwrap();
            n += 1;
        } else {
            break;
        }
        y0 += FATIA;
    }
    println!("{n} fatias, coluna {w} px");
}
