//! O **desfazer/refazer por camada** (SPEC §15) — filho de [`super`] por LOC.
//!
//! Cada passo é um [`Registo`] copy-on-write por tiles: guarda só o que foi escrito desde que abriu.
//! O último de `undo` está ABERTO e toda porta que escreve no grid chama `marca_*` ANTES de escrever
//! (o passo do sim inclusive — a água que corre depois do pointer_up entra no registo do traço).

use super::*;

/// Folga em volta de um dab: o rasto é uma janela de 123² ancorada num dab anterior, que contém o
/// dab actual — tudo o que um dab escreve cabe em `r + 130` células dele.
const FOLGA_DAB: f64 = 130.0;
/// Folga em volta da caixa viva num estágio do sim: a água anda menos de uma célula por passo e o
/// rebuild alarga a caixa por poucas; 64 é um tile inteiro.
const FOLGA_PASSO: i64 = 64;

impl Engine {
    // -----------------------------------------------------------------------
    // History (SPEC §15): per-layer undo/redo ring, depth 6.
    // -----------------------------------------------------------------------

    /// Abre o registo de um passo. NÃO drena o passo em voo: drenar no pen-down mudava o ritmo da água (o lote
    /// do traço já não pousava a meio de um passo). Os estágios que faltam escrevem depois de abrir e marcam os seus
    /// tiles nele ([`Self::marca_passo`]); o cursor do passo é estado fora das grades e quem fotografa o resto
    /// guarda-o ([`crate::sim::Sim::cursor_do_passo`]).
    pub fn capture_history(&mut self) {
        let layer = &mut self.layers[self.active_layer];
        layer.undo.push(Registo::abre(&layer.grid));
        if layer.undo.len() > HISTORY_DEPTH {
            layer.undo.remove(0);
        }
        layer.redo.clear();
        layer.desde = None;
    }

    /// Marca o retângulo (células finas, com pad) no registo aberto da camada activa ANTES de escrever.
    pub(crate) fn marca_ret(&mut self, x0: i64, y0: i64, x1: i64, y1: i64) {
        let layer = &mut self.layers[self.active_layer];
        let g = &layer.grid;
        if let Some(r) = layer.undo.last_mut() {
            r.marca(g, x0, y0, x1, y1);
        }
        if let Some(r) = layer.desde.as_mut() {
            r.marca(g, x0, y0, x1, y1);
        }
    }

    /// Marca a camada activa INTEIRA (escrita sem retângulo conhecido).
    pub(crate) fn marca_tudo(&mut self) {
        let g = &self.layers[self.active_layer].grid;
        let (s, rows) = (g.s as i64, g.rows as i64);
        self.marca_ret(0, 0, s, rows);
    }

    /// Marca o alcance de um dab em `(x, y)` px de raio `r`.
    pub(crate) fn marca_dab(&mut self, x: f64, y: f64, r: f64) {
        let m = r.abs() + FOLGA_DAB;
        let (x0, x1) = ((x - m).floor() as i64, (x + m).ceil() as i64 + 2);
        let (y0, y1) = ((y - m).floor() as i64, (y + m).ceil() as i64 + 2);
        let l = &mut self.layers[self.active_layer];
        l.agua_nova = Some(match l.agua_nova {
            Some([a0, b0, a1, b1]) => [a0.min(x0), b0.min(y0), a1.max(x1), b1.max(y1)],
            None => [x0, y0, x1, y1],
        });
        self.marca_ret(x0, y0, x1, y1);
    }

    /// Marca o que um estágio do sim pode escrever: a caixa viva do último rebuild UNIDA aos dabs que
    /// ela ainda não viu, com folga (nada sem fluido).
    pub(crate) fn marca_passo(&mut self) {
        let l = &self.layers[self.active_layer];
        let g = &l.grid;
        if !g.has_fluid && !self.sim.step_pending() {
            return;
        }
        let caixa = (g.bx1 >= g.bx0 && g.by1 >= g.by0).then(|| {
            [
                i64::from(g.bx0),
                i64::from(g.by0),
                i64::from(g.bx1) + 1,
                i64::from(g.by1) + 1,
            ]
        });
        let r = match (caixa, l.agua_nova) {
            (Some([a0, b0, a1, b1]), Some([c0, d0, c1, d1])) => {
                [a0.min(c0), b0.min(d0), a1.max(c1), b1.max(d1)]
            }
            (Some(r), None) | (None, Some(r)) => r,
            (None, None) => return self.marca_tudo(),
        };
        let f = FOLGA_PASSO;
        self.marca_ret(r[0] - f, r[1] - f, r[2] + f, r[3] + f);
    }

    pub fn can_undo(&self) -> bool {
        !self.layers[self.active_layer].undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.layers[self.active_layer].redo.is_empty()
    }

    pub fn undo(&mut self) -> bool {
        self.drain_step();
        let layer = &mut self.layers[self.active_layer];
        let Some(reg) = layer.undo.pop() else {
            return false;
        };
        // O registo estava aberto: o grid só difere da base dele nos tiles que ele guarda.
        layer.redo.push(Registo::inverso(&layer.grid, &reg, None));
        let paper_changed = reg.restaura(&mut layer.grid, None);
        layer.desde = Some(Registo::abre(&layer.grid));
        self.depois_do_restauro(paper_changed);
        true
    }

    pub fn redo(&mut self) -> bool {
        let layer = &mut self.layers[self.active_layer];
        let Some(reg) = layer.redo.pop() else {
            return false;
        };
        // Desde o último evento o grid andou nos tiles de `desde`; fora deles e dos do registo,
        // ele É a base do registo.
        let desde = layer.desde.take();
        let inv = Registo::inverso(&layer.grid, &reg, desde.as_ref());
        let paper_changed = reg.restaura(&mut layer.grid, desde.as_ref());
        layer.undo.push(inv);
        if layer.undo.len() > HISTORY_DEPTH {
            layer.undo.remove(0);
        }
        layer.desde = (!layer.redo.is_empty()).then(|| Registo::abre(&layer.grid));
        self.depois_do_restauro(paper_changed);
        true
    }

    fn depois_do_restauro(&mut self, paper_changed: bool) {
        if paper_changed {
            let g = &self.layers[self.active_layer].grid;
            self.paper_preset = g.paper_preset;
            self.paper_sheet = g.paper_sheet;
            self.rebake_paper();
        }
        self.mark_dirty_full();
    }

    /// Bytes retidos pelo histórico de uma camada (os registos de `undo`, `redo` e `desde`).
    #[must_use]
    pub fn bytes_do_historico(&self, camada: usize) -> usize {
        let l = &self.layers[camada];
        l.undo
            .iter()
            .chain(&l.redo)
            .chain(&l.desde)
            .map(Registo::bytes_retidos)
            .sum()
    }
}
