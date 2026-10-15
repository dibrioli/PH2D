//! **Sonda da lista 26 A8** (smoke do dono 15/10): na Watercolor, com a Texture Color do Shape ligada,
//! um dab a `Charge = 1` mostra as cores da rampa; a qualquer outro valor só fica a cor do pincel. Que
//! OUTROS parâmetros da Watercolor fazem o mesmo? Varre cada campo que o painel da aquarela despacha
//! (pela PORTA do painel, `route_brush_watercolor_event`) a dois valores fora do padrão e mede:
//!
//! - `rampa` — a fração média da cor que NÃO é a do pincel: pincel vermelho puro, rampa azul → verde,
//!   `1 − R/(R+G+B)` sobre os texels pintados (≈ 0 = só o pincel; ≈ 1 = só a rampa);
//! - `corr` — a correlação matiz × padrão do gate P38 (matiz `t = G/(G+B)` contra o α do mesmo Shape
//!   no Digital, queda dura, sem rampa), no dab único;
//! - `traço` — a mesma `rampa` sobre um traço de 88 px (onde a CARGA gasta-se com a distância).

use super::*;
use crate::tool::paint::PaintMedia;
use ph2d_color::{ColorRamp, RampColorMode, RampInterp, RampStop};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_painter_brush::texture::{TextureKind, TextureMapping};

const W: u32 = 96;
const R: f32 = 30.0;
const C: f32 = 48.5;

fn azul_verde() -> ColorRamp {
    ColorRamp::new(
        vec![
            RampStop::new(0.0, [0.0, 0.0, 1.0, 1.0]),
            RampStop::new(1.0, [0.0, 1.0, 0.0, 1.0]),
        ],
        RampColorMode::Rgb,
        RampInterp::Linear,
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Camadas {
    Textura,
    Queda,
    Bw,
    Off,
}

/// O pincel do caso: `meio`, Noise no Shape (View, Size 1,5), pincel vermelho, as camadas pedidas, e
/// `arma` (o parâmetro varrido, pela porta do painel). Devolve a ferramenta pronta a pintar.
fn pincel(meio: PaintMedia, camadas: Camadas, arma: &dyn Fn(&mut PainterTool)) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![0u8; (W * W * 4) as usize], W, W);
    t.set_paint_media(meio);
    let b = &mut t.paint.brush;
    b.radius_px = R;
    b.color = [1.0, 0.0, 0.0];
    b.space_attenuation = false;
    b.watercolor_shape_auto = false;
    (b.falloff, b.hardness) = (Falloff::Constant, 1.0);
    t.set_brush_shape_kind(TextureKind::Noise.to_u8());
    t.paint.brush.shape.mapping = TextureMapping::ViewPlane;
    t.paint.brush.shape.size = [1.5; 2];
    t.set_shape_color_ramp(azul_verde());
    t.set_shape_falloff_ramp(azul_verde());
    t.set_shape_ramp_enabled(camadas != Camadas::Off);
    t.paint.shape_color_ramp_bw = camadas == Camadas::Bw;
    t.paint.shape_camadas.textura = matches!(camadas, Camadas::Textura | Camadas::Bw);
    t.paint.shape_camadas.queda = camadas == Camadas::Queda;
    t.set_shape_falloff_ramp(azul_verde()); // re-assa as LUTs
    arma(&mut t);
    let seed = t.paint.brush;
    t.paint.brush_by_mode.fill(seed);
    t
}

fn dab(mut t: PainterTool) -> PainterTool {
    t.on_canvas_pointer(cp([C, C], PointerPhase::Down));
    t.on_canvas_pointer(cp([C, C], PointerPhase::Up));
    t
}

