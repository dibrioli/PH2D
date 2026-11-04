//! D14 — os campos personalizados do cartão (smoke do dono 30/10): a lista suspensa ABRE inteira por
//! baixo do valor e rola (roda, barra, setas); o campo de data tem o calendário no canto; e os nomes
//! não se cortam — nem na grelha do verso, nem nos chips da frente.

use crate::verso::{abre, alvo, celula, now, sem, toca};
use crate::{Mesa, pega};
use ph2d_editor_core::screens::hero::board_keys::BoardKey;
use ph2d_editor_core::widget::DatePickerLayout;
use ph2d_kanban_model::{
    CardId, FieldDef, FieldId, FieldKind, FieldOption, FieldValue, FracKey, KanbanId, KanbanOp,
    Meta, OptionId, Record,
};

const LISTA: FieldId = FieldId(9101);
const DATA: FieldId = FieldId(9102);

fn campo(m: &Mesa, id: KanbanId, f: FieldId, name: &str, kind: FieldKind, on_front: bool) {
    let def = FieldDef {
        id: f,
        name: name.into(),
        kind,
        on_front,
        pos: FracKey::between(None, None),
        meta: Meta::default(),
    };
    m.hero
        .documents
        .kanbans_shared()
        .borrow_mut()
        .apply(id, vec![KanbanOp::put(Record::Field(def))]);
}

fn opcoes(n: u64) -> FieldKind {
    FieldKind::Dropdown(
        (0..n)
            .map(|i| FieldOption {
                id: OptionId(i),
                text: format!("Option {i}"),
                color: None,
            })
            .collect(),
    )
}

fn valor(m: &Mesa, id: KanbanId, c: CardId, f: FieldId) -> Option<FieldValue> {
    m.kanban(id).cards[&c].fields.get(&f).cloned()
}

fn pintados(m: &mut Mesa, id: KanbanId, prefixo: &str) -> Vec<String> {
    m.docs()
        .probe_back_hits(id.0)
        .into_iter()
        .map(|(n, _)| n)
        .filter(|n| n.starts_with(prefixo))
        .collect()
}

#[test]
fn d14_the_dropdown_opens_the_whole_list_scrolls_and_picks() {
    let (mut m, id, c) = abre("Write the logline");
    campo(&m, id, LISTA, "Priority", opcoes(40), false);
    m.quadro();
    toca(&mut m, id, &format!("FieldValue({LISTA:?})"));
    m.quadro();
    let rows = pintados(&mut m, id, "Pop(Choice(");
    assert!(rows.len() > 3, "a lista abriu com várias linhas: {rows:?}");
    assert!(
        !rows.contains(&"Pop(Choice(40))".to_owned()),
        "40 opções não cabem: rola"
    );
    let barra = alvo(&mut m, id, "Pop(ChoiceBar)");
    // A roda sobre a lista rola-a.
    let lista = alvo(&mut m, id, "Pop(Choice(1))");
    for _ in 0..200 {
        m.roda(lista.x + 4.0, lista.y + 4.0, -16.0);
    }
    m.quadro();
    assert!(
        pintados(&mut m, id, "Pop(Choice(").contains(&"Pop(Choice(40))".to_owned()),
        "a roda chega ao fim da lista"
    );
    // O polegar (no fundo, logo acima da seta ▼ — D16) arrastado até ao topo volta ao princípio.
    let fundo = (barra.x + barra.w / 2.0, barra.y + barra.h - 20.0);
    m.arrasta(fundo, (barra.x + barra.w / 2.0, barra.y - 400.0), 4);
    m.quadro();
    assert!(
        pintados(&mut m, id, "Pop(Choice(").contains(&"Pop(Choice(0))".to_owned()),
        "a barra arrasta a lista"
    );
    toca(&mut m, id, "Pop(Choice(3))");
    m.quadro();
    assert_eq!(
        valor(&m, id, c, LISTA),
        Some(FieldValue::Option(OptionId(2)))
    );
    assert!(
        pintados(&mut m, id, "Pop(Choice(").is_empty(),
        "escolher fecha"
    );
    // As setas e o Enter: abre com a actual realçada; ↓ ↓ Enter escolhe duas abaixo.
    toca(&mut m, id, &format!("FieldValue({LISTA:?})"));
    m.quadro();
    m.tecla(BoardKey::Down, sem(), None);
    m.tecla(BoardKey::Down, sem(), None);
    m.tecla(BoardKey::Enter, sem(), None);
    m.quadro();
    assert_eq!(
        valor(&m, id, c, LISTA),
        Some(FieldValue::Option(OptionId(4)))
    );
    // «None» limpa; o Esc fecha sem mudar.
    toca(&mut m, id, &format!("FieldValue({LISTA:?})"));
    m.quadro();
    m.tecla(BoardKey::Escape, sem(), None);
    m.quadro();
    assert!(pintados(&mut m, id, "Pop(Choice(").is_empty());
    assert!(
        m.docs().probe_back(id.0).is_some(),
        "o Esc fecha a lista, não o verso"
    );
    toca(&mut m, id, &format!("FieldValue({LISTA:?})"));
    m.quadro();
    toca(&mut m, id, "Pop(Choice(0))");
    assert_eq!(valor(&m, id, c, LISTA), None);
}

