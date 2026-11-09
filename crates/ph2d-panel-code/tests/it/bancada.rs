//! ⭐ **A Bancada** (plano 13 Onda B, itens 6–10): cada teste CONDUZ a vista pelos métodos reais
//! (pintar, pairar, carregar, teclas, relógio) e afirma o que se vê — a frase, os pedidos, o balão, o
//! espiar, a dobra — sobre os alvos que a própria pintura deixou. As funções puras e as marcas da
//! margem do código moram em [`puras`].

mod puras;

use crate::ts;
use ph2d_code_viewmodel::{Cell, CellKind, Column, Entry, Holder};
use ph2d_editor_core::doc_surface::Key;
use ph2d_editor_core::zones::Rect;
use ph2d_host::Modifiers;
use ph2d_panel_code::bancada::{Aim, BancadaView, Intent, PEEK_PAUSE_NS, Who, phrase_height};
use ph2d_panel_code::field::FieldKey;
use ph2d_text::TextSystem;
use ph2d_tokens::{ROW_H_PX, Spacing, Theme};
use ph2d_vector::VectorScene;

/// A grelha e o mapa, no topo; as marcas da margem bem abaixo dela (nada se sobrepõe).
const GRADE: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1000.0,
    h: 260.0,
};
const MARCA: Rect = Rect {
    x: 0.0,
    y: 600.0,
    w: 20.0,
    h: 20.0,
};
const LIMITES: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1000.0,
    h: 900.0,
};
const LONGE: (f32, f32) = (5000.0, 5000.0);

/// As colunas: `speed`, `lives` (números), `alive` (sim/não), `mood` (escolha), `tint` (cor) e
/// `fresh` (ainda por salvar: a cena não a conhece).
const SPEED: usize = 0;
const LIVES: usize = 1;
const ALIVE: usize = 2;
const MOOD: usize = 3;
const TINT: usize = 4;
const FRESH: usize = 5;

fn coluna(
    name: &str,
    line: usize,
    kind: Option<CellKind>,
    default: &str,
    options: &[&str],
) -> Column {
    Column {
        name: name.into(),
        line,
        kind,
        default: default.into(),
        options: options.iter().map(|o| (*o).to_owned()).collect(),
    }
}

fn celula(text: &str, changed: bool) -> Cell {
    Cell {
        text: text.into(),
        changed,
        swatch: None,
    }
}

/// «Hero» (cor 0) com os valores do script; «Hero 2» (cor 1) mudou `speed` (180), `alive` e `mood`
/// (a última opção, para a escolha dar a volta).
fn quem() -> Who {
    let branco = Cell {
        text: "1, 1, 1, 1".into(),
        changed: false,
        swatch: Some([1.0, 1.0, 1.0, 1.0]),
    };
    let quadrado = |x: f32| vec![vec![[x, 0.0], [x + 10.0, 0.0], [x + 10.0, 10.0], [x, 10.0]]];
    Who {
        holders: vec![
            Holder {
                bits: 1,
                name: "Hero".into(),
                color: 0,
            },
            Holder {
                bits: 2,
                name: "Hero 2".into(),
                color: 1,
            },
        ],
        columns: vec![
            coluna("speed", 0, Some(CellKind::Number), "220", &[]),
            coluna("lives", 1, Some(CellKind::Number), "3", &[]),
            coluna("alive", 2, Some(CellKind::Bool), "true", &[]),
            coluna(
                "mood",
                3,
                Some(CellKind::Choice),
                "calm",
                &["calm", "angry", "sleepy"],
            ),
            coluna("tint", 4, Some(CellKind::Color), "1, 1, 1, 1", &[]),
            coluna("fresh", 5, None, "0", &[]),
        ],
        cells: vec![
            celula("220", false),
            celula("3", false),
            celula("true", false),
            celula("calm", false),
            branco.clone(),
            celula("0", false),
            celula("180", true),
            celula("3", false),
            celula("false", true),
            celula("sleepy", true),
            branco,
            celula("0", false),
        ],
        functions: vec![Entry("on_signal".into(), 14), Entry("update".into(), 20)],
        hears: vec![Entry("coin".into(), 15)],
        map: vec![(0, quadrado(0.0)), (1, quadrado(20.0))],
    }
}

fn pinta_grade(v: &mut BancadaView, who: &Who, ts: &mut TextSystem) {
    let mut scene = VectorScene::new();
    v.paint_bench(who, GRADE, None, &mut scene, ts, &Theme::default());
}

