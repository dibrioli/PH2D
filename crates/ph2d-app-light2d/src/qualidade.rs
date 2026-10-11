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

/// ⭐ O governador: lê o custo medido e decide o nível, sem passar do teto. ⭐ W5e: abaixo do Baixo
/// ele ainda tem a FINA ([`Degrau::com_fina_engrossada`]) — a parte do custo que não segue o nível
/// (`Custo::fixo`) cresce com os píxeis da janela, e acima de 1080p o Baixo sozinho pode não caber.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Governador {
    pub nivel: NivelDaLuz,
    /// ⭐ W5e: a fina engrossada `2^fina` (dentro da [`Self::faixa_da_fina`]).
    pub fina: u32,
    /// O mínimo (o que os tetos do dispositivo já impõem) e o máximo da fina.
    faixa: (u32, u32),
    caras: u32,
    folgadas: u32,
}

/// O passo acima: primeiro a fina volta (a sombra nítida), depois o nível.
enum Acima {
    Fina(u32),
    Nivel(NivelDaLuz),
}

impl Governador {
    #[must_use]
    pub fn novo(nivel: NivelDaLuz) -> Self {
        Self {
            nivel,
            fina: 0,
            faixa: (0, 0),
            caras: 0,
            folgadas: 0,
        }
    }

    /// ⭐ W5e — a faixa da fina para a janela de agora: o `min` que os tetos do dispositivo impõem
    /// ([`crate::teto::fina_que_cabe`]) e o `max` do degrau ([`Degrau::fina_max`]).
    pub fn faixa_da_fina(&mut self, min: u32, max: u32) {
        self.faixa = (min, max.max(min));
        self.fina = self.fina.clamp(self.faixa.0, self.faixa.1);
    }

    /// ⭐ W5e — desce um nível porque o dispositivo não comporta este nem com a fina mais grossa (o
    /// lado de textura ou a memória — não o tempo). `false` = já no Baixo.
    pub fn descer(&mut self) -> bool {
        let Some(b) = vizinho(self.nivel, -1) else {
            return false;
        };
        self.nivel = b;
        self.caras = 0;
        self.folgadas = 0;
        true
    }

    /// Uma leitura do custo da luz (`ms` de placa, o solver inteiro) com o teto `teto`. Devolve o
    /// nível a usar (a fina fica em [`Self::fina`]).
    pub fn ler(&mut self, custo: Custo, teto: NivelDaLuz) -> NivelDaLuz {
        if self.nivel > teto {
            *self = Self {
                faixa: self.faixa,
                fina: self.faixa.0,
                ..Self::novo(teto)
            };
            return self.nivel;
        }
        let ms = custo.total();
        // A fina `2^k` mais fina custa 4× no fixo (a lei `1/célula²` da grade fina).
        let acima = if self.fina > self.faixa.0 {
            Some((
                Acima::Fina(self.fina - 1),
                custo.por_nivel + custo.fixo * 4.0,
            ))
        } else {
            vizinho(self.nivel, 1).filter(|&a| a <= teto).map(|a| {
                let r = f64::from(self.nivel.celula_px() / a.celula_px());
                (Acima::Nivel(a), custo.por_nivel * r * r + custo.fixo)
            })
        };
        if ms > ORCAMENTO_MS {
            self.folgadas = 0;
            self.caras += 1;
            if self.caras >= LEITURAS_PARA_DESCER {
                if let Some(b) = vizinho(self.nivel, -1) {
                    self.nivel = b;
                } else if self.fina < self.faixa.1 {
                    self.fina += 1;
                }
                self.caras = 0;
            }
        } else if let Some((passo, p)) = acima
            && p < ORCAMENTO_MS * FOLGA_PARA_SUBIR
        {
            self.caras = 0;
            self.folgadas += 1;
            if self.folgadas >= LEITURAS_PARA_SUBIR {
                match passo {
                    Acima::Fina(k) => self.fina = k,
                    Acima::Nivel(a) => self.nivel = a,
                }
                self.folgadas = 0;
            }
        } else {
            self.caras = 0;
            self.folgadas = 0;
        }
        self.nivel
    }
}
