//! Seam da **SEGUNDA SUPERFÍCIE** — a lista de um dropdown aberto.
//!
//! # Por que este arquivo existe ao lado do `seam_authored`
//!
//! Aquele responde *"toda row que responde está viva sob o rato?"*, e a resposta dele é uma
//! varredura da tabela. Este responde outra coisa: **uma superfície que não é uma row** — ela não
//! está na tabela, não tem `key`, nasce de um clique e morre no seguinte, e é pintada num passe
//! que corre **depois** do recorte que contém todas as rows.
//!
//! ⚠️ **É o único controle do catálogo que um painel não consegue pintar num passe só**
//! ([`WidgetKind::defers_a_popover`]), e é por isso que ele tem gates próprios: o modo de falha
//! dele não é *"a row não responde"*, é *"a lista aparece e o clique nela cai na row por baixo"*.

use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::{Panel, PanelHostInternal};
use ph2d_editor_core::widget::{DropdownState, WidgetKind};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_authored::state::AuthoredPanelState;
use ph2d_panel_authored::{AuthoredIntent, AuthoredPanel, drain_intents, ids, rows};
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

fn host() -> (MockPanelHost, AuthoredPanelState) {
    let mut h = MockPanelHost::with_panel::<AuthoredPanel>();
    h.set_panel_visible(AuthoredPanel::ID, true);
    (h, AuthoredPanelState)
}

/// A row de dropdown da tabela gerada, com a chave e as opções que ela POSSUI.
///
/// ⚠️ Ela vem da tabela e não de uma fixture escrita à mão: um gate que montasse o seu próprio
/// dropdown provaria que o pintor sabe pintar um, e não que **o painel que o artista desenhou** o
/// mostra. É a mesma razão pela qual o `seam_authored` varre a tabela em vez de listar tipos.
fn the_dropdown() -> (ph2d_a11y::NodeId, String, Vec<String>) {
    let r = rows::baked()
        .iter()
        .find(|r| r.kind == WidgetKind::Dropdown)
        .expect("a tabela gerada perdeu o dropdown — o SUJEITO deste arquivo");
    (r.id, r.key.clone(), r.options.clone())
}

fn open_it(h: &mut MockPanelHost, id: ph2d_a11y::NodeId, index: usize) {
    h.store_mut().register(
        id,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: true,
            selected_index: Some(index),
        },
    );
}

/// **A lista SÓ existe aberta, e as opções dela vivem sob o rato.**
///
/// ⚠️ **As duas metades são o gate**, e cada uma sozinha é verde sobre um produto quebrado: sem a
/// primeira, um painel que pintasse a lista SEMPRE passaria (as opções estariam lá); sem a
/// segunda, o passe diferido inteiro pode ter sido apagado e o teste não sabe.
///
/// ⚠️ **E o oráculo da posição é *fora do chip*, e não um número.** A lista pende do chip, e para
/// onde ela pende depende do espaço na tela — cravar *"abaixo"* aqui seria pedir que ela nunca
/// virasse para cima, que é precisamente o que o gate irmão exige que ela faça.
#[test]
fn an_open_list_puts_its_options_under_the_pointer_and_a_closed_one_does_not() {
    let (id, key, options) = the_dropdown();
    assert!(
        options.len() >= 2,
        "o dropdown da cena tem menos de duas opcoes — o gate nao mediria uma LISTA"
    );

    let (mut h, mut st) = host();
    for i in 0..options.len() {
        assert!(
            h.painted_rect::<AuthoredPanel>(&mut st, VIEWPORT, ids::authored_option_id(&key, i))
                .is_none(),
            "a opcao {i} tem retangulo de hit com a lista FECHADA — a lista esta' sempre pintada"
        );
    }

    let (mut h, mut st) = host();
    open_it(&mut h, id, 0);
    let chip = h
        .painted_rect::<AuthoredPanel>(&mut st, VIEWPORT, id)
        .expect("o chip do dropdown nao foi pintado");
    for i in 0..options.len() {
        let r = h
            .painted_rect::<AuthoredPanel>(&mut st, VIEWPORT, ids::authored_option_id(&key, i))
            .unwrap_or_else(|| {
                panic!(
                    "a opcao {i} nao publicou retangulo de hit com a lista ABERTA — ela esta' \
                     desenhada e nao existe para o dispatcher"
                )
            });
        assert!(
            r.y + r.h <= chip.y + 0.5 || r.y >= chip.y + chip.h - 0.5,
            "a opcao {i} ({r:?}) esta' POR CIMA do chip ({chip:?}) — o clique nela cairia no chip"
        );
    }
}

