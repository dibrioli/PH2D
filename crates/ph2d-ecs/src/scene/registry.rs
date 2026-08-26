//! `ComponentRegistry` — manual map from stable canonical names to
//! their serde-driven insert/serialize fn pointers.
//!
//! # Why manual (not `inventory`)
//!
//! ADR-0025 §"Decisões finalizadas" → ComponentRegistry strategy =
//! manual via `register_*_components()`. Reasons:
//!
//! - `inventory` uses link-section magic that doesn't reliably
//!   survive wasm32 LLD linking — would break the web target
//!   (§11.12). M14.3a runs in CI on Linux + Mac + Windows + wasm32,
//!   and a flaky linker would gate the whole milestone.
//! - LLM auditability: a `grep` for the canonical name resolves to
//!   the registration call instantly. With `inventory` the
//!   discovery layer is hidden.
//! - HR-17 examples: scripting examples reference canonical
//!   component names. They need to be visible in source.
//!
//! # Stable type_id
//!
//! `std::any::TypeId::of::<T>()` is **not** stable across rustc
//! versions (Rust documents this). We compute the `ComponentTypeId`
//! as `blake3(canonical_name).first_8_bytes()` so cooked prefabs
//! produced by one toolchain still load on another.
//!
//! Canonical names follow `ph2d::<crate>::<TypeName>`:
//! - `ph2d::ecs::Transform`
//! - `ph2d::ecs::Name`
//! - `ph2d::render::Sprite`     (registered in ph2d-render)
//! - `ph2d::script::LuauScript` (registered in ph2d-script)

use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::BTreeMap;

/// Stable, content-addressed identifier for a `Component` type at
/// the cooked-wire level. Same numeric type as
/// `ph2d_asset::ComponentTypeId`.
pub type ComponentTypeId = u64;

/// Compute the canonical `ComponentTypeId` for a `Component` type by
/// hashing its `canonical_name` (e.g. `"ph2d::ecs::Transform"`).
///
/// This function is deterministic across architectures and rustc
/// versions (it depends only on blake3), so the result is safe to
/// embed in a cooked asset and stable across future compiler
/// upgrades. **Never** swap this for `std::any::TypeId::of::<T>()`
/// — that's documented as version-unstable.
pub fn stable_type_id(canonical_name: &str) -> ComponentTypeId {
    let h = blake3::hash(canonical_name.as_bytes());
    let bytes = h.as_bytes();
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&bytes[..8]);
    u64::from_le_bytes(buf)
}

/// Errors raised by the registry on insert / serialize.
#[derive(Debug)]
pub enum RegistryError {
    Decode(postcard::Error),
    Encode(postcard::Error),
    UnknownTypeId(ComponentTypeId),
    EntityMissing(Entity),
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Decode(e) => write!(f, "ComponentRegistry decode error: {e}"),
            Self::Encode(e) => write!(f, "ComponentRegistry encode error: {e}"),
            Self::UnknownTypeId(id) => {
                write!(f, "ComponentRegistry: unknown type_id 0x{id:016x}")
            }
            Self::EntityMissing(e) => {
                write!(f, "ComponentRegistry: entity {:?} not in world", e)
            }
        }
    }
}

impl std::error::Error for RegistryError {}

/// Decode `bytes` and insert the resulting component onto `entity`.
pub type InsertFromBytesFn = fn(&mut World, Entity, &[u8]) -> Result<(), RegistryError>;

/// Serialize the component (if present) on `entity` to postcard bytes.
/// `Ok(None)` = entity exists but has no component of this type.
pub type SerializeFn = fn(&World, Entity) -> Result<Option<Vec<u8>>, RegistryError>;

/// Remove the component (if present) from `entity`. A no-op when the
/// entity is gone or doesn't carry the component — detaching an absent
/// optional component is idempotent (Sprite Inspector v2 W3: toggling a
/// marker / unsetting an optional override).
pub type RemoveFn = fn(&mut World, Entity);

/// Insert `T::default()` onto `entity` — **o elo que faltava** para um UI genérico
/// de *Add Component* (ADR-0166 §4, plano F0).
///
/// A auditoria de 2026-08-21 mediu o buraco: a vtable tinha `insert_from_bytes`,
/// `serialize` e `remove`, e nenhum construtor. Um catálogo que recebe um
/// `ComponentTypeId` **não tem como produzir um valor inicial** sem conhecer o tipo
/// Rust em tempo de compilação — e é por isso que este ponteiro tem de ser capturado
/// no sítio do `register`, que é o único que ainda sabe o que `T` é.
///
/// ⚠️ **Anexar é INERTE**: o componente chega no ponto neutro do próprio tipo, nunca
/// num valor inventado pelo painel. É a lei que a §5 9-Slice já escrevia sozinha
/// (*"um botão que abre uma seção não pode ser uma edição destrutiva disfarçada"*),
/// aqui generalizada — e o desfazer vem de graça, porque a mudança de archetype
/// altera os bytes do snapshot e o diff a apanha.
pub type InsertDefaultFn = fn(&mut World, Entity) -> Result<(), RegistryError>;

