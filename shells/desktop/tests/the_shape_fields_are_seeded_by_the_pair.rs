//! **Arch-gate: a semente dos campos de forma é disparada pelo PAR `(alvo, tipo)`.**
//!
//! ⚠️ Ele existe porque nenhum teste de unidade alcança esta costura: a decisão mora dentro do
//! `render_frame`, que exige `gfx` (janela + GPU). Os gates de unidade provam que
//! `shape_seed_focus` decide certo e que `seed_shape_fields` sobrescreve o slot; **nenhum dos
//! dois prova que o produto os CHAMA quando o artista troca de forma** — e era exactamente aí
//! que o defeito vivia.
//!
//! # O defeito que este gate pina (report do Enio, 2026-08-01)
//!
//! *"quando se escolhe outra shape, quase todos os parâmetros não são atualizados … mostram os
//! parâmetros da outra shape previamente modificada"*.
//!
//! Os slots do store são por **ÍNDICE** (`vector_shape_field_id(i)`), compartilhados por TODAS as
//! formas — o campo 0 de uma estrela e o de um polígono são o MESMO widget. A memo guardava só o
//! `VecPathId` do alvo, então *"nada selecionado, catálogo em Star"* e *"nada selecionado,
//! catálogo em Polygon"* comparavam **iguais** (`None == None`), a semente nunca corria, e os
//! números da forma anterior ficavam na tela.
//!
//! É a mesma classe do `MARKER_TARGET` das pontas — uma memo cuja chave é ESTREITA demais para
//! distinguir dois estados que o artista distingue.

const SRC: &str = include_str!("../src/render_loop/mod.rs");

/// A memo comparada é o PAR, e ela vem da porta única.
#[test]
fn the_seed_memo_is_the_pair_not_just_the_target() {
    let focus = SRC
        .find("let focus = crate::vec_shape_params::shape_seed_focus(")
        .expect("a decisao tem de vir da porta unica `shape_seed_focus`");
    let cmp = SRC
        .find("if Some(focus) != self.vec_shape_last_focus {")
        .expect("a memo comparada tem de ser o PAR (`vec_shape_last_focus`)");
    assert!(
        focus < cmp,
        "o par e' computado DEPOIS de ser comparado (focus em {focus}, comparacao em {cmp})"
    );
    // …e o campo estreito não pode ter sobrevivido em lugar nenhum: um segundo memo com a chave
    // antiga é o defeito de volta, com a suíte verde.
    assert!(
        !SRC.contains("vec_shape_last_target"),
        "a memo ESTREITA (so' o alvo) ainda existe — foi ela que causou o report"
    );
}

/// **Sem alvo, os valores vêm do CATÁLOGO** — e pela porta única, sem um segundo downcast.
///
/// ⚠️ A metade que falta a este gate é a razão de o `shape_catalog` existir: se o sítio de
/// decisão fizesse o próprio downcast, haveria duas respostas a *"quem é a ferramenta de
/// vetor?"*, e elas divergem no dia em que a tool muda de nome.
#[test]
fn with_no_target_the_seed_comes_from_the_catalog() {
    assert!(
        SRC.contains("let catalog = vector_bridge::shape_catalog(tools);"),
        "o catalogo tem de vir da porta unica"
    );
    let seed = SRC
        .find("crate::vec_shape_params::seed_shape_fields(&mut hero.store, focus.1, &ui);")
        .expect("a semente tem de correr com o tipo em FOCO e os valores de UI");
    let none_arm = SRC
        .find("None => catalog.map(|(_, v)| v).unwrap_or_default(),")
        .expect("sem alvo, os valores sao os que a tool guarda para aquele tipo");
    assert!(
        none_arm < seed,
        "os valores sao escolhidos DEPOIS de semeados"
    );
}
