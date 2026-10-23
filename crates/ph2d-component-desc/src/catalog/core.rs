//! **O núcleo do `ph2d-ecs`** — identidade, pose, ordenação, e o que o renderer de sprite lê.
//!
//! # A aplicabilidade desta família foi MEDIDA, não arbitrada
//!
//! Método (2026-08-24, refazível):
//!
//! ```text
//! grep -rln '\bBlendMode\b' crates/ph2d-vec-render/src crates/ph2d-render/src \
//!                           crates/ph2d-flip-render/src
//! ```
//!
//! Os de `Rendering` (`BlendMode` · `TextureFilter` · `TextureRepeat` · `UvTransform` ·
//! `Mask2D` · `MaskInteraction` · `VisibilityLayer` · `OnScreenEnabler` · `SpriteEmissive`)
//! resolvem **só em `ph2d-render`** — nenhum é lido pelo renderer vetorial nem pelo do Flip.
//! Logo: [`ObjectKinds::IMAGE`]. *É uma medição do consumidor, não uma opinião sobre o nome.*
//!
//! ⚠️ **A ordenação NÃO é Image-only, e a medição é que o diz:** o `ZIndexOverride` é lido
//! pelo `ph2d-panel-vector`, e um `SortingGroup`/`ClipChildren`/`TopLevel` sobre um pai
//! **vazio** é o caso canónico do Unity (agrupar filhos que desenham). Por isso a família de
//! ordenação é [`ObjectKinds::ANY`] — incluindo `Empty`, que é o pai do agrupamento.
//!
//! ⚠️ **O que esta medição NÃO fecha:** ela responde *"quem lê o componente"*, e não *"em que
//! objeto ele tem efeito VISÍVEL"*. As duas coincidem acima porque o leitor é um renderer só;
//! onde houver dois leitores em famílias diferentes, a resposta pede o smoke.

//! # ⭐⭐ A RÉGUA DO DONO, corrida sobre esta família (2026-09-14)
//!
//! *«um objeto de física (Physics Body) e todas as opções aparecem com ele»* foi o veredito que
//! podou a [`super::physics`] de 30 portas para 1. A varredura completa (ordem dele, mesmo dia)
//! correu a mesma régua aqui, e ela tem **duas** metades — não basta o componente ser editável:
//!
//! 1. existe um controlo **sempre disponível** que o escreve? (⛔ não um que só aparece se o
//!    componente já lá estiver — esse é o caso do `BlendMode`, e é por isso que ele FICA);
//! 2. esse controlo **ANEXA** o componente, em vez de só editar o que já existe?
//!
//! Onde as duas são *sim*, a entrada da paleta escreve o ponto neutro que a row já mostra — a
//! razão 2 do [`crate::Attach::Intrinsic`], palavra por palavra. **11 das 22** estavam nesse caso:
//!
//! | componente | a porta que já existia |
//! |---|---|
//! | `OrderInLayer` · `ZIndexOverride` · `ZAsRelative` · `ShowBehindParent` · `SortingLayer` · `YSort` · `SortingGroup` · `TopLevel` | a **§7 Ordering**, que por decisão do dono (2026-08-30) é pintada em **todo** objecto (gate `the_ordering_section_belongs_to_every_object`), e cujo aplicador declara por escrito *«presence = override … queues a `SetComponent` (insert/update) or `RemoveComponent` (detach)»* — `ph2d-inspector-ordering` |
//! | `Visibility` · `Locked` · `GroupedChildren` | o **olho**, o **cadeado** e o **agrupador** da Hierarquia, que fazem `insert`/`remove` em toda linha (`render_loop/hierarchy.rs`) |
//!
//! ⛔ **E as 11 que FICAM, ficam por MEDIÇÃO e não por inércia:** a §9 Sampling, a §10 Blend, a §8
//! Visibility e a §5 9-Slice só são pintadas **se o componente estiver lá** (tabela `CASES` do
//! `inspector_presence_tests` da shell) — para elas a paleta é a **única** porta, e podá-las
//! tornaria a feature inalcançável. *A mesma régua dá respostas opostas dentro da mesma família, e
//! é isso que a torna uma régua em vez de uma opinião sobre a categoria.*
//!
//! ⚠️ **O `Transform` fica fora da poda de propósito:** toda entidade tem um, logo a paleta nunca o
//! oferece na prática (ela esconde o que o objecto já tem) — mas numa entidade **sem** pose ele é a
//! única porta, e não há medição que diga que essa entidade não pode existir. *Podar por «é óbvio
//! que ninguém precisa» é o que esta régua existe para não fazer.*
//!

