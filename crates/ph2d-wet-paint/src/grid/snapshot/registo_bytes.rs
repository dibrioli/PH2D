//! Um [`Registo`] do histórico em bytes, ao bit, para a tinta adormecida o comprimir. Com a lei arquivada no registo
//! (`tinta`, `pigmento`) ele fica como está.

use super::super::celula_bytes::{Celula, de_bytes, para_bytes};
use super::{Planos, Registo, TileFino};

fn escreve<T: Celula>(v: &[T], s: &mut Vec<u8>) {
    s.extend_from_slice(&(v.len() as u64).to_le_bytes());
    para_bytes(v, s);
}

fn le_u64(b: &mut &[u8]) -> u64 {
    let (a, resto) = b.split_at(8);
    *b = resto;
    u64::from_le_bytes(a.try_into().expect("8 bytes"))
}

fn le<T: Celula>(b: &mut &[u8]) -> Vec<T> {
    let n = le_u64(b) as usize;
    let (a, resto) = b.split_at(n * T::N * 4);
    *b = resto;
    de_bytes(a)
}

fn le_u8(b: &mut &[u8]) -> Vec<u8> {
    let n = le_u64(b) as usize;
    let (a, resto) = b.split_at(n);
    *b = resto;
    a.to_vec()
}

fn escreve_u8(v: &[u8], s: &mut Vec<u8>) {
    s.extend_from_slice(&(v.len() as u64).to_le_bytes());
    s.extend_from_slice(v);
}

impl Registo {
    /// Tira os tiles do registo para bytes. `None` = fica como está: não há tiles, ou há a lei arquivada.
    pub fn em_bytes(&mut self) -> Option<Vec<u8>> {
        if (self.finos.is_empty() && self.fluxo.is_empty())
            || self
                .finos
                .values()
                .any(|t| !t.tinta.is_empty() || !t.pigmento.is_empty())
        {
            return None;
        }
        let mut s = Vec::with_capacity(self.bytes_retidos());
        s.extend_from_slice(&(self.finos.len() as u64).to_le_bytes());
        for (k, t) in std::mem::take(&mut self.finos) {
            s.extend_from_slice(&u64::from(k).to_le_bytes());
            let TileFino {
                film,
                susp,
                susp_rgb,
                sett,
                sett_rgb,
                relevo,
                altura,
                espessura,
                crista,
                nivelado,
                vela,
                material,
                aparencia,
                oleo,
                tinta: _,
                pigmento: _,
                wet,
                active,
                bloom,
                guardados,
            } = t;
            s.extend_from_slice(&guardados.bits().to_le_bytes());
            for p in [
                &film, &susp, &sett, &relevo, &altura, &espessura, &crista, &nivelado, &vela,
            ] {
                escreve(p, &mut s);
            }
            escreve(&susp_rgb, &mut s);
            escreve(&sett_rgb, &mut s);
            escreve(&material, &mut s);
            escreve(&aparencia, &mut s);
            escreve(&oleo, &mut s);
            for p in [&wet, &active, &bloom] {
                escreve_u8(p, &mut s);
            }
        }
        s.extend_from_slice(&(self.fluxo.len() as u64).to_le_bytes());
        for (k, [a, b]) in std::mem::take(&mut self.fluxo) {
            s.extend_from_slice(&u64::from(k).to_le_bytes());
            escreve(&a, &mut s);
            escreve(&b, &mut s);
        }
        Some(s)
    }

    /// Devolve os tiles que [`Self::em_bytes`] tirou.
    pub fn de_bytes(&mut self, mut b: &[u8]) {
        let b = &mut b;
        for _ in 0..le_u64(b) {
            let k = le_u64(b) as u32;
            let (m, resto) = b.split_at(4);
            *b = resto;
            let guardados = Planos::de_bits(u32::from_le_bytes(m.try_into().expect("4 bytes")));
            let (film, susp, sett, relevo, altura, espessura, crista, nivelado, vela) = (
                le(b),
                le(b),
                le(b),
                le(b),
                le(b),
                le(b),
                le(b),
                le(b),
                le(b),
            );
            let (susp_rgb, sett_rgb, material, aparencia, oleo) =
                (le(b), le(b), le(b), le(b), le(b));
            let (wet, active, bloom) = (le_u8(b), le_u8(b), le_u8(b));
            self.finos.insert(
                k,
                TileFino {
                    film,
                    susp,
                    susp_rgb,
                    sett,
                    sett_rgb,
                    relevo,
                    altura,
                    espessura,
                    crista,
                    nivelado,
                    vela,
                    material,
                    aparencia,
                    oleo,
                    tinta: Vec::new(),
                    pigmento: Vec::new(),
                    wet,
                    active,
                    bloom,
                    guardados,
                },
            );
        }
        for _ in 0..le_u64(b) {
            let k = le_u64(b) as u32;
            let (x, y) = (le(b), le(b));
            self.fluxo.insert(k, [x, y]);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::painter::Engine;

    /// A ida e volta de um registo real é a mesma, medida pelo que ele restaura: o undo depois da volta dá a grade do
    /// undo sem a ida.
    #[test]
    fn um_registo_vai_e_volta_ao_bit() {
        let pinta = || {
            let mut e = Engine::new(128, 128);
            e.capture_history();
            e.begin_direct_stroke(0, 30.0, 30.0);
            for k in 0..20 {
                e.dispatch_pressure_dab_erase(30.0 + f64::from(k), 30.0, 0.5, 1.0, 8.0, None, None);
            }
            e.end_direct_stroke();
            e
        };
        let (mut a, mut b) = (pinta(), pinta());
        let mut guardados: Vec<Option<Vec<u8>>> = Vec::new();
        for r in b.registos_mut() {
            guardados.push(r.em_bytes());
        }
        assert!(guardados.iter().any(Option::is_some), "CONTROLO: há tiles");
        for (r, g) in b.registos_mut().into_iter().zip(&guardados) {
            if let Some(g) = g {
                r.de_bytes(g);
            }
        }
        assert_eq!(a.undo(), b.undo());
        let (ga, gb) = (&a.layers[0].grid, &b.layers[0].grid);
        let bits = |v: &[f32]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
        assert_eq!(bits(&ga.sett), bits(&gb.sett));
        assert_eq!(bits(&ga.film), bits(&gb.film));
        assert_eq!(ga.wet, gb.wet);
    }
}
