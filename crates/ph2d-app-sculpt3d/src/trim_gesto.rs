//! **O GESTO do corte** — o que a mão desenha, antes de virar volume.
//!
//! Filho (`#[path]`) de [`super`]. A lei que transforma o anel num prisma vive
//! na [`ph2d_trim`]; aqui fica o que é do GESTO: o que o pen-down fotografa, que
//! pontos o arrasto guarda, e o que o pen-up entrega.
//!
//! ⚠️ **Puro de propósito — sem cena, sem `wgpu::Device`, sem câmara.** É isso
//! que o torna gateável, e é a lição que o censo dos knobs desta linha pagou:
//! uma lei que precisa de um dispositivo para ser medida nasce `#[ignore]`, e o
//! CI deixa de a correr.

/// A forma que a mão desenha (espec §3).
///
/// ⚠️ **A polilinha é tratada como um LAÇO** no alvo — a diferença está só em
/// como o utilizador a desenha, não na máquina. ⏳ A **linha** (2 pontos, §4.2)
/// é a quarta variante e ainda não está aqui: ela é a mesma máquina com um
/// quadrilátero fabricado, e o modo dela é **forçado** a subtrair.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Forma {
    /// Dois cantos ⇒ o rectângulo de ecrã.
    Caixa,
    /// O caminho desenhado, ponto a ponto.
    Laco,
}

/// A orientação do varrimento que o artista pediu (espec §5).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) enum Orientacao {
    /// O eixo é a direcção da vista, invertida.
    ///
    /// ⚠️ **É o valor de fábrica**, e não por gosto: sem superfície sob o cursor
    /// é o único que a espec §3 deixa de pé.
    #[default]
    Vista,
    /// O eixo é a normal da superfície onde o gesto começou.
    Superficie,
}

/// ⭐ **A DISTÂNCIA MÍNIMA entre dois pontos guardados de um laço, em pixels.**
///
/// ⚠️ **Ela nomeia o recurso, que é o CUSTO DA TAMPA:** o prisma tem `2n`
/// vértices e a triangulação da tampa é quadrática em `n`, logo um laço cru de
/// um arrasto lento (centenas de eventos) pagaria o quadrado disso por nada —
/// dois pontos a meio pixel um do outro não descrevem forma nenhuma que o
/// artista consiga ver.
const PASSO_MINIMO_PX: f32 = 4.0;

/// **O ESTADO DO CORTE na cena** — armado, orientação, e o gesto em curso.
///
/// ⚠️ **Um tipo e não três campos soltos na cena**, e a razão está escrita ao
/// lado do campo `viewports` dela: as três coisas são UMA (*como o corte está
/// armado*), nascem juntas, morrem juntas e são lidas pelos mesmos módulos.
/// Espalhadas na struct de ~50 campos seriam indistinguíveis das que descrevem
/// *o que a peça É*.
#[derive(Default)]
pub(crate) struct Trim {
    /// A forma armada — `None` quando o corte não está na mão.
    pub(crate) armado: Option<Forma>,
    /// A orientação que o artista pediu.
    pub(crate) orientacao: Orientacao,
    /// O gesto a decorrer, entre o pen-down e o pen-up.
    pub(crate) gesto: Option<Gesto>,
}

/// O gesto em curso.
pub(crate) struct Gesto {
    forma: Forma,
    pontos: Vec<[f32; 2]>,
    /// O que havia sob o cursor no pen-down: `(posição, normal)` em mundo.
    ///
    /// ⚠️ **Fotografado no pen-down e nunca relido** — é a mesma lei que o
    /// polegar e o projectar desta linha já pagaram: uma grandeza relida do vivo
    /// deriva enquanto o gesto acontece.
    acerto: Option<([f32; 3], [f32; 3])>,
}

impl Gesto {
    /// O pen-down: a forma, onde começou, e o que havia por baixo.
    pub(crate) fn comeca(forma: Forma, em: [f32; 2], acerto: Option<([f32; 3], [f32; 3])>) -> Self {
        Self {
            forma,
            pontos: vec![em],
            acerto,
        }
    }

