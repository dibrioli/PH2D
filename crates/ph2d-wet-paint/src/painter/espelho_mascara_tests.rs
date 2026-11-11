//! B13.1: o sujo do espelho do PASSO são os tiles que ele declarou (a grelha do undo), não a caixa do fluido. O
//! caminho antigo é a mesma cópia com a caixa (`espelho_passo`, guardada só em teste) no lugar da máscara.

use super::super::faixas_da_caixa;
use super::*;

const W2: usize = 640;
const H2: usize = 384;

pub(super) fn uniao(a: Dirty, b: Dirty) -> Dirty {
    match (a, b) {
        (Dirty::Full, _) | (_, Dirty::Full) => Dirty::Full,
        (Dirty::Clean, x) | (x, Dirty::Clean) => x,
        (
            Dirty::Rect { x0, y0, x1, y1 },
            Dirty::Rect {
                x0: a0,
                y0: b0,
                x1: a1,
                y1: b1,
            },
        ) => Dirty::Rect {
            x0: x0.min(a0),
            y0: y0.min(b0),
            x1: x1.max(a1),
            y1: y1.max(b1),
        },
    }
}

/// As células que a cópia da camada activa leva: com a máscara e com a caixa do passo.
fn celulas(e: &Engine, sujo: Dirty, tiles: &[u64], passo: Dirty) -> (usize, usize) {
    let g = &e.layers[e.active_layer].grid;
    let soma = |d: Dirty, t: &[u64]| -> usize {
        let caixa = match d {
            Dirty::Full => return g.s * g.rows,
            Dirty::Clean if !t.iter().any(|b| *b != 0) => return 0,
            Dirty::Clean => None,
            Dirty::Rect { x0, y0, x1, y1 } => Some((x0, y0, x1, y1)),
        };
        faixas_da_caixa(g, caixa, t).iter().map(|r| r.len()).sum()
    };
    (soma(sujo, tiles), soma(uniao(sujo, passo), &[]))
}

/// Copia o MESMO sujo pelos dois caminhos e confere-os ao bit (planos e a imagem do Real Oil); devolve as células.
fn copia_e_confere(
    e: &mut Engine,
    nova: &mut Espelho,
    antiga: &mut Espelho,
    passo_n: usize,
) -> (usize, usize) {
    let (sujo, tiles, passo) = (e.espelho_sujo, e.espelho_tiles.clone(), e.espelho_passo);
    let n = celulas(e, sujo, &tiles, passo);
    e.copia_para(nova);
    e.espelho_sujo = uniao(sujo, passo);
    e.copia_para(antiga);
    for (k, l) in e.layers.iter().enumerate() {
        let (a, b) = (&nova.camadas[k].grid, &antiga.camadas[k].grid);
        assert_eq!(
            difere(a, b, &l.grid),
            None,
            "{passo_n}, camada {k}: máscara ≠ caixa"
        );
        assert_eq!(
            difere(a, &l.grid, &l.grid),
            None,
            "{passo_n}, camada {k}: máscara ≠ grade"
        );
    }
    assert_eq!(
        imagem(e, &nova.camadas_render()),
        imagem(e, &camadas_do_motor(e)),
        "{passo_n}: a imagem do espelho da máscara ≠ a da grade"
    );
    n
}

fn diagonal(e: &mut Engine, (x0, y0): (f64, f64), (x1, y1): (f64, f64)) {
    e.pointer_down(x0, y0, None);
    let n = ((x1 - x0).hypot(y1 - y0) / 4.0).ceil() as usize;
    for k in 1..=n {
        let t = k as f64 / n as f64;
        e.dispatch_pressure_dab(x0 + (x1 - x0) * t, y0 + (y1 - y0) * t, 8.0, 40.0);
    }
    e.pointer_up();
}

