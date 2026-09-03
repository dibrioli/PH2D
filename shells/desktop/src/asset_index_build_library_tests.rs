//! ⭐⭐ **Os gates da LÁPIDE** — cortados do [`super::tests`] pelo tecto de 600 LOC (HR-18), e o
//! corte é por responsabilidade: aqui vive *o que a biblioteca GUARDA contra o que ela MOSTRA*.
//!
//! ⚠️ A lei que estes gates defendem é a que torna o `Remove from Library` desfazível: **esquecer
//! marca, nunca apaga**. Ela não tinha instrumento nenhum até 2026-08-30 — um `forget` que
//! marcasse *e* apagasse passava na suíte inteira, e o gesto voltava a ser irreversível.

use super::*;
use ph2d_ecs::Transform;
use std::collections::BTreeMap;

/// Sem átlas — o irmão do que vive no [`super::tests`].
fn no_atlas() -> BTreeMap<u32, ph2d_asset::AssetId> {
    BTreeMap::new()
}

/// Sem taxonomia.
fn no_catalogs() -> ph2d_asset_index::CatalogTree {
    ph2d_asset_index::CatalogTree::new()
}

/// ⭐⭐⭐ **UMA TEXTURA QUE NINGUÉM USA PODE SAIR DA BIBLIOTECA — e uma usada VOLTA.**
///
/// ⛔ Report do Enio (2026-08-30, 2.ª ronda): *«uma sprite que foi deletada do canvas não consegui
/// deletar do painel»*. A biblioteca lembra por conteúdo e nunca subtrai sozinha (a cura de um
/// report anterior, e continua certa) — e não tinha porta de saída nenhuma.
///
/// ⚠️ **As DUAS metades num gate só, e é isso que impede a cura de virar «esquece sempre»:** com a
/// entidade ainda no mundo, o `build` seguinte **repõe** a entrada. *Esquecer é dizer «ninguém a
/// usa», e o mundo é quem confirma.*
///
/// **Mutação que deve sangrar:** fazer o `forget` não remover nada (a 1.ª metade), ou o `build`
/// deixar de repor o que tem entidade (a 2.ª).
#[test]
fn a_texture_nobody_uses_can_leave_the_library_and_a_used_one_comes_back() {
    let db = AssetDb::new();
    let tex = db.insert_image_rgba8(2, 2, vec![3u8; 16]);
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("OnCanvas"),
            StableId(1),
            SpritePixels(tex),
        ))
        .id();
    let mut cache = CardArt::new();
    let mut lib = TextureLibrary::default();
    // ⚠️ **A régua é o que o `build` DEVOLVE**, e não o que a biblioteca guarda (corrigida em
    // 2026-08-30). Desde que uma lápide cede a um utilizador vivo, a pergunta *«quantas se veem?»*
    // só tem resposta com o conjunto dos vivos na mão — e quem o calcula é o `build`. *Uma régua
    // que não vê o contexto da lei mede outra coisa.*
    let textures = |ix: &ph2d_asset_index::AssetIndex| {
        ix.entries()
            .iter()
            .filter(|e| matches!(e.key, AssetRef::Texture { .. }))
            .count()
    };
    let ix = build(
        &mut sim,
        &db,
        &no_atlas(),
        &no_catalogs(),
        &mut cache,
        &mut lib,
        100.0,
    );
    assert_eq!(textures(&ix), 1, "a textura entrou pela sprite");

    // (a) COM a entidade viva, esquecer não pega — o utilizador vivo ganha à lápide.
    lib.forget(tex);
    let ix = build(
        &mut sim,
        &db,
        &no_atlas(),
        &no_catalogs(),
        &mut cache,
        &mut lib,
        100.0,
    );
    assert_eq!(
        textures(&ix),
        1,
        "uma textura que uma entidade ainda referencia VOLTA — senão o gesto mentiria"
    );

    // (b) Sem a entidade, ela sai e FICA fora.
    sim.world_mut().despawn(e);
    let ix = build(
        &mut sim,
        &db,
        &no_atlas(),
        &no_catalogs(),
        &mut cache,
        &mut lib,
        100.0,
    );
    assert_eq!(textures(&ix), 0, "ninguém a usa: ela tem de poder sair");
}