use crate::{
    ComponentCategory as C, ComponentDesc as D, FieldDesc, FieldKind as K, Hidden,
    ObjectKinds as O, Propagation, RefKind, WHOLE,
};

/// Um campo simples que segue o mestre e não é referência — o caso esmagadoramente comum.
const fn f(field_id: u16, label_key: &'static str, kind: K) -> FieldDesc {
    FieldDesc {
        field_id,
        label_key,
        kind,
        policy: Propagation::Propagate,
        is_ref: None,
        name: "",
        key: "",
        view: crate::View::Same,
    }
}

/// **`Transform`** — ⚠️ a pose da RAIZ de uma instância é *Local da instância*, e a de uma
/// PEÇA propaga. A política aqui é a da peça (o caso geral); o sítio (raiz × peça) é o que
/// decide, e quem decide é o passe de sync da F4, não esta tabela. *Um tipo, duas respostas,
/// escolhidas pelo lugar.*
///
/// ⚠️ A rotação e os skews vivem em **radianos** no componente; `FieldKind::Angle` é o
/// CONTROLO (que fala graus, a unidade autorada do app). A conversão é do consumidor —
/// declarar graus aqui faria a tabela mentir sobre os bytes.
const TRANSFORM: &[FieldDesc] = &[
    f(1, "component.field.transform.1", K::Vec2),
    f(2, "component.field.transform.2", K::Angle),
    f(3, "component.field.transform.3", K::Vec2),
    f(4, "component.field.transform.4", K::Angle),
    f(5, "component.field.transform.5", K::Angle),
];

/// **`Name`** — ⚠️ `InstanceLocal`: o nome da raiz de uma instância é dela. Senão três
/// instâncias do mesmo mestre partilhariam o nome, e a unicidade que o editor impõe
/// (a crate `ph2d-unique-name`) entraria em guerra com o sync todo o quadro.
const NAME: &[FieldDesc] = &[FieldDesc {
    field_id: 1,
    label_key: "component.field.name.1",
    kind: K::Text,
    policy: Propagation::InstanceLocal,
    is_ref: None,
    name: "",
    key: "",
    view: crate::View::Same,
}];

/// ⏳ **`Editor` provisório (CodeEditor 08 d3):** o cartão pinta «Visible» e o campo guarda `hidden` —
/// o inverso. Um nome `visible` sobre a chave `hidden` leria ao contrário; decisão pendente.
const VISIBILITY: &[FieldDesc] = &[f(1, "component.field.visibility.1", K::Toggle)];

/// ⭐ A linha *Emissive* da §Render Source — o newtype da intensidade.
const SPRITE_EMISSIVE: &[FieldDesc] =
    &[f(1, "component.field.sprite_emissive.1", K::Scalar).named_as("emissive", WHOLE)];

/// **`InstanceOf`** — o `StableId` do mestre de que esta raiz nasceu (F4.2).
///
/// ⚠️ **A política segue o precedente do `Transform` acima:** aqui está a da PEÇA (uma instância
/// aninhada dentro de um mestre — F5), que propaga porque o elo dela faz parte da receita. Na
/// RAIZ de uma instância o elo é dela e nunca vem de cima; *um tipo, duas respostas, e quem
/// escolhe é o sítio* — o passe de sync, não esta tabela.
///
/// ⚠️ É `RefKind::Object` e por isso exige remapeador (censo em `instance_refs.rs`): copiar um
/// mestre que contenha uma instância DELE PRÓPRIO tem de religar a cópia à cópia. Quando o
/// mestre está fora do que se copiou — o caso normal — a busca falha e o elo fica, que é
/// exatamente o certo.
const INSTANCE_OF: &[FieldDesc] = &[FieldDesc {
    field_id: 1,
    label_key: "component.field.instance_of.1",
    kind: K::Ref,
    policy: Propagation::Propagate,
    is_ref: Some(crate::RefKind::Object),
    name: "",
    key: "",
    view: crate::View::Same,
}];

