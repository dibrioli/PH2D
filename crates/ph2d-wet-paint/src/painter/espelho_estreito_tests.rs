//! B11.2: a cópia ESTREITA (caixa recortada em X, só a camada activa, o anel em faixas contíguas) dá o espelho que a
//! de LINHAS INTEIRAS dava, e copia menos. O caminho antigo vive aqui verbatim, como oráculo.

use super::super::{copia_faixas, escalares, so_a_otica};
use super::*;

/// O `copia_para` de antes do B11.2, verbatim (todas as camadas, as linhas inteiras do `Rect`, ~2·rows faixas do anel).
fn copia_para_antiga(m: &mut Engine, e: &mut Espelho) {
    let sujo = m.take_espelho_sujo();
    let mesma_forma = e.camadas.len() == m.layers.len()
        && e.camadas.iter().zip(&m.layers).all(|(c, l)| {
            (c.grid.w, c.grid.h, c.grid.s, c.grid.rows)
                == (l.grid.w, l.grid.h, l.grid.s, l.grid.rows)
        });
    if !mesma_forma {
        e.camadas = m
            .layers
            .iter()
            .map(|l| CamadaEspelho {
                grid: so_a_otica(&l.grid),
                opacity: l.opacity,
                visible: l.visible,
            })
            .collect();
        return;
    }
    for (c, l) in e.camadas.iter_mut().zip(&m.layers) {
        c.opacity = l.opacity;
        c.visible = l.visible;
        escalares(&mut c.grid, &l.grid);
        let g = &l.grid;
        let faixas = match sujo {
            Dirty::Clean => Vec::new(),
            Dirty::Full => std::iter::once(0..usize::MAX).collect(),
            Dirty::Rect { y0, y1, .. } => {
                let a = (y0.max(1) as usize) - 1;
                let b = (y1.max(0) as usize + 1).min(g.rows - 1);
                let mut f: Vec<_> = std::iter::once(a * g.s..(b + 1) * g.s).collect();
                for y in [0, 1, g.h, g.h + 1].into_iter().filter(|&y| y < g.rows) {
                    f.push(y * g.s..(y + 1) * g.s);
                }
                for y in 0..g.rows {
                    f.extend([y * g.s..y * g.s + 2, y * g.s + g.w..y * g.s + g.w + 2]);
                }
                f
            }
        };
        copia_faixas(&mut c.grid, g, &faixas);
    }
}

