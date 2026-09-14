//! **O NÚMERO DO PINCEL DE CONTORNO** — irmão (`#[path]`) do [`super::rows`],
//! cortado por ASSUNTO, como os do tecido e os da pose.
//!
//! ⚠️ **Um só, e é a espec que o conta:** dos quatro controlos próprios do
//! alvo, dois são selectores (o modo e a queda no contorno, que vivem na
//! pintura), um é um número (este) e o quarto é o alvo de deformação, que
//! depende do solver de pano — outra espec.

use ph2d_sculpt3d::Verb;

use super::{Place, Row};
use crate::state::{Sculpt3dUi, UiLevel};

/// **Esta row é do pincel de CONTORNO?** — a pergunta é ao VERBO, como a dos
/// irmãos. ⛔ Uma lista paralela de nomes seria um knob que aparece noutra
/// ferramenta e não move um vértice.
fn is_boundary(u: &Sculpt3dUi) -> bool {
    u.brush.verb == Verb::Boundary
}

/// ⚠️⚠️ **O DESLOCAMENTO DA ORIGEM alonga a PROPAGAÇÃO e NÃO a queda no
/// contorno** (espec §8.4) — há **dois** raios em jogo e confundi-los é o erro
/// caro deste pincel:
///
/// | onde | valor | efeito |
/// |---|---|---|
/// | profundidade da propagação, e portanto o alcance `K` e o ponto-origem | `R × (1 + deslocamento)` | a deformação entra mais fundo e o **braço de alavanca cresce** |
/// | queda ao longo do contorno | **`R`**, sem deslocamento | o troço de borda afectado **não muda** |
///
/// **Medido no corpus:** deslocamento `1` leva os anéis de `5` para `9`
/// (`165 → 297` vértices movidos) e o deslocamento máximo do *Bend* de `0,367`
/// para `0,661`, enquanto o do *Expand* **fica em `0,1`** — *a lei do expandir
/// não tem braço de alavanca, logo só a contagem de anéis se move.*
///
/// ⚠️ **A faixa pública do alvo é `0 … 30`** e o tecto aqui é `4`: acima disso a
/// propagação consome a peça inteira nas malhas desta casa, e o que se ganha é
/// um número que já não muda a saída. ⛔ *É um tecto de PRODUTO com a medição ao
/// lado, não o número do alvo copiado* — quem tiver uma peça que o justifique
/// sobe-o com a tabela nova.
pub(super) const BOUNDARY_OFFSET: Row = Row {
    label: "panel.sculpt3d.boundary_offset",
    slider: crate::ids::SCULPT3D_BOUNDARY_OFFSET,
    chip: crate::ids::SCULPT3D_BOUNDARY_OFFSET_NUM,
    min: 0.0,
    max: 4.0,   // LITERAL-PX-OK: teto de produto, em raios de pincel — ver acima
    step: 0.05, // LITERAL-PX-OK: knob em raios de pincel
    decimals: 2,
    get: |u| u.brush.boundary.deslocamento_da_origem,
    set: |u, v| u.brush.boundary.deslocamento_da_origem = v,
    show: is_boundary,
    level: UiLevel::Basic,
    place: Place::Knobs,
};