fn traco(mut t: PainterTool) -> PainterTool {
    t.on_canvas_pointer(cp([4.0, C], PointerPhase::Down));
    for k in 1..=22 {
        #[allow(clippy::cast_precision_loss)]
        let x = 4.0 + 4.0 * k as f32;
        t.on_canvas_pointer(cp([x, C], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([92.0, C], PointerPhase::Up));
    t
}

/// `(rgb reto, α)` de cada texel do miolo do dab (fora da orla).
fn miolo(t: &PainterTool) -> Vec<([f32; 3], f32)> {
    let mut v = Vec::new();
    for y in 0..W {
        for x in 0..W {
            let (dx, dy) = (x as f32 + 0.5 - C, y as f32 + 0.5 - C);
            if dx * dx + dy * dy >= (R - 2.0) * (R - 2.0) {
                continue;
            }
            let i = ((y * W + x) * 4) as usize;
            let p = &t.canvas_rgba[i..i + 4];
            let f = |c: u8| f32::from(c) / 255.0;
            v.push(([f(p[0]), f(p[1]), f(p[2])], f(p[3])));
        }
    }
    v
}

/// A fração média da cor que não é a do pincel (vermelho), sobre os texels pintados.
fn fracao_da_rampa(px: &[u8]) -> f32 {
    let (mut s, mut n) = (0.0, 0.0);
    for p in px.chunks(4) {
        if p[3] < 13 {
            continue;
        }
        let (r, g, b) = (f32::from(p[0]), f32::from(p[1]), f32::from(p[2]));
        s += 1.0 - r / (r + g + b).max(1.0);
        n += 1.0;
    }
    if n > 0.0 { s / n } else { f32::NAN }
}

fn correlacao(a: &[f32], b: &[f32]) -> Option<f32> {
    let n = a.len() as f32;
    let (ma, mb) = (a.iter().sum::<f32>() / n, b.iter().sum::<f32>() / n);
    let (mut sab, mut saa, mut sbb) = (0.0, 0.0, 0.0);
    for (x, y) in a.iter().zip(b) {
        sab += (x - ma) * (y - mb);
        saa += (x - ma) * (x - ma);
        sbb += (y - mb) * (y - mb);
    }
    (saa > 1e-6 && sbb > 1e-6).then(|| sab / (saa * sbb).sqrt())
}

#[derive(Debug)]
struct Leitura {
    rampa: f32,
    corr: Option<f32>,
    traco: f32,
    /// `(dab, traço)` iguais ao byte ao mesmo caso SEM rampa.
    igual_off: (bool, bool),
}

fn le(camadas: Camadas, arma: &dyn Fn(&mut PainterTool)) -> Leitura {
    let padrao: Vec<f32> = miolo(&dab(pincel(PaintMedia::Digital, Camadas::Off, &|_| {})))
        .iter()
        .map(|p| p.1)
        .collect();
    let um = dab(pincel(PaintMedia::Watercolor, camadas, arma));
    let (mut ts, mut ps) = (Vec::new(), Vec::new());
    for (i, (c, a)) in miolo(&um).iter().enumerate() {
        if *a > 0.05 && c[1] + c[2] > 0.02 {
            ts.push(c[1] / (c[1] + c[2]));
            ps.push(padrao[i]);
        }
    }
    let tr = traco(pincel(PaintMedia::Watercolor, camadas, arma));
    // O controlo: o MESMO parâmetro sem rampa nenhuma — igual ao byte ⇒ a rampa sumiu inteira.
    let off = dab(pincel(PaintMedia::Watercolor, Camadas::Off, arma));
    let off_tr = traco(pincel(PaintMedia::Watercolor, Camadas::Off, arma));
    Leitura {
        rampa: fracao_da_rampa(&um.canvas_rgba),
        corr: correlacao(&ts, &ps),
        traco: fracao_da_rampa(&tr.canvas_rgba),
        igual_off: (
            um.canvas_rgba == off.canvas_rgba,
            tr.canvas_rgba == off_tr.canvas_rgba,
        ),
    }
}

impl std::fmt::Display for Leitura {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = self.corr.map_or("—".to_owned(), |c| format!("{c:.3}"));
        let s = |b: bool| if b { "SIM" } else { "não" };
        write!(
            f,
            "{:.3} | {c} | {:.3} | {} / {}",
            self.rampa,
            self.traco,
            s(self.igual_off.0),
            s(self.igual_off.1)
        )
    }
}

fn set(id: ph2d_a11y::NodeId, v: f64) -> impl Fn(&mut PainterTool) {
    move |t: &mut PainterTool| {
        assert!(
            t.route_brush_watercolor_event(&PanelEvent::SetValue(id, v)),
            "o painel não despachou o id"
        );
    }
}

#[test]
#[ignore = "sonda da lista 26 A8 — corra por nome com --ignored --nocapture"]
fn sonda_a8_que_parametros_da_aguada_apagam_a_rampa() {
    use crate::ids as i;
    let campos: Vec<(&str, ph2d_a11y::NodeId, [f64; 2])> = vec![
        ("Charge", i::PAINTER_WATERCOLOR_CHARGE, [0.5, 0.0]),
        (
            "Charge (0,9 / 0,99)",
            i::PAINTER_WATERCOLOR_CHARGE,
            [0.9, 0.99],
        ),
        ("Dilution", i::PAINTER_WATERCOLOR_DILUTION, [0.5, 1.0]),
        ("Pull", i::PAINTER_WATERCOLOR_PULL, [0.5, 1.0]),
        ("Smudge", i::PAINTER_WATERCOLOR_SMUDGE, [0.5, 1.0]),
        ("Rewet", i::PAINTER_WATERCOLOR_WET, [0.5, 1.0]),
        ("Pigment", i::PAINTER_WATERCOLOR_MIX, [0.5, 1.0]),
        ("Body", i::PAINTER_WATERCOLOR_FILL, [0.0, 1.0]),
        ("Concentration", i::PAINTER_WATERCOLOR_DEPTH, [0.3, 4.0]),
        ("Opacity", i::PAINTER_WATERCOLOR_OPACITY, [0.0, 1.0]),
        ("Edge Darkening", i::PAINTER_WATERCOLOR_EDGE, [0.0, 6.0]),
        ("Bleed", i::PAINTER_WATERCOLOR_SPREAD, [1.0, 30.0]),
        ("Ragged Edge", i::PAINTER_WATERCOLOR_WARP, [0.0, 20.0]),
        ("Paper Edge", i::PAINTER_WATERCOLOR_PAPER_EDGE, [0.5, 1.0]),
        ("Granulation", i::PAINTER_WATERCOLOR_GRANULATION, [0.0, 1.0]),
        ("Flow Size", i::PAINTER_WATERCOLOR_FLOW_SIZE, [0.5, 4.0]),
        ("Flow Angle", i::PAINTER_WATERCOLOR_FLOW_ANGLE, [45.0, 90.0]),
        ("Paper Depth", i::PAINTER_WATERCOLOR_PAPER_DEPTH, [0.0, 2.0]),
        ("Drying Time", i::PAINTER_WATERCOLOR_DRY_TIME, [2.0, 60.0]),
        ("Wet Preview", i::PAINTER_WATERCOLOR_WET_PREVIEW, [0.0, 1.0]),
    ];
    println!(
        "\n| camadas | parâmetro | valor | rampa (dab) | corr matiz×padrão | rampa (traço) | = sem rampa (dab / traço) |"
    );
    println!("|---|---|---|---|---|---|---|");
    for camadas in [Camadas::Textura, Camadas::Queda, Camadas::Off] {
        let l = le(camadas, &|_| {});
        println!("| {camadas:?} | (padrão) | — | {l} |");
    }
    for camadas in [Camadas::Textura, Camadas::Queda] {
        for (nome, id, vs) in &campos {
            if camadas == Camadas::Queda && !nome.starts_with("Charge") && *nome != "Pigment" {
                continue; // a Falloff Color só para a família suspeita
            }
            for v in vs {
                let l = le(camadas, &set(*id, *v));
                println!("| {camadas:?} | {nome} | {v} | {l} |");
            }
        }
    }
    // Combinações: a Charge com cada porta que troca o ramo do depósito (`mistura > 0`).
    for (nome, id, v) in [
        ("Charge 0,5 + Pigment 0,5", i::PAINTER_WATERCOLOR_MIX, 0.5),
        ("Charge 0,5 + Smudge 0,5", i::PAINTER_WATERCOLOR_SMUDGE, 0.5),
        ("Charge 0,5 + Rewet 0,5", i::PAINTER_WATERCOLOR_WET, 0.5),
        ("Charge 0,5 + Pull 1", i::PAINTER_WATERCOLOR_PULL, 1.0),
    ] {
        let arma = |t: &mut PainterTool| {
            set(i::PAINTER_WATERCOLOR_CHARGE, 0.5)(t);
            set(id, v)(t);
        };
        let l = le(Camadas::Textura, &arma);
        println!("| Textura | {nome} | — | {l} |");
    }
    // B&W: o tom sai no peso do carimbo (não no buffer da cor) — a Charge apaga-o também?
    let alfa = |t: &PainterTool| {
        t.canvas_rgba
            .chunks(4)
            .map(|p| u64::from(p[3]))
            .sum::<u64>()
    };
    for (nome, charge) in [("B&W Charge 1", 1.0), ("B&W Charge 0,5", 0.5)] {
        let bw = dab(pincel(
            PaintMedia::Watercolor,
            Camadas::Bw,
            &set(i::PAINTER_WATERCOLOR_CHARGE, charge),
        ));
        let off = dab(pincel(
            PaintMedia::Watercolor,
            Camadas::Off,
            &set(i::PAINTER_WATERCOLOR_CHARGE, charge),
        ));
        println!(
            "| Bw | {nome} | α total {} vs sem rampa {} | igual ao byte: {} |",
            alfa(&bw),
            alfa(&off),
            bw.canvas_rgba == off.canvas_rgba
        );
    }
}

/// **A metade aberta do A8**: sobre tinta que JÁ está no papel (molhada da sessão ou seca), a `Charge`
/// apanha a cor de baixo e mistura-a na cor do dab (`wet_mix_dab_colors`). A rampa troca a cor do dab
/// pela das camadas NO TEXEL — ela ainda leva o que a brocha apanhou? Uma poça amarela à esquerda, um
/// traço vermelho por cima (Noise no Shape) e mede-se, no buffer da cor do traço, `|Δrgb|` médio entre
/// `Charge 0,5` e `Charge 1` (sem apanha) — sobre a poça e, como controlo, sobre o papel em branco.
fn sobre_poca(camadas: Camadas, charge: f64, seca: bool) -> (Vec<u8>, Vec<bool>) {
    let mut t = PainterTool::default();
    t.set_source(vec![0u8; (W * W * 4) as usize], W, W);
    t.set_paint_media(PaintMedia::Watercolor);
    t.paint.brush.radius_px = 22.0;
    t.paint.brush.color = [1.0, 0.85, 0.0];
    let seed = t.paint.brush;
    t.paint.brush_by_mode.fill(seed);
    t = dab_em(t, [26.0, C]);
    if seca {
        t.dry_session_now();
    }
    let poca: Vec<bool> = t.canvas_rgba.chunks(4).map(|p| p[3] > 60).collect();
    let arma = set(crate::ids::PAINTER_WATERCOLOR_CHARGE, charge);
    let mut u = pincel_sobre(t, camadas, &arma);
    u.on_canvas_pointer(cp([4.0, C], PointerPhase::Down));
    for k in 1..=22 {
        #[allow(clippy::cast_precision_loss)]
        let x = 4.0 + 4.0 * k as f32;
        u.on_canvas_pointer(cp([x, C], PointerPhase::Move));
    }
    (u.paint.stroke_color.clone(), poca)
}

fn dab_em(mut t: PainterTool, p: [f32; 2]) -> PainterTool {
    t.on_canvas_pointer(cp(p, PointerPhase::Down));
    t.on_canvas_pointer(cp(p, PointerPhase::Up));
    t
}

/// O pincel do traço (o de [`pincel`]) armado sobre uma ferramenta que já tem tinta.
fn pincel_sobre(
    mut t: PainterTool,
    camadas: Camadas,
    arma: &dyn Fn(&mut PainterTool),
) -> PainterTool {
    let b = &mut t.paint.brush;
    b.radius_px = 14.0;
    b.color = [1.0, 0.0, 0.0];
    b.space_attenuation = false;
    b.watercolor_shape_auto = false;
    (b.falloff, b.hardness) = (Falloff::Constant, 1.0);
    t.set_brush_shape_kind(TextureKind::Noise.to_u8());
    t.paint.brush.shape.mapping = TextureMapping::ViewPlane;
    t.paint.brush.shape.size = [1.5; 2];
    t.set_shape_color_ramp(azul_verde());
    t.set_shape_falloff_ramp(azul_verde());
    t.set_shape_ramp_enabled(camadas != Camadas::Off);
    t.paint.shape_camadas.textura = camadas == Camadas::Textura;
    t.paint.shape_camadas.queda = camadas == Camadas::Queda;
    t.set_shape_falloff_ramp(azul_verde());
    arma(&mut t);
    let seed = t.paint.brush;
    t.paint.brush_by_mode.fill(seed);
    t
}

#[test]
#[ignore = "sonda da lista 26 A8 (metade aberta) — corra por nome com --ignored --nocapture"]
fn sonda_a8_a_rampa_leva_a_cor_apanhada() {
    println!("\n| camadas | poça | Δrgb poça (0,5 vs 1) | Δrgb papel | n poça | n papel |");
    println!("|---|---|---|---|---|---|");
    for seca in [false, true] {
        for camadas in [Camadas::Off, Camadas::Textura, Camadas::Queda] {
            let (cheia, poca) = sobre_poca(camadas, 1.0, seca);
            let (meia, _) = sobre_poca(camadas, 0.5, seca);
            let (mut s, mut n) = ([0.0f64; 2], [0u32; 2]);
            for (i, (a, b)) in cheia.chunks(4).zip(meia.chunks(4)).enumerate() {
                if a[3] < 13 || b[3] < 13 {
                    continue;
                }
                let d: f64 = (0..3)
                    .map(|c| (f64::from(a[c]) - f64::from(b[c])).abs())
                    .sum::<f64>()
                    / 3.0;
                let k = usize::from(!poca[i]);
                s[k] += d;
                n[k] += 1;
            }
            println!(
                "| {camadas:?} | {} | {:.1} | {:.1} | {} | {} |",
                if seca { "seca" } else { "molhada" },
                s[0] / f64::from(n[0].max(1)),
                s[1] / f64::from(n[1].max(1)),
                n[0],
                n[1]
            );
        }
    }
}