#[test]
fn d14_the_date_field_has_a_calendar_in_its_corner() {
    let (mut m, id, c) = abre("Write the logline");
    campo(&m, id, DATA, "Shoot day", FieldKind::Date, false);
    m.quadro();
    toca(&mut m, id, &format!("FieldDate({DATA:?})"));
    m.quadro();
    assert_eq!(m.docs().probe_back_state(id.0).0.as_deref(), Some("Dates"));
    let z = ph2d_kanban_dates::Zone::system();
    let hoje = ph2d_kanban_dates::civil(ph2d_kanban_model::Moment(now()).0, &z).date;
    let dia = celula(&mut m, id, hoje.year, hoje.month, 15);
    m.clica(dia);
    // Uma data só: sem «Start date» nem lembrete — o painel curto.
    let r = alvo(&mut m, id, "Pop(Picker)");
    assert_eq!(r.h, DatePickerLayout::preferred_height_for(true));
    let save = DatePickerLayout::of_mode(r, true).save;
    m.clica(pega(save));
    let Some(FieldValue::Date(at)) = valor(&m, id, c, DATA) else {
        panic!("a data gravou-se: {:?}", valor(&m, id, c, DATA));
    };
    let visto = ph2d_kanban_dates::civil(at.0, &z);
    assert_eq!((visto.date.month, visto.date.day), (hoje.month, 15));
    // O campo continua a escrever-se: o clique no valor abre a edição de texto.
    m.quadro();
    toca(&mut m, id, &format!("FieldValue({DATA:?})"));
    m.quadro();
    alvo(&mut m, id, "EditField");
    alvo(&mut m, id, &format!("FieldDate({DATA:?})"));
}

const LONGO: (&str, &str) = (
    "Estimated hours until final delivery",
    "Waiting for the director to approve the second pass",
);

/// A cena 1 com fontes, a lista (30 opções, na frente), a data e um campo de nome e valor longos.
fn cena_campos() -> (Mesa, KanbanId, CardId) {
    let mut m = Mesa::new();
    m.ts = ph2d_text::TextSystem::new();
    let id = ph2d_app_kanban::smoke::stage(&mut m.hero, 1).expect("cena");
    campo(&m, id, LISTA, "Priority", opcoes(30), true);
    campo(&m, id, DATA, "Shoot day", FieldKind::Date, false);
    campo(&m, id, FieldId(9103), LONGO.0, FieldKind::Text, true);
    let c = crate::verso::cartao(&m, id, "Write the logline");
    let shared = m.hero.documents.kanbans_shared();
    let mut card = shared.borrow().get(id).expect("k").cards[&c].clone();
    card.fields
        .insert(FieldId(9103), FieldValue::Text(LONGO.1.into()));
    card.fields.insert(LISTA, FieldValue::Option(OptionId(3)));
    card.meta.version += 1;
    let put = vec![KanbanOp::put(Record::Card(card.into()))];
    shared.borrow_mut().apply(id, put);
    m.quadros(2);
    (m, id, c)
}