/// As células que a cópia do espelho leva nos `passos` depois de uma diagonal de canto a canto: (caixa, máscara).
fn celulas_da_diagonal(w: usize, h: usize, passos: usize) -> (usize, usize) {
    let mut e = Engine::new(w, h);
    e.sim.set_tinta(LeiDaTinta::Pigmento);
    e.sliders.size = 1.0;
    diagonal(&mut e, (24.0, 24.0), (w as f64 - 24.0, h as f64 - 24.0));
    let mut esp = Espelho::default();
    e.copia_para(&mut esp);
    let (mut sem, mut com) = (0, 0);
    for _ in 0..passos {
        e.step_simulation();
        let (a, b) = celulas(&e, e.espelho_sujo, &e.espelho_tiles, e.espelho_passo);
        com += a;
        sem += b;
        e.copia_para(&mut esp);
    }
    (sem, com)
}

/// A diagonal a 4K (3 840 × 2 160), só a contagem. Medida: `--ignored`.
#[test]
#[ignore = "medida"]
fn mede_celulas_da_diagonal_4k() {
    let (sem, com) = celulas_da_diagonal(3840, 2160, 6);
    println!(
        "diagonal 4K: caixa {sem} · máscara {com} células em 6 passos ({:.2}×)",
        sem as f64 / com as f64
    );
}

/// A cena do B11.2 numa folha de 10 × 6 tiles com traços em DIAGONAL e duas manchas nas mesmas linhas: a cada acção a
/// cópia pela máscara é a da caixa ao bit e a da grade; a diagonal copia ≥ 1,4× menos células.
#[test]
fn a_mascara_ligada_e_desligada_dao_o_mesmo_byte() {
    let mut e = Engine::new(W2, H2);
    e.sim.set_tinta(LeiDaTinta::Pigmento);
    e.sim.oleo = true;
    e.sliders.size = 1.0;
    let mut r = Lcg(0x5EED_0B13_0001);
    let (mut nova, mut antiga) = (Espelho::default(), Espelho::default());
    e.copia_para(&mut nova);
    e.mark_dirty_full();
    e.copia_para(&mut antiga);
    // 1. A diagonal sozinha: o passo que corre sobre ela.
    diagonal(&mut e, (24.0, 24.0), (W2 as f64 - 24.0, H2 as f64 - 24.0));
    copia_e_confere(&mut e, &mut nova, &mut antiga, 0);
    for k in 0..6 {
        e.step_simulation();
        copia_e_confere(&mut e, &mut nova, &mut antiga, k + 1);
    }
    let (sem, com) = celulas_da_diagonal(1920, 1080, 6);
    let razao = sem as f64 / com.max(1) as f64;
    eprintln!("diagonal 1920×1080: caixa {sem} · máscara {com} células em 6 passos ({razao:.2}×)");
    assert!(
        razao >= 1.4,
        "a máscara não paga: {razao:.2}× (caixa {sem}, máscara {com})"
    );
    // 2. A cena aleatória: traços, passos, estágios, undo, camadas, Wet canvas, manchas nas mesmas linhas.
    for passo in 0..140 {
        match r.f(0.0, 12.0) as u32 {
            0..=2 => {
                let (x, y) = (r.f(10.0, W2 as f64 - 10.0), r.f(10.0, H2 as f64 - 10.0));
                let (x1, y1) = (r.f(10.0, W2 as f64 - 10.0), r.f(10.0, H2 as f64 - 10.0));
                e.sim.oleo = r.f(0.0, 1.0) < 0.6;
                diagonal(&mut e, (x, y), (x1, y1));
            }
            3 => {
                let y = r.f(40.0, H2 as f64 - 40.0);
                diagonal(&mut e, (30.0, y), (50.0, y + 30.0));
                diagonal(&mut e, (W2 as f64 - 60.0, y), (W2 as f64 - 30.0, y + 30.0));
            }
            4..=6 => {
                for _ in 0..(r.f(1.0, 10.0) as usize) {
                    e.step_simulation();
                }
            }
            7 => {
                for _ in 0..(r.f(1.0, 9.0) as usize) {
                    e.step_stage();
                }
            }
            8 => {
                if e.can_undo() {
                    e.undo();
                }
            }
            9 => {
                if e.can_redo() {
                    e.redo();
                }
            }
            10 => {
                if e.layers.len() >= 3 || !e.add_layer() {
                    e.select_layer(r.f(0.0, e.layers.len() as f64) as usize);
                }
            }
            _ => e.action_wet_canvas(),
        }
        copia_e_confere(&mut e, &mut nova, &mut antiga, passo + 100);
    }
}
