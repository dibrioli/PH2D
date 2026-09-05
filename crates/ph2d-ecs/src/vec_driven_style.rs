//! ⭐⭐⭐ **A APARÊNCIA QUE UM MOTOR CONDUZ** — a ponte entre a linha do tempo e a tinta de um
//! caminho vetorial.
//!
//! # O problema que ela resolve, e por que ele não tinha solução
//!
//! A linha do tempo escreve **no mundo ECS** ([`crate::World`]) e só nele: a assinatura de
//! `write_prop` é `(&mut World, Entity, …)`. A tinta de um caminho vetorial vive no `VecPath`,
//! dentro da `VecScene`, que é um **campo da shell** e não um recurso do ECS — a entidade só
//! carrega a identidade ([`crate::VecPathRef`], cuja doutrina é *"não põe geometria no ECS"*).
//!
//! ⇒ **Não existia assinatura pela qual a linha do tempo alcançasse a opacidade de um vetor**, e
//! por isso o canal `Opacity` estava lá, desenhava curva, aceitava chaves — e **não movia um
//! pixel**. Um controlo mudo, que é o defeito que este repositório caça há waves.
//!
//! Este componente é a ponte, e é o padrão que este módulo já usou sete vezes: o componente guarda
//! a **relação** e a aparência é uma função pura dela, resolvida pela shell a cada quadro (o
//! precedente literal é o [`crate::VecStrokeProfile`], ADR-0148).
//!
//! # ⛔⛔ Por que ele NÃO é registado — e por que a ausência do `Serialize` é o guarda
//!
//! Os irmãos dele (`VecStrokeProfile`, `VecFilter`, `VecOffset`) guardam **autoria**: o artista
//! escreveu-a, ela viaja no save e desfaz-se com Ctrl+Z. Este guarda **o que um motor está a
//! escrever agora**, que é a definição de PRÉ-VISUALIZAÇÃO (`shells/desktop/src/preview_drive.rs`:
//! *"o documento é o valor AUTORADO; o que um motor está a escrever agora é pré-visualização"*).
//!
//! O `world_to_snapshot` itera o `ComponentRegistry` (`scene/save.rs`), então **um componente fora
//! do registo não é fotografado, não é gravado e não empilha undo**. É a mesma decisão do
//! [`crate::StableId`], e pela mesma razão: *a ausência é a decisão*.
//!
//! ⚠️⚠️ **E o guarda contra alguém o registar por engano é ele não derivar `Serialize`:**
//! `ComponentRegistry::register_default` exige `Serialize + DeserializeOwned`, então uma linha de
//! registo **não compila**. ⛔ Acrescentar `Serialize` aqui não é um detalhe de conveniência — é
//! abrir a porta pela qual uma reprodução de 3 s a 60 fps vira **180 passos de undo**, que é
//! exactamente o defeito que o `preview_drive` existe para curar. Se alguma vez for preciso,
//! **PARE** e leia aquele módulo primeiro.
//!
//! # O neutro é a AUSÊNCIA
//!
//! Todos os campos a `None` ⇒ quem escreve **remove o componente**, a mesma lei do `VecOffset` com
//! `d = 0` e do `VecStrokeProfile` uniforme. Sem ela um caminho que já foi animado ficaria com uma
//! opacidade presa depois de a track ser apagada, e nada na tela diria porquê.

use bevy_ecs::component::Component;

use crate::SimComponent;

/// **A aparência viva de um caminho vetorial, escrita por um motor neste quadro.**
///
/// Um campo por canal; `None` = *este canal não está a ser conduzido* e o valor autorado do
/// documento vale. A shell funde isto na **mesma** entrada de estilo que os tokens de design e as
/// rows autoradas já produzem (`ph2d_vec_scene::BoundStyle`), nunca numa segunda entrada — o
/// consumidor lê **uma** por forma, e uma segunda seria descartada em silêncio.
///
/// ⚠️ **Sem `Serialize`, sem `Eq`, e sem registo — as três ausências são deliberadas** (ver o
/// cabeçalho do módulo). `PartialEq` chega: quem o compara pergunta *"mudou desde o quadro
/// passado?"*, nunca *"são a mesma chave?"*.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq)]
pub struct VecDrivenStyle {
    /// **A opacidade que o motor conduz**, `0.0..=1.0`. `None` = não conduzida.
    ///
    /// ⚠️ Ela **escala** o alfa que a forma já tem em vez de o substituir — a lei é a do
    /// `BoundStyle::alpha`, e é o que preserva a ESPÉCIE da tinta: um gradiente continua um
    /// gradiente, um padrão continua um padrão. Trocar a tinta por uma cor com alfa, que seria o
    /// atalho, achataria todo gradiente no primeiro quadro de um fade.
    pub alpha: Option<f32>,
}

impl VecDrivenStyle {
    /// **Nada conduzido** ⇒ quem escreve tem de REMOVER o componente, não guardá-lo vazio.
    #[must_use]
    pub fn is_idle(&self) -> bool {
        self.alpha.is_none()
    }
}

impl SimComponent for VecDrivenStyle {}
