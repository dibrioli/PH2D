//! Os gates do formato da árvore de tags — `docs/Components/08_plano_tags.md` §5.2 (gate 18, a
//! metade do FORMATO; a metade do undo vive na shell), escritos contra um *stub* e vistos vermelhos.

use super::{SavedTag, TAGS_DOC_VERSION, TagsCache, collect, restore};
use ph2d_tags::{TagId, TagTree};

fn arvore() -> TagTree {
    let mut t = TagTree::new();
    t.create("Enemy/Flying/Boss").expect("cria");
    let estatua = t.create("Statue").expect("cria");
    // ⚠️ Apagar a última faz o `next_id` ficar ACIMA de `max(id) + 1` — é o que prova que ele viaja.
    let _ = t.delete(estatua);
    t.create("Player").expect("cria");
    t
}

/// ⭐⭐⭐ **A árvore sobrevive aos próprios bytes**, e re-codificá-la dá os MESMOS bytes — é isso que
/// impede um restauro de undo de registar um passo espúrio.
///
/// **Mutações que devem sangrar:** perder um campo no `collect` · o `restore` a ordenar de outra
/// maneira (os bytes da segunda codificação mudariam).
#[test]
fn a_tag_tree_survives_its_own_bytes() {
    let t = arvore();
    let b = collect(&t);
    let (de_volta, remap) = restore(&b);
    assert_eq!(de_volta, t, "a arvore mudou na ida e volta");
    assert!(remap.is_empty(), "os proprios bytes nao tem gemeos");
    assert_eq!(collect(&de_volta), b, "re-codificar deu outros bytes");
}

/// ⛔ **O `next_id` viaja, então o id de uma tag apagada nunca é reciclado** depois de gravar e reler.
///
/// **Mutação que deve sangrar:** não gravar o `next_id` (o `restore` derivaria `max(id) + 1`).
#[test]
fn the_next_id_travels_so_a_deleted_tag_is_never_recycled() {
    let mut t = TagTree::new();
    t.create("A").expect("cria");
    let b = t.create("B").expect("cria");
    let _ = t.delete(b);
    let (mut de_volta, _) = restore(&collect(&t));
    assert_ne!(
        de_volta.create("C"),
        Ok(b),
        "a tag nova herdou o id da apagada"
    );
}

/// ⚠️ **Um blob ilegível ou de outra versão abre uma árvore VAZIA** (e diz), e um blob vazio é um
/// projecto sem árvore.
///
/// **Mutação que deve sangrar:** ignorar a versão (um blob `v99` com a forma certa seria lido).
#[test]
fn an_unreadable_or_foreign_blob_opens_an_empty_tree() {
    assert!(restore(&[]).0.is_empty());
    assert!(restore(&[0xff, 0x13, 0x37]).0.is_empty());
    let estrangeiro = postcard::to_allocvec(&(
        99u32,
        vec![SavedTag {
            id: 1,
            path: "Enemy".into(),
        }],
        2u64,
    ))
    .expect("serializa");
    assert!(
        restore(&estrangeiro).0.is_empty(),
        "uma versao desconhecida foi lida"
    );
}

/// ⭐⭐ **Um documento com GÉMEOS funde-os e devolve o mapa** — a porta do load chega à árvore.
#[test]
fn a_document_with_twins_merges_and_hands_back_the_remap() {
    let bytes = postcard::to_allocvec(&(
        TAGS_DOC_VERSION,
        vec![
            SavedTag {
                id: 3,
                path: "Enemy".into(),
            },
            SavedTag {
                id: 9,
                path: "énemy".into(),
            },
        ],
        10u64,
    ))
    .expect("serializa");
    let (t, remap) = restore(&bytes);
    assert_eq!(t.len(), 1);
    assert_eq!(remap.get(&TagId(9)), Some(&TagId(3)));
    assert_eq!(t.next_id(), 10);
}

/// ⭐⭐ **A cache codifica UMA vez por revisão, e outra vez depois de invalidada**.
///
/// **Mutações que devem sangrar:** tirar a guarda da revisão (codifica a cada quadro) · o
/// `invalidate` não esquecer a revisão (a árvore restaurada devolveria os bytes da anterior).
#[test]
fn the_cache_encodes_once_per_revision_and_again_after_invalidate() {
    let mut t = arvore();
    let mut cache = TagsCache::default();
    let primeiro = cache.doc(&t).to_vec();
    let _ = cache.doc(&t);
    assert_eq!(cache.encodes, 1, "codificou duas vezes a mesma revisao");
    assert_eq!(primeiro, collect(&t));

    t.create("Prop").expect("cria");
    assert_ne!(cache.doc(&t), primeiro.as_slice());
    assert_eq!(
        cache.encodes, 2,
        "uma mutacao da arvore custa UMA codificacao"
    );
}

/// ⛔⛔ **O `invalidate` é o que impede a cache de devolver a árvore ANTERIOR** — e a colisão não é
/// rara, é o caso NORMAL: toda árvore restaurada nasce com revisão `0`, e a cache de uma sessão que
/// só viu a árvore VAZIA está parada em `0` também.
///
/// ⚠️ **Sem ele, o dano não é um quadro lento: é PERDA.** A captura seguinte guardaria os bytes da
/// árvore vazia, e o `Ctrl+S` a seguir escreveria isso por cima das tags que o projecto tinha.
///
/// ⚠️ **A 1.ª redacção deste gate não media isto** (mutação SOBREVIVEU, 2026-09-13): ela substituía a
/// árvore por uma de revisão DIFERENTE da que a cache vira, e nesse caso a cache re-codifica de
/// qualquer maneira. *Um gate que troca o sujeito por um que não colide não testa a colisão.*
///
/// **Mutação que deve sangrar:** o `invalidate` a não esquecer a revisão.
#[test]
fn the_cache_forgets_the_previous_tree_even_when_the_revision_collides() {
    let mut cache = TagsCache::default();
    let vazia = TagTree::new();
    let bytes_vazios = cache.doc(&vazia).to_vec();
    assert_eq!(vazia.revision(), 0, "a arvore de arranque esta' em zero");

    // O load: a árvore que vem do ficheiro nasce em revisão `0`, como a vazia que a cache viu.
    let (restaurada, _) = restore(&collect(&arvore()));
    assert_eq!(
        restaurada.revision(),
        0,
        "uma arvore restaurada nasce em zero"
    );
    assert!(!restaurada.is_empty());

    cache.invalidate();
    let agora = cache.doc(&restaurada).to_vec();
    assert_ne!(
        agora, bytes_vazios,
        "a cache devolveu os bytes da arvore ANTERIOR — o Ctrl+S seguinte apagaria as tags"
    );
    assert_eq!(agora, collect(&restaurada));
}
