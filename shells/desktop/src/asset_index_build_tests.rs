//! Os gates da JUNÇÃO ([`super`]) — irmão por ASSUNTO, e pelo tecto de LOC do shell (HR-18).
//!
//! ⚠️ **Este corte foi imposto por um gate que esteve VERMELHO sem ninguém ver**
//! (`shell_files_respect_hr18_loc_cap`, 2026-08-30): ele vive em `shells/desktop/tests/` e o
//! portão de fecho desta linha corria `cargo test --bins`, que **não toca** naquele diretório. É a
//! 5.ª ocorrência registada desta família — *uma suíte com filtro não é a suíte*.

use super::*;
use ph2d_asset_index::AssetKind;
use ph2d_ecs::{ChildOf, Transform};

/// Um mundo com uma receita de duas peças, a de baixo com pixels próprios.
fn world_with_one_component(db: &AssetDb) -> (SimWorld, AssetId) {
    let mut sim = SimWorld::new();
    let pixels = vec![0u8; 4 * 4 * 4];
    let id = db.insert_image_rgba8(4, 4, pixels);
    let root = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Ragdoll"),
            MasterRoot,
            StableId(1),
        ))
        .id();
    sim.world_mut().spawn((
        Transform::IDENTITY,
        Name::new("Torso"),
        StableId(2),
        ChildOf(root),
        SpritePixels(id),
    ));
    (sim, id)
}

/// ⭐ **A junção**: uma travessia devolve as duas famílias, e a textura da peça aparece como
/// asset por direito próprio — não escondida dentro do componente.
#[test]
fn one_walk_returns_both_families() {
    let db = AssetDb::new();
    let (mut sim, _) = world_with_one_component(&db);
    let mut cache = CardArt::new();
    let mut lib = TextureLibrary::default();
    let index = build(&mut sim, &db, &mut cache, &mut lib);
    let counts = index.counts();
    assert_eq!(counts.get(&AssetKind::Component), Some(&1));
    assert_eq!(counts.get(&AssetKind::Texture), Some(&1));
}

/// A dependência é guardada **só num sentido**, e o índice inverte-a.
#[test]
fn the_component_declares_the_texture_and_the_texture_names_its_owner() {
    let db = AssetDb::new();
    let (mut sim, id) = world_with_one_component(&db);
    let mut cache = CardArt::new();
    let mut lib = TextureLibrary::default();
    let index = build(&mut sim, &db, &mut cache, &mut lib);
    let tex = AssetRef::Texture {
        asset: *id.as_bytes(),
    };
    let owners: Vec<&str> = index.owners(&tex).iter().map(|e| e.name.as_str()).collect();
    assert_eq!(owners, vec!["Ragdoll"]);
}

/// ⛔⛔ **A lente 1 da auditoria, executável:** apagar a receita do mundo tem de a tirar do
/// índice. É isto que a reconstrução compra e que a mutação por evento perderia.
#[test]
fn deleting_the_master_removes_it_from_the_next_build() {
    let db = AssetDb::new();
    let (mut sim, _) = world_with_one_component(&db);
    let mut cache = CardArt::new();
    let mut lib = TextureLibrary::default();
    assert_eq!(
        build(&mut sim, &db, &mut cache, &mut lib)
            .counts()
            .get(&AssetKind::Component),
        Some(&1)
    );
    let root = {
        let mut q = sim
            .world_mut()
            .query_filtered::<Entity, bevy_ecs::prelude::With<MasterRoot>>();
        q.iter(sim.world()).next().unwrap()
    };
    sim.world_mut().entity_mut(root).remove::<MasterRoot>();
    let after = build(&mut sim, &db, &mut cache, &mut lib);
    assert_eq!(after.counts().get(&AssetKind::Component), None);
}

/// ⚠️ **A média é ponderada por ALFA.** Uma sprite recortada — 1 pixel vermelho opaco em 15
/// transparentes — tem de dar VERMELHO. A média crua daria quase preto, que é a cor do nada.
#[test]
fn the_swatch_of_a_cut_out_sprite_is_the_colour_of_the_drawing_not_of_the_hole() {
    let db = AssetDb::new();
    let mut pixels = vec![0u8; 4 * 4 * 4];
    pixels[0..4].copy_from_slice(&[255, 0, 0, 255]);
    let id = db.insert_image_rgba8(4, 4, pixels);
    let mut cache = CardArt::new();
    let rgba = swatch_for(&db, id, &mut cache).expect("uma imagem rgba8 tem cor");
    assert!(rgba[0] > 200, "vermelho esperado, veio {rgba:?}");
    assert!(
        rgba[1] < 40 && rgba[2] < 40,
        "sem outros canais, veio {rgba:?}"
    );
}

/// A cor calcula-se **uma vez por conteúdo** — a cache é chaveada pelo `AssetId`, e é isso que
/// a torna reutilizável entre quadros e entre entidades.
#[test]
fn the_swatch_is_computed_once_per_content() {
    let db = AssetDb::new();
    let id = db.insert_image_rgba8(2, 2, vec![9u8; 16]);
    let mut cache = CardArt::new();
    let _ = swatch_for(&db, id, &mut cache);
    assert_eq!(cache.swatches.len(), 1);
    let _ = swatch_for(&db, id, &mut cache);
    assert_eq!(cache.swatches.len(), 1, "a segunda leitura nao recalcula");
}

