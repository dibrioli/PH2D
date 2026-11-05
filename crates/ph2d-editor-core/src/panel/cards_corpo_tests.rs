use super::*;

fn letras(eixos: Eixos, n: usize) -> Vec<String> {
    (0..n)
        .map(|i| match glifo("Half Size", eixos, n, i) {
            Glyph::Text(t) => t.to_owned(),
            g => panic!("glifo inesperado {g:?}"),
        })
        .collect()
}

/// ⭐ 05 §8: em nenhuma largura o par de caixas corta um nome: emparelha quando os dois cabem com a
/// coluna de animação, e senão empilha. *Mutação: `caixa_do_cartao` sem o `KEY_MARK_BOX`.*
#[test]
fn a_pair_of_checks_never_cuts_a_name_at_any_width() {
    use crate::property_row::{Seccao, cartao, paint_check_rows};
    use crate::text_elide::elisao;
    use ph2d_tokens::Theme;
    let (nomes, ids) = (["Active", "Look Through"], [NodeId(80), NodeId(81)]);
    let tema = Theme::default();
    let ink = CardInk::of_list(tema);
    let mut ts = TextSystem::without_system_fonts();
    for w in 150..=460 {
        let mut scene = VectorScene::new();
        let mut hit = HitIndex::default();
        let store = WidgetStore::default();
        let sec = Seccao::medida(&mut ts, 1, &nomes);
        let caixas = [(ids[0], nomes[0], true), (ids[1], nomes[1], false)];
        let (_, medidos) = elisao::medindo(|| {
            cartao::com(LINHAS, || {
                crate::widget::card_ink::com(ink, || {
                    paint_check_rows(
                        &mut scene, &mut ts, tema, &mut hit, &store, 0.0, w as f32, 0.0, &caixas,
                        sec,
                    )
                })
            })
        });
        for nome in nomes {
            assert!(
                medidos.iter().any(|m| m.texto == nome && m.coube()),
                "{nome:?} saiu cortado a {w} px: {medidos:?}"
            );
        }
    }
}

/// ⭐ T29: um TAMANHO mostra W · H (como o Figma), um retângulo X · Y · W · H; um eixo X · Y.
#[test]
fn a_size_pair_shows_w_and_h_like_figma() {
    assert_eq!(letras(Eixos::Tamanho, 2), ["W", "H"]);
    assert_eq!(letras(Eixos::Retangulo, 4), ["X", "Y", "W", "H"]);
    assert_eq!(letras(Eixos::Xyzw, 2), ["X", "Y"]);
    assert_eq!(
        letras(Eixos::Tamanho, 1),
        ["S"],
        "um campo só leva a inicial da última palavra"
    );
}

/// ⭐ T56: num campo só a letra é a da palavra que nomeia a grandeza ou o eixo, nunca a 1.ª.
#[test]
fn a_single_field_takes_the_letter_of_its_last_word() {
    let um = |r, e| match glifo(r, e, 1, 0) {
        Glyph::Text(t) => t.to_owned(),
        g => panic!("glifo inesperado {g:?}"),
    };
    assert_eq!(um("Half Width", Eixos::Xyzw), "W");
    assert_eq!(um("Offset Y", Eixos::Xyzw), "Y");
    assert_eq!(um("Radius (m)", Eixos::Xyzw), "R");
    assert_eq!(um("Half Width", Eixos::Letra("R")), "R");
    assert_eq!(
        um("Half height", Eixos::Xyzw),
        "H",
        "a letra é sempre maiúscula"
    );
    assert_eq!(um("jump_power", Eixos::Xyzw), "P", "o nome de um script");
    assert_eq!(um("speed", Eixos::Xyzw), "S");
}
