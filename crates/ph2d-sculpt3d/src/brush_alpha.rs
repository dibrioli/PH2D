//! **O QUE O PINCEL DIZ SOBRE O PADRÃO** — as portas do alpha.
//!
//! Módulo irmão (`#[path]`) do [`super`], e o corte é por RESPONSABILIDADE: lá
//! mora *que ferramenta está na mão* (o verbo, o falloff, os knobs); aqui, *que
//! número o padrão devolve neste ponto* — a régua, o frame e a pergunta que
//! decide se este pincel carrega um CARIMBO preso à tela ou um campo colado ao
//! barro.
//!
//! ⚠️ **Ele nasceu de um teto de LOC, e o corte foi escolhido em vez de
//! aceito:** as quatro funções abaixo respondem à MESMA pergunta em três
//! camadas (o peso · a escala resolvida · o frame · e a condição que as une), e
//! separá-las umas das outras é que teria sido um corte por tamanho.
//!
//! # ⚠️ TRÊS decisões dos campos do padrão, que moram aqui
//!
//! *(Elas desceram do doc dos campos do [`super::Brush`] quando o tecto de LOC
//! dele ficou vermelho por ACUMULAÇÃO. ⛔ A cura de um tecto é o CORTE, e um
//! campo de dados não tem para onde ir — o que se move é o mecanismo, para o
//! módulo que o implementa.)*
//!
//! ⚠️ **A elevação do eixo não tem PISO, e a lâmpada tem.** Lá o `MIN_ELEV_DEG`
//! existe porque uma luz rasante degenera a resposta plana; um EIXO não degenera
//! em lugar nenhum — o frame é ortonormal por identidade em qualquer elevação —,
//! e um piso aqui seria um limite copiado de um vizinho em vez de medido.
//!
//! ⚠️ **O `alpha_offset` é a TERCEIRA metade de colocar um carimbo** — tamanho
//! (`alpha_scale`), giro (`alpha_az_deg`, que no zénite ROLA o padrão no plano) e
//! posição. As duas primeiras já existiam; sem esta o artista podia dizer *quão
//! grande* e *para que lado*, nunca *onde*. E ele só alcança o motor com uma
//! IMAGEM armada, que é onde o [`super::Brush::alpha_frame`] garante a
//! neutralidade **por construção** em vez de por convenção.
//!
//! ⚠️⚠️ **O `alpha_stencil_scale` é um campo PRÓPRIO e não uma reinterpretação do
//! `alpha_scale`.** Os dois respondem a perguntas diferentes — *que tamanho tem
//! esta feição no MODELO* contra *que tamanho tem este carimbo na TELA* — e um
//! número só com duas unidades trocaria de significado **em silêncio** no
//! instante em que o artista trocasse de padrão, que é a doença que este módulo
//! varre a cada wave. *Duas perguntas, dois números, duas fileiras: cada uma
//! aparece no modo em que está viva.*

use crate::{Alpha, AlphaFrame, AlphaStencil, Brush};

impl Brush {
    /// **O peso do alpha no ponto `p`** (posição em espaço de OBJETO), ou `1.0`
    /// se não há alpha armado.
    ///
    /// **Porta única** — o motor multiplica o falloff por isto, e é a mesma
    /// função que o gate e a sonda perguntam. Um segundo sítio que resolvesse
    /// `Option` + escala por conta própria divergiria no dia em que a escala
    /// ganhasse um clamp diferente.
    ///
    /// ⚠️ **`None` devolve `1.0` EXATO, e é isso que dá a byte-identidade** —
    /// `x * 1.0` é `x` ao bit no IEEE-754, então o pincel liso continua a
    /// produzir a aritmética que ele produzia antes desta wave, sem um `if` no
    /// laço quente para provar.
    #[must_use]
    pub fn alpha_weight(&self, p: [f32; 3], frame: &AlphaFrame) -> f32 {
        match &self.alpha {
            Some(a) => a.weight_at(p, self.alpha_scale_resolved(), frame),
            None => 1.0,
        }
    }

