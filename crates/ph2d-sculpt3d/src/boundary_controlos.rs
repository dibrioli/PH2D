//! Os controlos **próprios** do pincel de CONTORNO — ver [`crate::Verb::Boundary`].
//!
//! ⚠️⚠️ **São EXACTAMENTE quatro, e a contagem é da espec (§15):** alvo de
//! deformação, tipo de deformação, queda no contorno e deslocamento da origem.
//! Tudo o resto vem dos ajustes gerais do pincel. *Um painel nosso com um quinto
//! knob próprio está a inventar, e um com três está a esconder.*
//!
//! ⛔ **O alvo de deformação (geometria · simulação de pano) NÃO está aqui**, e
//! a ausência é uma decisão: aquele modo faz a lei escrever **posições-alvo de
//! restrições** em vez de posições, e o solver que as consome é o do pincel de
//! tecido — outra espec. Ele entra quando alguém ligar a costura, e as duas
//! fixturas do corpus que o medem estão classificadas como **fora de espec** na
//! bancada, com o nome.

use crate::Brush;

/// O que o artista autora neste pincel.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct BoundaryControlos {
    pub modo: ph2d_boundary::Modo,
    pub queda_no_contorno: ph2d_boundary::QuedaNoContorno,
    /// ⚠️⚠️ **Ele alonga a PROPAGAÇÃO e NÃO a queda no contorno** — há dois
    /// raios em jogo e confundi-los é o erro caro (espec §8.4). Faixa pública
    /// `0 … 30`; omissão `0`.
    pub deslocamento_da_origem: f32,
}

impl Default for BoundaryControlos {
    fn default() -> Self {
        BoundaryControlos {
            modo: ph2d_boundary::Modo::default(),
            queda_no_contorno: ph2d_boundary::QuedaNoContorno::default(),
            deslocamento_da_origem: 0.0,
        }
    }
}

impl BoundaryControlos {
    /// Junta os próprios aos partilhados e devolve a lei.
    ///
    /// ⚠️ **A força entra LINEARMENTE** (espec §9.2) — ⛔ nunca ao quadrado, e
    /// **sem pressão**: a fixtura de pressão `0,3` do corpus é idêntica à de
    /// `1`. *Este repo já pagou essa confusão uma vez, num corpus inteiro à
    /// força cheia onde `s`, `s²` e `s⁴` coincidem.*
    #[must_use]
    pub fn lei(&self, brush: &Brush) -> ph2d_boundary::Controlos {
        ph2d_boundary::Controlos {
            modo: self.modo,
            queda_no_contorno: self.queda_no_contorno,
            deslocamento_da_origem: self.deslocamento_da_origem,
            // ⚠️ **Os dois raios são o MESMO aqui, e a ausência é declarada:**
            // este desktop não entrega pressão de caneta (há gate a afirmá-lo),
            // logo o raio não é modulado dentro do traço e o inicial e o
            // dinâmico coincidem. *Quem ligar a pressão tem de os separar — o
            // inicial é o do pen-down.*
            raio_inicial: brush.radius,
            raio_dinamico: brush.radius,
            forca: brush.strength,
            esbatimento_da_simetria: 1.0,
            // ⚠️ O modificador de inversão **encaixa o ângulo**, não inverte.
            encaixar_angulo: brush.invert,
            simetria: [false; 3],
        }
    }
}
