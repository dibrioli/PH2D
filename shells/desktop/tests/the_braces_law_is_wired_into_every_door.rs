//! ⛔⛔⛔ **A lei «as chaves mandam» está LIGADA em todas as portas — e isto é o gate da FIAÇÃO.**
//!
//! # Porque ele é textual, e porque existe
//!
//! A auditoria multiagêntica de 2026-08-31 (auditor 4) mutou as quatro costuras uma a uma —
//! o `follow` da seleção, o espelho do swap, o arrasto do rename, o publish do cartão — e **as
//! quatro mutações sobreviveram com 6 407 testes verdes**: os gates unitários chamam as funções
//! directamente, e os de seam alimentam o painel eles próprios. *Ninguém percorria a SEQUÊNCIA.*
//!
//! Um teste de integração provaria que ESTE caminho funciona, não que não existe um quadro sem
//! ele — a ausência de uma costura não se mede correndo o caminho certo. (O molde é o
//! `the_axis_chip_goes_through_the_swap_door`.)
//!
//! ⛔ Descasca comentários antes de varrer: documentar a cura não pode reprovar o portão.

use std::path::Path;

fn shell_src(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

fn strip_comments(s: &str) -> String {
    s.lines()
        .map(|l| match l.find("//") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// ⭐⭐⭐ **As quatro costuras, pelo nome** — cada linha desta tabela é uma mutação que sobreviveu.
#[test]
fn every_door_of_the_braces_law_is_wired() {
    let render = strip_comments(&shell_src("render_loop/mod.rs"));
    assert!(
        render.contains("instance_declared_value::follow("),
        "o elo nao segue as chaves em mudanca de selecao — a mutacao 1 da auditoria voltou"
    );
    assert!(
        render.contains("instance_declared_value::mirror_onto_copy("),
        "o swap do chip nao espelha as chaves — e' a BRIGA do cabecalho da lei (mutacao 2)"
    );
    assert!(
        render.contains("instance_declared_value::mirror_onto_copies_of("),
        "o rename do valor nao arrasta as copias (mutacao 3)"
    );
    assert!(
        render.contains("instance_declared_value::family_declares("),
        "o campo do chip nao prefere a TROCA — digitar um valor existente cria a receita duplicada"
    );

    let verbs = strip_comments(&shell_src("instance_verbs.rs"));
    assert!(
        verbs.contains("instance_declared_value::mirror_onto_copy("),
        "o *Make Variant* nao escreve nas chaves da copia — o follow da selecao DESFA'-LO no \
         quadro seguinte (sonda C da auditoria)"
    );

    let law = strip_comments(&shell_src("instance_declared_value.rs"));
    assert!(
        law.contains("mirror_onto_copies_of(sim, master_id)"),
        "o braco de AUTORIA do apply nao arrasta as copias irmas"
    );
    assert!(
        law.contains("mirror_onto_copies_of(sim, sid)"),
        "o commit de nome sobre uma RECEITA nao arrasta as copias dela"
    );

    let snapshots = strip_comments(&shell_src("render_loop/snapshots.rs"));
    assert!(
        snapshots.contains("build_properties_info")
            && snapshots.contains("set_current_inspector_properties"),
        "o cartao PROPERTIES nao e' construido/publicado — some do app inteiro (mutacao 4)"
    );
}

/// ⛔ **E o `follow` NÃO corre por quadro** — a forma que agia a meio da digitação.
///
/// ⚠️ A ausência aqui é a decisão: o campo de nome escreve o `Name` no mundo a cada TECLA, então
/// um `follow` por quadro trocava o elo no primeiro backspace de `Small 2` (achado 6 do auditor
/// 1). O certo é por MUDANÇA de seleção — e este gate prende a comparação que o garante.
#[test]
fn the_follow_runs_on_selection_change_never_per_frame() {
    let render = strip_comments(&shell_src("render_loop/mod.rs"));
    assert!(
        render.contains("hero.gizmo.selection != self.followed_selection"),
        "a guarda de mudanca de selecao sumiu — o follow voltou a correr por quadro, e age a meio \
         da digitacao"
    );
}
