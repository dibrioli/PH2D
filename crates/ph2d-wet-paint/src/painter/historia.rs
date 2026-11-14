//! O **desfazer/refazer por camada** (SPEC §15) — filho de [`super`] por LOC.
//!
//! Cada passo é um [`Registo`] copy-on-write por tiles: guarda só o que foi escrito desde que abriu.
//! O último de `undo` está ABERTO e toda porta que escreve no grid chama `marca_*` ANTES de escrever
//! (o passo do sim inclusive — a água que corre depois do pointer_up entra no registo do traço).

use super::*;
use crate::grid::PassoEmVoo;

/// Folga em volta de um dab: o rasto é uma janela de 123² ancorada num dab anterior, que contém o
/// dab actual — tudo o que um dab escreve cabe em `r + 130` células dele.
const FOLGA_DAB: f64 = 130.0;
/// Folga em volta da caixa viva num estágio do sim: a água anda menos de uma célula por passo e o
/// rebuild alarga a caixa por poucas; 64 é um tile inteiro.
const FOLGA_PASSO: i64 = 64;

/// B18.11 cura 3: o que as portas do dab escrevem — tudo menos o `bloom` (o orçamento de floração é só do passo da
/// água, `solver/flow_jacobi/backrun.rs`, e das acções de tela; nenhum `tools/`, `trail/` nem `brush` o toca).
/// `relevo` e `vela` ficam: o depósito escreve-os (`trail/transfer.rs`) quando a ponta os traz.
const PLANOS_DO_DAB: crate::grid::Planos = crate::grid::Planos {
    bloom: false,
    ..crate::grid::Planos::TODOS
};

/// B18.11 cura 3b: o que um estágio do passo da água escreve — tudo menos `relevo` e `vela` (só o depósito
/// `trail/transfer.rs`, as acções de tela `grid/canvas_ops.rs` e o `grid/patch.rs` os escrevem; o sim e a secagem
/// só os lêem).
const PLANOS_DO_PASSO: crate::grid::Planos = crate::grid::Planos {
    relevo: false,
    vela: false,
    ..crate::grid::Planos::TODOS
};

/// Um retângulo por banda de [`crate::grid::TILE`] linhas: a união das janelas da faixa viva das linhas a `f` dela,
/// com `f` de folga em x (meio-aberto, células finas com o pad). Banda sem linha viva a `f` não entra.
fn faixas_do_passo(g: &Grid, f: i64) -> Vec<[i64; 4]> {
    let t = crate::grid::TILE as i64;
    let rows = g.row_lo.len() as i64;
    let janela: Vec<(i64, i64)> = (0..rows)
        .map(|y| {
            let (a, b) = g.span_window(y as i32);
            (i64::from(a), i64::from(b))
        })
        .collect();
    let mut out = Vec::new();
    for b0 in (0..rows).step_by(t as usize) {
        let (lo, hi) = ((b0 - f).max(0), (b0 + t + f).min(rows));
        let (x0, x1) = janela[lo as usize..hi as usize]
            .iter()
            .filter(|(a, b)| a <= b)
            .fold((i64::MAX, i64::MIN), |(x0, x1), &(a, b)| {
                (x0.min(a), x1.max(b))
            });
        if x0 <= x1 {
            out.push([x0 - f, b0, x1 + 1 + f, b0 + t]);
        }
    }
    out
}

/// O que um restauro pode mudar na camada INTEIRA fora dos tiles: os planos preguiçosos que nascem/morrem, o
/// caminho do Real Oil e o tecto do Smooth Edges (o alcance da óptica).
type FormaGlobal = ([usize; 9], bool, u32);

#[cfg(test)]
mod enterro_tests;

fn forma_global(g: &Grid) -> FormaGlobal {
    (
        [
            g.altura.len(),
            g.espessura.len(),
            g.crista.len(),
            g.nivelado.len(),
            g.material.len(),
            g.aparencia.len(),
            g.oleo.len(),
            g.tinta.len(),
            g.pigmento.len(),
        ],
        g.oleo_real,
        g.suave_teto.to_bits(),
    )
}

impl Engine {
    // -----------------------------------------------------------------------
    // History (SPEC §15): per-layer undo/redo ring, depth 6.
    // -----------------------------------------------------------------------

    /// Abre o registo de um passo. NÃO drena o passo em voo: drenar no pen-down mudava o ritmo da água (o lote
    /// do traço já não pousava a meio de um passo). Os estágios que faltam escrevem depois de abrir e marcam os seus
    /// tiles nele ([`Self::marca_passo`]); o que o passo guarda fora das grades ([`PassoEmVoo`]) entra no registo e
    /// o undo repõe-no — quem chama não precisa saber.
    pub fn capture_history(&mut self) {
        let passo = self.passo_em_voo();
        let fundo = self.fundo_da_historia;
        let layer = &mut self.layers[self.active_layer];
        let mut reg = Registo::abre(&layer.grid);
        reg.passo = passo;
        layer.undo.push(reg);
        if layer.undo.len() > fundo {
            layer.undo.remove(0);
        }
        layer.redo.clear();
        layer.desde = None;
    }

