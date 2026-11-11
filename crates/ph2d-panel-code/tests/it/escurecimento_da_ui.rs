//! ⭐ **A sonda da quantidade do escurecimento da INTERFACE** (dono 10/11: «para a UI também não
//! faz diferença»): rótulos do chrome (`paint_text`, Inter) a 11/12/13/15 px, as tintas Text1/Text2/
//! Text3/Accent sobre as superfícies escuras (PanelBg, Bg1, Bg2, BgElev, KanbanList, KanbanCard) do
//! Dark e do Forge, o alargamento posto à mão em cada corrida. Imprime, por corpo, a média e o pior
//! de: píxeis da caixa do texto que mudam > 4/255 contra o `+0` e a massa da tinta relativa ao `+0`.
//! Sem placa, não mede. A tabela vive no doc de `ph2d_tokens::STEM_DARKENING_PX`.

use ph2d_editor_core::paint::{paint_text, rect_to_vello, resolve, set_text_rendering};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, TextRendering, Theme};
use ph2d_vector::{Color, VectorScene};

const W: usize = 420;
const H: usize = 40;
/// Rótulos reais do chrome: Hierarchy, Inspector, timeline, menu.
const ROTULO: &str = "Hierarchy  Transform  Position 120 px  Edit Preferences";
const SUPERFICIES: [ColorToken; 6] = [
    ColorToken::PanelBg,
    ColorToken::Bg1,
    ColorToken::Bg2,
    ColorToken::BgElev,
    ColorToken::KanbanList,
    ColorToken::KanbanCard,
];
const TINTAS: [ColorToken; 4] = [
    ColorToken::Text1,
    ColorToken::Text2,
    ColorToken::Text3,
    ColorToken::Accent,
];
const QUANTIDADES: [f64; 5] = [0.05, 0.075, 0.1, 0.125, 0.15];

fn lum(c: Color) -> f64 {
    f64::from(c.discard_alpha().relative_luminance())
}

/// A luminância de cada superfície e tinta, por tema: a régua do critério da interface.
#[test]
fn a_tabela_das_superficies_da_interface() {
    for t in Theme::ALL {
        let s: Vec<String> = SUPERFICIES
            .iter()
            .chain(TINTAS.iter())
            .map(|k| format!("{k:?} {:.3}", lum(resolve(*k, t))))
            .collect();
        eprintln!("{t:?} dark={} · {}", t.is_dark(), s.join(" · "));
    }
}

fn cena(ts: &mut TextSystem, px: f32, ink: Color, bg: Color) -> (VectorScene, [usize; 4]) {
    let mut s = VectorScene::new();
    s.fill_rect(rect_to_vello(Rect::new(0.0, 0.0, W as f32, H as f32)), bg);
    set_text_rendering(TextRendering::CrispHeavyPlus);
    paint_text(ts, &mut s, ROTULO, 4.0, 8.0, px, f32::INFINITY, ink);
    set_text_rendering(TextRendering::Default);
    let r = &s.inner().encoding().resources;
    let x1 = r.glyphs.iter().map(|g| g.x).fold(0.0_f32, f32::max) + px;
    let y = r.glyphs.iter().map(|g| g.y).fold(0.0_f32, f32::max);
    let caixa = [
        3,
        (x1 as usize).min(W - 1),
        (y - px) as usize,
        ((y + px * 0.35) as usize).min(H - 1),
    ];
    (s, caixa)
}

#[test]
fn a_quantidade_do_escurecimento_da_interface() {
    use ph2d_gpu::GpuContext;
    use ph2d_render::VelloPass;
    let Ok(gpu) = GpuContext::new(GpuContext::default_instance(), None) else {
        eprintln!("sem placa — não mede");
        return;
    };
    let mut pass = VelloPass::new(
        &gpu,
        ph2d_gpu::TextureFormat::Rgba8Unorm,
        (W as u32, H as u32),
    )
    .expect("vello");
    let mut ts = TextSystem::without_system_fonts();
    let escolhido = f64::from(ph2d_tokens::STEM_DARKENING_PX);
    let luma = |p: &[u8], i: usize| {
        (0.2126 * f64::from(p[i * 4])
            + 0.7152 * f64::from(p[i * 4 + 1])
            + 0.0722 * f64::from(p[i * 4 + 2]))
            / 255.0
    };
    for px in [11.0_f32, 12.0, 13.0, 15.0] {
        // (Σ muda, max muda, Σ cresce, max cresce, n) por quantidade.
        let mut acc = [[0.0_f64, 0.0, 0.0, f64::MIN, 0.0]; QUANTIDADES.len()];
        for theme in [Theme::Dark, Theme::Forge] {
            for bg_t in SUPERFICIES {
                let bg = resolve(bg_t, theme);
                for ink_t in TINTAS {
                    let ink = resolve(ink_t, theme);
                    let mut render = |a: f64| {
                        let (mut s, cx) = cena(&mut ts, px, ink, bg);
                        for g in &mut s.inner_mut().encoding_mut().resources.glyph_runs {
                            g.font_embolden =
                                ph2d_vector::FontEmbolden::new(ph2d_vector::Diagonal2::new(a, a));
                        }
                        let img = pass
                            .render_and_readback(&gpu, s.inner(), (W as u32, H as u32))
                            .expect("render");
                        (img, cx)
                    };
                    let (base, [x0, x1, y0, y1]) = render(0.0);
                    let idx: Vec<usize> = (y0..y1)
                        .flat_map(|y| (x0..x1).map(move |x| y * W + x))
                        .collect();
                    let lb = luma(&base, 0);
                    let massa =
                        |p: &[u8]| idx.iter().map(|&i| (luma(p, i) - lb).abs()).sum::<f64>();
                    let m0 = massa(&base).max(1.0);
                    for (k, a) in QUANTIDADES.iter().enumerate() {
                        let (img, _) = render(*a);
                        let muda = idx
                            .iter()
                            .filter(|&&i| {
                                (0..3).any(|c| img[i * 4 + c].abs_diff(base[i * 4 + c]) > 4)
                            })
                            .count() as f64
                            / idx.len() as f64;
                        let cresce = massa(&img) / m0 - 1.0;
                        let e = &mut acc[k];
                        e[0] += muda;
                        e[1] = e[1].max(muda);
                        e[2] += cresce;
                        e[3] = e[3].max(cresce);
                        e[4] += 1.0;
                    }
                }
            }
        }
        for (k, a) in QUANTIDADES.iter().enumerate() {
            let e = acc[k];
            let (muda, cresce) = (e[0] / e[4], e[2] / e[4]);
            eprintln!(
                "{px} px +{a:.3}: mudam {:.1} % (pior {:.1} %) · tinta {:+.1} % (pior {:+.1} %)",
                muda * 100.0,
                e[1] * 100.0,
                cresce * 100.0,
                e[3] * 100.0
            );
            if (a - escolhido).abs() < 1e-6 {
                assert!(muda >= 0.02, "{px} px: +{a} não se vê ({muda:.3})");
                assert!(cresce <= 0.15, "{px} px: +{a} lê-se negrito ({cresce:+.3})");
            }
        }
    }
}
