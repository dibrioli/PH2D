//! **AS LEIS DE LINHA de um pincel** — as portas que respondem *que lei procedural este pincel
//! arma, e com que números* (plano 38: `Sketchy` · `Wire` · `Ribbon` · `Rough`).
//!
//! Módulo irmão de [`super::spec`], cortado por RESPONSABILIDADE quando o pai cruzou o teto de LOC
//! do workspace: o que sobrou lá é *o que este pincel É* (os campos, o material), e o que veio para
//! cá é *que lei de LINHA ele arma*. As portas continuam em `impl BrushSpec`, então **nenhum
//! chamador muda de caminho**.
//!
//! ⚠️ **Elas moram no `BrushSpec` e não no [`crate::LineKind`] por um motivo MEDIDO**, e o doc do
//! `sews_threads` traz o número: a pergunta *"este pincel arma tal lei?"* é sempre **o tipo E o
//! parâmetro daquele tipo** — um tipo escolhido com o knob no neutro não arma nada. Ter metade da
//! resposta no enum foi o que deixou uma delas apodrecer, com o motor a costurar 343 travessas por
//! traço e o depósito mudo.

use crate::BrushSpec;

impl BrushSpec {
    /// **O Sketchy está armado?** — o tipo escolhido E um alcance que alcança alguma coisa.
    ///
    /// ⚠️ Porta ÚNICA: o motor pergunta para decidir se guarda a memória do traço, e o painel para
    /// decidir se oferece as rows. Duas cópias divergem no dia em que o alcance ganhar um piso.
    #[must_use]
    pub fn sketchy_active(&self) -> bool {
        self.line_kind == crate::line_kind::LineKind::Sketchy && self.sketchy_reach > 0.0
    }

    /// **O Rough está armado?** — o tipo escolhido E alguma amplitude, em QUALQUER das duas oitavas.
    ///
    /// ⚠️ **O `||` é a correção, não conveniência:** um artista que zera o `Roughness` e mantém o
    /// `Bowing` pede a linha que só ARQUEIA, que é um look legítimo do `rough.js` — e um `&&` aqui
    /// desligaria o tipo inteiro sob ele, deixando os dois sliders vivos e o traço reto. Irmã da
    /// [`Self::sketchy_active`] na forma, e diferente dela na aritmética porque este tipo tem duas
    /// amplitudes em vez de um alcance.
    ///
    /// ⚠️ **As `Passes` NÃO entram aqui.** Uma passada é o traço de sempre; a lei não deixa de
    /// existir por o artista pedir uma. Se elas entrassem, `Passes = 1` desligaria o desvio inteiro
    /// e o slider de cima ficaria morto sem dizer porquê.
    #[must_use]
    pub fn rough_active(&self) -> bool {
        self.line_kind == crate::line_kind::LineKind::Rough
            && (self.rough_amount > 0.0 || self.rough_bowing > 0.0)
    }

    /// **A amplitude do desvio CURTO em PIXELS** — a porta única que converte a fração de diâmetro
    /// no número que o campo usa, irmã da [`Self::sketchy_reach_px`].
    #[must_use]
    pub fn rough_amount_px(&self) -> f32 {
        self.rough_amount * 2.0 * self.clamped_radius()
    }

    /// **A amplitude do ARQUEAMENTO em PIXELS** — a irmã da [`Self::rough_amount_px`].
    #[must_use]
    pub fn rough_bowing_px(&self) -> f32 {
        self.rough_bowing * 2.0 * self.clamped_radius()
    }

    /// **Quantas caminhadas este traço deixa** — clampada aqui, na porta única, para o motor e o
    /// painel nunca discordarem sobre o teto.
    #[must_use]
    pub fn rough_pass_count(&self) -> u32 {
        if self.rough_active() {
            self.rough_passes
                .clamp(1, crate::line_kind::ROUGH_PASSES_MAX)
        } else {
            1
        }
    }

