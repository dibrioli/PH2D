//! J13b P2: o espelho incremental é a cópia total ao bit, e os planos dele são os da grade; o render do Real Oil
//! sobre o espelho é o da grade, byte a byte (nenhuma leitura de um plano que o espelho deixa vazio).

use super::*;
use crate::oleo_real::{CampoAltura, OleoReal};
use crate::render::{PigmentVisual, render_pigment_row_oleo};
use crate::sim::LeiDaTinta;
use crate::tuning::Knob;

const W: usize = 96;
const H: usize = 64;

struct Lcg(u64);
impl Lcg {
    fn f(&mut self, a: f64, b: f64) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        a + (b - a) * ((self.0 >> 11) as f64 / (1u64 << 53) as f64)
    }
}

fn traco(e: &mut Engine, r: &mut Lcg) {
    let (mut x, y) = (r.f(4.0, W as f64 - 30.0), r.f(4.0, H as f64 - 4.0));
    let x1 = x + r.f(8.0, 26.0);
    e.pointer_down(x, y, None);
    while x < x1 {
        x = (x + 4.0f64).min(x1);
        e.dispatch_pressure_dab(x, y + r.f(-2.0, 2.0), 6.0, 40.0);
    }
    e.pointer_up();
    while e.release_frame(x, y) {}
}

fn motor() -> Engine {
    let mut e = Engine::new(W, H);
    e.sim.set_tinta(LeiDaTinta::Pigmento);
    e.sim.oleo = true;
    let (diluicao, volume) =
        crate::painter::receita::migra_para_controlos(3000.0, 1.0, 0.0, 10.0, 1.0);
    e.set_knob(Knob::Dilution, diluicao);
    e.set_knob(Knob::Volume, volume);
    e.sliders.size = 1.0;
    e
}

fn f32s(a: &[f32], b: &[f32]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.to_bits() == y.to_bits())
}

fn rgbs(a: &[[f32; 3]], b: &[[f32; 3]]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| f32s(x, y))
}

/// Iguais em toda célula com tinta em `m`. A orla VIRGEM que o rasto veste (`*::veste_virgens`, `trail/transfer.rs:438`)
/// cai fora do sujo da porta: o espelho só a recebe quando a tinta lá chega (por uma porta) — invisível até lá.
fn fora_das_virgens<T: PartialEq>(a: &[T], b: &[T], m: &Grid) -> bool {
    a.len() == b.len()
        && (0..a.len()).all(|i| a[i] == b[i] || (m.susp[i] <= 0.0 && m.sett[i] <= 0.0))
}

/// Quantas células virgens de `m` diferem nos três campos da orla.
fn virgens_diferentes(a: &Grid, m: &Grid) -> usize {
    (0..m.cells)
        .filter(|&i| {
            a.oleo.get(i) != m.oleo.get(i)
                || a.material.get(i) != m.material.get(i)
                || a.aparencia.get(i) != m.aparencia.get(i)
        })
        .count()
}

/// O primeiro plano da óptica que difere (`None` = todos iguais ao bit; a orla virgem de `m` à parte).
fn difere(a: &Grid, b: &Grid, m: &Grid) -> Option<&'static str> {
    let planos = [
        ("film", f32s(&a.film, &b.film)),
        ("susp", f32s(&a.susp, &b.susp)),
        ("susp_rgb", rgbs(&a.susp_rgb, &b.susp_rgb)),
        ("sett", f32s(&a.sett, &b.sett)),
        ("sett_rgb", rgbs(&a.sett_rgb, &b.sett_rgb)),
        ("relevo", f32s(&a.relevo, &b.relevo)),
        ("altura", f32s(&a.altura, &b.altura)),
        ("espessura", f32s(&a.espessura, &b.espessura)),
        ("crista", f32s(&a.crista, &b.crista)),
        ("nivelado", f32s(&a.nivelado, &b.nivelado)),
        ("vela", f32s(&a.vela, &b.vela)),
        ("paper", f32s(&a.paper, &b.paper)),
        ("wet", a.wet == b.wet),
        ("material", fora_das_virgens(&a.material, &b.material, m)),
        (
            "aparencia",
            fora_das_virgens(&a.aparencia.para_vec(), &b.aparencia.para_vec(), m),
        ),
        ("oleo", fora_das_virgens(&a.oleo, &b.oleo, m)),
        ("tinta", a.tinta == b.tinta),
        ("pigmento", a.pigmento == b.pigmento),
        ("oleo_real", a.oleo_real == b.oleo_real),
    ];
    planos.iter().find(|(_, igual)| !igual).map(|(n, _)| *n)
}

/// A imagem do Real Oil (relevo, papel, campo de altura) sobre estas camadas.
fn imagem(e: &Engine, camadas: &[RenderLayer<'_>]) -> Vec<u8> {
    let o = OleoReal::default();
    let grades: Vec<_> = camadas.iter().map(|l| (l.grid, l.visible)).collect();
    let campo = CampoAltura::de(&grades, &o, 1, 1, 1, W, H);
    let p = e.sim.gather_params(&e.tuning);
    let visual = PigmentVisual {
        paper: true,
        relevo: true,
    };
    let mut out = vec![0u8; W * H * 4];
    for (k, linha) in out.chunks_mut(W * 4).enumerate() {
        render_pigment_row_oleo(Some(&p), camadas, visual, &o, &campo, k + 1, 1, W, linha);
    }
    out
}

fn camadas_do_motor(e: &Engine) -> Vec<RenderLayer<'_>> {
    e.layers
        .iter()
        .map(|l| RenderLayer {
            grid: &l.grid,
            opacity: l.opacity,
            visible: l.visible,
        })
        .collect()
}

