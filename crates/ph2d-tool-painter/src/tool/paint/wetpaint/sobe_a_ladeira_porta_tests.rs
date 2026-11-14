//! SONDA J9 §2 — por que PORTA a água do Wet Flat entra no miolo do óleo (`a_agua_nao_sobe_a_ladeira_do_oleo`, a
//! 4 px: 56 012). O campo de fluxo foi refutado na J8 (o dique). Aqui: quando entra (durante o traço × depois), em
//! que forma (`susp` × `sett`), o perfil por linha a partir da orla, e o que muda com a ladeira, o tamanho do pincel
//! e a folga.

use super::agua_sobre_oleo_tests::{com_oleo, ponteiro, preset, relato};
use super::*;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};

fn grade(t: &mut PainterTool) -> (usize, Vec<f32>, Vec<f32>, Vec<f32>) {
    t.wet_bring_home();
    let g = &t
        .paint
        .wetpaint
        .session
        .as_ref()
        .expect("a sessão")
        .engine
        .layers[0]
        .grid;
    (g.s, g.susp.clone(), g.sett.clone(), g.film.clone())
}

/// `(massa nova no miolo ≥ 4 px, a parte em susp, a parte em sett, filme no miolo, perfil por linha 0..16 da orla)`.
fn le(
    t: &mut PainterTool,
    antes: &[f32],
    orla: &[Option<usize>],
) -> (f64, f64, f64, f64, Vec<f64>) {
    let (gs, susp, sett, film) = grade(t);
    let (sa, se) = antes_partes(antes);
    let _ = (sa, se);
    let (mut m, mut ms, mut mt, mut f) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
    let mut perfil = vec![0.0f64; 16];
    for (k, o) in orla.iter().enumerate() {
        let Some(o) = *o else { continue };
        let cx = k + 1;
        for cy in o..o + 40 {
            let i = cx + cy * gs;
            if i >= antes.len() / 3 || antes[i] <= 1000.0 {
                continue;
            }
            let nova = f64::from(susp[i] + sett[i] - antes[i]).max(0.0);
            if cy - o < 16 {
                perfil[cy - o] += nova;
            }
            if cy >= o + 4 {
                m += nova;
                ms += f64::from(susp[i] - antes[i + antes.len() / 3]).max(0.0);
                mt += f64::from(sett[i] - antes[i + 2 * antes.len() / 3]).max(0.0);
                f += f64::from(film[i]);
            }
        }
    }
    (m, ms, mt, f, perfil)
}

fn antes_partes(a: &[f32]) -> (&[f32], &[f32]) {
    let n = a.len() / 3;
    (&a[n..2 * n], &a[2 * n..])
}

fn cena(folga: f32, tamanho: f32, nome: &str) {
    let mut t = com_oleo();
    let (gs, susp, sett, _) = grade(&mut t);
    let n = susp.len();
    // `antes` = [massa | susp | sett].
    let mut antes: Vec<f32> = (0..n).map(|i| susp[i] + sett[i]).collect();
    antes.extend_from_slice(&susp);
    antes.extend_from_slice(&sett);
    let (gw, gh) = t.paint.wetpaint.session.as_ref().expect("a sessão").grid;
    let orla: Vec<Option<usize>> = (1..=gw)
        .map(|cx| (1..=gh).find(|&cy| antes[cx + cy * gs] > 1000.0))
        .collect();
    let topo = orla[120].expect("o óleo") as f32 - 1.0;
    t.preset_aplicar(&preset("Watercolor/Fluid/Wet Flat.ph2d-brush.png"))
        .expect("o meio");
    t.set_brush_size_px(tamanho);
    t.set_brush_color_srgb8([240, 130, 60]);
    let y = topo - tamanho / 2.0 - folga;
    t.on_canvas_pointer(ponteiro(40.0, y, PointerPhase::Down));
    let mut linhas = Vec::new();
    for k in 1..=40 {
        t.on_canvas_pointer(ponteiro(40.0 + 4.0 * k as f32, y, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
        if k % 10 == 0 {
            linhas.push((format!("traço k{k}"), le(&mut t, &antes, &orla)));
        }
    }
    t.on_canvas_pointer(ponteiro(200.0, y, PointerPhase::Up));
    linhas.push(("ao soltar".into(), le(&mut t, &antes, &orla)));
    for k in 1..=600 {
        t.on_tick(1000.0 / 60.0);
        if [30, 120, 600].contains(&k) {
            linhas.push((format!("+{k} tiques"), le(&mut t, &antes, &orla)));
        }
    }
    for (q, (m, ms, mt, f, p)) in linhas {
        let p: Vec<String> = p.iter().map(|x| format!("{x:.0}")).collect();
        relato(format!(
            "{nome} | {q}: miolo {m:.0} (susp {ms:.0} · sett {mt:.0}) filme {f:.1} | perfil {}",
            p.join(" ")
        ));
    }
}

/// A pegada do mesmo traço no papel LIMPO: a massa por linha relativa ao centro do pincel (`dy` de −20 a +20).
fn pegada(tamanho: f32, simula: bool) {
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; 240 * 160 * 4], 240, 160);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.preset_aplicar(&preset("Watercolor/Fluid/Wet Flat.ph2d-brush.png"))
        .expect("o meio");
    t.set_brush_size_px(tamanho);
    t.set_brush_color_srgb8([240, 130, 60]);
    let y = 80.0f32;
    t.on_canvas_pointer(ponteiro(40.0, y, PointerPhase::Down));
    for k in 1..=40 {
        t.on_canvas_pointer(ponteiro(40.0 + 4.0 * k as f32, y, PointerPhase::Move));
        if simula {
            t.on_tick(1000.0 / 60.0);
        }
    }
    t.on_canvas_pointer(ponteiro(200.0, y, PointerPhase::Up));
    let (gs, susp, sett, film) = grade(&mut t);
    let (gw, gh) = t.paint.wetpaint.session.as_ref().expect("a sessão").grid;
    let cy = (y as usize * gh) / 160;
    let mut l = Vec::new();
    for dy in -20i32..=20 {
        let yy = (cy as i32 + dy) as usize;
        let (mut m, mut f) = (0.0f64, 0.0f64);
        for x in 1..=gw {
            let i = x + yy * gs;
            m += f64::from(susp[i] + sett[i]);
            f += f64::from(film[i]);
        }
        l.push(format!("{dy}:{m:.0}/{f:.1}"));
    }
    relato(format!(
        "pegada {tamanho} px simula {simula} (grade {gw}×{gh}) | {}",
        l.join(" ")
    ));
}

#[test]
#[ignore = "sonda J9"]
fn diag_a_pegada_do_wet_flat() {
    pegada(20.0, true);
    pegada(20.0, false);
    pegada(12.0, true);
}

#[test]
#[ignore = "sonda J9"]
fn diag_por_que_porta_a_agua_sobe() {
    use ph2d_wet_paint::solver::LADEIRA_ROTA;
    use ph2d_wet_paint::solver::regua::Regua;
    cena(4.0, 20.0, "produto 4 px");
    cena(8.0, 20.0, "folga 8 px");
    cena(4.0, 12.0, "pincel 12 px");
    let _r = Regua::poe(&LADEIRA_ROTA, 2);
    cena(4.0, 20.0, "sem ladeira 4 px");
}
