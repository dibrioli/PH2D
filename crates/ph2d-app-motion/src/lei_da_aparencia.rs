//! ⭐⭐⭐ **A LEI DA APARÊNCIA, do lado do DISPOSITIVO** — *uma corrente que não veio de uma forma
//! não vira pixel.*
//!
//! A lei em si vive no avaliador ([`ph2d_eval_motion::tem_aparencia`] + a porta de produto
//! [`ph2d_eval_motion::so_com_forma_por_ordem`]) e morde nos dois lowerings de CPU. Este módulo é a
//! metade que **o caminho da GPU** precisa, e ela mora aqui porque pergunta ao
//! [`MotionState`] — um tipo desta crate, que o avaliador não alcança.
//!
//! ## Porque este ficheiro existe sozinho
//!
//! Ele nasceu dentro do gizmo de posições, que a ordem do dono de 2026-09-19 **retirou** (*«retire
//! tudo relacionado a gizmos desses nós, que devem apenas passar as posições e direções»*). ⚠️ A
//! lei **não** foi retirada com ele: é ela que torna verdadeira a frase que o cartão desses nós
//! agora mostra — *sem um `motion.duplicator` e um objecto a copiar, eles são invisíveis*.
//! *Apagar a lei junto com o desenho dela deixaria o aviso a mentir.*

use crate::motion_state::MotionState;

/// ⭐⭐⭐ **A ARTE DO DISPOSITIVO DESENHA NESTE QUADRO?** — a metade da lei que o caminho da GPU
/// devia ter e não tinha.
///
/// ⛔⛔⛔ **Report do dono, 2026-09-19: *«os retângulos voltaram»*.** A [W1] escreveu a saída cedo
/// nos **dois lowerings de CPU** e o [doc 115 §32.2] declarou, por escrito, que *«por corrente o
/// device apenas não despacha»* — **uma propriedade que ninguém construiu**. Medido:
/// `grep -c so_com_forma crates/ph2d-gpu-cook/src` devolve **`0`**. E a cena que eu próprio lhe
/// apontei — a `=116`, *«102 400 peças no dispositivo»* — é exactamente uma cena de device.
/// *Escrever a propriedade no doc não a constrói; foi preciso o dono abrir o app para a cobrar.*
///
/// ## Como a pergunta se responde SEM ler o dispositivo de volta
///
/// No caminho da GPU a aparência só pode chegar por uma **FRONTEIRA** — o `source.object` lê um
/// external que a membrana publica na CPU, e é a fronteira que viaja para o device. Logo:
///
/// > *a arte do device tem aparência* ⟺ *alguma corrente de fronteira tem aparência*
///
/// ⚠️ E a pergunta é a MESMA porta que o lowering usa ([`ph2d_eval_motion::tem_aparencia`]) — um
/// segundo predicado aqui divergiria no dia em que uma origem nova nascesse.
///
/// ⛔ **A partição de texturas NÃO serve para isto**, e a razão está escrita no doc dela: ela
/// também fica vazia num *«grafo de objectos cujos ladrilhos vivem todos no atlas partilhado»* —
/// usá-la apagaria uma cena de objectos legítima.
///
/// ⚠️ **A lei entra como ARGUMENTO e não é lida do ambiente aqui** — é a lição da auditoria do
/// §31: quem lê o ambiente é a porta do produto, num sítio só.
#[must_use]
pub fn a_arte_desenha(motion: &MotionState, so_com_forma: bool) -> bool {
    if !so_com_forma {
        return true;
    }
    motion
        .pump
        .boundary_streams()
        .iter()
        .any(|(_, s)| ph2d_eval_motion::tem_aparencia(s))
}