    /// Marca o retângulo (células finas, com pad) no registo aberto da camada activa ANTES de escrever.
    pub(crate) fn marca_ret(&mut self, x0: i64, y0: i64, x1: i64, y1: i64) {
        self.marca_ret_planos(x0, y0, x1, y1, crate::grid::Planos::TODOS);
    }

    /// O [`Self::marca_ret`] de quem só escreve os planos `m`.
    fn marca_ret_planos(&mut self, x0: i64, y0: i64, x1: i64, y1: i64, m: crate::grid::Planos) {
        let layer = &mut self.layers[self.active_layer];
        let g = &layer.grid;
        if let Some(r) = layer.undo.last_mut() {
            r.marca_planos(g, x0, y0, x1, y1, m);
        }
        if let Some(r) = layer.desde.as_mut() {
            r.marca_planos(g, x0, y0, x1, y1, m);
        }
    }

    /// Marca a camada activa INTEIRA (escrita sem retângulo conhecido).
    pub(crate) fn marca_tudo(&mut self) {
        self.marcas_da_tela_inteira += 1;
        let g = &self.layers[self.active_layer].grid;
        let (s, rows) = (g.s as i64, g.rows as i64);
        self.marca_ret(0, 0, s, rows);
    }

    /// B44.9: marca as linhas finas `y0..=y1` (largura inteira) antes de uma escrita que só as toca (a descida da placa).
    pub fn marca_linhas(&mut self, y0: usize, y1: usize) {
        let s = self.layers[self.active_layer].grid.s as i64;
        self.marca_ret(0, y0 as i64, s, y1 as i64 + 1);
    }

    /// B44.9 (GT-08): as escritas de tela inteira marcadas até agora (o marcador do censo «nenhum quadro de
    /// movimento anda a tela inteira»).
    #[doc(hidden)]
    #[must_use]
    pub fn marcas_da_tela_inteira(&self) -> u64 {
        self.marcas_da_tela_inteira
    }

    /// Marca o alcance de um dab em `(x, y)` px de raio `r`.
    pub(crate) fn marca_dab(&mut self, x: f64, y: f64, r: f64) {
        let m = r.abs() + FOLGA_DAB;
        let (x0, x1) = ((x - m).floor() as i64, (x + m).ceil() as i64 + 2);
        let (y0, y1) = ((y - m).floor() as i64, (y + m).ceil() as i64 + 2);
        let une = |c: Option<[i64; 4]>| match c {
            Some([a0, b0, a1, b1]) => [a0.min(x0), b0.min(y0), a1.max(x1), b1.max(y1)],
            None => [x0, y0, x1, y1],
        };
        let l = &mut self.layers[self.active_layer];
        l.agua_nova = Some(une(l.agua_nova));
        if let Some(r) = l.undo.last_mut() {
            r.dabs = Some(une(r.dabs));
        }
        self.marca_ret_planos(x0, y0, x1, y1, PLANOS_DO_DAB);
    }

    /// O casco dos dabs do registo aberto da camada activa (células finas com o pad, meio-aberto). `None`: sem dabs.
    #[must_use]
    pub fn casco_dos_dabs_do_aberto(&self) -> Option<[i64; 4]> {
        self.layers[self.active_layer].undo.last()?.dabs
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
        let f = FOLGA_PASSO;
        let r = match (caixa, l.agua_nova) {
            (Some([a0, b0, a1, b1]), Some([c0, d0, c1, d1])) if !g.spans_enabled => {
                [a0.min(c0), b0.min(d0), a1.max(c1), b1.max(d1)]
            }
            (Some(r), None) if !g.spans_enabled => r,
            (None, Some(r)) => r,
            (None, None) => {
                let (s, rows) = (g.s as i64, g.rows as i64);
                return self.marca_do_passo(0, 0, s, rows);
            }
            (Some(_), dabs) => {
                // B12.1: a faixa viva (`{film, susp, vel ≠ 0} ⊆ faixa`, `verify_spans`) por banda de tiles; o casco
                // levava as linhas paradas entre pinceladas afastadas.
                for [x0, y0, x1, y1] in faixas_do_passo(g, f) {
                    self.marca_do_passo(x0, y0, x1, y1);
                }
                match dabs {
                    Some(r) => r,
                    None => return,
                }
            }
        };
        self.marca_do_passo(r[0] - f, r[1] - f, r[2] + f, r[3] + f);
    }

