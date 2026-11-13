//! **B18.5:** três gémeos (Wet no papel do dono, «+», traços, troca de camada, undo/redo) no relógio fixo dão o mesmo
//! bit a cada quadro e em qualquer largura do pool. Antes, o quadro em que a tinta adormecida acordava dependia do fio
//! que a comprime (`wet_acorda_sem_esperar`): um gémeo acordava-a no undo, o outro um quadro depois.

use super::agua_sobre_oleo_tests::ponteiro;
use super::camada_nova_toque_tests::no_papel_do_dono;
use crate::tool::PainterTool;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase, RasterEditTool};

fn h(v: &[u8]) -> u64 {
    v.iter().fold(0xcbf2_9ce4_8422_2325u64, |a, x| {
        (a ^ u64::from(*x)).wrapping_mul(0x0100_0000_01b3)
    })
}

/// O que o quadro mostra e onde a tinta viva está.
fn marca(t: &mut PainterTool) -> [u64; 4] {
    let w = &t.paint.wetpaint;
    let (sessao, dorme) = (u64::from(w.session.is_some()), w.adormecidas.len() as u64);
    let relevo = t.impasto_gpu_planes().map_or(0, |p| h(&p.cover));
    let tela = t.current_preview().map_or(0, |(p, _, _)| h(p));
    [tela, sessao, dorme, relevo]
}

type Passo<'a> = &'a dyn Fn(&mut PainterTool);

fn sequencia(f: &mut dyn FnMut(&str, Passo)) {
    let traco = |f: &mut dyn FnMut(&str, Passo), y: f32| {
        f("down", &|t| {
            t.on_canvas_pointer(ponteiro(200.0, y, PointerPhase::Down));
        });
        for k in 1..=6 {
            let x = 200.0 + 6.0 * k as f32;
            f("move", &|t| {
                t.on_canvas_pointer(ponteiro(x, y, PointerPhase::Move));
            });
        }
        f("up", &|t| {
            t.on_canvas_pointer(ponteiro(240.0, y, PointerPhase::Up));
        });
        for _ in 0..3 {
            f("repouso", &|_| {});
        }
    };
    let troca = |t: &mut PainterTool| {
        let outra = t
            .layers
            .root()
            .iter()
            .copied()
            .find(|id| Some(*id) != t.layers.active());
        t.select_layer(outra.expect("a outra camada"));
    };
    traco(f, 100.0);
    traco(f, 160.0);
    f("+", &|t| {
        let id = t.add_raster_layer("nova").expect("a camada nova");
        t.select_layer(id);
    });
    traco(f, 260.0);
    traco(f, 320.0);
    f("troca", &troca);
    traco(f, 380.0);
    f("volta", &troca);
    traco(f, 420.0);
    for (caso, desfaz) in [("undo", true), ("redo", false)] {
        for _ in 0..4 {
            f(caso, &|t| {
                if desfaz {
                    t.undo_last();
                } else {
                    t.redo_last();
                }
            });
            for _ in 0..3 {
                f("repouso", &|_| {});
            }
        }
    }
}

/// Os gémeos quadro a quadro; devolve as marcas do 1.º.
fn gemeos() -> Vec<[u64; 4]> {
    let fixo = || {
        let mut t = no_papel_do_dono();
        t.set_wet_relogio_fixo(true);
        t
    };
    let mut ts = [fixo(), fixo(), fixo()];
    let (mut n, mut fita) = (0, vec![]);
    sequencia(&mut |caso, f| {
        n += 1;
        for t in &mut ts {
            f(t);
            t.on_tick(1000.0 / 60.0);
        }
        let m: Vec<_> = ts.iter_mut().map(marca).collect();
        assert!(
            m.iter().all(|x| *x == m[0]),
            "quadro {n} ({caso}): os gémeos divergem [tela, sessão, adormecidas, relevo] {m:?}"
        );
        fita.push(m[0]);
    });
    fita
}

#[test]
fn undo_os_gemeos_da_camada_adormecida_dao_o_mesmo_bit_em_qualquer_largura_do_pool() {
    let cheio = gemeos();
    for n in [1, 2, 7, 16, 32] {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(n)
            .build()
            .expect("o pool");
        let fita = pool.install(gemeos);
        assert!(fita == cheio, "{n} fios: a sequência ≠ a do pool cheio");
    }
}
