//! **A MÃO do lápis** — o estabilizador, entre o ponteiro e o lápis.
//!
//! O gesto de mão livre grava o que a mão fez, e a mão TREME. O decimador (RDP) não resolve isto e
//! nunca vai: ele preserva **extremos locais** de propósito (é o que faz uma quina desenhada
//! sobreviver a qualquer tolerância), e um tremor é exactamente um extremo local. São duas
//! perguntas — *que detalhe eu guardo?* (Fidelity, na SAÍDA) e *que mão eu escuto?* (Stabilizer, na
//! ENTRADA) — e ter dois controles é o que impede um deles de mentir.
//!
//! # A porta é a do Painter, não uma segunda
//!
//! O filtro é **`ph2d_painter_brush::lazy_mouse_step`**, a MESMA função que o pincel do Painter e o
//! Free Hand dele usam. Uma segunda suavização divergiria do que o artista já aprendeu com o
//! raster, e o app passaria a ter duas respostas para *"o que o slider de estabilização faz?"*.
//!
//! ⚠️ **A aresta de dependência custa ZERO, e isto foi MEDIDO** (`cargo tree -i`): a shell já
//! recebe a `ph2d-painter-brush` transitivamente pela `ph2d-tool-painter`, então a crate já é
//! compilada em todo build da shell e a aresta direta só acrescenta uma linha ao `Cargo.toml`. A
//! objeção de 23.250 LOC que a W1a mediu era contra pôr a aresta na **`ph2d-vec-edit`** — uma crate
//! leaf com UMA dependência, onde ela seria uma sub-árvore NOVA. É por isso que o filtro é aplicado
//! aqui (a shell, que possui a entrada) e não dentro do lápis.
//!
//! # Em que espaço se filtra, e por quê
//!
//! Em **pixels de TELA**, antes de converter para mundo. O tremor é um fato da mão sobre a mesa, e
//! é em px que ele tem tamanho; o `lazy_mouse_step` é uma mistura proporcional (não um limiar de
//! distância), então filtrar em mundo daria a mesma forma — mas o número que o artista arrasta
//! passaria a significar coisas diferentes em zooms diferentes se algum dia o filtro ganhasse um
//! termo absoluto. Filtrar onde o tremor vive mantém o slider honesto.
//!
//! # O default, MEDIDO
//!
//! **MEDIDO** (`measure_pencil_stabilizer`, o S trémulo de 300 px com ±1,5 px de tremor sintético,
//! Fidelity 2 px) — o tremor que sobra, o atraso no fim do gesto, e os nós que o traço ganha:
//!
//! | stabilizer | tremor residual | atraso final | nós |
//! |---|---|---|---|
//! | 0,00 | 2,06 px | 0,00 px | **97** |
//! | 0,25 | 1,41 px | 1,02 px | 25 |
//! | **0,50** | **0,89 px** | **2,63 px** | **14** |
//! | 0,75 | 0,58 px | 6,33 px | 14 |
//! | 0,90 | 1,00 px | 13,08 px | 15 |
//! | 0,97 | 2,39 px | 21,72 px | 13 |
//! | 1,00 | 4,23 px | 29,05 px | 14 |
//!
//! ⚠️ **O tremor residual NÃO é monotónico, e é isso que escolhe o teto útil:** ele tem MÍNIMO em
//! 0,75 e depois PIORA — a 1,00 o traço está 2× mais longe da curva pretendida que a mão crua. Um
//! lazy mouse é um passa-baixa, e passado o joelho ele deixa de remover o RUÍDO e começa a comer o
//! SINAL: o ponto filtrado atrasa tanto que corta as curvas que o artista quis desenhar. Isto não é
//! defeito a corrigir — é o que "lazy" custa —, mas mata a leitura ingénua de *"mais é mais liso"*.
//!
//! ⚠️ **E o número que decide o default é a CONTAGEM DE NÓS, não o tremor:** num lápis VETORIAL o
//! que o artista herda é a curva que vai editar, e com a mão crua o RDP guarda **97 nós** para um S
//! — porque o tremor É um extremo local e a Fidelity preserva extremos por desenho. A 0,50 são
//! **14**, com 2,63 px de atraso (um quarto da largura de traço default, imperceptível). O default é
//! **0,50** por esse par; 0,75 compra 0,3 px de tremor por 2,4× o atraso, e não paga.
//!
//! ⚠️ **Isto CORRIGE uma nota minha da W1a**, que anunciava *"nove nós descrevem um S inteiro"*: era
//! medido sobre uma fixture de ±0,4 px de tremor — uma mão mais firme do que a real. Com ±1,5 px a
//! Fidelity sozinha dá 97, e é o estabilizador que torna a contagem editável.
//!
//! ⚠️ **O VALOR vive no tool** (`ph2d_tool_vector::params::PENCIL_STABILIZER_DEFAULT`), que é quem
//! o autora e semeia o slider; aqui fica a MEDIÇÃO, ao lado do filtro que ela mede. Uma segunda
//! const com o mesmo número seria a segunda porta que esta wave passou a tarde a evitar.
//!
//! ⚠️ **Preço, e é intrínseco a qualquer lazy mouse:** com estabilização alta o ponto filtrado
//! ATRASA o cursor, então o traço termina um pouco antes de onde a mão levantou (medido na tabela
//! acima). O Painter e o *Stabilize Stroke* do Blender têm exactamente a mesma
//! propriedade — é o que "lazy" quer dizer.

