//! ⭐⭐⭐ **A APRESENTAÇÃO DA CENA** — o olhar, o estilo e a escala da peça, num tipo só.
//!
//! # ⚠️ Porque é um ficheiro irmão do [`super::shade_render`]
//!
//! ⛔ **Corte por RESPONSABILIDADE, forçado pelo tecto de `700` linhas** e melhor por isso
//! (`CLAUDE.md` §5.0: *corte por responsabilidade, nunca uma entrada no `FILE_OVERAGE_OK`*). O irmão
//! responde *«que luz é que este pixel devolve ao olho»*; isto responde *«sob que apresentação»* — e
//! são duas populações que crescem por razões diferentes: aquela cresce quando a física muda, esta
//! cresce **sempre que a direcção de arte ganha uma pergunta**.
//!
//! ⭐ E a auditoria de 2026-09-19 (`docs/Render3d/11` §10) foi exactamente isso: ela acrescentou aqui
//! a [`Presentation::curvature_eps`] — a distância a que a curvatura do estilo é medida —, que é uma
//! pergunta de APRESENTAÇÃO e não de sombreamento.

use ph2d_view_transform::Look;

/// ⭐⭐⭐ **A APRESENTAÇÃO DA CENA** — o OLHAR e o ESTILO, juntos porque viajam juntos
/// (`docs/Render3d/03`, a `W8`).
///
/// ```text
/// material + luz  →  [ESTILO]  →  olhar (exposição + vista)  →  sRGB
///  (a física)        (a mentira)   (a ph2d-view-transform)
/// ```
///
/// # ⛔⛔ Porque ela entra na ASSINATURA em vez de haver um `shade_render_com_estilo`
///
/// Uma função-irmã seria a segunda porta pela qual o defeito volta — e este módulo acabou de pagar
/// exactamente isso (`docs/Render3d/10` §24: *uma cura gateada ao bit, num ramo que o produto não
/// corre*). Com ela na assinatura, **esquecer o estilo é erro de compilação** em todo chamador, que
/// é a mesma lei que o ledger do `ProjectState::capture` já aplica um nível acima.
///
/// ⚠️ E a [`Presentation::of`] existe para quem não tem estilo nenhum a dizer — ela é a **identidade
/// exacta** (ver o [`ph2d_style`]), e é isso que deixa os gates de material e de luz medirem a
/// física sem uma mentira por cima.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Presentation {
    /// A exposição e a vista — ver [`ph2d_view_transform::Look`].
    pub look: Look,
    /// Os botões da direcção de arte — ver [`ph2d_style::Style`].
    ///
    /// ⚠️ **Ele chega SANEADO** ([`ph2d_style::Style::sanitized`]), e quem o saneia é quem o monta:
    /// a cerca é de QUADRO e correria por pixel se vivesse aqui.
    pub style: ph2d_style::Style,
    /// ⭐⭐ **O raio da bola que envolve a PEÇA**, em unidades de mundo — o que torna a curvatura
    /// adimensional antes de ela chegar ao estilo.
    ///
    /// # ⚠️ Porque ele é presentação e não geometria
    ///
    /// O G-buffer guarda `H` em `1/comprimento` (é o que a subsuperfície pede). Um botão de estilo
    /// calibrado nisso mudaria de sentido ao **escalar a peça** — a mesma quina daria outra tinta
    /// numa peça de `0,3` e numa de `3`. Multiplicado por este raio, o que o estilo lê é *«quantas
    /// vezes esta zona é mais curva do que a peça inteira»*, que é o que um artista quer dizer.
    ///
    /// ⚠️ **`1,0` é o valor de quem não tem peça** (uma fixtura, um gate de material) e é inofensivo:
    /// com as tintas de fábrica ninguém lê a curvatura.
    pub piece_radius: f32,
    /// ⭐⭐⭐ **O BRILHO** (`docs/Render3d/12`, a `W7`) — o passe que lê o quadro em HDR.
    ///
    /// ⚠️ **Ele entra na APRESENTAÇÃO e não num argumento à parte**, pela mesma razão que o estilo:
    /// uma função-irmã «com brilho» seria a segunda porta pela qual o defeito do `docs/Render3d/10`
    /// §24 volta. Com [`ph2d_bloom::Bloom::contributes`] falso o quadro é o de sempre **ao bit**.
    pub bloom: ph2d_bloom::Bloom,
}

