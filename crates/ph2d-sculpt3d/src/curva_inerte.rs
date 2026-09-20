//! ⭐⭐⭐ **PORQUE É QUE A CURVA DO PINCEL NÃO CHEGA AO BARRO** — a porta que
//! impede o painel de pintar um selector vivo sobre uma lei que não o lê.
//!
//! # O defeito que ela cura
//!
//! O censo dos knobs mede, para cada `(verbo, knob)`, se o barro sente as duas
//! pontas da faixa. A fileira da **curva** é a única que o painel pinta
//! **sempre**, por cerca de produto medida e gateada
//! (`the_basic_level_never_hides_the_curve_that_shapes_the_dab`): o *Falloff* é
//! painel de primeira classe na referência, e esconder um selector que a nossa
//! malha tem doze vezes seria apagar um controlo que o artista conhece.
//!
//! ⇒ ela **não pode ser escondida**, e três pincéis não a leem. *Um controlo que
//! o artista arrasta e o barro não sente é pior que um ausente* — a saída que
//! não viola a cerca é desenhá-la com a **razão à vista**.
//!
//! # ⚠️ O que é MEDIDO e o que seria palpite
//!
//! A pose foi varrida nas **cinco** deformações × `1/2/4/8` segmentos pela porta
//! do produto (a sonda `diag_a_pose_por_deformacao` do censo):
//!
//! | deformação | seg 1 | seg 2 | seg 4 | seg 8 |
//! |---|---|---|---|---|
//! | Rotate · Scale · Translate · Squash | `0` | `0` | `0` | `0` |
//! | **Twist** | `0` | `2,755e-1` | `3,866e-1` | `3,216e-1` |
//!
//! ⇒ **duas** condições, e nenhuma delas se adivinha da outra: só a torção lê a
//! curva (espec §1.2), e com **um** segmento não há nada para ela repartir.
//! ⚠️ *Escrever só a primeira faria o painel prometer uma curva viva no valor de
//! FÁBRICA do número de segmentos, que é exactamente onde ela não faz nada.*

/// **A RAZÃO** de a curva do dab não chegar — ver o cabeçalho.
///
/// ⚠️ **Um enum e não uma chave de i18n**, e a fronteira é de propósito: esta
/// crate é o MOTOR e não sabe o que o painel mostra. Devolver uma chave daqui
/// poria o vocabulário da interface dentro da lei, e o dia em que o painel
/// renomeasse a chave o motor mentiria em silêncio.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CurvaInerte {
    /// O canal de máscara tem a **segunda curva** da referência
    /// ([`crate::Brush::mask_weight`]), e a do carimbo não o alcança.
    OCanalTemCurvaPropria,
    /// O verbo não tem lei por-vértice ([`crate::Verb::sem_lei_por_vertice`]):
    /// o efeito dele é sobre a TOPOLOGIA.
    SemLeiPorVertice,
    /// A pose só lê a curva na **torção**, e só com mais de um segmento.
    APoseSoNaTorcaoComSegmentos,
    /// O gesto não CARIMBA: ele desenha uma forma de ecrã, e o que muda a peça
    /// é uma booleana ([`crate::Verb::BoxTrim`]).
    ///
    /// ⚠️ **Não é a [`Self::SemLeiPorVertice`] com outro nome**, embora o verbo
    /// responda `true` às duas: a razão que o artista lê tem de descrever a
    /// ferramenta que ele tem na mão. *Dizer-lhe «o efeito é sobre a topologia»
    /// sobre uma ferramenta de CORTE é um rótulo que mente.*
    OGestoNaoCarimba,
}

impl crate::Brush {
    /// **A curva do dab chega ao barro deste pincel?** — `None` quando sim.
    ///
    /// ⚠️ **Ela pergunta ao PINCEL e não ao VERBO**, porque a resposta da pose
    /// depende de dois controlos dela (a deformação e o número de segmentos).
    /// *Uma porta sobre o verbo teria de mentir num dos dois sentidos.*
    #[must_use]
    pub fn curva_inerte(&self) -> Option<CurvaInerte> {
        match self.verb {
            // ⭐ **TODO verbo de CANAL**, e pela mesma razão exacta: cada um
            // tem a curva DELE (`mask_hardness` · `paint_hardness`, a mesma
            // fórmula da referência com durezas diferentes), e a curva que o
            // artista escolhe no pincel governa a GEOMETRIA. Quem o apanhou foi
            // o censo dos knobs mortos, no minuto em que a pintura acordou nele.
            //
            // ⛔⛔ **E ela é DERIVADA, não uma lista — a lista mordeu primeiro:**
            // este braço dizia `Mask | Paint` por serem os dois que existiam, e
            // os dois verbos de cor que leem o anel chegaram **no mesmo dia** a
            // ler a mesma curva de canal. *Uma enumeração de uma família é uma
            // lista que a próxima família nasce sem*, e o censo acusou-os na
            // primeira corrida em que o arnês os acordou.
            v if v.escreve_um_canal() => Some(CurvaInerte::OCanalTemCurvaPropria),
            // ⚠️ **ANTES do braço geral**, senão ele responde `SemLeiPorVertice`
            // e o artista lê uma frase sobre topologia com um corte na mão.
            crate::Verb::BoxTrim => Some(CurvaInerte::OGestoNaoCarimba),
            v if v.sem_lei_por_vertice() => Some(CurvaInerte::SemLeiPorVertice),
            crate::Verb::Pose
                if !(self.pose.deformacao == crate::PoseDeformacao::Torcer
                    && self.pose.segmentos >= 2) =>
            {
                Some(CurvaInerte::APoseSoNaTorcaoComSegmentos)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "curva_inerte_tests.rs"]
mod tests;
