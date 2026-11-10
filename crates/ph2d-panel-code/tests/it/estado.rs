//! A barra de estado: posição 1-based (tabs até à paragem), problemas e o selo do salvar.

use crate::{Deco, frame, spans_de};
use ph2d_code_buffer::{Buffer, Range, Selection};
use ph2d_panel_code::{Badge, Problem, Severity, StatusLine, Tone};

fn status(buf: &Buffer, d: &Deco) -> StatusLine {
    let mut f = |r: std::ops::Range<usize>| spans_de(buf, r);
    let mut m = crate::marcas(d);
    StatusLine::of(&frame(buf, d, &mut f, &mut m))
}

fn p(severity: Severity) -> Problem {
    Problem {
        line: 0,
        severity,
        culprit: None,
        message: String::new(),
        on: String::new(),
        explanation: None,
    }
}

#[test]
fn posicao_problemas_e_selo() {
    let mut buf = Buffer::new("a\n\tif x\n");
    buf.set_selection(Selection::single(Range::cursor(
        buf.text().byte_of_line(1) + 1,
    )));
    let s = status(&buf, &Deco::default());
    assert_eq!(s.position, "Ln 2, Col 5");
    assert_eq!((s.counter.errors, s.counter.warnings), (0, 0));
    assert_eq!(s.counter.label, "Errors: 0, Warnings: 0");
    assert_eq!(s.badge, None);
    let d = Deco {
        problems: vec![
            p(Severity::Error),
            p(Severity::Warning),
            p(Severity::Warning),
        ],
        badge: Some(Badge::Reloaded { objects: 3 }),
        ..Deco::default()
    };
    let s = status(&buf, &d);
    assert_eq!((s.counter.errors, s.counter.warnings), (1, 2));
    assert_eq!(s.counter.label, "Errors: 1, Warnings: 2");
    assert_eq!(
        s.badge,
        Some(("Reloaded · 3 objects".to_owned(), Tone::Success))
    );
    let d = Deco {
        problems: vec![p(Severity::Warning)],
        badge: Some(Badge::Refused { line: Some(7) }),
        ..Deco::default()
    };
    let s = status(&buf, &d);
    assert_eq!((s.counter.errors, s.counter.warnings), (0, 1));
    let (texto, tom) = s.badge.expect("selo");
    assert!(texto.contains("line 7"), "{texto}");
    assert_eq!(tom, Tone::Error);
}

/// O assistente de código só fala quando NÃO está pronto, e diz que a edição continua.
#[test]
fn o_assistente_so_aparece_quando_nao_esta_pronto() {
    use ph2d_panel_code::Assist;
    let buf = Buffer::new("a\n");
    assert_eq!(status(&buf, &Deco::default()).assist, None);
    let mut textos = std::collections::BTreeSet::new();
    for (a, tom) in [
        (Assist::Starting, Tone::Plain),
        (Assist::Restarting, Tone::Warning),
        (Assist::Missing, Tone::Warning),
        (Assist::Stopped, Tone::Warning),
    ] {
        let d = Deco {
            assist: Some(a),
            ..Deco::default()
        };
        let (texto, t) = status(&buf, &d).assist.expect("peça do assistente");
        if a == Assist::Stopped {
            assert!(texto.contains("editing works"), "{a:?}: {texto}");
        }
        // Sem o programa, a barra diz o que se perde (crítica 08/11: «not installed» não dizia).
        if a == Assist::Missing {
            for perdido in ["suggestions", "help", "rename"] {
                assert!(
                    texto.contains(perdido),
                    "{a:?} não diz «{perdido}»: {texto}"
                );
            }
            assert!(!texto.contains("not installed"), "{texto}");
        }
        assert_eq!(t, tom, "{a:?}");
        textos.insert(texto);
    }
    // Cada estado diz uma coisa sua (uma chave trocada daria dois textos iguais).
    assert_eq!(textos.len(), 4, "{textos:?}");
}

/// ⛔ «Console · 1 errors» (foto `code_smoke_2.png`): a forma de UM. O contador da barra conta só
/// erros e avisos (o molde do leitor de ecrã do VS Code não tem plural).
#[test]
fn uma_contagem_de_um_fala_no_singular() {
    let buf = Buffer::new("a\n");
    let com = |problems: Vec<Problem>| {
        status(
            &buf,
            &Deco {
                problems,
                ..Deco::default()
            },
        )
        .counter
        .label
    };
    assert_eq!(com(vec![p(Severity::Error)]), "Errors: 1, Warnings: 0");
    // Informação e dica pintam-se, mas não são problemas que a barra conte.
    assert_eq!(com(vec![p(Severity::Info)]), "Errors: 0, Warnings: 0");
    assert_eq!(
        com(vec![p(Severity::Info), p(Severity::Warning)]),
        "Errors: 0, Warnings: 1"
    );
    assert_eq!(ph2d_panel_code::problems_title(0), "Problems");
    assert_eq!(ph2d_panel_code::problems_title(5), "Problems (5)");
    // Onda A: os erros contam-se nos Problems, o console já não os conta.
    assert_eq!(ph2d_panel_code::console_title_text(), "Console");
    let um = ph2d_panel_code::FindCount {
        current: None,
        total: 1,
    };
    assert_eq!(
        ph2d_panel_code::count_text(Some(um)).as_deref(),
        Some("1 result")
    );
}

