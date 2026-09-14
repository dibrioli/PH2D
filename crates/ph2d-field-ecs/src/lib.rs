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

/// ⭐⭐⭐ **O MATERIAL DESTA FORMA** — com que aspecto ela responde à luz (`docs/Render3d/05`).
///
/// # ⚠️ Componente PRÓPRIO e OPCIONAL, pelas duas razões do [`FieldMods`]
///
/// A ausência dele significa *«o material de omissão»*, que é o do nodedef do OpenPBR — e um campo
/// apendado ao [`FieldNode`] quebraria todo projecto já gravado (o blob de um componente é postcard
/// **posicional**).
///
/// # ⛔ Ele NÃO viaja no `FieldDoc`, e isso é a decisão
///
/// O documento é **geometria**: é ele que a marcha compila, e uma cor não muda uma distância. ⇒ o
/// `FIELD_DOC_VERSION` **não se mexe** e uma peça gravada antes desta wave continua a abrir, com o
/// material de omissão. Quem junta os dois é o sombreamento, por pixel, pela folha que o ponto
/// nomeia ([`ph2d_field_eval::owners`]).
///
/// # ⚠️ Os números são os do OpenPBR, e o DEFAULT é o da nodedef
///
/// ⛔ **E ele não é copiado daqui para lá:** há gate na `ph2d-app-field3d` (que vê as duas crates) a
/// exigir que este `Default` e o `ph2d_material::OpenPbr::default()` digam o mesmo. *Uma constante
/// escrita em dois sítios ainda não é uma constante — a segunda é a que envelhece.*
#[derive(Component, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct FieldMaterial {
    /// A cor difusa, em **linear** e por canal.
    pub base_color: [f32; 3],
    /// `0` espelho, `1` completamente difuso.
    pub roughness: f32,
    /// `0` dieléctrico (plástico, cerâmica), `1` metal.
    pub metalness: f32,
    /// ⭐⭐⭐ **A LUZ QUE A PRÓPRIA FORMA DÁ** — o `emission_luminance` do OpenPBR.
    ///
    /// `0` é uma superfície que só devolve a luz que recebe; acima disso ela **acrescenta**
    /// radiância, e a peça acende-se sem lâmpada nenhuma.
    ///
    /// ⚠️ **A ponta útil é `1` e foi MEDIDA** (`docs/Render3d/05` §20): com o olhar do produto e a
    /// esfera de omissão, `0 → 1` move o verde médio de `188` para `246` — **`87 %`** de toda a
    /// excursão possível —, `2` compra mais `7 %`, e acima de `32` a saída é **bit a bit a mesma**.
    /// *A faixa do slider é do modelo; o campo numérico continua aberto.*
    pub emission: f32,
    /// A cor dessa luz, em **linear** e por canal — o `emission_color`.
    ///
    /// ⚠️ **Ela multiplica a [`Self::emission`]**, logo com luminância `0` é **inerte**: é por isso
    /// que a linha dela só é publicada acima de zero (`edit_params::params_of`), e não porque um
    /// painel mais curto seja mais bonito. *Um controlo cujo efeito é sempre zero é um controlo
    /// morto com aparência de vivo.*
    pub emission_color: [f32; 3],
    /// ⭐⭐⭐ **O VERNIZ** — o `coat_weight` do OpenPBR, a película transparente por cima de tudo o
    /// resto: a laca de um carro, o verniz de uma madeira, o brilho molhado de um plástico.
    ///
    /// ⚠️ **Ele é um SEGUNDO realce, não um realce mais forte:** a base continua com a rugosidade
    /// dela e o verniz põe um lóbulo próprio por cima — é isso que faz uma superfície baça parecer
    /// envernizada em vez de simplesmente polida. Medido (`docs/Render3d/05` §21): a `0,1` já move
    /// `7` bytes em `50 182` pixels, e a `1` move `63`.
    ///
    /// ⭐ **Os outros quatro números dele são inertes com este a zero** (o `prepare` mistura-os todos
    /// por `coat_weight`), e é por isso que as linhas deles só são publicadas acima de zero.
    pub coat: f32,
    /// A rugosidade do verniz — `0` é uma laca de espelho, `1` é um acabamento acetinado.
    pub coat_roughness: f32,
    /// A cor do verniz, em **linear** e por canal — o `coat_color`, que é uma TINTA: ela multiplica o
    /// que atravessa a película, logo um verniz âmbar amarela a peça por baixo dele.
    ///
    /// ⚠️ É o número do verniz que mais move o quadro: **`120` bytes** no pior pixel.
    pub coat_color: [f32; 3],
    /// O índice de refracção do verniz.
    ///
    /// ⚠️⚠️ **A faixa dele NÃO sai da régua, e sim da FÍSICA** (`docs/Render3d/05` §21): varrido até
    /// `20` o quadro nunca pára de se mexer, logo *«onde deixa de ser observável»* não responde aqui.
    /// O piso é `1` — a luz não atravessa nada mais depressa do que o vácuo — e o tecto é `2,5`,
    /// acima do diamante (`2,42`), que é o mais alto dos materiais transparentes conhecidos.
    pub coat_ior: f32,
    /// Quanto o verniz **escurece** a base por baixo dele — `1` é o físico, `0` desliga o efeito.
    ///
    /// ⚠️ **O Blender não expõe este**, e nós expomos: medido, ele move `29` bytes no pior pixel, e
    /// a lei desta casa é que um número vivo tem controlo. *Uma referência é um oráculo do que a
    /// LEI faz, não um censo do que um painel deve ter.*
    pub coat_darkening: f32,
}

