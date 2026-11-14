//! A fotografia do motor FORA das grades (óleo de antes, fluxo, relógio) — filho de [`super`] por LOC (B52.R11).

use super::super::foto_em_tiles::PlanoEmTiles;
use ph2d_wet_paint::painter::Engine;

/// O estado do motor fora das grades que a física lê entre dois traços (a cadência da secagem, o nº do traço).
#[derive(Clone, Copy)]
pub(crate) struct Relogio {
    frame: u64,
    dry_every: u64,
    evap_scale: f64,
    rewet_base: f64,
    semente_do_po: u32,
}

impl Relogio {
    fn de(e: &Engine) -> Self {
        let s = &e.sim;
        Self {
            frame: s.frame,
            dry_every: s.dry_every,
            evap_scale: s.evap_scale,
            rewet_base: s.rewet_base,
            semente_do_po: s.semente_do_po,
        }
    }

    fn poe(self, e: &mut Engine) {
        let s = &mut e.sim;
        s.frame = self.frame;
        s.dry_every = self.dry_every;
        s.evap_scale = self.evap_scale;
        s.rewet_base = self.rewet_base;
        s.semente_do_po = self.semente_do_po;
    }
}

/// O motor num instante, FORA das grades (essas voltam pelo histórico do motor): a ladeira do óleo, o fluxo e o
/// relógio.
#[derive(Clone)]
pub(crate) struct FotoDoMotor {
    oleo_antes: Vec<PlanoEmTiles>,
    /// O fluxo do último passo: o restauro zera-o, mas o Jacobi parte dele (medido: 13 B na aguada seguinte).
    fluxo: Vec<(PlanoEmTiles, PlanoEmTiles)>,
    relogio: Relogio,
}

impl FotoDoMotor {
    /// Fotografa o motor, partilhando com `anterior` os tiles que não mudaram.
    pub(super) fn de(e: &Engine, anterior: Option<&Self>) -> Self {
        let ate = |i: usize| anterior.and_then(|a| a.oleo_antes.get(i));
        let fluxo = |i: usize| anterior.and_then(|a| a.fluxo.get(i));
        Self {
            oleo_antes: e
                .layers
                .iter()
                .enumerate()
                .map(|(i, l)| PlanoEmTiles::de(&l.grid.oleo_antes, l.grid.s, ate(i)))
                .collect(),
            fluxo: e
                .layers
                .iter()
                .enumerate()
                .map(|(i, l)| {
                    (
                        // O fluxo mora na grade de FLUXO: outra largura que a fina, se `flow_ratio` > 1.
                        PlanoEmTiles::de(&l.grid.flow_x, l.grid.flow.s, fluxo(i).map(|f| &f.0)),
                        PlanoEmTiles::de(&l.grid.flow_y, l.grid.flow.s, fluxo(i).map(|f| &f.1)),
                    )
                })
                .collect(),
            relogio: Relogio::de(e),
        }
    }

    /// Repõe o que fica fora das grades — DEPOIS do `undo`/`redo` do motor (que zera o fluxo). Nada disto é lido pela
    /// óptica (o espelho ignora `oleo_antes` e o fluxo): o sujo é o que o restauro do motor marcou (B12.2).
    #[cfg(test)]
    pub(super) fn repoe(&self, e: &mut Engine) {
        self.repoe_sobre(e, None);
    }

    /// O [`Self::repoe`] sobre um motor cujos planos são `ja` ao bit (fotografado dele, sem escrita nesses planos no
    /// meio): os tiles partilhados com `ja` não se reescrevem.
    pub(super) fn repoe_sobre(&self, e: &mut Engine, ja: Option<&Self>) {
        for (i, (l, o)) in e.layers.iter_mut().zip(&self.oleo_antes).enumerate() {
            o.repoe_sobre(&mut l.grid.oleo_antes, ja.and_then(|j| j.oleo_antes.get(i)));
        }
        for (i, (l, (fx, fy))) in e.layers.iter_mut().zip(&self.fluxo).enumerate() {
            let j = ja.and_then(|j| j.fluxo.get(i));
            fx.repoe_sobre(&mut l.grid.flow_x, j.map(|j| &j.0));
            fy.repoe_sobre(&mut l.grid.flow_y, j.map(|j| &j.1));
        }
        self.relogio.poe(e);
    }

    /// Se o [`Self::repoe`] reescreve o fluxo da camada `i` inteiro (cópia de todos os tiles ou esvaziamento), sem ler o
    /// que lá estava — a condição para o undo do motor não o zerar antes.
    pub(super) fn repoe_o_fluxo_de(&self, i: usize) -> bool {
        i < self.fluxo.len()
    }

    fn planos(&self) -> impl Iterator<Item = &PlanoEmTiles> {
        self.oleo_antes
            .iter()
            .chain(self.fluxo.iter().flat_map(|(x, y)| [x, y]))
    }

    /// Os bytes da fotografia sozinha.
    pub(crate) fn bytes(&self) -> usize {
        self.planos().map(PlanoEmTiles::bytes).sum()
    }

    /// Os bytes que ela não partilha com a anterior.
    pub(super) fn bytes_novos(&self) -> usize {
        self.planos().map(PlanoEmTiles::bytes_novos).sum()
    }

    pub(super) fn planos_mut(&mut self) -> Vec<&mut PlanoEmTiles> {
        let mut v: Vec<&mut PlanoEmTiles> = self.oleo_antes.iter_mut().collect();
        for (x, y) in &mut self.fluxo {
            v.push(x);
            v.push(y);
        }
        v
    }
}

#[cfg(test)]
impl FotoDoMotor {
    pub(crate) fn de_para_sonda(e: &ph2d_wet_paint::painter::Engine) -> Self {
        Self::de(e, None)
    }

    pub(crate) fn repoe_para_sonda(&self, e: &mut ph2d_wet_paint::painter::Engine) {
        self.repoe(e);
    }

    /// Os bytes que a fotografia do motor `e` seguraria, sem a tirar.
    pub(crate) fn bytes_de_para_sonda(e: &ph2d_wet_paint::painter::Engine) -> usize {
        use std::mem::size_of_val as b;
        e.layers
            .iter()
            .map(|l| {
                let g = &l.grid;
                b(&g.oleo_antes[..]) + b(&g.flow_x[..]) + b(&g.flow_y[..])
            })
            .sum()
    }
}
