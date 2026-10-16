//! ⭐ **O sub-dab da cápsula é o dab INTERPOLADO** (prompt 40, lista 26 A11): o traço a 0,25 com o
//! Continuous contra o mesmo traço a 0,025 sem ele, com o pincel do smoke do dono (Voronoi + Shape Color),
//! no Digital (a tela) e no Impasto (a tela ACESA) × recto/curva × Follow Off/Rake/Flow × pressão 1/variável.
//! A cápsula levava o raio, a rotação, a pressão e o `arc_len` do dab da FRENTE a todos os sub-dabs.

use super::super::media::PaintMedia;
use super::capsula::{H, W, acesa, dif_cor, relevo, verde_laranja};
use super::*;
use ph2d_painter_brush::TextureKind;

const X0: f32 = 60.0;
const X1: f32 = 420.0;
const EVENTOS: usize = 72;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Follow {
    Off,
    Rake,
    Flow,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Caso {
    pub(super) impasto: bool,
    pub(super) curva: bool,
    pub(super) follow: Follow,
    pub(super) pressao: bool,
}

/// O traço de 40 px do caso, a Spacing `sp`, e a tela que o dono vê (acesa no Impasto).
pub(super) fn traco_caso(c: Caso, sp: f32, cont: bool) -> Vec<u8> {
    let t = traco_tool(c, sp, cont, |_| {});
    let cru = t.canvas_rgba.to_vec();
    if c.impasto { acesa(&t, &cru) } else { cru }
}

/// A pressão do caso ao longo do traço (`f` em `[0, 1]`, o evento `k`).
fn pressao_de(c: Caso, k: usize) -> f32 {
    if c.pressao {
        PRESSAO.with(std::cell::Cell::get)(k)
    } else {
        1.0
    }
}

fn pressao_seno(k: usize) -> f32 {
    0.4 + 0.6 * (k as f32 / 9.0).sin().abs()
}

thread_local! {
    /// A lei da pressão variável — as sondas trocam-na.
    pub(super) static PRESSAO: std::cell::Cell<fn(usize) -> f32> = const { std::cell::Cell::new(pressao_seno) };
}

/// O tool depois do traço do caso, com `ajusta` antes do primeiro evento.
pub(super) fn traco_tool(
    c: Caso,
    sp: f32,
    cont: bool,
    ajusta: fn(&mut PainterTool),
) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (W * H * 4) as usize], W, H);
    t.set_paint_media(if c.impasto {
        PaintMedia::Impasto
    } else {
        PaintMedia::Digital
    });
    t.set_brush_color_srgb8([20, 30, 120]);
    t.set_brush_shape_kind(TextureKind::Voronoi.to_u8());
    t.set_brush_size_px(40.0);
    t.set_brush_spacing(sp);
    t.set_brush_shape_follow(c.follow as u8);
    t.set_shape_color_ramp(verde_laranja());
    t.set_shape_ramp_enabled(true);
    t.paint.brush.continuous_spacing = cont;
    ajusta(&mut t);
    let seed = t.paint.brush;
    t.paint.brush_by_mode.fill(seed);
    let ponto = |k: usize, phase| {
        let x = X0 + (X1 - X0) * k as f32 / EVENTOS as f32;
        let y = H as f32 / 2.0
            + if c.curva {
                30.0 * (x / 40.0).sin()
            } else {
                0.0
            };
        let p = pressao_de(c, k);
        CanvasPointer {
            pos: [x, y],
            pressure: p,
            tilt: [0.0, 0.0],
            phase,
        }
    };
    t.on_canvas_pointer(ponto(0, PointerPhase::Down));
    t.on_tick(1000.0 / 60.0);
    for k in 1..=EVENTOS {
        t.on_canvas_pointer(ponto(k, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(ponto(EVENTOS, PointerPhase::Up));
    for _ in 0..30 {
        t.on_tick(1000.0 / 60.0);
    }
    t
}

pub(super) fn casos() -> impl Iterator<Item = Caso> {
    [false, true].into_iter().flat_map(|impasto| {
        [false, true].into_iter().flat_map(move |curva| {
            [Follow::Off, Follow::Rake, Follow::Flow]
                .into_iter()
                .flat_map(move |follow| {
                    [false, true].into_iter().map(move |pressao| Caso {
                        impasto,
                        curva,
                        follow,
                        pressao,
                    })
                })
        })
    })
}

/// `(Δ com a caixa, Δ sem ela)` contra o traço denso, no miolo de raio 20.
pub(super) fn mede(c: Caso) -> (f32, f32) {
    let alvo = traco_caso(c, 0.025, false);
    (
        dif_cor(&traco_caso(c, 0.25, true), &alvo, 20.0),
        dif_cor(&traco_caso(c, 0.25, false), &alvo, 20.0),
    )
}

/// A barra, pela medida (15/10, depois da cura): o pior caso é o Impasto recto com pressão em `|seno|`,
/// 0,0221 — o traço denso muda de DENSIDADE com o raio e a pressão tem quinas entre dois dabs (em rampa
/// linear o mesmo caso mede 0,0106). Antes da cura: Impasto com pressão 0,099 (sem a caixa 0,147); Flow
/// 0,131 (sem 0,000) e 0,028 no Digital; Rake em curva 0,021.
const BARRA: f32 = 0.025;

/// ⭐ A régua por caso: o Continuous fica a [`BARRA`] do denso e a menos de METADE do traço sem a caixa
/// (o portão discrimina; onde o traço sem ela já é o denso — o Flow recto no Impasto — basta a barra).
#[test]
fn o_sub_dab_da_capsula_e_o_dab_interpolado() {
    let mut falhas = Vec::new();
    for c in casos() {
        let (com, sem) = mede(c);
        println!("{c:?}: com {com:.4} · sem {sem:.4}");
        if com > BARRA || (sem > BARRA && com > 0.5 * sem) {
            falhas.push(format!("{c:?}: com {com:.4} · sem {sem:.4}"));
        }
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

/// Um traço de base no Impasto sobre listras e depois, no modo `modo` e no método `metodo`, o traço
/// a 0,25 com ou sem a caixa: a tela, a máscara e o relevo, e se a caixa está ESMAECIDA.
fn no_modo(
    modo: &str,
    metodo: ph2d_painter_brush::StrokeMethod,
    cont: bool,
) -> (Vec<u8>, Vec<f32>, bool) {
    let mut t = PainterTool::default();
    let mut src = vec![255u8; (W * H * 4) as usize];
    for (i, p) in src.chunks_mut(4).enumerate() {
        if (i % W as usize / 12) % 2 == 0 {
            p[..3].copy_from_slice(&[200, 60, 40]);
        }
    }
    t.set_source(src, W, H);
    t.set_paint_media(PaintMedia::Impasto);
    t.set_brush_color_srgb8([20, 30, 120]);
    t.set_brush_shape_kind(TextureKind::Voronoi.to_u8());
    t.set_brush_size_px(30.0);
    t.set_brush_spacing(0.25);
    let seed = t.paint.brush;
    t.paint.brush_by_mode.fill(seed);
    let risca = |t: &mut PainterTool, y0: f32, y1: f32| {
        let ev = |k: usize, phase| CanvasPointer {
            pos: [
                X0 + (X1 - X0) * k as f32 / EVENTOS as f32,
                y0 + (y1 - y0) * k as f32 / EVENTOS as f32,
            ],
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        };
        t.on_canvas_pointer(ev(0, PointerPhase::Down));
        t.on_tick(1000.0 / 60.0);
        for k in 1..=EVENTOS {
            t.on_canvas_pointer(ev(k, PointerPhase::Move));
            t.on_tick(1000.0 / 60.0);
        }
        t.on_canvas_pointer(ev(EVENTOS, PointerPhase::Up));
        for _ in 0..10 {
            t.on_tick(1000.0 / 60.0);
        }
    };
    risca(&mut t, 64.0, 64.0);
    t.set_paint_tool_mode(modo);
    t.paint.brush.continuous_spacing = cont;
    t.paint.brush.spacing = 0.25;
    t.paint.brush.stroke_method = metodo;
    let inerte = t
        .inercias()
        .inerte(super::super::inercia::Dependente::ContinuoSemPasso);
    risca(&mut t, 40.0, 90.0);
    let mut tela = t.canvas_rgba.to_vec();
    tela.extend_from_slice(&t.paint.mask_scratch_rgba);
    (tela, relevo(&t), inerte)
}

/// ⛔ **A caixa esmaecida é a caixa que não muda nada** (prompt 40 #5/#6): o Continuous só o lêem as rotas
/// do carimbo (Paint, a borracha, a Máscara) num método que ESPAÇA; nos métodos sem Spacing (Dots, Airbrush
/// — a caixa nem aparece lá, e agia) e nos modos de kernel próprio (Smear, Blur, Clone, Inpaint, Liquify,
/// Sculpt, Knife: 0 texels medidos) ela esmaece. Por modo e método: inerte ⇔ com = sem, AO BYTE.
#[test]
fn a_caixa_continuous_esmaece_onde_nao_age() {
    use ph2d_painter_brush::StrokeMethod as M;
    let mut casos: Vec<(&str, M)> = [
        "paint", "eraser", "mask", "smear", "blur", "clone", "inpaint", "liquify", "sculpt",
        "knife",
    ]
    .into_iter()
    .map(|m| (m, M::Space))
    .collect();
    casos.extend([
        ("paint", M::Dots),
        ("paint", M::Airbrush),
        ("paint", M::DragDot),
    ]);
    let mut falhas = Vec::new();
    for (modo, metodo) in casos {
        let (ta, ra, _) = no_modo(modo, metodo, false);
        let (tb, rb, inerte) = no_modo(modo, metodo, true);
        let muda = ta != tb || ra.iter().zip(&rb).any(|(a, b)| a.to_bits() != b.to_bits());
        println!("{modo} {metodo:?}: muda {muda} · esmaecida {inerte}");
        if muda == inerte {
            falhas.push(format!(
                "{modo} {metodo:?}: muda {muda} · esmaecida {inerte}"
            ));
        }
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}
