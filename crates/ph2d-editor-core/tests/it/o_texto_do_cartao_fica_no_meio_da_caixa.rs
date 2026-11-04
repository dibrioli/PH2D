//! ⭐ **O texto de cartão fica no meio da caixa** (`docs/CartoesUI/15_CHROME_CENSO.md`, medido nas fotos:
//! `+2,5` px em qualquer altura com a conta `(h − px) / 2`). A porta é `paint::folga_do_texto`.

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::paint::folga_do_texto;
use ph2d_text::TextSystem;
use ph2d_tokens::{UiFont, UiTextStyle};

/// Com a linha de base do layout e a letra alta das fontes (`0,67`–`0,73 em`), o meio das maiúsculas cai
/// a menos de 1 px do meio da caixa, em cada fonte, altura de caixa e tamanho do cartão.
#[test]
fn o_meio_das_maiusculas_cai_no_meio_da_caixa() {
    let mut ts = TextSystem::without_system_fonts();
    for font in UiFont::ALL {
        ph2d_text::set_active_text_style(UiTextStyle {
            font,
            ..UiTextStyle::default()
        });
        for px in [
            m::VALUE_PX,
            m::LABEL_PX,
            m::SEG_TEXT_PX,
            m::CARD_TITLE_PX,
            m::LIST_TITLE_PX,
        ] {
            let layout = ts.layout("Hx", px, f32::INFINITY);
            let base = layout.lines().next().expect("uma linha").metrics().baseline;
            for h in [
                22.0_f32,
                23.0,
                24.0,
                27.0,
                m::FIELD_H,
                m::CARD_HEAD_H,
                m::LIST_HEAD_H,
            ] {
                let topo = folga_do_texto(h, px).round();
                let meio = topo + base - 0.35 * px;
                assert!(
                    (meio - h * 0.5).abs() < 1.0,
                    "{font:?} a {px} em {h}: meio {meio}"
                );
            }
        }
    }
    ph2d_text::set_active_text_style(UiTextStyle::default());
}

/// Nenhum pintor volta à conta de antes: ela não dá erro, só desce o texto.
#[test]
fn nenhum_pintor_centra_pela_conta_de_antes() {
    let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut achados = Vec::new();
    let mut pilha = vec![raiz];
    while let Some(d) = pilha.pop() {
        for e in std::fs::read_dir(&d).expect("ler pasta").flatten() {
            let p = e.path();
            let nome = e.file_name();
            if p.is_dir() {
                if nome != "target" {
                    pilha.push(p);
                }
            } else if p.extension().is_some_and(|x| x == "rs") && !p.ends_with(file!()) {
                let s = std::fs::read_to_string(&p).unwrap_or_default();
                for k in [
                    "VALUE_PX",
                    "LABEL_PX",
                    "SEG_TEXT_PX",
                    "CARD_TITLE_PX",
                    "LIST_TITLE_PX",
                ] {
                    if s.contains(&format!(" - m::{k}) * 0.5")) {
                        achados.push(format!("{}: {k}", p.display()));
                    }
                }
            }
        }
    }
    assert!(
        achados.is_empty(),
        "use paint::folga_do_texto: {achados:#?}"
    );
}
