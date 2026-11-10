//! Sonda do MECANISMO do B14.1 (filho de [`super`]): seis rodadas de óleo + traço molhado numa folha de 1440² células
//! (2880² px a ratio 2, a cena do FPS). A cada passo da sim: a janela do campo pela caixa que o motor entrega
//! (`take_dirty`, a do chamador) × a caixa da grade (água, tinta) e a do campo que MUDARAM de facto. Medida 10/11 no
//! BACKLOG (B14.1): a água desta cena seca em 2 passos — falta-lhe a água viva do Wet Drip do app.
//! `bash scripts/cargo-test-narrow.sh ph2d-wet-paint sonda_a_caixa_do_passo_e_a_que_mudou --profile smoke -- --ignored --nocapture`

use super::oleo_campo_sonda::Faixas;
use super::*;
use crate::painter::{Dirty, Engine};
use crate::sim::LeiDaTinta;
use crate::tuning::Knob;

const LADO: usize = 1440;

/// A caixa de células `1..=LADO` de um [`Dirty`] (`None` = limpo).
fn caixa(d: Dirty) -> Option<(usize, usize, usize, usize)> {
    let p = |v: i32| usize::try_from(v.max(1)).unwrap_or(1).min(LADO);
    match d {
        Dirty::Clean => None,
        Dirty::Full => Some((1, 1, LADO, LADO)),
        Dirty::Rect { x0, y0, x1, y1 } => Some((p(x0), p(y0), p(x1), p(y1))),
    }
}

/// A caixa das células cujo `v`, `detalhe` ou `sombra` difere ao bit entre dois campos da folha inteira.
fn caixa_mudada(a: &CampoAltura, b: &CampoAltura) -> Option<(usize, usize, usize, usize)> {
    let mut r: Option<(usize, usize, usize, usize)> = None;
    for y in 0..a.h {
        for x in 0..a.w {
            let i = y * a.w + x;
            let igual = a.v[i].to_bits() == b.v[i].to_bits()
                && a.detalhe[i].to_bits() == b.detalhe[i].to_bits()
                && a.sombra[i].to_bits() == b.sombra[i].to_bits();
            if !igual {
                let (cx, cy) = (x + a.x0, y + a.y0);
                r = Some(r.map_or((cx, cy, cx, cy), |(x0, y0, x1, y1)| {
                    (x0.min(cx), y0.min(cy), x1.max(cx), y1.max(cy))
                }));
            }
        }
    }
    r
}

fn area((x0, y0, x1, y1): (usize, usize, usize, usize)) -> usize {
    (x1 + 1 - x0) * (y1 + 1 - y0)
}

/// A caixa das células cuja água ou tinta (`film`, `susp`, `sett` e as cores) difere ao bit entre duas grades.
fn caixa_da_grade_mudada(a: &Grid, b: &Grid) -> Option<(usize, usize, usize, usize)> {
    let mut r: Option<(usize, usize, usize, usize)> = None;
    for y in 1..=a.h {
        for x in 1..=a.w {
            let i = x + y * a.s;
            let f = |u: &[f32], v: &[f32]| u[i].to_bits() == v[i].to_bits();
            let c =
                |u: &[[f32; 3]], v: &[[f32; 3]]| u[i].map(f32::to_bits) == v[i].map(f32::to_bits);
            let igual = f(&a.film, &b.film)
                && f(&a.susp, &b.susp)
                && f(&a.sett, &b.sett)
                && c(&a.susp_rgb, &b.susp_rgb)
                && c(&a.sett_rgb, &b.sett_rgb);
            if !igual {
                r = Some(r.map_or((x, y, x, y), |(x0, y0, x1, y1)| {
                    (x0.min(x), y0.min(y), x1.max(x), y1.max(y))
                }));
            }
        }
    }
    r
}

fn risca(e: &mut Engine, y: f64) {
    let mut x = 100.0;
    e.pointer_down(x, y, None);
    while x < 1340.0 {
        x += 20.0;
        e.dispatch_pressure_dab(x, y, 1.0, 50.0);
    }
    e.pointer_up();
    while e.release_frame(x, y) {}
}

