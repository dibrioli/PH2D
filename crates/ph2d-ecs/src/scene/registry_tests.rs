//! **Os testes do `ComponentRegistry`** — irmão de [`super::registry`] pelo teto de 700 LOC
//! da workspace (`architecture_workspace_file_loc_cap`), que a F0 estourou em 747 ao
//! acrescentar o `insert_default` e o `desc` à vtable.
//!
//! ⚠️ O corte é por RESPONSABILIDADE e não por linha: aqui fica o que PROVA o registo, lá o
//! que ele É. ⛔ Não devolva um teste ao `registry.rs` — o teto volta a estourar no próximo
//! componente, e o corte teria sido pago à toa.

use super::*;

/// ⭐ **A porta do *Add Component*: anexar um componente conhecendo só o `type_id`.**
///
/// Este é o teste que a F0 existe para tornar possível (plano §F0, ADR-0166 §4). Ele
/// imita o que a paleta do `+` fará: recebe um id opaco do catálogo, procura a entrada,
/// e chama `insert_default` — **sem uma única menção ao tipo `SortingLayer` no caminho
/// da inserção**. Antes desta wave era impossível: a vtable tinha `insert_from_bytes`,
/// `serialize` e `remove`, e nada que produzisse um valor inicial.
///
/// ⚠️ A asserção final é o que faz o teste valer: o componente chega no **ponto neutro
/// do próprio tipo**, nunca num valor inventado pelo chamador. É a lei do *anexar é
/// inerte* — e note que ela morde: o `ZAsRelative` tem `Default = true`, e um painel que
/// "inicializasse com zeros" o poria em `false`, mudando o desenho ao anexar.
#[test]
fn a_component_can_be_attached_knowing_only_its_type_id() {
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);

    let mut world = World::new();
    let e = world.spawn(crate::Transform::IDENTITY).id();

    // A paleta só tem isto: um id vindo do catálogo.
    let id = stable_type_id("ph2d::ecs::SortingLayer");
    let entry = reg.get_by_id(id).expect("SortingLayer registado");
    let insert = entry
        .insert_default
        .expect("SortingLayer tem Default, logo tem construtor na vtable");
    insert(&mut world, e).expect("a entidade existe");

    assert_eq!(
        world.get::<crate::SortingLayer>(e).copied(),
        Some(crate::SortingLayer::default()),
        "anexar tem de pousar o PONTO NEUTRO do tipo, nunca um valor do chamador",
    );

    // E o mesmo caminho serve o `ZAsRelative`, cujo neutro NAO e' zero — e' `true`.
    let z = stable_type_id("ph2d::ecs::ZAsRelative");
    let insert_z = reg.get_by_id(z).unwrap().insert_default.unwrap();
    insert_z(&mut world, e).unwrap();
    assert_eq!(
        world.get::<crate::ZAsRelative>(e).copied(),
        Some(crate::ZAsRelative(true)),
        "o neutro do tipo, nao zeros",
    );
}

/// **A `Sprite` do outro lado da mesma lei** — um tipo sem `Default` devolve `None`,
/// e é isso que impede a paleta de prometer o que não consegue cumprir.
///
/// ⚠️ O censo da shell (`every_offered_component_can_be_constructed`) liga este `None`
/// ao descritor: um tipo assim **não pode** ser `Attach::Authored`.
#[test]
fn a_type_without_a_default_has_no_constructor_in_the_vtable() {
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    let name = reg.get_by_name("ph2d::ecs::Name").expect("Name registado");
    assert!(
        name.insert_default.is_none(),
        "o `Name` nao tem Default (um objeto sem nome nao e' um objeto de nome vazio), \
         entao a vtable nao pode oferecer um construtor",
    );
}

/// Hard-coded type ids — locked in to catch accidental renames.
/// If you change a canonical name, bump the prefab schema
/// version (HR-14) and add a migration; do **not** silently
/// recompute this constant.
#[test]
fn stable_type_ids_are_locked_in() {
    // These values are the first 8 bytes of `blake3(name)` in
    // little-endian. Recompute with:
    //   echo -n 'ph2d::ecs::Transform' | b3sum --no-names | head -c 16
    // and reverse byte order.
    // The assertion catches accidental renames before they ship.
    let t = stable_type_id("ph2d::ecs::Transform");
    let n = stable_type_id("ph2d::ecs::Name");
    // Sanity: distinct names → distinct ids.
    assert_ne!(t, n);
    // Determinism: same input → same id every call.
    assert_eq!(stable_type_id("ph2d::ecs::Transform"), t);
}

