//! B12.2: o enterro percorre só as linhas tocadas, ao bit do de antes.

use super::*;
use crate::painter::historia_tests::{corre, hash_grid, traco};
use std::cell::Cell;

thread_local! {
    /// Células que os laços do enterro percorrem depois de achar a primeira e a última tocadas.
    pub(crate) static VISITADAS: Cell<usize> = const { Cell::new(0) };
}

impl Engine {
    /// O enterro de antes de B12.2, verbatim: o oráculo.
    fn enterra_celulas_antigo(&mut self, tocada: &[bool], gw: usize) {
        self.drain_step();
        let Some(primeira) = tocada.iter().position(|t| *t) else {
            return;
        };
        let ultima = tocada.iter().rposition(|t| *t).unwrap_or(primeira);
        let (mut x0, mut x1) = (gw, 0);
        for (k, _) in tocada.iter().enumerate().filter(|(_, t)| **t) {
            x0 = x0.min(k % gw);
            x1 = x1.max(k % gw);
        }
        self.marca_ret(
            x0 as i64 + 1,
            (primeira / gw) as i64 + 1,
            x1 as i64 + 2,
            (ultima / gw) as i64 + 2,
        );
        let g = &mut self.layers[self.active_layer].grid;
        let s = g.s;
        for (k, _) in tocada.iter().enumerate().filter(|(_, t)| **t) {
            crate::grid::enterra_celula(g, (k / gw + 1) * s + k % gw + 1);
        }
    }
}

fn molhado(w: usize, h: usize) -> Engine {
    let mut e = Engine::new(w, h);
    e.sliders.water = 1.0;
    traco(&mut e, 40.0, 40.0, 5);
    traco(&mut e, w as f64 - 90.0, h as f64 - 60.0, 5);
    corre(&mut e, 4);
    e.capture_history();
    e
}

/// Um enterro de 3×3 células a 1920×1080 percorre as 3 linhas delas, não a tela.
#[test]
fn historia_o_enterro_percorre_so_as_linhas_tocadas() {
    let (gw, gh) = (1920, 1080);
    let mut e = molhado(gw, gh);
    let mut t = vec![false; gw * gh];
    for y in 500..503 {
        for x in 700..703 {
            t[y * gw + x] = true;
        }
    }
    VISITADAS.with(|v| v.set(0));
    e.enterra_celulas(&t, gw);
    let n = VISITADAS.with(Cell::get);
    assert!(
        n <= 2 * 3 * gw,
        "{n} células percorridas (as 3 linhas: {})",
        3 * gw
    );
}

/// O enterro dá a grade, o registo e o undo ao bit do oráculo: células espalhadas, linhas vazias no meio, os cantos.
#[test]
fn historia_o_enterro_por_linhas_da_o_bit_do_antigo() {
    let (gw, gh) = (600, 400);
    let mut t = vec![false; gw * gh];
    let mut z = 0x9e37_79b9_u32;
    for k in 0..gw * gh {
        z ^= z << 13;
        z ^= z >> 17;
        z ^= z << 5;
        let y = k / gw;
        t[k] = (y % 7 == 2 || (30..60).contains(&y)) && z.is_multiple_of(5);
    }
    t[0] = true;
    t[gw * gh - 1] = true;
    t[2 * gw - 1] = true;
    for caso in [t.clone(), {
        let mut u = vec![false; gw * gh];
        u[123 * gw + 77] = true;
        u
    }] {
        let (mut a, mut b) = (molhado(gw, gh), molhado(gw, gh));
        assert_eq!(hash_grid(a.active_grid()), hash_grid(b.active_grid()));
        a.enterra_celulas_antigo(&caso, gw);
        b.enterra_celulas(&caso, gw);
        assert_eq!(
            hash_grid(a.active_grid()),
            hash_grid(b.active_grid()),
            "a grade"
        );
        assert_eq!(
            a.bytes_do_historico(0),
            b.bytes_do_historico(0),
            "o registo"
        );
        assert_eq!(
            a.caixa_do_restauro(false),
            b.caixa_do_restauro(false),
            "a caixa"
        );
        assert!(a.undo() && b.undo());
        assert_eq!(
            hash_grid(a.active_grid()),
            hash_grid(b.active_grid()),
            "o undo"
        );
    }
}
