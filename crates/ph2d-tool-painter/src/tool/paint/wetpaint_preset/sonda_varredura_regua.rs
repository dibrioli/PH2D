//! A RÉGUA da varredura F0/E9 (o que se lê de UM render): as distâncias à mistura, as manchas, a ferrugem, os
//! degraus, o render PLANO (seco, sem relevo nem óleo) e a cobertura física que diz «coberto» na lei `Pigmento`.
//! Partida de `sonda_varredura_oleo.rs` (tecto de LOC); doc: `docs/Painter/Arquitetura/E9_portao_mae.md` §7.

use super::*;

/// A distância (níveis) de `x` à curva Kubelka–Munk entre `a` e `b` — a de `assenta/oleo_sobre_oleo_tests.rs`.
pub(super) fn fora_da_mistura(x: [f64; 3], a: [f64; 3], b: [f64; 3]) -> f64 {
    (0..=200)
        .map(|i| {
            let c = f64::from(i) / 200.0;
            (0..3)
                .map(|k| {
                    let ks = (1.0 - c) * ks_of_srgb255(a[k]) + c * ks_of_srgb255(b[k]);
                    (x[k] - srgb255_of_linear(reflectance_of_ks(ks))).powi(2)
                })
                .sum::<f64>()
                .sqrt()
        })
        .fold(f64::INFINITY, f64::min)
}

pub(super) fn lum(c: [f64; 3]) -> f64 {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

/// Matiz (graus) e saturação HSV.
pub(super) fn matiz(c: [f64; 3]) -> (f64, f64) {
    let mx = c[0].max(c[1]).max(c[2]);
    let mn = c[0].min(c[1]).min(c[2]);
    let d = mx - mn;
    if d < 1e-9 || mx < 1e-9 {
        return (0.0, 0.0);
    }
    let h = if (mx - c[0]).abs() < 1e-9 {
        60.0 * ((c[1] - c[2]) / d).rem_euclid(6.0)
    } else if (mx - c[1]).abs() < 1e-9 {
        60.0 * ((c[2] - c[0]) / d + 2.0)
    } else {
        60.0 * ((c[0] - c[1]) / d + 4.0)
    };
    (h, d / mx)
}

/// Ferrugem: matiz fora do arco azul(~220°)–verde–amarelo(~49°) — vermelho, laranja, castanho, magenta —, com
/// saturação que não seja a do papel creme (0,13).
pub(super) fn ferrugem(c: [f64; 3]) -> bool {
    let (h, s) = matiz(c);
    s > 0.2 && (h < 40.0 || h > 250.0)
}

pub(super) fn sobre_creme(q: &[u8]) -> ([f64; 3], f64) {
    let a = f64::from(q[3]) / 255.0;
    (
        [0, 1, 2].map(|k| f64::from(q[k]) * a + CREME[k] * (1.0 - a)),
        a,
    )
}

/// As métricas de UM render (com luz ou plano) sobre o recorte.
#[derive(Clone, Copy, Default)]
pub(in crate::tool::paint::wetpaint_preset) struct Leitura {
    pub km_max: f64,
    pub km_p99: f64,
    /// A distância ao segmento Oklab azul–amarelo (a mistura da lei `Pigmento`), máx e p99.
    pub mix_max: f64,
    pub mix_p99: f64,
    /// A mesma distância na CAMADA FINA (`Pigmento`, cobertura física `[0,9, 0,99)`): a borda, p99 (E9 §7).
    pub orla_p99: f64,
    /// % dos píxeis cobertos (alfa ≥ 0,9) mais escuros que o azul sozinho (p50) − 6 níveis.
    pub manchas: f64,
    /// O mesmo, mas no azul SOZINHO antes do amarelo.
    pub manchas_azul: f64,
    pub ferrugem: f64,
    pub ref_azul: f64,
    /// Degraus de matiz no verde: saltos > 3° entre linhas vizinhas (perfil vertical, x 150..210, através das
    /// passadas) e entre colunas vizinhas (perfil ao longo do eixo do traço, na faixa y 130..146).
    pub degraus_y: usize,
    pub degraus_x: usize,
}

pub(in crate::tool::paint::wetpaint_preset) struct Celula {
    pub luz: Leitura,
    pub plano: Leitura,
    /// % de papel à vista (alfa < 0,9) no miolo do azul (x 100..260, y 118..142).
    pub furos: f64,
    pub rug: f64,
    pub p99_altura: f64,
}

pub(super) fn pct(v: &mut [f64], q: f64) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(f64::total_cmp);
    v[((v.len() - 1) as f64 * q).round() as usize]
}

