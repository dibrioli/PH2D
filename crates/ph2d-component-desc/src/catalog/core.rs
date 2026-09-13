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
const fn f(field_id: u16, name: &'static str, kind: K) -> FieldDesc {
    FieldDesc {
        field_id,
        name,
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
    f(1, "Position", K::Vec2),
    f(2, "Rotation", K::Angle),
    f(3, "Scale", K::Vec2),
    f(4, "Skew X", K::Angle),
    f(5, "Skew Y", K::Angle),
];

/// **`Name`** — ⚠️ `InstanceLocal`: o nome da raiz de uma instância é dela. Senão três
/// instâncias do mesmo mestre partilhariam o nome, e a unicidade que o editor impõe
/// (a crate `ph2d-unique-name`) entraria em guerra com o sync todo o quadro.
const NAME: &[FieldDesc] = &[FieldDesc {
    field_id: 1,
    name: "Name",
    kind: K::Text,
    policy: Propagation::InstanceLocal,
    is_ref: None,
}];

const VISIBILITY: &[FieldDesc] = &[f(1, "Hidden", K::Toggle)];

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
    name: "Master",
    kind: K::Ref,
    policy: Propagation::Propagate,
    is_ref: Some(crate::RefKind::Object),
}];

// ── Ordenação: a família PILOTO da F0 (a §7 do Inspector) ──────────────────────────

const SORTING_LAYER: &[FieldDesc] = &[f(1, "Sorting Layer", K::Enum)];
const ORDER_IN_LAYER: &[FieldDesc] = &[f(1, "Order in Layer", K::Int)];
const Z_INDEX: &[FieldDesc] = &[f(1, "Z Index", K::Int)];
const Z_AS_RELATIVE: &[FieldDesc] = &[f(1, "Z as Relative", K::Toggle)];
// ⚠️ **Os rótulos abaixo são os que o Inspector JÁ PINTA, ao byte** — não os que eu
// escolheria. A seção §7 é o piloto da F0, e ao ligá-la ao descritor apareceu o defeito que
// justifica a ligação: as duas fontes já discordavam (*"Sort At Root"* × *"Sort at Root"*,
// *"Y-Sort"* × *"Enabled"*). Quem manda é o produto, então o descritor foi corrigido para
// ele — e agora há **uma** fonte, que é o ponto.
const SORTING_GROUP: &[FieldDesc] = &[f(1, "Sort At Root", K::Toggle)];
const Y_SORT: &[FieldDesc] = &[
    f(1, "Y-Sort", K::Toggle),
    f(2, "Axis", K::Vec2),
    f(3, "Sort Point", K::Enum),
];
const CLIP_CHILDREN: &[FieldDesc] = &[f(1, "Mode", K::Enum), f(2, "Alpha Cutoff", K::Scalar)];

/// Um marcador de tamanho zero: a **presença** é o valor.
const MARKER: &[FieldDesc] = &[f(1, "Present", K::Marker)];

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
    name: "Tags",
    kind: K::Ref,
    policy: Propagation::Propagate,
    is_ref: Some(RefKind::Tag),
}];

