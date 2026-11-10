//! SONDA J13 (FPS do Real Oil, 02/11): a cena de `ph2d-app-painter/src/wetpaint_fps.rs` sem janela — 1024², seis
//! rodadas (Dabber, seca, Wet Drip, corre) — com o custo de cada composição partido (campo · pigmento · sombra/grão ·
//! laço de píxel) e QUANTO da janela do composite mudou desde a composição anterior: as entradas do campo de altura,
//! o pigmento e a tela. Medição, nunca portão.

use super::agua_sobre_oleo_tests::{preset, relato};
use super::*;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
use ph2d_wet_paint::grid::oleo_celula::OleoCelula;
use ph2d_wet_paint::grid::tinta::CelulaTinta;
use ph2d_wet_paint::material::MaterialCelula;
use std::cell::RefCell;

const LADO: usize = 1024;
const HZ: f64 = 250.0;
const DT_MS: f64 = 1000.0 / 60.0;
const BLOCO: usize = 32;
/// Uma fase da cena: nome, duração (s) e o pincel a carregar (preset, cor) ou nenhum.
type FaseDaCena = (&'static str, f64, Option<(&'static str, [u8; 3])>);
const FASES: [FaseDaCena; 4] = [
    (
        "dabber",
        2.5,
        Some(("Oil/Dabber/Dabber.ph2d-brush.png", [40, 90, 220])),
    ),
    ("seca", 2.0, None),
    (
        "wet drip",
        2.5,
        Some(("Watercolor/Fluid/Wet Drip.ph2d-brush.png", [230, 200, 30])),
    ),
    ("corre", 5.0, None),
];

/// As grades da última composição, por camada (só as entradas do `CampoAltura::de_com`).
#[derive(Default)]
struct Grades {
    f: [Vec<f32>; 7],
    rgb: [Vec<[f32; 3]>; 2],
    material: Vec<MaterialCelula>,
    oleo: Vec<OleoCelula>,
    tinta: Vec<CelulaTinta>,
}

#[derive(Default, Clone, Copy)]
struct Acum {
    n: f64,
    cel: f64,
    a_rel: f64,
    a_bit: f64,
    b: f64,
    px: f64,
    c: f64,
    blocos: f64,
    blocos_a: f64,
    blocos_b: f64,
}

#[derive(Default)]
struct Estado {
    grades: Vec<Grades>,
    pigment: Vec<u8>,
    canvas: Vec<u8>,
    acum: Acum,
}

thread_local! {
    static ESTADO: RefCell<Option<Estado>> = const { RefCell::new(None) };
}

/// `(rel, bit)`: mudou além de `1e-4` relativo · mudou um bit.
fn muda_f(a: f32, b: f32) -> (bool, bool) {
    (
        (a - b).abs() > 1e-4 * a.abs().max(b.abs()),
        a.to_bits() != b.to_bits(),
    )
}

/// Compara `cur` com `prev` na janela, marca `rel`/`bit` e copia as linhas da janela para `prev`.
fn campo<T: Clone>(
    cur: &[T],
    prev: &mut Vec<T>,
    (x0, y0, x1, y1, s): (usize, usize, usize, usize, usize),
    (rel, bit): (&mut [bool], &mut [bool]),
    muda: impl Fn(&T, &T) -> (bool, bool),
) {
    let ww = x1 + 1 - x0;
    if prev.len() != cur.len() {
        if !cur.is_empty() || !prev.is_empty() {
            rel.iter_mut().chain(bit.iter_mut()).for_each(|m| *m = true);
        }
        *prev = cur.to_vec();
        return;
    }
    if cur.is_empty() {
        return;
    }
    for y in y0..=y1 {
        for x in x0..=x1 {
            let i = x + y * s;
            let (r, b) = muda(&cur[i], &prev[i]);
            let k = (y - y0) * ww + (x - x0);
            rel[k] |= r;
            bit[k] |= b;
        }
        prev[x0 + y * s..=x1 + y * s].clone_from_slice(&cur[x0 + y * s..=x1 + y * s]);
    }
}

