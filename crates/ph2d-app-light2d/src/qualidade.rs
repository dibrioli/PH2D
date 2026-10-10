//! ⭐ **A nitidez da luz e o governador** (pedido do dono, 2026-10-09: *«escolha no editor,
//! automática em runtime»*). Baixa · Média · Alta = células de 4 · 3 · 2 px de tela; com o
//! «Automático», o nível escolhido é o TETO e o editor desce quando a placa não aguenta.
//!
//! O recurso que limita é o TEMPO DE PLACA por quadro: a luz leva no máximo [`ORCAMENTO_MS`], um
//! terço do quadro a 60 Hz. Medido na W3 a 1080p (RTX 5060 Ti, `smoke`): células de 4 · 3 · 2 px =
//! 2,29 · 4,16 · 8,82 ms (margem 12,5 %) — o custo segue `1/célula²` (×1,82 e ×3,85 contra 1,78 e
//! 4). A subida é PREVISTA por essa lei a partir do custo medido, com folga, para não oscilar.

use ph2d_editor_core::luz_edits::NivelDaLuz;

use crate::grade::{DEGRAU_PADRAO, Degrau};

/// O teto do tempo de placa da luz: um terço do quadro a 60 Hz.
pub const ORCAMENTO_MS: f64 = 1000.0 / 60.0 / 3.0;
/// Só sobe se o nível acima, previsto, couber em 75 % do orçamento (a medida varia de quadro a
/// quadro; sem folga o governador subia e descia em ciclo).
const FOLGA_PARA_SUBIR: f64 = 0.75;
/// Leituras seguidas acima do orçamento antes de descer (um pico isolado não muda a imagem).
const LEITURAS_PARA_DESCER: u32 = 3;
/// Leituras seguidas com folga antes de subir (subir recria o solver: é raro de propósito).
const LEITURAS_PARA_SUBIR: u32 = 30;

/// O degrau da grade de cada nível (a margem e a camada do vetor são as do padrão medido).
#[must_use]
pub fn degrau_do(nivel: NivelDaLuz) -> Degrau {
    Degrau {
        c0_px: f64::from(nivel.celula_px()),
        ..DEGRAU_PADRAO
    }
}

fn vizinho(n: NivelDaLuz, passo: isize) -> Option<NivelDaLuz> {
    let i = NivelDaLuz::ALL.iter().position(|&x| x == n)?;
    NivelDaLuz::ALL.get(i.checked_add_signed(passo)?).copied()
}

/// ⭐ O governador: lê o custo medido e decide o nível, sem passar do teto.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Governador {
    pub nivel: NivelDaLuz,
    caras: u32,
    folgadas: u32,
}

impl Governador {
    #[must_use]
    pub fn novo(nivel: NivelDaLuz) -> Self {
        Self {
            nivel,
            caras: 0,
            folgadas: 0,
        }
    }

    /// Uma leitura do custo da luz (`ms` de placa, o solver inteiro) com o teto `teto`. Devolve o
    /// nível a usar.
    pub fn ler(&mut self, ms: f64, teto: NivelDaLuz) -> NivelDaLuz {
        if self.nivel > teto {
            *self = Self::novo(teto);
            return self.nivel;
        }
        let acima = vizinho(self.nivel, 1).filter(|&a| a <= teto);
        let previsto = acima.map(|a| {
            let r = f64::from(self.nivel.celula_px() / a.celula_px());
            ms * r * r
        });
        if ms > ORCAMENTO_MS {
            self.folgadas = 0;
            self.caras += 1;
            if self.caras >= LEITURAS_PARA_DESCER
                && let Some(b) = vizinho(self.nivel, -1)
            {
                *self = Self::novo(b);
            }
        } else if let (Some(a), Some(p)) = (acima, previsto)
            && p < ORCAMENTO_MS * FOLGA_PARA_SUBIR
        {
            self.caras = 0;
            self.folgadas += 1;
            if self.folgadas >= LEITURAS_PARA_SUBIR {
                *self = Self::novo(a);
            }
        } else {
            self.caras = 0;
            self.folgadas = 0;
        }
        self.nivel
    }
}
