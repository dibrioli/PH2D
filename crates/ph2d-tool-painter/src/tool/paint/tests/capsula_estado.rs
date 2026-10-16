//! ⭐ **O sub-dab da cápsula é o dab INTERPOLADO** (prompt 40, lista 26 A11): o traço a 0,25 com o
//! Continuous contra o mesmo traço a 0,025 sem ele, com o pincel do smoke do dono (Voronoi + Shape Color),
//! no Digital (a tela) e no Impasto (a tela ACESA) × recto/curva × Follow Off/Rake/Flow × pressão 1/variável.
//! A cápsula levava o raio, a rotação, a pressão e o `arc_len` do dab da FRENTE a todos os sub-dabs.

use super::super::media::PaintMedia;
use super::capsula::{H, W, acesa, relevo, verde_laranja};
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

/// O miolo do traço: os texels que o ALVO pinta (tinta `255 − min(rgb)` acima de 8 na tela crua), longe
/// das pontas. ⚠️ A faixa recta em volta do eixo (`capsula::miolo`) mede só um pedaço de uma curva.
fn miolo_do_alvo(cru: &[u8], r: f32) -> Vec<usize> {
    let (x0, x1) = (
        (X0 + 2.0 * r + 10.0) as usize,
        (X1 - 2.0 * r - 10.0) as usize,
    );
    (0..(W * H) as usize)
        .filter(|&i| {
            let x = i % W as usize;
            let p = &cru[i * 4..i * 4 + 3];
            x >= x0 && x < x1 && 255 - p[0].min(p[1]).min(p[2]) > 8
        })
        .collect()
}

/// Média `|ΔRGB| / 255` nos texels `m`.
fn dif(a: &[u8], b: &[u8], m: &[usize]) -> f32 {
    let s: f32 = m
        .iter()
        .map(|&i| {
            (0..3)
                .map(|k| (f32::from(a[i * 4 + k]) - f32::from(b[i * 4 + k])).abs())
                .sum::<f32>()
                / 3.0
        })
        .sum();
    s / m.len().max(1) as f32 / 255.0
}

/// A tela que o dono vê: acesa no Impasto.
fn mostrada(t: &PainterTool, impasto: bool) -> Vec<u8> {
    let cru = t.canvas_rgba.to_vec();
    if impasto { acesa(t, &cru) } else { cru }
}

/// Contra o traço denso: o Continuous (`com`), o traço sem a caixa (`sem`) e a lei de antes do prompt 40
/// (`velha`: a cápsula só desloca o centro, [`ph2d_painter_brush::capsula::com_so_o_centro`]).
#[derive(Clone, Copy, Debug)]
pub(super) struct Medida {
    pub(super) com: f32,
    pub(super) sem: f32,
    pub(super) velha: f32,
}

pub(super) fn mede_com(c: Caso, ajusta: fn(&mut PainterTool)) -> Medida {
    let alvo = traco_tool(c, 0.025, false, ajusta);
    let m = miolo_do_alvo(&alvo.canvas_rgba, 20.0);
    let alvo = mostrada(&alvo, c.impasto);
    let d = |t: PainterTool| dif(&mostrada(&t, c.impasto), &alvo, &m);
    Medida {
        com: d(traco_tool(c, 0.25, true, ajusta)),
        sem: d(traco_tool(c, 0.25, false, ajusta)),
        velha: d(ph2d_painter_brush::capsula::com_so_o_centro(|| {
            traco_tool(c, 0.25, true, ajusta)
        })),
    }
}

pub(super) fn mede(c: Caso) -> Medida {
    mede_com(c, |_| {})
}

/// A barra, pela medida (15/10, depois da cura, régua do traço inteiro): o pior caso é o Impasto em curva
/// com pressão em `|seno|`, 0,0287 — o traço denso muda de DENSIDADE com o raio e a pressão tem quinas entre
/// dois dabs (em rampa linear, o Impasto recto mede metade). A lei velha nos mesmos casos: Impasto com
/// pressão 0,079–0,090, Flow no Impasto 0,101–0,103 (sem a caixa 0,015–0,043: a caixa PIORAVA), Flow no
/// Digital 0,025–0,040, Rake em curva no Impasto 0,029.
const BARRA: f32 = 0.032;

