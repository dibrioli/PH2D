//! Gates do **registro** (plano UI/UX W8b.2).

use super::*;
use ph2d_editor_core::interaction::WidgetStore;

fn populated() -> WidgetStore {
    let mut store = WidgetStore::with_capacity(64);
    populate(&mut store);
    store
}

/// **As duas metades da porta concordam.**
///
/// ⚠️ `is_control` decide se a row ganha retângulo de hit e se o evento é rotado; `initial` decide
/// se ela é registada. Se divergirem, o modo de falha é diferente em cada direção — registada e
/// sem rect nunca é clicada; rect sem registo tem o clique descartado **em silêncio**. Por isso a
/// pergunta é UMA e este gate a pina.
#[test]
fn every_kind_agrees_with_itself_about_being_a_control() {
    for kind in WidgetKind::ALL {
        let row = Row {
            kind,
            label: "x",
            key: "x",
            id: ph2d_a11y::NodeId(1),
        };
        assert_eq!(
            initial(kind).is_some(),
            row.is_control(),
            "{kind:?}: `initial` e `is_control` discordam"
        );
    }
}

/// **Toda row que responde está REGISTADA.**
///
/// ⚠️ Um widget pintado, hit-registrado e ausente do store tem o clique descartado **em silêncio**
/// — sem erro, sem warning. Este gate prova o REGISTRO; *estar viva sob o mouse* é o que o seam
/// (`tests/seam_authored.rs`) prova, dirigindo um ponteiro real: `is_focusable` é privado do
/// dispatch de propósito, e reimplementá-lo aqui seria a segunda resposta a *"o que é clicável?"*.
#[test]
fn every_control_row_is_registered() {
    let store = populated();
    for row in rows() {
        if row.is_control() {
            assert!(
                store.get(row.id).is_some(),
                "a row `{}` responde a gestos e nao foi registada",
                row.key
            );
        }
    }
}

/// **E a que só desenha NÃO está.**
///
/// ⚠️ A metade oposta do gate acima, e não é redundante: um `populate` que registasse tudo passaria
/// no primeiro e daria um painel onde o cabeçalho de seção acende sob o rato e não faz nada.
#[test]
fn a_display_only_row_is_not_registered() {
    let store = populated();
    let mut checked = 0;
    for row in rows() {
        if !row.is_control() {
            assert!(
                store.get(row.id).is_none(),
                "a row `{}` so' desenha e foi registada",
                row.key
            );
            checked += 1;
        }
    }
    // Controle positivo: sem uma row de desenho puro na tabela este gate seria verde por vácuo.
    assert!(
        checked > 0,
        "a tabela perdeu a row de desenho puro — este gate deixou de medir algo"
    );
}

/// **O chrome do painel está registado** — fechar, mover e redimensionar.
#[test]
fn the_panel_chrome_is_registered() {
    let store = populated();
    for (id, what) in [
        (ids::AUTHORED_CLOSE, "close"),
        (ids::AUTHORED_DRAG_HANDLE, "drag"),
        (ids::AUTHORED_RESIZE_HANDLE, "resize"),
        (ids::AUTHORED_RESIZE_HANDLE_BL, "resize_bl"),
    ] {
        assert!(
            store.get(id).is_some(),
            "o chrome `{what}` nao foi registado"
        );
    }
}