fn meio(r: Rect) -> (f32, f32) {
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

fn alvo(v: &BancadaView, aim: Aim) -> (f32, f32) {
    meio(
        v.target(aim)
            .unwrap_or_else(|| panic!("o alvo {aim:?} foi pintado no último quadro")),
    )
}

fn carrega(v: &mut BancadaView, who: &Who, aim: Aim) -> Vec<Intent> {
    let (x, y) = alvo(v, aim);
    assert!(v.press(who, x, y), "o carregar em {aim:?} é da Bancada");
    v.take_intents()
}

fn tecla(key: Key) -> FieldKey<'static> {
    FieldKey {
        key,
        mods: Modifiers::default(),
        text: None,
        paste: None,
    }
}

fn escreve(s: &'static str) -> FieldKey<'static> {
    FieldKey {
        key: Key::Char(s.chars().next().unwrap_or(' ')),
        mods: Modifiers::default(),
        text: Some(s),
        paste: None,
    }
}

fn ctrl(c: char) -> FieldKey<'static> {
    FieldKey {
        key: Key::Char(c),
        mods: Modifiers {
            ctrl: true,
            ..Modifiers::default()
        },
        text: None,
        paste: None,
    }
}

#[test]
fn pairar_a_celula_mudada_acende_a_e_a_frase_diz_o_valor_e_o_do_script() {
    let (who, mut ts) = (quem(), ts());
    let mut v = BancadaView::default();
    pinta_grade(&mut v, &who, &mut ts);
    let (x, y) = alvo(&v, Aim::Cell(1, SPEED));
    assert!(v.hover(x, y, 0), "a célula é uma peça da Bancada");
    assert_eq!(v.aim(), Aim::Cell(1, SPEED));
    let mut scene = VectorScene::new();
    v.paint_phrase(
        &who,
        "hero.luau",
        Rect::new(0.0, 800.0, 1000.0, phrase_height()),
        &mut scene,
        &mut ts,
        &Theme::default(),
    );
    assert!(
        v.said()
            .contains("Hero 2 \u{00b7} speed is 180 here (220 in the script)"),
        "a frase: {:?}",
        v.said()
    );
    // Controlo: sem nada sob o rato, a frase volta ao resumo do script.
    assert!(!v.hover(LONGE.0, LONGE.1, 1));
    assert_eq!(v.aim(), Aim::Nothing);
    v.paint_phrase(
        &who,
        "hero.luau",
        Rect::new(0.0, 800.0, 1000.0, phrase_height()),
        &mut scene,
        &mut ts,
        &Theme::default(),
    );
    assert!(
        v.said().starts_with("hero.luau runs on 2 objects"),
        "a frase: {:?}",
        v.said()
    );
}

#[test]
fn a_celula_de_numero_escreve_se_e_o_enter_pede_a_edicao() {
    let (who, mut ts) = (quem(), ts());
    let mut v = BancadaView::default();
    pinta_grade(&mut v, &who, &mut ts);
    assert!(
        carrega(&mut v, &who, Aim::Cell(0, SPEED)).is_empty(),
        "o clique só abre o campo"
    );
    assert!(v.has_focus(), "a célula em escrita tem o teclado");
    for k in [ctrl('a'), escreve("3"), escreve("0"), escreve("0")] {
        assert!(
            v.key(k).is_some_and(|o| o.consumed),
            "a célula toma a tecla"
        );
    }
    assert!(v.take_intents().is_empty(), "nada se pede antes do Enter");
    assert!(v.key(tecla(Key::Enter)).is_some_and(|o| o.consumed));
    assert_eq!(v.take_intents(), vec![Intent::Edit(0, SPEED, "300".into())]);
    assert!(
        v.has_focus(),
        "o último clique foi na Bancada: o Ctrl+Z é do projecto"
    );
    assert!(
        v.key(tecla(Key::Enter)).is_none(),
        "fechado o campo, o Enter não é daqui"
    );
    assert!(v.take_intents().is_empty(), "o Enter não repete a edição");
}

