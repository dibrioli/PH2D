//! A costura do fundo REAL (10/11): cada sítio que pinta na mono fora do poço entrega ao
//! escurecimento o fundo que está debaixo do texto. Nenhum tema muda a decisão para estas tintas, por
//! isso a costura lê-se no gravador [`bg_probe`] (o fundo de cada `paint_mono`), não na cena.

use ph2d_code_buffer::Buffer;
use ph2d_code_viewmodel::{Entry, Holder};
use ph2d_editor_core::paint::resolve;
use ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread;
use ph2d_editor_core::widget::section_cards::CardDepth;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_panel_code::bancada::{Aim, BancadaView, Who};
use ph2d_panel_code::help::{Block, HelpDoc, Span, SpanKind};
use ph2d_panel_code::{
    CodeView, HelpFrame, HelpView, Ink, ListRow, ScriptList, ScriptListView, bg_probe,
};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Color, VectorScene};

/// As cores das peças que a fonte usa (`chrome::Ui`): (fundo, linha acesa, sob o rato).
fn ui(cartoes: bool, theme: Theme) -> (Color, Color, Color) {
    let c = |t| resolve(t, theme);
    if cartoes {
        (
            c(ColorToken::KanbanCard),
            c(ColorToken::KanbanCardRaised),
            c(ColorToken::CardFieldHover),
        )
    } else {
        (
            CardDepth::Section.fill(theme),
            c(ColorToken::AccentSoft),
            c(ColorToken::Bg2),
        )
    }
}

/// O poço do código e a caixa de um exemplo (`palette::Well`): (fundo, caixa).
fn poco(cartoes: bool, theme: Theme) -> (Color, Color) {
    let c = |t| resolve(t, theme);
    if cartoes {
        (
            c(ph2d_panel_code::cards::card_tone()),
            c(ColorToken::CardField),
        )
    } else {
        (c(ColorToken::CodeBg), c(ColorToken::Bg3))
    }
}

/// Os fundos gravados de `text` (texto exacto).
fn fundos(rec: &[(String, Color)], text: &str) -> Vec<Color> {
    rec.iter()
        .filter(|(t, _)| t == text)
        .map(|(_, c)| *c)
        .collect()
}

/// Corre `f` nos dois modos (cartões e clássico), com o modo reposto no fim.
fn nos_dois_modos(mut f: impl FnMut(bool)) {
    for cartoes in [true, false] {
        force_cards_on_this_thread(Some(cartoes));
        f(cartoes);
    }
    force_cards_on_this_thread(None);
}

/// (a) A lista de scripts: o nome da linha corrente decide pela linha acesa, o outro pelo cartão.
#[test]
fn o_nome_na_lista_escurece_pelo_fundo_da_sua_linha() {
    let mut ts = TextSystem::without_system_fonts();
    nos_dois_modos(|cartoes| {
        let theme = Theme::default();
        let (bg, on, _) = ui(cartoes, theme);
        assert_ne!(bg, on, "cartões {cartoes}: os dois fundos coincidem");
        let filtro = Buffer::new("");
        let linha = |id, name, current| ListRow {
            id,
            name,
            dirty: false,
            current,
            users: 1,
        };
        let linhas = [
            linha(1, "corrente.luau", true),
            linha(2, "outro.luau", false),
        ];
        let lista = ScriptList {
            open: true,
            filter: &filtro,
            focus: None,
            rows: &linhas,
            renaming: None,
            notice: None,
            hover_count: None,
        };
        let mut l = ScriptListView::default();
        let la = Rect::new(0.0, 0.0, 240.0, ScriptListView::height_for(2));
        bg_probe::start();
        l.paint(&lista, la, &mut VectorScene::new(), &mut ts, &theme);
        let rec = bg_probe::take();
        assert_eq!(
            fundos(&rec, "corrente.luau"),
            [on],
            "cartões {cartoes}: {rec:?}"
        );
        assert_eq!(
            fundos(&rec, "outro.luau"),
            [bg],
            "cartões {cartoes}: {rec:?}"
        );
    });
}

/// Um objecto com duas funções e nenhuma propriedade: entradas no «In this script» e a frase
/// «sem propriedades» na grelha.
fn quem() -> Who {
    Who {
        holders: vec![Holder {
            bits: 1,
            name: "Hero".into(),
            color: 0,
        }],
        functions: vec![Entry("on_signal".into(), 14), Entry("update".into(), 20)],
        ..Who::default()
    }
}

