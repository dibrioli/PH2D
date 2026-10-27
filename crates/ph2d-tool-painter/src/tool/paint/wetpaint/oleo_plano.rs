//! **O papel liso da sombra do Óleo Real** (W2, 27/10). `sombra_no_papel` de uma célula só lê o campo de altura
//! até [`MARGEM`] à volta (a marcha da luz, 24 + 1 da bilinear; o horizonte, ≤ 24); onde esse campo é todo `+0`
//! o resultado é o MESMO em qualquer célula — basta uma conta. Medido (`sonda_custo_oleo`, cena `gestos` da bancada,
//! 1000×640, Filbert): na água parada o dirty do motor é a caixa molhada inteira (533 mil células, 606 mil com o
//! alcance) e a sombra no papel custava 2,9× o pigmento com óleo (35,7 × 12,2 s em 160 quadros); com o atalho
//! 25,3 s (o resto da caixa tem tinta a menos de `MARGEM`). Bit a bit a conta por célula, por construção (gate
//! `o_papel_liso_da_sombra_e_bit_identico_a_conta_por_celula`).
use ph2d_wet_paint::oleo_real::{CampoAltura, MARGEM};

/// A soma 2D das células NÃO lisas na região alargada pela margem: uma célula é lisa quando nenhuma da caixa
/// `±MARGEM` em volta (presa à grade) tem altura fina ou de sombra diferente de `+0`.
pub(super) struct PapelLiso {
    z: (usize, usize, usize, usize),
    sw: usize,
    soma: Vec<u32>,
}

impl PapelLiso {
    /// `campo` tem de ser exacto até `MARGEM` à volta da região `[x0..=x1]×[y0..=y1]` (1-based) — o do composite é.
    pub(super) fn de(
        campo: &CampoAltura,
        (x0, y0, x1, y1): (usize, usize, usize, usize),
        (gw, gh): (usize, usize),
    ) -> Self {
        let r = MARGEM;
        let (zx0, zy0) = (x0.saturating_sub(r).max(1), y0.saturating_sub(r).max(1));
        let (zx1, zy1) = ((x1 + r).min(gw), (y1 + r).min(gh));
        let (zw, zh) = (zx1 + 1 - zx0, zy1 + 1 - zy0);
        let sw = zw + 1;
        let mut soma = vec![0u32; sw * (zh + 1)];
        for y in 0..zh {
            let mut linha = 0u32;
            for x in 0..zw {
                let (gx, gy) = ((zx0 + x) as isize, (zy0 + y) as isize);
                let alto =
                    campo.at_fina(gx, gy).to_bits() != 0 || campo.at_sombra(gx, gy).to_bits() != 0;
                linha += u32::from(alto);
                soma[(x + 1) + (y + 1) * sw] = soma[(x + 1) + y * sw] + linha;
            }
        }
        Self {
            z: (zx0, zy0, zx1, zy1),
            sw,
            soma,
        }
    }

    /// A célula `(x, y)` da região só vê papel liso até ao alcance da óptica.
    pub(super) fn liso(&self, x: usize, y: usize) -> bool {
        let (zx0, zy0, zx1, zy1) = self.z;
        let r = MARGEM;
        let (bx0, bx1) = (
            x.saturating_sub(r).max(zx0) - zx0,
            (x + r).min(zx1) - zx0 + 1,
        );
        let (by0, by1) = (
            y.saturating_sub(r).max(zy0) - zy0,
            (y + r).min(zy1) - zy0 + 1,
        );
        let s = |a: usize, b: usize| self.soma[a + b * self.sw];
        s(bx1, by1) + s(bx0, by0) - s(bx0, by1) - s(bx1, by0) == 0
    }
}

/// O atalho desligado (só sob `cfg(test)`): toda célula conta-se — a referência do gate e o «antes» da sonda.
#[cfg(test)]
pub(in crate::tool::paint) mod desligado {
    use std::cell::Cell;

    thread_local! {
        static D: Cell<bool> = const { Cell::new(false) };
    }

    pub(in crate::tool::paint) fn set(v: bool) {
        D.with(|c| c.set(v));
    }

    pub(in crate::tool::paint) fn get() -> bool {
        D.with(Cell::get)
    }
}

/// O campo de altura do Óleo Real com as linhas de cada passagem repartidas pelo rayon (ADR-0109: o mesmo map
/// por linhas do composite): o mesmo campo bit a bit que o em série (gate
/// `o_campo_em_faixas_e_bit_identico_ao_em_serie` em `ph2d-wet-paint`, e o de ponta a ponta
/// `as_curas_de_custo_do_oleo_sao_bit_identicas_ao_caminho_antigo`).
pub(super) struct EmFaixas;

impl ph2d_wet_paint::oleo_real::Reparte for EmFaixas {
    fn por_linhas<A: Send, B: Send>(
        &self,
        w: usize,
        a: &mut [A],
        b: &mut [B],
        f: &(dyn Fn(usize, &mut [A], &mut [B]) + Sync),
    ) {
        use rayon::prelude::*;
        a.par_chunks_mut(w)
            .zip(b.par_chunks_mut(w))
            .enumerate()
            .for_each(|(y, (la, lb))| f(y, la, lb));
    }
}

/// As duas curas de custo de 27/10 desligadas (só sob `cfg(test)`): `(sombra da tinta pela conta própria, campo
/// em série)` — a referência do gate e o «antes» da sonda.
#[cfg(test)]
pub(in crate::tool::paint) mod curas_desligadas {
    use std::cell::Cell;

    thread_local! {
        static D: Cell<(bool, bool)> = const { Cell::new((false, false)) };
    }

    pub(in crate::tool::paint) fn set(sombra: bool, campo: bool) {
        D.with(|c| c.set((sombra, campo)));
    }

    pub(in crate::tool::paint) fn get() -> (bool, bool) {
        D.with(Cell::get)
    }
}
