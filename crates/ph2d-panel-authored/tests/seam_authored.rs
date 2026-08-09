//! Seam do painel **AUTORADO** (plano UI/UX W8b.2) — as rows estão vivas sob o MOUSE.
//!
//! O gesto é REAL (Down+Up sobre o retângulo que o painel pintou), e não um `WidgetEvent::Click`
//! sintético: o sintético prova a allowlist e **pula a checagem de focabilidade no store** — a
//! lacuna que já deixou as 36 células da matriz de física e os dez chips do impasto *pintados,
//! hit-registrados e mortos sob o ponteiro*.
//!
//! ⚠️ **E este seam é o único gate desta crate que pode falar de *clicável*:** o `is_focusable` do
//! dispatch é privado, e reimplementá-lo num unit test seria a segunda resposta à pergunta que o
//! dispatcher já responde.

use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{Panel, PanelHostInternal};
use ph2d_editor_core::widget::WidgetKind;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_authored::state::AuthoredPanelState;
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

/// Um host com o painel aberto. ⚠️ `with_panel` é o construtor que RODA o `populate` — o
/// `MockPanelHost::new()` o pula, e um gate escrito sobre ele fica verde com os widgets mortos sob
/// o mouse (o defeito que a lista de dez ferramentas do impasto já pagou).
fn host() -> (MockPanelHost, AuthoredPanelState) {
    let mut h = MockPanelHost::with_panel::<AuthoredPanel>();
    h.set_panel_visible(AuthoredPanel::ID, true);
    (h, AuthoredPanelState)
}

/// **Toda row que RESPONDE é pintada com área e vive sob o ponteiro.**
///
/// ⚠️ É a metade que os unit gates não alcançam. E ela varre a tabela GERADA — a lista que o
/// artista desenhou —, então uma row nova nasce coberta em vez de precisar de alguém que lembre
/// de a acrescentar a um segundo lugar.
#[test]
fn every_control_row_is_alive_under_a_real_pointer() {
    let mut checked = 0;
    for row in rows::rows() {
        if !row.is_control() {
            continue;
        }
        let _ = drain_intents();
        let (mut h, mut st) = host();
        let r = h
            .painted_rect::<AuthoredPanel>(&mut st, VIEWPORT, row.id)
            .unwrap_or_else(|| panic!("a row `{}` nao foi PINTADA com area clicavel", row.key));
        let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        // ⚠️ Os eventos das DUAS fases, e a razão é uma lição do próprio gate: um `Button` fala no
        // **Up** e um `Slider` fala no **Down** (ele já pousa o valor onde o dedo caiu). Enumerar
        // uma fase passaria para a família que fala nela e reprovaria a outra — que foi
        // exactamente o que este gate fez na primeira corrida, sobre um painel correto.
        let mut evs = h.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
        evs.extend(h.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100)));
        assert!(
            !evs.is_empty(),
            "o ponteiro sobre a row `{}` nao virou evento nenhum — ela esta' desenhada e nao \
             existe para o dispatcher (falta o `register` no populate)",
            row.key
        );
        checked += 1;
    }
    // Controle positivo: uma tabela sem controles tornaria este gate verde por vácuo.
    assert!(
        checked > 0,
        "a tabela gerada nao tem nenhuma row que responda — este gate deixou de medir algo"
    );
}

/// **E a que só DESENHA não é clicável.**
///
/// ⚠️ A metade oposta, e ela é o que separa *"esta row responde"* de *"esta row existe"*: um painel
/// que registasse tudo daria uma `ColorSwatch` que acende sob o rato e não faz nada.
///
/// ⚠️ **A pergunta é ao `wants_pointer`, e não ao `is_control` — a premissa antiga MORREU.** Ela
/// dizia *"não-controle ⇒ não clicável"*, e o colapso de seção a tornou falsa: um cabeçalho não é
/// um controle (não tem valor, não emite intent) **e é clicável**, porque dobrar é um gesto de
/// vista. Manter o `is_control` aqui teria sido pedir que a seção deixasse de dobrar para o gate
/// ficar verde — o oposto da cura.
#[test]
fn a_display_only_row_is_not_clickable() {
    let Some(row) = rows::rows().iter().find(|r| !r.wants_pointer()) else {
        panic!("a tabela gerada perdeu a row de desenho puro — o CONTROLE deste gate");
    };
    let (mut h, mut st) = host();
    assert!(
        h.painted_rect::<AuthoredPanel>(&mut st, VIEWPORT, row.id)
            .is_none(),
        "a row `{}` so' desenha e publicou retangulo de hit",
        row.key
    );
}