/// (b) O «In this script»: a entrada sob o rato decide pelo realce, a outra pelo cartão.
#[test]
fn a_entrada_do_in_this_script_escurece_pelo_realce_sob_o_rato() {
    let mut ts = TextSystem::without_system_fonts();
    nos_dois_modos(|cartoes| {
        let theme = Theme::default();
        let (bg, _, hover) = ui(cartoes, theme);
        assert_ne!(bg, hover, "cartões {cartoes}: os dois fundos coincidem");
        let (who, mut v) = (quem(), BancadaView::default());
        let area = Rect::new(0.0, 0.0, 300.0, 600.0);
        v.paint_outline(&who, area, &mut VectorScene::new(), &mut ts, &theme);
        let r = v.target(Aim::Entry(20)).expect("a entrada `update` é alvo");
        v.hover(r.x + r.w * 0.5, r.y + r.h * 0.5, 0);
        assert_eq!(v.aim(), Aim::Entry(20), "o rato não mira a entrada");
        bg_probe::start();
        v.paint_outline(&who, area, &mut VectorScene::new(), &mut ts, &theme);
        let rec = bg_probe::take();
        assert_eq!(
            fundos(&rec, "update"),
            [hover],
            "cartões {cartoes}: {rec:?}"
        );
        assert_eq!(
            fundos(&rec, "on_signal"),
            [bg],
            "cartões {cartoes}: {rec:?}"
        );
    });
}

/// (c) A grelha sem propriedades: a frase decide pelo fundo do cartão.
#[test]
fn a_frase_sem_propriedades_escurece_pelo_cartao() {
    let mut ts = TextSystem::without_system_fonts();
    nos_dois_modos(|cartoes| {
        let theme = Theme::default();
        let (bg, on, hover) = ui(cartoes, theme);
        assert!(
            bg != on && bg != hover,
            "cartões {cartoes}: o cartão coincide com outra superfície"
        );
        let (who, mut v) = (quem(), BancadaView::default());
        let grade = Rect::new(0.0, 0.0, 1000.0, 260.0);
        let mut view = CodeView::new();
        bg_probe::start();
        v.paint_bench(
            &who,
            grade,
            None,
            &mut view,
            &mut VectorScene::new(),
            &mut ts,
            &theme,
        );
        let rec = bg_probe::take();
        let frase = tr("code.bancada.grid.no_props");
        assert_eq!(fundos(&rec, frase), [bg], "cartões {cartoes}: {rec:?}");
    });
}

/// (d) A ajuda: o exemplo dentro da caixa decide pela caixa; o código no meio da prosa, pelo poço.
#[test]
fn o_exemplo_da_ajuda_escurece_pela_caixa_e_o_codigo_da_prosa_pelo_poco() {
    let mut ts = TextSystem::without_system_fonts();
    nos_dois_modos(|cartoes| {
        let theme = Theme::default();
        let (well, caixa) = poco(cartoes, theme);
        assert_ne!(well, caixa, "cartões {cartoes}: os dois fundos coincidem");
        let doc = HelpDoc {
            key: 1,
            title: "math".into(),
            blocks: vec![
                Block::Title("math".into()),
                Block::Body(vec![
                    Span::new("Read about ", SpanKind::Prose),
                    Span::new("math.floor", SpanKind::Code(Ink::Function)),
                    Span::new(" here.", SpanKind::Prose),
                ]),
                Block::Code {
                    lines: vec![vec![Span::new(
                        "local a = math.floor(1)",
                        SpanKind::Code(Ink::Plain),
                    )]],
                    copy: 0,
                },
            ],
            sections: vec![],
        };
        let frame = HelpFrame {
            doc: &doc,
            back: false,
            forward: false,
            hover: None,
            focus_link: None,
            selection: None,
            find: None,
            matches: &[],
            current: None,
        };
        let mut v = HelpView::new();
        let area = Rect::new(0.0, 0.0, 1000.0, 400.0);
        bg_probe::start();
        v.paint(&frame, area, &mut VectorScene::new(), &mut ts, &theme);
        let rec = bg_probe::take();
        let no_exemplo: Vec<Color> = rec
            .iter()
            .filter(|(t, _)| t.contains("local") || t.contains("floor(1)"))
            .map(|(_, c)| *c)
            .collect();
        assert!(!no_exemplo.is_empty(), "cartões {cartoes}: {rec:?}");
        assert!(
            no_exemplo.iter().all(|&c| c == caixa),
            "cartões {cartoes}: {rec:?}"
        );
        assert_eq!(
            fundos(&rec, "math.floor"),
            [well],
            "cartões {cartoes}: {rec:?}"
        );
    });
}