/// ⭐⭐⭐ **ESQUECER MARCA, NUNCA APAGA** — a lei-título do undo da biblioteca, que não tinha gate.
///
/// ⛔⛔ Achado da auditoria de 2026-08-30: um `forget` que marcasse **e** apagasse a entrada
/// sobrevivia à suíte inteira. O gate vizinho
/// (`a_texture_nobody_uses_can_leave_the_library_and_a_used_one_comes_back`) passava igual com o
/// `forget` ANTIGO — ele mede o que se VÊ, e a lápide e o `remove` escondem exactamente o mesmo.
///
/// ⚠️ **A régua é a diferença entre as duas contagens:** `len()` responde *«quantas mostro»* e
/// `stored_len()` *«quantas posso devolver»*. É a segunda que prova que o Ctrl+Z tem o que
/// restaurar — sem entrada guardada, levantar a lápide devolve um cartão **vazio**.
///
/// **Mutação que deve sangrar:** `fn forget(&mut self, id) { self.forgotten.insert(id); self.entries.remove(&id); }`
#[test]
fn forgetting_a_texture_keeps_the_entry_so_undo_has_something_to_bring_back() {
    let mut lib = super::TextureLibrary::default();
    let tex = ph2d_asset::AssetId::from_digest([5; 32]);
    lib.remember(tex, entry_for(tex));
    assert_eq!(lib.len(), 1);
    assert_eq!(lib.stored_len(), 1);

    lib.forget(tex);
    assert_eq!(lib.len(), 0, "a lápide tem de a esconder");
    assert_eq!(
        lib.stored_len(),
        1,
        "a entrada foi APAGADA — o undo levanta a lápide e não há nada para voltar"
    );
}

/// ⛔⛔ **Uma imagem que o mundo usa AGORA ganha à lápide — e ninguém edita o documento.**
///
/// ⚠️ A 1.ª versão levantava a lápide dentro do `remember`, que corre **por quadro**: abrir o
/// painel Assets depois de re-importar a imagem apagava a marca **sem gesto nenhum**, o quadro
/// seguinte registava um passo espúrio, e um Ctrl+Z a repô-la era desfeito no quadro a seguir —
/// *o Ctrl+Z não pegava e queimava um passo*.
///
/// ⇒ hoje a regra é a mesma que a recusa do verbo já usava: **quem tem utilizador nunca está
/// escondido**, e isso decide-se na leitura, não escrevendo no documento.
///
/// **Mutação que deve sangrar:** tirar o `|| live.contains(id)` do `entries`.
#[test]
fn a_tombstoned_texture_the_world_uses_now_is_visible_without_editing_the_document() {
    let mut lib = super::TextureLibrary::default();
    let tex = ph2d_asset::AssetId::from_digest([6; 32]);
    lib.remember(tex, entry_for(tex));
    lib.forget(tex);

    let none = std::collections::BTreeSet::new();
    assert_eq!(
        lib.entries(&none).count(),
        0,
        "sem utilizador ela fica fora"
    );

    let live: std::collections::BTreeSet<_> = [tex].into_iter().collect();
    assert_eq!(
        lib.entries(&live).count(),
        1,
        "com utilizador vivo ela tem de aparecer — senão o artista vê um objecto sem asset"
    );
    // ⭐ **E a lápide CONTINUA lá**: nada no caminho de leitura editou o documento.
    assert_eq!(
        lib.forgotten.len(),
        1,
        "ler a biblioteca apagou a lápide — o laço de render voltou a editar o documento"
    );
}

/// Uma entrada mínima, para os gates da lápide. ⚠️ Pelo construtor da crate, e não por um literal:
/// um campo novo no `AssetEntry` não pode partir um gate sobre lápides.
fn entry_for(id: ph2d_asset::AssetId) -> ph2d_asset_index::AssetEntry {
    ph2d_asset_index::AssetEntry::new(
        AssetRef::Texture {
            asset: *id.as_bytes(),
        },
        String::from("fixtura"),
    )
}

