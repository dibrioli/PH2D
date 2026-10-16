//! A régua comum do texto pequeno RASTERIZADO: pinta pela porta do produto (`paint_text` dentro
//! do `pintar_no_chrome`), rasteriza no `VelloPass` e devolve a cobertura (alfa) de cada píxel.
//! Usada pelo gate `as_duas_barras_do_igual_sao_iguais` e pela sonda `mede_o_peso_do_texto_com_hint`.

use parley::{FontFamily, StyleProperty};
use ph2d_editor_core::paint::{paint_text, set_text_rendering};
use ph2d_editor_core::ui_scale::{UiScaleMap, ao_pixel, pintar_no_chrome};
use ph2d_editor_core::zones::Rect;
use ph2d_gpu::GpuContext;
use ph2d_render::VelloPass;
use ph2d_text::{PositionedLayoutItem, TextSystem, set_active_text_style};
use ph2d_tokens::{TextRendering, TypeToken, UiFont, UiScale, UiTextSize, UiTextStyle, UiWeight};
use ph2d_vector::{Affine, Color, Fill, Glyph, VectorScene};

pub(crate) const X0: f32 = 4.3;
pub(crate) const Y0: f32 = 6.4;

/// O estilo do dono em 15/10 (`~/.ph2d/prefs.txt`: noto_sans · light · small).
pub(crate) fn estilo_do_dono() -> UiTextStyle {
    UiTextStyle {
        font: UiFont::NotoSans,
        weight: UiWeight::Light,
        size: UiTextSize::Small,
    }
}

pub(crate) struct Placa {
    gpu: GpuContext,
    pass: VelloPass,
}

/// `None` sem adaptador (a CI): quem chama diz que não mediu.
pub(crate) fn placa(max: (u32, u32)) -> Option<Placa> {
    let gpu = GpuContext::new(GpuContext::default_instance(), None).ok()?;
    let pass = VelloPass::new(&gpu, ph2d_gpu::TextureFormat::Rgba8Unorm, max).ok()?;
    Some(Placa { gpu, pass })
}

/// Uma imagem de cobertura: `a[r * w + c]` em `0..=1`.
pub(crate) struct Alfa {
    pub w: u32,
    pub h: u32,
    pub a: Vec<f32>,
}

impl Alfa {
    /// A tinta de cada linha de píxel.
    pub(crate) fn linhas(&self) -> Vec<f32> {
        (0..self.h as usize)
            .map(|r| {
                self.a[r * self.w as usize..(r + 1) * self.w as usize]
                    .iter()
                    .sum()
            })
            .collect()
    }
}

/// O que se pinta: texto da interface (estilo do artista) ou a mono a `px`.
#[derive(Copy, Clone)]
pub(crate) enum Letra {
    Ui(UiTextStyle),
    Mono(f32),
}

/// Pinta `texto` com o `preset` à escala `s` (`hint`: `None` = o do preset) e lê a cobertura.
pub(crate) fn rasteriza(
    placa: &mut Placa,
    ts: &mut TextSystem,
    letra: Letra,
    (preset, hint): (TextRendering, Option<bool>),
    s: f32,
    texto: &str,
    (lw, lh): (f32, f32),
) -> Alfa {
    set_text_rendering(preset);
    let mapa = UiScaleMap::no_ecra(UiScale::P100, s);
    let (w, h) = ((lw * s).ceil() as u32, (lh * s).ceil() as u32);
    let mut cena = VectorScene::new();
    pintar_no_chrome(
        mapa,
        Rect::new(0.0, 0.0, w as f32, h as f32),
        &mut cena,
        |_, c| match letra {
            Letra::Ui(estilo) => {
                set_active_text_style(estilo);
                paint_text(ts, c, texto, X0, Y0, TypeToken::Sm.px(), lw, Color::WHITE);
            }
            Letra::Mono(px) => emite_mono(ts, c, texto, px, preset.params().hint),
        },
    );
    if let Some(hint) = hint {
        for run in &mut cena.inner_mut().encoding_mut().resources.glyph_runs {
            run.hint = hint;
        }
    }
    set_text_rendering(TextRendering::Default);
    set_active_text_style(UiTextStyle::default());
    let px = placa
        .pass
        .render_and_readback(&placa.gpu, cena.inner(), (w, h))
        .unwrap();
    let a = px.chunks(4).map(|p| f32::from(p[3]) / 255.0).collect();
    Alfa { w, h, a }
}

/// A mono ainda não tem pintor no produto: o laço do `paint_text_lines` sem o estilo da interface.
fn emite_mono(ts: &mut TextSystem, cena: &mut VectorScene, texto: &str, px: f32, hint: bool) {
    let (fcx, lcx, stack) = ts.mono_contexts();
    let mut b = lcx.ranged_builder(fcx, texto, 1.0, true);
    b.push_default(StyleProperty::FontFamily(FontFamily::Source(stack.into())));
    b.push_default(StyleProperty::FontSize(px));
    let mut layout = b.build(texto);
    layout.break_all_lines(None);
    let t = Affine::translate((f64::from(ao_pixel(X0)), f64::from(ao_pixel(Y0))));
    let inner = cena.inner_mut();
    for line in layout.lines() {
        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(gr) = item else {
                continue;
            };
            let run = gr.run();
            inner
                .draw_glyphs(run.font())
                .font_size(run.font_size())
                .hint(hint)
                .normalized_coords(run.normalized_coords())
                .brush(Color::WHITE)
                .transform(t)
                .draw(
                    Fill::NonZero,
                    gr.positioned_glyphs().map(|g| Glyph {
                        id: g.id,
                        x: g.x,
                        y: ao_pixel(g.y),
                    }),
                );
        }
    }
}

/// Recorte ×`k` (vizinho mais próximo), branco sobre preto, das linhas e colunas com tinta.
pub(crate) fn grava_png(alfa: &Alfa, k: u32, caminho: &std::path::Path) {
    let (w, h) = (alfa.w, alfa.h);
    let tem = |r: u32, c: u32| alfa.a[(r * w + c) as usize] > 0.0;
    let rows: Vec<u32> = (0..h).filter(|&r| (0..w).any(|c| tem(r, c))).collect();
    let cols: Vec<u32> = (0..w).filter(|&c| (0..h).any(|r| tem(r, c))).collect();
    let (Some(&r0), Some(&r1), Some(&c0), Some(&c1)) =
        (rows.first(), rows.last(), cols.first(), cols.last())
    else {
        return;
    };
    let (r0, c0) = (r0.saturating_sub(1), c0.saturating_sub(1));
    let (r1, c1) = ((r1 + 1).min(h - 1), (c1 + 1).min(w - 1));
    let (cw, ch) = ((c1 - c0 + 1) * k, (r1 - r0 + 1) * k);
    let mut out = vec![0u8; (cw * ch) as usize];
    for y in 0..ch {
        for x in 0..cw {
            let v = alfa.a[((r0 + y / k) * w + c0 + x / k) as usize];
            out[(y * cw + x) as usize] = (v * 255.0).round() as u8;
        }
    }
    image::GrayImage::from_raw(cw, ch, out)
        .unwrap()
        .save(caminho)
        .unwrap();
}