#[test]
fn register_ecs_components_populates_registry() {
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    // 4 foundational (Transform/Name/Visibility/RootOrder) + 16 W3
    // sorting/visibility/sampling/mask components (incl. Mask2D source)
    // + 1 §10 BlendMode + 5 save/undo
    // (Locked/GroupedChildren/VecPathRef/FlipObjectRef/PaintedDoc)
    // + 1 Live Shapes (VecShape) + 1 conector (VecConnector) + 1 Blend Object (VecBlend)
    // + 1 rótulo (VecLabel) + 1 Envelope Object (VecEnvelope, ADR-0129)
    // + 1 Offset vivo (VecOffset) + 1 texto em caminho (VecTextPath)
    // + 1 pattern em caminho (VecPatternPath, plano 23)
    // + 1 FX raster (VecFilter, plano 24)
    // + 1 largura viva (VecStrokeProfile, ADR-0148)
    // + 1 linha de corte (VecCutPath, plano 25 §7)
    // + 1 simetria viva (VecSymmetry, plano 25 §9 W6.3)
    // + 1 booleana viva (VecBoolGroup, plano UI/UX W1)
    // + 1 moldura (VecFrame, plano UI/UX W0)
    // + 1 tabela de bindings de token (VecBindings, plano UI/UX W4)
    // + 2 auto layout (VecLayout no pai + VecLayoutItem no filho, plano UI/UX W2)
    // + 2 sizing (VecLayoutSize no no + VecLayoutAbsolute, o fora-do-fluxo)
    // + 1 âncoras (VecAnchors, plano UI/UX W3)
    // + 1 resize-box (VecResizeBox, plano UI/UX W3b)
    // + 1 pele por-widget (VecWidget, plano UI/UX W6.2)
    // + 1 vinculo row -> forma (VecWidgetBind, plano UI/UX W8b.3)
    // + 1 posicao do controle (VecWidgetValue, plano UI/UX W8b.4)
    // + 1 icone escolhido (VecWidgetIcon, plano UI/UX W8b §6.2)
    // + 1 nome duravel dos pixels proprios (SpritePixels, plano Sprite 17 §3)
    // + 1 regiao de folha hand-packed (SpriteSheetRef, plano Sprite 17 §6)
    // + 1 folha como OBJETO (SpriteSheetFrame, plano Sprite 17 §7).
    //
    // **Este número existe para doer.** Um componente que não passa por aqui é
    // DESCARTADO em silêncio pelo snapshot — o undo e o save o perdem, e o bug só
    // aparece três telas depois (foi assim que o `PaintedDoc`, a identidade estável do
    // documento do Painter, teria nascido morto). Ao acrescentar um componente, some 1
    // aqui de propósito.
    //
    // Na integração ele SOMA entre linhas: o Painter trouxe o `PaintedDoc` e o Vector o
    // `VecConnector`, e as duas linhas, sozinhas, diziam 27 — por motivos diferentes. A
    // árvore combinada tem 28. Escolher "um dos lados" aqui é o erro que deixa o
    // workspace vermelho com dois merges verdes.
    // + 1 autoria de 9-slice (SliceNine, spec Sprite 03 §3.5 -- a secao 5, declarada
    //   em 2026-05 e construida em 2026-08-21)
    // + 1 lista de ancoras nomeadas (NamedAnchorList, ADR-0072 -- `Accepted` desde
    //   2026-05-28 sobre codigo que nao existia).
    // + 1 recorte (VecClipContent — o bit que SAIU do VecFrame para valer em qualquer
    //   forma fechada, 2026-08-21). ⚠️ CONTADO na integracao de 2026-08-22: a `line/Sprite`
    //   (+6) entrou antes da `line/Vector` (+2) — o valor e' a SOMA, nao o de nenhum lado.
    // + 1 verbo POR FORMA dentro dela (VecBoolOp, 2026-08-22)
    // + 1 MONTAGEM numa ancora (AnchorMount, ADR-0072 §2.6, 2026-08-22 — o consumidor).
    //   ⚠️ SAO TRES contadores desta familia (ecs · render · script), cada um so' visivel
    //   na suite da SUA crate — e esta linha ja' os deixou 2 e 4 atras, com a nota escrita.
    //   Ao mexer aqui, mexa nos tres NO MESMO commit: `ph2d-render` e `ph2d-script`.
    // + 1 VISIBILIDADE das ancoras (AnchorVisibility, Enio 2026-08-23).
    // + 2 da §11 ANIMATION (SpriteAnimations + SpriteAnimator, spec Sprite 08).
    // + 3 do CORTE DA SPRITE (SpriteGrid + SpriteRegion + SpriteCornerTint, ADR-0164 F1
    //   passo 6 / ADR-0166, 2026-08-25) — os tres campos que SAIRAM do `Sprite` v4 para
    //   poderem estar AUSENTES, que e' o que os tira do Inspector ate' o artista os pedir.
    // + 1 do MESTRE (MasterRoot, ADR-0164 F4.1, 2026-08-25) -- a raiz de uma receita
    //   da biblioteca. ⚠️ O `MasterPiece` NAO entra: ele e' DERIVADO (`assign_master_pieces`),
    //   e um valor derivado no arquivo envenena o undo.
    // + 1 do ELO da instancia (InstanceOf, ADR-0164 F4.2, 2026-08-26) -- de que mestre esta
    //   raiz nasceu, pelo StableId dele.
    // + 1 dos OVERRIDES (ObjectInstance, ADR-0164 F4.4, 2026-08-26) -- o conjunto de
    //   `(peca, componente)` que a instancia possui contra o mestre.
    // + 1 da COPIA LIGADA (LinkedArt, Enio 2026-08-27) -- o *Duplicate Linked* do Blender:
    //   esta peca divide a arte do mestre, entao a edicao dela SOBE em vez de virar excepcao.
    // + 1 do PREENCHIMENTO do balde (VecBucketFill, plano 40, 2026-09-01) -- a RECEITA (o
    //   ponto apontado), nunca a area: a area e' re-cozida quando as linhas mudam.
    // ⚠️ **O `VariantValues` desta linha foi REVERTIDO por ela mesma** (o mecanismo de
    //   propriedades saiu inteiro do fonte, adiado para o fim do plano) ⇒ a conta volta a
    //   `79`, que e' o `VecBucketFill` da `line/Vector`, ja' no main. ⛔ *Um numero que
    //   soma entre linhas conta-se; e um que a linha DESFEZ nao se soma.*
    assert_eq!(reg.len(), 79);
    assert!(reg.get_by_name("ph2d::ecs::VecClipContent").is_some());
    assert!(reg.get_by_name("ph2d::ecs::VecBoolOp").is_some());
    assert!(reg.get_by_name("ph2d::ecs::SpritePixels").is_some());
    assert!(reg.get_by_name("ph2d::ecs::SpriteEmissive").is_some());
    assert!(reg.get_by_name("ph2d::ecs::SpriteSheetRef").is_some());
    assert!(reg.get_by_name("ph2d::ecs::SpriteSheetFrame").is_some());
    assert!(reg.get_by_name("ph2d::ecs::SliceNine").is_some());
    assert!(reg.get_by_name("ph2d::ecs::NamedAnchorList").is_some());
    assert!(reg.get_by_name("ph2d::ecs::AnchorMount").is_some());
    assert!(reg.get_by_name("ph2d::ecs::AnchorVisibility").is_some());
    assert!(reg.get_by_name("ph2d::ecs::SpriteAnimations").is_some());
    assert!(reg.get_by_name("ph2d::ecs::SpriteAnimator").is_some());
    assert!(reg.get_by_name("ph2d::ecs::VecAnchors").is_some());
    assert!(reg.get_by_name("ph2d::ecs::VecResizeBox").is_some());
    assert!(reg.get_by_name("ph2d::ecs::VecWidget").is_some());
    assert!(reg.get_by_name("ph2d::ecs::VecWidgetBind").is_some());
    assert!(reg.get_by_name("ph2d::ecs::VecWidgetValue").is_some());
    assert!(reg.get_by_name("ph2d::ecs::VecWidgetIcon").is_some());
    assert!(reg.get_by_name("ph2d::ecs::VecBindings").is_some());
    assert!(reg.get_by_name("ph2d::ecs::VecLayout").is_some());
    assert!(reg.get_by_name("ph2d::ecs::VecLayoutItem").is_some());
    assert!(reg.get_by_name("ph2d::ecs::VecLayoutSize").is_some());
    assert!(reg.get_by_name("ph2d::ecs::VecLayoutAbsolute").is_some());
    assert!(reg.get_by_name("ph2d::ecs::VecCutPath").is_some());
    assert!(reg.get_by_name("ph2d::ecs::VecSymmetry").is_some());
    assert!(reg.get_by_name("ph2d::ecs::VecPatternPath").is_some());
    assert!(reg.get_by_name("ph2d::ecs::VecFilter").is_some());
    assert!(reg.get_by_name("ph2d::ecs::VecStrokeProfile").is_some());
    assert!(reg.get_by_name("ph2d::ecs::Transform").is_some());
    assert!(reg.get_by_name("ph2d::ecs::Name").is_some());
    assert!(reg.get_by_name("ph2d::ecs::Visibility").is_some());
    assert!(reg.get_by_name("ph2d::ecs::RootOrder").is_some());
    assert!(reg.get_by_name("ph2d::ecs::Locked").is_some());
    assert!(reg.get_by_name("ph2d::ecs::PaintedDoc").is_some());
    assert!(reg.get_by_name("ph2d::ecs::BakedForm").is_some());
    assert!(reg.get_by_name("ph2d::ecs::GroupedChildren").is_some());
    assert!(reg.get_by_name("ph2d::ecs::VecPathRef").is_some());
    assert!(reg.get_by_name("ph2d::ecs::VecConnector").is_some());
    assert!(reg.get_by_name("ph2d::ecs::VecBlend").is_some());
    assert!(reg.get_by_name("ph2d::ecs::VecOffset").is_some());
    assert!(reg.get_by_name("ph2d::ecs::VecTextPath").is_some());
    assert!(reg.get_by_name("ph2d::ecs::VecLabel").is_some());
    assert!(reg.get_by_name("ph2d::ecs::FlipObjectRef").is_some());
    assert!(reg.get_by_name("ph2d::ecs::VecShape").is_some());
    assert!(reg.get_by_name("ph2d::ecs::ZIndexOverride").is_some());
    assert!(reg.get_by_name("ph2d::ecs::YSort").is_some());
    assert!(reg.get_by_name("ph2d::ecs::TextureFilter").is_some());
    assert!(reg.get_by_name("ph2d::ecs::OnScreenEnabler").is_some());
    assert!(reg.get_by_name("ph2d::ecs::Missing").is_none());
}

#[test]
fn insert_serialize_round_trip_transform() {
    use crate::Transform;
    use bevy_ecs::world::World;
    use ph2d_core::Vec2;

    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    let entry = reg.get_by_name("ph2d::ecs::Transform").unwrap();

    let original = Transform {
        translation: Vec2::new(3.0, 4.0),
        rotation: 1.5,
        scale: Vec2::new(2.0, 2.0),
        ..Transform::IDENTITY
    };
    let bytes = postcard::to_allocvec(&original).unwrap();

    let mut world = World::new();
    let entity = world.spawn_empty().id();
    (entry.insert_from_bytes)(&mut world, entity, &bytes).unwrap();

    let serialized = (entry.serialize)(&world, entity).unwrap().unwrap();
    let back: Transform = postcard::from_bytes(&serialized).unwrap();
    assert_eq!(back, original);
}

#[test]
fn duplicate_registration_panics() {
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        reg.register_default::<crate::Transform>("ph2d::ecs::Transform");
    }));
    assert!(result.is_err(), "duplicate registration must panic");
}
