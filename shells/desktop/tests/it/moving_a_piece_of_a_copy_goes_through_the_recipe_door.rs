//! ⛔⛔ **O ARRASTO pergunta se a peça veio da RECEITA antes de lhe mudar o pai** (ADR-0164 / F5.12).
//!
//! # Porque este gate é textual
//!
//! O `drain_reparent` recebe o `HeroLive` — a ponte entre os nós da Hierarquia e as entidades —, que
//! um teste não monta sem uma janela. Um teste de integração ali seria uma montagem maior do que a
//! lei que ele mede, e provaria que **este** caminho pergunta, nunca que **não existe um segundo**
//! que não pergunta. *A ausência de uma segunda escrita não se mede correndo o caminho certo.*
//!
//! ⛔ **Ele descasca comentários antes de varrer** — um censo textual que não separa prosa de código
//! mente nos dois sentidos, e esta linha já o pagou duas vezes.
//!
//! # ⚠️ A lei que ele defende
//!
//! Desde a F5.12 o passe estrutural arruma cada peça de cópia no pai que a receita lhe dá. Sem a
//! guarda, o arrasto do artista **desfazia-se sozinho no quadro seguinte** — a mão a perder para um
//! passe invisível, que é exactamente o defeito que o apagar pagou em 2026-09-05. *A guarda vive no
//! GESTO; a lei fica no passe.*

use std::path::Path;

fn code_of(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(rel);
    let body = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    body.lines()
        .map(|l| match l.find("//") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// ⭐⭐⭐ **A pergunta corre ANTES de o `ChildOf` mudar.**
///
/// Depois já não serve: a peça está no sítio novo, e a única coisa que a devolve é o passe — em
/// silêncio, no quadro seguinte.
///
/// **Mutação que deve sangrar:** apagar a guarda, ou pô-la depois do `insert(ChildOf(p))`.
#[test]
fn the_drag_asks_the_door_before_changing_the_parent() {
    // ⚠️⚠️ **A janela é o CORPO do gesto, e não o ficheiro** — a 1.ª redacção procurava o
    // `is_a_recipe_given_piece` em qualquer sítio, e quando a lei saiu para a porta
    // `refuses_reparent` (acima nesta ficheiro) o censo ficou **VÁCUO**: a mutação que apagava a
    // chamada dentro do gesto **sobreviveu**, porque o nome continuava a aparecer no ficheiro.
    // *Um censo que casa um nome onde ele passou a viver deixa de medir quem o invoca.*
    let file = code_of("hero_intents/hierarchy.rs");
    let at = file
        .find("pub(crate) fn drain_reparent(")
        .expect("o gesto mudou de nome — reancore este censo");
    let body = &file[at..];
    let ask = body
        .find("refuses_reparent(sim, dragged, new_parent_entity)")
        .expect("o arrasto deixou de consultar a porta da recusa — ele desfaz-se sozinho");
    let write = body
        .find("entry.insert(ph2d_ecs::ChildOf(p))")
        .expect("a escrita do pai novo");
    assert!(
        ask < write,
        "a pergunta corre DEPOIS de o pai mudar — a peca ja' se mexeu quando alguem se lembra de \
         perguntar"
    );
}

/// ⚠️ **A guarda só dispara quando o PAI muda** — reordenar entre irmãos continua a valer, porque a
/// ordem viaja no `SiblingOrder`, que **é** componente registado e vira excepção da cópia como
/// qualquer outro valor.
///
/// ⛔ Uma guarda sem esta metade proibiria arrastar uma peça de cópia **para qualquer sítio**,
/// incluindo o lugar onde ela já está. *Duas perguntas diferentes sobre o mesmo arrasto, e só uma
/// delas é sobre a forma.*
///
/// **Mutação que deve sangrar:** tirar o `!same_parent` da condição.
#[test]
fn reordering_between_siblings_is_still_allowed() {
    let file = code_of("hero_intents/hierarchy.rs");
    let at = file
        .find("pub(crate) fn refuses_reparent(")
        .expect("a porta da recusa mudou de nome — reancore este censo");
    let end = file[at..]
        .find("\npub(crate) fn ")
        .map_or(file.len(), |n| at + n);
    let door = &file[at..end];
    assert!(
        door.contains("!same_parent"),
        "a porta deixou de exigir que o PAI mude — ela passa a recusar tambem o reordenar entre \
         irmaos, que e' uma excepcao legitima da copia:\n{door}"
    );
    assert!(
        door.contains("is_a_recipe_given_piece("),
        "a porta deixou de perguntar se a receita DEU a peca — ela passa a recusar tambem o que o \
         artista pendurou lá dentro:\n{door}"
    );
}

/// ⛔ **A recusa diz ONDE fazer.** A receita alcança-se pelo *Edit Prefab* da biblioteca, e uma
/// recusa muda deixa o artista a repetir o mesmo gesto até desistir.
#[test]
fn the_refusal_says_where_to_do_it_instead() {
    let body = code_of("hero_intents/hierarchy.rs");
    assert!(
        body.contains("Edit \\\n             Prefab") || body.contains("Edit Prefab"),
        "a recusa nao nomeia a porta que abre a receita"
    );
}
