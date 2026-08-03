//! **Os ids dos COMPONENTES** (plano UI/UX W5) — irmão de [`super::vector_anchors`] pelo teto de
//! LOC, e o corte é o assunto: aqui mora o prefab (mestre, instância, override).
//!
//! # Quatro verbos, e cada um aparece onde faz sentido
//!
//! *Create Component* só com uma forma comum selecionada · *Place Instance* só com um mestre ·
//! *Detach* e *Reset Overrides* só com uma instância. ⚠️ A alternativa — quatro botões sempre
//! pintados, três deles inertes — é o botão-morto que este repo persegue: um botão que não faz
//! nada é pior que um botão que falta, porque ensina o artista a duvidar dos outros.
//!
//! # O que NÃO está aqui, e por quê
//!
//! *Update Main* (empurrar as edições de uma instância de volta ao mestre) e *Swap* (trocar a
//! instância por outro componente mantendo overrides compatíveis) são o resto da lista do plano.
//! Os dois pedem uma metade que esta wave não tem: o primeiro precisa de **editar uma instância
//! no lugar** (hoje as edições de instância são os overrides, e o mestre é editado no mestre), o
//! segundo precisa de um **picker de componentes** e da regra de compatibilidade. Ficam nomeados
//! em vez de meio-construídos.

use ph2d_a11y::NodeId;

use crate::ids::hash_node_id;

/// O cabeçalho da seção **Component**.
pub const VECTOR_SECTION_COMPONENT: NodeId = hash_node_id("vector.section.component");

/// **Create Component** — a forma selecionada (e a sub-árvore dela) vira um mestre.
pub const VECTOR_COMPONENT_CREATE: NodeId = hash_node_id("vector.component.create");
/// **Place Instance** — põe uma cópia derivada do mestre selecionado.
pub const VECTOR_COMPONENT_PLACE: NodeId = hash_node_id("vector.component.place");
/// **Detach** — a instância deixa de derivar: o que estava na tela vira geometria dela.
pub const VECTOR_COMPONENT_DETACH: NodeId = hash_node_id("vector.component.detach");
/// **Reset Overrides** — a instância volta a ser exactamente o mestre.
pub const VECTOR_COMPONENT_RESET: NodeId = hash_node_id("vector.component.reset");