/// One registered Component type's vtable: canonical name + id +
/// fn pointers for `(de)serialize through postcard` + type-erased
/// removal + type-erased default construction.
pub struct ComponentTypeEntry {
    pub canonical_name: &'static str,
    pub type_id: ComponentTypeId,
    pub insert_from_bytes: InsertFromBytesFn,
    pub serialize: SerializeFn,
    pub remove: RemoveFn,
    /// `None` quando o tipo **não implementa `Default`** — hoje só a `Sprite`, que
    /// precisa de uma `source` e por isso não é uma escolha do Inspector: ela chega
    /// pelo gesto que cria a imagem. Um `None` aqui é o que impede a paleta de
    /// oferecer algo que ela não consegue construir.
    pub insert_default: Option<InsertDefaultFn>,
    /// O descritor deste tipo ([`ph2d_component_desc`]), resolvido pelo **nome
    /// canónico** no momento do registo.
    ///
    /// ⚠️ **Side-metadata, não contrato.** O catálogo vive numa crate-folha e é
    /// chaveado por string, precisamente para que acrescentar um componente não
    /// obrigue a tocar nos 107 sítios de chamada em 5 crates (DIRETRIZ §1.5.2.1 —
    /// projete foundational novo para ISOLAMENTO). O preço é a deriva silenciosa, e
    /// quem a paga é o censo de dois lados na shell, que tem o registo COMPLETO.
    pub desc: Option<&'static ph2d_component_desc::ComponentDesc>,
    /// ⭐ **O `ComponentId` do bevy para este tipo, NESTE mundo** (ADR-0164 F2).
    ///
    /// ⚠️ **É uma função e não um valor, porque um `ComponentId` é do MUNDO, não do tipo.**
    /// Dois `World` diferentes dão ids diferentes ao mesmo `T` (o id é a ordem de registo
    /// dentro daquele mundo), e o registo é construído **uma vez** e partilhado. Guardar o
    /// número aqui seria guardar a resposta de um mundo e usá-la noutro — e a colisão passaria
    /// muda, porque os dois lados são um `usize`.
    ///
    /// `None` quando o mundo ainda nunca viu este componente (nenhuma entidade o teve): a
    /// varredura da captura incremental simplesmente não o observa, o que é a resposta certa.
    ///
    /// ⚠️ **A F0 deixou isto de fora de propósito** — *"sem consumidor ainda"* — e é a F2 que o
    /// acrescenta **junto com a varredura por archetype que o lê**. Um campo sem leitor é uma
    /// aposta sobre a forma do leitor futuro.
    pub bevy_component_id: fn(&World) -> Option<bevy_ecs::component::ComponentId>,
}

