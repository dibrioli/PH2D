//! `ph2d-field-ecs` — **a peça é uma CENA de objetos**, não um documento escondido num componente
//! ([ADR-0161]).
//!
//! # ⭐ A árvore de modelagem **é** a hierarquia da cena
//!
//! Cada nó — cada cilindro, cada caixa, cada união — é uma **entidade**. Ela aparece na Hierarquia
//! com nome, é selecionável, tem pose própria, é salva e é desfeita. O documento que o traçador
//! avalia ([`ph2d_field::FieldDoc`]) é **cozido** a partir do mundo, uma vez por quadro ([`cook`]).
//!
//! Isto é a lei da casa dita duas vezes:
//!
//! - **ADR-0110** — *"todo path é entidade ECS com pose no `Transform`; uma hierarquia"*. Um módulo
//!   3D que guardasse a árvore inteira num só componente teria uma segunda forma de organizar
//!   objetos, e o artista teria de aprender as duas.
//! - **ADR-0121/0132** — *fonte ≠ cozido*. A fonte é editável e é o que se vê na Hierarquia; o
//!   cozido é derivado e ninguém o autora.
//!
//! ⚠️ **Até 2026-08-19 não era assim**, e o smoke do Enio encontrou-o em uma frase: *"na hierarchy
//! apenas um objeto e não 3 cilindro"*. O documento inteiro vivia num único `FieldObject { doc }`,
//! e a consequência não era estética — era que **não havia o que um gizmo agarrasse**. Um objeto
//! que a cena não enumera não tem pose que se mova.
//!
//! # ⚠️ Por que a pose NÃO é o `ph2d_ecs::Transform`
//!
//! Medido: o `Transform` da casa é uma afim **2D** — `translation: Vec2`, `rotation: f32` (um
//! ângulo escalar), `scale: Vec2`. Não há onde pôr uma rotação 3D. Escrever meia pose lá e a outra
//! metade aqui seria a segunda verdade na sua forma mais cara: o Inspector mostraria uma posição
//! que a peça não tem.
//!
//! Então a pose 3D é [`FieldPose`], e os nós **não** carregam `Transform`. Isso é seguro, e é
//! medido, não suposto:
//!
//! | Pergunta | Onde está a resposta | Medido |
//! |---|---|---|
//! | A Hierarquia enumera um filho sem `Transform`? | `build_hierarchy_snapshot` | **Sim** — só a RAIZ é filtrada por `With<Transform>`; o DFS desce por `Children` |
//! | O snapshot (save + undo) captura esse filho? | `world_to_snapshot` | **Sim** — a fase 1 desce `Children` sem filtro nenhum |
//!
//! A **raiz** de cada peça leva `Transform` + `RootOrder`, porque é ela que a Hierarquia enumera
//! como objeto de topo.
//!
//! # O que entra num componente, e o que **nunca** entra
//!
//! Entra o que é **autorado** — a forma do nó e a pose dele. ⛔ Não entra nada **derivado**: o
//! documento cozido, a árvore compilada, a malha, o quadro traçado. A lei é da casa e está paga: o
//! `canonicalize` do undo ordena as linhas pelos **bytes** do componente, então algo que mude a
//! cada quadro faz **todo quadro virar um passo espúrio de undo**.
//!
//! [ADR-0161]: ../../../docs/architecture/decisions/0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md

mod cook;
mod edit;
mod spawn;

pub use cook::{cook, world_xform};
pub use edit::{
    add_leaf, add_mod, add_sampled, dims_of, duplicate, mods_of, params_of, radius_bound,
    radius_of, remove, remove_mod, rotate_world, scale_by, set_dim, set_op, set_param, set_radius,
    translate_world, walk, wrap_in_op,
};
pub use spawn::{shape_name, spawn_doc};

use ph2d_ecs::scene::ComponentRegistry;
use ph2d_ecs::{Component, SimComponent};
use ph2d_field::{Blend, FieldDoc, FieldError, NodeShape, Unary, Xform};
use serde::{Deserialize, Serialize};

/// **A raiz de uma peça de modelagem.** Marca a entidade que a Hierarquia mostra como objeto.
///
/// ⚠️ Marcador de tamanho zero **de propósito**: ele não guarda o documento. Guardá-lo aqui foi a
/// forma da W1, e era ela que impedia os nós de existirem como objetos (ver o doc do módulo).
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldObject;

