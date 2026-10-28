//! O VERSO do cartão (W3) pela costura do hero: abrir por clique e `Enter`, `Esc` fecha, a
//! descrição grava markdown, as datas pelo seletor, a capa por cor e o desfazer dentro do verso.

use crate::{Mesa, pega};
use ph2d_editor_core::screens::hero::board_keys::BoardKey;
use ph2d_editor_core::widget::{CalendarLayout, DatePickerLayout};
use ph2d_editor_core::zones::Rect;
use ph2d_host::Modifiers;
use ph2d_kanban_model::{CardId, CoverFill, Hue, KanbanId, Member};

pub(crate) fn sem() -> Modifiers {
    Modifiers::default()
}

pub(crate) fn ctrl() -> Modifiers {
    Modifiers {
        ctrl: true,
        ..Modifiers::default()
    }
}

/// O cartão `titulo` da cena.
pub(crate) fn cartao(m: &Mesa, id: KanbanId, titulo: &str) -> CardId {
    m.kanban(id)
        .cards
        .values()
        .find(|c| c.title == titulo)
        .unwrap_or_else(|| panic!("cartão {titulo}"))
        .id
}

/// A cena 1 com o verso de `titulo` aberto por um clique no cartão.
pub(crate) fn abre(titulo: &str) -> (Mesa, KanbanId, CardId) {
    let (mut m, id) = Mesa::cena1();
    let c = cartao(&m, id, titulo);
    let r = m.rect_cartao(id, c);
    m.clica(pega(r));
    assert_eq!(m.docs().probe_back(id.0), Some(c), "o clique abre o verso");
    m.quadro();
    (m, id, c)
}

/// O rect pintado do alvo `nome` (o de cima).
pub(crate) fn alvo(m: &mut Mesa, id: KanbanId, nome: &str) -> Rect {
    m.docs()
        .probe_back_hits(id.0)
        .into_iter()
        .rev()
        .find(|(n, _)| n == nome)
        .unwrap_or_else(|| panic!("o alvo {nome} não foi pintado"))
        .1
}

/// Um clique no meio do alvo `nome`.
pub(crate) fn toca(m: &mut Mesa, id: KanbanId, nome: &str) {
    let r = alvo(m, id, nome);
    m.clica((r.x + r.w / 2.0, r.y + r.h / 2.0));
}

pub(crate) fn eu() -> Member {
    Member(ph2d_i18n::tr("kanban.back.me").to_owned())
}

#[test]
fn o_clique_e_o_enter_abrem_o_verso_e_o_esc_fecha() {
    let (mut m, id, c) = abre("Write the logline");
    let bytes = m.bytes();
    assert!(m.tecla(BoardKey::Escape, sem(), None), "o Esc é do verso");
    m.quadro();
    assert_eq!(m.docs().probe_back(id.0), None, "o Esc fecha");
    assert_eq!(m.bytes(), bytes, "abrir e fechar não escreve");
    assert_eq!(m.docs().probe_focus(id.0), Some(c), "o foco fica no cartão");
    m.enter();
    assert_eq!(m.docs().probe_back(id.0), Some(c), "o Enter abre");
    // Clicar fora do modal (no véu) fecha.
    let area = crate::AREA;
    m.clica((area.x + 4.0, area.y + area.h - 4.0));
    assert_eq!(m.docs().probe_back(id.0), None, "o clique fora fecha");
    // O E continua a edição rápida do título.
    assert!(m.tecla(BoardKey::Char('e'), sem(), Some("e")));
    m.quadro();
    assert_eq!(
        m.docs().probe_field(id.0).as_deref(),
        Some("Write the logline")
    );
}

#[test]
fn a_descricao_editada_grava_markdown() {
    let (mut m, id, c) = abre("Write the logline");
    toca(&mut m, id, "DescEdit");
    assert!(m.docs().probe_back_state(id.0).2, "em edição");
    m.escreve(" Then ");
    assert!(
        m.tecla(BoardKey::Char('b'), ctrl(), Some("b")),
        "Ctrl+B é do editor"
    );
    m.escreve("bold");
    assert!(m.tecla(BoardKey::Enter, ctrl(), None), "Ctrl+Enter guarda");
    m.quadro();
    let d = m.kanban(id).cards[&c].description.clone();
    assert!(
        d.starts_with("One sentence. Then **bold**"),
        "o markdown gravado: {d:?}"
    );
    assert!(!m.docs().probe_back_state(id.0).2, "saiu da edição");
    assert!(m.desfaz().is_some());
    assert_eq!(
        m.kanban(id).cards[&c].description,
        "One sentence.",
        "um passo de desfazer"
    );
}

