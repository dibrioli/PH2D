//! O restauro e o inverso do histórico de ANTES (tiles em série, recontagens célula a célula), verbatim: os oráculos
//! do gate `o_restauro_por_plano_da_o_bit_do_restauro_em_serie` e o lado A da sonda (ADR-0204).

use super::*;

impl Registo {
    /// O [`Self::inverso`] de antes (tile a tile, serial): o oráculo dele.
    pub(super) fn inverso_antigo(g: &Grid, a: &Registo, b: Option<&Registo>) -> Self {
        let mut out = Registo::abre(g);
        for k in a
            .finos
            .keys()
            .chain(b.into_iter().flat_map(|b| b.finos.keys()))
        {
            out.finos
                .entry(*k)
                .or_insert_with(|| TileFino::tira(g, ret_do_tile(*k, g.s, g.rows)));
        }
        let f = &g.flow;
        for k in a
            .fluxo
            .keys()
            .chain(b.into_iter().flat_map(|b| b.fluxo.keys()))
        {
            out.fluxo.entry(*k).or_insert_with(|| {
                let r = ret_do_tile(*k, f.s, f.rows);
                [tira(&g.vel_x, f.s, r), tira(&g.vel_y, f.s, r)]
            });
        }
        out
    }

    /// O [`Self::restaura_com`] de antes (tiles em série, recontagens célula a célula): o oráculo dele.
    pub(super) fn restaura_com_antigo(
        &self,
        g: &mut Grid,
        extra: Option<&Registo>,
        zera_o_fluxo: bool,
    ) -> bool {
        let tiles = || {
            self.finos.iter().chain(
                extra
                    .into_iter()
                    .flat_map(|e| e.finos.iter().filter(|(k, _)| !self.finos.contains_key(k))),
            )
        };
        let v = self.base.vazios;
        macro_rules! amostra {
            ($f:ident) => {
                tiles().find_map(|(_, t)| t.$f.first().cloned())
            };
        }
        let (a0, a1, a2, a3) = (
            amostra!(altura),
            amostra!(espessura),
            amostra!(crista),
            amostra!(nivelado),
        );
        let (a5, a6, a7) = (amostra!(material), amostra!(aparencia), amostra!(oleo));
        ajusta(&mut g.altura, v[0], a0.as_ref());
        ajusta(&mut g.espessura, v[1], a1.as_ref());
        ajusta(&mut g.crista, v[2], a2.as_ref());
        ajusta(&mut g.nivelado, v[3], a3.as_ref());
        ajusta_campo(&mut g.material, v[4], a5.is_some());
        ajusta_campo(&mut g.aparencia, v[5], a6.is_some());
        ajusta_campo(&mut g.oleo, v[6], a7.is_some());
        let a8 = amostra!(tinta);
        ajusta(&mut g.tinta, v[7], a8.as_ref());
        let a9 = amostra!(pigmento);
        ajusta(&mut g.pigmento, v[8], a9.as_ref());
        for (k, t) in tiles() {
            t.poe(g, ret_do_tile(*k, g.s, g.rows));
        }
        // Contam o que os tiles acabaram de pôr: antes deles, um plano recém-criado ainda está neutro.
        super::aparencia::reconta_teto_antigo(g);
        super::oleo_celula::reconta_real_antigo(g);
        let fluxo = self.fluxo.iter().chain(
            extra
                .into_iter()
                .flat_map(|e| e.fluxo.iter().filter(|(k, _)| !self.fluxo.contains_key(k))),
        );
        let (fs, frows) = (g.flow.s, g.flow.rows);
        for (k, [vx, vy]) in fluxo {
            let r = ret_do_tile(*k, fs, frows);
            poe(&mut g.vel_x, fs, r, vx);
            poe(&mut g.vel_y, fs, r, vy);
        }
        if zera_o_fluxo {
            g.flow_x.fill(0.0);
            g.flow_y.fill(0.0);
        }
        let b = &self.base;
        g.bx0 = b.bx0;
        g.by0 = b.by0;
        g.bx1 = b.bx1;
        g.by1 = b.by1;
        g.has_fluid = b.has_fluid;
        g.row_lo.copy_from_slice(&b.row_lo);
        g.row_hi.copy_from_slice(&b.row_hi);
        g.fill_tiles_all();
        let paper_changed = g.paper_preset != b.paper_preset || g.paper_sheet != b.paper_sheet;
        g.paper_preset = b.paper_preset;
        g.paper_sheet = b.paper_sheet;
        paper_changed
    }
}