// ── Ordenação: a família PILOTO da F0 (a §7 do Inspector) ──────────────────────────

// ⭐ Os newtypes da §7: um campo só, o valor inteiro (`WHOLE`) — o script lê o índice da camada.
const SORTING_LAYER: &[FieldDesc] =
    &[f(1, "component.field.sorting_layer.1", K::Enum).named_as("sorting_layer", WHOLE)];
const ORDER_IN_LAYER: &[FieldDesc] =
    &[f(1, "component.field.order_in_layer.1", K::Int).named_as("order_in_layer", WHOLE)];
const Z_INDEX: &[FieldDesc] =
    &[f(1, "component.field.z_index.1", K::Int).named_as("z_index", WHOLE)];
const Z_AS_RELATIVE: &[FieldDesc] =
    &[f(1, "component.field.z_as_relative.1", K::Toggle).named_as("z_as_relative", WHOLE)];
// ⚠️ **Os rótulos abaixo são os que o Inspector JÁ PINTA, ao byte** — não os que eu
// escolheria. A seção §7 é o piloto da F0, e ao ligá-la ao descritor apareceu o defeito que
// justifica a ligação: as duas fontes já discordavam (*"Sort At Root"* × *"Sort at Root"*,
// *"Y-Sort"* × *"Enabled"*). Quem manda é o produto, então o descritor foi corrigido para
// ele — e agora há **uma** fonte, que é o ponto.
const SORTING_GROUP: &[FieldDesc] =
    &[f(1, "component.field.sorting_group.1", K::Toggle).named("sort_at_root")];
const Y_SORT: &[FieldDesc] = &[
    f(1, "component.field.y_sort.1", K::Toggle).named_as("y_sort", "enabled"),
    f(2, "component.field.y_sort.2", K::Vec2).named("axis"),
    f(3, "component.field.y_sort.3", K::Enum).named("sort_point"),
];
// ⚠️ A §8 pinta só o MODO (a linha «Clip Children»; `Disabled` destaca o componente) — o corte
// alfa dele não tem linha (a «Mask Alpha Cutoff» é da `MaskInteraction`).
const CLIP_CHILDREN: &[FieldDesc] = &[
    f(1, "component.field.clip_children.1", K::Enum).named_as("clip_children", "mode"),
    f(2, "component.field.clip_children.2", K::Scalar).keyed("alpha_cutoff"),
];

// ── As linhas da §8 Visibility, §9 Sampling e §10 Material & Blend (CodeEditor 08 d3) ──────────

const BLEND_MODE: &[FieldDesc] =
    &[f(1, "component.field.blend_mode.1", K::Enum).named_as("blend_mode", WHOLE)];
const MASK_INTERACTION: &[FieldDesc] = &[
    f(1, "component.field.mask_interaction.1", K::Enum).named_as("mask_interaction", "mode"),
    f(2, "component.field.mask_interaction.2", K::Scalar)
        .named_as("mask_alpha_cutoff", "alpha_cutoff"),
];
const ON_SCREEN_ENABLER: &[FieldDesc] =
    &[f(1, "component.field.on_screen_enabler.1", K::Vec4).named_as("enabler_rect", "rect")];
const TEXTURE_FILTER: &[FieldDesc] =
    &[f(1, "component.field.texture_filter.1", K::Enum).named_as("texture_filter", WHOLE)];
const TEXTURE_REPEAT: &[FieldDesc] =
    &[f(1, "component.field.texture_repeat.1", K::Enum).named_as("texture_repeat", WHOLE)];
const UV_TRANSFORM: &[FieldDesc] = &[
    f(1, "component.field.uv_transform.1", K::Vec2).named_as("uv_scale", "scale"),
    f(2, "component.field.uv_transform.2", K::Vec2).named_as("uv_offset", "offset"),
];
// ⚠️ Uma máscara de bits (32 camadas); o cartão pinta-a como caixas, o script lê o inteiro.
const VISIBILITY_LAYER: &[FieldDesc] =
    &[f(1, "component.field.visibility_layer.1", K::Int).named_as("visibility_layer", WHOLE)];

