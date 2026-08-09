//! Gates do **roteamento** (plano UI/UX W8b.2) — o que uma row DIZ quando alguém a mexe.

use super::*;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{SliderState, ToggleState, WidgetKind};
use ph2d_ui_testkit::MockPanelHost;

use crate::state::{AuthoredIntent, drain_intents};

/// **PUBLICA** uma row do tipo pedido e devolve o `(id, chave)` dela.
///
/// ⚠️ **Ele PROCURAVA na tabela gerada, e os quatro gates abaixo saíam com `else { return; }`** —
/// se a cena que gerou o `panel.rs` não tivesse aquele tipo, o teste passava **sem testar nada**,
/// em silêncio. E a tabela gerada é conteúdo AUTORADO: o Enio pode reautorar o painel a qualquer
/// momento e deixar os quatro verdes sobre um barramento de intents que ninguém verifica.
///
/// ⚠️ **A cura não é um `expect`, é não depender da cena.** O irmão `seam_authored_popover` usa
/// `expect` porque o SUJEITO dele é a lista aberta *daquele* dropdown; aqui o sujeito é o
/// ROTEAMENTO, que não é sobre nenhuma row em particular. Publicar a tabela viva — o mesmo canal
/// que a autoria usa, e o que os outros seams desta crate já fazem — torna os gates
/// determinísticos **e** cobre tipos que a cena por acaso não tem.
///
/// ⚠️ Devolve os DADOS e não a `Row`: a tabela viva vive sob um `RefCell` thread-local, então uma
/// referência não escapa dela.
fn row_of(kind: WidgetKind) -> (ph2d_a11y::NodeId, String) {
    let key = "mode".to_string();
    let id = crate::ids::authored_row_id(&key);
    crate::rows::set_live_rows(Some(vec![crate::rows::Row {
        kind,
        label: key.clone(),
        key: key.clone(),
        id,
        rgba: None,
        icon: None,
        icon_id: None,
        options: vec!["A".into(), "B".into(), "C".into()],
    }]));
    (id, key)
}

/// Dirige um evento e devolve o que o painel enfileirou.
///
/// ⚠️ A fila é thread-local: sem o dreno de entrada um teste herdaria o intent do vizinho.
fn fire(store_edit: impl FnOnce(&mut WidgetStore), ev: WidgetEvent) -> Vec<AuthoredIntent> {
    let _ = drain_intents();
    let mut host = MockPanelHost::with_panel::<AuthoredPanel>();
    store_edit(host.store_mut());
    let mut st = AuthoredPanelState;
    apply_event(&mut st, &mut host, ev);
    // ⚠️ Devolve a tabela ao estado gerado DEPOIS de o evento ser roteado (o `apply_event` lê-a).
    // Não consegui fazer o vazamento aparecer — nem em seis corridas, nem sob `--test-threads=1`,
    // nem com `--nocapture` —, e a linha fica na mesma: a ausência de prova de vazamento não é
    // prova de ausência, e o preço dela é zero. O que ela garante é que um teste futuro nesta
    // mesma thread lê a tabela GERADA, e não a que este publicou.
    crate::rows::set_live_rows(None);
    drain_intents()
}

/// **Um slider arrastado diz QUAL chave mudou e PARA QUANTO.**
///
/// ⚠️ O valor é RELIDO do store, nunca carregado pelo evento — é o contrato do `WidgetEvent`, e é
/// o que garante que o número que sai daqui é o mesmo que o `paint` desenha no frame seguinte.
#[test]
fn a_slider_says_which_key_changed_and_to_what() {
    let (row_id, row_key) = row_of(WidgetKind::Slider);
    let out = fire(
        |s| {
            s.register(
                row_id,
                InteractiveState::Slider {
                    state: SliderState::Normal,
                    value: 0.42,
                    orientation: Default::default(),
                },
            );
        },
        WidgetEvent::ValueChanged(row_id),
    );
    assert_eq!(
        out,
        vec![AuthoredIntent::Value {
            key: row_key.clone(),
            value: 0.42,
        }]
    );
}

/// **Um toggle diz o estado que ele PASSOU a ter.**
#[test]
fn a_toggle_says_the_flag_it_now_carries() {
    let (row_id, row_key) = row_of(WidgetKind::Toggle);
    let out = fire(
        |s| {
            s.register(
                row_id,
                InteractiveState::Toggle {
                    state: ToggleState::Normal,
                    on: true,
                },
            );
        },
        WidgetEvent::Toggled(row_id),
    );
    assert_eq!(
        out,
        vec![AuthoredIntent::Flag {
            key: row_key.clone(),
            on: true,
        }]
    );
}

/// **Um botão DISPARA — sem valor, porque ele não tem nenhum.**
#[test]
fn a_button_fires_without_a_value() {
    let (row_id, row_key) = row_of(WidgetKind::Button);
    let out = fire(|_| {}, WidgetEvent::Click(row_id));
    assert_eq!(
        out,
        vec![AuthoredIntent::Fired {
            key: row_key.clone(),
        }]
    );
}

/// **O X fecha o painel** — a MESMA visibilidade que o interruptor da seção Frame lê.
#[test]
fn the_close_button_hides_the_panel() {
    let mut host = MockPanelHost::with_panel::<AuthoredPanel>();
    host.set_panel_visible(AuthoredPanel::ID, true);
    let mut st = AuthoredPanelState;
    apply_event(&mut st, &mut host, WidgetEvent::Click(ids::AUTHORED_CLOSE));
    assert!(
        !host.panel_visible(AuthoredPanel::ID),
        "o X nao fechou — o chip da secao Frame ficaria aceso sobre um painel escondido"
    );
}

/// **Um id que não é row nem chrome é IGNORADO.**
///
/// ⚠️ `Ignored` é o que deixa o evento seguir para o próximo handler; consumir tudo faria este
/// painel engolir cliques de quem está por baixo dele.
#[test]
fn an_id_that_is_not_a_row_is_ignored() {
    let _ = drain_intents();
    let mut host = MockPanelHost::with_panel::<AuthoredPanel>();
    let mut st = AuthoredPanelState;
    let out = apply_event(
        &mut st,
        &mut host,
        WidgetEvent::Click(ph2d_a11y::NodeId(0xDEAD_BEEF)),
    );
    assert_eq!(out, EventOutcome::Ignored);
    assert!(drain_intents().is_empty());
}

/// **A família de LISTA diz QUAL opção está marcada, não só que alguém mexeu.**
///
/// ⚠️ **Este gate nasce a fechar um vão que a wave anterior shipou:** as três primeiras da família
/// (`Tabs`, `RadioGroup`, `SegmentedAdaptive`) caíam em [`AuthoredIntent::Fired`] — o intent que
/// diz *"este controle foi accionado"* e **não** diz o que foi escolhido. Um ouvinte a ligar uma
/// faixa de abas a uma propriedade recebia o gesto sem a informação inteira que ele carrega.
///
/// ⚠️ E ele é dirigido pela **porta única** (`rows::selected_of`), a mesma que o `paint` lê para
/// desenhar a marcada — é isso que impede o controle de desenhar uma opção e devolver outra.
#[test]
fn a_list_control_says_which_option_is_marked() {
    let (row_id, row_key) = row_of(WidgetKind::Tabs);
    let out = fire(
        |s| s.register(row_id, InteractiveState::Tabs { selected: 2 }),
        WidgetEvent::Click(row_id),
    );
    assert_eq!(
        out,
        vec![AuthoredIntent::Choice {
            key: row_key,
            index: 2,
        }],
        "a faixa de abas emitiu um gesto sem dizer QUAL aba"
    );
}
