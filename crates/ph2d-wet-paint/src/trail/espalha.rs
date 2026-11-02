//! **O óleo molhado ESPALHA o relevo de baixo** (queixa do dono 31/10: «grumos exagerados» com pinceladas
//! sobrepostas; sonda `ph2d-tool-painter` `sonda_oleo_grumos`). A espessura ([`crate::grid::espessura`]) recebe o
//! `v` inteiro de cada dab sem tecto (`transfer.rs`), por isso o relevo crescia LINEAR com as passadas (Filbert:
//! 2,0 · 4,8 · 8,9 · 14,0 px a N = 1·3·6·10; Metallic Flat, de massa quase constante, 3,7 → 31,2 px). Num óleo real
//! a tinta nova ARRASTA a de baixo: no PRIMEIRO toque de um traço numa célula com tinta de traços anteriores, a
//! espessura que lá estava baixa a `(1 − ESPALHA)` dela antes de a tinta nova pousar. Com `E` por passada o relevo
//! tende a `E ÷ ESPALHA` (satura), e o traço único sobre a tela limpa não muda um bit (nenhuma célula tem tinta
//! antes do 1.º toque).
//!
//! ⛔ **O espalhar nunca CAVA** (Mecanismo 2 de `docs/Painter/Arquitetura/RELEVO_oleo_ondas_e_crateras.md`, 31/10): o
//! que o 1.º toque leva é o que o pincel RECOLHE, e um pincel carregado só recolhe o que deposita — a célula acaba em
//! `max(t₀, t₀·(1 − ESPALHA) + Σv)`, nunca abaixo da altura `t₀` que tinha antes do traço ([`Espalha::piso`]). Sem o
//! piso o traço 2 meio sobreposto a um óleo grosso deixava 2 609 crateras no contacto e −42 % do volume. Abaixo da
//! saturação (`t₀ ≤ Σv ÷ ESPALHA`) é a lei de antes, ao bit.

use crate::grid::Grid;
use std::sync::atomic::{AtomicU64, Ordering};

/// A fracção da espessura de baixo que o primeiro toque de um traço novo leva (medida na sonda, 31/10).
pub const ESPALHA: f64 = 0.5;

static SOBREPOSTO: AtomicU64 = AtomicU64::new(u64::MAX);

/// Só para as sondas: força a fracção (`None` = [`ESPALHA`]); `Some(0.0)` = o comportamento antigo.
pub fn sobrepoe(f: Option<f64>) {
    SOBREPOSTO.store(f.map_or(u64::MAX, f64::to_bits), Ordering::Relaxed);
}

fn fraccao() -> f64 {
    match SOBREPOSTO.load(Ordering::Relaxed) {
        u64::MAX => ESPALHA,
        b => f64::from_bits(b),
    }
}

/// As células que o traço vivo já tocou (vazio até ao 1.º toque sobre tinta com espessura).
#[derive(Debug, Clone, Default)]
pub(super) struct Espalha {
    tocada: Vec<bool>,
    /// A espessura que cada célula tocada tinha ANTES do traço (`0` nas virgens) — o piso do depósito.
    piso: Vec<f32>,
    /// O que o piso já ERGUEU na célula neste traço (sai do cru do dab seguinte).
    erguido: Vec<f32>,
    sujo: bool,
}

impl Espalha {
    pub(super) fn traco_novo(&mut self) {
        if self.sujo {
            self.tocada.fill(false);
            self.piso.fill(0.0);
            self.erguido.fill(0.0);
            self.sujo = false;
        }
    }

    /// Antes de o dab do pincel de Real Oil pousar na célula `i`: no 1.º toque do traço, a espessura que lá estava
    /// (de traços anteriores) espalha-se. A célula vazia também se marca: a tinta que o traço lá pousar é dele.
    #[inline]
    pub(super) fn primeiro_toque(&mut self, g: &mut Grid, i: usize) {
        if self.sujo && self.tocada.get(i).copied().unwrap_or(false) {
            return;
        }
        self.marca(g.cells, i);
        let m = f64::from(g.sett[i]) + f64::from(g.susp[i]);
        let a = fraccao();
        if m > 0.0 && a > 0.0 {
            let t = crate::grid::espessura::em(g, i);
            self.piso[i] = t as f32;
            crate::grid::espessura::nivela(g, i, t * (1.0 - a));
        }
    }

    /// A espessura da célula `i` depois de um dab de `v` sobre `atual`: `max(t₀, t₀·(1 − ESPALHA) + Σv)` sobre o
    /// ACUMULADO do traço (`com_piso`: o óleo, sem a sonda `SEM_PISO`), nunca por dab — o erguer do piso de um dab é
    /// devolvido pelos seguintes (`erguido`). Erguer `atual` ao piso a cada dab desfazia o espalhar e o relevo crescia
    /// linear com as passadas (`sonda_oleo_furos`: Thick Impasto p99 24,1 → 110,9 em 8 passadas; com a cura, 39,7).
    #[inline]
    pub(super) fn deposita(&mut self, i: usize, atual: f64, v: f64, com_piso: bool) -> f64 {
        let (Some(&piso), Some(erguido), true) =
            (self.piso.get(i), self.erguido.get_mut(i), com_piso)
        else {
            return atual + v;
        };
        let cru = atual - f64::from(*erguido) + v;
        let novo = cru.max(f64::from(piso));
        *erguido = (novo - cru) as f32;
        novo
    }

    fn marca(&mut self, n: usize, i: usize) {
        if self.tocada.len() != n {
            self.tocada = vec![false; n];
            self.piso = vec![0.0; n];
            self.erguido = vec![0.0; n];
        }
        self.tocada[i] = true;
        self.sujo = true;
    }
}
