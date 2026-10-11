//! ⭐ **A nitidez da luz e o governador** (pedido do dono, 2026-10-09: *«escolha no editor,
//! automática em runtime»*). Baixa · Média · Alta = células de 4 · 2 · 1 px (uma OITAVA entre
//! níveis — ver `NivelDaLuz`); com o «Automático», o nível escolhido é o TETO e o editor desce
//! quando a placa não aguenta.
//!
//! O recurso que limita é o TEMPO DE PLACA por quadro: a luz leva no máximo [`ORCAMENTO_MS`], um
//! terço do quadro a 60 Hz. Medido (sonda `medida_borda_reta`, 1920×1080, vista de fábrica, RTX
//! 5060 Ti, `smoke`): célula na tela 6,8 · 3,4 · 1,7 px = 1,32 · 5,40 · 21,06 ms — ×4 por nível,
//! a lei `1/célula²`. A subida é PREVISTA por ela a partir do custo medido, com folga.

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
        rebotes_por_quadro: if nivel == NivelDaLuz::Baixa {
            DEGRAU_PADRAO.rebotes_por_quadro
        } else {
            1
        },
        ..DEGRAU_PADRAO
    }
}

fn vizinho(n: NivelDaLuz, passo: isize) -> Option<NivelDaLuz> {
    let i = NivelDaLuz::ALL.iter().position(|&x| x == n)?;
    NivelDaLuz::ALL.get(i.checked_add_signed(passo)?).copied()
}

/// ⭐ Uma leitura do custo de placa da luz, partida pelo que segue a célula do nível (`1/célula²`:
/// a captura, o campo, as cascatas e a visibilidade das lâmpadas) e o que não segue (a grade FINA da
/// luz direta, do mesmo tamanho em todo nível).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Custo {
    pub por_nivel: f64,
    pub fixo: f64,
}

impl Custo {
    #[must_use]
    pub fn total(&self) -> f64 {
        self.por_nivel + self.fixo
    }
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
    pub fn ler(&mut self, custo: Custo, teto: NivelDaLuz) -> NivelDaLuz {
        if self.nivel > teto {
            *self = Self::novo(teto);
            return self.nivel;
        }
        let ms = custo.total();
        let acima = vizinho(self.nivel, 1).filter(|&a| a <= teto);
        let previsto = acima.map(|a| {
            let r = f64::from(self.nivel.celula_px() / a.celula_px());
            custo.por_nivel * r * r + custo.fixo
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
