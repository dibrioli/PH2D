//! A vista Tabela: pintura, toque e o que ela pede ao documento; e a foto.

use crate::support::{World, now};
use ph2d_editor_core::widget::{CalendarLayout, DatePickerLayout};
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_dates::{civil, day_of};
use ph2d_kanban_model::{CardId, FieldValue, KanbanSet, OptionId};
use ph2d_kanban_views::table::{Column, Pop};
use ph2d_kanban_views::{PointerKind, Reply, ViewKind, Views};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::{Affine, VectorScene};

const AREA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1440.0,
    h: 900.0,
};

fn paint(v: &mut Views, w: &World, ts: &mut TextSystem) {
    let mut s = VectorScene::new();
    v.paint(&mut s, Affine::IDENTITY, AREA, &w.set, w.id, ts, Theme::Light, &w.cx());
}

fn click(v: &mut Views, w: &World, (x, y): (f32, f32)) -> Reply {
    let r = v.pointer(&w.set, w.id, &w.cx(), PointerKind::Down, x, y);
    v.pointer(&w.set, w.id, &w.cx(), PointerKind::Up, x, y);
    r
}

fn mid(r: Rect) -> (f32, f32) {
    (r.x + r.w / 2.0, r.y + r.h / 2.0)
}

fn titles(v: &Views, w: &World) -> Vec<String> {
    let k = w.set.get(w.id).expect("kanban");
    v.table.rows().iter().map(|id| k.cards[id].title.clone()).collect()
}

fn find(w: &World, title: &str) -> CardId {
    let k = w.set.get(w.id).expect("kanban");
    k.cards.values().find(|c| c.title == title).expect("cartão").id
}

fn row_of(v: &Views, id: CardId) -> usize {
    v.table.rows().iter().position(|r| *r == id).expect("linha")
}

fn setup() -> (Views, World, TextSystem) {
    let w = World::scene1();
    let mut v = Views::default();
    v.kind = ViewKind::Table;
    let mut ts = TextSystem::without_system_fonts();
    paint(&mut v, &w, &mut ts);
    (v, w, ts)
}

#[test]
fn table_paints_one_row_per_visible_card_in_board_order() {
    let (v, w, _) = setup();
    let t = titles(&v, &w);
    assert_eq!(t.len(), 18, "os 18 cartões da cena 1");
    assert_eq!(t[0], "Write the logline");
    assert_eq!(t[4], "Storyboard act 1", "a To Do depois da Backlog");
    let g = v.table.grid();
    assert_eq!(g.row(0).y, g.body.y, "a 1.ª linha logo abaixo do cabeçalho fixo");
    assert_eq!(g.row_at(g.row(3).y + 1.0), Some(3));
    let cols: Vec<Column> = g.cols.iter().map(|c| c.0).collect();
    assert_eq!(&cols[..5], &[Column::Card, Column::List, Column::Labels, Column::Members, Column::Due]);
    assert_eq!(cols.len(), 7, "e os dois campos personalizados");
}

#[test]
fn table_header_sorts_by_due_then_descending_then_board_order() {
    let (mut v, w, mut ts) = setup();
    let board = titles(&v, &w);
    let due_head = v.table.grid().cols.iter().find(|c| c.0 == Column::Due).map(|c| (c.1 + 4.0, v.table.grid().header.y + 4.0)).expect("Due");
    click(&mut v, &w, due_head);
    paint(&mut v, &w, &mut ts);
    let k = w.set.get(w.id).expect("kanban");
    let dues: Vec<_> = v.table.rows().iter().map(|id| k.cards[id].due.map(|d| d.at)).collect();
    let dated = dues.iter().take_while(|d| d.is_some()).count();
    assert!(dues[..dated].windows(2).all(|p| p[0] <= p[1]), "ascendente");
    assert!(dues[dated..].iter().all(Option::is_none), "sem data no fim");
    assert_eq!(titles(&v, &w)[0], "Rough the opening shot", "a mais antiga primeiro");
    click(&mut v, &w, due_head);
    paint(&mut v, &w, &mut ts);
    assert_eq!(titles(&v, &w)[0], "Pick the music", "descendente: a mais longe primeiro");
    click(&mut v, &w, due_head);
    paint(&mut v, &w, &mut ts);
    assert_eq!(titles(&v, &w), board, "o 3.º clique volta à ordem do quadro");
}

#[test]
fn table_header_sorts_by_card_name() {
    let (mut v, w, mut ts) = setup();
    let at = (v.table.grid().cols[0].1 + 4.0, v.table.grid().header.y + 4.0);
    click(&mut v, &w, at);
    paint(&mut v, &w, &mut ts);
    let t = titles(&v, &w);
    let mut sorted = t.clone();
    sorted.sort_by_key(|s| s.to_lowercase());
    assert_eq!(t, sorted);
}