    /// **O tamanho de uma feature na régua que o motor usa** — unidades de OBJETO
    /// para os nove padrões, fração da ALTURA DA TELA para um carimbo.
    ///
    /// ⚠️ **Porta ÚNICA, e ela existe porque a UNIDADE depende do modo.** O dab,
    /// o preview no barro e o retrato do painel perguntam à mesma função; um
    /// deles que resolvesse por conta própria seria a conversão escrita uma vez
    /// por consumidor, com o próximo nascendo sem ela — exatamente como o
    /// `arc_len` do Painter chegou a 2 de 7 rotas em silêncio.
    ///
    /// ⚠️ **E para um carimbo ela não CONVERTE nada, o que é a wave inteira numa
    /// linha:** a fração vai crua ao kernel porque a
    /// [`crate::AlphaFrame::project`] já devolve fração de tela. Converter aqui
    /// exigiria uma profundidade — *qual?* —, e foi essa pergunta sem resposta
    /// única que fez o preview desenhar um carimbo 24,8% maior que o depositado.
    ///
    /// ⚠️ **Sem estêncil devolve o campo CRU** — `alpha_scale` sem tocar num
    /// bit —, e é isso que mantém todo padrão procedural byte-idêntico.
    #[must_use]
    pub fn alpha_scale_resolved(&self) -> f32 {
        match self.stencil() {
            Some(_) => self.alpha_stencil_scale,
            None => self.alpha_scale,
        }
    }

    /// **A vista, SE ela governa este pincel** — a pergunta *"isto é um
    /// estêncil?"*, feita uma vez.
    ///
    /// ⚠️ Ela junta as DUAS condições (há vista **e** o padrão é uma imagem)
    /// porque separá-las é como um consumidor passa a honrar metade: o frame
    /// olharia a tela e a escala olharia o modelo, e o carimbo sairia do
    /// tamanho errado sem nada apontar para o motivo.
    fn stencil(&self) -> Option<&AlphaStencil> {
        self.alpha
            .as_ref()
            .is_some_and(Alpha::is_image)
            .then_some(self.alpha_stencil.as_ref())
            .flatten()
    }

    /// **O FRAME que orienta um padrão direcional** — os dois ângulos autorados,
    /// resolvidos.
    ///
    /// ⚠️ **Chame UMA vez por dab, nunca por vértice.** O rotor deste app é a
    /// rotação de um grau ACUMULADA, ou seja **O(graus)** — até 359 iterações —,
    /// e por vértice ele custaria mais que o padrão inteiro que orienta. É por
    /// isso que [`Self::alpha_weight`] o RECEBE em vez de o derivar: a assinatura
    /// é o que impede o caminho quente de pagar o preço errado.
    ///
    /// ⚠️ **É o ÚNICO construtor de [`AlphaFrame`] que o produto tem** (ele não
    /// implementa `Default`), então o frame que chega ao padrão é sempre o do
    /// pincel que o carrega — o mesmo desenho do `ShapeFrame` do Painter 2D.
    #[must_use]
    pub fn alpha_frame(&self) -> AlphaFrame {
        // ⚠️ **O deslocamento é a colocação de um CARIMBO, e por isso ele só
        // chega ao motor com uma imagem armada.** Os nove procedurais são
        // CAMPOS homogêneos e infinitos: eles não têm posição, só fase, e uma
        // fase é outro controle (uma semente) que este módulo não tem. Zerar
        // aqui — e não esconder a row e torcer — é o que torna impossível um
        // padrão herdar em silêncio um deslocamento que ninguém pode ver nem
        // desfazer, porque a row dele não está na tela.
        let offset = if self.alpha.as_ref().is_some_and(Alpha::is_image) {
            self.alpha_offset
        } else {
            [0.0, 0.0]
        };
        // ⚠️ **Com a vista na mão, uma IMAGEM é lida da TELA** — projeção
        // frontal, que não gira quando o objeto gira e não cresce quando a
        // câmera aproxima. É pedido do Enio (2026-08-09), e é o modelo do
        // estêncil do ZBrush / do *View Plane* do Blender: o carimbo fica preso
        // ao viewport e o barro passa por baixo dele.
        //
        // ⚠️ **E ele SUBSUME a semeadura de eixo que existia para as imagens**
        // (o `elev = 90` que a wave anterior instalava ao armar): com o eixo
        // vindo da vista por construção, não há ângulo a semear — o carimbo
        // encara quem o aplica em qualquer órbita, e não só naquela em que foi
        // armado.
        match self.stencil() {
            Some(s) => AlphaFrame::stencil(s, self.alpha_az_deg, offset),
            None => AlphaFrame::placed(self.alpha_az_deg, self.alpha_elev_deg, offset),
        }
    }
}