    /// **O Wire está armado?** — o tipo escolhido E uma janela que alcança alguma coisa. Irmã exata
    /// da [`Self::sketchy_active`].
    #[must_use]
    pub fn wire_active(&self) -> bool {
        self.line_kind == crate::line_kind::LineKind::Wire && self.wire_history > 0.0
    }

    /// **A FITA está armada?** — o tipo escolhido E um peso que atrasa alguma coisa. Irmã exata da
    /// [`Self::sketchy_active`] e da [`Self::wire_active`], e porta ÚNICA pela mesma razão: o motor
    /// pergunta para decidir se integra a mola, e o painel para decidir se oferece as rows.
    #[must_use]
    pub fn ribbon_active(&self) -> bool {
        self.line_kind == crate::line_kind::LineKind::Ribbon && self.ribbon_weight > 0.0
    }

    /// **O ATRASO da fita em SEGUNDOS** — a constante de tempo `τ` da mola.
    ///
    /// ⚠️ **Com PISO** ([`crate::line_kind::RIBBON_LAG_MIN_S`]), e ele não é higiene: `ω = 1/τ`, então
    /// um peso de `1e-9` pede uma mola infinitamente rígida que NENHUM número de sub-passos integra.
    /// O piso é onde a fita deixa de ser distinguível do traço comum — medido —, e é ele que torna o
    /// custo do integrador um número FECHADO em vez de uma função do que o artista digitar.
    #[must_use]
    pub fn ribbon_lag_s(&self) -> f32 {
        let tau = self.ribbon_weight.clamp(0.0, 1.0) * crate::line_kind::RIBBON_LAG_MAX_S;
        if tau <= 0.0 {
            return 0.0; // o NEUTRO: sem atraso não há fita, e o `ribbon_active` já o diz
        }
        tau.max(crate::line_kind::RIBBON_LAG_MIN_S)
    }

    /// **O AMORTECIMENTO `ζ` da fita** — projeção do `Friction` sobre a faixa medida.
    ///
    /// ⚠️ **O piso não é zero, e o motivo é RECURSO, não gosto:** `ζ = 0` é o oscilador perpétuo, e
    /// um traço que nunca assenta pinta para sempre com a mão parada (ver [`crate::line_kind::RIBBON_DAMPING_MIN`]).
    #[must_use]
    pub fn ribbon_damping(&self) -> f32 {
        use crate::line_kind::{RIBBON_DAMPING_MAX, RIBBON_DAMPING_MIN};
        RIBBON_DAMPING_MIN
            + self.ribbon_friction.clamp(0.0, 1.0) * (RIBBON_DAMPING_MAX - RIBBON_DAMPING_MIN)
    }

    /// **A GRAVIDADE da fita em px/s²** — para BAIXO no canvas (`+y`).
    #[must_use]
    pub fn ribbon_gravity_px_s2(&self) -> f32 {
        self.ribbon_gravity.clamp(0.0, 1.0) * crate::line_kind::RIBBON_GRAVITY_MAX_PX_S2
    }

    /// **A fita desenha a FAIXA?** — a porta única que o motor pergunta antes de costurar uma
    /// travessa e o depósito antes de pintar o feixe.
    ///
    /// ⚠️ Ela exige o ATRASO junto com a densidade, e não é redundância: **a largura da faixa É o
    /// atraso**. Sem atraso os dois trilhos coincidem e cada travessa é um segmento de comprimento
    /// zero — fios gastos a pintar nada.
    #[must_use]
    pub fn ribbon_band_active(&self) -> bool {
        self.ribbon_active() && self.ribbon_rungs > 0.0
    }

