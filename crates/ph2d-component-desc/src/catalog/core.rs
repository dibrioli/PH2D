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
    ComponentCategory as C, ComponentDesc as D, FieldDesc, FieldKind as K, ObjectKinds as O,
    Propagation, RefKind,
};

/// Um campo simples que segue o mestre e não é referência — o caso esmagadoramente comum.
const fn f(field_id: u16, label_key: &'static str, kind: K) -> FieldDesc {
    FieldDesc {
        field_id,
        label_key,
        kind,
        policy: Propagation::Propagate,
        is_ref: None,
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
}];

const VISIBILITY: &[FieldDesc] = &[f(1, "component.field.visibility.1", K::Toggle)];

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
}];

// ── Ordenação: a família PILOTO da F0 (a §7 do Inspector) ──────────────────────────

const SORTING_LAYER: &[FieldDesc] = &[f(1, "component.field.sorting_layer.1", K::Enum)];
const ORDER_IN_LAYER: &[FieldDesc] = &[f(1, "component.field.order_in_layer.1", K::Int)];
const Z_INDEX: &[FieldDesc] = &[f(1, "component.field.z_index.1", K::Int)];
const Z_AS_RELATIVE: &[FieldDesc] = &[f(1, "component.field.z_as_relative.1", K::Toggle)];
// ⚠️ **Os rótulos abaixo são os que o Inspector JÁ PINTA, ao byte** — não os que eu
// escolheria. A seção §7 é o piloto da F0, e ao ligá-la ao descritor apareceu o defeito que
// justifica a ligação: as duas fontes já discordavam (*"Sort At Root"* × *"Sort at Root"*,
// *"Y-Sort"* × *"Enabled"*). Quem manda é o produto, então o descritor foi corrigido para
// ele — e agora há **uma** fonte, que é o ponto.
const SORTING_GROUP: &[FieldDesc] = &[f(1, "component.field.sorting_group.1", K::Toggle)];
const Y_SORT: &[FieldDesc] = &[
    f(1, "component.field.y_sort.1", K::Toggle),
    f(2, "component.field.y_sort.2", K::Vec2),
    f(3, "component.field.y_sort.3", K::Enum),
];
const CLIP_CHILDREN: &[FieldDesc] = &[
    f(1, "component.field.clip_children.1", K::Enum),
    f(2, "component.field.clip_children.2", K::Scalar),
];

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
}];