#[test]
fn table_click_on_title_opens_the_back() {
    let (mut v, w, _) = setup();
    let id = find(&w, "Design the fox");
    let cell = v.table.grid().cell(row_of(&v, id), Column::Card).expect("célula");
    let r = click(&mut v, &w, mid(cell));
    assert_eq!(r.open.map(|o| o.card), Some(id));
    assert!(r.plan.is_empty(), "abrir não escreve");
}

#[test]
fn table_list_dropdown_moves_the_card_to_the_end_of_the_chosen_list() {
    let (mut v, mut w, mut ts) = setup();
    let id = find(&w, "Write the logline");
    let cell = v.table.grid().cell(row_of(&v, id), Column::List).expect("célula");
    click(&mut v, &w, mid(cell));
    let Some(Pop::List { dd, chip, .. }) = &v.table.pop else {
        panic!("o dropdown das listas abriu");
    };
    let i = dd.options.iter().position(|o| o.label == "Done").expect("Done");
    let at = mid(dd.option_rect(*chip, i));
    let done = dd.options[i].value;
    let r = click(&mut v, &w, at);
    assert!(v.table.pop.is_none(), "escolher fecha");
    assert_eq!(r.plan.ops(w.id).len(), 1, "um put do cartão");
    w.apply(r.plan);
    let k = w.set.get(w.id).expect("kanban");
    assert_eq!(k.cards[&id].list, done);
    assert_eq!(k.cards_in(done).last().map(|c| c.id), Some(id), "no fim da lista");
    paint(&mut v, &w, &mut ts);
    assert_eq!(titles(&v, &w).last().map(String::as_str), Some("Write the logline"));
}

#[test]
fn table_labels_popover_toggles_a_label_and_stays_open() {
    let (mut v, mut w, mut ts) = setup();
    let id = find(&w, "Collect references for the forest");
    let cell = v.table.grid().cell(row_of(&v, id), Column::Labels).expect("célula");
    click(&mut v, &w, mid(cell));
    let Some(Pop::Labels { at, .. }) = v.table.pop.clone() else {
        panic!("o popover das etiquetas abriu");
    };
    // A 1.ª linha depois do título (as etiquetas pelo nome: «Animation» primeiro).
    let row = ph2d_kanban_views::table::edit::pop_row(at, 1);
    let r = click(&mut v, &w, mid(row));
    assert!(v.table.pop.is_some(), "alternar deixa o popover aberto");
    w.apply(r.plan);
    paint(&mut v, &w, &mut ts);
    let k = w.set.get(w.id).expect("kanban");
    let names: Vec<_> = k.cards[&id].labels.iter().map(|l| k.labels[l].name.clone()).collect();
    assert_eq!(names, ["Animation"]);
}

#[test]
fn table_due_picker_moves_the_day_and_keeps_the_hour() {
    let (mut v, mut w, _) = setup();
    let id = find(&w, "Write the logline");
    let before = w.set.get(w.id).expect("kanban").cards[&id].due.expect("entrega");
    let cell = v.table.grid().cell(row_of(&v, id), Column::Due).expect("célula");
    click(&mut v, &w, mid(cell));
    let Some(Pop::Dates { picker, at, .. }) = v.table.pop.clone() else {
        panic!("o seletor de datas abriu");
    };
    let l = DatePickerLayout::of(at);
    let cal = CalendarLayout::of(l.calendar);
    let i = 30;
    let want = picker.widget.calendar.cells[i];
    click(&mut v, &w, mid(cal.cell(i)));
    let r = click(&mut v, &w, mid(l.save));
    assert!(v.table.pop.is_none(), "guardar fecha");
    w.apply(r.plan);
    let after = w.set.get(w.id).expect("kanban").cards[&id].due.expect("entrega");
    let d = day_of(after.at.0, &w.zone);
    assert_eq!((i32::from(d.year), d.month, d.day), (want.year, want.month, want.day));
    let (h0, h1) = (civil(before.at.0, &w.zone), civil(after.at.0, &w.zone));
    assert_eq!((h0.hour, h0.minute), (h1.hour, h1.minute), "a hora fica");
}

#[test]
fn table_field_dropdown_sets_and_clears_an_option() {
    let (mut v, mut w, mut ts) = setup();
    let id = find(&w, "Design the fox");
    let col = v.table.grid().cols[6].0;
    let cell = v.table.grid().cell(row_of(&v, id), col).expect("célula");
    click(&mut v, &w, mid(cell));
    let Some(Pop::Option { dd, chip, field, .. }) = v.table.pop.clone() else {
        panic!("o dropdown do campo abriu");
    };
    let i = dd.options.iter().position(|o| o.label == "High").expect("High");
    let r = click(&mut v, &w, mid(dd.option_rect(chip, i)));
    w.apply(r.plan);
    let k = w.set.get(w.id).expect("kanban");
    assert_eq!(k.cards[&id].fields.get(&field), Some(&FieldValue::Option(OptionId(1))));
    paint(&mut v, &w, &mut ts);
    let cell = v.table.grid().cell(row_of(&v, id), col).expect("célula");
    click(&mut v, &w, mid(cell));
    let Some(Pop::Option { dd, chip, .. }) = v.table.pop.clone() else {
        panic!("reabriu");
    };
    let r = click(&mut v, &w, mid(dd.option_rect(chip, 0)));
    w.apply(r.plan);
    assert!(!w.set.get(w.id).expect("kanban").cards[&id].fields.contains_key(&field), "«—» limpa");
}

