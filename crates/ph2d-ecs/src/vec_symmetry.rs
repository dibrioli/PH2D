//! **A SIMETRIA VIVA de uma forma** — o eixo é do artista, e o outro lado é derivado.
//!
//! Irmão do [`crate::VecOffset`] / [`crate::VecStrokeProfile`] no padrão que esta linha já usou
//! várias vezes: o componente guarda a **relação** (que espelho, onde, quantas cópias) e a
//! aparência é uma **função pura** dela, re-cozida pela shell a cada frame. Consequência de graça:
//! **undo e save cobrem a simetria sem uma linha a mais** — os dois capturam o mundo ECS, e este
//! componente está no `ComponentRegistry`.
//!
//! # Por que um componente, e não um `PathEffect` da pilha
//!
//! ⚠️ **Foi um, e foi REPROVADO** (Enio, 2026-08-01: *"funciona bem mas não é legal como um
//! efeito; melhor como uma opção para as tools de desenho exatamente como o modo painter"*). O que
//! estava errado era o LUGAR, não a matemática. Um efeito é um passo anónimo de uma pilha; uma
//! simetria de desenho é um **modo**: liga-se numa seção própria, vê-se como linhas no canvas, e
//! consolida-se com um **Apply**.
//!
//! E há a consequência técnica: enquanto ligada, as cópias são **DESENHO**, não documento —
//! desligar antes do Apply faz as cópias sumirem **sem destruir nada**, porque nunca houve nada a
//! destruir. Isso é exactamente o que um componente + `LiveGeometry` entrega, e o que um efeito
//! (que reescreve o `cooked()` da forma) tornaria uma pergunta.
//!
//! # O eixo é LOCAL, e é isso que o faz seguir o desenho
//!
//! *"se o usuário mover o objeto no canvas a linha de simetria acompanha mantendo a mesma
//! distância relativa ao objeto"* (Enio). O `center` da [`SymmetrySpec`] vive no espaço da
//! geometria — o mesmo em que os vértices do `VecPath` vivem —, então mover a forma é mexer no
//! `Transform` dela e o eixo viaja junto **sem que ninguém o actualize**. Guardar MUNDO obrigaria
//! um passe a re-derivar o local a cada frame, e re-derivar contra a pose VIVA é precisamente o
//! defeito que a âncora de joint da `line/physics` pagou em 2026-07-25.
//!
//! A outra metade da promessa (*"ao ligar, a linha aparece no centro da TELA"*) é da shell: ela
//! converte o centro da vista para o local da forma **UMA vez**, no instante em que a simetria é
//! armada. Depois disso ninguém mais converte nada.

use bevy_ecs::component::Component;
use ph2d_symmetry::SymmetrySpec;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// **A simetria viva de uma forma.** A entidade que a carrega também tem um [`crate::VecPathRef`]:
/// o `VecPath` dela continua sendo a curva **AUTORADA** (é ela que o modo Node edita), e as cópias
/// são DESENHO — a shell coze-as por frame e o passe de render desenha-as no lugar da fonte, no z
/// dela.
///
/// ⚠️ **A ausência é o neutro.** Uma forma sem simetria não carrega o componente: desarmar
/// REMOVE-o, em vez de guardar um espelho inerte (a mesma lei do `VecOffset` com `d = 0`). É por
/// isso que "desmarcar antes do Apply" é uma operação sem perda — não há estado a preservar.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VecSymmetry {
    /// O que espelhar, onde, quantas vezes. Coordenadas **LOCAIS** da forma.
    pub spec: SymmetrySpec,
}

impl SimComponent for VecSymmetry {}

impl VecSymmetry {
    /// Uma simetria viva nova.
    #[must_use]
    pub fn new(spec: SymmetrySpec) -> Self {
        Self { spec }
    }
}
