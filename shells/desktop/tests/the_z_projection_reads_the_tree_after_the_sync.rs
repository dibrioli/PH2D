//! **Arch-gate da ORDEM DO FRAME** — a projeção de z lê a árvore DEPOIS do `sync` (BUGS #15).
//!
//! ## O que este gate protege
//!
//! A pilha de z do vetor é a projeção da árvore (ADR-0110). Se ela for projetada de uma árvore
//! lida **antes** de `vec_entities::sync` — que é quem dá entidade à forma recém-criada —, a
//! forma nova fica de fora da ordem, o `VecScene::reorder_to` lhe dá chave 0 e a manda pro
//! **FUNDO**. A cena só converge no frame seguinte.
//!
//! Isso não é só um z errado por um frame. A captura do undo (`post_frame_undo`) é tirada no fim
//! do frame da AÇÃO — ou seja, **antes de convergir**. Uma captura que não é **ponto fixo dos
//! sistemas** faz o diff por-frame ler a convergência como se fosse ação do usuário: nasce um
//! passo espúrio, ele LIMPA a pilha de redo, e o Ctrl+Z seguinte desfaz o lixo que ele mesmo
//! criou. Para o usuário: *"o undo só faz uma etapa e não funciona mais"*.
//!
//! ## Por que um gate de TEXTO, e não só os unit tests
//!
//! Os gates de comportamento (`vec_zorder_fixpoint_tests.rs`) rodam um **espelho** da sequência
//! do frame, escrito à mão — a convenção desta shell (ver `label_live_tests::Doc::frame`). Um
//! espelho não vê o dia em que alguém reordena o frame **de verdade**, e foi assim que esta
//! classe de bug entrou. Este gate lê o **arquivo do produto** e afirma a ordem que importa:
//!
//! ```text
//! vec_entities::sync                 (a forma nova ganha entidade)
//!   → assign_missing_root_order      (toda raiz ganha número: sem empate, sem `to_bits`)
//!   → build_hierarchy_snapshot(z_)   (a árvore lida AGORA)
//!   → z_order → reorder_to           (a projeção)
//! ```
//!
//! Se você precisar mesmo mover um destes passes, o teste falha com a razão — não o silencie sem
//! rodar `PH2D_BUILD_SMOKE=6 PH2D_UNDO_LOG=1` e ler o log de undo.

const SRC: &str = include_str!("../src/render_loop/mod.rs");

/// A posição (em bytes) da 1ª ocorrência de `needle`, ou pânico com a razão.
fn at(needle: &str) -> usize {
    SRC.find(needle).unwrap_or_else(|| {
        panic!(
            "o passe `{needle}` sumiu do render_loop — se ele foi renomeado, atualize este gate \
             (e confira que a ordem do §BUGS #15 continua de pé)"
        )
    })
}

#[test]
fn the_z_projection_reads_the_tree_after_the_sync() {
    let sync = at("vec_entities::sync(");
    let root_order = at("assign_missing_root_order(");
    let snapshot = at("&mut live.z_snapshot,");
    let reorder = at("vec_scene.reorder_to(");

    assert!(
        sync < root_order,
        "`assign_missing_root_order` tem de rodar DEPOIS do `sync`: a entidade da forma nova \
         nasce no sync, e é ela que precisa do número"
    );
    assert!(
        root_order < snapshot,
        "a árvore tem de ser lida DEPOIS de toda raiz ter `RootOrder`: sem número, o sort de \
         raízes desempata por `Entity::to_bits()` (id de ALOCAÇÃO), e o respawn do undo o troca"
    );
    assert!(
        snapshot < reorder,
        "o `reorder_to` tem de projetar o snapshot DESTE frame (`z_snapshot`), não a lista que o \
         painel publicou no prólogo — ela é anterior ao nascimento da entidade"
    );
}

/// O contrapeso do gate acima: a projeção **não pode voltar a ler a lista do painel**.
///
/// Era essa a fonte antiga (`hero.store.hierarchy_order()`, via a ponte de `NodeId`). Ela é
/// publicada no prólogo do frame e chega tarde para a forma que acabou de nascer.
#[test]
fn the_z_projection_never_reads_the_panels_hierarchy_order() {
    const VEC_ENTITIES: &str = include_str!("../src/vec_entities.rs");
    assert!(
        !VEC_ENTITIES.contains("hierarchy_order()"),
        "`vec_entities` voltou a projetar a pilha de z da lista do PAINEL. Ela é publicada no \
         prólogo do frame, antes de o `sync` dar entidade à forma nova — a forma cai pro fundo e \
         a captura do undo deixa de ser ponto fixo (BUGS #15). A fonte é o snapshot da ÁRVORE."
    );
}
