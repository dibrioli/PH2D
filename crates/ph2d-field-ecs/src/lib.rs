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
mod edit_verb;
mod spawn;

pub use cook::{contributes, cook, field_world_xform, is_hidden, set_world_xform, world_xform};
pub use edit::{
    add_leaf, add_mod, add_sampled, can_detach, can_wrap, dims_of, duplicate, mods_of, params_of,
    promote_leaf_hosts, radius_bound, radius_of, remove, remove_mod, rotate_world,
    rotate_world_about, scale_about, scale_by, set_dim, set_op, set_param, set_radius, top_level,
    translate_world, walk, wrap_in_op,
};
pub use edit_verb::{VerbRole, character_of, set_character, set_verb, verb_of, verb_role};
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

/// ⭐⭐ **O DESENHO DE ONDE ESTA FORMA VEIO** — o vínculo vivo entre o contorno do editor vetorial e
/// a peça (W55).
///
/// # O que ele torna possível, e o que ele deliberadamente NÃO faz
///
/// Até esta wave, `+ Extrude` cozia o contorno **uma vez** e o resultado era tudo o que sobrava: o
/// desenho continuava na cena, a peça já não o conhecia, e as duas coisas divergiam em silêncio ao
/// primeiro gesto do artista sobre a curva. Enio, no smoke da W53: *"contudo sem ajustes de
/// resolução"* — e o knob era inexprimível **pela mesma ausência**, porque afinar a conversão exige
/// ter a fonte.
///
/// Com o vínculo, a forma é **derivada** do desenho a cada quadro
/// ([`crate::field3d_profile_live`], no shell) e o [`Self::level`] é o número que decide a finura.
///
/// ⚠️ **Ele segue a FORMA do desenho, nunca a POSE dele.** A pose 3D da peça é do artista — ele
/// colocou-a onde quis, com o gizmo — e o desenho vive noutro espaço, com uma pose 2D própria.
/// Arrastar a curva no canvas 2D **não** teleporta a peça; mudar a curva muda a peça. *Uma pose, um
/// dono.*
///
/// ⚠️ **Componente PRÓPRIO e opcional**, pelas duas razões que o [`FieldMods`] já paga: quase nenhum
/// nó tem um, e o blob de um componente é postcard **posicional** — apendar um campo ao [`FieldNode`]
/// quebraria todo projeto já gravado.
///
/// ⚠️ **`u64` e não `VecPathId`**: esta crate é a ponte ECS do modelador e **não** conhece o
/// documento vetorial, pela mesma lei que o [`ph2d_field::Profile`] copia a `FillRule` em vez de a
/// importar. O tipo lá é um alias de `u64`, e quem traduz é o shell — que é quem tem as duas cenas.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldProfileSource {
    /// O contorno na cena vetorial (`ph2d_vec_scene::VecPathId`).
    pub path: u64,
    /// **Com que finura** o contorno é convertido — `1` é o joelho medido na W54, e o teto é
    /// [`ph2d_field::MAX_PROFILE_RESOLUTION`].
    ///
    /// ⚠️ **O NÍVEL, e não a tolerância.** O número que o artista escreve tem de sobreviver a
    /// mudanças na lei que o traduz: guardar a tolerância cozida faria uma peça salva hoje ficar
    /// presa ao valor de hoje, e re-abri-la depois de o joelho se mover daria uma finura que já
    /// ninguém escolheria. *Guarda-se a intenção, deriva-se o número.*
    pub level: u32,
}

/// ⭐⭐⭐ **O VERBO desta forma** — com que operação ela dobra sobre o resultado dos irmãos que vêm
/// antes dela na Hierarquia. A lei inteira está em [`ph2d_field::fold_verb`].
///
/// # ⚠️ A AUSÊNCIA é que carrega o significado
///
/// Um nó **sem** este componente herda o verbo do pai — e é por isso que ele é um componente
/// opcional e não um campo do [`FieldNode`]. As duas leituras coincidem de propósito:
///
/// | no mundo | no documento | quer dizer |
/// |---|---|---|
/// | sem `FieldVerb` | `Node::verb == None` | *«use o do meu pai»* |
/// | com `FieldVerb` | `Node::verb == Some(op)` | *«eu dobro assim»* |
///
/// ⇒ toda peça anterior a esta wave coze **byte-idêntica**, e o seletor do pai não morre: ele passa
/// a ser o **padrão** de quem não se pronunciou.
///
/// ⚠️ **Componente PRÓPRIO**, pelas duas razões que o [`FieldMods`] já paga (bytes em todo nó ·
/// postcard posicional), mais uma terceira que é só desta: *a ausência é um estado do modelo*, e um
/// `Option` dentro de um componente que existe sempre não a saberia dizer sem a inventar.
///
/// ⚠️ **O verbo do PRIMEIRO irmão não é usado** — ele semeia o acumulado. Guardá-lo mesmo assim é
/// deliberado: reordenar não destrói a escolha de quem passou pelo topo, e arrastar de volta
/// devolve o que estava lá.
#[derive(Component, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct FieldVerb {
    pub op: ph2d_field::Op,
}

impl SimComponent for FieldNode {}
impl SimComponent for FieldPose {}
impl SimComponent for FieldMods {}
impl SimComponent for FieldVerb {}
impl SimComponent for FieldProfileSource {}

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
    reg.register_default::<FieldPose>("ph2d::field::FieldPose");
    reg.register_default::<FieldMods>("ph2d::field::FieldMods");
    // ⚠️ `register`, e **não** `register_default`: este componente não tem neutro. A ausência dele
    // já quer dizer uma coisa (*«herda do pai»*), e um default inventaria um verbo que ninguém
    // escolheu em todo nó que o não tenha.
    reg.register::<FieldVerb>("ph2d::field::FieldVerb");
    reg.register::<FieldProfileSource>("ph2d::field::FieldProfileSource");
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

#[cfg(test)]
mod verb_tests;