    /// O que o passo marca no registo é o que ele DECLARA ao espelho (B13.1: a mesma grelha de tiles).
    fn marca_do_passo(&mut self, x0: i64, y0: i64, x1: i64, y1: i64) {
        self.marca_ret_planos(x0, y0, x1, y1, PLANOS_DO_PASSO);
        self.declara_tiles_do_passo(x0, y0, x1, y1);
    }

    /// Enterra as células marcadas em `tocada` (`gw` por linha, sem o pad): ver [`crate::grid::enterra_celula`]. O
    /// histórico guarda o que o enterro apaga: o undo do passo devolve as células.
    pub fn enterra_celulas(&mut self, tocada: &[bool], gw: usize) {
        self.drain_step();
        let Some(primeira) = tocada.iter().position(|t| *t) else {
            return;
        };
        let ultima = tocada.iter().rposition(|t| *t).unwrap_or(primeira);
        // B12.2: só as linhas de `primeira` a `ultima`; em cada uma, as colunas da primeira à última tocada.
        let (ya, yb) = (primeira / gw, ultima / gw);
        let linhas: Vec<(usize, usize, usize)> = (ya..=yb)
            .filter_map(|y| {
                let l = &tocada[y * gw..(y + 1) * gw];
                let a = l.iter().position(|t| *t)?;
                let b = l.iter().rposition(|t| *t).unwrap_or(a);
                Some((y, a, b))
            })
            .collect();
        let x0 = linhas.iter().map(|l| l.1).min().unwrap_or(0);
        let x1 = linhas.iter().map(|l| l.2).max().unwrap_or(0);
        self.marca_ret(x0 as i64 + 1, ya as i64 + 1, x1 as i64 + 2, yb as i64 + 2);
        // ⛔ `active_grid_mut` marcaria a grade inteira: 662 MB por enterro a 4096×2560.
        let g = &mut self.layers[self.active_layer].grid;
        let s = g.s;
        for &(y, a, b) in &linhas {
            #[cfg(test)]
            enterro_tests::VISITADAS.with(|v| v.set(v.get() + gw + b + 1 - a));
            for x in (a..=b).filter(|x| tocada[y * gw + x]) {
                crate::grid::enterra_celula(g, (y + 1) * s + x + 1);
            }
        }
    }

    /// Todos os registos de undo e redo, de todas as camadas, numa ordem que só muda com a história.
    pub fn registos_mut(&mut self) -> Vec<&mut crate::grid::Registo> {
        self.layers
            .iter_mut()
            .flat_map(|l| l.undo.iter_mut().chain(&mut l.redo).chain(&mut l.desde))
            .collect()
    }

    /// Larga o registo de undo mais velho da camada activa e devolve os bytes que ele retinha.
    pub fn solta_o_registo_mais_velho(&mut self) -> usize {
        let undo = &mut self.layers[self.active_layer].undo;
        if undo.is_empty() {
            return 0;
        }
        undo.remove(0).bytes_retidos()
    }

    /// O motor esquece os registos de undo: o que eles guardam já não descreve a história.
    pub fn esquece_a_historia(&mut self) {
        let layer = &mut self.layers[self.active_layer];
        layer.undo.clear();
        layer.redo.clear();
        layer.desde = None;
    }

    pub fn can_undo(&self) -> bool {
        !self.layers[self.active_layer].undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.layers[self.active_layer].redo.is_empty()
    }

    /// O passo da água fora das grades, agora (o que um registo guarda ao abrir).
    fn passo_em_voo(&self) -> PassoEmVoo {
        PassoEmVoo {
            cursor: self.sim.cursor_do_passo(),
        }
    }

    /// Repõe o passo de um registo restaurado: os estágios que faltavam correm, sobre a grade dele.
    fn repoe_passo(&mut self, p: &PassoEmVoo) {
        self.sim.repoe_cursor_do_passo(p.cursor.clone());
    }

    /// Desfaz o último passo; sem nada a desfazer não mexe em nada (nem no ritmo da água).
    pub fn undo(&mut self) -> bool {
        self.undo_com(true)
    }

    /// O [`Self::undo`] que deixa o fluxo (`flow_x`/`flow_y`) da camada activa por zerar: só para quem o REESCREVE
    /// inteiro logo a seguir, antes de qualquer leitura (o undo vivo do Painter repõe-no da fotografia).
    pub fn undo_sem_zerar_o_fluxo(&mut self) -> bool {
        self.undo_com(false)
    }

