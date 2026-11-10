//! R7b (revisão lado a lado da B): as frases do cartão aberto partem-se dentro dele a 1280, as barras não tapam a
//! letra da tecla, a frase do hover entra no lugar da legenda sem mexer no desenho, e o comando diz «button or stick».

use super::Mesa;
use super::teclado_censo::AREAS;
use ph2d_app_input_map::View;
use ph2d_app_input_map::view_keyboard::state::{KbHit, Sub};
use ph2d_input::Key;

fn named(n: &str) -> Key {
    Key::named_in_script(n).expect("tecla com nome")[0]
}

fn abre(m: &mut Mesa, name: &str) {
    let id = m.hero.input_map.id(name).expect("a acção existe");
    m.click(m.at(KbHit::Row(id).id()));
}

#[test]
fn a_frase_do_cartao_aberto_e_o_aviso_partem_se_dentro_do_cartao_a_1280() {
    super::em_cada_modo_de_texto(|| {
        let area = AREAS[1];
        let mut m = Mesa::aberta(View::Keyboard);
        m.hero.input_map = super::lista::onze();
        m.frame();
        for name in ["jump", "reload"] {
            abre(&mut m, name);
            m.paint_in(area);
            let c = &m.tab().doc.census;
            assert_eq!(
                c.elided(),
                Vec::<&str>::new(),
                "{name} a 1280: texto cortado"
            );
            for (r, s, _) in &c.texts {
                assert!(
                    r.x + r.w <= area.x + area.w,
                    "{name}: «{s}» sai da aba ({r:?})"
                );
            }
        }
    });
}

#[test]
fn as_barras_das_accoes_nao_tapam_a_letra_da_tecla_a_1280() {
    let mut m = Mesa::aberta(View::Keyboard);
    m.paint_in(AREAS[1]);
    let s_key = m
        .tab()
        .doc
        .rect_of(KbHit::Key(named("s")).id())
        .expect("o S pintado");
    let c = &m.tab().doc.census;
    let (lr, _, _) = c
        .texts
        .iter()
        .find(|(r, s, _)| s == "S" && s_key.contains(r.x + r.w * 0.5, r.y + r.h * 0.5))
        .expect("a letra do S");
    let letter = 12.0; // LITERAL-PX-OK: o `VALUE_PX` do cartão, a letra da tecla
    assert!(lr.h >= letter, "o S tem {} px acima das barras", lr.h);
}

#[test]
fn a_frase_do_hover_substitui_a_legenda_sem_mexer_no_desenho() {
    let mut m = Mesa::aberta(View::Keyboard);
    m.paint();
    let up = KbHit::Key(named("up")).id();
    let before = m.tab().doc.rect_of(up).expect("a ↑");
    let caption = |m: &Mesa| {
        m.tab()
            .doc
            .census
            .texts
            .iter()
            .any(|(_, s, _)| s == "Keyboard · click a key to add it to the open action")
    };
    assert!(caption(&m), "sem hover, a legenda");
    m.hover(m.at(up));
    m.paint();
    let after = m.tab().doc.rect_of(up).expect("a ↑");
    assert_eq!(
        (before.y, before.h),
        (after.y, after.h),
        "o desenho não salta"
    );
    assert!(!caption(&m), "com hover, a frase no lugar da legenda");
    let t: Vec<&str> = m
        .tab()
        .doc
        .census
        .texts
        .iter()
        .map(|t| t.1.as_str())
        .collect();
    assert!(t.contains(&"Up Arrow:"), "{t:?}");
    assert!(
        !t.iter().any(|s| s.starts_with("Point at")),
        "sem frase de repouso: {t:?}"
    );
}

#[test]
fn o_comando_diz_botao_ou_manche_e_nenhum_alvo_se_sobrepoe() {
    for area in AREAS {
        let mut m = Mesa::aberta(View::Keyboard);
        m.click(m.at(KbHit::Sub(Sub::Gamepad).id()));
        m.paint_in(area);
        let doc = &m.tab().doc;
        let t: Vec<&str> = doc.census.texts.iter().map(|t| t.1.as_str()).collect();
        assert!(t.iter().any(|s| s.contains("button or a stick")), "{t:?}");
        let rects: Vec<_> = doc
            .kb
            .bars
            .iter()
            .map(|(h, _)| *h)
            .filter(|h| matches!(h, KbHit::Pad(_)))
            .filter_map(|h| doc.rect_of(h.id()).map(|r| (h, r)))
            .collect();
        assert!(rects.len() >= 22, "os alvos do comando: {}", rects.len());
        for (i, (ha, a)) in rects.iter().enumerate() {
            for (hb, b) in &rects[i + 1..] {
                let apart =
                    a.x + a.w <= b.x || b.x + b.w <= a.x || a.y + a.h <= b.y || b.y + b.h <= a.y;
                assert!(apart, "{area:?}: {ha:?} {a:?} cobre {hb:?} {b:?}");
            }
        }
    }
}
