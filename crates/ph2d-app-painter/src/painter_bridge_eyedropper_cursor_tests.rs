//! ⭐ **O cursor do conta-gotas mostra o que o clique apanha** — pela porta real do quadro
//! ([`crate::painter_bridge_overlays::draw_overlays`]): armado, a caixa leva a cor NOVA (o pixel debaixo
//! do cursor) e a ANTERIOR (a do pincel) e o anel do pincel sai; desarmado, o anel volta e a caixa sai.
//!
//! A régua lê as cores que entram na codificação do Vello (`draw_data`): cada cor sólida vira uma
//! palavra, e a palavra de uma cor tira-se codificando-a sozinha — a régua não sabe o empacotamento.

use super::*;
use ph2d_ecs::{PresentWorld, Transform};
use ph2d_editor_core::tool::{PanelEvent, RasterEditTool, Tool};
use ph2d_render::Sprite;

const N: u32 = 64;
const JANELA: WindowSize = WindowSize {
    width: 480,
    height: 480,
};
/// As quatro cores da tela, um quadrante cada (cima-esq, cima-dir, baixo-esq, baixo-dir).
const QUADRANTES: [[u8; 4]; 4] = [
    [220, 40, 30, 255],
    [30, 90, 220, 255],
    [250, 210, 40, 255],
    [20, 20, 24, 255],
];
const PINCEL: [u8; 3] = [40, 190, 90];

struct Cena {
    sim: SimWorld,
    present: PresentWorld,
    hero: HeroScreen,
    camera: Camera2d,
    afim: Affine,
}

fn cena() -> Cena {
    let mut sim = SimWorld::default();
    let sprite = Sprite::atlas(0, [4.0, 4.0], [1.0; 4]);
    let e = sim.world_mut().spawn((Transform::IDENTITY, sprite)).id();
    let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
    hero.gizmo.selection = Some(e.to_bits());
    let camera = Camera2d::new([0.0, 0.0], 6.0);
    let tr = ph2d_ecs::world_transform(sim.world(), e).expect("pose");
    let afim =
        ph2d_sprite_screen::sprite_image_to_screen_affine(N, N, tr, &sprite, None, &camera, JANELA);
    Cena {
        sim,
        present: PresentWorld::new(),
        hero,
        camera,
        afim,
    }
}

fn tela() -> Vec<u8> {
    let mut px = Vec::with_capacity((N * N * 4) as usize);
    for y in 0..N {
        for x in 0..N {
            let q = usize::from(x >= N / 2) + 2 * usize::from(y >= N / 2);
            px.extend_from_slice(&QUADRANTES[q]);
        }
    }
    px
}

fn painter(armado: bool) -> PainterTool {
    let mut p = PainterTool::default();
    p.set_source(tela(), N, N);
    p.set_brush_color_srgb8(PINCEL);
    if armado {
        p.handle_panel_event(PanelEvent::SelectOption(
            ph2d_editor_core::ids::PAINTER_PAINT_MODE,
            "eyedropper".to_string(),
        ));
        assert!(p.eyedropper_armed(), "o conta-gotas armou");
    }
    p
}

/// O cursor no centro do pixel `img` da tela.
fn no_pixel(c: &Cena, img: [f64; 2]) -> (f32, f32) {
    let p = c.afim * Point::new(img[0] + 0.5, img[1] + 0.5);
    (p.x as f32, p.y as f32)
}

/// O quadro de overlays do Painter, pela porta real.
fn quadro(c: &Cena, p: &PainterTool, cursor: (f32, f32)) -> VectorScene {
    let mut vs = VectorScene::new();
    let mut texto = ph2d_text::TextSystem::without_system_fonts();
    let mut perf = [0.0; crate::paint_perf::CHROME_SUB];
    esquece_a_mira();
    crate::painter_bridge_overlays::draw_overlays(
        p,
        &c.hero,
        &c.sim,
        c.present.world(),
        &c.camera,
        JANELA,
        &mut vs,
        &mut texto,
        cursor,
        &mut perf,
        false,
        false,
    );
    vs
}

/// A palavra que uma cor sólida põe no `draw_data`.
fn palavra(c: Color) -> u32 {
    let mut s = VectorScene::new();
    s.fill_rect(Rect::new(0.0, 0.0, 1.0, 1.0), c);
    *s.inner().encoding().draw_data.last().expect("a cor entrou")
}

fn tem(vs: &VectorScene, c: Color) -> bool {
    vs.inner().encoding().draw_data.contains(&palavra(c))
}

fn rgb(c: [u8; 3]) -> Color {
    Color::from_rgba8(c[0], c[1], c[2], u8::MAX)
}