/// O render PLANO: sem o Real Oil (a luz, o verniz, a sombra da crista), sem relevo e SECO — só a cor da tinta.
/// O óleo, o emboss e a água são da CÉLULA (`Grid::oleo`, `Grid::relevo`/`altura`, `Grid::film`), não da caixa:
/// tiram-se da grade só para este composite e repõem-se logo (o motor que pinta a seguir não muda). E9 §5.2, §7.
/// Seco porque a cor escolhida é o masstone SECO (F4_1 §5): molhada, a aquarela escurece de propósito.
/// Devolve, na lei `Pigmento`, a cobertura FÍSICA por píxel (a da célula, seca): o `α` do píxel é o da decomposição
/// para o compositor e passa de 0,9 numa película de amarelo (E9 §7) — «coberto» lê-se desta.
pub(super) fn render_plano(t: &mut PainterTool) -> Option<Vec<f32>> {
    use ph2d_editor_core::Tool;
    t.set_wet_oleo_real(None);
    t.handle_panel_event(PanelEvent::SetValue(
        crate::ids::wet_tuning_slider_id("emboss"),
        0.0,
    ));
    let guarda = |t: &mut PainterTool| {
        let s = t.paint.wetpaint.session.as_mut()?;
        s.bring_home();
        Some(
            s.engine
                .layers
                .iter_mut()
                .map(|l| {
                    let g = &mut l.grid;
                    let zero = vec![0.0; g.relevo.len()];
                    let seco = vec![0.0; g.film.len()];
                    (
                        std::mem::replace(&mut g.relevo, zero),
                        std::mem::take(&mut g.altura),
                        std::mem::replace(
                            &mut g.oleo,
                            ph2d_wet_paint::grid::oleo_celula::sem_campo(),
                        ),
                        std::mem::replace(&mut g.film, seco),
                        std::mem::take(&mut g.oleo_real),
                    )
                })
                .collect::<Vec<_>>(),
        )
    };
    let guardado = guarda(t);
    t.wet_recomposite_full();
    let s = t.paint.wetpaint.session.as_mut()?;
    let cob = (s.engine.sim.tinta == ph2d_wet_paint::sim::LeiDaTinta::Pigmento).then(|| {
        let (r, g) = (usize::from(s.ratio), &s.engine.layers[0].grid);
        (0..(W * H) as usize)
            .map(|k| {
                let (x, y) = (k % W as usize, k / W as usize);
                ph2d_wet_paint::grid::tinta::cobertura(g, (x / r + 1) + (y / r + 1) * g.s)
            })
            .collect()
    });
    if let Some(v) = guardado {
        for (l, (r, a, o, f, real)) in s.engine.layers.iter_mut().zip(v) {
            (l.grid.relevo, l.grid.altura, l.grid.oleo, l.grid.film) = (r, a, o, f);
            l.grid.oleo_real = real;
        }
    }
    cob
}

/// A cobertura física a partir da qual a régua da MISTURA (`mix_*`) lê o píxel na lei `Pigmento` (E9 §7).
pub(super) const COBRE_A_MISTURA: f64 = 0.99;

/// O «coberto» do píxel `(x, y)`: a cobertura física `cob` (lei `Pigmento`) ou o `α` do píxel (lei velha).
pub(super) fn coberto(cob: Option<&[f32]>, x: usize, y: usize, a: f64) -> f64 {
    cob.map_or(a, |c| f64::from(c[x + y * W as usize]))
}

/// O recorte sobre o creme (o que as réguas lêem), para o PNG.
pub(super) fn recorte(t: &PainterTool) -> Vec<[u8; 3]> {
    let mut v = Vec::with_capacity((RX.1 - RX.0) * (RY.1 - RY.0));
    for y in RY.0..RY.1 {
        for x in RX.0..RX.1 {
            let (c, _) = sobre_creme(&t.canvas_rgba[(x + y * W as usize) * 4..][..4]);
            v.push(c.map(|v| v.round().clamp(0.0, 255.0) as u8));
        }
    }
    v
}

/// Luminância de referência do azul sozinho (p50 dos cobertos) e % de manchas.
pub(super) fn le_azul(t: &PainterTool, cob: Option<&[f32]>) -> (f64, f64) {
    let mut la = vec![];
    for y in RY.0..RY.1 {
        for x in RX.0..RX.1 {
            let (c, a) = sobre_creme(&t.canvas_rgba[(x + y * W as usize) * 4..][..4]);
            if coberto(cob, x, y, a) >= 0.9 {
                la.push(lum(c));
            }
        }
    }
    let r = pct(&mut la.clone(), 0.5);
    let m = if la.is_empty() {
        0.0
    } else {
        100.0 * la.iter().filter(|l| **l < r - 6.0).count() as f64 / la.len() as f64
    };
    (r, m)
}

