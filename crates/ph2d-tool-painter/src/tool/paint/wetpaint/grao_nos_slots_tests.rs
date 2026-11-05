//! **O Grain aplicado e os caminhos fora do traço do Pincel** (Bug #65, auditoria 05/11): a Borracha (A1), a macro (A2),
//! o slot único da imagem (A4) e o valor lido do ficheiro (D4). Medições: `docs/Painter/BUGS_painter.md` #65.
use super::same_as_paper_volume_tests::{H, W, le, ponto, traco, traco_com};
use crate::tool::PainterTool;
use crate::tool::paint::wetpaint_preset::de_fabrica_ou_candidato;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase, RasterEditTool};
use ph2d_wet_paint::tuning::Knob;

fn imagem(v: u8) -> Vec<u8> {
    vec![v; 64 * 64]
}

/// Uma imagem com desenho, de média ~0,5.
fn imagem_com_desenho() -> Vec<u8> {
    (0..64 * 64)
        .map(|i| {
            if (i % 64) / 8 % 2 == (i / 64) / 8 % 2 {
                240
            } else {
                16
            }
        })
        .collect()
}

fn difs(a: &PainterTool, b: &PainterTool) -> usize {
    a.canvas_rgba
        .iter()
        .zip(b.canvas_rgba.iter())
        .filter(|(x, y)| x != y)
        .count()
}

fn risca(t: &mut PainterTool, x0: f32, n: usize) {
    t.on_canvas_pointer(ponto(x0, PointerPhase::Down));
    for k in 1..=n {
        t.on_canvas_pointer(ponto(x0 + 4.0 * k as f32, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(ponto(x0 + 4.0 * n as f32, PointerPhase::Up));
    for _ in 0..8 {
        t.on_tick(1000.0 / 60.0);
    }
}

/// A tinta na tela e a Borracha na mão, `muda`da, a passar por cima: `(massa apagada, a ferramenta)`.
fn apaga(muda: impl FnOnce(&mut PainterTool)) -> (f64, PainterTool) {
    let (antes, mut t) = traco(&de_fabrica_ou_candidato("Gouache Opaque"), false);
    t.set_paint_tool_mode("eraser");
    t.set_brush_size_px(40.0);
    muda(&mut t);
    risca(&mut t, 100.0, 50);
    (antes[0] - le(&t)[0], t)
}

/// ⭐ **A1: o Grain armado à mão morde o que a Borracha apaga, sem mudar a quantidade.** A regra «a ponta fica e o grão
/// desenha depois do portão» mandava-o pelo canal do padrão, que a Borracha não tem: Noise à mão 0 bytes de diferença.
#[test]
fn o_grain_a_mao_morde_na_borracha_e_apaga_o_mesmo() {
    let (liso, base) = apaga(|_| {});
    assert!(liso > 0.0, "premissa: a borracha apagou");
    type Muda = fn(&mut PainterTool);
    let casos: [(&str, Muda); 4] = [
        ("Noise", |t| t.set_brush_texture_kind(1)),
        ("Clouds", |t| t.set_brush_texture_kind(6)),
        ("imagem", |t| {
            t.set_brush_texture_image(imagem_com_desenho(), 64, 64);
        }),
        ("Same as Paper", |t| {
            t.set_brush_texture_kind(1);
            t.toggle_brush_grain_same_as_paper();
        }),
    ];
    let mut falhas = Vec::new();
    for (nome, muda) in casos {
        let (a, t) = apaga(muda);
        let dif = difs(&base, &t);
        println!("{nome}: {dif} bytes · apagada {:.3}", a / liso);
        // O xadrez da imagem satura o carimbo onde passa de 1: 0,80.
        if dif <= W * H / 50 || !(0.75..1.1).contains(&(a / liso)) {
            falhas.push(format!("{nome}: {dif} bytes, apagou {:.3}×", a / liso));
        }
    }
    assert!(falhas.is_empty(), "{falhas:#?}");
    // O Grain que veio com a borracha continua a ser o `texv` dela, como sempre.
    let (a, t) = apaga(|t| {
        t.set_brush_texture_kind(1);
        t.paint.brush.grao_aplicado.grao = t.paint.brush.texture;
    });
    println!(
        "Noise de preset: {} bytes · apagada {:.3}",
        difs(&base, &t),
        a / liso
    );
    assert!(difs(&base, &t) > W * H / 50, "o Noise de preset");
    // Com o filme do traço na Borracha o grão não passa pelo braço da ponta.
    let (f, filme) = apaga(|t| t.paint.wetpaint.knobs.set(Knob::StrokeFilm, 1.0));
    let (g, t) = apaga(|t| {
        t.paint.wetpaint.knobs.set(Knob::StrokeFilm, 1.0);
        t.set_brush_texture_kind(1);
    });
    println!(
        "Noise com filme: {} bytes · apagada {:.3}",
        difs(&filme, &t),
        g / f
    );
    assert!(difs(&filme, &t) > W * H / 50, "o Noise com o filme");
    // A Borracha de cerdas com o Grain tirado: as cerdas continuam a morder (o mesmo canal, Bug #61).
    let cerdas = |tira: bool| {
        apaga(move |t| {
            let (_, k) = traco(&de_fabrica_ou_candidato("Knife Dry"), false);
            t.paint.brush = k.paint.brush;
            t.set_brush_size_px(40.0);
            if tira {
                t.set_brush_texture_kind(0);
            } else {
                t.paint.brush.shape.kind = ph2d_painter_brush::TextureKind::None;
                t.set_brush_texture_kind(0);
            }
        })
    };
    let ((c, com), (s, sem)) = (cerdas(true), cerdas(false));
    println!(
        "Knife Dry sem Grain: {} bytes · apagada {:.3}",
        difs(&com, &sem),
        c / s
    );
    assert!(difs(&com, &sem) > W * H / 50, "as cerdas não morderam");
}
