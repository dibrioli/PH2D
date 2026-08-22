//! **O que um traço DESENHA** — a porta única.
//!
//! Um `StrokeSpec` não é uma caneta só. Com pontas (arrowheads) o traço vira TRÊS coisas: a
//! linha **encurtada** para caber nas pontas, e uma ponta em cada extremo — cheia
//! (preenchida) ou vazada (traçada com a mesma caneta, mas **sempre sólida**: o tracejado é
//! da LINHA, a ponta é um símbolo).
//!
//! Essa receita existia **uma vez, dentro do renderer**. Isso bastava enquanto desenhar era
//! a única coisa que se fazia com um traço. Deixou de bastar quando o **Outline Stroke**
//! (`ph2d_vec_boolean::outline_stroke`) passou a converter um traço em forma: se ele
//! reescrevesse a receita, as duas responderiam *"o que este traço desenha?"* separadamente,
//! e o dia em que uma mudasse — um recuo de ponta, um `marker_scale` — a outra ficaria
//! calada e errada. O artista clicaria em Outline Stroke e receberia uma forma que **não é a
//! que estava na tela**, sem erro nenhum. [[feedback_two_doors_to_the_same_question_diverge]]
//!
//! Então a receita mora aqui e tem dois consumidores: quem **pinta** e quem **assa**.
//!
//! ⚠️ **O caso comum não paga nada.** 99% dos paths não têm ponta, e aí o plano é uma peça
//! só cujo caminho é `Cow::Borrowed` — o próprio path, sem cópia (o mesmo truque do
//! [`crate::VecPath::cooked`]). Só quem tem ponta constrói geometria.

use std::borrow::Cow;

use crate::{StrokeSpec, VecPath, stroke_head};

/// Uma peça do que o traço desenha.
#[derive(Clone, Debug, PartialEq)]
pub enum StrokePiece<'a> {
    /// **A linha**, traçada com a caneta do estilo: largura · cap · join · dash.
    Line { path: Cow<'a, VecPath> },
    /// **Uma ponta vazada**, traçada com uma caneta SIMPLES na largura crua — sem dash
    /// (o tracejado é da linha; a ponta é um símbolo) e sem o cap/join do estilo
    /// (engrossar o risco fecharia um losango vazado).
    Symbol { path: VecPath },
    /// **Uma ponta cheia**, preenchida com a cor do traço (seta, losango, bolinha).
    Fill { path: VecPath },
}

/// O que `path` desenha sob `s`, na ordem em que desenha (a linha primeiro, as pontas por
/// cima). Vazio é possível e correto: uma linha mais curta que os recuos somados de duas
/// pontas gordas não tem linha nenhuma — só as pontas.
/// **O tracejado que este caminho desenha** — comprimentos já reescalados para ENCAIXAR.
///
/// Porta única para os dois consumidores da receita do traço (quem pinta e quem assa). Eles falam
/// com versões diferentes da kurbo e cada um constrói o próprio `Stroke`, mas têm de concordar
/// sobre *quanto mede um traço* — e agora também sobre *quantos cabem*. Uma segunda medição faria
/// o Outline Stroke assar o tracejado noutra cadência que a desenhada, que é o defeito que o doc
/// de [`StrokeSpec::dash_lengths`] já nomeia.
///
/// ⚠️ **Recebe a LINHA que vai ser traçada**, que desde 2026-08-22 é o caminho INTEIRO: a ponta
/// cresce para fora do nó e a linha deixou de ser encurtada, então o padrão encaixa no comprimento
/// todo. (Enquanto a linha recuava, era o comprimento da parte que SOBRAVA que contava — e essa
/// dependência é uma das coisas que a lei nova apaga.)
///
/// ⚠️ **O caso comum não paga NADA, e a ordem das duas perguntas é o que garante isso.** Medir
/// custa um `arclen` por segmento (Gauss-Legendre de 16 nós), e isto corre por caminho e por
/// frame — pagá-lo nos 99% dos traços sólidos seria uma regressão silenciosa em toda cena. O
/// `dash_lengths()` sai primeiro e devolve `None` sem tocar na geometria.
///
/// ⚠️ **Mede o COZIDO, nunca a fonte.** Quem desenha traça a geometria derivada, então é o
/// perímetro dela que o padrão fecha. Num retângulo com raio de quina os cantos encurtam a volta,
/// e ajustar pela fonte angulosa deixa um resíduo de traço na junta — a costura fonte≠cozido do
/// ADR-0121, no nível do tracejado.
///
/// ⚠️ **Mede o contorno PRINCIPAL, não os subpaths.** Um composto (forma com buracos) traça todos
/// os anéis com o MESMO padrão — é um `Stroke` só —, e anéis de perímetros diferentes não têm um
/// fator comum que feche os dois. O anel de fora é o que o olho segue; os buracos ficam como
/// estavam. Fechá-los todos exigiria traçar anel a anel, que é outra estrutura.
///
/// ⚠️ **UMA lei, DUAS portas — e a diferença entre elas é só quem já cozeu.** A conta mora em
/// [`crate::dash_fit`] (`fit` + `longest_contour`), e [`crate::dash_fit::dash_lengths_for`] é o
/// núcleo que mede um caminho **já cozido** — é o que o cache de tesselação do renderer chama,
/// porque ele já pagou o cozimento. Esta porta coze e delega; ela existe para a peça que chega
/// da fonte (a linha encurtada pelos marcadores). Na integração de 2026-08-22 duas linhas
/// tinham escrito a mesma lei duas vezes (`dash_fit` na `line/motion-value`,
/// `dash_lengths_fitted` na `line/Vector`), com a MESMA fórmula; ficou uma, e as duas suítes
/// provam-na.
#[must_use]
pub fn dash_for(path: &VecPath, s: &StrokeSpec) -> Option<[f64; 2]> {
    // ⚠️ A guarda ANTES da medição — ver o ⚠️ do custo acima.
    s.dash_lengths()?;
    crate::dash_fit::dash_lengths_for(&path.cooked(), s)
}

#[must_use]
pub fn stroke_plan<'a>(path: &'a VecPath, s: &StrokeSpec) -> Vec<StrokePiece<'a>> {
    let mut out = Vec::new();
    // ⚠️ **A LINHA NÃO É MAIS ENCURTADA** (2026-08-22). A ponta cresce para FORA do nó
    // (`stroke_head`), então não há espaço a libertar — e o `Cow::Borrowed` que era o prémio dos
    // 99% sem marcador passa a valer para TODOS os traços, marcados ou não.
    //
    // ⚠️ O que se ganha não é a alocação, é a CLASSE DE DEFEITOS que desaparece: encurtar obrigava
    // a converter o corpo RETO do marcador num recuo de ARCO sobre a curva, e essa conversão
    // depende da curvatura. Ela custou três waves — a ponta descolada, a extremidade fora da
    // curva, e o traço a desaparecer em certos ângulos.
    let line = Some(Cow::Borrowed(path));
    if let Some(path) = line {
        out.push(StrokePiece::Line { path });
    }
    for at_start in [true, false] {
        let Some((marker, geo)) = stroke_head(path, s, at_start) else {
            continue;
        };
        out.push(if marker.is_filled() {
            StrokePiece::Fill { path: geo }
        } else {
            StrokePiece::Symbol { path: geo }
        });
    }
    out
}

#[cfg(test)]
#[path = "stroke_plan_tests.rs"]
mod tests;
