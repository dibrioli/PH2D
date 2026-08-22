//! **A OPERAÇÃO DE UMA FORMA** dentro de uma booleana viva — o que torna exprimível *"somo com
//! esta, subtraio aquela"* sem grafo, sem janela e sem gesto novo (Enio, 2026-08-22).
//!
//! Irmão do [`crate::VecBoolGroup`]: lá mora a operação do **grupo**; aqui, a de uma **forma**.
//!
//! # O modelo, numa frase
//!
//! *As formas do grupo combinam-se na ordem da hierarquia, e cada uma traz o verbo com que ela
//! dobra sobre o resultado das anteriores.*
//!
//! É o **compound shape vivo do Illustrator** (cada componente guarda o seu Shape Mode — Add,
//! Subtract, Intersect, Exclude — e a pilha resolve de baixo para cima) e a **pilha de
//! modificadores booleanos do Blender** (um verbo por cortador, aplicados em ordem). ⚠️ Não é o
//! Figma, onde a operação é do grupo inteiro e não da forma — é justamente essa a limitação que
//! este componente remove.
//!
//! # Ausência é HERANÇA, não *"sem operação"*
//!
//! Sem este componente a forma dobra com o `op` do **grupo**. Duas consequências, e as duas são o
//! desenho:
//!
//! 1. **Todo documento anterior a esta feature desenha byte-idêntico** — nenhuma forma tem o
//!    componente, logo todas herdam o `op` do grupo, que é exatamente o que o grupo fazia.
//! 2. **Os oito botões do painel não morrem.** Um seletor que só mexesse no grupo ficaria inerte
//!    assim que as formas passassem a mandar — o defeito *"parâmetro que não muda nada"*. Aqui o
//!    botão do grupo continua a decidir: ele é o **padrão** de quem não se pronunciou.
//!
//! # ⛔ Só as QUATRO operações de conjunto cabem numa forma
//!
//! `op` é o discriminante de `ph2d_vec_boolean::PathfinderOp`, como no irmão — mas apenas
//! `Union`/`Subtract`/`Intersect`/`Exclude` têm sentido aqui. As outras quatro
//! (`MinusBack`/`Trim`/`Crop`/`Merge`) são afirmações sobre a **PILHA INTEIRA** — *"cada forma
//! menos a união do que está acima dela"* não é uma relação entre duas coisas —, e por isso
//! continuam a ser verbos do grupo. Um código de receita aqui **degrada para herança**, que é a
//! leitura que não perde arte.
//!
//! ⚠️ **Na BASE ele é inerte, e isso é estrutural.** A forma mais ao fundo não dobra sobre nada:
//! quando o motor recebe a cadeia, o verbo viaja emparelhado com o path *que entra*, e a base não
//! entra — ela **é** o acumulador inicial (`ph2d_vec_boolean::apply_chain_checked`). O Illustrator
//! tem a mesma inércia no componente de baixo. É por isso que a UI não pode oferecer o seletor na
//! linha da base: seria um controlo que não controla nada.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// **O verbo com que ESTA forma dobra** sobre o resultado das anteriores, dentro de uma booleana
/// viva. Ausência = herda o `op` do grupo.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VecBoolOp {
    /// O discriminante de `ph2d_vec_boolean::PathfinderOp` (append-only: é o que fica gravado no
    /// save). Fora das quatro operações de conjunto, degrada para a herança do grupo.
    pub op: u8,
}

impl SimComponent for VecBoolOp {}
