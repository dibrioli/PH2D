//! B13.11 (DF-27): os `static` de MEDIÇÃO do solver (`LADEIRA_ROTA`, `PAPEL_BINGHAM`, `REMOLHA_TAXA`, `CHAO_*`…) mudam o
//! PROCESSO inteiro. Quem os mexe pede uma [`Regua`]: o valor de antes volta no `Drop`, também quando o teste reprova
//! (um `store(0)` no fim não corria depois do pânico e o teste seguinte do mesmo processo pintava com a rota errada).
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, Ordering::Relaxed};

/// Um atómico de régua: troca e repõe.
pub trait Atomo: Sync {
    /// O valor guardado.
    type V: Copy;
    /// Grava `v` e devolve o de antes.
    fn troca(&self, v: Self::V) -> Self::V;
    /// Repõe `v`.
    fn repoe(&self, v: Self::V);
}

impl Atomo for AtomicBool {
    type V = bool;
    fn troca(&self, v: bool) -> bool {
        self.swap(v, Relaxed)
    }
    fn repoe(&self, v: bool) {
        self.store(v, Relaxed);
    }
}

impl Atomo for AtomicU8 {
    type V = u8;
    fn troca(&self, v: u8) -> u8 {
        self.swap(v, Relaxed)
    }
    fn repoe(&self, v: u8) {
        self.store(v, Relaxed);
    }
}

impl Atomo for AtomicU32 {
    type V = u32;
    fn troca(&self, v: u32) -> u32 {
        self.swap(v, Relaxed)
    }
    fn repoe(&self, v: u32) {
        self.store(v, Relaxed);
    }
}

/// O valor de régua enquanto viver; o de antes volta no `Drop`.
#[must_use = "a régua repõe o valor quando sai de cena: guarde-a numa variável"]
pub struct Regua<A: Atomo + 'static> {
    a: &'static A,
    antes: A::V,
}

impl<A: Atomo + 'static> Regua<A> {
    /// Grava `v` em `a` até a régua cair.
    pub fn poe(a: &'static A, v: A::V) -> Self {
        let antes = a.troca(v);
        Self { a, antes }
    }

    /// Troca o valor durante a vida da régua (o de antes do [`Regua::poe`] continua a ser o que volta).
    pub fn muda(&self, v: A::V) {
        self.a.repoe(v);
    }
}

impl<A: Atomo + 'static> Drop for Regua<A> {
    fn drop(&mut self) {
        self.a.repoe(self.antes);
    }
}

/// O `f32` como os `static` de papel o guardam (os bits).
#[must_use]
pub fn bits(v: f32) -> u32 {
    v.to_bits()
}

#[cfg(test)]
mod tests {
    use super::*;

    static SONDA: AtomicU8 = AtomicU8::new(0);
    static PAPEL: AtomicU32 = AtomicU32::new(0);

    /// Red (o padrão de antes, `store` … `store(0)`): o pânico salta o `store(0)` e o processo fica na rota 2. A régua
    /// repõe o 0 no pânico, e o valor de antes (não o 0) quando já havia outro.
    #[test]
    fn a_regua_repoe_o_valor_mesmo_quando_o_teste_reprova() {
        let velho = std::panic::catch_unwind(|| {
            SONDA.store(2, Relaxed);
            panic!("o teste reprova");
            #[allow(unreachable_code)]
            SONDA.store(0, Relaxed);
        });
        assert!(velho.is_err());
        assert_eq!(SONDA.load(Relaxed), 2, "o padrão velho vaza a rota");
        SONDA.store(0, Relaxed);

        let novo = std::panic::catch_unwind(|| {
            let r = Regua::poe(&SONDA, 2);
            r.muda(3);
            assert_eq!(SONDA.load(Relaxed), 3);
            panic!("o teste reprova");
        });
        assert!(novo.is_err());
        assert_eq!(SONDA.load(Relaxed), 0);

        PAPEL.store(bits(0.5), Relaxed);
        {
            let _r = Regua::poe(&PAPEL, bits(-1.0));
            assert_eq!(f32::from_bits(PAPEL.load(Relaxed)), -1.0);
        }
        assert_eq!(f32::from_bits(PAPEL.load(Relaxed)), 0.5);
    }
}