/// ⭐ A régua por caso: o Continuous fica a [`BARRA`] do denso e a menos de METADE do traço sem a caixa e do
/// traço pela lei velha, onde esses passam da barra (o portão discrimina as duas coisas: a caixa age, e a
/// interpolação age).
#[test]
fn o_sub_dab_da_capsula_e_o_dab_interpolado() {
    let mut falhas = Vec::new();
    for c in casos() {
        let Medida { com, sem, velha } = mede(c);
        println!("{c:?}: com {com:.4} · sem {sem:.4} · velha {velha:.4}");
        let longe = |x: f32| x > BARRA && com > 0.5 * x;
        if com > BARRA || longe(sem) || longe(velha) {
            falhas.push(format!(
                "{c:?}: com {com:.4} · sem {sem:.4} · velha {velha:.4}"
            ));
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
/// Sculpt, Knife: 0 texels medidos) ela esmaece. Por modo e método: inerte ⇔ com = sem, AO BYTE, e muda
/// exactamente onde a lei a lê.
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
        // E QUEM a lê é a lei: só as rotas do carimbo, num método que espaça.
        let le = metodo == M::Space && matches!(modo, "paint" | "eraser" | "mask");
        if muda == inerte || muda != le {
            falhas.push(format!(
                "{modo} {metodo:?}: muda {muda} · esmaecida {inerte}"
            ));
        }
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

/// Os ramos da lei que o pincel do smoke não aperta: a ROTAÇÃO (uma ponta achatada com Rake numa curva, e
/// com a Simetria, onde o estado de trás é espelhado com o dab) e a PRESSÃO NA OPACIDADE (o Strength pela
/// caneta, desligado de fábrica, num pincel ralo que não satura). No Digital, contra o traço denso.
pub(super) fn mede_ramos() -> Vec<(&'static str, Medida)> {
    let curva = Caso {
        impasto: false,
        curva: true,
        follow: Follow::Rake,
        pressao: false,
    };
    let recta = Caso {
        curva: false,
        follow: Follow::Off,
        pressao: true,
        ..curva
    };
    let ramos: [(&str, Caso, fn(&mut PainterTool)); 3] = [
        ("Rake, ponta achatada", curva, |t| {
            t.paint.brush.dab_flatten = 0.8;
        }),
        ("Rake, ponta achatada, Simetria", curva, |t| {
            t.paint.brush.dab_flatten = 0.8;
            t.paint.brush.symmetry.enabled = true;
        }),
        ("pressão na opacidade", recta, |t| {
            t.paint.brush.strength = 0.15;
            t.paint.dynamics.size_pressure = false;
            t.paint.dynamics.strength_pressure = true;
        }),
    ];
    ramos
        .into_iter()
        .map(|(nome, c, f)| (nome, mede_com(c, f)))
        .collect()
}

/// ⭐ Os ramos: a rotação interpolada fica a menos de 70 % da lei velha (medido 0,61 nos dois: 0,021 ×
/// 0,034 e 0,020 × 0,033 com a Simetria) e a [`BARRA`] do denso; a opacidade só pela barra — a União de ~10
/// sub-dabs quase não a vê ponta a ponta (0,0080 × 0,0083), e o portão dela é o do leitor
/// (`capsula_tests::o_sub_dab_leva_o_estado_interpolado`).
#[test]
fn os_ramos_da_capsula_giram_e_apertam_com_o_dab() {
    let mut falhas = Vec::new();
    for (n, m) in mede_ramos() {
        println!("{n}: {m:?}");
        let gira = n.starts_with("Rake");
        if m.com > BARRA || (gira && m.com > 0.7 * m.velha) {
            falhas.push(format!("{n}: {m:?}"));
        }
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}
