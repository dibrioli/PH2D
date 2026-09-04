//! ⛔⛔ **A ESCADA do *Aplicar* tem UMA porta, e o barramento CHEGA a ela** (ADR-0164 / F5,
//! critério 4).
//!
//! # Porque este gate é textual, e porque ele é preciso
//!
//! **(a) O dreno de UM BRAÇO SÓ.** O `match` que consome o `EditorAction` na `render_loop`
//! termina num `_ => {}`, então uma acção **nova sem braço compila, corre e não faz nada** — é a
//! primeira das duas espécies de controlo morto que a caça de 2026-08-30 nomeou, e nenhum gate de
//! registo a apanha. Um seam de painel prova que o clique chega ao **barramento**; ele não prova
//! que alguém do outro lado o lê.
//!
//! **(b) A porta única.** O `apply_to_level` é quem apaga a excepção **em todos os degraus** e quem
//! escreve o valor em cada um — as duas metades medidas em
//! `instance_apply_deep_tests`. Uma segunda escrita da receita, noutro sítio, produziria o
//! *no-op visível* que a regra do Unity nomeia (*«the value on the instance would change right
//! after being applied»*) e não haveria nada a apontá-la.
//!
//! ⚠️ **Ele descasca comentários antes de varrer.** Um censo textual que não separa prosa de código
//! mente nos **dois** sentidos: acusa a prosa que descreve a cura, e absolve o código quando um
//! comentário vizinho nomeia a porta. Esta linha pagou as duas em 2026-09-02.

use std::path::Path;

fn src(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

/// Só as linhas de CÓDIGO — ver a nota de cabeçalho.
fn code_of(body: &str) -> String {
    body.lines()
        .map(|l| match l.find("//") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// ⭐⭐⭐ **A acção tem BRAÇO** — sem ele o botão do cartão publica no barramento e o `_ => {}` come-o.
///
/// **Mutação que deve sangrar:** apagar o braço `EditorAction::InspectorApplyToLevel` do dreno.
#[test]
fn the_apply_level_action_reaches_the_verb() {
    let body = code_of(&src("render_loop/mod.rs"));
    assert!(
        body.contains("EditorAction::InspectorApplyToLevel"),
        "a accao nao tem braco no dreno — o `_ => {{}}` do fim do match come-a em silencio"
    );
    assert!(
        body.contains("crate::instance_apply_deep::apply_to_level("),
        "o braco existe e nao chama a porta do verbo — o clique morre a um passo do efeito"
    );
}

/// ⭐⭐ **O *Aplicar ao mestre* de sempre é um DEGRAU desta escada, e não uma segunda escrita.**
///
/// ⚠️ O `apply_to_master` continua a existir (é o item do menu da linha e o que os gates dele
/// medem), mas o corpo dele **resolve o degrau mais externo e delega**. Duas travessias que
/// escrevem na receita divergiriam no dia em que uma aprendesse a apagar a excepção intermédia e a
/// outra não — que é exactamente o defeito que o critério 4 nomeia.
///
/// **Mutação que deve sangrar:** reescrever o laço de escrita dentro do `instance_verbs.rs`.
#[test]
fn the_menu_verb_goes_through_the_same_door() {
    let body = code_of(&src("instance_verbs.rs"));
    assert!(
        body.contains("crate::instance_apply_deep::apply_to_level("),
        "o `apply_to_master` deixou de delegar — ha' uma segunda escrita da receita"
    );
    assert!(
        !body.contains("insert_from_bytes"),
        "o ficheiro dos verbos voltou a escrever bytes na receita por conta propria"
    );
}

/// ⛔⛔ **A metade que apaga a excepção INTERMÉDIA vive na porta, e em mais lado nenhum.**
///
/// Ela é a regra do Unity (*«this override in the 'Table' Prefab is reverted at the same time»*), e
/// é a diferença entre o verbo funcionar e o valor **voltar atrás** — medido em
/// `applying_to_the_inner_master_clears_the_override_in_the_middle`.
///
/// ⚠️ **A régua é a CADEIA, não a contagem de chamadas:** o `revert_override` tem outros
/// chamadores legítimos (o verbo *Revert*, o passe estrutural), e contá-los seria um número que
/// envelhece. O que este gate afirma é que o laço que desce a escada existe **aqui**.
#[test]
fn the_middle_override_is_cleared_by_the_door_itself() {
    let body = code_of(&src("instance_apply_deep.rs"));
    assert!(
        body.contains("piece_chain(sim, &by_id, key.piece)"),
        "a porta deixou de percorrer a CADEIA da chave — sem ela nao ha' degrau intermedio a limpar"
    );
    assert!(
        body.contains("crate::instance_sync::revert_override("),
        "a porta deixou de apagar a excepcao — o valor aplicado volta atras no passe seguinte"
    );
}
