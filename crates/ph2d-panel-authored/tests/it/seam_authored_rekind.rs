//! Seam da **TROCA DE TIPO** — o estado no store segue o tipo que a row veste AGORA.
//!
//! ⚠️ **Isto não é um caso de canto: é o caminho NORMAL de toda row que o artista cria.**
//! `WidgetEdit::Wear` insere `WidgetKind::Button` (o `vec_widget_edit.rs` di-lo em código), e só
//! depois o artista escolhe o tipo que queria. Então todo controle autorado passa por Button, e o
//! id dele entra no store como Button antes de ser o que é.
//!
//! ⚠️ **O report que trouxe isto tinha DUAS metades e uma causa** (Enio, 2026-08-09): *"o checkbox
//! que criei não fica checado mas EMITE SINAL; o que você criou não emite sinal mas pode ser
//! checado"*. As duas saem do mesmo estado obsoleto — o `paint` lê um `Button` onde devia ler um
//! `Checkbox` (logo não há o que marcar), e o `event` cai no `else` final da cadeia, que é o
//! `Fired` dos BOTÕES — e `Fired` é o único intent que vira **sinal**.
//!
//! ⚠️ E a metade "o meu funciona" é o CONTROLE que nomeia a causa: a row da tabela compilada foi
//! registada pelo `populate` com o tipo certo, no arranque, e nunca trocou.

use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::{Panel, PanelHostInternal};
use ph2d_editor_core::widget::{CheckboxValue, WidgetKind};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_authored::state::{AuthoredIntent, AuthoredPanelState};
use ph2d_panel_authored::{AuthoredPanel, drain_intents, ids, rows};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};
const SEC: u128 = 1_000_000_000;

fn pointer(kind: PointerKind, x: f32, y: f32, t: u128) -> PointerEvent {
    PointerEvent {
        kind,
        x,
        y,
        button: PointerButton::Primary,
        source: PointerSource::Mouse,
        pressure: 1.0,
        timestamp_ns: t,
    }
}

fn row(key: &str, kind: WidgetKind) -> rows::Row {
    rows::Row {
        kind,
        label: key.to_string(),
        key: key.to_string(),
        id: ids::authored_row_id(key),
        rgba: None,
        icon: None,
        icon_id: None,
        options: Vec::new(),
    }
}

/// O host **na ordem do app**, com a row já a ter passado por Button.
///
/// ⚠️ A passagem por Button é encenada publicando a tabela viva DUAS vezes: primeiro como o
/// `Wear` a deixa, depois como o artista a quis. É o gesto real, e não um estado arrumado à mão —
/// um `store.register(id, Button)` escrito aqui provaria a mesma coisa por um caminho que o
/// produto não tem.
fn worn_then_rekinded(kind: WidgetKind) -> (MockPanelHost, AuthoredPanelState) {
    rows::set_live_rows(None);
    let mut h = MockPanelHost::with_panel::<AuthoredPanel>();
    h.set_panel_visible(AuthoredPanel::ID, true);
    let mut st = AuthoredPanelState;
    // 1. O artista aperta "Wear a Widget": a forma vira um Button, e o painel pinta-a.
    rows::set_live_rows(Some(vec![row("mine", WidgetKind::Button)]));
    let _ = h.paint::<AuthoredPanel>(&mut st, VIEWPORT);
    // 2. Ele escolhe o tipo que queria.
    rows::set_live_rows(Some(vec![row("mine", kind)]));
    (h, st)
}

/// **Um checkbox autorado MARCA, e o intent dele é o de um checkbox.**
///
/// ⚠️ Gate red-first: com o `register_if_absent` sozinho ele falha nas DUAS asserções, e cada uma
/// é uma metade do report — a que MARCA e a que diz que o clique não é um disparo de botão.
#[test]
fn a_checkbox_that_was_born_a_button_still_checks() {
    let (mut h, mut st) = worn_then_rekinded(WidgetKind::Checkbox);
    let _ = drain_intents();
    let r = h
        .painted_rect::<AuthoredPanel>(&mut st, VIEWPORT, ids::authored_row_id("mine"))
        .expect("a row nao foi pintada");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    h.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    for ev in h.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100)) {
        AuthoredPanel::apply_event(&mut st, &mut h, ev);
    }

    assert!(
        matches!(
            h.store().get(ids::authored_row_id("mine")),
            Some(InteractiveState::Checkbox { .. })
        ),
        "o store guardou o tipo ANTIGO: a row veste Checkbox e o estado dela continua o do Button \
         que o `Wear` criou — {:?}",
        h.store().get(ids::authored_row_id("mine"))
    );
    assert_eq!(
        h.store()
            .checkbox(ids::authored_row_id("mine"))
            .map(|c| c.1),
        Some(CheckboxValue::Checked),
        "o clique chegou e a caixa nao ficou marcada"
    );
    let intents = drain_intents();
    assert!(
        intents
            .iter()
            .any(|i| matches!(i, AuthoredIntent::Flag { .. })),
        "o clique nao virou um `Flag` — {intents:?}"
    );
    assert!(
        !intents
            .iter()
            .any(|i| matches!(i, AuthoredIntent::Fired { .. })),
        "o clique virou `Fired`, o intent dos BOTOES — e `Fired` e' o unico que vira SINAL. E' a \
         cadeia do `apply_event` a cair no `else` final porque `store.checkbox` disse `None`"
    );
    rows::set_live_rows(None);
}