/// **Escolher uma opção MARCA-a e FECHA a lista — num gesto só.**
///
/// ⚠️ **Fechar é a metade que o despacho genérico não faz**, e é a que se sente: ele alterna o
/// `open` de quem foi CLICADO, e quem foi clicado é a opção, não o chip. Sem ela a lista fica
/// aberta depois da escolha, e o clique que o artista dá para a fechar escolhe outra coisa.
///
/// ⚠️ E o gesto é um ponteiro REAL sobre o retângulo que o painel pintou — o `WidgetEvent`
/// sintético pula a checagem de focabilidade e ficaria verde sobre uma opção morta sob o rato.
#[test]
fn choosing_an_option_marks_it_and_closes_the_list() {
    let (id, key, options) = the_dropdown();
    let pick = options.len() - 1;

    let _ = drain_intents();
    let (mut h, mut st) = host();
    open_it(&mut h, id, 0);
    let r = h
        .painted_rect::<AuthoredPanel>(&mut st, VIEWPORT, ids::authored_option_id(&key, pick))
        .expect("a opcao escolhida nao tem retangulo de hit");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let mut evs = h.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    evs.extend(h.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100)));
    assert!(
        !evs.is_empty(),
        "o ponteiro sobre a opcao nao virou evento nenhum — ela esta' desenhada e nao existe para \
         o dispatcher (falta o `register` no populate)"
    );
    // ⚠️ O despacho DEVOLVE os eventos; quem os leva ao painel é a shell. Um harness que parasse
    // na linha acima provaria que a opção está viva sob o rato e **não** que o painel faz alguma
    // coisa com o clique — as duas metades que este gate precisa de ter juntas.
    for ev in evs {
        AuthoredPanel::apply_event(&mut st, &mut h, ev);
    }

    assert!(
        matches!(
            h.store().get(id),
            Some(InteractiveState::Dropdown {
                open: false,
                selected_index: Some(i),
                ..
            }) if *i == pick
        ),
        "escolher a opcao {pick} nao a marcou e/ou nao fechou a lista: {:?}",
        h.store().get(id)
    );
    assert_eq!(
        drain_intents(),
        vec![AuthoredIntent::Choice { key, index: pick }],
        "a escolha nao virou o intent que a nomeia"
    );
}

/// **Um dropdown SEM opções continua um controle VIVO** — o chip está lá, e não há opção nenhuma.
///
/// ⚠️ **É o estado em que todo dropdown NASCE**, e é por isso que ele tem gate: o artista põe o
/// controle na moldura e só depois lhe dá filhos. Entre um gesto e o outro o painel tem de mostrar
/// alguma coisa em que se possa clicar — sem o chip, o controle recém-criado seria invisível, e
/// dar-lhe as opções que o tornariam visível exigiria seleccioná-lo.
///
/// ⚠️ **O que este gate NÃO prova é a guarda de pintura** (`paint_open_list` sai cedo com zero
/// opções): a metade `is_none` abaixo passa **também** quando o passe diferido inteiro é apagado,
/// porque não há opções a registar de qualquer maneira. Isso está medido — a mutação que remove a
/// guarda sobrevive à suíte —, e o efeito dela é só visual, num harness que lê retângulos de hit e
/// nunca a cena. A guarda fica declarada no `paint.rs`, não fingida coberta aqui.
///
/// ⚠️ **A fixture vem da tabela VIVA**, que é o canal que a autoria já usa: o artista que acabou de
/// pôr um dropdown na moldura e ainda não lhe deu filhos está exactamente neste estado.
#[test]
fn a_dropdown_without_options_is_still_a_live_control() {
    let (id, key, _) = the_dropdown();
    rows::set_live_rows(Some(vec![rows::Row {
        kind: WidgetKind::Dropdown,
        label: "Blend".into(),
        key: key.clone(),
        id,
        rgba: None,
        icon: None,
        icon_id: None,
        options: Vec::new(),
    }]));

    let (mut h, mut st) = host();
    open_it(&mut h, id, 0);
    let chip = h.painted_rect::<AuthoredPanel>(&mut st, VIEWPORT, id);
    let opt = h.painted_rect::<AuthoredPanel>(&mut st, VIEWPORT, ids::authored_option_id(&key, 0));
    rows::set_live_rows(None);

    assert!(
        chip.is_some(),
        "o chip sumiu — um dropdown recem-criado ficaria invisivel, e dar-lhe as opcoes que o          tornariam visivel exigiria seleciona-lo"
    );
    assert!(
        opt.is_none(),
        "um dropdown sem opcoes publicou retangulo de opcao — ha' uma lista vazia a comer cliques"
    );
}

/// **Perto do fundo da tela a lista vira para CIMA — e as opções vão com ela.**
///
/// ⚠️ **É o gate que separa duas funções que dão a mesma resposta no caso comum**, e é por isso que
/// nenhum dos outros o alcança: `option_rect` (que pendura a lista sempre ABAIXO) e `option_rect_in`
/// (que a pendura no painel que o pintor de facto desenhou) só divergem quando o chip está a menos
/// de uma lista do fundo. Sem esta altura de viewport, trocar uma pela outra passa em tudo — e o
/// artista fica com uma lista desenhada em cima e os cliques a caírem em baixo dela.
///
/// ⚠️ **Os números vêm de uma VARREDURA**, não de gosto: com 900 px e 600 px a lista desce, e a
/// partir de ~420 px ela vira. Uma fixture escolhida no olho ficaria do lado errado do joelho e o
/// gate mediria o caso que os irmãos já cobrem.
#[test]
fn a_list_near_the_bottom_flips_up_and_its_options_go_with_it() {
    const SHORT: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 1600.0,
        h: 420.0,
    };
    let (id, key, options) = the_dropdown();
    let (mut h, mut st) = host();
    open_it(&mut h, id, 0);
    let chip = h
        .painted_rect::<AuthoredPanel>(&mut st, SHORT, id)
        .expect("o chip do dropdown nao foi pintado");

    let mut last_y = f32::NEG_INFINITY;
    for i in 0..options.len() {
        let r = h
            .painted_rect::<AuthoredPanel>(&mut st, SHORT, ids::authored_option_id(&key, i))
            .unwrap_or_else(|| panic!("a opcao {i} nao publicou retangulo de hit"));
        assert!(
            r.y + r.h <= chip.y,
            "a lista virou para cima mas a opcao {i} ({r:?}) ficou ABAIXO do chip ({chip:?}) — os \
             retangulos de hit sairam da regra NAO-clampada e a lista desenhada esta' noutro lugar"
        );
        // A ordem tem de sobreviver à volta: a opção 0 continua a ser a de cima.
        assert!(r.y > last_y, "as opcoes sairam fora de ordem ao virar");
        last_y = r.y;
    }
}
