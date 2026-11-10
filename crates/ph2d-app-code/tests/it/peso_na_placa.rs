//! O peso da aba de scripts MEDIDO NO PÍXEL: a tinta (soma da cobertura) de um texto pintado pela
//! porta com peso no preset de nitidez de FÁBRICA (`CrispHeavyPlus`) contra o mesmo texto no peso
//! do CSS da maquete 14 sem reforço (`Default`). Antes da cura o corpo `400` saía `700` (o `+300` do
//! preset a `≤ 12 px`) e o eixo lido na cena no `Default` dizia `400`. Sem placa, não mede.

use ph2d_editor_core::paint::{paint_text_in_weight, set_text_rendering};
use ph2d_gpu::GpuContext;
use ph2d_render::VelloPass;
use ph2d_text::{FontWeight, TextSystem, set_active_text_style};
use ph2d_tokens::{TextRendering, TypeToken, UiTextStyle};
use ph2d_vector::{Color, VectorScene};

const CAIXA: (u32, u32) = (160, 32);
const TEXTOS: [&str; 3] = ["number", "Hexagon", "Click a call to go to its line"];

fn tinta(
    gpu: &GpuContext,
    pass: &mut VelloPass,
    ts: &mut TextSystem,
    preset: TextRendering,
    texto: &str,
    peso: FontWeight,
) -> f32 {
    set_text_rendering(preset);
    let mut cena = VectorScene::new();
    let ink = Color::from_rgba8(255, 255, 255, 255);
    let (px, w) = (TypeToken::Sm.px(), CAIXA.0 as f32 * 2.0);
    paint_text_in_weight(ts, &mut cena, texto, [2.0, 4.0], px, w, ink, peso);
    set_text_rendering(TextRendering::Default);
    let (cw, ch) = (CAIXA.0 * 2, CAIXA.1);
    let rgba = pass
        .render_and_readback(gpu, cena.inner(), (cw, ch))
        .expect("render");
    rgba.chunks(4).map(|p| f32::from(p[3]) / 255.0).sum()
}

#[test]
fn o_corpo_da_aba_tem_no_pixel_a_tinta_do_css() {
    let Ok(gpu) = GpuContext::new(GpuContext::default_instance(), None) else {
        eprintln!("sem placa — não mede");
        return;
    };
    let tam = (CAIXA.0 * 2, CAIXA.1);
    let mut pass = VelloPass::new(&gpu, ph2d_gpu::TextureFormat::Rgba8Unorm, tam).expect("vello");
    let mut ts = TextSystem::without_system_fonts();
    set_active_text_style(UiTextStyle::default());
    let (app, css) = (TextRendering::CrispHeavyPlus, TextRendering::Default);
    for texto in TEXTOS {
        let mut t = |p, w| tinta(&gpu, &mut pass, &mut ts, p, texto, w);
        let aba = t(app, FontWeight::NORMAL);
        let css400 = t(css, FontWeight::NORMAL);
        let css600 = t(css, FontWeight::SEMI_BOLD);
        let antes = t(css, FontWeight::BOLD);
        eprintln!(
            "«{texto}»: aba {aba:.1} · css400 {css400:.1} · css600 {css600:.1} · antes(700) {antes:.1}"
        );
        assert!(
            (aba / css400 - 1.0).abs() <= 0.05,
            "«{texto}»: a aba pinta {aba:.1} de tinta, o CSS 400 {css400:.1}"
        );
        assert!(
            css600 / css400 >= 1.15,
            "a régua não separa 400 de 600 ({css400:.1} × {css600:.1})"
        );
    }
}
