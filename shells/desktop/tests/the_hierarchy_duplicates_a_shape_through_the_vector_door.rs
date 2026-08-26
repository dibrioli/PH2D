//! **Arch gate: a row Duplicate da Hierarchy duplica uma FORMA pela porta do vetor.**
//!
//! A decisão mora dentro de `render_loop::hierarchy::dispatch`, que precisa de um `HeroLive` com
//! a ponte row→entidade — nenhum teste de unidade a alcança, e é por isso que o gate lê o FONTE.
//! O invariante de comportamento (o clone recebe o próprio path e o `sync` cunha uma entidade
//! para ele) é do irmão `duplicating_a_shape_gives_the_copy_its_own_path_and_its_own_entity`.
//!
//! ⚠️ O defeito que ele existe para impedir não era um botão a faltar: a row **já existia** e
//! clonava a ENTIDADE (Transform + Name + ChildOf). Uma forma vetorial guarda a geometria no
//! documento e é apontada por `VecPathRef`, então o clone nascia **sem geometria nenhuma** — uma
//! linha na Hierarchy que não desenha nada.

const SRC: &str = include_str!("../src/render_loop/hierarchy.rs");

/// **A pergunta é feita ANTES de a cópia nascer.** Um caminho genérico que corresse primeiro já
/// teria criado a entidade-fantasma que o gate existe para impedir.
///
/// ⚠️ **O CONTROLE mudou de marco em 2026-08-26, e a mudança é o registo de um facto:** o braço
/// genérico deixou de ser um `spawn_empty()` (a cópia RASA de quatro componentes) e passou a ser a
/// porta `duplicate_subtree` da F4.2 — a cópia PROFUNDA. O controle apanhou-o e disse a frase
/// certa (*«este gate mede o arquivo errado»*); mover o marco é honrá-lo, não afrouxá-lo.
#[test]
fn the_duplicate_row_asks_for_a_vec_path_before_it_spawns() {
    let asks = SRC
        .find("get::<ph2d_ecs::VecPathRef>")
        .expect("o drain nao pergunta se a row e' uma forma vetorial");
    let spawns = SRC
        .find("duplicate_subtree(")
        .expect("CONTROLE: o caminho generico deixou de copiar — este gate mede o arquivo errado");
    assert!(
        asks < spawns,
        "o spawn corre ANTES da pergunta: uma forma vetorial ganharia um sosia sem geometria"
    );
}

/// **E ela roteia para a PORTA**, nunca para uma segunda implementação de duplicar.
#[test]
fn the_shape_route_goes_through_the_one_duplicate_door() {
    assert!(
        SRC.contains("duplicate_vec_paths("),
        "a rota de forma nao chama a porta `duplicate_vec_paths`"
    );
    assert!(
        SRC.contains("PASTE_OFFSET_PX"),
        "a rota de forma nao usa o offset PARTILHADO — ele ja' divergiu uma vez"
    );
}

/// **O caminho do sprite sobrevive** — a metade que impede a cura de comer o que já funcionava.
///
/// ⚠️ Sem ela, mandar TODA row pela porta do vetor passaria nos dois gates acima e quebraria a
/// duplicação de sprite em silêncio.
#[test]
fn the_sprite_route_is_still_there() {
    assert!(
        SRC.contains("Duplicated entity"),
        "o caminho do sprite sumiu"
    );
    assert!(
        SRC.contains("Duplicated shape"),
        "o caminho da forma nao confirma nada ao artista"
    );
}