#[test]
fn o_esc_desiste_da_celula_sem_pedir_nada() {
    let (who, mut ts) = (quem(), ts());
    let mut v = BancadaView::default();
    pinta_grade(&mut v, &who, &mut ts);
    assert!(carrega(&mut v, &who, Aim::Cell(0, LIVES)).is_empty());
    for k in [ctrl('a'), escreve("9"), tecla(Key::Escape)] {
        assert!(v.key(k).is_some_and(|o| o.consumed));
    }
    assert!(v.take_intents().is_empty(), "o Esc cancela");
    assert!(
        v.key(escreve("7")).is_none(),
        "o campo fechou: a tecla volta ao texto"
    );
}

#[test]
fn clicar_noutra_celula_ou_fora_guarda_a_que_se_escrevia() {
    let (who, mut ts) = (quem(), ts());
    let mut v = BancadaView::default();
    pinta_grade(&mut v, &who, &mut ts);
    assert!(carrega(&mut v, &who, Aim::Cell(0, SPEED)).is_empty());
    for k in [ctrl('a'), escreve("2"), escreve("5"), escreve("0")] {
        v.key(k);
    }
    assert_eq!(
        carrega(&mut v, &who, Aim::Cell(1, LIVES)),
        vec![Intent::Edit(0, SPEED, "250".into())],
        "o clique noutra célula guarda a primeira e abre a segunda"
    );
    v.blur();
    assert_eq!(v.take_intents(), vec![Intent::Edit(1, LIVES, "3".into())]);
    assert!(!v.has_focus(), "o carregar fora devolve o teclado ao texto");
    assert!(
        !v.press(&who, LONGE.0, LONGE.1),
        "longe da Bancada nada é tomado"
    );
}

#[test]
fn sim_nao_troca_escolha_passa_a_seguinte_cor_seleciona_e_por_salvar_nada() {
    let (who, mut ts) = (quem(), ts());
    let mut v = BancadaView::default();
    pinta_grade(&mut v, &who, &mut ts);
    assert_eq!(
        carrega(&mut v, &who, Aim::Cell(0, ALIVE)),
        vec![Intent::Edit(0, ALIVE, "false".into())]
    );
    assert_eq!(
        carrega(&mut v, &who, Aim::Cell(1, ALIVE)),
        vec![Intent::Edit(1, ALIVE, "true".into())]
    );
    assert_eq!(
        carrega(&mut v, &who, Aim::Cell(0, MOOD)),
        vec![Intent::Edit(0, MOOD, "angry".into())]
    );
    assert_eq!(
        carrega(&mut v, &who, Aim::Cell(1, MOOD)),
        vec![Intent::Edit(1, MOOD, "calm".into())],
        "da última opção dá a volta à primeira"
    );
    assert_eq!(
        carrega(&mut v, &who, Aim::Cell(1, TINT)),
        vec![Intent::Select(1)]
    );
    assert!(
        carrega(&mut v, &who, Aim::Cell(0, FRESH)).is_empty(),
        "a cena ainda não conhece «fresh»"
    );
    assert!(
        v.key(escreve("1")).is_none(),
        "nenhum destes cliques abriu um campo"
    );
    // O nome do objecto selecciona-o; o da propriedade salta para a linha dela.
    assert_eq!(
        carrega(&mut v, &who, Aim::Holder(1)),
        vec![Intent::Select(1)]
    );
    assert_eq!(
        carrega(&mut v, &who, Aim::Column(MOOD)),
        vec![Intent::Jump(3)]
    );
}

/// A vista com a grelha pintada, a marca da linha 0 e o balão aberto por pairar sobre ela.
fn balao_aberto(who: &Who, ts: &mut TextSystem) -> BancadaView {
    let mut v = BancadaView::default();
    pinta_grade(&mut v, who, ts);
    v.set_marks(&[(0, MARCA)]);
    let (x, y) = meio(MARCA);
    assert!(v.hover(x, y, 0));
    assert_eq!(v.popover_line(), Some(0), "pairar a marca abre o balão");
    let mut scene = VectorScene::new();
    v.paint_popover(who, LIMITES, &mut scene, ts, &Theme::default());
    v
}

#[test]
fn o_balao_da_marca_abre_ao_pairar_e_o_select_e_do_primeiro_que_mudou() {
    let (who, mut ts) = (quem(), ts());
    let mut v = balao_aberto(&who, &mut ts);
    assert!(
        v.target(Aim::Select(0)).is_none(),
        "«Hero» não mudou a speed"
    );
    let (x, y) = alvo(&v, Aim::Select(1));
    assert!(v.hover(x, y, 1), "o rato no balão não o fecha");
    assert_eq!(v.popover_line(), Some(0));
    assert_eq!(v.aim(), Aim::Select(1));
    assert!(v.press(&who, x, y));
    assert_eq!(v.take_intents(), vec![Intent::Select(1)]);
}