/// O chip de uma linha leva o texto que o fez, qualquer que seja a largura dele: o corte de `1` ulp
/// dependia de onde a medida da fonte caía (`Priority: Option 3` passava na Inter e não na Noto), logo
/// o portão varre as fontes e as opções. *Mutação: `room` volta a ser o chip menos o respiro.*
#[test]
fn d14_a_chip_never_cuts_the_text_that_sized_it_in_any_font() {
    use ph2d_editor_core::text_elide::elisao;
    use ph2d_tokens::{UiFont, UiTextStyle};
    let mut cortados = Vec::new();
    for font in UiFont::ALL {
        ph2d_text::set_active_text_style(UiTextStyle {
            font,
            ..UiTextStyle::default()
        });
        let (mut m, id, c) = cena_campos();
        for i in 0..30 {
            let shared = m.hero.documents.kanbans_shared();
            let mut card = shared.borrow().get(id).expect("k").cards[&c].clone();
            card.fields.insert(LISTA, FieldValue::Option(OptionId(i)));
            card.meta.version += 1;
            shared
                .borrow_mut()
                .apply(id, vec![KanbanOp::put(Record::Card(card.into()))]);
            let ((), medidos) = elisao::medindo(|| m.pinta(&mut ph2d_vector::VectorScene::new()));
            let pintou = medidos.iter().any(|x| x.texto.contains("Priority"));
            assert!(pintou, "{font:?}: o chip da opção {i} foi pintado");
            cortados.extend(
                medidos
                    .into_iter()
                    .filter(|x| x.texto.contains("Priority") && !x.coube())
                    .map(|x| format!("{font:?}: {} → {}", x.texto, x.pintado)),
            );
        }
    }
    ph2d_text::set_active_text_style(UiTextStyle::default());
    assert_eq!(cortados, Vec::<String>::new());
}

#[test]
fn d14_field_names_are_never_cut_while_they_fit() {
    use ph2d_editor_core::text_elide::elisao;
    let (mut m, id, c) = cena_campos();
    // Na frente o chip leva nome e valor; no verso, o NOME (o valor numa caixa de uma linha abre a
    // edição inteira ao clique).
    let cortados = |m: &mut Mesa, valor: bool| {
        let ((), medidos) = elisao::medindo(|| m.pinta(&mut ph2d_vector::VectorScene::new()));
        let nossos = |t: &str| {
            t.contains("Priority") || LONGO.0.contains(t) || (valor && LONGO.1.contains(t))
        };
        let v: Vec<_> = medidos
            .into_iter()
            .filter(|x| !x.coube() && nossos(&x.texto))
            .collect();
        v.into_iter().map(|x| x.texto).collect::<Vec<_>>()
    };
    assert_eq!(
        cortados(&mut m, true),
        Vec::<String>::new(),
        "a frente cortou"
    );
    // A altura medida (a da disposição) == a que a pintura percorre, com o chip quebrado.
    m.docs().probe_arm_fronts();
    m.quadro();
    m.pinta(&mut ph2d_vector::VectorScene::new());
    let pintada = m
        .docs()
        .probe_painted_front("Write the logline")
        .expect("pintado")
        .0;
    let medida = m.rect_cartao(id, c).h;
    assert!(
        (medida - pintada).abs() < 0.01,
        "medida {medida} == pintada {pintada}"
    );
    let r = pega(m.rect_cartao(id, c));
    m.clica(r);
    m.quadros(2);
    assert_eq!(
        cortados(&mut m, false),
        Vec::<String>::new(),
        "o verso cortou"
    );
}

/// 📷 As fotos (`#[ignore]`: a placa) em `docs/TrelloClone/smoke/D14/`.
/// `bash scripts/ph2d-run.sh cargo test -p ph2d-app-kanban --profile smoke --test it d14_foto -- --ignored`
#[test]
#[ignore = "a placa: fotos para docs/TrelloClone/smoke/D14/"]
fn d14_foto_campos() {
    let foto = |m: &mut Mesa, nome: &str| crate::j2_fotos::grava_em("D14", nome, &m.cena());
    let (mut m, id, c) = cena_campos();
    // Os cartões de baixo deslizam para o lugar novo (a mola): a foto espera que assentem.
    for _ in 0..90 {
        std::thread::sleep(std::time::Duration::from_millis(10));
        m.quadro();
    }
    foto(&mut m, "frente_campos_sem_corte");
    let r = pega(m.rect_cartao(id, c));
    m.clica(r);
    m.quadros(2);
    foto(&mut m, "verso_campos");
    toca(&mut m, id, &format!("FieldValue({LISTA:?})"));
    foto(&mut m, "verso_lista_aberta");
    toca(&mut m, id, &format!("FieldValue({LISTA:?})"));
    toca(&mut m, id, &format!("FieldDate({DATA:?})"));
    foto(&mut m, "verso_calendario_do_campo");
}