/// A cena do FPS (`wetpaint_fps.rs`): por rodada um traço de óleo, um traço MUITO molhado por cima (o Wet Drip) e
/// a água a correr; seis rodadas em faixas diferentes da folha.
#[test]
#[ignore = "sonda de custo (smoke): a caixa do passo × a caixa que mudou, 6 rodadas a 2880²"]
fn sonda_a_caixa_do_passo_e_a_que_mudou() {
    let mut e = Engine::new(LADO, LADO);
    e.sim.set_tinta(LeiDaTinta::Pigmento);
    let receita = crate::painter::receita::migra_para_controlos;
    let (oleo_d, oleo_v) = receita(3000.0, 1.0, 0.0, 10.0, 1.0);
    let (agua_d, agua_v) = receita(300.0, 1.0, 0.0, 10.0, 1.0);
    e.sliders.size = 1.0;
    let o = OleoReal::default();
    let folha = (1, 1, LADO, LADO);
    let (mut ras_folha, mut ras_caixa) = (RascunhoCampo::default(), RascunhoCampo::default());
    let campo_da_folha = |e: &Engine, ras: &mut RascunhoCampo| {
        let camadas: Vec<_> = e.layers.iter().map(|l| (&l.grid, l.visible)).collect();
        CampoAltura::de_com_em(ras, &Faixas, &camadas, &o, 1, folha)
    };
    for rodada in 0..6 {
        let y = 120.0 + 220.0 * f64::from(rodada);
        e.sim.oleo = true;
        e.sliders.water = 0.2;
        e.set_knob(Knob::Dilution, oleo_d);
        e.set_knob(Knob::Volume, oleo_v);
        risca(&mut e, y);
        e.sim.oleo = false;
        e.sliders.water = 1.0;
        e.set_knob(Knob::Dilution, agua_d);
        e.set_knob(Knob::Volume, agua_v);
        risca(&mut e, y);
        let _ = e.take_dirty();
        let mut campo_antes = campo_da_folha(&e, &mut ras_folha);
        let (mut soma_sujo, mut soma_grade, mut soma_campo, mut ms_max, mut passos) =
            (0, 0, 0, 0.0f64, 0);
        for _ in 0..24 {
            let grade_antes = e.layers[e.active_layer].grid.clone();
            e.step_simulation();
            let Some(sujo) = caixa(e.take_dirty()) else {
                continue;
            };
            passos += 1;
            let camadas: Vec<_> = e.layers.iter().map(|l| (&l.grid, l.visible)).collect();
            let a = alcance_na(&o, &camadas, 1, sujo);
            let janela = (
                sujo.0.saturating_sub(a).max(1),
                sujo.1.saturating_sub(a).max(1),
                (sujo.2 + a).min(LADO),
                (sujo.3 + a).min(LADO),
            );
            let t = std::time::Instant::now();
            let c = CampoAltura::de_com_em(&mut ras_caixa, &Faixas, &camadas, &o, 1, janela);
            ms_max = ms_max.max(t.elapsed().as_secs_f64() * 1e3);
            drop(camadas);
            soma_sujo += c.w * c.h;
            soma_grade +=
                caixa_da_grade_mudada(&grade_antes, &e.layers[e.active_layer].grid).map_or(0, area);
            let depois = campo_da_folha(&e, &mut ras_folha);
            soma_campo += caixa_mudada(&campo_antes, &depois).map_or(0, area);
            campo_antes = depois;
        }
        let m = |v: usize| v / passos.max(1);
        eprintln!(
            "rodada {rodada} · {passos} passos sujos · janela do campo {} texels/passo (máx {ms_max:.1} ms) · \
             grade mudou {} cél./passo · campo mudou {} cél./passo",
            m(soma_sujo),
            m(soma_grade),
            m(soma_campo)
        );
    }
}