impl Presentation {
    /// **Só o olhar** — estilo de fábrica, que é a identidade exacta.
    #[must_use]
    pub fn of(look: Look) -> Self {
        Self {
            look,
            style: ph2d_style::Style::default(),
            piece_radius: 1.0,
            bloom: ph2d_bloom::Bloom::default(),
        }
    }

    /// A curvatura deste pixel **em unidades da peça**, que é o que o estilo lê.
    ///
    /// ⚠️ **O sinal ATRAVESSA** — é ele que separa uma aresta de uma cova, e é a razão de a
    /// [`crate::curvatura`] ter deixado de o deitar fora.
    ///
    /// ⚠️ **`pub(crate)` e não `pub`**: ela é a conversão que o sombreador faz por pixel, e expô-la
    /// à workspace convidaria um segundo chamador a normalizar a curvatura por conta própria —
    /// *duas respostas à mesma pergunta, e a que o artista vê é a que envelhece.*
    pub(crate) fn styled_curvature(&self, k: f32) -> f32 {
        k * self.piece_radius
    }

    /// ⭐⭐⭐ **A DISTÂNCIA a que a curvatura do ESTILO é medida** — a porta da queixa do dono.
    ///
    /// # ⛔⛔ Ela NÃO é a [`crate::curvatura::eps_para`], e a diferença é a wave inteira
    ///
    /// Aquela devolve o **óptimo de PRECISÃO** (o vale do erro da segunda diferença, medido no mesmo
    /// sítio em três raios). Esta devolve uma **ESCOLHA ARTÍSTICA**: a que distância a peça é
    /// palpada para decidir o que é aresta e o que é cova.
    ///
    /// ⚠️ **Elas têm de ser dois números porque servem duas perguntas** — e enquanto foram um só, a
    /// resposta à pergunta artística era o óptimo numérico, que é o degrau de `169` bytes que o dono
    /// fotografou em 2026-09-19 (`docs/Render3d/11` §10).
    ///
    /// ⚠️ **A fracção é do RAIO DA PEÇA**, e é isso que a torna invariante à escala: a mesma peça
    /// duas vezes maior recebe a mesma imagem. As duas cercas vivem na lei
    /// ([`ph2d_style::Curvature::MIN_SOFTNESS`] / [`ph2d_style::Curvature::MAX_SOFTNESS`]) e chegam
    /// aqui já apertadas pelo [`ph2d_style::Style::sanitized`].
    ///
    /// ⛔ **Nunca derivada da vista.** A curvatura é uma propriedade da peça; um passo que seguisse
    /// o zoom daria duas tintas para o mesmo ponto — o defeito de ecrã que a [`crate::curvatura`]
    /// existe para não ter.
    #[must_use]
    pub fn curvature_eps(&self) -> f32 {
        (self.piece_radius.abs() * self.style.sanitized().curvature.softness)
            .max(crate::PRECISION_FLOOR)
    }

    /// **O estilo desta apresentação lê a curvatura?** — a porta do censo, para o chamador não
    /// pagar a assadura que ninguém consome.
    /// **Este quadro paga o passe do brilho?** — a porta que decide se o HDR é sequer construído.
    ///
    /// ⚠️ É ela que faz o caminho de omissão custar **zero**: sem consumidor, o buffer de cena não
    /// nasce (a mesma lei da [`crate::curvatura::assar_canais`], um módulo ao lado).
    #[must_use]
    pub fn blooms(&self) -> bool {
        self.bloom.contributes()
    }

    #[must_use]
    pub fn reads_curvature(&self) -> bool {
        self.style.reads_curvature()
    }
}

impl From<Look> for Presentation {
    fn from(look: Look) -> Self {
        Self::of(look)
    }
}