/// A mão filtrada do lápis: a posição corrida do estabilizador.
///
/// Vive na shell porque é a shell que possui a ENTRADA (o `input_dispatch` é quem vê o ponteiro), e
/// porque o filtro mora numa crate que o lápis não pode ver. O lápis continua a receber pontos e a
/// não saber de onde vêm — é o que o mantém puro e testável sem janela.
#[derive(Default)]
pub(crate) struct PencilHand {
    /// A posição FILTRADA corrente, em px de tela.
    stab: [f32; 2],
}

impl PencilHand {
    /// **O press semeia a mão.** Sem isto o 1º move mistura a partir da posição em que o gesto
    /// ANTERIOR acabou, e o traço nasce com um salto vindo do outro lado da tela.
    ///
    /// Semear (em vez de guardar `Option` e limpar no release) é deliberado: o release e o cancel
    /// teriam de lembrar-se de limpar, e um 3º caminho de morte nasceria sem a limpeza. Um valor
    /// obsoleto aqui nunca é lido, porque o filtro só corre com um gesto vivo e todo gesto começa
    /// por um press.
    pub(crate) fn begin(&mut self, px: (f32, f32)) {
        self.stab = [px.0, px.1];
    }

    /// **A amostra que o lápis de facto vê.** `strength` 0 devolve o ponteiro cru (a igualdade é
    /// exata — o `lazy_mouse_step` faz early-return), então o slider no mínimo é o produto de
    /// antes desta wave, ao bit.
    pub(crate) fn filter(&mut self, px: (f32, f32), strength: f32) -> (f32, f32) {
        self.stab = ph2d_painter_brush::lazy_mouse_step(self.stab, [px.0, px.1], strength);
        (self.stab[0], self.stab[1])
    }
}

impl crate::App {
    /// **A DINÂMICA que o ponteiro carrega agora** — a porta única do W1d.
    ///
    /// A `pencil_width` deriva a largura de duas grandezas: a **pressão** do dispositivo e o
    /// **relógio de parede** (de onde sai a velocidade). Esta função é o único lugar da shell que
    /// as responde para o lápis.
    ///
    /// ⚠️ **A pressão é `1.0`, e é um fato MEDIDO da shell, não um placeholder solto.** Os dois
    /// únicos sítios que constroem um `PointerEvent` (`input_dispatch.rs`) cravam `pressure: 1.0`
    /// com `source: PointerSource::Mouse`, e o laço de eventos do winit **não casa
    /// `WindowEvent::Touch`** — o único evento que carrega `force`. O `CursorMoved`, que é o que
    /// a shell escuta, não tem pressão no protocolo. Logo, hoje, nenhum dispositivo entrega
    /// pressão a este app.
    ///
    /// Ela mora aqui numa função só **exatamente por isso**: quando o caminho do tablet existir,
    /// é ESTA linha que muda, e a fonte `Pressure` do lápis passa a funcionar sem que nada mais
    /// se mexa. Repetir o literal no press e no move seria a terceira cópia de um número que já
    /// mente em duas.
    pub(crate) fn pointer_dynamics(&self) -> ph2d_vec_edit::pencil_width::PenDynamics {
        ph2d_vec_edit::pencil_width::PenDynamics {
            pressure: 1.0,
            t_ns: Self::timestamp_ns(),
        }
    }
}

#[cfg(test)]
#[path = "vec_pencil_input_tests.rs"]
mod tests;