impl Default for FieldMaterial {
    fn default() -> Self {
        Self {
            base_color: [0.8; 3],
            roughness: 0.3,
            metalness: 0.0,
            emission: 0.0,
            emission_color: [1.0; 3],
            coat: 0.0,
            coat_roughness: 0.0,
            coat_color: [1.0; 3],
            coat_ior: 1.6,
            coat_darkening: 1.0,
        }
    }
}

impl FieldMaterial {
    /// Lê um dos [`ph2d_field::MATERIAL_FIELDS`] números, pela posição — ver [`ph2d_field::Param`].
    ///
    /// ⚠️ **Uma tabela e não um `match` por chamador:** a leitura e a escrita têm de concordar sobre
    /// qual número é o `3`, e a única forma de não divergirem é serem irmãs no mesmo ficheiro.
    #[must_use]
    pub fn get(&self, field: u8) -> Option<f32> {
        match field {
            0..=2 => Some(self.base_color[field as usize]),
            3 => Some(self.roughness),
            4 => Some(self.metalness),
            5 => Some(self.emission),
            6..=8 => Some(self.emission_color[field as usize - 6]),
            9 => Some(self.coat),
            10 => Some(self.coat_roughness),
            11..=13 => Some(self.coat_color[field as usize - 11]),
            14 => Some(self.coat_ior),
            15 => Some(self.coat_darkening),
            _ => None,
        }
    }

    /// Escreve um dos números. `false` quando a posição não existe.
    pub fn set(&mut self, field: u8, value: f32) -> bool {
        match field {
            0..=2 => self.base_color[field as usize] = value,
            3 => self.roughness = value,
            4 => self.metalness = value,
            5 => self.emission = value,
            6..=8 => self.emission_color[field as usize - 6] = value,
            9 => self.coat = value,
            10 => self.coat_roughness = value,
            11..=13 => self.coat_color[field as usize - 11] = value,
            14 => self.coat_ior = value,
            15 => self.coat_darkening = value,
            _ => return false,
        }
        true
    }
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
    // ⭐⭐⭐ **O MATERIAL** — `register_default` como os irmãos com neutro: a ausência dele quer dizer
    // *«o material de omissão»*, e esse é exactamente o `Default`.
    reg.register_default::<FieldMaterial>("ph2d::field::FieldMaterial");
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

#[cfg(test)]
mod verb_joint_tests;