#[test]
fn the_eyedropper_cursor_shows_the_new_and_the_previous_colour_and_hides_the_ring() {
    let c = cena();
    let anel = Color::new(crate::painter_bridge_brush_ring::COR_DO_ANEL);
    let armado = painter(true);
    for (q, img) in [[10.0, 12.0], [50.0, 3.0], [7.0, 40.0], [60.0, 60.0]]
        .into_iter()
        .enumerate()
    {
        let vs = quadro(&c, &armado, no_pixel(&c, img));
        let [r, g, b, _] = QUADRANTES[q];
        assert!(
            tem(&vs, rgb([r, g, b])),
            "q{q}: a metade de cima é o pixel debaixo do cursor"
        );
        assert!(
            tem(&vs, rgb(PINCEL)),
            "q{q}: a metade de baixo é a cor do pincel"
        );
        assert!(
            !tem(&vs, anel),
            "q{q}: com o conta-gotas o anel do pincel sai"
        );
        assert!(
            mira_na_tela(),
            "q{q}: a shell põe o cursor do sistema em cruz"
        );
        // As OUTRAS cores da tela não entram: a caixa mostra só o pixel amostrado.
        for (k, o) in QUADRANTES.iter().enumerate().filter(|(k, _)| *k != q) {
            assert!(
                !tem(&vs, rgb([o[0], o[1], o[2]])),
                "q{q}: a cor do quadrante {k} vazou"
            );
        }
    }
    // Desarmado: o anel volta, a caixa e a mira saem.
    let vs = quadro(&c, &painter(false), no_pixel(&c, [10.0, 12.0]));
    assert!(tem(&vs, anel), "sem o conta-gotas o anel do pincel está lá");
    let [r, g, b, _] = QUADRANTES[0];
    assert!(
        !tem(&vs, rgb([r, g, b])) && !tem(&vs, rgb(PINCEL)),
        "sem o conta-gotas não há caixa"
    );
    assert!(
        !mira_na_tela(),
        "sem o conta-gotas o cursor do sistema não muda"
    );
    // Fora da tela não há o que apanhar.
    let vs = quadro(&c, &armado, (2.0, 2.0));
    assert!(
        !tem(&vs, rgb(PINCEL)) && !mira_na_tela(),
        "fora da tela não há caixa"
    );
}

/// A caixa nunca cobre o pixel amostrado, e vira para o outro lado na borda da janela.
#[test]
fn the_eyedropper_swatch_never_covers_the_sample_and_flips_at_the_window_edge() {
    let c = cena();
    let p = painter(true);
    let (w, h) = (f64::from(JANELA.width), f64::from(JANELA.height));
    let janela = Rect::new(0.0, 0.0, w, h);
    for img in [
        [32.0, 32.0],
        [0.0, 0.0],
        [63.0, 0.0],
        [63.0, 63.0],
        [0.0, 63.0],
    ] {
        let cur = no_pixel(&c, img);
        let pl = plano(&p, c.afim, cur, JANELA, 1.0).expect("dentro da tela");
        assert!(!pl.caixa.contains(pl.mira), "{img:?}: a caixa cobre a mira");
        assert!(
            janela.contains(pl.caixa.origin()) && pl.caixa.x1 <= w && pl.caixa.y1 <= h,
            "{img:?}: a caixa saiu da janela {:?}",
            pl.caixa
        );
    }
    // Encostada à direita e em cima: a caixa vai para a esquerda e para baixo.
    let pl = plano(&p, c.afim, (w as f32 - 2.0, 2.0), JANELA, 1.0);
    assert!(pl.is_none(), "fora da tela não há plano");
    let canto = Affine::translate((w - 40.0, 0.0)) * Affine::scale(0.5);
    let pl = plano(&p, canto, (w as f32 - 10.0, 4.0), JANELA, 1.0).expect("dentro da tela");
    assert!(
        pl.caixa.x1 < pl.mira.x && pl.caixa.y0 > pl.mira.y,
        "não virou: {:?}",
        pl.caixa
    );
}

/// **A foto** do cursor sobre a tela pintada (o canvas, a mira e a caixa), gravada em PNG.
/// `PH2D_FOTO_DIR=<pasta> cargo test -p ph2d-app-painter foto_do_conta_gotas -- --ignored`.
#[test]
#[ignore = "foto do cursor do conta-gotas — PH2D_FOTO_DIR=<pasta> e --ignored"]
fn foto_do_conta_gotas() {
    let Ok(dir) = std::env::var("PH2D_FOTO_DIR") else {
        return;
    };
    let gpu = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None)
        .expect("a foto precisa da placa");
    std::fs::create_dir_all(&dir).unwrap();
    let c = cena();
    let p = painter(true);
    let mut vs = VectorScene::new();
    vs.fill_rect(
        Rect::new(0.0, 0.0, f64::from(JANELA.width), f64::from(JANELA.height)),
        {
            let k = ColorToken::WindowGround.resolve(c.hero.theme);
            Color::from_rgba8(k.r, k.g, k.b, u8::MAX)
        },
    );
    vs.draw_image_rgba_transformed(
        &std::sync::Arc::new(tela()),
        N,
        N,
        c.afim,
        ph2d_vector::ImageQuality::Low,
    );
    let por_cima = quadro(&c, &p, no_pixel(&c, [12.0, 44.0]));
    vs.append_transformed(&por_cima, Affine::IDENTITY);
    let mut pass = ph2d_render::VelloPass::new(
        &gpu,
        ph2d_gpu::TextureFormat::Rgba8Unorm,
        (JANELA.width, JANELA.height),
    )
    .unwrap();
    let px = pass
        .render_and_readback(&gpu, vs.inner(), (JANELA.width, JANELA.height))
        .unwrap();
    image::save_buffer(
        format!("{dir}/conta_gotas_depois.png"),
        &px,
        JANELA.width,
        JANELA.height,
        image::ExtendedColorType::Rgba8,
    )
    .unwrap();
}