#[test]
fn o_espelho_incremental_e_a_copia_total_ao_bit() {
    let mut e = motor();
    let mut r = Lcg(0x5EED_0013_B002);
    let mut inc = Espelho::default();
    e.copia_para(&mut inc);
    let (mut rects, mut fulls, mut virgens, mut vistos) = (0, 0, 0, [false; 5]);
    for passo in 0..80 {
        let accao = r.f(0.0, 11.0) as u32;
        match accao {
            0..=2 => {
                e.sim.oleo = r.f(0.0, 1.0) < 0.6;
                e.sliders.water = r.f(0.0, 1.0);
                traco(&mut e, &mut r);
            }
            3..=6 => {
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
                if !e.add_layer() {
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
        match e.espelho_sujo {
            Dirty::Full => fulls += 1,
            Dirty::Rect { .. } => rects += 1,
            Dirty::Clean => {}
        }
        e.copia_para(&mut inc);
        let mut total = Espelho::default();
        e.copia_para(&mut total);
        assert_eq!(inc.camadas.len(), e.layers.len());
        for (k, (c, l)) in inc.camadas.iter().zip(&e.layers).enumerate() {
            assert_eq!(
                difere(&c.grid, &l.grid, &l.grid),
                None,
                "passo {passo}, camada {k}: espelho ≠ grade"
            );
            assert_eq!(
                difere(&c.grid, &total.camadas[k].grid, &l.grid),
                None,
                "passo {passo}: incremental ≠ total"
            );
            assert_eq!(
                difere(&total.camadas[k].grid, &l.grid, &l.grid),
                None,
                "passo {passo}: total ≠ grade"
            );
            virgens = virgens.max(virgens_diferentes(&c.grid, &l.grid));
            let g = &l.grid;
            let agora = [
                !g.oleo.is_empty(),
                !g.altura.is_empty(),
                !g.tinta.is_empty(),
                !g.espessura.is_empty(),
            ];
            for (v, a) in vistos
                .iter_mut()
                .zip(agora.into_iter().chain([g.has_fluid]))
            {
                *v |= a;
            }
            assert_eq!((c.opacity, c.visible), (l.opacity, l.visible));
        }
        assert_eq!(
            imagem(&e, &inc.camadas_render()),
            imagem(&e, &camadas_do_motor(&e)),
            "passo {passo}"
        );
    }
    // Controlos: a cena exercitou o caminho incremental e a óptica do óleo e da lei da tinta.
    eprintln!("rects {rects} · fulls {fulls} · máx. de células virgens diferentes {virgens}");
    assert!(rects >= 20, "rects {rects} fulls {fulls}");
    assert_eq!(
        vistos, [true; 5],
        "oleo · altura · tinta · espessura · água viva"
    );
}

/// O custo de `copia_para` com `Dirty::Full` (no lugar, e a reconstrução) e os bytes por célula. Medida: `--ignored`.
#[test]
#[ignore = "medida"]
fn mede_copia_para_full() {
    for n in [1024usize, 4096] {
        let mut e = Engine::new(n, n);
        e.sim.set_tinta(LeiDaTinta::Pigmento);
        e.sim.oleo = true;
        e.sliders.size = 1.0;
        let mut r = Lcg(7);
        for _ in 0..3 {
            let (x, y) = (r.f(10.0, n as f64 - 40.0), r.f(10.0, n as f64 - 10.0));
            e.pointer_down(x, y, None);
            e.dispatch_pressure_dab(x + 8.0, y, 6.0, 40.0);
            e.pointer_up();
        }
        let mut esp = Espelho::default();
        let t = std::time::Instant::now();
        e.copia_para(&mut esp);
        let reconstroi = t.elapsed();
        let mut melhor = std::time::Duration::MAX;
        for _ in 0..5 {
            e.mark_dirty_full();
            let t = std::time::Instant::now();
            e.copia_para(&mut esp);
            melhor = melhor.min(t.elapsed());
        }
        let g = &e.layers[0].grid;
        let planos = [
            ("altura", g.altura.len()),
            ("espessura", g.espessura.len()),
            ("crista", g.crista.len()),
            ("nivelado", g.nivelado.len()),
            ("material", g.material.len()),
            ("aparencia", g.aparencia.len()),
            ("oleo", g.oleo.len()),
            ("tinta", g.tinta.len()),
            ("pigmento", g.pigmento.len()),
        ];
        println!(
            "{n}²: reconstrói {:.1} ms · Full no lugar (mín 5) {:.1} ms · {:.1} B/célula · planos {planos:?}",
            reconstroi.as_secs_f64() * 1e3,
            melhor.as_secs_f64() * 1e3,
            esp.bytes() as f64 / g.film.len() as f64,
        );
    }
    println!(
        "tamanhos: material {} · aparencia {} · oleo {} · tinta {} · pigmento {}",
        std::mem::size_of::<crate::material::MaterialCelula>(),
        std::mem::size_of::<crate::grid::aparencia::AparenciaCelula>(),
        std::mem::size_of::<crate::grid::oleo_celula::OleoCelula>(),
        std::mem::size_of::<crate::grid::tinta::CelulaTinta>(),
        std::mem::size_of::<crate::grid::tinta::pigmento::LadoEspectral>(),
    );
}

#[path = "espelho_estreito_tests.rs"]
mod estreito; // B11.2: a cópia estreita contra a de linhas inteiras
