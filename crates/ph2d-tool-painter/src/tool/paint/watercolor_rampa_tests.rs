//! Os gates da **Shape Color Ramp na aquarela** (doc 46 §2-7) — a rampa entra no splat da COR
//! (`watercolor_accum_cor::RampaDaAguada`) e, no modo B&W, no peso das DUAS passadas.

use super::measure_shape_system::{cp, tool};
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_color::{ColorRamp, RampColorMode, RampInterp, RampStop};
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};

/// Uma aguada azul de `(20,64)` a `(108,64)`, com a rampa como `arma` a deixar.
fn aguada_t(arma: impl Fn(&mut PainterTool)) -> PainterTool {
    let mut t = tool(128, PaintMedia::Watercolor, 10.0);
    t.set_brush_color_srgb8([30, 60, 220]);
    arma(&mut t);
    t.on_canvas_pointer(cp([20.0, 64.0], PointerPhase::Down));
    for k in 1..=22 {
        #[allow(clippy::cast_precision_loss)]
        let x = 20.0 + 4.0 * k as f32;
        t.on_canvas_pointer(cp([x, 64.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([108.0, 64.0], PointerPhase::Up));
    t
}

fn aguada(arma: impl Fn(&mut PainterTool)) -> Vec<u8> {
    aguada_t(arma).canvas_rgba.to_vec()
}

fn rampa(cor: [f32; 4]) -> ColorRamp {
    ColorRamp::new(
        vec![RampStop::new(0.0, cor), RampStop::new(1.0, cor)],
        RampColorMode::Rgb,
        RampInterp::Linear,
    )
}

/// Texels onde o vermelho domina o azul por uma margem visível.
fn avermelhados(px: &[u8]) -> usize {
    px.chunks(4)
        .filter(|p| i32::from(p[0]) > i32::from(p[2]) + 40)
        .count()
}

/// **A RAMPA É A DONA DA COR NA AGUADA** — uma rampa vermelha sobre um pincel azul pinta uma aguada
/// vermelha (a lei do traço digital, `shape_colour_ramp_colourises_the_silhouette_when_grain_is_none`).
#[test]
fn a_rampa_da_shape_pinta_a_cor_da_aguada() {
    let sem = aguada(|_| {});
    let com = aguada(|t| {
        t.set_shape_color_ramp(rampa([1.0, 0.0, 0.0, 1.0]));
        t.set_shape_ramp_enabled(true);
    });
    assert_eq!(
        avermelhados(&sem),
        0,
        "controlo: a aguada azul não tem vermelho"
    );
    let n = avermelhados(&com);
    assert!(
        n > 300,
        "a rampa vermelha não chegou à aguada ({n} texels vermelhos)"
    );
}

/// **No modo B&W a rampa é o TOM da silhueta da Shape** — a lei do traço digital (`stamp.rs`,
/// `remap_shape_value` sobre o valor CRU): com uma Shape procedural na aguada (Automatic desligado) o
/// valor dela é a DENSIDADE da ponta, e uma rampa de cinzento escuro baixa-a — a aguada sai mais
/// CLARA, na cor do pincel. Sem Shape o tom não tem silhueta a remapear, como no digital.
#[test]
fn o_tom_da_rampa_pesa_a_silhueta_da_shape() {
    let soma = |px: &[u8]| {
        px.chunks(4)
            .map(|p| u64::from(p[0]) + u64::from(p[1]))
            .sum::<u64>()
    };
    let shape = |t: &mut PainterTool| {
        t.paint.brush.watercolor_shape_auto = false;
        t.paint.brush.shape.kind = ph2d_painter_brush::TextureKind::Noise;
    };
    let tom = |t: &mut PainterTool| {
        t.set_shape_color_ramp(rampa([0.05, 0.05, 0.05, 1.0]));
        t.set_shape_ramp_enabled(true);
        t.paint.shape_color_ramp_bw = true;
    };
    let sem_t = aguada_t(shape);
    let com_t = aguada_t(|t| {
        shape(t);
        tom(t);
    });
    assert_ne!(
        sem_t.paint.stroke_density, com_t.paint.stroke_density,
        "o tom não chegou à densidade da ponta"
    );
    let (sem, com) = (sem_t.canvas_rgba.to_vec(), com_t.canvas_rgba.to_vec());
    assert_eq!(
        avermelhados(&com),
        0,
        "o tom não pode trocar a cor do pincel"
    );
    assert!(
        soma(&com) > soma(&sem) + 20_000,
        "o tom escuro não aclarou a aguada ({} contra {})",
        soma(&com),
        soma(&sem)
    );
    assert_eq!(
        aguada(|_| {}),
        aguada(tom),
        "sem Shape o tom remapeou alguma coisa — no digital ele não age sem silhueta"
    );
}

/// **O tom vale já no PRIMEIRO carimbo depois de mexer na rampa** — com uma ponta de IMAGEM o valor
/// da Shape é a própria cobertura, e a passada da cobertura corre ANTES da da cor: se só a da cor
/// garantisse a LUT do tom, o 1.º lote leria a LUT velha e a cobertura discordaria da cor.
#[test]
fn o_tom_vale_no_primeiro_carimbo_de_uma_ponta_de_imagem() {
    let pinga = |tom: bool| {
        let mut t = tool(64, PaintMedia::Watercolor, 12.0);
        t.paint.brush.watercolor_shape_auto = false;
        // Uma rampa de luminância a meio-tom: a ponta de imagem satura o miolo, o tom puxa-o para baixo.
        let mut lum = vec![0u8; 16 * 16];
        for (i, v) in lum.iter_mut().enumerate() {
            let (x, y) = (i % 16, i / 16);
            *v = if (4..12).contains(&x) && (4..12).contains(&y) {
                255
            } else {
                40
            };
        }
        t.set_brush_shape_image(lum, 16, 16);
        if tom {
            t.set_shape_color_ramp(rampa([0.02, 0.02, 0.02, 1.0]));
            t.set_shape_ramp_enabled(true);
            t.paint.shape_color_ramp_bw = true;
        }
        t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
        t.paint.stroke_coverage.clone()
    };
    let (sem, com) = (pinga(false), pinga(true));
    assert!(
        sem.iter().any(|&c| c > 0),
        "controlo: a ponta de imagem molha o papel"
    );
    assert_ne!(
        sem, com,
        "o 1.º carimbo leu a LUT do tom velha — a cobertura ignorou o tom"
    );
}
