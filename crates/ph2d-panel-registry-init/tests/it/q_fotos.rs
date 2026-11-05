//! A foto do Inspector armado, todos os cartões abertos, em fatias: `PH2D_FOTO_DIR=<pasta>`, `--ignored`,
//! por nome `q_foto`. Serve para OLHAR cartão a cartão (`docs/CartoesUI/18_REVISAO_DE_CAPRICHO.md`).

use super::lei_da_compactacao::abre_tudo;
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
            abre_tudo(host.store_mut());
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

/// Uma pintura de um painel no retrato alto: de fábrica, armado e em cada estado semeado, em fatias.
/// `PH2D_FOTO_DIR=<pasta> PH2D_FOTO_PAINEIS=vector,physics`, por nome `q_foto_paineis`.
#[test]
#[ignore = "fotos: PH2D_FOTO_DIR=<pasta>, PH2D_FOTO_PAINEIS=<ids> e --ignored"]
fn q_foto_paineis() {
    let Ok(dir) = std::env::var("PH2D_FOTO_DIR") else {
        return;
    };
    let quais = std::env::var("PH2D_FOTO_PAINEIS").unwrap_or_default();
    let _ = ph2d_panel_registry_init::register_all_panels();
    ph2d_panel_inspector::set_cards_on(true);
    let gpu = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None)
        .expect("a foto precisa da placa");
    let layout = HeroLayout::for_viewport(JANELA);
    let mut text = TextSystem::new();
    ph2d_editor_core::panel::with_registry(|reg| {
        for painel in reg.panels_mut() {
            let id = painel.manifest.id;
            if !quais.split(',').any(|q| q == id) {
                continue;
            }
            let slot_id = painel.manifest.default_slot;
            let coluna = layout.slot_rects(SlotSet::of(slot_id)).get(slot_id);
            let mut pinta = |nome: String,
                             painel: &mut ph2d_editor_core::panel::ErasedPanel,
                             host: &mut MockPanelHost| {
                painel.populate(host.store_mut());
                host.set_panel_visible(id, true);
                let mut cena = VectorScene::new();
                for _ in 0..4 {
                    abre_tudo(host.store_mut());
                    cena = VectorScene::new();
                    host.hit_index_mut().clear_for_frame();
                    let mut ctx = PaintCtx {
                        host: &mut *host,
                        layout: &layout,
                        slot: coluna,
                        viewport: JANELA,
                        scene: &mut cena,
                        text_system: &mut text,
                    };
                    ph2d_editor_core::aquecimento::com_passo_fixo(0.1, || painel.paint(&mut ctx));
                }
                fatias(&gpu, &cena, coluna, &format!("{dir}/{id}_{nome}"));
            };
            let novo = || MockPanelHost::new().in_theme(ph2d_tokens::Theme::Dark);
            pinta("fabrica".into(), painel, &mut novo());
            if let Some(arm) = super::paineis_armados::TABELA
                .iter()
                .find(|a| a.painel == id)
            {
                let mut host = novo();
                (arm.arma)(host.store_mut());
                pinta("armado".into(), painel, &mut host);
                (arm.desarma)();
            }
            let estados = super::censo_estados::ESTADOS
                .iter()
                .filter(|e| e.painel == id);
            for (n, e) in estados.enumerate() {
                (e.arma)();
                pinta(format!("e{n:02}"), painel, &mut novo());
                (e.desarma)();
            }
        }
    });
}

/// A cena toda, em fatias da janela inteira, recortadas ao que foi pintado: um painel flutuante não mora na coluna.
fn fatias(gpu: &ph2d_gpu::GpuContext, cena: &VectorScene, coluna: Rect, base: &str) {
    let (w, h) = (JANELA.w as u32, FATIA);
    let mut pass =
        ph2d_render::VelloPass::new(gpu, ph2d_gpu::TextureFormat::Rgba8Unorm, (w, h)).unwrap();
    let (mut y0, mut n) = (0_u32, 0_u32);
    while y0 < JANELA.h as u32 {
        let mut fatia = VectorScene::new();
        fatia.append_transformed(cena, ph2d_vector::Affine::translate((0.0, -f64::from(y0))));
        let px = pass
            .render_and_readback(gpu, fatia.inner(), (w, h))
            .unwrap();
        // Só o opaco: a sombra de uma janela flutuante não é conteúdo.
        let opaco = |p: &[u8]| p[3] > 200;
        let (mut x0, mut x1, mut y_min, mut y_max) = (w, 0, h, 0);
        for (i, p) in px.chunks(4).enumerate() {
            if opaco(p) {
                let (x, y) = (i as u32 % w, i as u32 / w);
                (x0, x1, y_min, y_max) = (x0.min(x), x1.max(x), y_min.min(y), y_max.max(y));
            }
        }
        y0 += FATIA;
        if x1 < x0 {
            if n == 0 {
                continue;
            }
            break;
        }
        let img = image::RgbaImage::from_raw(w, h, px).unwrap();
        let img = image::imageops::crop_imm(&img, x0, y_min, x1 - x0 + 1, y_max - y_min + 1);
        let img = image::imageops::resize(
            &img.to_image(),
            (x1 - x0 + 1) * AMPLIA,
            (y_max - y_min + 1) * AMPLIA,
            image::imageops::FilterType::Nearest,
        );
        img.save(format!("{base}_{n:02}.png")).unwrap();
        n += 1;
    }
    println!("{base}: {n} fatias, coluna {} px", coluna.w);
}
