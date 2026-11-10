//! F3 (onda 2): a região que rola da B empilhada (desvanecer só no cartão cortado, a barra na calha da LISTA, nada
//! de barra sem transbordo), o Add da A (aresta neutra desligado, acento quando age), a caixa «No key» da linha sem
//! teclas e a nota das receitas da C sem palavra viúva — nas duas áreas reais, nos dois modos de texto.

use super::Mesa;
use super::lista::onze;
use super::teclado_censo::AREAS;
use ph2d_app_input_map::View;
use ph2d_app_input_map::ids::{self, ListHit};
use ph2d_app_input_map::view_keyboard::state::{KbHit, Sub};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::ColorToken;

fn within_x(r: Rect, col: Rect) -> bool {
    r.x >= col.x - 0.01 && r.x + r.w <= col.x + col.w + 0.01
}

/// 1: o desvanecer só cobre a coluna da lista (o cartão da aberta, curto, não tem faixa por baixo), o polegar mora
/// na calha da lista, e sem transbordo não há polegar nem desvanecer.
#[test]
fn f3_a_regiao_da_b_desvanece_so_o_cartao_cortado() {
    super::em_cada_modo_de_texto(|| {
        for area in AREAS {
            for name in ["reload", "jump", "laser"] {
                let mut m = Mesa::aberta(View::Keyboard);
                m.hero.input_map = onze();
                m.frame();
                m.click(m.at(KbHit::Sub(Sub::Keyboard).id()));
                let id = m.hero.input_map.id(name).expect("a acção");
                m.tab_mut().doc.smoke_open(Some(id));
                m.paint_in(area);
                let doc = &m.tab().doc;
                let row = doc.rect_of(KbHit::Row(id).id()).expect("a linha");
                let Some(view) = doc.census.clip else {
                    assert!(
                        doc.census.scrollbars.is_empty(),
                        "{}: polegar sem transbordo",
                        area.w
                    );
                    assert!(
                        doc.census.fades.is_empty(),
                        "{}: desvanecer sem transbordo",
                        area.w
                    );
                    continue;
                };
                assert!(
                    !doc.census.scrollbars.is_empty(),
                    "{}: cortada, mostra o polegar",
                    area.w
                );
                // A coluna da lista: a da linha aberta, com o recuo da lista dos dois lados.
                let list = Rect::new(
                    view.x,
                    view.y,
                    row.x + row.w + (row.x - view.x) - view.x,
                    view.h,
                );
                for f in &doc.census.fades {
                    assert!(
                        within_x(*f, list),
                        "{} {name}: desvanecer {f:?} fora da coluna da lista {list:?}",
                        area.w
                    );
                }
                for (thumb, _, gutter) in &doc.census.scrollbars {
                    assert!(
                        within_x(*gutter, list),
                        "{} {name}: a calha na lista",
                        area.w
                    );
                    assert!(
                        within_x(*thumb, list),
                        "{} {name}: o polegar na lista",
                        area.w
                    );
                }
            }
        }
    });
}

/// 2: o Add desligado tem aresta NEUTRA; com um nome novo que pode entrar, a aresta acende no acento.
#[test]
fn f3_o_add_desligado_e_neutro_e_acende_quando_age() {
    super::em_cada_modo_de_texto(|| {
        for area in AREAS {
            let mut m = Mesa::aberta(View::List);
            m.paint_in(area);
            let edge = |m: &Mesa| {
                let doc = &m.tab().doc;
                let add = doc.rect_of(ids::ADD).expect("o Add");
                (doc.census.filled.iter())
                    .filter(|(r, _)| *r == add)
                    .map(|(_, t)| *t)
                    .collect::<Vec<_>>()
            };
            assert_eq!(
                edge(&m),
                vec![ColorToken::BorderStrong],
                "{}: desligado",
                area.w
            );
            m.click(m.at(ids::NEW_NAME));
            m.type_text("roll");
            m.paint_in(area);
            assert_eq!(
                edge(&m),
                vec![ColorToken::BorderEmph],
                "{}: pode agir",
                area.w
            );
        }
    });
}

/// 2: a linha sem teclas mostra a caixa «No key» (o alvo da escuta, o texto dentro dele, em `Text2`) e clicá-la
/// escuta.
#[test]
fn f3_a_linha_sem_tecla_diz_no_key_e_escuta_ao_clique() {
    super::em_cada_modo_de_texto(|| {
        for area in AREAS {
            let mut m = Mesa::aberta(View::List);
            m.hero.input_map = onze();
            m.frame();
            m.paint_in(area);
            let reload = m.hero.input_map.id("reload").expect("onze");
            let doc = &m.tab().doc;
            // A 1280 a linha pode estar abaixo do recorte (a lista rola): mede-se onde está pintada.
            let Some(target) = doc.rect_of(ListHit::Listen(reload).id()) else {
                assert!(area.w < 900.0, "a 1600 a linha à vista"); // LITERAL-PX-OK: separa as duas áreas
                continue;
            };
            let msg = tr("input_map_list.no_key");
            let text = (doc.census.texts.iter())
                .find(|t| t.1 == msg)
                .map(|t| t.0)
                .expect("«No key» pintado");
            assert!(
                text.x >= target.x && text.x + text.w <= target.x + target.w + 0.01,
                "{}: o texto {text:?} dentro da caixa {target:?}",
                area.w
            );
            // O clique pela mesa repinta na janela do teste a cada quadro: o alvo mede-se nela.
            m.paint();
            let at = m.at(ListHit::Listen(reload).id());
            m.click(at);
            assert_eq!(
                m.tab().doc.snap.listening,
                Some(reload),
                "{}: escuta",
                area.w
            );
        }
    });
}

/// 6: a nota das receitas quebra equilibrada — a última linha leva mais de uma palavra.
#[test]
fn f3_a_nota_das_receitas_nao_tem_palavra_viuva() {
    super::em_cada_modo_de_texto(|| {
        for area in AREAS {
            let mut m = Mesa::aberta(View::TryIt);
            m.paint_in(area);
            let cap = tr("input_map_try.recipe_caption");
            let lines: Vec<String> = (m.tab().doc.census.texts.iter())
                .filter(|t| t.1.contains(' ') && cap.contains(t.1.as_str()))
                .map(|t| t.1.clone())
                .collect();
            assert_eq!(
                lines.join(" "),
                cap,
                "{}: a nota inteira, por ordem",
                area.w
            );
            if lines.len() > 1 {
                for l in &lines {
                    assert!(
                        l.contains(' '),
                        "{}: linha de uma palavra «{l}» em {lines:?}",
                        area.w
                    );
                }
            }
        }
    });
}
