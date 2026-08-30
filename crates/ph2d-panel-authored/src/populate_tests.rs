//! Gates do **registro** (plano UI/UX W8b.2).

use super::*;
use crate::rows::baked;
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
            label: "x".to_string(),
            key: "x".to_string(),
            id: ph2d_a11y::NodeId(1),
            rgba: None,
            icon: None,
            icon_id: None,
            options: Vec::new(),
        };
        assert_eq!(
            initial(kind).is_some(),
            row.is_control(),
            "{kind:?}: `initial` e `is_control` discordam"
        );
    }
}

/// **O CENSO DA FAMÍLIA DE LISTA** — a lista que um LAÇO itera contra o `match` que a lê.
///
/// ⚠️ **O irmão acima ([`every_kind_agrees_with_itself_about_being_a_control`]) prova que
/// `initial` e `is_control` concordam, e isso NÃO é esta pergunta.** Um controle de opções passa
/// por mais duas portas que nenhum outro atravessa, e as duas são escritas com um `_ =>` no fim:
/// [`crate::rows::selected_of`] (*qual está marcada?*) e o `set_index` sob
/// [`crate::rows::select_in`] (*marca esta*). Quem decide **que rows sequer têm opções** não é
/// nenhuma delas — é um LAÇO sobre `WidgetKind::takes_options()`, em três sítios
/// ([`options`], [`crate::rows::option_for`], [`retire_vanished`]).
///
/// ⛔ **É a lei que o repo já pagou noutro sítio: *um `match` exaustivo não guarda a lista que um
/// laço itera*.** Os dois `match` da seleção **não são exaustivos** — têm `_ => None` / `_ =>
/// return false` —, então uma quinta família de lista compila em silêncio: o `takes_options`
/// passa a dizer `true`, o `populate` regista as N opções como botões, o `paint` desenha-as e
/// dá-lhes retângulo de hit, o clique CHEGA ao painel — e o `select_in` devolve `false`, o
/// `apply_event` cai no `Ignored`, e a marca nunca se move. **Pintado, vivo sob o dedo, e morto.**
/// É a mesma forma do dreno de um braço só, um nível abaixo.
///
/// A régua é dos DOIS lados, e é isso que a torna um censo em vez de uma amostra:
///
/// 1. `takes_options()` ⟺ a porta de LEITURA responde para o estado que aquele tipo nasce a ter;
/// 2. a porta de ESCRITA move a marca que a de leitura devolve (ida-e-volta, não duas leituras);
/// 3. e o [`crate::rows::clamp_selection_to`] — a terceira consumidora do mesmo `set_index` —
///    conhece a mesma variante. Sem ela, a reconciliação por quadro fica muda para o tipo novo e
///    o índice guardado aponta para uma opção que o artista apagou.
///
/// ⚠️ **A metade JUSTA é a contagem.** Sem ela, um catálogo em que `takes_options()` respondesse
/// `false` a tudo passaria este gate por vácuo — verde a afirmar nada. O baseline medido em
/// 2026-08-30 é **4** (`Tabs` · `RadioGroup` · `SegmentedAdaptive` · `Dropdown`), e ele só sobe.
#[test]
fn every_kind_that_takes_options_has_both_doors_of_the_selection() {
    let mut family = 0usize;
    for kind in WidgetKind::ALL {
        let takes = kind.takes_options();
        let Some(mut live) = initial(kind) else {
            assert!(
                !takes,
                "{kind:?} toma opcoes e o `initial` nem o regista — as opcoes ganhariam \
                 retangulo de hit sobre uma row que o store nao conhece"
            );
            continue;
        };
        // (1) as duas respostas à mesma pergunta.
        assert_eq!(
            takes,
            crate::rows::selected_of(Some(&live)).is_some(),
            "{kind:?}: `WidgetKind::takes_options` e `rows::selected_of` discordam sobre se este \
             tipo tem uma OPCAO MARCADA. O laco do `populate` segue a primeira e o `event` segue \
             a segunda: com `takes=true` e leitura muda, o clique na opcao cai no `Ignored`; ao \
             contrario, o `event` inventa uma escolha numa row sem opcoes"
        );
        if !takes {
            continue;
        }
        family += 1;
        // (2) a porta de ESCRITA move o que a de leitura devolve.
        assert!(
            crate::rows::select_in(&mut live, 2),
            "{kind:?}: `rows::select_in` recusou marcar — o `set_index` nao tem braco para a \
             variante que o `initial` deste tipo produz, e escolher uma opcao vira um no-op"
        );
        assert_eq!(
            crate::rows::selected_of(Some(&live)),
            Some(2),
            "{kind:?}: escreveu-se a marca 2 e a porta de leitura devolveu outra coisa — o \
             controle desenha uma opcao e reporta outra"
        );
        // (3) a terceira consumidora do mesmo `set_index`.
        assert!(
            crate::rows::clamp_selection_to(&mut live, 2),
            "{kind:?}: a marca 2 sobrevive a uma row com 2 opcoes — a reconciliacao por quadro \
             ficou muda para este tipo"
        );
        assert_eq!(
            crate::rows::selected_of(Some(&live)),
            Some(1),
            "{kind:?}: o clamp nao encolheu para a ULTIMA opcao que a row consegue oferecer"
        );
    }
    assert_eq!(
        family, 4,
        "a familia de LISTA mudou de tamanho (baseline 2026-08-30: 4). Se um tipo entrou, ele tem \
         de atravessar as tres portas acima antes de este numero subir; se um saiu, o gate deixou \
         de medir o que dizia medir"
    );
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
    for row in baked() {
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
    for row in baked() {
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
