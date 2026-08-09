//! Seam das **SEÇÕES** do painel autorado — duas dobram, e cada uma manda só nas suas.
//!
//! ⚠️ **O colapso JÁ tinha dois gates** (`clicking_a_section_header_folds_the_rows_under_it` e o
//! irmão da altura, no `seam_authored`), e a primeira versão deste cabeçalho afirmava que não —
//! um over-claim que teria envelhecido como verdade. O que eles **não** conseguem cobrir é o que
//! mora aqui, e são duas coisas que a tabela COMPILADA torna inexprimíveis:
//!
//! 1. **DOIS cabeçalhos.** Aquele gate procura *o* cabeçalho da tabela gerada, e ela tem um só —
//!    com um cabeçalho sozinho, *"esconde as rows até o PRÓXIMO"* e *"esconde tudo abaixo"* dão a
//!    mesma foto. É preciso um segundo para as duas leis se separarem.
//! 2. **Uma row que o documento tem e o código colado não.** Os gates existentes varrem
//!    `rows::baked()`, cujos ids o `populate` registou no arranque — então eles nunca chegam ao
//!    caso do artista, que é desenhar uma seção a mais e clicá-la no mesmo minuto.
//!
//! ⚠️ E o gesto é REAL (Down+Up sobre o retângulo pintado), pela mesma razão do `seam_authored`:
//! um `WidgetEvent::Click` sintético pula a checagem de focabilidade no store, que é exactamente
//! a metade onde uma row autorada morre.

use ph2d_editor_core::panel::{Panel, PanelHostInternal};
use ph2d_editor_core::widget::WidgetKind;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_authored::state::AuthoredPanelState;
use ph2d_panel_authored::{AuthoredPanel, ids, rows};
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

/// **Uma tabela VIVA com DUAS seções** — e as chaves não existem na tabela compilada.
///
/// ⚠️ Isso é deliberado e é metade do assunto: uma row autorada que o `generated/panel.rs` ainda
/// não conhece é precisamente o caso que o artista tem em mãos ao desenhar a segunda seção.
fn two_sections() -> Vec<rows::Row> {
    [
        ("Alpha", WidgetKind::SectionHeader),
        ("alpha_one", WidgetKind::Slider),
        ("alpha_two", WidgetKind::Toggle),
        ("Beta", WidgetKind::SectionHeader),
        ("beta_one", WidgetKind::Slider),
        ("beta_two", WidgetKind::Button),
    ]
    .into_iter()
    .map(|(key, kind)| rows::Row {
        kind,
        label: key.to_string(),
        key: key.to_string(),
        id: ids::authored_row_id(key),
        rgba: None,
        icon: None,
        icon_id: None,
        options: Vec::new(),
    })
    .collect()
}

fn key_id(k: &str) -> ph2d_a11y::NodeId {
    ids::authored_row_id(k)
}

/// Um host com o painel aberto, **na ORDEM do app**.
///
/// ⚠️ **A ordem é metade do gate, e a primeira versão deste arquivo a tinha ao contrário.** Eu
/// publiquei a tabela viva ANTES do `with_panel`, e os três testes passaram — porque assim o
/// `populate` já vê as rows autoradas e regista tudo. No app não é isso que acontece: o
/// `populate` corre **uma vez**, no arranque, sobre a tabela COMPILADA, e a viva chega
/// **por quadro** depois disso (`render_loop` → `set_live_rows`). Uma fixture que arruma o estado
/// na ordem conveniente não contém o fenômeno que diz medir.
fn host_with_two_sections() -> (MockPanelHost, AuthoredPanelState) {
    rows::set_live_rows(None);
    let mut h = MockPanelHost::with_panel::<AuthoredPanel>();
    h.set_panel_visible(AuthoredPanel::ID, true);
    // Só AGORA o documento aparece — a ordem que o `render_loop` produz.
    rows::set_live_rows(Some(two_sections()));
    (h, AuthoredPanelState)
}