/// **O caminho INVERSO: quem foi cabeçalho e virou controle não dobra nada.**
///
/// ⚠️ Uma seção é MARCADA no store (`mark_collapsible_section`) e o despacho trata a marca **antes**
/// do switch de widget — então uma marca que sobrevive à troca de tipo faz o clique DOBRAR uma
/// seção que já não existe, em vez de marcar a caixa. É a mesma doença do gate acima pelo outro
/// lado: o store a lembrar-se do que a row costumava ser.
#[test]
fn a_control_that_was_born_a_section_header_no_longer_folds() {
    rows::set_live_rows(None);
    let mut h = MockPanelHost::with_panel::<AuthoredPanel>();
    h.set_panel_visible(AuthoredPanel::ID, true);
    let mut st = AuthoredPanelState;
    rows::set_live_rows(Some(vec![row("mine", WidgetKind::SectionHeader)]));
    let _ = h.paint::<AuthoredPanel>(&mut st, VIEWPORT);
    rows::set_live_rows(Some(vec![row("mine", WidgetKind::Checkbox)]));
    let _ = drain_intents();

    let r = h
        .painted_rect::<AuthoredPanel>(&mut st, VIEWPORT, ids::authored_row_id("mine"))
        .expect("a row nao foi pintada");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    h.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    for ev in h.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100)) {
        AuthoredPanel::apply_event(&mut st, &mut h, ev);
    }
    assert_eq!(
        h.store()
            .checkbox(ids::authored_row_id("mine"))
            .map(|c| c.1),
        Some(CheckboxValue::Checked),
        "o clique nao marcou a caixa — a marca de SEÇÃO sobreviveu a' troca de tipo e o despacho \
         dobrou uma secao que ja' nao existe"
    );
    rows::set_live_rows(None);
}

/// **E o TERCEIRO canal que sobrevive à troca de tipo: o alvo do PICKER.**
///
/// ⚠️ Esta é a irmã exata da marca de seção, e foi introduzida a consertar aquela: o `adopt`
/// REGISTA a swatch como alvo de picker e **nunca a retira**, então uma row que vestiu
/// `ColorSwatch` um momento atrás carrega a marca para sempre.
///
/// ⚠️ **O modo de falha é pior que o da seção**, e é por onde o `pointer_down` está escrito: o
/// ramo do picker **corta o Down inteiro** (`return`) antes de o foco ser computado, logo antes de
/// o widget existir para o despacho. O artista clica na caixa, o **picker OKLCH abre por cima**, e
/// o que ele escolher não pinta nada — o `picker_shape` procura uma swatch naquela moldura e não
/// acha nenhuma, porque a row já não é uma. Um modal que se abre sobre o controle errado e não faz
/// nada.
///
/// ⚠️ E ele **não se cura sozinho**: `picker_swatch_ids` não tinha porta de saída nenhuma no
/// codebase inteiro — o id fica no conjunto pelo resto da sessão.
#[test]
fn a_control_that_was_born_a_colour_swatch_no_longer_opens_the_picker() {
    rows::set_live_rows(None);
    let mut h = MockPanelHost::with_panel::<AuthoredPanel>();
    h.set_panel_visible(AuthoredPanel::ID, true);
    let mut st = AuthoredPanelState;
    let mut swatch = row("mine", WidgetKind::ColorSwatch);
    swatch.rgba = Some([0x20, 0x80, 0xC0, 0xFF]);
    rows::set_live_rows(Some(vec![swatch]));
    let _ = h.paint::<AuthoredPanel>(&mut st, VIEWPORT);
    rows::set_live_rows(Some(vec![row("mine", WidgetKind::Checkbox)]));
    let _ = drain_intents();

    let r = h
        .painted_rect::<AuthoredPanel>(&mut st, VIEWPORT, ids::authored_row_id("mine"))
        .expect("a row nao foi pintada");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    h.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    for ev in h.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100)) {
        AuthoredPanel::apply_event(&mut st, &mut h, ev);
    }

    assert!(
        h.store().picker_target().is_none(),
        "o Down abriu o picker de cor sobre um CHECKBOX — a marca de swatch sobreviveu a' troca \
         de tipo, e o ramo do picker corta o Down antes de o widget sequer ser considerado"
    );
    assert_eq!(
        h.store()
            .checkbox(ids::authored_row_id("mine"))
            .map(|c| c.1),
        Some(CheckboxValue::Checked),
        "o clique nao marcou a caixa"
    );
    rows::set_live_rows(None);
}