#[test]
fn o_balao_solto_fecha_ao_afastar_o_rato() {
    use ph2d_panel_code::bancada::POP_HIDE_NS;
    let (who, mut ts) = (quem(), ts());
    let mut v = balao_aberto(&who, &mut ts);
    assert!(!v.hover(LONGE.0, LONGE.1, 1));
    assert_eq!(v.popover_line(), Some(0), "a caminho, o balão espera");
    v.tick(POP_HIDE_NS);
    assert_eq!(v.popover_line(), Some(0), "ainda dentro da espera");
    v.tick(1 + POP_HIDE_NS);
    assert_eq!(v.popover_line(), None, "passada a espera, fecha sozinho");
}

/// Smoke do dono 09/11: o balão encosta-se ao canto da marca (na margem), e o rato que vai da
/// marca até ele — mesmo cruzando o que não é nem um nem outro — não o fecha.
#[test]
fn o_balao_encosta_na_marca_e_o_caminho_ate_ele_nao_o_fecha() {
    use ph2d_panel_code::bancada::POP_HIDE_NS;
    let (who, mut ts) = (quem(), ts());
    let mut v = balao_aberto(&who, &mut ts);
    let caixa = v.target(Aim::Mark(0)).expect("o corpo do balão");
    assert!(
        (caixa.x - (MARCA.x + MARCA.w)).abs() < 0.5 && (caixa.y - (MARCA.y + MARCA.h)).abs() < 0.5,
        "no canto de baixo à direita da marca, encostado: {caixa:?} × {MARCA:?}"
    );
    // Um píxel fora dos dois (o vão), e depois dentro do balão antes da espera acabar.
    v.hover(MARCA.x + MARCA.w + 1.0, MARCA.y - 1.0, 10);
    v.hover(
        caixa.x + caixa.w * 0.5,
        caixa.y + caixa.h * 0.5,
        9 + POP_HIDE_NS,
    );
    v.tick(10 + POP_HIDE_NS * 2);
    assert_eq!(v.popover_line(), Some(0), "o rato chegou ao balão: fica");
}

#[test]
fn o_clique_na_marca_prende_o_balao_e_o_esc_fecha_o() {
    let (who, mut ts) = (quem(), ts());
    let mut v = balao_aberto(&who, &mut ts);
    let (x, y) = meio(MARCA);
    assert!(v.press(&who, x, y));
    assert!(v.take_intents().is_empty(), "prender não pede nada");
    v.hover(LONGE.0, LONGE.1, 1);
    assert_eq!(v.popover_line(), Some(0), "preso, sobrevive ao rato longe");
    assert!(
        v.key(tecla(Key::Escape)).is_some_and(|o| o.consumed),
        "o Esc é do balão preso"
    );
    assert_eq!(v.popover_line(), None);
    assert!(
        v.key(tecla(Key::Escape)).is_none(),
        "sem balão o Esc não é daqui"
    );
    // Prender e soltar pela própria marca: o segundo clique fecha-o; pairar reabre-o solto.
    v.hover(x, y, 2);
    v.press(&who, x, y);
    v.hover(LONGE.0, LONGE.1, 3);
    assert_eq!(v.popover_line(), Some(0));
    v.hover(x, y, 4);
    v.press(&who, x, y);
    assert_eq!(v.popover_line(), None, "o segundo clique na marca solta-o");
    v.hover(x + 1.0, y, 5);
    assert_eq!(v.popover_line(), Some(0), "o rato na marca reabre-o");
    v.hover(LONGE.0, LONGE.1, 6);
    v.tick(6 + ph2d_panel_code::bancada::POP_HIDE_NS);
    assert_eq!(v.popover_line(), None, "reaberto solto, fecha ao afastar");
}