    /// Um evento de movimento.
    pub(crate) fn move_para(&mut self, p: [f32; 2]) {
        match self.forma {
            // ⚠️ A caixa guarda só DOIS pontos: os cantos. Guardar o caminho
            // seria descrever um rectângulo por uma linha que serpenteia.
            Forma::Caixa => {
                self.pontos.truncate(1);
                self.pontos.push(p);
            }
            Forma::Laco => {
                let ultimo = *self.pontos.last().expect("o pen-down pôs um");
                let d = (p[0] - ultimo[0]).hypot(p[1] - ultimo[1]);
                if d >= PASSO_MINIMO_PX {
                    self.pontos.push(p);
                }
            }
        }
    }

    /// **O anel de ecrã** que a lei recebe (espec §4).
    ///
    /// ⚠️ Devolve vazio quando o gesto não delimita área — quem chama recusa em
    /// voz alta, e a recusa é do gesto, não do motor.
    pub(crate) fn anel(&self) -> Vec<[f32; 2]> {
        match self.forma {
            Forma::Caixa => {
                let [a, b] = [self.pontos[0], *self.pontos.last().expect("um")];
                if (a[0] - b[0]).abs() < 1.0 || (a[1] - b[1]).abs() < 1.0 {
                    return Vec::new();
                }
                vec![[a[0], a[1]], [b[0], a[1]], [b[0], b[1]], [a[0], b[1]]]
            }
            Forma::Laco => {
                if self.pontos.len() < 3 {
                    return Vec::new();
                }
                self.pontos.clone()
            }
        }
    }

    /// ⭐⭐ **A ORIENTAÇÃO EFECTIVA, e se ela foi COAGIDA** (espec §3).
    ///
    /// Sem superfície sob o cursor no pen-down não há normal de onde tirar um
    /// eixo ⇒ a orientação é forçada a [`Orientacao::Vista`].
    ///
    /// ⚠️⚠️ **O alvo faz isto em SILÊNCIO — é a única correcção calada de um
    /// parâmetro autorado em toda a família dele.** Nós devolvemos o facto para
    /// quem chama poder **dizê-lo**, que é a divergência que a espec nomeia
    /// (**N**): *o resto dos vereditos desta casa é recusa em voz alta, e um
    /// knob que muda de valor sem avisar é a espécie de controlo que mente.*
    pub(crate) fn orientacao(&self, pedida: Orientacao) -> (Orientacao, bool) {
        match (pedida, self.acerto) {
            (Orientacao::Superficie, None) => (Orientacao::Vista, true),
            _ => (pedida, false),
        }
    }

    /// O plano da forma (espec §5): origem e eixo do varrimento.
    ///
    /// `direccao_da_vista` é para onde a câmara olha; o eixo de
    /// [`Orientacao::Vista`] é ela **invertida**.
    pub(crate) fn plano(
        &self,
        pedida: Orientacao,
        direccao_da_vista: [f32; 3],
        origem_sem_acerto: [f32; 3],
    ) -> (ph2d_trim::Plano, bool) {
        let (efectiva, coagida) = self.orientacao(pedida);
        let origem = self.acerto.map_or(origem_sem_acerto, |(p, _)| p);
        let normal = match efectiva {
            Orientacao::Superficie => self.acerto.map_or(
                [
                    -direccao_da_vista[0],
                    -direccao_da_vista[1],
                    -direccao_da_vista[2],
                ],
                |(_, n)| n,
            ),
            Orientacao::Vista => [
                -direccao_da_vista[0],
                -direccao_da_vista[1],
                -direccao_da_vista[2],
            ],
        };
        (ph2d_trim::Plano { origem, normal }, coagida)
    }
}

#[cfg(test)]
#[path = "trim_gesto_tests.rs"]
mod tests;