/// Ordenado por `canonical_name` (gate `the_catalog_is_sorted_and_unique`).
pub const DESCS: &[D] = &[
    D::authored(
        "ph2d::ecs::BlendMode",
        "component.blend_mode.name",
        C::Rendering,
        O::IMAGE,
        &[],
    ),
    D::authored(
        "ph2d::ecs::ClipChildren",
        "component.clip_children.name",
        C::Ordering,
        O::ANY,
        CLIP_CHILDREN,
    ),
    // ⇒ a porta é o alternador de GRUPO da Hierarquia, em toda linha.
    D::intrinsic(
        "ph2d::ecs::GroupedChildren",
        "component.grouped_children.name",
        C::Identity,
        MARKER,
    ),
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
    ),
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
    ),
    // ⇒ a porta é o CADEADO da Hierarquia, em toda linha.
    D::intrinsic(
        "ph2d::ecs::Locked",
        "component.locked.name",
        C::Identity,
        MARKER,
    ),
    D::authored(
        "ph2d::ecs::Mask2D",
        "component.mask_2d.name",
        C::Rendering,
        O::IMAGE,
        &[],
    ),
    D::authored(
        "ph2d::ecs::MaskInteraction",
        "component.mask_interaction.name",
        C::Rendering,
        O::IMAGE,
        &[],
    ),
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
    ),
    // ⭐⭐⭐ **O CATAVENTO** (a rota B do `02.2`, 2026-09-21): a malha que um sprite mantém VIVA, e a
    // POSE 3D dela. A pose mora no componente por MEDIÇÃO — o `Transform` tem `rotation: f32` e
    // exprime só o plano do ecrã, que é a rotação que a rota A já dá **exactamente**
    // (`docs/Render3d/17_a_rota_b_o_catavento.md` §1.5).
    //
    // ⛔⛔ **Ele NÃO é um `owned_bridge`, e a 1.ª redacção pô-lo em `bridges` por ANALOGIA com o
    // `Sculpt3dPieceRef`.** A analogia é falsa e o gate da cópia profunda obrigou a medi-la: aquele
    // **É** a peça na árvore do editor (1:1, e a ponte impõe-no), enquanto este apenas **REFERE**
    // uma, e o passe vivo só a **LÊ**. ⇒ duas sprites a apontar para a mesma peça, com poses
    // diferentes, é um campo de cataventos — legítimo, seguro e de graça. *Dropá-la ao duplicar
    // daria uma cópia que perde a forma em silêncio; copiá-la dá uma cópia que funciona.*
    //
    // ⭐⭐ **E ela passou a `authored` no mesmo dia, por ORDEM DO DONO** (*«sim. quero»*, sobre o
    // controlo): o `Attach::Machinery` declara por escrito *«nunca oferecido na paleta, **nunca uma
    // seção do Inspector**»* — ele era o que IMPEDIA a secção de existir. *A pergunta «onde mora o
    // controlo» respondeu-se a MEDIR o enum, não a escolher: o ADR-0166 diz que o Inspector mostra
    // o que o objecto TEM, e o `+` é a rota.*
    //
    // ⚠️ **`IMAGE` e não `ANY`, e é a fase que o decide:** ela só acende quem tem um `BakedForm`, e
    // isso é um sprite. *Um `applies_to` mais largo do que o consumidor entrega um componente que a
    // paleta oferece e que não faz nada* — o controlo morto que esta casa varre a cada wave.
    D::authored(
        "ph2d::ecs::Mesh3D",
        "component.mesh_3d.name",
        C::Model3D,
        O::IMAGE,
        &[],
    ),
    D::intrinsic("ph2d::ecs::Name", "component.name.name", C::Identity, NAME),
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
        &[],
    ),
    // ⇒ a porta é §7 Ordering, pintada em TODO objecto.
    D::intrinsic(
        "ph2d::ecs::OrderInLayer",
        "component.order_in_layer.name",
        C::Ordering,
        ORDER_IN_LAYER,
    ),
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
    ),
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
    ),
    // ⇒ a porta é §7 Ordering.
    D::intrinsic(
        "ph2d::ecs::SortingLayer",
        "component.sorting_layer.name",
        C::Ordering,
        SORTING_LAYER,
    ),
    // ⇒ a porta é a row *Emissive* do §Render Source, pintada em toda sprite (ausente **é** `EMISSIVE_OFF`).
    D::intrinsic(
        "ph2d::ecs::SpriteEmissive",
        "component.sprite_emissive.name",
        C::Rendering,
        &[],
    ),
    // ⭐⭐⭐ **As TAGS** (TOP-20 #9). `C::Identity` porque dizem O QUE o objecto é (a secção nasce
    // logo abaixo da *Identity*), e `O::ANY` porque o objecto vazio que o artista usa como
    // *«o cérebro da cena»* e um grupo são tão marcáveis quanto uma sprite.
    D::authored(
        "ph2d::ecs::Tags",
        "component.tags.name",
        C::Identity,
        O::ANY,
        TAGS,
    ),
    D::authored(
        "ph2d::ecs::TextureFilter",
        "component.texture_filter.name",
        C::Rendering,
        O::IMAGE,
        &[],
    ),
    D::authored(
        "ph2d::ecs::TextureRepeat",
        "component.texture_repeat.name",
        C::Rendering,
        O::IMAGE,
        &[],
    ),
    // ⇒ a porta é §7 Ordering.
    D::intrinsic(
        "ph2d::ecs::TopLevel",
        "component.top_level.name",
        C::Ordering,
        MARKER,
    ),
    D::authored(
        "ph2d::ecs::Transform",
        "component.transform.name",
        C::Transform,
        O::ANY,
        TRANSFORM,
    ),
    D::authored(
        "ph2d::ecs::UvTransform",
        "component.uv_transform.name",
        C::Rendering,
        O::IMAGE,
        &[],
    ),
    // ⇒ a porta é o OLHO da Hierarquia, em toda linha.
    D::intrinsic(
        "ph2d::ecs::Visibility",
        "component.visibility.name",
        C::Identity,
        VISIBILITY,
    ),
    D::authored(
        "ph2d::ecs::VisibilityLayer",
        "component.visibility_layer.name",
        C::Rendering,
        O::IMAGE,
        &[],
    ),
    // ⇒ a porta é §7 Ordering (tres rows: ligar · eixo · ponto).
    D::intrinsic(
        "ph2d::ecs::YSort",
        "component.y_sort.name",
        C::Ordering,
        Y_SORT,
    ),
    // ⇒ a porta é §7 Ordering.
    D::intrinsic(
        "ph2d::ecs::ZAsRelative",
        "component.z_as_relative.name",
        C::Ordering,
        Z_AS_RELATIVE,
    ),
    // ⇒ a porta é §7 Ordering — e o `—` dela é a AUSENCIA deste componente.
    D::intrinsic(
        "ph2d::ecs::ZIndexOverride",
        "component.z_index_override.name",
        C::Ordering,
        Z_INDEX,
    ),
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
        const PORTAS: [&str; 12] = [
            "ph2d::ecs::BlendMode",
            "ph2d::ecs::ClipChildren",
            "ph2d::ecs::Mask2D",
            "ph2d::ecs::MaskInteraction",
            // ⭐⭐ **A 12.ª — o CATAVENTO** (`Mesh3D`, `docs/3D/02.2` rota B), e a pergunta de
            // duas metades responde-se NÃO: a secção LIVE MESH só é pintada COM o componente
            // (ADR-0166), logo **não há controlo sempre visível que o anexe** e a paleta é a
            // única rota. *Fosse `intrinsic`, ele seria inalcançável em todo objecto que ainda
            // não o tivesse — que é a definição de um componente morto.*
            "ph2d::ecs::Mesh3D",
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