/// Conta saltos > 3° num perfil de matiz médio (posições sem verde ficam de fora e partem a contagem).
pub(super) fn saltos(perfil: &[Option<f64>]) -> usize {
    perfil
        .windows(2)
        .filter(|w| matches!(w, [Some(a), Some(b)] if (a - b).abs() > 3.0))
        .count()
}

/// O verde: matiz entre 65° e 190° com saturação > 0,15 (nem o amarelo puro ~49°, nem o azul ~220°).
pub(super) fn verde(c: [f64; 3]) -> Option<f64> {
    let (h, s) = matiz(c);
    (s > 0.15 && (65.0..190.0).contains(&h)).then_some(h)
}

pub(super) fn le_tudo(
    t: &PainterTool,
    ref_azul: f64,
    manchas_azul: f64,
    px: Option<&mut Vec<[u8; 3]>>,
    cob: Option<&[f32]>,
) -> Leitura {
    let (af, bf) = (AZ.map(f64::from), AM.map(f64::from));
    let seg = segmento_oklab(af, bf);
    let mut mix = vec![];
    let (mut km, mut n_cob, mut tinta, mut esc, mut fer) = (vec![], 0usize, 0usize, 0usize, 0usize);
    let mut px = px;
    let mut piores: Vec<(f64, usize, usize, [f64; 3], f64)> = vec![];
    let mut orla = vec![];
    let at = |x: usize, y: usize| sobre_creme(&t.canvas_rgba[(x + y * W as usize) * 4..][..4]);
    for y in RY.0..RY.1 {
        for x in RX.0..RX.1 {
            let (c, a) = at(x, y);
            if let Some(v) = px.as_deref_mut() {
                v.push(c.map(|v| v.round().clamp(0.0, 255.0) as u8));
            }
            let ac = coberto(cob, x, y, a);
            if ac >= 0.3 {
                tinta += 1;
                if ferrugem(c) {
                    fer += 1;
                }
            }
            if ac >= 0.9 {
                n_cob += 1;
                km.push(fora_da_mistura(c, af, bf));
                // A MISTURA mede-se onde a tinta cobre (a lei `Pigmento`: C ≥ 0,99); a película entre 0,9 e 0,99 é a
                // CAMADA FINA (o azul fino mais vivo pelas 3 bandas), outra pergunta — `orla`, E9 §7.
                let d = fora_do_segmento(c, &seg, a);
                if cob.is_none() || ac >= COBRE_A_MISTURA {
                    mix.push(d);
                } else {
                    orla.push(d);
                }
                if std::env::var_os("PH2D_SONDA_MIX_PIOR").is_some() {
                    piores.push((d, x, y, c, ac));
                }
                if lum(c) < ref_azul - 6.0 {
                    esc += 1;
                }
            }
        }
    }
    if !piores.is_empty() {
        println!(
            "[orla] n {} p99 {:.1} max {:.1}",
            orla.len(),
            pct(&mut orla.clone(), 0.99),
            orla.iter().copied().fold(0.0, f64::max)
        );
        piores.sort_by(|p, q| q.0.total_cmp(&p.0));
        let da_mistura = piores
            .iter()
            .filter(|p| cob.is_none() || p.4 >= COBRE_A_MISTURA);
        for (d, x, y, c, a) in da_mistura.step_by(40).take(12) {
            println!(
                "[mix] d {d:.1} x {x} y {y} cor {:?} cob {a:.3}",
                c.map(|v| v.round())
            );
        }
    }
    let media = |it: &mut dyn Iterator<Item = (usize, usize)>| {
        let v: Vec<f64> = it.filter_map(|(x, y)| verde(at(x, y).0)).collect();
        (v.len() >= 8).then(|| v.iter().sum::<f64>() / v.len() as f64)
    };
    let py: Vec<Option<f64>> = (RY.0..RY.1)
        .map(|y| media(&mut (150..210).map(|x| (x, y))))
        .collect();
    let pxp: Vec<Option<f64>> = (60..300)
        .map(|x| media(&mut (130..146).map(|y| (x, y))))
        .collect();
    let pc = |a: usize, b: usize| {
        if b == 0 {
            0.0
        } else {
            100.0 * a as f64 / b as f64
        }
    };
    Leitura {
        km_max: km.iter().copied().fold(0.0, f64::max),
        km_p99: pct(&mut km, 0.99),
        mix_max: mix.iter().copied().fold(0.0, f64::max),
        mix_p99: pct(&mut mix, 0.99),
        orla_p99: pct(&mut orla, 0.99),
        manchas: pc(esc, n_cob),
        manchas_azul,
        ferrugem: pc(fer, tinta),
        ref_azul,
        degraus_y: saltos(&py),
        degraus_x: saltos(&pxp),
    }
}
