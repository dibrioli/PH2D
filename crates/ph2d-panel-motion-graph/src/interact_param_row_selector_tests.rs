//! ⭐⭐⭐ **AS DUAS SETAS E A LISTA DE UM SELECTOR** (report do Enio, 2026-09-07: *«deveríamos ter
//! duas setas laterais e se clicar no centro abre-se um dropdown»*) — os gates do gesto.
//!
//! ⚠️ Irmão de [`super`] por responsabilidade: lá ficam os gestos de uma row de NÚMERO (arrastar,
//! escrever, virar um interruptor); aqui os de uma row de ESCOLHA.

use super::super::tests::{CENTER, RECT};
use super::{card, click_zone, enum_param, gesture};
use crate::snapshot::{GraphIntent, drain_intents};
use ph2d_editor_core::interaction::{GesturePhase, GraphHitKind};

/// ⭐⭐ **UM CLIQUE AVANÇA O ENUM, COM VOLTA AO PRINCÍPIO** — é o gesto de quem quer *a
/// seguinte*, e o `Enum` é **20 % de todas as rows do catálogo, em 85 nós** (censo de
/// 2026-09-05). Arrastar continua a varrer, que é como se atravessa um enum de 48 opções.
/// FALSIFICADO por o clique não emitir nada (o artista fica com um rótulo que não muda) ou por
/// não dar a volta (a última opção prende).
#[test]
fn a_click_advances_an_enum_and_wraps() {
    use crate::geom::RowZone;
    let passo = |v: f32, zona: RowZone| -> f32 {
        let snap = card(vec![enum_param(v, &["Sine", "Triangle", "Square"])]);
        let saiu = click_zone(&snap, 0, zona).1;
        let Some(GraphIntent::SetParam { value, param, .. }) = saiu.first() else {
            panic!("a seta tem de mover o enum, e saiu {saiu:?}");
        };
        assert_eq!(*param, "mode");
        *value
    };
    assert_eq!(passo(0.0, RowZone::Next), 1.0, "de Sine para Triangle");
    assert_eq!(
        passo(2.0, RowZone::Next),
        0.0,
        "da ultima volta ao principio"
    );
    // ⭐ E o OUTRO sentido, que é o que a seta da esquerda comprou: sem ela, chegar à opção
    // anterior custava `n-1` cliques numa lista de 48.
    assert_eq!(passo(1.0, RowZone::Prev), 0.0, "de Triangle para Sine");
    assert_eq!(
        passo(0.0, RowZone::Prev),
        2.0,
        "e a anterior a` primeira e' a ULTIMA"
    );
}

/// ⭐⭐⭐ **UM CLIQUE NO NOME ABRE A LISTA** (report do Enio, 2026-09-07, com a foto do selector
/// do Blender: *«se clicar no centro (nome) abre-se um dropdown»*).
///
/// ⚠️ **E não escreve nada.** Abrir uma lista é uma pergunta, não uma resposta — um clique que
/// mudasse o valor *e* abrisse a lista faria o artista escolher a partir de um estado que ele
/// não pediu.
///
/// FALSIFICADO por o centro voltar a avançar a opção (o menu fica fechado e sai um `SetParam`).
#[test]
fn a_click_on_the_name_opens_the_list_and_writes_nothing() {
    crate::snapshot::set_card_choices(vec![(
        7,
        "mode",
        crate::CardChoices::Static(&["Sine", "Triangle", "Square"]),
        0,
    )]);
    let snap = card(vec![enum_param(0.0, &["Sine", "Triangle", "Square"])]);
    let (st, saiu) = click_zone(&snap, 0, crate::geom::RowZone::Centre);
    assert!(
        crate::menu_is_open(&st),
        "o clique no nome tem de abrir a lista"
    );
    assert!(
        saiu.is_empty(),
        "abrir uma lista nao escreve nada: {saiu:?}"
    );
    crate::snapshot::set_card_choices(Vec::new());
}

/// ⭐⭐⭐ **E ESCOLHER UMA LINHA DA LISTA ESCREVE ESSA OPÇÃO** — a segunda metade do dropdown, e
/// a que o gate de abertura não alcança.
///
/// ⚠️ **O clique na linha chega como `Background`, e isso é o mecanismo**: com o menu aberto o
/// painel regista um escudo de canvas inteiro por cima de tudo, para uma linha desenhada sobre
/// um cartão não ser comida pelo cartão. Um teste que mandasse o gesto como `ParamRow` estaria a
/// testar um caminho que o produto não usa.
///
/// FALSIFICADO por o braço de `ParamOptions` do `resolve_menu` não escrever (o artista escolhe e
/// nada muda — o defeito mais caro de um dropdown, porque a lista fecha e parece que funcionou).
#[test]
fn picking_a_line_of_the_list_writes_that_option() {
    use crate::geom::RowZone;
    crate::snapshot::set_card_choices(vec![(
        7,
        "mode",
        crate::CardChoices::Static(&["Sine", "Triangle", "Square"]),
        2,
    )]);
    let snap = card(vec![enum_param(2.0, &["Sine", "Triangle", "Square"])]);
    let (mut st, _) = click_zone(&snap, 0, RowZone::Centre);
    let linha = crate::first_menu_row(&st, RECT).expect("a lista abriu com linhas");
    let _ = drain_intents();
    super::apply_gesture(
        &mut st,
        gesture(
            GraphHitKind::Background,
            GesturePhase::Click,
            linha.x + linha.w * 0.5,
            linha.y + linha.h * 0.5,
        ),
        RECT,
        CENTER,
        &snap,
    );
    let saiu = drain_intents();
    let Some(GraphIntent::SetParam { value, param, .. }) = saiu.first() else {
        panic!("escolher a 1.a linha tem de escrever a 1.a opcao, e saiu {saiu:?}");
    };
    assert_eq!(*param, "mode");
    assert_eq!(*value, 0.0, "a 1.a linha e' `Sine`, que e' o indice 0");
    assert!(!crate::menu_is_open(&st), "e a lista fecha-se ao escolher");
    crate::snapshot::set_card_choices(Vec::new());
}

/// ⛔ **SEM OPÇÕES PUBLICADAS, O CENTRO NÃO ABRE NADA** — e é a resposta certa: uma lista vazia
/// diria que não há o que escolher quando o que houve foi a shell não ter publicado, e um popup
/// vazio ainda come o clique seguinte para se fechar.
#[test]
fn the_name_opens_nothing_when_the_shell_published_no_options() {
    crate::snapshot::set_card_choices(Vec::new());
    let snap = card(vec![enum_param(0.0, &["Sine", "Triangle", "Square"])]);
    let (st, saiu) = click_zone(&snap, 0, crate::geom::RowZone::Centre);
    assert!(!crate::menu_is_open(&st), "sem lista publicada, nada abre");
    assert!(saiu.is_empty(), "e nada se escreve: {saiu:?}");
}