/// ⛔⛔⛔ **UM PREFAB DE SPRITES DE ÁTLAS TEM COR E RETRATO** — report do Enio, 2026-09-01, com foto:
/// *«objeto pai é o objeto vazio com 3 texturas como filhas»*, e o cartão saiu **cinzento**: nem
/// retrato, nem cor.
///
/// # A causa, e por que ela é a MESMA de duas vezes anteriores
///
/// «Uma peça aponta para os pixels» tem **duas** formas, e o `SpritePixels` é a da **minoria** — o
/// caminho normal de todo import e de todo canvas novo é o **átlas**. O
/// [`super::texture_of`] existe exactamente para as unificar, e o doc dele já dizia *«UMA porta,
/// dois leitores»*. O retrato e a cor dominante fizeram-se **terceiro e quarto leitores, com a
/// metade errada**.
///
/// ⚠️ **A fixtura CARREGA O FENÓMENO**: as peças são de átlas e **não** têm `SpritePixels`. Com o
/// carimbo raro ela ficava verde sobre o defeito que a foto mostra.
///
/// **Mutações que devem sangrar:** `largest_piece_texture` voltar a ler `SpritePixels` directo · o
/// resolvedor do `compose` fazer o mesmo.
#[test]
fn a_prefab_of_atlas_sprites_gets_a_colour_and_a_portrait() {
    let mut sim = ph2d_ecs::SimWorld::new();
    // O pai é um objecto VAZIO — sem `Sprite`, como no report.
    let root = sim
        .world_mut()
        .spawn((ph2d_ecs::Transform::IDENTITY, ph2d_ecs::MasterRoot))
        .id();
    for (i, at) in [[-1.0, 0.0], [1.0, 0.0], [0.0, 1.0]]
        .into_iter()
        .enumerate()
    {
        sim.world_mut().spawn((
            ph2d_ecs::Transform::from_translation(ph2d_core::Vec2::new(at[0], at[1])),
            ph2d_render::Sprite::atlas(i as u32, [1.0, 1.0], [1.0; 4]),
            ph2d_ecs::ChildOf(root),
        ));
    }
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_master_pieces(sim.world_mut());

    // Um `AssetDb` com três imagens, e o mapa de átlas a apontar-lhes — é o que o import faz.
    let db = ph2d_asset::AssetDb::new();
    let mut atlas: std::collections::BTreeMap<u32, ph2d_asset::AssetId> =
        std::collections::BTreeMap::new();
    for i in 0..3u32 {
        let rgba = vec![u8::try_from(60 + i * 60).unwrap_or(255); 8 * 8 * 4];
        let id = db.insert_image_rgba8(8, 8, rgba);
        atlas.insert(i, id);
    }

    let mut art = super::CardArt::new();
    let mut lib = super::TextureLibrary::default();
    let index = super::build(
        &mut sim,
        &db,
        &atlas,
        &ph2d_asset_index::CatalogTree::default(),
        &mut art,
        &mut lib,
        100.0,
    );
    let prefab = index
        .entries()
        .iter()
        .find(|e| matches!(e.key, ph2d_asset_index::AssetRef::Component { .. }))
        .expect("o prefab tem de estar no indice");
    assert!(
        prefab.thumb.is_some(),
        "o prefab de sprites de atlas ficou SEM RETRATO — o cartao cinzento do report"
    );
    assert_ne!(
        prefab.swatch,
        ph2d_asset_index::AssetEntry::new(prefab.key, String::new()).swatch,
        "o prefab ficou com a cor NEUTRA do construtor — nem a cor dominante foi achada"
    );
    // ⛔⛔ **A TERCEIRA pergunta, que este gate não fazia** (report do Enio, 2026-09-02: *«não
    // conseguiu listar os ítens»*). Ele media a COR e o RETRATO — as duas curadas pela porta — e
    // deixava as `deps` de fora, que continuavam a perguntar só por `SpritePixels`. ⇒ o
    // *«Show what it uses»* respondia *«This asset uses nothing else»* sobre um prefab de três
    // texturas. ⚠️ E como o sentido `Owners` é DERIVADO por inversão, a mesma linha calava os dois.
    assert_eq!(
        prefab.deps.len(),
        3,
        "o prefab de sprites de atlas nao declara as texturas que usa: {:?}",
        prefab.deps
    );
    // ⭐⭐ **E o sentido INVERSO na mesma corrida.** Ele é derivado por inversão, então uma `deps`
    // vazia calava os dois — mas *derivado* não é *medido*: sem esta metade, o gate afirma a
    // metade que o report nomeou e deixa a outra por conta de um raciocínio.
    let first = prefab.deps[0];
    let key = prefab.key;
    let owners = index.owners(&first);
    assert_eq!(
        owners.iter().map(|e| e.key).collect::<Vec<_>>(),
        vec![key],
        "a textura nao sabe que prefab a usa"
    );
}
