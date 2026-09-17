//! ⭐⭐ **UM PAINEL TEM UM NOME, não dois.**
//!
//! O menu *Window* nomeia treze painéis pela tabela de strings (`chrome.menu.*`) e a aba de um
//! encaixe nomeia o mesmo painel pelo `Panel::TITLE`, que desde 2026-09-17 é **outra** chave da
//! tabela (`panel.<id>.title`). São duas superfícies a responder à MESMA pergunta — e a que o
//! artista lê menos é a que envelhece.
//!
//! ⚠️⚠️ **Os dois lados são CHAVES e este gate compara o TEXTO de propósito.** Comparar as chaves
//! leria verde sobre uma tradução que divergisse, que é o único regime em que isto pode partir hoje;
//! e ⛔ **colapsá-las numa só chave é obra própria com bloqueador nomeado** — a camada de chrome não
//! depende de painel nenhum (é por isso que o `MODULE_TRUTHS` guarda o `Panel::ID` como literal), e
//! a ponte seria o registo em runtime, que é o que este ficheiro já faz.
//!
//! ⛔ **É por isso que o `TITLE` não tem default.** Um derivado do `Panel::ID` daria *"Tokens"*,
//! *"Sculpt3d"* e *"Grid Snap"*: três divergências no dia em que nascesse, sem uma linha de erro.
//!
//! ⭐ A ponte entre as duas é a tabela [`menu_bar::MODULE_TRUTHS`], que já existia — ela é quem
//! sabe que o pill `TOPBAR_TOKENS` fala do painel `"tokens"`.

use ph2d_editor_core::interaction::ContextMenuKind;
use ph2d_editor_core::screens::hero::{menu_bar, menu_rows};

#[test]
fn the_tab_and_the_menu_call_a_panel_the_same_thing() {
    let _ = ph2d_panel_registry_init::register_all_panels();

    let mut titles = std::collections::BTreeMap::new();
    ph2d_editor_core::panel::with_registry_ref(|reg| {
        for p in reg.panels() {
            titles.insert(p.manifest.id, p.manifest.title);
        }
    });

    // Onde um id de módulo pode aparecer rotulado. ⚠️ A *View* também nomeia dois painéis
    // (`Hierarchy` / `Inspector`), e deixá-la de fora perderia metade da população.
    let menus = [ContextMenuKind::MenuBarWindow, ContextMenuKind::MenuBarView];

    let mut disagree = Vec::new();
    let mut checked = 0usize;
    for kind in menus {
        for (row_id, label, _) in menu_rows::menu_rows(kind) {
            let label = label.tr();
            let Some((_, truth)) = menu_bar::MODULE_TRUTHS.iter().find(|(id, _)| id == row_id)
            else {
                continue; // esta linha não fala de um painel
            };
            let menu_bar::ModuleTruth::Panel(panel_id) = truth else {
                continue; // ferramenta, modo de imagem, régua — nenhum tem aba
            };
            let Some(title) = titles.get(panel_id) else {
                continue; // o painel não está nas features desta build
            };
            checked += 1;
            // ⚠️ **Os DOIS lados são chaves desde 2026-09-17** — o que se compara é o que o
            //    artista LÊ, não o identificador: dois painéis podem partilhar uma palavra.
            let title = title.tr();
            if title != label {
                disagree.push(format!(
                    "{panel_id}: o menu diz {label:?} e a aba diz {title:?}"
                ));
            }
        }
    }

    assert!(
        checked >= 8,
        "só {checked} painéis nomeados nos dois sítios — a ponte `MODULE_TRUTHS` deixou de ligar \
         as duas superfícies e este gate mede o vazio"
    );
    assert!(
        disagree.is_empty(),
        "o mesmo painel tem dois nomes; escolha um e escreva-o nos dois:\n  {}",
        disagree.join("\n  ")
    );
}
