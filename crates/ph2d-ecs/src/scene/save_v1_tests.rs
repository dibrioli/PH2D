//! Testes da migração v1 → v2 do [`WorldSnapshot`].

use super::*;

/// Uma árvore v1 de 3 linhas: raiz (0) → meio (1) → folha (2), como o
/// `canonicalize` do shell as deixava (ordenadas por conteúdo).
fn tree_v1() -> WorldSnapshotV1 {
    let blob = |b: u8| ComponentBlob {
        type_id: 0x1234_5678_9abc_def0,
        data: vec![b],
    };
    WorldSnapshotV1 {
        version: WorldSnapshotV1::VERSION,
        entities: vec![
            EntitySnapshotRowV1 {
                components: vec![blob(1)],
                parent: None,
            },
            EntitySnapshotRowV1 {
                components: vec![blob(2)],
                parent: Some(0),
            },
            EntitySnapshotRowV1 {
                components: vec![blob(3)],
                parent: Some(1),
            },
        ],
    }
}

/// **A migração dá id a toda linha e traduz o `parent` de índice para id.**
#[test]
fn the_migration_turns_indices_into_ids() {
    let new = migrate_v1_to_v2(&tree_v1());
    assert_eq!(new.version, WorldSnapshot::VERSION);
    let ids: Vec<u64> = new.entities.iter().map(|r| r.id.0).collect();
    assert_eq!(
        ids,
        vec![1, 2, 3],
        "os ids saem na ordem das linhas, a partir de 1"
    );
    assert_eq!(new.entities[0].parent, None, "a raiz continua raiz");
    assert_eq!(
        new.entities[1].parent,
        Some(StableId(1)),
        "o filho aponta para o ID do pai, nao para o indice",
    );
    assert_eq!(new.entities[2].parent, Some(StableId(2)));
}

/// **Nenhum id migrado é o reservado `0`** — senão todas as linhas colidiriam no mapa do
/// restore (o defeito que a peça 3D expôs).
#[test]
fn no_migrated_row_gets_the_reserved_id() {
    let new = migrate_v1_to_v2(&tree_v1());
    assert!(new.entities.iter().all(|r| !r.id.is_none()));
}

/// **A migração é REPRODUTÍVEL** — o mesmo ficheiro dá os mesmos ids, sempre.
///
/// É o que torna o gate de round-trip possível, e vem de graça do formato antigo: a v1
/// chegava ao disco em ordem canónica (o `canonicalize` ordenava por conteúdo), então a
/// ordem das linhas do ficheiro **é** a ordem canónica.
#[test]
fn migrating_the_same_file_twice_gives_the_same_ids() {
    let a = migrate_v1_to_v2(&tree_v1());
    let b = migrate_v1_to_v2(&tree_v1());
    assert_eq!(a, b);
    assert_eq!(a.state_hash(), b.state_hash());
}

/// **O contador é semeado depois do último id** — senão a primeira entidade criada após o
/// load reusaria um id vivo.
#[test]
fn the_counter_seed_clears_every_migrated_id() {
    let old = tree_v1();
    let seed = next_free_after_migration(&old);
    let new = migrate_v1_to_v2(&old);
    let highest = new.entities.iter().map(|r| r.id.0).max().unwrap();
    assert!(
        seed > highest,
        "o contador ({seed}) tem de comecar depois do maior id migrado ({highest})",
    );
}

/// **Um `parent` fora de alcance vira raiz**, e não um pânico nem um id inventado.
///
/// Só alcançável por ficheiro adulterado — a v1 escrevia índices que ela própria produzira.
/// A alternativa (recusar o load) perderia a cena inteira por causa de uma aresta.
#[test]
fn an_out_of_range_parent_becomes_a_root() {
    let mut old = tree_v1();
    old.entities[2].parent = Some(99);
    let new = migrate_v1_to_v2(&old);
    assert_eq!(new.entities[2].parent, None);
}

/// ⭐ **A FIXTURA: bytes v1 congelados, e a garantia de que ainda os sabemos ler.**
///
/// O plano da F1 pede *"um corpus de fixtures geradas ANTES de mexer"*. Estes bytes são a
/// codificação postcard de [`tree_v1`] tal como a v1 a escrevia — colados aqui como
/// **literais**, e não gerados pelo tipo, que é o que os torna uma fixtura de verdade.
///
/// ⚠️ **Se este teste falhar, a v1 mudou de forma** — e o `WorldSnapshotV1` deixou de ler o
/// que se propõe a ler. Nesse caso o defeito não é este teste: é que alguém "arrumou" um
/// tipo congelado, e todo projeto antigo passa a abrir com o conteúdo trocado, **em
/// silêncio** (o postcard não é auto-descritivo — ele não recusa, ele lê errado).
#[test]
fn the_frozen_v1_bytes_still_decode() {
    // Gerado uma vez com `postcard::to_allocvec(&tree_v1())` e congelado.
    // ⚠️ **Gerados, não escritos.** A 1.ª versão desta fixtura foi escrita à mão a partir do
    // que eu achava que o postcard emitia — e falhou: o `type_id` é um varint, não oito bytes
    // LE. *Uma fixtura adivinhada testa a minha ideia do formato, não o formato.* Estes saíram
    // de `postcard::to_allocvec(&tree_v1())` (ver `print_v1_bytes_for_the_fixture`).
    const V1_BYTES: &[u8] = &[
        0x01, // version = 1
        0x03, // 3 entidades
        0x01, 0xf0, 0xbd, 0xf3, 0xd5, 0x89, 0xcf, 0x95, 0x9a, 0x12, 0x01, 0x01, 0x00, 0x01, 0xf0,
        0xbd, 0xf3, 0xd5, 0x89, 0xcf, 0x95, 0x9a, 0x12, 0x01, 0x02, 0x01, 0x00, 0x01, 0xf0, 0xbd,
        0xf3, 0xd5, 0x89, 0xcf, 0x95, 0x9a, 0x12, 0x01, 0x03, 0x01, 0x01,
    ];
    let decoded: WorldSnapshotV1 =
        postcard::from_bytes(V1_BYTES).expect("os bytes v1 congelados ainda desserializam");
    assert_eq!(
        decoded,
        tree_v1(),
        "o tipo `WorldSnapshotV1` mudou de forma e deixou de ler bytes v1 reais",
    );
    // E a migração deles é a esperada.
    let new = migrate_v1_to_v2(&decoded);
    assert_eq!(new.entities.len(), 3);
    assert_eq!(new.entities[2].parent, Some(StableId(2)));
}

/// O gerador da fixtura acima. `#[ignore]` e **panica de proposito** — ele existe para ser
/// corrido a mao (`-- --ignored`) quando alguem precisar de re-gerar os bytes, e um teste que
/// imprime sem falhar nao mostra a saida.
#[test]
#[ignore]
fn print_v1_bytes_for_the_fixture() {
    let b = postcard::to_allocvec(&tree_v1()).unwrap();
    let hex: Vec<String> = b.iter().map(|x| format!("0x{x:02x}")).collect();
    panic!("BYTES({}): {}", b.len(), hex.join(", "));
}
