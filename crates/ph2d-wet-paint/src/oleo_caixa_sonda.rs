//! Sonda do MECANISMO do B14.1 (filho de [`super`]): seis rodadas de óleo + traço molhado numa folha de 1440² células
//! (2880² px a ratio 2, a cena do FPS), a água VIVA (a do Wet Drip: a mesa inclinada, `Gravity` no tecto). A cada
//! passo da sim: a janela do campo pela união (`take_dirty`, a de antes) × pela caixa da óptica
//! (`take_dirty_pela_otica` contra o espelho, a do `serve`), com a caixa da grade que MUDOU. Medida no BACKLOG (B14.1).
//! `bash scripts/cargo-test-narrow.sh ph2d-wet-paint sonda_a_caixa_do_passo_e_a_que_mudou --profile smoke -- --ignored --nocapture`

use super::oleo_campo_sonda::Faixas;
use super::*;
use crate::painter::espelho::Espelho;
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

/// O campo da óptica na janela de `sujo` (a caixa + o alcance do óleo): `(texels, ms)`; limpo = `(0, 0)`.
fn campo_na(e: &Engine, o: &OleoReal, ras: &mut RascunhoCampo, sujo: Dirty) -> (usize, f64) {
    let Some(s) = caixa(sujo) else {
        return (0, 0.0);
    };
    let camadas: Vec<_> = e.layers.iter().map(|l| (&l.grid, l.visible)).collect();
    let a = alcance_na(o, &camadas, 1, s);
    let janela = (
        s.0.saturating_sub(a).max(1),
        s.1.saturating_sub(a).max(1),
        (s.2 + a).min(LADO),
        (s.3 + a).min(LADO),
    );
    let t = std::time::Instant::now();
    let c = CampoAltura::de_com_em(ras, &Faixas, &camadas, o, 1, janela);
    (c.w * c.h, t.elapsed().as_secs_f64() * 1e3)
}

/// A cena do FPS (`wetpaint_fps.rs`): por rodada um traço de óleo, um traço MUITO molhado por cima (o Wet Drip, a
/// mesa inclinada) e a água a correr; seis rodadas em faixas diferentes da folha.
#[test]
#[ignore = "sonda de custo (smoke): a caixa da união × a da óptica, 6 rodadas a 2880²"]
fn sonda_a_caixa_do_passo_e_a_que_mudou() {
    let mut e = Engine::new(LADO, LADO);
    e.sim.set_tinta(LeiDaTinta::Pigmento);
    e.sim.tilt_on = true;
    let receita = crate::painter::receita::migra_para_controlos;
    let (oleo_d, oleo_v) = receita(3000.0, 1.0, 0.0, 10.0, 1.0);
    let (agua_d, agua_v) = receita(300.0, 1.0, 0.0, 10.0, 1.0);
    e.sliders.size = 1.0;
    let o = OleoReal::default();
    let mut esp = Espelho::default();
    let (mut ras_a, mut ras_b) = (RascunhoCampo::default(), RascunhoCampo::default());
    let (mut tot_uniao, mut tot_otica) = (0usize, 0usize);
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
        e.set_knob(Knob::Gravity, 0.05);
        risca(&mut e, y);
        let _ = e.take_dirty();
        e.copia_para(&mut esp);
        let (mut passos, mut molhados, mut grade) = (0, 0, 0);
        let (mut uniao, mut otica) = ((0usize, 0.0f64, 0.0f64), (0usize, 0.0f64, 0.0f64));
        for k in 0..40 {
            let grade_antes = e.layers[e.active_layer].grid.clone();
            e.step_simulation();
            let d_uniao = e.dirty;
            let d_otica = e.take_dirty_pela_otica(&esp);
            e.copia_para(&mut esp);
            if matches!(d_uniao, Dirty::Clean) {
                continue;
            }
            passos += 1;
            let g = caixa_da_grade_mudada(&grade_antes, &e.layers[e.active_layer].grid);
            molhados += usize::from(g.is_some());
            grade += g.map_or(0, area);
            // Intercaladas: a ordem troca a cada passo.
            let (a, b) = if k % 2 == 0 {
                let a = campo_na(&e, &o, &mut ras_a, d_uniao);
                (a, campo_na(&e, &o, &mut ras_b, d_otica))
            } else {
                let b = campo_na(&e, &o, &mut ras_b, d_otica);
                (campo_na(&e, &o, &mut ras_a, d_uniao), b)
            };
            for (t, (texels, ms)) in [(&mut uniao, a), (&mut otica, b)] {
                t.0 += texels;
                t.1 += ms;
                t.2 = t.2.max(ms);
            }
        }
        tot_uniao += uniao.0;
        tot_otica += otica.0;
        let m = |v: usize| v / passos.max(1);
        eprintln!(
            "rodada {rodada} · {passos} passos sujos ({molhados} com a grade mudada, {} cél./passo) · união {} texels/passo \
             {:.1} ms (máx {:.1}) · óptica {} texels/passo {:.1} ms (máx {:.1})",
            m(grade),
            m(uniao.0),
            uniao.1 / passos.max(1) as f64,
            uniao.2,
            m(otica.0),
            otica.1 / passos.max(1) as f64,
            otica.2
        );
    }
    eprintln!("total: união {tot_uniao} texels · óptica {tot_otica} texels");
}