    fn undo_com(&mut self, zera_o_fluxo: bool) -> bool {
        if !self.can_undo() {
            return false;
        }
        crate::sonda_fases::abre();
        // O «depois» que o redo devolve é o passo TERMINADO (o undo_vivo fotografa o resto no mesmo instante).
        self.drain_step();
        let agora = self.passo_em_voo();
        let caixa = self.caixa_do_restauro(false);
        let forma = forma_global(&self.layers[self.active_layer].grid);
        let layer = &mut self.layers[self.active_layer];
        let Some(mut reg) = layer.undo.pop() else {
            return false;
        };
        crate::sonda_fases::marca("motor: drain + caixa + forma");
        // O registo estava aberto: o grid só difere da base dele nos tiles que ele guarda — trocam-se com ela.
        let (mut inv, paper_changed) = reg.troca(&mut layer.grid, None, zera_o_fluxo);
        inv.passo = agora;
        layer.redo.push(inv);
        layer.desde = Some(Registo::abre(&layer.grid));
        self.repoe_passo(&reg.passo);
        crate::sonda_fases::marca("motor: desde + passo");
        drop(reg);
        crate::sonda_fases::marca("motor: larga o registo");
        self.depois_do_restauro(paper_changed, caixa, forma);
        crate::sonda_fases::marca("motor: depois_do_restauro");
        true
    }

    /// Refaz; o passo em voo de agora entra no registo inverso (um undo devolve-o) e o do registo é reposto.
    pub fn redo(&mut self) -> bool {
        self.redo_com(true)
    }

    /// O [`Self::redo`] que deixa o fluxo por zerar — o mesmo contrato do [`Self::undo_sem_zerar_o_fluxo`].
    pub fn redo_sem_zerar_o_fluxo(&mut self) -> bool {
        self.redo_com(false)
    }

    fn redo_com(&mut self, zera_o_fluxo: bool) -> bool {
        crate::sonda_fases::abre();
        let agora = self.passo_em_voo();
        let caixa = self.caixa_do_restauro(true);
        let forma = forma_global(&self.layers[self.active_layer].grid);
        let fundo = self.fundo_da_historia;
        let layer = &mut self.layers[self.active_layer];
        let Some(mut reg) = layer.redo.pop() else {
            return false;
        };
        crate::sonda_fases::marca("motor: drain + caixa + forma");
        // Desde o último evento o grid andou nos tiles de `desde`; fora deles e dos do registo,
        // ele É a base do registo.
        let mut desde = layer.desde.take();
        let (mut inv, paper_changed) = reg.troca(&mut layer.grid, desde.as_mut(), zera_o_fluxo);
        inv.passo = agora;
        layer.undo.push(inv);
        if layer.undo.len() > fundo {
            layer.undo.remove(0);
        }
        layer.desde = (!layer.redo.is_empty()).then(|| Registo::abre(&layer.grid));
        self.repoe_passo(&reg.passo);
        crate::sonda_fases::marca("motor: desde + passo");
        drop((reg, desde));
        crate::sonda_fases::marca("motor: larga o registo");
        self.depois_do_restauro(paper_changed, caixa, forma);
        crate::sonda_fases::marca("motor: depois_do_restauro");
        true
    }

    /// B12.2: o sujo é a CAIXA do que o restauro escreveu (`caixa`, de antes dele); `Full` só se o papel mudou ou um
    /// estado de toda a camada ([`forma_global`]) mudou. Passo vazio (sem caixa) não suja nada.
    fn depois_do_restauro(
        &mut self,
        paper_changed: bool,
        caixa: Option<[usize; 4]>,
        forma: FormaGlobal,
    ) {
        if paper_changed {
            let g = &self.layers[self.active_layer].grid;
            self.paper_preset = g.paper_preset;
            self.paper_sheet = g.paper_sheet;
            self.rebake_paper();
        }
        let g = &self.layers[self.active_layer].grid;
        if forma != forma_global(g) {
            return self.mark_dirty_full();
        }
        if let Some([x0, y0, x1, y1]) = caixa {
            let (w, h) = (g.w as i32, g.h as i32);
            let (x0, y0) = ((x0 as i32).max(1), (y0 as i32).max(1));
            let (x1, y1) = ((x1 as i32 - 1).min(w), (y1 as i32 - 1).min(h));
            if x0 <= x1 && y0 <= y1 {
                self.merge_dirty(x0, y0, x1, y1);
            }
        }
    }

    /// A caixa (células com o pad, meio-aberta) de tudo o que o próximo `undo`/`redo` escreve na grade. `None`: nada.
    #[must_use]
    pub fn caixa_do_restauro(&self, refaz: bool) -> Option<[usize; 4]> {
        let l = &self.layers[self.active_layer];
        let (reg, extra) = if refaz {
            (l.redo.last()?, l.desde.as_ref())
        } else {
            (l.undo.last()?, None)
        };
        let g = &l.grid;
        let a = reg.caixa(g);
        match (a, extra.and_then(|e| e.caixa(g))) {
            (Some(a), Some(b)) => Some([
                a[0].min(b[0]),
                a[1].min(b[1]),
                a[2].max(b[2]),
                a[3].max(b[3]),
            ]),
            (a, b) => a.or(b),
        }
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