/// A cada acção da cena do `o_espelho_incremental_e_a_copia_total_ao_bit` (3 camadas, troca de camada, undo, Wet
/// canvas, pente), o MESMO sujo copiado pelos dois caminhos: planos iguais ao bit (fora da orla virgem, que o antigo
/// só levava quando caía nas linhas do sujo), o estreito igual à grade e a imagem do Real Oil dele igual à da grade.
#[test]
fn a_copia_estreita_e_a_de_linhas_inteiras_ao_bit() {
    let mut e = motor();
    let mut r = Lcg(0x5EED_0B11_0002);
    let (mut nova, mut antiga) = (Espelho::default(), Espelho::default());
    e.copia_para(&mut nova);
    e.mark_dirty_full();
    copia_para_antiga(&mut e, &mut antiga);
    let (mut rects, mut camadas_max) = (0, 0);
    for passo in 0..120 {
        match r.f(0.0, 11.0) as u32 {
            0..=3 => {
                e.sim.oleo = r.f(0.0, 1.0) < 0.6;
                e.sliders.water = r.f(0.0, 1.0);
                traco(&mut e, &mut r);
            }
            4..=6 => {
                for _ in 0..(r.f(1.0, 12.0) as usize) {
                    e.step_simulation();
                }
            }
            7 => {
                if e.can_undo() {
                    e.undo();
                }
            }
            8 => {
                if e.layers.len() >= 3 || !e.add_layer() {
                    e.select_layer(r.f(0.0, e.layers.len() as f64) as usize);
                }
            }
            9 => e.action_wet_canvas(),
            _ => {
                let (x, y) = (r.f(8.0, W as f64 - 8.0), r.f(8.0, H as f64 - 8.0));
                e.pentear_relevo(
                    x,
                    y,
                    [5.0, 3.0],
                    &|_, _| 1.0,
                    &|cx, _| f64::from(cx % 3) * 0.3,
                    4.0,
                );
            }
        }
        let sujo = e.espelho_sujo;
        rects += usize::from(matches!(sujo, Dirty::Rect { .. }));
        camadas_max = camadas_max.max(e.layers.len());
        e.copia_para(&mut nova);
        e.espelho_sujo = sujo;
        copia_para_antiga(&mut e, &mut antiga);
        for (k, l) in e.layers.iter().enumerate() {
            let (n, a) = (&nova.camadas[k].grid, &antiga.camadas[k].grid);
            assert_eq!(
                difere(n, a, &l.grid),
                None,
                "passo {passo}, camada {k}: estreita ≠ antiga"
            );
            assert_eq!(
                difere(n, &l.grid, &l.grid),
                None,
                "passo {passo}, camada {k}: estreita ≠ grade"
            );
        }
        assert_eq!(
            imagem(&e, &nova.camadas_render()),
            imagem(&e, &camadas_do_motor(&e)),
            "passo {passo}: a imagem do espelho estreito ≠ a da grade"
        );
    }
    eprintln!("rects {rects} · camadas {camadas_max}");
    assert!(
        rects >= 30 && camadas_max == 3,
        "rects {rects} camadas {camadas_max}"
    );
}

/// O custo da cópia de um `Rect` de aguada a 4K (3 840 × 2 160, 3 camadas, o sujo de uma caixa de 400 × 900 células):
/// o caminho antigo e o estreito INTERCALADOS no mesmo processo, o mínimo de 7. Medida: `--ignored`.
#[test]
#[ignore = "medida"]
fn mede_copia_estreita_4k() {
    let (w, h) = (3840usize, 2160usize);
    let mut e = Engine::new(w, h);
    e.sim.set_tinta(LeiDaTinta::Pigmento);
    e.sim.oleo = true;
    e.sliders.size = 1.0;
    e.add_layer();
    e.add_layer();
    e.select_layer(1);
    for k in 0..6 {
        let y = 700.0 + 120.0 * f64::from(k);
        e.pointer_down(1500.0, y, None);
        let mut x = 1500.0;
        while x < 1850.0 {
            x += 6.0;
            e.dispatch_pressure_dab(x, y, 60.0, 40.0);
        }
        e.pointer_up();
    }
    let (mut nova, mut antiga) = (Espelho::default(), Espelho::default());
    e.copia_para(&mut nova);
    e.mark_dirty_full();
    copia_para_antiga(&mut e, &mut antiga);
    let rect = Dirty::Rect {
        x0: 1450,
        y0: 600,
        x1: 1850,
        y1: 1500,
    };
    let (mut ms_nova, mut ms_antiga) = (f64::MAX, f64::MAX);
    for _ in 0..7 {
        e.espelho_sujo = rect;
        let t = std::time::Instant::now();
        copia_para_antiga(&mut e, &mut antiga);
        ms_antiga = ms_antiga.min(t.elapsed().as_secs_f64() * 1e3);
        e.espelho_sujo = rect;
        let t = std::time::Instant::now();
        e.copia_para(&mut nova);
        ms_nova = ms_nova.min(t.elapsed().as_secs_f64() * 1e3);
    }
    println!(
        "4K Rect 401×901 células, 3 camadas, {:.1} B/célula: linhas inteiras {ms_antiga:.2} ms · estreita {ms_nova:.2} ms (mín 7, intercalado)",
        nova.bytes() as f64 / (3.0 * e.layers[0].grid.film.len() as f64),
    );
}
