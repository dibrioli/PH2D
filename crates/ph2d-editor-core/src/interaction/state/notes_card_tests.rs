//! ⭐⭐ **O CARTÃO de uma nota e o CROMO de uma secção** (2026-10-01, ordem do dono: *«a outline
//! não está englobando as notas da seção. faça englobar. … O card das notas não cresce com o
//! aumento do número de palavras e linhas»*).

use crate::ids::{self, note_ids};
use crate::interaction::{HitIndex, InteractiveState, WidgetStore};
use crate::widget::showcase::notes_chrome::{caixa_do_contorno, fecha_seccao, pinta_as_que_sobram};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

const P: NodeId = ids::PHYSICS_PANEL;
const S: NodeId = ids::INSP_LIVE_TRANSFORM_SECTION;

fn rect_de(hit: &HitIndex, id: NodeId) -> Option<Rect> {
    hit.iter_registrations()
        .find(|(i, _)| *i == id)
        .map(|(_, r)| r)
}

fn escreve_corpo(store: &mut WidgetStore, slot: usize, texto: &str) {
    if let Some(InteractiveState::TextInput { text, .. }) = store.get_mut(note_ids(P).body[slot]) {
        *text = texto.to_string();
    }
}

/// Pinta a nota `0` e devolve quanto ela ocupou e o índice do quadro.
fn altura_da_nota(store: &WidgetStore) -> (f32, HitIndex) {
    let mut scene = VectorScene::new();
    let mut ts = TextSystem::without_system_fonts();
    let mut hit = HitIndex::new();
    let mut y = 0.0;
    crate::widget::showcase::paint_one_note(
        &mut scene,
        &mut ts,
        &mut hit,
        store,
        0.0,
        240.0,
        &mut y,
        &store.notes_for_panel(P)[0],
        &note_ids(P),
        0,
    );
    (y, hit)
}

/// ⭐⭐ **O contorno abraça a secção E as notas dela.** A nota pinta-se abaixo do fim da secção, e
/// a caixa do contorno desce até ao fim dela. *Mutação: a caixa até ao fim da SECÇÃO (o de antes)
/// ⇒ a nota fica de fora — é o controlo `pintou = false`.*
#[test]
fn o_contorno_abraca_as_notas_da_seccao() {
    let mut store = WidgetStore::with_capacity(64);
    assert_eq!(store.notes_push(P, 0, Some(S)), Some(0));
    store.set_section_outline_color(S, Some(0));
    let mut scene = VectorScene::new();
    let mut ts = TextSystem::without_system_fonts();
    let mut hit = HitIndex::new();
    let (topo, fim_da_seccao) = (100.0, 200.0);
    let fim = fecha_seccao(
        &mut scene,
        &mut ts,
        &mut hit,
        &store,
        P,
        S,
        0.0,
        300.0,
        topo,
        fim_da_seccao,
    );
    let nota = rect_de(&hit, note_ids(P).slot[0]).expect("a nota da secção foi pintada");
    assert!(
        hit.note_painted(P, 0),
        "a nota não entrou no livro do quadro"
    );
    assert!(nota.y >= fim_da_seccao, "a nota não ficou abaixo da secção");
    let caixa = caixa_do_contorno(0.0, 300.0, topo, fim_da_seccao, fim, true);
    assert!(
        caixa.y <= topo && caixa.y + caixa.h >= nota.y + nota.h,
        "o contorno ({caixa:?}) não abraça a nota ({nota:?})"
    );
    let curta = caixa_do_contorno(0.0, 300.0, topo, fim_da_seccao, fim, false);
    assert!(
        curta.y + curta.h < nota.y + nota.h,
        "controlo: até ao fim da secção a nota fica de fora"
    );
}

/// ⭐⭐ **O cartão CRESCE com o texto** — uma frase comprida parte-se em linhas e o cartão
/// acompanha; o `\n` também conta. *Mutação: o corpo com três linhas fixas ⇒ as três alturas são
/// iguais.*
#[test]
fn o_cartao_cresce_com_o_texto() {
    let mut store = WidgetStore::with_capacity(64);
    store.notes_push(P, 0, None);
    let (vazio, _) = altura_da_nota(&store);
    escreve_corpo(&mut store, 0, &"palavra ".repeat(60));
    let (comprido, hit) = altura_da_nota(&store);
    let linha = crate::widget::text_area_metrics(Rect::new(0.0, 0.0, 200.0, 0.0)).line_h;
    assert!(
        comprido >= vazio + linha * 2.0,
        "o cartão não cresceu com o texto ({vazio} -> {comprido})"
    );
    let corpo = rect_de(&hit, note_ids(P).body[0]).expect("o corpo foi registado");
    assert!(
        corpo.h > linha * 3.0,
        "o corpo ficou com três linhas ({corpo:?})"
    );
    escreve_corpo(&mut store, 0, "a\nb\nc\nd\ne\nf");
    let (seis, _) = altura_da_nota(&store);
    assert!(
        seis > vazio,
        "seis linhas lógicas não fizeram o cartão crescer"
    );
}

/// ⭐⭐ **Minimizada, a nota é só a fileira do título** — sem corpo registado, mais baixa, e o botão
/// continua lá para a voltar a abrir. *Mutação: ignorar o `minimized` no pintor ⇒ o corpo continua
/// registado.*
#[test]
fn minimizada_e_so_a_fileira_do_titulo() {
    let mut store = WidgetStore::with_capacity(64);
    store.notes_push(P, 0, None);
    let (aberta, _) = altura_da_nota(&store);
    store.note_toggle_minimized(P, 0);
    let (fechada, hit) = altura_da_nota(&store);
    assert!(fechada < aberta, "minimizar não encolheu o cartão");
    assert!(
        rect_de(&hit, note_ids(P).body[0]).is_none(),
        "o corpo continua lá"
    );
    assert!(
        rect_de(&hit, note_ids(P).fold[0]).is_some(),
        "o botão desapareceu"
    );
}

/// ⭐ **As notas do fim do corpo correm UMA vez por painel por quadro** — um painel que as chama do
/// próprio corpo e depois fecha pela porta de rolagem não pinta duas vezes (nem o fantasma). E é a
/// 1.ª chamada que declara o painel anfitrião. *Mutação: sem a guarda ⇒ a 2.ª chamada pinta a nota
/// que nasceu entre as duas.*
#[test]
fn as_notas_do_fim_correm_uma_vez_por_quadro() {
    let mut store = WidgetStore::with_capacity(64);
    store.notes_push(P, 0, None);
    let mut scene = VectorScene::new();
    let mut ts = TextSystem::without_system_fonts();
    let mut hit = HitIndex::new();
    let tema = ph2d_tokens::Theme::default();
    let y1 = pinta_as_que_sobram(
        &mut scene, &mut ts, &mut hit, &store, P, tema, 0.0, 300.0, 10.0,
    );
    assert!(
        hit.is_note_host(P),
        "a porta não declarou o painel anfitrião"
    );
    assert!(y1 > 10.0 && hit.note_painted(P, 0));
    store.notes_push(P, 0, None);
    let y2 = pinta_as_que_sobram(
        &mut scene, &mut ts, &mut hit, &store, P, tema, 0.0, 300.0, y1,
    );
    assert_eq!(y2, y1, "a 2.ª chamada no mesmo quadro pintou outra vez");
    assert!(!hit.note_painted(P, 1));
}