/// **A família inteira, e não só o checkbox** — o Enio pediu-o ao dizer *"o Toggle está igual"*.
///
/// ⚠️ Uma lista de tipos escrita à mão apodreceria no dia do próximo; esta varre o CATÁLOGO e
/// pergunta a cada tipo que responde. O oráculo é a `discriminant` — a mesma pergunta que a cura
/// faz —, mas ele é aplicado ao que o produto DEIXOU no store depois do gesto, não ao que a cura
/// calcula.
#[test]
fn every_control_kind_survives_being_born_a_button() {
    let mut checked = 0;
    for kind in WidgetKind::ALL {
        let probe = row("mine", kind);
        if !probe.is_control() || kind == WidgetKind::Button {
            continue;
        }
        let (mut h, mut st) = worn_then_rekinded(kind);
        let _ = h.paint::<AuthoredPanel>(&mut st, VIEWPORT);
        let got = h.store().get(ids::authored_row_id("mine"));
        // O que este tipo PEDE, perguntado a um store limpo pelo mesmo caminho do produto.
        rows::set_live_rows(None);
        let mut fresh = MockPanelHost::with_panel::<AuthoredPanel>();
        fresh.set_panel_visible(AuthoredPanel::ID, true);
        rows::set_live_rows(Some(vec![row("mine", kind)]));
        let mut st2 = AuthoredPanelState;
        let _ = fresh.paint::<AuthoredPanel>(&mut st2, VIEWPORT);
        let want = fresh.store().get(ids::authored_row_id("mine"));
        assert_eq!(
            got.map(std::mem::discriminant),
            want.map(std::mem::discriminant),
            "o tipo `{kind:?}` ficou com o estado do Button que o `Wear` criou"
        );
        checked += 1;
        rows::set_live_rows(None);
    }
    assert!(
        checked >= 10,
        "a varredura olhou {checked} tipos — o catalogo encolheu ou o filtro comeu-o"
    );
}

/// **Uma row que MORREU não deixa o valor dela para a próxima com o mesmo nome.**
///
/// ⚠️ O id de uma row é `authored_row_id(key_of(nome))` — **função do RÓTULO, não da forma**. E
/// nada retirava uma entrada do `WidgetStore`: `register`/`register_if_absent` só inserem. Então
/// apagar a forma `Speed` e criar outra chamada `Speed` do mesmo tipo dava um controle que **nasce
/// na posição do morto** — o `adopt` vê o discriminante bater e PRESERVA o estado.
///
/// ⚠️ **E o store é do APP, não do documento:** nada no load o limpa, então a herança atravessa
/// projetos.
///
/// ⚠️ **O painel não consegue distinguir *"a mesma forma"* de *"outra com o mesmo nome"*** — o id é
/// o mesmo nos dois casos. O único discriminador que existe é a AUSÊNCIA: a row sumiu da tabela
/// viva por um quadro. É por isso que a cura é retirar no desaparecimento, e não comparar nomes.
///
/// ⚠️ **E retirar é sem perda:** o valor durável mora no `VecWidgetValue` da ENTIDADE, e o
/// `reconcile` re-semeia o store na primeira vista. Delete+Ctrl+Z devolve a forma *e* o valor; o
/// que não volta é o valor de um objeto que o artista apagou de propósito.
#[test]
fn a_row_that_died_does_not_leave_its_value_to_the_next_one_of_the_same_name() {
    rows::set_live_rows(None);
    let mut h = MockPanelHost::with_panel::<AuthoredPanel>();
    h.set_panel_visible(AuthoredPanel::ID, true);
    let mut st = AuthoredPanelState;

    // 1. A forma `Speed` veste um Toggle, e o artista o LIGA.
    rows::set_live_rows(Some(vec![row("Speed", WidgetKind::Toggle)]));
    let r = h
        .painted_rect::<AuthoredPanel>(&mut st, VIEWPORT, ids::authored_row_id("Speed"))
        .expect("a row nao foi pintada");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    h.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    for ev in h.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100)) {
        AuthoredPanel::apply_event(&mut st, &mut h, ev);
    }
    assert_eq!(
        h.store().toggle(ids::authored_row_id("Speed")).map(|t| t.1),
        Some(true),
        "o gesto nao ligou o toggle — a premissa do gate nao vale"
    );

    // 2. O artista APAGA a forma: ela some da tabela viva.
    rows::set_live_rows(Some(Vec::new()));
    let _ = h.paint::<AuthoredPanel>(&mut st, VIEWPORT);

    // 3. Outra forma, nome igual, tipo igual — um objeto NOVO.
    rows::set_live_rows(Some(vec![row("Speed", WidgetKind::Toggle)]));
    let _ = h.paint::<AuthoredPanel>(&mut st, VIEWPORT);

    assert_eq!(
        h.store().toggle(ids::authored_row_id("Speed")).map(|t| t.1),
        Some(false),
        "o controle NOVO nasceu ligado — ele herdou a posicao de um controle que o artista apagou, \
         e o store guarda isso pelo resto da sessao, atravessando ate' a troca de projeto"
    );
    rows::set_live_rows(None);
}
