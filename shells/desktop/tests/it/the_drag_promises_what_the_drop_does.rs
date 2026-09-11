//! ⭐⭐⭐ **O fantasma promete o que a queda faz** — a lei da wave B4
//! (plano `docs/Components/07`, etapa B).
//!
//! # ⛔⛔ Porque este gate é textual
//!
//! A propriedade é sobre a **fiação**, não sobre um valor: *o `Move` e o `Up` têm de perguntar
//! pela MESMA porta e decidir pela MESMA lei*. Um gate unitário chama uma função e fica verde com
//! ela nunca sendo chamada — foi assim que quatro mutações de fiação sobreviveram a 6 407 testes
//! verdes em 2026-08-31.
//!
//! ⚠️ **A discordância que ele impede não tem sintoma imediato:** o fantasma diria «aceita» e a
//! queda recusaria, e ninguém saberia dizer quando é que os dois deixaram de concordar.

use std::path::Path;

fn code(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&p)
        .unwrap_or_else(|e| panic!("nao li {}: {e}", p.display()))
        .lines()
        .map(str::trim_start)
        .filter(|l| !l.starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// ⭐⭐⭐ **UMA porta responde *«o que aconteceria se eu largasse aqui?»*, e ela tem DOIS
/// chamadores.**
///
/// (Mutação: o `Move` resolver o alvo por conta própria ⇒ o `drop_target_at` fica com um chamador,
/// e este gate sangra.)
#[test]
fn the_move_and_the_drop_ask_the_same_door() {
    let s = code("src/asset_drag_wire.rs");
    assert_eq!(
        s.matches("self.drop_target_at(x, y)").count(),
        2,
        "o `drop_target_at` tem de ter EXACTAMENTE dois chamadores — o `Move` e o `Up`"
    );
    assert!(
        s.contains("fn drop_target_at("),
        "a porta unica desapareceu"
    );
}

/// ⭐⭐ **E os dois decidem pela MESMA lei** — `asset_drop::resolve`.
///
/// ⚠️ Sem isto, o fantasma poderia ganhar uma tabela própria de *«isto aceita?»*, que é a segunda
/// resposta à mesma pergunta.
#[test]
fn both_decide_through_the_same_law() {
    let s = code("src/asset_drag_wire.rs");
    assert_eq!(
        s.matches("crate::asset_drop::resolve(").count(),
        2,
        "a lei da queda tem de ser consultada nos DOIS caminhos, e em nenhum outro sitio"
    );
}

/// ⛔ **Desistir não é recusa** — voltar ao painel de origem é o gesto universal de largar o
/// assunto, e ele é silencioso em todo o software que o tem.
///
/// ⚠️ **A régua é textual porque a distinção vive num braço de `match`** que nenhum valor devolve:
/// o `Cancel` tem de virar `Unknown`, nunca `Refuse`.
#[test]
fn giving_up_is_not_a_refusal() {
    let s = code("src/asset_drag_wire.rs");
    assert!(
        s.contains("DropAction::Cancel => DragVerdict::Unknown"),
        "desistir passou a pintar-se como recusa — o fantasma acusaria o artista de um erro que \
         ele nao cometeu"
    );
    assert!(
        s.contains("DropAction::Refuse => DragVerdict::Refuse"),
        "a recusa deixou de ser dita antes da queda"
    );
}