/// Um marcador de tamanho zero: a **presença** é o valor.
const MARKER: &[FieldDesc] = &[f(1, "component.field.marker.1", K::Marker)];

/// **`Tags`** — as tags DIRECTAS do objecto (TOP-20 #9, `docs/Components/08_plano_tags.md`).
///
/// ⚠️ **`Ref` + [`RefKind::Tag`], e não `Text`:** o controlo é um *picker* (chips + a busca dobrada),
/// e o que ele guarda é uma IDENTIDADE da árvore do projecto — descrevê-lo como texto prometeria uma
/// caixa onde se escreve o nome, e renomear a tag desligaria o objecto.
///
/// ⚠️ **`Propagate` (decisão do dono D4):** a tag da receita chega às cópias, e a cópia pode ter a
/// sua própria lista — um override por campo como outro qualquer. O `TagId` não se remapeia ao
/// copiar (ver [`RefKind::Tag`]).
const TAGS: &[FieldDesc] = &[FieldDesc {
    field_id: 1,
    label_key: "component.field.tags.1",
    kind: K::Ref,
    policy: Propagation::Propagate,
    is_ref: Some(RefKind::Tag),
    name: "",
    key: "",
    view: crate::View::Same,
}];

/// Ordenado por `canonical_name` (gate `the_catalog_is_sorted_and_unique`).
pub const DESCS: &[D] = &[
    D::authored(
        "ph2d::ecs::BlendMode",
        "component.blend_mode.name",
        C::Rendering,
        O::IMAGE,
        BLEND_MODE,
    )
    .coded("blend_mode"),
    D::authored(
        "ph2d::ecs::ClipChildren",
        "component.clip_children.name",
        C::Ordering,
        O::ANY,
        CLIP_CHILDREN,
    )
    .coded("clip_children")
    .hiding(&[("alpha_cutoff", Hidden::NoCard)]),
    // ⇒ a porta é o alternador de GRUPO da Hierarquia, em toda linha.
    D::intrinsic(
        "ph2d::ecs::GroupedChildren",
        "component.grouped_children.name",
        C::Identity,
        MARKER,
    )
    .hidden(Hidden::Editor),
    // ⭐ **O ELO de uma instância ao mestre** (ADR-0164 / F4.2) — a raiz de uma instância diz
    // de que receita ela nasceu, pelo `StableId` do mestre.
    //
    // ⚠️ **`Intrinsic` pela razão do `MasterRoot`:** ele chega pelo gesto *«instanciar»*, que
    // COPIA a subárvore antes de o pôr. Oferecê-lo na paleta daria uma raiz que se diz instância
    // de um mestre de que ninguém copiou peça nenhuma — e o sync (F4.3) escreveria por cima do
    // que o artista tem, ou não escreveria nada, os dois calados.
    D::intrinsic(
        "ph2d::ecs::InstanceOf",
        "component.instance_of.name",
        C::Instancing,
        INSTANCE_OF,
    )
    .hidden(Hidden::Editor),
    // ⭐⭐ **A cópia LIGADA** (Enio, 2026-08-27) — o `Alt+D` do Blender: esta peça divide a ARTE da
    // receita, então editar a tinta ou o desenho dela **sobe** e chega às irmãs, em vez de virar
    // excepção dela.
    //
    // ⚠️ **`Intrinsic` pela razão do `InstanceOf`:** ela chega pelo verbo *Instantiate Linked*, que
    // copia a subárvore inteira antes de a pôr. Oferecê-la na paleta poria a marca numa entidade
    // que não é cópia de nada — e a subida de pixels procuraria um mestre que não existe, calada.
    //
    // ⭐ E ela aparece no Inspector porque é a única superfície que diz ao artista **qual das duas
    // leis** esta cópia segue: as duas são iguais na tela até ao gesto seguinte.
    D::intrinsic(
        "ph2d::ecs::LinkedArt",
        "component.linked_art.name",
        C::Instancing,
        MARKER,
    )
    .hidden(Hidden::Editor),
    // ⇒ a porta é o CADEADO da Hierarquia, em toda linha.
    D::intrinsic(
        "ph2d::ecs::Locked",
        "component.locked.name",
        C::Identity,
        MARKER,
    )
    .hidden(Hidden::Editor),
    // ⚠️ `Presence`: a §8 só o liga e desliga («Mask Source»); o corte alfa dele não tem linha.
    D::authored(
        "ph2d::ecs::Mask2D",
        "component.mask_2d.name",
        C::Rendering,
        O::IMAGE,
        &[],
    )
    .hidden(Hidden::Presence),
    D::authored(
        "ph2d::ecs::MaskInteraction",
        "component.mask_interaction.name",
        C::Rendering,
        O::IMAGE,
        MASK_INTERACTION,
    )
    .coded("mask_interaction"),
    // ⚠️ `Name` NÃO tem `Default` (o compilador disse-o ao converter o registador), e a
    // ausência está certa: um objeto sem nome não é um objeto com nome vazio. Ele chega com
    // o gesto que cria o objeto e é **base** — a paleta oferecê-lo seria oferecer algo que
    // toda entidade já tem.
    // ⭐ **A raiz de um MESTRE** (ADR-0164 / F4.1) — a receita que a biblioteca guarda.
    //
    // ⚠️ **`Intrinsic`, e não `Authored`:** ela chega pelo GESTO *«criar componente»*, que faz
    // muito mais do que pôr um marcador (leva a subárvore para a biblioteca e deixa uma instância
    // no lugar). Oferecê-la na paleta daria ao artista um caminho que põe a marca **sem** o resto —
    // uma biblioteca com uma receita que ninguém instanciou, e uma subárvore que deixou de cair
    // sem nada na tela a explicar porquê.
    //
    // ⛔ O `MasterPiece` **não tem entrada aqui**, e a ausência é a decisão: ele não é registado
    // (é derivado), e o censo de dois lados proíbe um descritor que nomeie um tipo fora do registo.
    D::intrinsic(
        "ph2d::ecs::MasterRoot",
        "component.master_root.name",
        C::Instancing,
        &[],
    )
    .hidden(Hidden::Editor),
    // ⭐ **O objecto Motion** (doc ObjectsAndComponents 01 §13, F5) — chega com o gesto que o cria
    // (Add ▸ Motion) e é marcador do tipo [`crate::ObjectKind::Motion`]; anexá-lo pela paleta faria
    // um gerador sem nós.
    D::intrinsic(
        "ph2d::ecs::MotionGenerator",
        "component.motion_generator.name",
        C::Animation,
        &[],
    )
    .hidden(Hidden::Editor),
    // ⚠️ `Editor`: o nome é a IDENTIDADE por que o script acha objectos (`ph2d.find_by_name`) e que
    // as referências duráveis seguem — renomear a correr é outro verbo, não uma propriedade.
    D::intrinsic("ph2d::ecs::Name", "component.name.name", C::Identity, NAME)
        .hidden(Hidden::Editor),
    // ⭐ **As EXCEPÇÕES de uma instância** (ADR-0164 / F4.4) — o conjunto de `(peça, componente)`
    // que a cópia possui contra a receita.
    //
    // ⚠️ **`Machinery`, e não `Intrinsic`:** os outros dois da família chegam por um GESTO do
    // artista (*criar componente*, *instanciar*); este é **mantido pelo passe de sync**, que o
    // escreve sozinho quando o artista mexe numa peça. Oferecê-lo em qualquer porta daria um
    // conjunto de chaves que ninguém sabe preencher à mão.
    D::machinery(
        "ph2d::ecs::ObjectInstance",
        "component.object_instance.name",
        C::Instancing,
    ),
    D::authored(
        "ph2d::ecs::OnScreenEnabler",
        "component.on_screen_enabler.name",
        C::Rendering,
        O::IMAGE,
        ON_SCREEN_ENABLER,
    )
    .coded("on_screen_enabler")
    .hiding(&[("mode", Hidden::NoCard)]),
    // ⇒ a porta é §7 Ordering, pintada em TODO objecto.
    D::intrinsic(
        "ph2d::ecs::OrderInLayer",
        "component.order_in_layer.name",
        C::Ordering,
        ORDER_IN_LAYER,
    )
    .coded("order_in_layer"),
    // ⚠️ Máquina: o editor mantém-no para desempatar raízes (*"não se escolhe um desempate
    // melhor, não se tem empate"*). Um artista que o pusesse à mão estaria a escrever num
    // campo que o próprio editor reescreve no quadro seguinte.
    D::machinery(
        "ph2d::ecs::RootOrder",
        "component.root_order.name",
        C::Ordering,
    ),
    // ⇒ a porta é §7 Ordering.
    D::intrinsic(
        "ph2d::ecs::ShowBehindParent",
        "component.show_behind_parent.name",
        C::Ordering,
        MARKER,
    )
    .hidden(Hidden::Presence),
    // ⚠️ Máquina, pela MESMA razão do `RootOrder` (o gémeo dele para raízes): a ordem entre
    // irmãos é escrita pelo GESTO de arrastar na Hierarquia, e o editor mantém-na. Um artista
    // que a pusesse à mão estaria a escrever num campo que a varredura reescreve.
    D::machinery(
        "ph2d::ecs::SiblingOrder",
        "component.sibling_order.name",
        C::Ordering,
    ),
    // ⇒ a porta é §7 Ordering (*Sorting Group* + *Sort At Root*).
    D::intrinsic(
        "ph2d::ecs::SortingGroup",
        "component.sorting_group.name",
        C::Ordering,
        SORTING_GROUP,
    )
    .coded("sorting_group"),
    // ⇒ a porta é §7 Ordering.
    D::intrinsic(
        "ph2d::ecs::SortingLayer",
        "component.sorting_layer.name",
        C::Ordering,
        SORTING_LAYER,
    )
    .coded("sorting_layer"),
    // ⇒ a porta é a row *Emissive* do §Render Source, pintada em toda sprite (ausente **é** `EMISSIVE_OFF`).
    D::intrinsic(
        "ph2d::ecs::SpriteEmissive",
        "component.sprite_emissive.name",
        C::Rendering,
        SPRITE_EMISSIVE,
    )
    .coded("emissive"),
    // ⭐⭐⭐ **As TAGS** (TOP-20 #9). `C::Identity` porque dizem O QUE o objecto é (a secção nasce
    // logo abaixo da *Identity*), e `O::ANY` porque o objecto vazio que o artista usa como
    // *«o cérebro da cena»* e um grupo são tão marcáveis quanto uma sprite.
    // ⏳ `Reference`: são identidades da árvore de tags — ganham nome pelo NOME da tag (d4).
    D::authored(
        "ph2d::ecs::Tags",
        "component.tags.name",
        C::Identity,
        O::ANY,
        TAGS,
    )
    .hidden(Hidden::Reference),
    D::authored(
        "ph2d::ecs::TextureFilter",
        "component.texture_filter.name",
        C::Rendering,
        O::IMAGE,
        TEXTURE_FILTER,
    )
    .coded("texture_filter"),
    D::authored(
        "ph2d::ecs::TextureRepeat",
        "component.texture_repeat.name",
        C::Rendering,
        O::IMAGE,
        TEXTURE_REPEAT,
    )
    .coded("texture_repeat"),
    // ⇒ a porta é §7 Ordering.
    D::intrinsic(
        "ph2d::ecs::TopLevel",
        "component.top_level.name",
        C::Ordering,
        MARKER,
    )
    .hidden(Hidden::Presence),
    // ⏳ A pose não leva prefixo: o script tem `x y rotation scale_x scale_y` (doc CodeEditor 08
    // §3.1). `Editor` provisório — o cartão mostra também o `Skew X / Y`, que o script não alcança.
    D::authored(
        "ph2d::ecs::Transform",
        "component.transform.name",
        C::Transform,
        O::ANY,
        TRANSFORM,
    )
    .hidden(Hidden::Editor),
    D::authored(
        "ph2d::ecs::UvTransform",
        "component.uv_transform.name",
        C::Rendering,
        O::IMAGE,
        UV_TRANSFORM,
    )
    .coded("uv_transform"),
    // ⇒ a porta é o OLHO da Hierarquia, em toda linha.
    D::intrinsic(
        "ph2d::ecs::Visibility",
        "component.visibility.name",
        C::Identity,
        VISIBILITY,
    )
    .hidden(Hidden::Editor),
    D::authored(
        "ph2d::ecs::VisibilityLayer",
        "component.visibility_layer.name",
        C::Rendering,
        O::IMAGE,
        VISIBILITY_LAYER,
    )
    .coded("visibility_layer"),
    // ⇒ a porta é §7 Ordering (tres rows: ligar · eixo · ponto).
    D::intrinsic(
        "ph2d::ecs::YSort",
        "component.y_sort.name",
        C::Ordering,
        Y_SORT,
    )
    .coded("y_sort"),
    // ⇒ a porta é §7 Ordering.
    D::intrinsic(
        "ph2d::ecs::ZAsRelative",
        "component.z_as_relative.name",
        C::Ordering,
        Z_AS_RELATIVE,
    )
    .coded("z_as_relative"),
    // ⇒ a porta é §7 Ordering — e o `—` dela é a AUSENCIA deste componente.
    D::intrinsic(
        "ph2d::ecs::ZIndexOverride",
        "component.z_index_override.name",
        C::Ordering,
        Z_INDEX,
    )
    .coded("z_index"),
];

