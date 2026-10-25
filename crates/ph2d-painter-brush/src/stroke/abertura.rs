//! ⭐ **A abertura do traço sob o Continuous com o Accumulate desligado e o Flow < 1** (dono 25/10, R1: «o primeiro
//! carimbo do traço usando Continuous fica muito marcado»). Cada sub-dab de uma cápsula leva a taxa de UM sub-dab,
//! `1 − (1 − flow)^(1/n)`; o dab de abertura, sem passo, levava o Flow INTEIRO por baixo da 1.ª cápsula — medido
//! no Glaze Round a 40 px e Flow 0,3: o início `+23 %` sobre o miolo a Spacing 0,4, `+74 %` a 1,0.
//!
//! A cura: o dab de abertura (e as suas cópias: Symmetry, Spray, Rough) NÃO se carimba; a 1.ª cápsula leva-o como o
//! sub-dab de trás ([`crate::Ante::fecho`]), à taxa de um sub-dab — o traço denso começa no ponto do pen-down, como
//! acaba no do último dab. Sem cápsula a seguir (um toque, um passo curto demais), os dabs retidos carimbam-se
//! como eram.

use super::Stroke;
use crate::Dab;

#[derive(Clone, Debug, Default)]
pub(super) struct Abertura {
    /// O 1.º passo do traço ainda não foi construído: o `advance_step` marca-o com o fecho.
    pub(super) fecho_por_dar: bool,
    /// Os dabs de abertura retidos, até se saber se a 1.ª cápsula os leva.
    retidos: Vec<Dab>,
    /// Ainda à espera do 1.º dab com passo.
    a_reter: bool,
}

impl Stroke {
    /// O traço em que a abertura vai na 1.ª cápsula — onde a taxa por sub-dab vale (`dab/bands.rs`, `c_cap`).
    fn abertura_na_capsula(&self) -> bool {
        let s = &self.spec;
        s.leis_do_digital() && s.lays_capsules() && !s.accumulate && !s.wash && s.flow < 1.0
    }

    /// Arma a abertura num traço (ou num preenchimento de forma) novo.
    pub(super) fn arma_a_abertura(&mut self) {
        let on = self.abertura_na_capsula();
        self.abertura = Abertura {
            fecho_por_dar: on,
            retidos: Vec::new(),
            a_reter: on,
        };
    }

    /// Passa `out[de..]` pela abertura: retém os dabs sem passo até ao 1.º com passo; se ele os leva (cápsula com
    /// fecho), descarta-os, senão devolve-os antes dele. `fim` = o traço acabou: os retidos carimbam-se.
    pub(super) fn passa_a_abertura(&mut self, out: &mut Vec<Dab>, de: usize, fim: bool) {
        if !self.abertura.a_reter {
            return;
        }
        let novos: Vec<Dab> = out.drain(de..).collect();
        for d in novos {
            if self.abertura.a_reter {
                if d.step == [0.0, 0.0] {
                    self.abertura.retidos.push(d);
                    continue;
                }
                self.abertura.a_reter = false;
                let leva = d.ante.is_some_and(|a| a.fecho) && d.capsula(&self.spec).is_some();
                if leva {
                    self.abertura.retidos.clear();
                } else {
                    out.append(&mut self.abertura.retidos);
                }
            }
            out.push(d);
        }
        if fim && self.abertura.a_reter {
            self.abertura.a_reter = false;
            out.append(&mut self.abertura.retidos);
        }
    }
}