    /// **O ESPAÇAMENTO das travessas em PIXELS de arco do trilho da fita.**
    ///
    /// ⚠️ **Porta única, e ela tem DOIS leitores com perguntas diferentes:** o motor a lê para saber
    /// *onde cai a próxima travessa* e a sonda para medir *quantos fios uma faixa custa*. Duas
    /// cópias divergiriam no dia em que o piso deixasse de ser lido, e o sintoma seria uma faixa que
    /// chapa só no pincel pequeno.
    ///
    /// A pista é em DIÂMETROS (livre de escala) com **piso em larguras-de-FIO** — as duas metades
    /// estão nos docs das consts.
    #[must_use]
    pub fn ribbon_rung_px(&self) -> f32 {
        use crate::line_kind::{RIBBON_RUNG_DENSE_D, RIBBON_RUNG_DUTY, RIBBON_RUNG_SPARSE_D};
        let t = self.ribbon_rungs.clamp(0.0, 1.0);
        let d = RIBBON_RUNG_SPARSE_D + t * (RIBBON_RUNG_DENSE_D - RIBBON_RUNG_SPARSE_D);
        (d * 2.0 * self.clamped_radius()).max(RIBBON_RUNG_DUTY * self.thread_width_px.max(0.0))
    }

    /// **Este pincel costura fios?** — a porta ÚNICA que decide se a memória do traço é mantida, se
    /// o motor costura e se o depósito drena o canal.
    ///
    /// ⚠️ **O `match` é EXAUSTIVO de propósito — sem braço `_`.** A família tem três membros
    /// (Sketchy, Wire e as travessas da FITA) e vai ter mais, e a pergunta *"este pincel costura?"*
    /// é sempre **o tipo E o parâmetro daquele tipo**: um tipo que costura com o knob no neutro não
    /// costura. Um braço curinga aqui engoliria o quarto membro **em silêncio** — ele nasceria com
    /// os fios produzidos pelo motor e **nunca pintados**, que é exatamente o defeito que esta
    /// função já teve.
    ///
    /// ⚠️ **E ele já teve:** havia um segundo portão (`LineKind::sews_threads`) que respondia à
    /// metade *do tipo*, com um doc que dizia que ESTA função o consultava — e ela não consultava,
    /// enumerava. Ligar a fita lá deixou o motor a costurar 343 travessas por traço com o depósito
    /// mudo, medido. O portão do enum **não existe mais**: duas respostas para uma pergunta divergem
    /// no dia em que alguém edita só uma.
    #[must_use]
    pub fn sews_threads(&self) -> bool {
        match self.line_kind {
            crate::line_kind::LineKind::Sketchy => self.sketchy_active(),
            crate::line_kind::LineKind::Wire => self.wire_active(),
            crate::line_kind::LineKind::Ribbon => self.ribbon_band_active(),
            // O `Rough` desenha o traço N vezes; ele não costura nada entre pontos.
            crate::line_kind::LineKind::None
            | crate::line_kind::LineKind::Speed
            | crate::line_kind::LineKind::Rough => false,
        }
    }

    /// **A JANELA do Wire em PIXELS de arco** — irmã da [`Self::sketchy_reach_px`], mesma unidade
    /// (diâmetros) e mesma razão para ser porta única: o motor a lê para escolher os pares e a sonda
    /// para medir o orçamento.
    #[must_use]
    pub fn wire_history_px(&self) -> f32 {
        self.wire_history * 2.0 * self.clamped_radius()
    }

    /// **O ALCANCE do Sketchy em PIXELS** — o raio dentro do qual dois pontos do traço se costuram.
    ///
    /// ⚠️ **Porta ÚNICA, e ela existe porque o motor a lê DUAS vezes, para duas perguntas:** *quão
    /// longe no CANVAS um par pode estar* e — sem o `Magnetify` — *quão longe no PERCURSO*. As duas
    /// são a mesma régua de propósito: um alcance escrito duas vezes divergiria no dia em que ele
    /// deixasse de ser medido em diâmetros, e o sintoma seria uma teia que ignora o próprio slider.
    ///
    /// A unidade é o **DIÂMETRO** (`reach = 1` ⇒ um diâmetro), que é o que torna a lei livre de
    /// escala: a W0.3 mediu `fios/dab ≈ 8` em qualquer tamanho de pincel justamente por isto.
    #[must_use]
    pub fn sketchy_reach_px(&self) -> f32 {
        self.sketchy_reach * 2.0 * self.clamped_radius()
    }
}
