//! **O alinhamento do traço** — Inner e Outer, o *Align Stroke* do Illustrator.
//!
//! # Zero geometria nova, e isso é o desenho inteiro
//!
//! Um traço centrado de largura `w` cobre `[−w/2, +w/2]` em volta da borda. Então a faixa de
//! largura `2w` cobre `[−w, +w]` — e a metade de dentro é exactamente `banda ∩ interior`, a de
//! fora exactamente `banda − interior`. As duas peças já existem: [`crate::outline_stroke`]
//! produz a banda (com o tracejado e a quina que o renderer pinta), e [`crate::apply`] corta.
//! Escrever um traçador assimétrico seria uma **segunda resposta** a *"que forma tem este
//! traço?"*, e ela divergiria da primeira no dia em que a quina Miter mudasse.
//!
//! # Por que aqui e não no `stroke_plan`
//!
//! O [`ph2d_vec_scene::stroke_plan`] é a porta única do *"o que este traço desenha?"*, e seria o
//! lugar natural. Ele **não pode**: a `ph2d-vec-boolean` já depende da `ph2d-vec-scene` (o
//! `expand.rs` consome o `stroke_plan`), então chamar a booleana de lá é o ciclo que o cargo
//! recusa por nome. O alinhamento mora um nível acima, e quem o consome são os **dois** leitores
//! do plano — o que PINTA (via `LiveGeometry`) e o que ASSA (o Outline Stroke) —, que é o que
//! impede o desenho e o bake de discordarem.
//!
//! # A lasca NÃO acontece aqui, e a medição corrigiu a afirmação oposta
//!
//! A 1ª versão deste cabeçalho dizia que o recorte reproduz o BUGS #16 (*subtrair uma curva dela
//! mesma depois da volta pelo arranjo deixa resíduo*) — **está errado, e a sonda mediu**: numa
//! forma de borda CURVA, Inner e Outer saem com **uma peça e ZERO lascas** em toda largura testada.
//!
//! O mecanismo é a diferença que eu não tinha visto. No Shape Builder os dois operandos
//! **compartilham a fronteira** (a borda da face é a borda da fonte, recalculada pelo arranjo), e
//! é aí que a tolerância deixa fiapo. Aqui eles **não se tocam**: a linha da forma passa pelo
//! MIOLO da banda — as duas trilhas dela ficam a `w` para cada lado —, então o corte é limpo por
//! construção. O filtro [`crate::expand::drop_slivers`] **fica** (a booleana é a mesma, e uma
//! entrada degenerada não é impossível), mas é **precaução medida, não a espinha**: a mutação que
//! o neutraliza **não sangra**, e isso está gateado e explicado em vez de escondido.

use ph2d_vec_scene::{Paint, StrokeAlign, StrokeSpec, VecPath};

use crate::{BoolOp, apply, expand::drop_slivers, outline_stroke};

/// **A faixa de tinta de um traço ALINHADO**, já em forma preenchida com a cor dele.
///
/// `None` significa *"não há alinhamento a executar — pinte o traço como sempre"*, e cobre quatro
/// casos que se leem igual na tela: sem traço · [`StrokeAlign::Centre`] · largura zero · e a
/// geometria que **não delimita um interior** (ver abaixo). `Some` é a faixa; `Some` vazio seria a
/// aniquilação, e a distinção é a mesma que a [`ph2d_vec_render::LiveGeometry`] faz — colapsar as
/// duas faria a forma reaparecer inteira no instante em que o recorte a mata.
///
/// ⚠️ **Só caminhos inteiramente FECHADOS.** *Dentro* e *fora* são perguntas sobre uma região, e
/// um contorno aberto não tem uma; um caminho MISTO (uma silhueta mais uma linha de construção)
/// também não, porque a linha teria de ser desenhada centrada enquanto a silhueta é recortada — e
/// então uma forma só teria duas leis de traço ao mesmo tempo. A recusa é aqui e o painel a
/// espelha pela mesma [`StrokeAlign::needs_a_region`], para o botão nunca prometer o que a
/// geometria devolve como nada.
///
/// ⚠️ **Falha do motor devolve `None`, não vazio.** Um sweep que desiste tem de deixar a arte na
/// tela do jeito que estava; devolver vazio a apagaria em silêncio.
#[must_use]
pub fn aligned_stroke(path: &VecPath) -> Option<Vec<VecPath>> {
    let s = path.stroke.as_ref()?;
    if !s.is_aligned() || !all_contours_closed(path) {
        return None;
    }
    // A banda DUPLA: o mesmo traço com o dobro da largura, que cobre `[−w, +w]`.
    let band = outline_stroke(&VecPath {
        stroke: Some(doubled(s)),
        ..path.clone()
    });
    if band.is_empty() {
        return None; // o motor não respondeu — a arte fica como estava
    }
    // O operando de recorte: a MESMA forma sem traço, e carregando o estilo de TINTA. O
    // `apply_many` toma o estilo do path do TOPO (o último), então dá-lo aqui é o que faz o
    // recorte sair já com a cor do traço — em vez de a carimbar depois, que seria confiar duas
    // vezes na mesma decisão.
    let region = VecPath {
        fill: Some(Paint::Solid(s.color())),
        stroke: None,
        ..path.clone()
    };
    let op = match s.align {
        StrokeAlign::Inner => BoolOp::Intersect,
        StrokeAlign::Outer => BoolOp::Subtract,
        // `is_aligned()` já respondeu por este caso; o braço existe porque o `match` é a porta.
        StrokeAlign::Centre => return None,
    };
    // ⚠️ `Subtract` preserva o de TRÁS: a banda entra primeiro e a região por cima.
    Some(drop_slivers(
        band.iter().flat_map(|b| apply(b, &region, op)).collect(),
    ))
}

/// O mesmo traço com o **dobro da largura** — o insumo dos dois alinhamentos.
///
/// ⚠️ **O tracejado é compensado, e sem isso o Inner sai com a cadência errada.** O campo `dash`
/// guarda MÚLTIPLOS da largura (é o que faz engrossar a linha alongar traço e vão na proporção),
/// então dobrar a largura dobraria o dash em COMPRIMENTO — e o recorte devolveria metade da
/// espessura com o dobro do passo. Halvar os multiplicadores mantém o `dash_lengths()` idêntico,
/// que é a grandeza que o olho vê.
fn doubled(s: &StrokeSpec) -> StrokeSpec {
    StrokeSpec {
        width: s.width * 2.0,
        dash: s.dash.map(|(d, g)| (d * 0.5, g * 0.5)),
        ..s.clone()
    }
}

/// **Todo contorno desta forma fecha?** — a pergunta que decide se existe um interior.
///
/// Lê a geometria **COZIDA**: o raio de quina vivo (ADR-0121) e a pilha de efeitos (ADR-0132)
/// correm antes, e é a forma cozida que a banda vai traçar. Perguntar à autorada deixaria um
/// efeito que ABRE um contorno (um Trim) responder que ainda há região.
fn all_contours_closed(path: &VecPath) -> bool {
    let cooked = path.cooked();
    let n = cooked.contour_count();
    n > 0 && (0..n).all(|c| cooked.contour(c).is_some_and(|(_, closed)| closed))
}

#[cfg(test)]
#[path = "align_tests.rs"]
mod tests;
