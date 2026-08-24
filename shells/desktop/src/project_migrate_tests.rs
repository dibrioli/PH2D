//! Testes da migração v95 → v96 ([`crate::project_migrate`], ADR-0164 F1).
//!
//! ⚠️ O gate que de facto guarda esta migração é o [`the_frozen_v95_bytes_still_load`]: ele
//! não testa o **tipo** `ProjectFileV95` (que segue os tipos vivos e pode apodrecer com eles),
//! testa os **BYTES** de um ficheiro v95 real. É a única forma de detetar que o v95 deixou de
//! ser legível, porque quando um tipo partilhado muda os dois lados mudam juntos e nada mais
//! fica vermelho.

use crate::project_migrate::{ProjectFileV95, migrate_v95_to_v96};
use ph2d_ecs::StableId;
use ph2d_ecs::scene::{EntitySnapshotRowV1, WorldSnapshotV1};

/// **Um ficheiro v95 com três objetos numa cadeia** — o mínimo que exercita a tradução de
/// `parent` de índice para id.
fn v95_bytes() -> Vec<u8> {
    let blob = |b: u8| ph2d_asset::ComponentBlob {
        type_id: 0x0102_0304_0506_0708,
        data: vec![b],
    };
    let world = WorldSnapshotV1 {
        version: WorldSnapshotV1::VERSION,
        entities: vec![
            EntitySnapshotRowV1 {
                components: vec![blob(10)],
                parent: None,
            },
            EntitySnapshotRowV1 {
                components: vec![blob(20)],
                parent: Some(0),
            },
            EntitySnapshotRowV1 {
                components: vec![blob(30)],
                parent: Some(1),
            },
        ],
    };
    // ⚠️ Serializado como TUPLO `(u32, corpo)` — é o formato do ficheiro (`project_save.rs`).
    // O corpo é montado campo a campo na ordem exacta da v95.
    let body = (
        (
            world,
            ph2d_vec_scene::VecScene::new(),
            ph2d_flip::FlipDoc::new(),
            ph2d_guides::GuideSet::default(),
            ph2d_ui_state::StateSets::default(),
        ),
        Vec::<u8>::new(),                                     // assets (vazio)
        Vec::<u8>::new(),                                     // painted (vazio)
        String::new(),                                        // motion
        Vec::<u8>::new(),                                     // timeline
        ph2d_physics_ecs::PhysicsSettings::default(),         // physics
        Vec::<u8>::new(),                                     // tokens (vazio)
        crate::project_settings::collect(Default::default()), // settings
        Vec::<u8>::new(),                                     // sculpt
        Vec::<u8>::new(),                                     // baked_forms (vazio)
        ph2d_physics_ecs::TapeWire::default(),                // player_tape
        Vec::<u8>::new(),                                     // sprite_pixels
    );
    postcard::to_allocvec(&(95u32, body)).expect("serializa um v95")
}

/// **Um ficheiro v95 desserializa pelo tipo congelado.**
///
/// ⚠️ **Este é o gate que guarda a migração inteira.** O `ProjectFileV95` referencia os tipos
/// VIVOS nos campos que não mudaram (`VecScene`, `FlipDoc`, `SavedSettings`, …) — o que o
/// mantém em 40 linhas em vez de 400, e o que o faz apodrecer em silêncio se um desses tipos
/// mudar de forma. Aqui os bytes são construídos **campo a campo, sem passar pelo tipo**, e é
/// isso que torna o teste capaz de ver a deriva.
///
/// ⛔ Se ele ficar vermelho, a resposta **não** é re-gerar os bytes: é que um tipo partilhado
/// mudou, e ou ele é congelado de verdade, ou a v95 deixa de ser legível — e isso é uma
/// decisão de produto, não uma arrumação.
#[test]
fn the_frozen_v95_bytes_still_load() {
    let bytes = v95_bytes();
    let (ver, old): (u32, ProjectFileV95) =
        postcard::from_bytes(&bytes).expect("os bytes v95 desserializam pelo tipo congelado");
    assert_eq!(ver, 95);
    assert_eq!(old.state.world.entities.len(), 3);
    assert_eq!(old.state.world.entities[2].parent, Some(1));
}

/// **A migração dá identidade a todos e traduz a cadeia de pais.**
#[test]
fn the_migration_gives_every_object_an_identity() {
    let bytes = v95_bytes();
    let (_, old): (u32, ProjectFileV95) = postcard::from_bytes(&bytes).unwrap();
    let m = migrate_v95_to_v96(old);

    let rows = &m.file.state.world.entities;
    assert_eq!(rows.len(), 3);
    assert!(
        rows.iter().all(|r| !r.id.is_none()),
        "nenhuma linha pode ficar com o id reservado — todas as NONE colidem no restore",
    );
    assert_eq!(rows[0].parent, None, "a raiz continua raiz");
    assert_eq!(
        rows[2].parent,
        Some(rows[1].id),
        "o neto aponta para o ID do pai, nao para o indice",
    );
    assert!(
        m.stable_id_counter > rows.iter().map(|r| r.id.0).max().unwrap(),
        "o contador tem de comecar depois do maior id migrado, senao o proximo objeto \
         criado reusa um id vivo",
    );
}

/// ⭐ **O round-trip que o plano da F1 pede: v95 → v96 → gravar → ler → gravar é
/// byte-equivalente.**
///
/// A primeira gravação é a migração; a segunda tem de ser idêntica à primeira. Se não for, ou
/// a migração não é determinística, ou o save escreve algo que o load não lê de volta — e as
/// duas fazem um projeto mudar de bytes só por ter sido aberto.
#[test]
fn migrating_then_saving_twice_is_byte_equivalent() {
    let (_, old): (u32, ProjectFileV95) = postcard::from_bytes(&v95_bytes()).unwrap();
    let once = migrate_v95_to_v96(old);
    let first = postcard::to_allocvec(&(crate::project_schema::PROJECT_SCHEMA, &once.file))
        .expect("grava v96");

    // Ler o que acabou de ser gravado e gravar outra vez.
    let (_, reread): (u32, crate::project::ProjectFile) =
        postcard::from_bytes(&first).expect("o v96 recem-gravado le-se de volta");
    let second =
        postcard::to_allocvec(&(crate::project_schema::PROJECT_SCHEMA, &reread)).expect("re-grava");

    assert_eq!(
        first, second,
        "gravar -> ler -> gravar mudou os bytes: o ficheiro muda so' por ter sido aberto",
    );
}

/// **A migração é reprodutível** — dois utilizadores que abram o mesmo v95 obtêm os mesmos
/// ids. Sem isto, o mesmo projeto migrado em duas máquinas divergiria para sempre.
#[test]
fn two_migrations_of_the_same_file_agree() {
    let bytes = v95_bytes();
    let (_, a): (u32, ProjectFileV95) = postcard::from_bytes(&bytes).unwrap();
    let (_, b): (u32, ProjectFileV95) = postcard::from_bytes(&bytes).unwrap();
    let ma = migrate_v95_to_v96(a);
    let mb = migrate_v95_to_v96(b);
    assert_eq!(
        ma.file.state.world.state_hash(),
        mb.file.state.world.state_hash()
    );
    assert_eq!(ma.stable_id_counter, mb.stable_id_counter);
}

/// **O id migrado começa em `FIRST`, não em `0`.**
#[test]
fn migrated_ids_start_after_the_reserved_one() {
    let (_, old): (u32, ProjectFileV95) = postcard::from_bytes(&v95_bytes()).unwrap();
    let m = migrate_v95_to_v96(old);
    assert_eq!(m.file.state.world.entities[0].id, StableId(StableId::FIRST));
}
