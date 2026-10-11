//! 📷 O fotógrafo das cenas ao vivo (12 e 13): uma janela — o ecrã, o texto e a SUA placa — que pinta o
//! hero inteiro e grava o PNG numa pasta do temporário. Só os testes `#[ignore]` de fotos o usam.

use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::screens::hero::{HERO_VIEWPORT_H, HERO_VIEWPORT_W, paint_hero_screen};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

/// Uma janela: o ecrã, o texto e a placa dela.
pub(crate) struct Shots {
    gpu: ph2d_gpu::GpuContext,
    pass: ph2d_render::VelloPass,
    ts: TextSystem,
    dir: std::path::PathBuf,
}

impl Shots {
    /// As fotos vão para `<temp>/<folder>/`.
    pub(crate) fn new(folder: &str) -> Self {
        ph2d_editor_core::test_support::ensure_panel_registry();
        let gpu = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None).unwrap();
        let size = (HERO_VIEWPORT_W as u32, HERO_VIEWPORT_H as u32);
        let pass =
            ph2d_render::VelloPass::new(&gpu, ph2d_gpu::TextureFormat::Rgba8Unorm, size).unwrap();
        let dir = std::env::temp_dir().join(folder);
        std::fs::create_dir_all(&dir).unwrap();
        Self {
            gpu,
            pass,
            ts: TextSystem::without_system_fonts(),
            dir,
        }
    }

    fn paint(&mut self, h: &mut HeroScreen) -> VectorScene {
        let mut scene = VectorScene::new();
        let vp = Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H);
        paint_hero_screen(h, vp, &mut scene, &mut self.ts);
        scene
    }

    /// Pinta duas vezes (a 1.ª assenta o que o desenho ajusta) e grava `<name>.png`.
    pub(crate) fn shot(&mut self, h: &mut HeroScreen, name: &str) {
        self.paint(h);
        let scene = self.paint(h);
        let size = (HERO_VIEWPORT_W as u32, HERO_VIEWPORT_H as u32);
        let px = self
            .pass
            .render_and_readback(&self.gpu, scene.inner(), size)
            .unwrap();
        let img = ph2d_board_export::png::Image {
            width: size.0,
            height: size.1,
            rgba: px,
        };
        let path = self.dir.join(format!("{name}.png"));
        std::fs::write(&path, ph2d_board_export::png::encode(&img)).unwrap();
        println!("foto: {}", path.display());
    }
}