/// **O CABEÇALHO é clicável, e o clique DOBRA a seção — sumindo com as rows dela.**
///
/// ⚠️ Três coisas num gate porque as três falham separadas e o produto só funciona com as três: o
/// retângulo de hit (senão o chevron desenha e o rato não o alcança), a MARCA de seção colapsável
/// (senão o clique chega e o despacho não sabe que aquele id dobra — o caso mais enganoso, porque
/// tudo parece ligado), e as rows a sumirem de facto.
///
/// ⚠️ E o cabeçalho **continua pintado** dobrado: ele é a alça de volta. Um colapso que escondesse
/// o próprio cabeçalho seria uma seção que o artista fecha e não consegue reabrir.
#[test]
fn clicking_a_section_header_folds_the_rows_under_it() {
    let Some(header) = rows::rows().iter().find(|r| r.folds_a_section()) else {
        panic!("a tabela gerada perdeu o cabecalho de secao — o SUJEITO deste gate");
    };
    let Some(under) = rows::rows()
        .iter()
        .skip_while(|r| r.id != header.id)
        .find(|r| r.wants_pointer() && r.id != header.id)
    else {
        panic!("nao ha row clicavel sob o cabecalho — o gate mediria o vazio");
    };

    let (mut h, mut st) = host();
    let hr = h
        .painted_rect::<AuthoredPanel>(&mut st, VIEWPORT, header.id)
        .expect("o cabecalho nao publicou retangulo de hit — o chevron desenha e nao dobra");
    assert!(
        h.painted_rect::<AuthoredPanel>(&mut st, VIEWPORT, under.id)
            .is_some(),
        "a row sob o cabecalho ja' estava escondida antes do clique"
    );

    let (cx, cy) = (hr.x + hr.w * 0.5, hr.y + hr.h * 0.5);
    h.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    h.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));

    assert!(
        h.painted_rect::<AuthoredPanel>(&mut st, VIEWPORT, header.id)
            .is_some(),
        "o cabecalho sumiu com a propria secao — nao ha alca de volta"
    );
    assert!(
        h.painted_rect::<AuthoredPanel>(&mut st, VIEWPORT, under.id)
            .is_none(),
        "a secao nao dobrou: a row `{}` continua pintada",
        under.key
    );
}

/// **DOBRAR ENCOLHE a altura reportada** — a propriedade de que o scrollbar depende.
///
/// ⚠️ Este gate não é redundante com o de cima, e a diferença é o modo de falha: uma implementação
/// que escondesse as rows **e continuasse a avançar o `y`** passaria lá (elas não são pintadas) e
/// cairia aqui — o painel ficaria com um buraco do tamanho da seção e um scrollbar a prometer
/// conteúdo que não existe. É a lição do `the_reported_height_is_the_true_height_of_the_rows`: uma
/// altura que não é a das rows convence o painel de coisas que não são verdade.
#[test]
fn folding_a_section_shrinks_the_reported_height() {
    let Some(header) = rows::rows().iter().find(|r| r.folds_a_section()) else {
        panic!("a tabela gerada perdeu o cabecalho de secao");
    };
    let (mut h, mut st) = host();
    let hr = h
        .painted_rect::<AuthoredPanel>(&mut st, VIEWPORT, header.id)
        .expect("o cabecalho nao publicou retangulo de hit");
    let open_h = h
        .store()
        .panel_content_h(ids::AUTHORED_PANEL)
        .expect("o painel nao publicou a altura do conteudo");

    let (cx, cy) = (hr.x + hr.w * 0.5, hr.y + hr.h * 0.5);
    h.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    h.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
    h.painted_rect::<AuthoredPanel>(&mut st, VIEWPORT, header.id);
    let folded_h = h
        .store()
        .panel_content_h(ids::AUTHORED_PANEL)
        .expect("o painel nao publicou a altura do conteudo");

    assert!(
        folded_h < open_h,
        "dobrar nao encolheu o conteudo ({open_h} -> {folded_h}) — as rows sumiram e o `y` \
         continuou a andar, deixando um buraco e um scrollbar a mentir"
    );
}

/// **O X do painel está vivo sob o ponteiro** — e fechar por ele apaga o chip da seção Frame,
/// porque os dois escrevem a MESMA visibilidade.
#[test]
fn the_close_button_is_alive_and_hides_the_panel() {
    let (mut h, mut st) = host();
    let r = h
        .painted_rect::<AuthoredPanel>(&mut st, VIEWPORT, ids::AUTHORED_CLOSE)
        .expect("o X nao foi pintado com area clicavel");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    h.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    let evs = h.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
    assert!(
        evs.iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) if *c == ids::AUTHORED_CLOSE)),
        "o ponteiro sobre o X nao virou Click"
    );
    for ev in evs {
        AuthoredPanel::apply_event(&mut st, &mut h, ev);
    }
    assert!(
        !h.panel_visible(AuthoredPanel::ID),
        "o X nao fechou o painel"
    );
}

/// **Um SLIDER desenhado numa row responde ao ARRASTO** — e é aqui que a fronteira do §2 é
/// verificada em vez de prometida.
///
/// ⚠️ Nada nesta crate implementa *drag*: o valor muda porque o dispatch genérico do catálogo o
/// muda. Se alguém "consertasse" isto com um cálculo local, o gate continuaria verde — e é por
/// isso que ele afirma o VALOR que o store carrega, que é o mesmo lugar de onde o `paint` lê.
#[test]
fn dragging_a_slider_row_moves_the_value_the_paint_reads() {
    let Some(row) = rows::rows().iter().find(|r| r.kind == WidgetKind::Slider) else {
        return;
    };
    let (mut h, mut st) = host();
    let r = h
        .painted_rect::<AuthoredPanel>(&mut st, VIEWPORT, row.id)
        .expect("a row de slider nao foi pintada com area clicavel");
    let before = h.store().slider(row.id).map(|(_, v)| v).unwrap_or(0.0);
    let cy = r.y + r.h * 0.5;
    h.dispatch_pointer_event(pointer(PointerKind::Down, r.x + r.w * 0.1, cy, SEC));
    h.dispatch_pointer_event(pointer(
        PointerKind::Move,
        r.x + r.w * 0.9,
        cy,
        SEC + SEC / 100,
    ));
    h.dispatch_pointer_event(pointer(
        PointerKind::Up,
        r.x + r.w * 0.9,
        cy,
        SEC + SEC / 50,
    ));
    let after = h.store().slider(row.id).map(|(_, v)| v).unwrap_or(0.0);
    assert!(
        after > before,
        "arrastar o slider da row `{}` nao moveu o valor ({before} -> {after}) — o comportamento \
         do catalogo nao esta' a chegar a' row",
        row.key
    );
}