/// **Um cabeçalho AUTORADO dobra** — pintado, registado e vivo sob o ponteiro.
///
/// ⚠️ Este é o gate que nasceu VERMELHO com o report do Enio (*"não consigo contraí-la"*): a
/// tabela viva é publicada por quadro e o `populate` corre **uma vez**, então uma seção que o
/// documento tem e o código colado não tinha ficava *pintada, com retângulo de hit, e morta* —
/// `is_focusable` diz `false` para um id que ninguém registou, e o dispatcher larga o clique em
/// silêncio.
#[test]
fn an_authored_section_header_folds_under_a_real_pointer() {
    let (mut h, mut st) = host_with_two_sections();
    let id = key_id("Beta");
    let r = h
        .painted_rect::<AuthoredPanel>(&mut st, VIEWPORT, id)
        .expect("o cabecalho 'Beta' nao foi PINTADO com area clicavel");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let mut evs = h.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    evs.extend(h.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100)));
    assert!(
        !evs.is_empty(),
        "o ponteiro sobre o cabecalho 'Beta' nao virou evento nenhum — ele esta' desenhado e nao \
         existe para o dispatcher (a tabela VIVA e' publicada por quadro, o `populate` corre uma \
         vez)"
    );
    assert!(
        h.store().is_collapsed(id),
        "o clique chegou e a secao NAO dobrou — o id nao foi marcado como dobravel"
    );
    rows::set_live_rows(None);
}

/// **Cada cabeçalho manda só nas rows ATÉ o próximo.**
///
/// ⚠️ O oráculo é o que o painel PINTA (o retângulo existe ou não), e não o modelo: uma lei
/// *"esconde tudo abaixo"* passaria por qualquer asserção feita sobre o `folded` interno, porque
/// ela é a mesma variável nas duas leis. O que as separa é a segunda seção continuar na tela.
#[test]
fn folding_the_first_section_leaves_the_second_one_alone() {
    let (mut h, mut st) = host_with_two_sections();
    // O controle: com nada dobrado, as seis rows têm área.
    for k in [
        "Alpha",
        "alpha_one",
        "alpha_two",
        "Beta",
        "beta_one",
        "beta_two",
    ] {
        assert!(
            h.painted_rect::<AuthoredPanel>(&mut st, VIEWPORT, key_id(k))
                .is_some(),
            "a row `{k}` nao foi pintada no controle (nada dobrado)"
        );
    }
    h.store_mut().mark_collapsible_section(key_id("Alpha"));
    h.store_mut().set_collapsed(key_id("Alpha"), true);
    // As duas metades: o que a primeira seção esconde, e o que ela NÃO pode esconder.
    for k in ["alpha_one", "alpha_two"] {
        assert!(
            h.painted_rect::<AuthoredPanel>(&mut st, VIEWPORT, key_id(k))
                .is_none(),
            "a row `{k}` continua pintada com a secao 'Alpha' dobrada"
        );
    }
    for k in ["Alpha", "Beta", "beta_one", "beta_two"] {
        assert!(
            h.painted_rect::<AuthoredPanel>(&mut st, VIEWPORT, key_id(k))
                .is_some(),
            "a row `{k}` sumiu ao dobrar a secao ANTERIOR — um cabecalho esta' a mandar alem do \
             proximo"
        );
    }
    rows::set_live_rows(None);
}