impl SimComponent for FieldObject {}

/// **O que este nó é** — uma primitiva, ou uma operação. Sem os filhos.
///
/// ⚠️ Os filhos são a hierarquia ECS (`Children`) e **só** ela. Ver [`NodeShape`]: é a distinção
/// que impede a forma traçada de discordar da árvore que o artista vê.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FieldNode {
    pub shape: NodeShape,
}

/// **A pose 3D do nó**, local ao pai. Ver a nota do módulo sobre não ser o `Transform` da casa.
#[derive(Component, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct FieldPose {
    pub xform: Xform,
}

/// ⭐ **A pilha de modificadores do nó** — casca, afastamento. Ver [`ph2d_field::mods`].
///
/// ⚠️ **Componente PRÓPRIO, e opcional**, e não um campo apendado ao [`FieldNode`]. As duas razões
/// pesam para o mesmo lado:
///
/// - a esmagadora maioria dos nós **não tem** modificador nenhum, e um `Vec` vazio em cada um é
///   bytes em todo save por uma coisa que quase ninguém usa;
/// - o blob de um componente é postcard **posicional**, então apendar um campo ao `FieldNode`
///   quebraria todo projeto que já o gravou — enquanto um componente **novo** custa zero (é o
///   precedente do `VecStrokeProfile`/ADR-0148 e dos overrides da física, escrito na escada do
///   `PROJECT_SCHEMA`).
#[derive(Component, Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct FieldMods {
    pub stack: Vec<Unary>,
}

impl SimComponent for FieldNode {}
impl SimComponent for FieldPose {}
impl SimComponent for FieldMods {}

impl Default for FieldPose {
    fn default() -> Self {
        Self {
            xform: Xform::IDENTITY,
        }
    }
}

/// Registra os componentes do módulo no registro compartilhado.
///
/// Sem esta chamada o `WorldSnapshot` **descarta o componente em silêncio** — e o sintoma não é um
/// erro: é o objeto sumir ao desfazer ou ao reabrir o arquivo.
///
/// ⚠️ **O identificador vem do NOME** (`stable_type_id`), não de um contador. É o que torna
/// registrar um componente seguro entre linhas paralelas: duas linhas só colidem se escolherem a
/// **mesma string**, e o registro entra em pânico ao vê-lo — em vez de trocar de id em silêncio.
pub fn register_field_components(reg: &mut ComponentRegistry) {
    reg.register::<FieldObject>("ph2d::field::FieldObject");
    reg.register::<FieldNode>("ph2d::field::FieldNode");
    reg.register::<FieldPose>("ph2d::field::FieldPose");
    reg.register::<FieldMods>("ph2d::field::FieldMods");
}

/// O campo de uma **cena**: a união de todos os objetos, na ordem da chave.
///
/// ⚠️ **A chave estável não é cerimônia — é o que impede um bug de undo.** A ordem de uma consulta
/// ECS não é garantida, e unir os documentos na ordem em que a consulta os devolve produziria uma
/// árvore com os mesmos objetos e **bytes diferentes** a cada quadro. O snapshot compara bytes;
/// logo, cada quadro viraria um passo de undo — que é literalmente o bug que o `canonicalize()`
/// do shell existe para matar, e que este repositório já pagou uma vez.
///
/// Por isso a assinatura **exige** a chave em vez de aceitar um iterador solto: uma API que
/// permitisse a ordem errada seria usada na ordem errada.
///
/// Devolve `None` quando não há objeto nenhum — uma cena vazia não tem campo.
///
/// # Errors
/// Propaga a validação de [`FieldDoc::union_all`].
pub fn scene_field<K: Ord>(
    objects: impl IntoIterator<Item = (K, FieldDoc)>,
    blend: Blend,
) -> Option<Result<FieldDoc, FieldError>> {
    let mut v: Vec<(K, FieldDoc)> = objects.into_iter().collect();
    v.sort_by(|a, b| a.0.cmp(&b.0));
    let docs: Vec<FieldDoc> = v.into_iter().map(|(_, d)| d).collect();
    FieldDoc::union_all(&docs, blend)
}

#[cfg(test)]
mod tests;