/// Manual registry of component types known to the spawn / save
/// pipeline.
///
/// `BTreeMap` keyed by `ComponentTypeId` (sorted iteration → HR-5
/// determinism friendly). Lookups are O(log N) but `N` is the
/// number of registered types — single digits in practice.
pub struct ComponentRegistry {
    by_id: BTreeMap<ComponentTypeId, ComponentTypeEntry>,
    by_name: BTreeMap<&'static str, ComponentTypeId>,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self {
            by_id: BTreeMap::new(),
            by_name: BTreeMap::new(),
        }
    }

    /// Register a component type under its canonical name. Panics
    /// if the same name is registered twice — caller bug, not a
    /// runtime condition we want to recover from.
    pub fn register<T>(&mut self, canonical_name: &'static str)
    where
        T: Component<Mutability = bevy_ecs::component::Mutable>
            + Serialize
            + DeserializeOwned
            + 'static,
    {
        self.register_inner::<T>(canonical_name, None);
    }

    /// Como [`Self::register`], **mais** o construtor de default — a porta pela qual um
    /// UI genérico anexa um componente que ele não conhece (ADR-0166).
    ///
    /// ⚠️ **Prefira SEMPRE este.** O [`Self::register`] cru fica para os tipos que
    /// genuinamente não têm um ponto neutro (hoje só a `Sprite`, que exige uma
    /// `source`), e um tipo registado sem `insert_default` é um tipo que a paleta do
    /// `+` **não pode oferecer** — o que é a resposta certa quando não há default, e
    /// um buraco silencioso quando há e ninguém o ligou.
    pub fn register_default<T>(&mut self, canonical_name: &'static str)
    where
        T: Component<Mutability = bevy_ecs::component::Mutable>
            + Serialize
            + DeserializeOwned
            + Default
            + 'static,
    {
        self.register_inner::<T>(
            canonical_name,
            Some(|world, entity| {
                let mut e = world
                    .get_entity_mut(entity)
                    .map_err(|_| RegistryError::EntityMissing(entity))?;
                e.insert(T::default());
                Ok(())
            }),
        );
    }

    fn register_inner<T>(
        &mut self,
        canonical_name: &'static str,
        insert_default: Option<InsertDefaultFn>,
    ) where
        T: Component<Mutability = bevy_ecs::component::Mutable>
            + Serialize
            + DeserializeOwned
            + 'static,
    {
        let id = stable_type_id(canonical_name);
        if let Some(prev) = self.by_name.get(canonical_name) {
            panic!(
                "ComponentRegistry: '{canonical_name}' already registered \
                 (existing id 0x{prev:016x})"
            );
        }
        if let Some(prev) = self.by_id.get(&id) {
            panic!(
                "ComponentRegistry: id collision on 0x{id:016x} between \
                 '{}' and '{canonical_name}' — choose a different name",
                prev.canonical_name
            );
        }
        let entry = ComponentTypeEntry {
            canonical_name,
            type_id: id,
            bevy_component_id: |world| world.component_id::<T>(),
            insert_from_bytes: |world, entity, bytes| {
                let v: T = postcard::from_bytes(bytes).map_err(RegistryError::Decode)?;
                let mut e = world
                    .get_entity_mut(entity)
                    .map_err(|_| RegistryError::EntityMissing(entity))?;
                e.insert(v);
                Ok(())
            },
            serialize: |world, entity| {
                let e = world
                    .get_entity(entity)
                    .map_err(|_| RegistryError::EntityMissing(entity))?;
                match e.get::<T>() {
                    Some(v) => {
                        let bytes = postcard::to_allocvec(v).map_err(RegistryError::Encode)?;
                        Ok(Some(bytes))
                    }
                    None => Ok(None),
                }
            },
            remove: |world, entity| {
                if let Ok(mut e) = world.get_entity_mut(entity) {
                    e.remove::<T>();
                }
            },
            insert_default,
            // Resolvido AQUI, uma vez por tipo, e não a cada consulta: o Inspector
            // pergunta por componente presente por quadro, e uma busca por string no
            // caminho de pintura seria um custo de UI que ninguém vê até crescer.
            desc: ph2d_component_desc::desc_for(canonical_name),
        };
        self.by_id.insert(id, entry);
        self.by_name.insert(canonical_name, id);
    }

    pub fn get_by_id(&self, id: ComponentTypeId) -> Option<&ComponentTypeEntry> {
        self.by_id.get(&id)
    }

    pub fn get_by_name(&self, canonical_name: &str) -> Option<&ComponentTypeEntry> {
        self.by_name
            .get(canonical_name)
            .and_then(|id| self.by_id.get(id))
    }

    /// Count of registered types.
    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    /// Iterate registered entries in `type_id`-sorted order. Useful
    /// for cooker tooling that wants to enumerate the supported set.
    pub fn iter(&self) -> impl Iterator<Item = &ComponentTypeEntry> {
        self.by_id.values()
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Register the components owned by `ph2d-ecs` (Transform, Name,
/// Visibility, RootOrder). Other crates contribute via their own
/// `register_*_components` functions, called once at boot from the
/// shell.
pub fn register_ecs_components(reg: &mut ComponentRegistry) {
    reg.register_default::<crate::Transform>("ph2d::ecs::Transform");
    reg.register::<crate::Name>("ph2d::ecs::Name");
    reg.register_default::<crate::Visibility>("ph2d::ecs::Visibility");
    reg.register_default::<crate::RootOrder>("ph2d::ecs::RootOrder");
    // ⛔ **O `StableId` NÃO é registado, e a ausência é a decisão.** Ele viaja no campo
    // `EntitySnapshotRow::id` — uma fonte só. Registá-lo poria a identidade também num
    // `ComponentBlob`, e a cópia profunda da F4 (`extract_component_snapshot` +
    // `insert_from_bytes`, que copiam blobs VERBATIM) daria à cópia a identidade do
    // original. O ADR-0164 §2.7 exige *"remapeado em toda cópia de blobs"*; mantê-lo fora
    // do registo torna esse erro impossível em vez de o deixar por lembrar.
    // A ordem entre irmãos como DADO (F1) — o gémeo do `RootOrder`. Sem registo ela não
    // entra no snapshot, e reordenar continuaria a não ser desfazível (classe BUGS #15).
    reg.register_default::<crate::SiblingOrder>("ph2d::ecs::SiblingOrder");
    // Sprite Inspector v2 W3 — sorting / visibility / sampling
    // components (spec §02). Optional: serialized only when present, so
    // legacy scenes are byte-unchanged.
    reg.register_default::<crate::SortingLayer>("ph2d::ecs::SortingLayer");
    reg.register_default::<crate::OrderInLayer>("ph2d::ecs::OrderInLayer");
    reg.register_default::<crate::ZIndexOverride>("ph2d::ecs::ZIndexOverride");
    reg.register_default::<crate::ZAsRelative>("ph2d::ecs::ZAsRelative");
    reg.register_default::<crate::YSort>("ph2d::ecs::YSort");
    reg.register_default::<crate::SortingGroup>("ph2d::ecs::SortingGroup");
    reg.register_default::<crate::ShowBehindParent>("ph2d::ecs::ShowBehindParent");
    reg.register_default::<crate::TopLevel>("ph2d::ecs::TopLevel");
    // ⭐ **A raiz de um MESTRE** (ADR-0164 / F4) — autoria, logo viaja no arquivo.
    // ⚠️ O `MasterPiece` NÃO entra aqui, e a ausência é a decisão: ele é DERIVADO
    // (`assign_master_pieces`), e um valor derivado no arquivo envenena o undo.
    reg.register_default::<crate::MasterRoot>("ph2d::ecs::MasterRoot");
    reg.register_default::<crate::ClipChildren>("ph2d::ecs::ClipChildren");
    reg.register_default::<crate::MaskInteraction>("ph2d::ecs::MaskInteraction");
    reg.register_default::<crate::Mask2D>("ph2d::ecs::Mask2D");
    reg.register_default::<crate::TextureFilter>("ph2d::ecs::TextureFilter");
    reg.register_default::<crate::TextureRepeat>("ph2d::ecs::TextureRepeat");
    reg.register_default::<crate::VisibilityLayer>("ph2d::ecs::VisibilityLayer");
    reg.register_default::<crate::OnScreenEnabler>("ph2d::ecs::OnScreenEnabler");
    reg.register_default::<crate::UvTransform>("ph2d::ecs::UvTransform");
    reg.register_default::<crate::BlendMode>("ph2d::ecs::BlendMode");
    // A sprite como FONTE DE LUZ (plano `docs/Sprite_projeto/18` W8). Opcional: sem o componente o
    // quadro é byte-idêntico, e um projeto antigo carrega sem ele — mas quem o autorou tem de o
    // reencontrar depois de gravar, e sem esta linha o `world_to_snapshot` descartava-o em silêncio.
    reg.register_default::<crate::SpriteEmissive>("ph2d::ecs::SpriteEmissive");
    // ADR-0164 F1 passo 6 / ADR-0166 — os três grupos que saíram do `Sprite` v4. Cada um é
    // OPCIONAL, e a ausência é o default benigno que o campo tinha (grelha de 1 célula ·
    // textura inteira · cantos brancos), então um projeto que nunca os tocou continua
    // byte-idêntico. ⚠️ A PRESENÇA do `SpriteRegion` é o antigo `region_enabled`.
    reg.register_default::<crate::SpriteGrid>("ph2d::ecs::SpriteGrid");
    reg.register_default::<crate::SpriteRegion>("ph2d::ecs::SpriteRegion");
    reg.register_default::<crate::SpriteCornerTint>("ph2d::ecs::SpriteCornerTint");
    // Trava e group-lock: markers que o Hierarchy edita e que o save/undo precisa
    // preservar (sem eles, `world_to_snapshot` os descartava em silêncio).
    reg.register_default::<crate::Locked>("ph2d::ecs::Locked");
    reg.register_default::<crate::GroupedChildren>("ph2d::ecs::GroupedChildren");
    // ADR-0110: a referência que faz de um path vetorial uma entidade. Sem ela um
    // save do mundo perderia o vínculo path↔entidade e o load duplicaria as formas.
    reg.register_default::<crate::VecPathRef>("ph2d::ecs::VecPathRef");
    // A identidade ESTÁVEL do documento do Painter (camadas + pixels + relevo). Sem ela
    // um save/load não teria como devolver a um sprite o documento que era dele — os bits
    // da entidade são id de alocação e morrem no restore —, e a pintura voltaria como um
    // bake achatado, sem camadas e sem espessura.
    reg.register_default::<crate::PaintedDoc>("ph2d::ecs::PaintedDoc");
    // ADR-0150 (W8.7): a identidade ESTÁVEL dos canais assados de uma malha (`base` + `form`).
    // Mesmo mecanismo do `PaintedDoc` e mesma consequência de esquecê-la — mas com um agravante
    // próprio: os canais existem justamente para o objeto sobreviver ao módulo 3D sair do build, e
    // sem esta identidade eles não sobreviveriam nem ao arquivo ser reaberto com ele DENTRO.
    reg.register_default::<crate::BakedForm>("ph2d::ecs::BakedForm");
    // ADR-0114: idem para um objeto Flip (animação quadro-a-quadro). Sem ela o
    // save perderia o vínculo objeto↔entidade e o load duplicaria os objetos Flip.
    reg.register_default::<crate::FlipObjectRef>("ph2d::ecs::FlipObjectRef");
    // Live Shapes: os parâmetros de uma forma paramétrica viva (a geometria é
    // derivada deles). Sem registrar, um save/undo perderia a "forma-ness" e o texto
    // não saberia se re-cozinhar / converter em curvas.
    reg.register::<crate::VecShape>("ph2d::ecs::VecShape");
    // Sem este registro, o conector seria DESCARTADO pelo snapshot — o undo e o save o
    // perderiam em silêncio (foi o que aconteceu com Locked/GroupedChildren/VecPathRef).
    reg.register::<crate::VecConnector>("ph2d::ecs::VecConnector");
    // ADR-0128: o Blend Object vivo. Mesma razão do conector — sem este registro o snapshot o
    // DESCARTA, e o undo/save perderiam o vínculo blend↔fontes em silêncio.
    reg.register::<crate::VecBlend>("ph2d::ecs::VecBlend");
    // O Offset vivo: a forma engorda sem que a curva autorada mude. Mesma razão de todos os
    // irmãos — sem o registro o snapshot o DESCARTA, e um Ctrl+Z (ou um save) devolveria a
    // forma sem o offset, em silêncio, com a curva certa por baixo.
    reg.register::<crate::VecOffset>("ph2d::ecs::VecOffset");
    reg.register_default::<crate::VecStrokeProfile>("ph2d::ecs::VecStrokeProfile");
    // A SIMETRIA viva: o eixo autorado de uma forma. Mesma razão de todos os irmãos — sem o
    // registro o snapshot a DESCARTA, e um Ctrl+Z (ou reabrir o projeto) devolveria a forma com
    // metade do desenho, sem que nada dissesse por quê: as cópias são derivadas, então o que tem
    // de sobreviver é a RELAÇÃO, e é este componente.
    reg.register_default::<crate::VecSymmetry>("ph2d::ecs::VecSymmetry");
    // A LÂMINA: qual caminho é linha de corte. Mesma razão de todos os irmãos, e mais afiada —
    // sem o registro, reabrir o projeto devolveria a linha como um caminho SEM fill e SEM stroke:
    // invisível na tela, inerte, e fora do alcance do botão que existe para a apagar.
    reg.register_default::<crate::VecCutPath>("ph2d::ecs::VecCutPath");
    // A BOOLEANA viva: com que operação os filhos de um grupo se combinam. Mesma razão de todos
    // os irmãos, e aqui a perda seria a mais visível de todas — sem o registro, um Ctrl+Z (ou
    // reabrir o projeto) devolveria os operandos SOLTOS, empilhados uns sobre os outros, e a
    // forma combinada simplesmente não existiria mais: ela é desenho derivado, e o que tem de
    // sobreviver é a RELAÇÃO.
    reg.register::<crate::VecBoolGroup>("ph2d::ecs::VecBoolGroup");
    // A PELE: que widget do catálogo esta forma veste (plano UI/UX W6.2). Mesma razão de todos
    // os irmãos — sem o registro, reabrir o projeto devolveria a forma como DESENHO cru, e o
    // artista teria de re-vestir cada controle da tela que acabou de compor.
    reg.register::<crate::VecWidget>("ph2d::ecs::VecWidget");
    reg.register_default::<crate::VecWidgetBind>("ph2d::ecs::VecWidgetBind");
    reg.register_default::<crate::VecWidgetValue>("ph2d::ecs::VecWidgetValue");
    // O ÍCONE escolhido de um botão de ícone (plano UI/UX W8b, §6.2). Sem o registro, reabrir o
    // projeto devolveria o botão desenhando a FORMA — a escolha do artista evaporaria e o glifo
    // trocaria sozinho, que é o modo de falha mais enganoso: nada some, e o desenho está errado.
    reg.register::<crate::VecWidgetIcon>("ph2d::ecs::VecWidgetIcon");
    // A MOLDURA: que esta entidade CONTÉM, e se ela esconde o transbordo. Sem o registro, um
    // Ctrl+Z (ou reabrir o projeto) devolveria a moldura como um retângulo comum — a arte
    // continuaria toda lá, e o recorte simplesmente teria evaporado. É o modo de falha mais
    // enganoso da lista: nada some, tudo aparece, e o desenho está errado.
    reg.register::<crate::VecFrame>("ph2d::ecs::VecFrame");
    // Os BINDINGS de token. Sem o registro, reabrir o projeto devolveria a forma com o LITERAL
    // que estava debaixo do token — a arte apareceria inteira, com a cor de antes de o artista
    // bindar, e nada indicaria que uma referência tinha evaporado.
    reg.register_default::<crate::VecBindings>("ph2d::ecs::VecBindings");
    // O AUTO LAYOUT (plano UI/UX W2, ADR-0153): a regra no PAI e o comportamento no FILHO. Dois
    // componentes porque são duas perguntas — *"esta moldura empilha?"* × *"este filho cresce?"* —
    // e porque o que eles descrevem é uma REGRA: a posição que dela sai é derivada por frame e
    // nunca entra no `Transform` autorado (é o que impede cada redimensionamento de virar um passo
    // de undo).
    reg.register_default::<crate::VecLayout>("ph2d::ecs::VecLayout");
    reg.register_default::<crate::VecLayoutItem>("ph2d::ecs::VecLayoutItem");
    reg.register_default::<crate::VecLayoutSize>("ph2d::ecs::VecLayoutSize");
    reg.register_default::<crate::VecLayoutAbsolute>("ph2d::ecs::VecLayoutAbsolute");
    // AS ÂNCORAS (plano UI/UX W3): a outra metade da responsividade — a regra do filho que NÃO
    // está num fluxo. Sem o registro, um Ctrl+Z (ou reabrir o projeto) devolveria a arte inteira,
    // com todas as formas no lugar certo, e a REGRA evaporada: a moldura voltaria a redimensionar
    // sem que nada a acompanhasse, e nada na tela diria porquê. E há aqui um agravante que o
    // `VecLayout` não tem — o componente carrega a RÉGUA (a moldura contra a qual a regra foi
    // autorada), que é insubstituível: re-armar depois de a perder captura a moldura de AGORA, e
    // o redimensionamento que o artista já tinha feito fica assado no lugar errado.
    reg.register::<crate::VecAnchors>("ph2d::ecs::VecAnchors");
    // RESIZE BOX (plano UI/UX W3b): o override de *"a alca reescreve a caixa deste objeto?"*.
    // Sem o registro, um Ctrl+Z (ou reabrir o projeto) devolveria a arte inteira com a moldura
    // que o artista mandou ESCALAR de volta a redimensionar -- e o gesto seguinte faria a coisa
    // errada em silencio. O componente so' existe quando ele discorda do default derivado, entao
    // perde-lo e' perder exatamente a decisao que ele tomou.
    reg.register::<crate::VecResizeBox>("ph2d::ecs::VecResizeBox");
    // O vínculo TEXTO -> caminho-guia. Mesma razão de todos os irmãos, e mais forte: sem o
    // registro, um Ctrl+Z (ou reabrir o projeto) devolveria o texto DESLIGADO do caminho, reto,
    // no meio da cena -- e o caminho continuaria lá, parecendo certo.
    reg.register::<crate::VecTextPath>("ph2d::ecs::VecTextPath");
    // O vínculo MOTIVO -> caminho-guia (Pattern Along Path, plano 23). Mesma razão do texto: sem o
    // registro, um Ctrl+Z (ou reabrir) devolveria o motivo SOLTO do caminho, uma cópia só, no meio
    // da cena -- e o caminho continuaria lá. As cópias são desenho derivado, então o que o snapshot
    // tem de guardar é a RELAÇÃO, e é este componente.
    reg.register_default::<crate::VecPatternPath>("ph2d::ecs::VecPatternPath");
    // A ORIENTAÇÃO do motivo sobre a guia (o par opcional do vínculo acima). Componente separado, e
    // não um campo no `VecPatternPath`, porque o blob é postcard POSICIONAL: apender campo bumparia
    // o `PROJECT_SCHEMA`, e um bump RECUSA todo projeto já salvo. Ausência = sem rotação, então
    // documento antigo carrega inalterado. Sem o registro, o ângulo autorado morreria no primeiro
    // Ctrl+Z com as cópias a voltarem deitadas -- e o vínculo, esse, sobreviveria: o pattern
    // pareceria certo e estaria errado.
    reg.register_default::<crate::VecPatternRotation>("ph2d::ecs::VecPatternRotation");
    // O Contour: N anéis concêntricos com progressão de cor. Mesma razão de todos os irmãos — sem
    // o registro, o snapshot o DESCARTA e um Ctrl+Z devolveria a forma sozinha, com os anéis
    // sumidos e a cor-alvo perdida.
    reg.register_default::<crate::VecContour>("ph2d::ecs::VecContour");
    // O vínculo do RÓTULO com o objeto que ele nomeia. Mesma razão, e mais forte: a pose do
    // rótulo é DERIVADA dele — sem o componente no snapshot, o undo devolveria um texto solto
    // no meio da forma, com o offset do usuário perdido.
    // O Morph vivo (o irmão animável do Blend). Mesma razão: sem o registro, o snapshot o
    // DESCARTA e o undo/save perderiam o vínculo morph↔fontes — e o `t` autorado junto.
    reg.register::<crate::VecMorph>("ph2d::ecs::VecMorph");
    // ⭐ O GRAFO da maquina de estados do Morph (plano 32). Mesma razao de todos acima: sem o
    // registo, o snapshot o DESCARTA e o undo/save perderiam as setas que o artista desenhou.
    // ⚠️ E ele viaja SEM mexer no `PROJECT_SCHEMA`: o `ComponentBlob` e' chaveado por
    // `blake3(nome canonico)`, entao um ficheiro antigo simplesmente NAO TEM este blob e a
    // entidade volta sem maquina -- que e' a leitura correcta de "ninguem desenhou seta nenhuma".
    reg.register::<crate::VecMorphMachine>("ph2d::ecs::VecMorphMachine");
    // ADR-0129: o Envelope Object vivo. Mesma razão de todos acima — sem o registro, o snapshot o
    // DESCARTA e o undo/save perderiam a gaiola E a fonte autorada em silêncio (e a fonte, aqui,
    // é insubstituível: o recook já sobrescreveu o path da cena com a cozida).
    reg.register::<crate::VecEnvelope>("ph2d::ecs::VecEnvelope");
    reg.register::<crate::VecLabel>("ph2d::ecs::VecLabel");
    // O FX raster de uma forma (Blur/Glow/Drop Shadow, plano 24). Mesma razão de todos os irmãos —
    // sem o registro, o snapshot o DESCARTA e um Ctrl+Z (ou reabrir) devolveria a forma NUA, sem a
    // sombra/brilho, com a curva certa por baixo. O FX é DESENHO derivado; o snapshot guarda a
    // RELAÇÃO, e é este componente.
    reg.register_default::<crate::VecFilter>("ph2d::ecs::VecFilter");
    // OS COMPONENTES (plano UI/UX W5): o mestre e a instância. Mesma razão de todos os irmãos, e
    // aqui com dois modos de falha distintos — sem o registro do MARCADOR, um Ctrl+Z devolveria a
    // arte com as instâncias a apontar para um caminho que já não se declara mestre (todas órfãs
    // de uma vez); sem o registro da INSTÂNCIA, o que se perde é o vínculo E os overrides
    // autorados, e o que sobra é um caminho vazio no lugar onde havia uma cópia.
    reg.register_default::<crate::VecComponentMain>("ph2d::ecs::VecComponentMain");
    reg.register_default::<crate::VecInstance>("ph2d::ecs::VecInstance");
    // O NOME DURÁVEL dos pixels próprios de um sprite (plano 17 §3). Mesma razão de todos os
    // irmãos, e o modo de falha aqui já estava a acontecer em produção: `SpriteSource::Individual`
    // guarda um id de alocação da GPU, que recomeça em `1` a cada processo — sem esta identidade no
    // snapshot, reabrir o projeto devolve o sprite INVISÍVEL (o `bind_group` não resolve) ou a
    // exibir os pixels de OUTRO sprite que ficou com aquele id no restore.
    reg.register::<crate::SpritePixels>("ph2d::ecs::SpritePixels");
    // DE QUE REGIAO DE QUE FOLHA um sprite hand-packed e' (plano Sprite 17 §6). Mesma
    // razao de todos os irmaos: o `Sprite.source` guarda um `texture_id` de runtime, que
    // morre com o processo -- sem esta componente no snapshot, reabrir o projeto (ou um
    // Ctrl+Z) devolveria o sprite sem saber de que folha ele era, e a regiao autorada
    // sobreviveria apenas como um retangulo solto que ninguem sabe re-derivar.
    reg.register::<crate::SpriteSheetRef>("ph2d::ecs::SpriteSheetRef");
    // ESTE RETANGULO E' UMA FOLHA (plano Sprite 17 §7) -- a "imagem virtual" onde as pecas sao
    // montadas. Gemeo do `VecFrame`: a entidade e' um retangulo vivo e o componente so' diz o que
    // ela FAZ com os filhos. Sem o registro, o snapshot o DESCARTA e reabrir o projeto devolveria
    // um retangulo qualquer com sprites soltos por baixo -- a folha deixaria de ser uma folha, em
    // silencio, e o proximo bake nao teria o que assar.
    reg.register::<crate::SpriteSheetFrame>("ph2d::ecs::SpriteSheetFrame");
    // A AUTORIA DE 9-SLICE (spec Sprite 03 §3.5) -- bordas, modo de desenho e os oito modos
    // por-regiao. Declarada em 2026-05 e construida em 2026-08-21; ate' la' `git grep -c
    // SliceNine` dava ZERO. Sem o registro, anexar 9-slice a uma caixa de dialogo e gravar o
    // projeto devolveria, ao reabrir, um sprite esticado: as bordas nao sao re-derivaveis de
    // nada -- sao uma medida que o artista tirou da imagem.
    reg.register_default::<crate::SliceNine>("ph2d::ecs::SliceNine");
    // AS ANCORAS NOMEADAS (ADR-0072, spec Sprite 07) -- socket, slice e regiao 9-slice num tipo
    // so'. Sem o registro, um artista que marca a boca da arma e grava o projeto reabre-o sem
    // ela: uma ancora nao e' re-derivavel de nada, e' uma medida que alguem tirou.
    reg.register_default::<crate::NamedAnchorList>("ph2d::ecs::NamedAnchorList");
    // QUEM MONTA numa dessas ancoras (ADR-0072 §2.6, 2026-08-22) -- o CONSUMIDOR que faltava.
    // Sem o registro, reabrir o projeto devolve a espada como filha COMUM do personagem: ela
    // continua la', no mesmo sitio, e deixou de andar com a mao. E' o modo de falha caro --
    // nada some, nada avisa, e o defeito so' aparece quando o braco se mexe.
    reg.register_default::<crate::AnchorMount>("ph2d::ecs::AnchorMount");
    // QUANDO as ancoras de uma entidade se desenham (Enio, 2026-08-23): sem selecao, e em
    // runtime. Sem o registro, marcar «manter visiveis» e gravar o projeto devolve, ao reabrir,
    // uma cena onde os pontos voltaram a aparecer so' com o dono selecionado -- e o artista
    // remarcaria a caixa todos os dias sem perceber que ela nunca guardou.
    reg.register_default::<crate::AnchorVisibility>("ph2d::ecs::AnchorVisibility");
    // A §11 ANIMATION (spec Sprite 08) -- a biblioteca de tags e o estado de reproducao. Sem
    // o registro, o artista autora `idle`/`walk`/`attack`, grava, reabre, e a sprite volta a
    // ser uma grelha parada: as tags nao sao re-derivaveis de nada, sao intervalos que alguem
    // escolheu. ⚠️ O `SpriteAnimator` grava TAMBEM o estado (frame, ciclo, acumulador), e e
    // isso que faz o replay reproduzir a mesma animacao -- a razao de ele ser SimComponent.
    reg.register_default::<crate::SpriteAnimations>("ph2d::ecs::SpriteAnimations");
    reg.register_default::<crate::SpriteAnimator>("ph2d::ecs::SpriteAnimator");
    // O RECORTE, que deixou de ser um campo da moldura para valer em qualquer forma FECHADA
    // (2026-08-21). Sem o registro, o modo de falha é o mesmo da moldura e igualmente enganoso:
    // um Ctrl+Z devolveria a forma inteira, com todos os filhos no lugar, e o recorte
    // simplesmente teria evaporado — nada some, tudo aparece, e o desenho está errado.
    reg.register::<crate::VecClipContent>("ph2d::ecs::VecClipContent");
    // O VERBO DE UMA FORMA dentro dessa booleana (2026-08-22). Sem o registro, reabrir o
    // projeto devolveria toda a receita achatada no `op` do GRUPO: as formas continuariam lá,
    // a combinação continuaria a existir, e ela desenharia OUTRA coisa — a perda mais
    // traiçoeira desta familia, porque nao ha nada em falta na tela a denunciá-la.
    reg.register::<crate::VecBoolOp>("ph2d::ecs::VecBoolOp");
}

#[cfg(test)]
#[path = "registry_tests.rs"]
mod tests;