/// ⭐⭐ **E a MINIATURA também** — a metade que o A6 acrescentou, e a que de facto obriga a
/// memória: a cor tem tecto de amostras, a miniatura lê a imagem inteira.
///
/// ⚠️ **A barra não é «a cache tem 1 entrada», é «o `Arc` é o MESMO»** — é isso que o painel
/// compara em `O(1)` para não reconstruir a textura de GPU. Um cache que devolvesse bytes
/// iguais num `Arc` novo passaria na contagem e faria o `vello` reenviar cada cartão ao atlas
/// todo o quadro, sem um único gate vermelho.
#[test]
fn the_thumbnail_is_reduced_once_and_hands_back_the_same_arc() {
    let db = AssetDb::new();
    let id = db.insert_image_rgba8(2, 2, vec![9u8; 16]);
    let mut cache = CardArt::new();
    let mut budget = THUMB_BUDGET_PX;
    let a = thumb_for(&db, id, &mut cache, &mut budget).expect("a miniatura sai de 2x2");
    assert_eq!(cache.thumbs.len(), 1);
    let b =
        thumb_for(&db, id, &mut cache, &mut budget).expect("a segunda leitura acerta na memória");
    assert_eq!(cache.thumbs.len(), 1, "a segunda leitura nao recalcula");
    assert!(
        std::sync::Arc::ptr_eq(&a.rgba, &b.rgba),
        "o mesmo conteúdo tem de devolver o MESMO ponteiro"
    );
}

/// ⚠️ **A ordem NÃO é a de iteração do ECS.** Ela sai do `StableId`, e por isso é a mesma em
/// dois builds do mesmo mundo — o cartão debaixo do dedo não muda entre quadros.
#[test]
fn two_builds_of_the_same_world_agree_entry_for_entry() {
    let db = AssetDb::new();
    let (mut sim, _) = world_with_one_component(&db);
    let mut cache = CardArt::new();
    let mut lib = TextureLibrary::default();
    let a: Vec<AssetRef> = build(&mut sim, &db, &mut cache, &mut lib)
        .entries()
        .iter()
        .map(|e| e.key)
        .collect();
    let b: Vec<AssetRef> = build(&mut sim, &db, &mut cache, &mut lib)
        .entries()
        .iter()
        .map(|e| e.key)
        .collect();
    assert_eq!(a, b);
}

/// ⛔⛔ **O átlas do ARRANQUE não é a biblioteca do artista.** Report do Enio, 2026-08-30:
/// *«o painel de assets apareceu e está com várias sprites que ninguém colocou lá»*.
///
/// **Mutação que deve sangrar:** voltar a percorrer `db.tracked_paths()` como fonte de
/// entradas — que é literalmente o estado em que o painel estava.
#[test]
fn textures_the_boot_loaded_but_nobody_placed_are_not_assets() {
    let db = AssetDb::new();
    // O boot carrega 16 texturas para o `AssetDb`; nenhuma entidade as referencia.
    for i in 0..16u8 {
        let _ = db.insert_image_rgba8(4, 4, vec![i; 64]);
    }
    let mut sim = SimWorld::new();
    let mut cache = CardArt::new();
    let mut lib = TextureLibrary::default();
    let index = build(&mut sim, &db, &mut cache, &mut lib);
    assert!(
        index.is_empty(),
        "o painel mostrou {} assets que ninguem colocou la'",
        index.len()
    );
}

/// ⭐⭐ **A biblioteca LEMBRA.** Report do Enio: *«ao deletar o objeto do canvas, o do painel
/// assets foi deletado»*.
///
/// **Mutação que deve sangrar:** reconstruir as texturas do mundo a cada quadro, sem memória.
#[test]
fn deleting_the_sprite_does_not_delete_the_texture_from_the_library() {
    let db = AssetDb::new();
    let (mut sim, id) = world_with_one_component(&db);
    let mut cache = CardArt::new();
    let mut lib = TextureLibrary::default();
    assert_eq!(
        build(&mut sim, &db, &mut cache, &mut lib)
            .counts()
            .get(&AssetKind::Texture),
        Some(&1)
    );

    // O artista apaga a sprite da cena.
    let victim = {
        let mut q = sim.world_mut().query::<(Entity, &SpritePixels)>();
        q.iter(sim.world()).map(|(e, _)| e).next().unwrap()
    };
    sim.world_mut().despawn(victim);

    let after = build(&mut sim, &db, &mut cache, &mut lib);
    assert_eq!(
        after.counts().get(&AssetKind::Texture),
        Some(&1),
        "a textura saiu da biblioteca quando o objecto foi apagado"
    );
    assert!(
        after
            .get(&AssetRef::Texture {
                asset: *id.as_bytes()
            })
            .is_some(),
        "e' a MESMA textura que tem de ficar, pelo endereco de conteudo"
    );
    assert_eq!(lib.len(), 1);
}