#[test]
fn table_virtual_scroll_paints_only_the_visible_rows() {
    let mut set = KanbanSet::default();
    let id = ph2d_kanban_views::fixture::big(&mut set, 2_000, now());
    let w = World::over(set, id);
    let mut v = Views::default();
    let mut ts = TextSystem::without_system_fonts();
    paint(&mut v, &w, &mut ts);
    assert_eq!(v.table.rows().len(), 2_000);
    let fits = (v.table.grid().body.h / 40.0).ceil() as usize;
    let d = v.table.drawn();
    assert_eq!(d.start, 0);
    assert!(d.len() <= fits + 1, "{} linhas pintadas para {fits} que cabem", d.len());
    v.wheel(&w.set, w.id, &w.cx(), (400.0, 400.0), -16.0 * 100.0, (false, false));
    paint(&mut v, &w, &mut ts);
    let d = v.table.drawn();
    assert_eq!(d.start, 120, "100 linhas de roda × 48 px / 40 px");
    assert!(d.len() <= fits + 1);
}

/// 📏 A tabela a 20 000 cartões: pintar a cena inteira e ordenar (mín de N, perfil do teste).
#[test]
#[ignore = "medição: bash scripts/cargo-test-narrow.sh ph2d-kanban-views measure_table_20k -- --ignored --nocapture"]
fn measure_table_20k() {
    let mut set = KanbanSet::default();
    let id = ph2d_kanban_views::fixture::big(&mut set, 20_000, now());
    let w = World::over(set, id);
    let mut v = Views::default();
    let mut ts = TextSystem::new();
    paint(&mut v, &w, &mut ts);
    let min = |n: usize, f: &mut dyn FnMut()| {
        (0..n)
            .map(|_| {
                let t = std::time::Instant::now();
                f();
                t.elapsed().as_secs_f64() * 1e3
            })
            .fold(f64::INFINITY, f64::min)
    };
    let board = min(15, &mut || paint(&mut v, &w, &mut ts));
    v.table.toggle_sort(Column::Due);
    let by_due = min(15, &mut || paint(&mut v, &w, &mut ts));
    v.table.toggle_sort(Column::Due);
    v.table.toggle_sort(Column::Card);
    let by_name = min(15, &mut || paint(&mut v, &w, &mut ts));
    let k = w.set.get(w.id).expect("kanban");
    let rows = min(15, &mut || {
        let _ = ph2d_kanban_views::table::layout::rows(k, &w.cx(), None);
    });
    let name = Some(ph2d_kanban_views::table::Sort { by: Column::Card, desc: false });
    let rows_name = min(5, &mut || {
        let _ = ph2d_kanban_views::table::layout::rows(k, &w.cx(), name);
    });
    let profile = if cfg!(debug_assertions) { "dev/debug" } else { "otimizado" };
    let msg = format!(
        "medição tabela 20k ({profile}): pintar (linhas em cache) ordem do quadro {board:.2} ms · por entrega {by_due:.2} ms · por nome {by_name:.2} ms · refazer as linhas sem cache: ordem do quadro {rows:.2} ms, por nome {rows_name:.2} ms · linhas pintadas {}",
        v.table.drawn().len()
    );
    println!("{msg}");
    let _ = std::fs::write(std::env::temp_dir().join("ph2d_medicao_tabela_20k.txt"), msg);
}

#[test]
#[ignore = "a placa: fotos para docs/TrelloClone/smoke/W7/"]
fn photo_table() {
    crate::photos::photo("tabela", ViewKind::Table, |_, _| {});
    crate::photos::photo("tabela_lista_aberta", ViewKind::Table, |v, w| {
        let mut ts = TextSystem::new();
        paint(v, w, &mut ts);
        v.table.toggle_sort(Column::Due);
        paint(v, w, &mut ts);
        let id = find(w, "Storyboard act 1");
        let cell = v.table.grid().cell(row_of(v, id), Column::List).expect("célula");
        click(v, w, mid(cell));
        let hover = v.table.grid().cell(row_of(v, find(w, "Animate shot 12")), Column::Card).expect("c");
        v.pointer(&w.set, w.id, &w.cx(), PointerKind::Move, hover.x + 10.0, hover.y + 10.0);
    });
}