/// I14: com `--!strict` a barra di-lo; sem ele, nada.
#[test]
fn o_modo_estrito_aparece_na_barra() {
    let buf = Buffer::new("--!strict\nlocal x = 1\n");
    assert_eq!(status(&buf, &Deco::default()).mode, None);
    let d = Deco {
        strict: true,
        ..Deco::default()
    };
    assert_eq!(status(&buf, &d).mode.as_deref(), Some("Strict types"));
}

/// Com a letra da interface fora do Normal, a posição da barra de estado pinta no tamanho escolhido
/// (aplicado UMA vez) e fica no meio dela: maiúsculas a menos de 1 px do meio da barra.
#[test]
fn o_texto_da_barra_fica_no_meio_com_outro_tamanho_de_letra() {
    use ph2d_tokens::{UiTextSize, UiTextStyle};
    let antes = ph2d_text::active_text_style();
    for size in [UiTextSize::Small, UiTextSize::Large] {
        ph2d_text::set_active_text_style(UiTextStyle { size, ..antes });
        let buf = Buffer::new("local x = 1\n");
        let mut ts = crate::ts();
        let mut v = ph2d_panel_code::CodeView::default();
        let area = crate::area_para(&mut v, &buf, &mut ts, 4);
        let scene = crate::pinta(&mut v, &buf, &mut ts, area, &Deco::default());
        let m = *v.metrics();
        let bar_y = area.y + area.h - m.status_h;
        let (px, x, base) = crate::linhas_de_base(&scene)
            .into_iter()
            .filter(|r| r.2 > bar_y)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .expect("a posição pintou");
        let nominal = px / size.scale();
        let pos = status(&buf, &Deco::default()).position;
        let meio = crate::meio_das_maiusculas(&mut ts, &pos, nominal, base);
        let alvo = bar_y + m.status_h * 0.5;
        let quer = ph2d_text::displayed_font_px(ph2d_tokens::TypeToken::Sm.px());
        ph2d_text::set_active_text_style(antes);
        assert!(
            (px - quer).abs() < 1e-3,
            "{size:?}: a posição pinta a {px} px; o tamanho da letra aplica-se UMA vez ({quer})"
        );
        assert!(
            (meio - alvo).abs() <= 1.0,
            "{size:?}: posição em x {x}, meio {meio} contra o da barra {alvo}"
        );
    }
}

/// A ajuda da caixinha do rename fica no meio da linha dela em Small e Large: as maiúsculas a
/// menos de 1 px do meio da caixa (sem o campo, a caixa é só essa linha).
#[test]
fn a_ajuda_do_rename_fica_no_meio_da_caixa() {
    use ph2d_tokens::{UiTextSize, UiTextStyle};
    let antes = ph2d_text::active_text_style();
    let mut falhas = Vec::new();
    for size in [UiTextSize::Small, UiTextSize::Large] {
        ph2d_text::set_active_text_style(UiTextStyle { size, ..antes });
        let buf = Buffer::new("local speed = 1\n");
        let mut ts = crate::ts();
        let mut v = ph2d_panel_code::CodeView::default();
        let area = crate::area_para(&mut v, &buf, &mut ts, 4);
        let d = Deco::default();
        let mut f = |r: std::ops::Range<usize>| spans_de(&buf, r);
        let mut m = crate::marcas(&d);
        let mut fr = frame(&buf, &d, &mut f, &mut m);
        let hint = "Enter to rename";
        fr.rename = Some(ph2d_panel_code::RenameView {
            word: 6..11,
            field: None,
            hint,
            error: None,
        });
        let mut scene = ph2d_vector::VectorScene::new();
        v.paint(
            fr,
            area,
            &mut scene,
            &mut ts,
            &ph2d_tokens::Theme::default(),
        );
        let caixa = v.rename_rect().expect("a caixinha pintou-se");
        let Some((px, _, base)) = crate::linhas_de_base(&scene)
            .into_iter()
            .find(|r| caixa.contains(r.1, r.2))
        else {
            falhas.push(format!("{size:?}: a ajuda não pintou"));
            continue;
        };
        let meio = crate::meio_das_maiusculas(&mut ts, hint, px / size.scale(), base);
        let alvo = caixa.y + caixa.h * 0.5;
        if (meio - alvo).abs() > 1.0 {
            falhas.push(format!("{size:?}: meio {meio} contra {alvo}"));
        }
    }
    ph2d_text::set_active_text_style(antes);
    assert!(falhas.is_empty(), "{falhas:#?}");
}