/// ⛔ **A visibilidade NÃO alcança a biblioteca.** Report do Enio: *«mudei o hide no objeto da
/// cena e o objeto do painel foi modificado»*. Esconder é vista; um asset é conteúdo.
#[test]
fn hiding_an_object_changes_nothing_in_the_library() {
    let db = AssetDb::new();
    let (mut sim, _) = world_with_one_component(&db);
    let mut cache = CardArt::new();
    let mut lib = TextureLibrary::default();
    let before: Vec<(String, String, [u8; 4])> = build(&mut sim, &db, &mut cache, &mut lib)
        .entries()
        .iter()
        .map(|e| (e.name.clone(), e.detail.clone(), e.swatch))
        .collect();

    // Esconde a raiz — a mesma marca que o olho da Hierarquia escreve.
    let root = {
        let mut q = sim
            .world_mut()
            .query_filtered::<Entity, bevy_ecs::prelude::With<MasterRoot>>();
        q.iter(sim.world()).next().unwrap()
    };
    sim.world_mut()
        .entity_mut(root)
        .insert(ph2d_ecs::Visibility { hidden: true });

    let after: Vec<(String, String, [u8; 4])> = build(&mut sim, &db, &mut cache, &mut lib)
        .entries()
        .iter()
        .map(|e| (e.name.clone(), e.detail.clone(), e.swatch))
        .collect();
    assert_eq!(before, after, "esconder mudou o que o painel mostra");
}

/// ⭐⭐ **Apagar a CÓPIA não apaga a RECEITA.** Report do Enio, 2026-08-30: *«o objeto foi até o
/// painel corretamente mas ao deletar o objeto do canvas, o do painel assets foi deletado»*.
///
/// ⚠️ Este gate mede o ÍNDICE, que é a metade que eu possuo: dada uma receita viva no mundo, o
/// painel mostra-a. Se ele ficar verde e o report persistir, o defeito está no **verbo de
/// apagar** (ele leva a receita junto), e não aqui — e é essa a próxima pergunta.
#[test]
fn deleting_the_copy_leaves_the_recipe_in_the_panel() {
    let db = AssetDb::new();
    let (mut sim, _) = world_with_one_component(&db);
    // A cópia que o *Make Component* deixa no lugar: uma raiz própria, SEM `MasterRoot`.
    let copy = sim
        .world_mut()
        .spawn((
            ph2d_ecs::Transform::IDENTITY,
            Name::new("Ragdoll"),
            StableId(50),
        ))
        .id();
    let mut cache = CardArt::new();
    let mut lib = TextureLibrary::default();
    assert_eq!(
        build(&mut sim, &db, &mut cache, &mut lib)
            .counts()
            .get(&AssetKind::Component),
        Some(&1)
    );
    sim.world_mut().despawn(copy);
    assert_eq!(
        build(&mut sim, &db, &mut cache, &mut lib)
            .counts()
            .get(&AssetKind::Component),
        Some(&1),
        "apagar a copia tirou a receita do painel"
    );
}

/// ⭐⭐ **O orçamento do quadro TRAVA a rajada, e o acerto na memória NÃO o gasta.**
///
/// ⛔ A auditoria de 2026-08-30 apanhou a tabela do custo sem tecto: `12,079 ms` para uma textura
/// de 4096², num laço sem orçamento, na thread que desenha — dez texturas grandes ⇒ ~120 ms de
/// congelamento **ao abrir o painel**, porque é aí que a biblioteca inteira é reduzida de uma vez.
///
/// **Mutação que deve sangrar:** apagar o `if *budget == 0 { return None; }`.
#[test]
fn the_frame_budget_stops_the_burst_and_a_cache_hit_does_not_spend_it() {
    let db = AssetDb::new();
    let big = 1200u32; // 1,44 M px — três destas passam o orçamento de 4 M
    let a = db.insert_image_rgba8(big, big, vec![7u8; (big as usize) * (big as usize) * 4]);
    let b = db.insert_image_rgba8(big, big, vec![8u8; (big as usize) * (big as usize) * 4]);
    let c = db.insert_image_rgba8(big, big, vec![9u8; (big as usize) * (big as usize) * 4]);
    let d = db.insert_image_rgba8(big, big, vec![10u8; (big as usize) * (big as usize) * 4]);
    let mut cache = CardArt::new();
    let mut budget = THUMB_BUDGET_PX;
    let got: Vec<bool> = [a, b, c, d]
        .iter()
        .map(|id| thumb_for(&db, *id, &mut cache, &mut budget).is_some())
        .collect();
    assert_eq!(
        got,
        vec![true, true, true, false],
        "o orçamento tem de deixar passar o que cabe e travar o resto — {got:?}"
    );
    // ⚠️ A metade que impede a cura de partir o caso comum: com o orçamento a zero, uma miniatura
    // JÁ REDUZIDA continua a sair. Sem isto o painel deixaria de mostrar o que já tem.
    assert!(
        thumb_for(&db, a, &mut cache, &mut budget).is_some(),
        "um acerto na memória não pode ser travado pelo orçamento"
    );
}