/// O gancho do fim de [`PainterTool::wetpaint_composite_veiled`] (só sob `cfg(test)`; inerte sem a sonda ligada).
pub(super) fn gancho(
    t: &PainterTool,
    (cx0, cy0, cx1, cy1): (usize, usize, usize, usize),
    (px0, py0, px1, py1): (usize, usize, usize, usize),
) {
    ESTADO.with(|e| {
        let mut e = e.borrow_mut();
        let Some(e) = e.as_mut() else {
            return;
        };
        let Some(sess) = t.paint.wetpaint.session.as_ref() else {
            return;
        };
        let (gw, gh) = sess.grid;
        let primeira = e.pigment.len() != sess.planos.pigment.len();
        let ww = cx1 + 1 - cx0;
        let nj = ww * (cy1 + 1 - cy0);
        let (mut rel, mut bit) = (vec![false; nj], vec![false; nj]);
        let layers = &sess.engine.layers;
        e.grades.resize_with(layers.len(), Grades::default);
        for (l, gs) in layers.iter().zip(e.grades.iter_mut()) {
            let g = &l.grid;
            let jan = (cx0, cy0, cx1, cy1, g.s);
            let film: Vec<f32> = if gs.f[6].len() == g.film.len() {
                // Só a janela importa: `film > 0` (molhado) é o que o albedo do campo lê.
                let mut v = gs.f[6].clone();
                for y in cy0..=cy1 {
                    for x in cx0..=cx1 {
                        let i = x + y * g.s;
                        v[i] = f32::from(u8::from(g.film[i] > 0.0));
                    }
                }
                v
            } else {
                g.film
                    .iter()
                    .map(|&w| f32::from(u8::from(w > 0.0)))
                    .collect()
            };
            let fs: [&[f32]; 7] = [
                &g.sett,
                &g.susp,
                &g.altura,
                &g.espessura,
                &g.crista,
                &g.nivelado,
                &film,
            ];
            for (cur, prev) in fs.into_iter().zip(gs.f.iter_mut()) {
                campo(cur, prev, jan, (&mut rel, &mut bit), |a, b| muda_f(*a, *b));
            }
            for (cur, prev) in [&g.sett_rgb, &g.susp_rgb]
                .into_iter()
                .zip(gs.rgb.iter_mut())
            {
                campo(cur, prev, jan, (&mut rel, &mut bit), |a, b| {
                    let m = [0, 1, 2].map(|k| muda_f(a[k], b[k]));
                    (m.iter().any(|x| x.0), m.iter().any(|x| x.1))
                });
            }
            let ex = |a: bool| (a, a);
            campo(
                &g.material.para_vec(),
                &mut gs.material,
                jan,
                (&mut rel, &mut bit),
                |a, b| ex(a != b),
            );
            campo(
                &g.oleo.para_vec(),
                &mut gs.oleo,
                jan,
                (&mut rel, &mut bit),
                |a, b| ex(a != b),
            );
            campo(
                &g.tinta,
                &mut gs.tinta,
                jan,
                (&mut rel, &mut bit),
                |a, b| ex(a != b),
            );
        }
        // (b) o pigmento RGBA da grade, (c) a tela.
        let mut pig = vec![false; nj];
        if !primeira {
            for y in cy0..=cy1 {
                for x in cx0..=cx1 {
                    let p = ((y - 1) * gw + (x - 1)) * 4;
                    pig[(y - cy0) * ww + (x - cx0)] =
                        sess.planos.pigment[p..p + 4] != e.pigment[p..p + 4];
                }
            }
        }
        e.pigment.clone_from(&sess.planos.pigment);
        let (w, _) = t.source_size;
        let w = w as usize;
        let cv = t.canvas_rgba.as_ref();
        let mut px_mudou = 0usize;
        if e.canvas.len() == cv.len() {
            for y in py0..py1 {
                let (a, b) = ((y * w + px0) * 4, (y * w + px1) * 4);
                px_mudou += cv[a..b]
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .zip(e.canvas[a..b].as_chunks::<4>().0)
                    .filter(|(p, q)| p != q)
                    .count();
                e.canvas[a..b].copy_from_slice(&cv[a..b]);
            }
        } else {
            e.canvas = cv.to_vec();
        }
        if primeira {
            return;
        }
        // Blocos de 32×32 células na grade (índice de célula − 1).
        let nbx = gw.div_ceil(BLOCO);
        let nby = gh.div_ceil(BLOCO);
        let (mut toca, mut ba, mut bb) = (
            vec![false; nbx * nby],
            vec![false; nbx * nby],
            vec![false; nbx * nby],
        );
        for y in cy0..=cy1 {
            for x in cx0..=cx1 {
                let k = (y - cy0) * ww + (x - cx0);
                let bk = ((y - 1) / BLOCO) * nbx + (x - 1) / BLOCO;
                toca[bk] = true;
                ba[bk] |= rel[k];
                bb[bk] |= pig[k];
            }
        }
        let conta = |v: &[bool]| v.iter().filter(|&&b| b).count() as f64;
        let a = &mut e.acum;
        a.n += 1.0;
        a.cel += nj as f64;
        a.a_rel += conta(&rel);
        a.a_bit += conta(&bit);
        a.b += conta(&pig);
        a.px += ((px1 - px0) * (py1 - py0)) as f64;
        a.c += px_mudou as f64;
        a.blocos += conta(&toca);
        a.blocos_a += conta(&ba);
        a.blocos_b += conta(&bb);
    });
}