/// A célula do dia `dia` do mês mostrado no seletor aberto.
fn celula(m: &mut Mesa, id: KanbanId, ano: i16, mes: u8, dia: u8) -> (f32, f32) {
    let r = alvo(m, id, "Pop(Picker)");
    let cal = CalendarLayout::of(DatePickerLayout::of(r).calendar);
    let grelha =
        ph2d_kanban_dates::CalendarMonth::new(ano, mes, None, ph2d_kanban_dates::Weekday::Monday)
            .expect("mês")
            .cells();
    let i = grelha
        .iter()
        .position(|d| d.in_month && d.day == dia)
        .expect("dia");
    let c = cal.cell(i);
    (c.x + c.w / 2.0, c.y + c.h / 2.0)
}

#[test]
fn as_datas_pelo_seletor_mudam_a_entrega() {
    let (mut m, id, c) = abre("Write the logline");
    assert!(
        m.tecla(BoardKey::Char('d'), sem(), Some("d")),
        "D abre as datas"
    );
    m.quadro();
    assert_eq!(m.docs().probe_back_state(id.0).0.as_deref(), Some("Dates"));
    let z = ph2d_kanban_dates::Zone::system();
    let hoje = ph2d_kanban_dates::civil(ph2d_kanban_model::Moment(now()).0, &z).date;
    let dia = celula(&mut m, id, hoje.year, hoje.month, 15);
    m.clica(dia);
    let save = DatePickerLayout::of(alvo(&mut m, id, "Pop(Picker)")).save;
    m.clica((save.x + save.w / 2.0, save.y + save.h / 2.0));
    let due = m.kanban(id).cards[&c].due.expect("a entrega gravou-se");
    let visto = ph2d_kanban_dates::civil(due.at.0, &z);
    assert_eq!((visto.date.month, visto.date.day), (hoje.month, 15));
    assert_eq!((visto.hour, visto.minute), (12, 0), "a hora por omissão");
    assert_eq!(
        m.docs().probe_back_state(id.0).0,
        None,
        "o Save fecha o seletor"
    );
    // A frente e o verso leem a mesma entrega: o verso mostra o grupo «Dates» com a caixa.
    alvo(&mut m, id, "DueDone");
    toca(&mut m, id, "DueDone");
    assert!(
        m.kanban(id).cards[&c].due.expect("entrega").done,
        "concluída"
    );
}

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(0))
}

#[test]
fn a_capa_por_cor_e_o_desfazer_dentro_do_verso() {
    let (mut m, id, c) = abre("Write the logline");
    toca(&mut m, id, "Side(Cover)");
    toca(&mut m, id, "Pop(CoverHue(3))");
    let capa = m.kanban(id).cards[&c].cover.expect("capa");
    assert_eq!(capa.fill, CoverFill::Color(Hue::ALL[3]));
    assert!(!capa.full, "metade por omissão");
    toca(&mut m, id, "Pop(CoverSize(true))");
    assert!(m.kanban(id).cards[&c].cover.expect("capa").full, "inteira");
    // Votar (V) e desfazer pelo menu Edit: o verso continua aberto.
    toca(&mut m, id, "Pop(Close)");
    assert!(m.tecla(BoardKey::Char('v'), sem(), Some("v")));
    m.quadro();
    assert_eq!(m.kanban(id).cards[&c].votes, vec![eu()]);
    assert_eq!(m.desfaz(), Some(true));
    assert!(
        m.kanban(id).cards[&c].votes.is_empty(),
        "o desfazer tira o voto"
    );
    assert_eq!(
        m.docs().probe_back(id.0),
        Some(c),
        "o verso continua aberto"
    );
    assert!(
        m.kanban(id).cards[&c].cover.is_some(),
        "um passo: só o voto saiu"
    );
}