/// **O que vem DEPOIS sobe** — a metade do "sem vão" que uma seção sozinha não pode afirmar.
///
/// ⚠️ O irmão no `seam_authored` mede a ALTURA reportada, e é a pergunta do scrollbar. Esta é a
/// outra: com uma segunda seção adiante, o vão deixaria de ser um número no fim do corpo e
/// passaria a ser um buraco **entre** as duas — visível, e no meio do painel.
///
/// ⚠️ **O oráculo é o passo MEDIDO, e a primeira versão dele era um limiar de 1 px.** A mutação
/// que a apanhou é reveladora: fazer a row dobrada avançar `ROW_H_PX` **sem o espaçamento**
/// deixava-a passar — a última row ainda subia, só que o dobro do gap a menos. *Um limiar
/// responde "mexeu?"; a pergunta é "mexeu QUANTO?"*, e o passo sai das duas rows visíveis da
/// segunda seção, não de uma constante de token importada para cá.
#[test]
fn a_folded_section_shrinks_the_body_instead_of_leaving_a_hole() {
    let (mut h, mut st) = host_with_two_sections();
    let tall = h
        .painted_rect::<AuthoredPanel>(&mut st, VIEWPORT, key_id("beta_two"))
        .expect("a ultima row nao foi pintada no controle");
    // O PASSO de uma row, medido no próprio painel: duas vizinhas visíveis, uma sob a outra.
    let above = h
        .painted_rect::<AuthoredPanel>(&mut st, VIEWPORT, key_id("beta_one"))
        .expect("a penultima row nao foi pintada no controle");
    let pitch = tall.y - above.y;
    assert!(
        pitch > 0.0,
        "o passo entre duas rows nao e' positivo: {pitch}"
    );

    h.store_mut().mark_collapsible_section(key_id("Alpha"));
    h.store_mut().set_collapsed(key_id("Alpha"), true);
    let short = h
        .painted_rect::<AuthoredPanel>(&mut st, VIEWPORT, key_id("beta_two"))
        .expect("a ultima row sumiu ao dobrar uma secao que nao e' a dela");
    // Duas rows escondidas ⇒ a última sobe exactamente dois passos.
    let moved = tall.y - short.y;
    assert!(
        (moved - 2.0 * pitch).abs() < 0.5,
        "a ultima row subiu {moved} e o vao das duas rows escondidas mede {} — o que sobra e' um \
         buraco no meio do painel ({} -> {})",
        2.0 * pitch,
        tall.y,
        short.y
    );
    rows::set_live_rows(None);
}

/// **A borda de CIMA da primeira row é da ROW, não da faixa de arraste.**
///
/// ⚠️ Gate red-first, e o defeito é de ARITMÉTICA: a faixa era registada com
/// `PANEL_HEADER_H_DEFAULT` (56 px), uma constante dimensionada para um cabeçalho com SUBTÍTULO,
/// enquanto este painel abre o corpo em `PANEL_TITLE_BASELINE + título + Md` = **44**. Os 12 px de
/// diferença caem dentro do corpo, e como o chrome é registado **depois** (o hit é
/// último-registado-ganha, de propósito, para blindar o cabeçalho de rows roladas), a faixa
/// GANHAVA — 43% da altura de uma row de 28 px.
///
/// ⚠️ **Nenhum seam via, e a razão é a fixture:** todos clicam no CENTRO da row, que cai 2 px
/// abaixo do fim da faixa. Ela passava raspando por cima do defeito. Este clica na BORDA.
///
/// ⚠️ E o sintoma lê como intermitente: clicar 3 px mais abaixo funciona, e o canto direito da row
/// (o que a reserva do X deixa livre) responde sempre — o artista conclui que o painel é errático.
///
/// O irmão `ph2d-panel-wet-tuning` já passa `body_top - rect.y` com o motivo escrito ao lado.
#[test]
fn the_top_edge_of_the_first_row_belongs_to_the_row_not_to_the_drag_band() {
    let (mut h, mut st) = host_with_two_sections();
    let r = h
        .painted_rect::<AuthoredPanel>(&mut st, VIEWPORT, key_id("Alpha"))
        .expect("o cabecalho nao foi pintado");
    // A BORDA de cima, não o centro — é ali que a faixa mordia.
    let (cx, cy) = (r.x + r.w * 0.5, r.y + 2.0);
    h.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    for ev in h.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100)) {
        AuthoredPanel::apply_event(&mut st, &mut h, ev);
    }
    assert!(
        h.store().is_collapsed(key_id("Alpha")),
        "o clique na borda de cima do cabecalho nao dobrou a secao — a faixa de arraste do painel \
         comeu-o, e ela e' registada DEPOIS do corpo"
    );
}