#[test]
fn o_clique_no_corpo_do_balao_prende_o_e_o_clique_fora_solta_o() {
    let (who, mut ts) = (quem(), ts());
    let mut v = balao_aberto(&who, &mut ts);
    // O primeiro alvo `Mark(0)` é o balão inteiro (os alvos do balão vêm à frente das marcas).
    let corpo = v.target(Aim::Mark(0)).expect("o balão aberto é um alvo");
    assert!(
        corpo.x >= MARCA.x + MARCA.w,
        "o balão encosta à direita da marca: {corpo:?}"
    );
    let (x, y) = (corpo.x + 1.0, corpo.y + 1.0);
    assert!(v.hover(x, y, 1), "o rato no balão é da Bancada");
    assert_eq!(v.popover_line(), Some(0));
    assert!(v.press(&who, x, y));
    assert!(v.take_intents().is_empty(), "o corpo do balão só o prende");
    v.hover(LONGE.0, LONGE.1, 2);
    assert_eq!(v.popover_line(), Some(0), "preso pelo corpo");
    v.blur();
    assert_eq!(
        v.popover_line(),
        None,
        "um carregar fora da Bancada fecha o balão preso"
    );
    // A linha de um objecto no balão selecciona-o.
    let mut v = balao_aberto(&who, &mut ts);
    assert_eq!(
        carrega(&mut v, &who, Aim::Holder(0)),
        vec![Intent::Select(0)]
    );
}

#[test]
fn espiar_a_cena_comeca_na_pausa_e_acaba_ao_sair_do_nome() {
    let (who, mut ts) = (quem(), ts());
    let mut v = BancadaView::default();
    pinta_grade(&mut v, &who, &mut ts);
    let t = 1_000_000_000;
    let (x, y) = alvo(&v, Aim::Holder(1));
    assert!(v.hover(x, y, t));
    v.tick(t + PEEK_PAUSE_NS - 1);
    assert_eq!(v.peeking(), None, "antes da pausa, nada");
    v.tick(t + PEEK_PAUSE_NS);
    assert_eq!(v.peeking(), Some(1), "na pausa, a cena de «Hero 2»");
    let mut scene = VectorScene::new();
    v.paint_phrase(
        &who,
        "hero.luau",
        Rect::new(0.0, 800.0, 1000.0, phrase_height()),
        &mut scene,
        &mut ts,
        &Theme::default(),
    );
    assert!(
        v.said().starts_with("Hero 2 in the scene"),
        "a frase: {:?}",
        v.said()
    );
    assert!(
        v.hover(x + 1.0, y, t + PEEK_PAUSE_NS + 1),
        "dentro do nome o espiar continua"
    );
    assert_eq!(v.peeking(), Some(1));
    v.hover(LONGE.0, LONGE.1, t + PEEK_PAUSE_NS + 2);
    assert_eq!(v.peeking(), None, "fora do nome a aba volta");
}

#[test]
fn com_o_espiar_desligado_a_cena_nunca_aparece() {
    let (who, mut ts) = (quem(), ts());
    let mut v = BancadaView::default();
    v.peek_on = false;
    pinta_grade(&mut v, &who, &mut ts);
    let t = 1_000_000_000;
    let (x, y) = alvo(&v, Aim::Holder(1));
    v.hover(x, y, t);
    v.tick(t + PEEK_PAUSE_NS * 10);
    assert_eq!(v.peeking(), None);
    assert_eq!(
        v.aim(),
        Aim::Holder(1),
        "controlo: o rato está mesmo no nome"
    );
}

#[test]
fn a_dobra_deixa_so_o_titulo() {
    let (who, mut ts) = (quem(), ts());
    let mut v = BancadaView::default();
    pinta_grade(&mut v, &who, &mut ts);
    let aberta = v.bench_height(&who, 900.0);
    let (x, y) = meio(v.fold_rect());
    assert!(v.press(&who, x, y));
    assert!(v.folded);
    let dobrada = v.bench_height(&who, 900.0);
    let titulo = Spacing::Sm.px() * 2.0 + ROW_H_PX;
    assert!(
        (dobrada - titulo).abs() < 1e-3,
        "dobrada {dobrada} ≠ título {titulo}"
    );
    assert!(dobrada < aberta, "dobrada {dobrada} < aberta {aberta}");
    pinta_grade(&mut v, &who, &mut ts);
    assert!(
        v.target(Aim::Cell(0, SPEED)).is_none(),
        "dobrada não há células"
    );
    assert!(v.target(Aim::Holder(1)).is_none(), "nem o mapa");
    let (x, y) = meio(v.fold_rect());
    assert!(v.press(&who, x, y));
    assert!(!v.folded);
    assert!((v.bench_height(&who, 900.0) - aberta).abs() < 1e-3);
}