/// Ordenado por `canonical_name` (gate `the_catalog_is_sorted_and_unique`).
pub const DESCS: &[D] = &[
    D::authored(
        "ph2d::ecs::BlendMode",
        "Blend Mode",
        C::Rendering,
        O::IMAGE,
        &[],
    ),
    D::authored(
        "ph2d::ecs::ClipChildren",
        "Clip Children",
        C::Ordering,
        O::ANY,
        CLIP_CHILDREN,
    ),
    // ⇒ a porta é o alternador de GRUPO da Hierarquia, em toda linha.
    D::intrinsic(
        "ph2d::ecs::GroupedChildren",
        "Grouped Children",
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
        "Instance",
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
    D::intrinsic("ph2d::ecs::LinkedArt", "Linked Art", C::Instancing, MARKER),
    // ⇒ a porta é o CADEADO da Hierarquia, em toda linha.
    D::intrinsic("ph2d::ecs::Locked", "Locked", C::Identity, MARKER),
    D::authored("ph2d::ecs::Mask2D", "Mask", C::Rendering, O::IMAGE, &[]),
    D::authored(
        "ph2d::ecs::MaskInteraction",
        "Mask Interaction",
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
    D::intrinsic("ph2d::ecs::MasterRoot", "Master", C::Instancing, &[]),
    D::intrinsic("ph2d::ecs::Name", "Name", C::Identity, NAME),
    // ⭐ **As EXCEPÇÕES de uma instância** (ADR-0164 / F4.4) — o conjunto de `(peça, componente)`
    // que a cópia possui contra a receita.
    //
    // ⚠️ **`Machinery`, e não `Intrinsic`:** os outros dois da família chegam por um GESTO do
    // artista (*criar componente*, *instanciar*); este é **mantido pelo passe de sync**, que o
    // escreve sozinho quando o artista mexe numa peça. Oferecê-lo em qualquer porta daria um
    // conjunto de chaves que ninguém sabe preencher à mão.
    D::machinery("ph2d::ecs::ObjectInstance", "Overrides", C::Instancing),
    D::authored(
        "ph2d::ecs::OnScreenEnabler",
        "On-Screen Enabler",
        C::Rendering,
        O::IMAGE,
        &[],
    ),
    // ⇒ a porta é §7 Ordering, pintada em TODO objecto.
    D::intrinsic(
        "ph2d::ecs::OrderInLayer",
        "Order in Layer",
        C::Ordering,
        ORDER_IN_LAYER,
    ),
    // ⚠️ Máquina: o editor mantém-no para desempatar raízes (*"não se escolhe um desempate
    // melhor, não se tem empate"*). Um artista que o pusesse à mão estaria a escrever num
    // campo que o próprio editor reescreve no quadro seguinte.
    D::machinery("ph2d::ecs::RootOrder", "Root Order", C::Ordering),
    // ⇒ a porta é §7 Ordering.
    D::intrinsic(
        "ph2d::ecs::ShowBehindParent",
        "Show Behind Parent",
        C::Ordering,
        MARKER,
    ),
    // ⚠️ Máquina, pela MESMA razão do `RootOrder` (o gémeo dele para raízes): a ordem entre
    // irmãos é escrita pelo GESTO de arrastar na Hierarquia, e o editor mantém-na. Um artista
    // que a pusesse à mão estaria a escrever num campo que a varredura reescreve.
    D::machinery("ph2d::ecs::SiblingOrder", "Sibling Order", C::Ordering),
    // ⇒ a porta é §7 Ordering (*Sorting Group* + *Sort At Root*).
    D::intrinsic(
        "ph2d::ecs::SortingGroup",
        "Sorting Group",
        C::Ordering,
        SORTING_GROUP,
    ),
    // ⇒ a porta é §7 Ordering.
    D::intrinsic(
        "ph2d::ecs::SortingLayer",
        "Sorting Layer",
        C::Ordering,
        SORTING_LAYER,
    ),
    // ⇒ a porta é a row *Emissive* do §Render Source, pintada em toda sprite (ausente **é** `EMISSIVE_OFF`).
    D::intrinsic("ph2d::ecs::SpriteEmissive", "Emissive", C::Rendering, &[]),
    // ⭐⭐⭐ **As TAGS** (TOP-20 #9). `C::Identity` porque dizem O QUE o objecto é (a secção nasce
    // logo abaixo da *Identity*), e `O::ANY` porque o objecto vazio que o artista usa como
    // *«o cérebro da cena»* e um grupo são tão marcáveis quanto uma sprite.
    D::authored("ph2d::ecs::Tags", "Tags", C::Identity, O::ANY, TAGS),
    D::authored(
        "ph2d::ecs::TextureFilter",
        "Texture Filter",
        C::Rendering,
        O::IMAGE,
        &[],
    ),
    D::authored(
        "ph2d::ecs::TextureRepeat",
        "Texture Repeat",
        C::Rendering,
        O::IMAGE,
        &[],
    ),
    // ⇒ a porta é §7 Ordering.
    D::intrinsic("ph2d::ecs::TopLevel", "Top Level", C::Ordering, MARKER),
    D::authored(
        "ph2d::ecs::Transform",
        "Transform",
        C::Transform,
        O::ANY,
        TRANSFORM,
    ),
    D::authored(
        "ph2d::ecs::UvTransform",
        "UV Transform",
        C::Rendering,
        O::IMAGE,
        &[],
    ),
    // ⇒ a porta é o OLHO da Hierarquia, em toda linha.
    D::intrinsic(
        "ph2d::ecs::Visibility",
        "Visibility",
        C::Identity,
        VISIBILITY,
    ),
    D::authored(
        "ph2d::ecs::VisibilityLayer",
        "Visibility Layer",
        C::Rendering,
        O::IMAGE,
        &[],
    ),
    // ⇒ a porta é §7 Ordering (tres rows: ligar · eixo · ponto).
    D::intrinsic("ph2d::ecs::YSort", "Y Sort", C::Ordering, Y_SORT),
    // ⇒ a porta é §7 Ordering.
    D::intrinsic(
        "ph2d::ecs::ZAsRelative",
        "Z as Relative",
        C::Ordering,
        Z_AS_RELATIVE,
    ),
    // ⇒ a porta é §7 Ordering — e o `—` dela é a AUSENCIA deste componente.
    D::intrinsic("ph2d::ecs::ZIndexOverride", "Z Index", C::Ordering, Z_INDEX),
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
        const PORTAS: [&str; 10] = [
            "ph2d::ecs::BlendMode",
            "ph2d::ecs::ClipChildren",
            "ph2d::ecs::Mask2D",
            "ph2d::ecs::MaskInteraction",
            "ph2d::ecs::OnScreenEnabler",
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
            "a paleta do NUCLEO tem de oferecer exatamente estas 11 intencoes.\n\
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