#[cfg(test)]
mod tests {
    use super::DESCS;

    /// ⭐⭐ **O núcleo oferece INTENÇÕES, e nenhuma row de secção sempre-pintada.**
    ///
    /// Irmão do `the_physics_family_offers_one_door_and_not_its_rows`, e pela mesma razão: a
    /// família nasceu inteira com o helper autorado e **ninguém classificou item a item** —
    /// medido em 2026-09-14, `22` das 29 entradas eram oferecidas, e 11 delas duplicavam um
    /// controlo que o artista já tem sem abrir a paleta.
    ///
    /// ⚠️ **A lista é a afirmação, não o número.** Uma entrada nova aqui é uma pergunta com duas
    /// metades: *existe um controlo SEMPRE visível que o escreve, e ele ANEXA?* Se sim, é `i`/
    /// `D::intrinsic` e esta lista não cresce; se não, é `D::authored` e cresce com ela.
    ///
    /// ⚠️ **O piso de população é a metade de obsolescência:** sem ele, apagar a família deixaria
    /// o `assert_eq` a comparar dois vazios — *um zero de «não medido» e um de «perfeito» são o
    /// mesmo byte*.
    ///
    /// (Mutação: devolver o `ZIndexOverride` a `D::authored` ⇒ RED, nomeando-o.)
    #[test]
    fn the_core_family_offers_intentions_and_not_rows() {
        const PORTAS: [&str; 11] = [
            "ph2d::ecs::BlendMode",
            "ph2d::ecs::ClipChildren",
            "ph2d::ecs::Mask2D",
            "ph2d::ecs::MaskInteraction",
            "ph2d::ecs::OnScreenEnabler",
            "ph2d::ecs::Tags",
            "ph2d::ecs::TextureFilter",
            "ph2d::ecs::TextureRepeat",
            "ph2d::ecs::Transform",
            "ph2d::ecs::UvTransform",
            "ph2d::ecs::VisibilityLayer",
        ];
        let offered: Vec<&str> = DESCS
            .iter()
            .filter(|d| d.is_offered())
            .map(|d| d.canonical_name)
            .collect();
        assert_eq!(
            offered,
            PORTAS.to_vec(),
            "a paleta do NUCLEO tem de oferecer exatamente estas 12 intencoes.\n\
             Um componente novo aqui e' uma pergunta de DUAS metades: existe um controlo SEMPRE \
             visivel que o escreve (nao um que so' aparece se ele ja' la' estiver), e esse \
             controlo ANEXA-o? Se sim, e' `D::intrinsic` — a row ja' e' a porta, e a entrada da \
             paleta escreveria o ponto neutro que ela ja' mostra. Ver a tabela no cabecalho."
        );
        assert!(
            DESCS.len() >= 25,
            "a familia encolheu para {} — o gate acima passaria a comparar duas listas quase \
             vazias e a nao afirmar nada",
            DESCS.len()
        );
    }
}