fn ponto(x: f32, y: f32, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos: [x, y],
        pressure: 0.7,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// O que se drena dos cronómetros do composite numa fase.
#[derive(Default, Clone, Copy)]
struct Fase {
    tiques: f64,
    tique_ms: f64,
    chamadas: f64,
    motor: f64,
    render: f64,
    campo: f64,
    pig: f64,
    som: f64,
    engine: f64,
    pixel: f64,
    refeitos: f64,
    mud: Acum,
}

fn drena() -> Fase {
    let (n, m, r) = super::composite_contagem::take();
    let (pig, som) = super::composite_contagem::take_tempos();
    let campo = super::composite_contagem::take_campo();
    let (engine, pixel) = super::composite::split::take();
    let refeitos = super::oleo_plano::sem_cache_do_campo::take_calculados() as f64;
    let mud = ESTADO.with(|e| {
        e.borrow_mut()
            .as_mut()
            .map(|e| std::mem::take(&mut e.acum))
            .unwrap_or_default()
    });
    Fase {
        chamadas: n as f64,
        motor: m as f64,
        render: r as f64,
        campo,
        pig,
        som,
        engine,
        pixel,
        refeitos,
        mud,
        ..Fase::default()
    }
}

#[test]
#[ignore = "sonda J13"]
fn diag_j13_fps_real_oil_o_que_muda() {
    use std::time::Instant;
    ESTADO.with(|e| *e.borrow_mut() = Some(Estado::default()));
    campo_bit::liga();
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; LADO * LADO * 4], LADO as u32, LADO as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    let lado = LADO as f32;
    let _ = drena();
    relato(format!("[j13] tela {LADO}², real oil, {} rodadas", 6));
    for r in 0..6 {
        let y = lado * (0.9 - 0.8 * r as f32 / 6.0);
        for (fi, &(nome, dur, pincel)) in FASES.iter().enumerate() {
            if let Some((rel, cor)) = pincel {
                t.preset_aplicar(&preset(rel)).expect("o preset");
                if fi == 0 {
                    t.set_wet_oleo_real_ligado(true);
                }
                t.set_brush_size_px(30.0);
                t.set_brush_color_srgb8(cor);
            }
            let x = |k: usize| lado * 0.1 + lado * 0.8 * ((k as f64 / HZ) / dur).min(1.0) as f32;
            let _ = drena();
            let ticks = (dur * 1000.0 / DT_MS).round() as usize;
            let (mut eventos, mut tique_ms) = (0usize, 0.0);
            for j in 0..ticks {
                if pincel.is_some() {
                    let alvo = ((j as f64 * DT_MS / 1000.0 * HZ) as usize).min((dur * HZ) as usize);
                    if eventos == 0 && alvo > 0 {
                        t.on_canvas_pointer(ponto(x(0), y, PointerPhase::Down));
                    }
                    while eventos < alvo {
                        eventos += 1;
                        t.on_canvas_pointer(ponto(x(eventos), y, PointerPhase::Move));
                    }
                }
                let t0 = Instant::now();
                t.on_tick(DT_MS as f32);
                let _ = t.take_preview_arc();
                tique_ms += t0.elapsed().as_secs_f64() * 1e3;
            }
            if pincel.is_some() {
                t.on_canvas_pointer(ponto(x(eventos), y, PointerPhase::Up));
            }
            let mut f = drena();
            f.tiques = ticks as f64;
            f.tique_ms = tique_ms;
            let n = f.chamadas.max(1.0);
            let m = f.mud;
            let mn = m.n.max(1.0);
            let resto_motor = f.engine - f.campo - f.pig - f.som;
            relato(format!(
                "[j13] r{r} {nome:<8}| tique {:.1} ms | comp/tique {:.2} | ms/comp: campo {:.2} pig {:.2} som+grão {:.2} \
                 resto-motor {:.2} píxel {:.2} total {:.2} | campo refeito {:.0}/{:.0} | cél motor {:.0} janela {:.0} | \
                 mudou (n={:.0}): entradas rel {:.0} ({:.1}%) bit {:.0} ({:.1}%) · pigmento {:.0} ({:.1}%) · tela {:.0}/{:.0} px \
                 ({:.1}%) | blocos32 {:.1}: c/entradas {:.1} ({:.1}%) c/pigmento {:.1} ({:.1}%)",
                f.tique_ms / f.tiques,
                f.chamadas / f.tiques,
                f.campo / n,
                f.pig / n,
                f.som / n,
                resto_motor / n,
                f.pixel / n,
                (f.engine + f.pixel) / n,
                f.refeitos,
                f.chamadas,
                f.motor / n,
                f.render / n,
                m.n,
                m.a_rel / mn,
                100.0 * m.a_rel / m.cel.max(1.0),
                m.a_bit / mn,
                100.0 * m.a_bit / m.cel.max(1.0),
                m.b / mn,
                100.0 * m.b / m.cel.max(1.0),
                m.c / mn,
                m.px / mn,
                100.0 * m.c / m.px.max(1.0),
                m.blocos / mn,
                m.blocos_a / mn,
                100.0 * m.blocos_a / m.blocos.max(1.0),
                m.blocos_b / mn,
                100.0 * m.blocos_b / m.blocos.max(1.0),
            ));
            relato(campo_bit::relato_da_fase(r, nome));
        }
    }
    ESTADO.with(|e| *e.borrow_mut() = None);
    campo_bit::desliga();
}
pub(super) mod campo_bit;
