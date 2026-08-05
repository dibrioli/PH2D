//! **O VÍNCULO** — a forma que este widget DIRIGE (plano UI/UX W8b.3).
//!
//! A W6.2 fez uma forma **vestir** um controle do catálogo e a W8b.2 pôs a tabela gerada num
//! painel VIVO, onde as rows respondem ao ponteiro. O que faltava era o outro lado do fio: o
//! valor da row não chegava a lugar nenhum. Este componente é esse fio — *que forma este widget
//! dirige?* — e mais nada.
//!
//! # Um campo só, e o que ele NÃO carrega
//!
//! ⚠️ **O componente não diz o que a row FAZ com a forma**, e essa ausência é a decisão: *o que*
//! é derivado do TIPO do widget (um Slider produz um número, um Toggle produz um sim/não), pela
//! porta única do lado da shell. Guardá-lo aqui daria um segundo controle a manter de acordo com
//! o tipo — e no dia em que os dois discordassem, o painel pintaria um slider que apaga a forma.
//! É a mesma lei que a W7 usou para o PAPEL de um estado (`Default`/`Hover`/…) em vez de um nome
//! livre: *com um nome, o gatilho exigiria uma segunda tabela*.
//!
//! # Por que componente, e não um campo do `VecWidget`
//!
//! Um campo bumparia `VEC_SCENE_SCHEMA` **e** `PROJECT_SCHEMA` se vivesse no documento, e mesmo
//! dentro do `VecWidget` ele obrigaria todo sítio de construção a decidir um alvo. Um componente
//! cunha blob-key própria (`stable_type_id` do NOME) ⇒ **zero bump**, e um widget sem vínculo
//! simplesmente não o tem — o precedente exato do [`crate::VecCutPath`] e do
//! [`crate::VecStrokeProfile`].
//!
//! ⚠️ **Sem o registro no `ComponentRegistry` este vínculo é DESCARTADO pelo snapshot** — o
//! artista prenderia a row à forma, salvaria, reabriria, e o painel estaria mudo outra vez.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// **Este widget dirige aquela forma.**
///
/// O alvo é o `VecPathId` (como `u64`) — nunca o nome e nunca os bits da entidade. Bits são id de
/// ALOCAÇÃO e o undo respawna tudo com bits novos; o nome é do artista e muda quando ele quiser.
/// É o mesmo endereço que o [`crate::VecTextPath`] e o [`crate::VecPatternPath`] usam para o
/// caminho-guia deles.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct VecWidgetBind {
    /// O `VecPathId` da forma dirigida, como `u64`.
    pub target: u64,
}

impl SimComponent for VecWidgetBind {}
