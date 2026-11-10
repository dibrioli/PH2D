//! R3-B (plano `docs/InputMap/01_plano.md` §13.2–§13.3): a vista B empilhada a 1280 e os controlos dela no registo
//! (hover instantâneo só no pairado, premido com o botão em baixo, arrastar para fora cancela, dispara no Up dentro,
//! Tab com o anel, Enter activa, nó a11y com papel e nome, cursor) e o Remove desligado com o porquê.

use super::Mesa;
use super::lista::onze;
use super::teclado_censo::AREAS;
use ph2d_app_input_map::view_keyboard::geometry::{Arrangement, drawn_keys};
use ph2d_app_input_map::view_keyboard::state::{KbHit, Sub};
use ph2d_app_input_map::{InputMapEdit, View};
use ph2d_editor_core::doc_surface::{Key as DocKey, PointerLook};
use ph2d_editor_core::zones::Rect;

fn inside(r: Rect, a: Rect) -> bool {
    r.x >= a.x - 0.01 && r.y >= a.y - 0.01 && r.x + r.w <= a.x + a.w + 0.01 && r.y + r.h <= a.y + a.h + 0.01
}

fn mesa_b() -> Mesa {
    let mut m = Mesa::aberta(View::Keyboard);
    m.hero.input_map = onze();
    m.frame();
    m.click(m.at(KbHit::Sub(Sub::Keyboard).id()));
    m
}

/// A 1280 a coluna não segura o Wide: empilha (o teclado na largura inteira, Wide); a 1600 fica ao lado. Cada
/// tecla e cada linha da lista (rolando) cabe na área; o cartão aberto inteiro dentro dela.
#[test]
fn a_1280_a_b_empilha_e_tudo_se_alcanca() {
    super::em_cada_modo_de_texto(|| {
        for (i, area) in AREAS.into_iter().enumerate() {
            for sub in [Sub::Keyboard, Sub::Gamepad] {
                let mut m = mesa_b();
                m.click(m.at(KbHit::Sub(sub).id()));
                m.paint_in(area);
                let doc = &m.tab().doc;
                let body = doc.body();
                let fit = doc.kb.fit;
                eprintln!(
                    "{}×{} {sub:?}: stacked {} fit {fit:?} body {body:?}",
                    area.w, area.h, doc.kb.stacked
                );
                assert_eq!(doc.kb.stacked, i == 1, "{}: empilha só a 1280", area.w);
                if sub == Sub::Keyboard {
                    let f = fit.expect("o teclado pintado");
                    assert_eq!(f.arrangement, Arrangement::Wide, "{}: o arranjo da foto", area.w);
                    let rs: Vec<Rect> = drawn_keys()
                        .into_iter()
                        .map(|k| doc.rect_of(KbHit::Key(k).id()).expect("tecla pintada"))
                        .collect();
                    let top = rs.iter().map(|r| r.y).fold(f32::INFINITY, f32::min);
                    let bot = rs.iter().map(|r| r.y + r.h).fold(0.0, f32::max);
                    eprintln!("  pitch {:.1} · altura do teclado {:.1}", f.pitch, bot - top);
                    for r in &rs {
                        assert!(inside(*r, body), "tecla {r:?} fora de {body:?}");
                    }
                }
                // O cartão aberto: todo texto dele dentro do cartão da vista.
                for (r, s, _) in &doc.census.texts {
                    let cut = doc.census.clip.filter(|c| r.x >= c.x && r.x < c.x + c.w);
                    if cut.is_none() {
                        assert!(r.y + r.h <= body.y + body.h + 0.01, "«{s}» {r:?} abaixo de {body:?}");
                    }
                }
                // Cada linha alcança-se: rola até ela caber no recorte.
                let ids: Vec<_> = m.hero.input_map.actions().iter().map(|a| a.id).collect();
                for a in ids {
                    let mut seen = false;
                    m.tab_mut().doc.wheel(f32::MAX);
                    for _ in 0..64 {
                        m.paint_in(area);
                        let doc = &m.tab().doc;
                        let view = doc.census.clip.unwrap_or(body);
                        if doc.rect_of(KbHit::Row(a).id()).is_some_and(|r| inside(r, view)) {
                            seen = true;
                            break;
                        }
                        if !m.tab_mut().doc.wheel(-24.0) // LITERAL-PX-OK: um entalhe da roda
 {
                            break;
                        }
                    }
                    assert!(seen, "{}×{} {sub:?}: a linha {a:?} nunca se alcança", area.w, area.h);
                }
            }
        }
    });
}

/// Os alvos de cada espécie de controlo da B (com a acção aberta analógica para a barra).
fn kinds(m: &Mesa) -> Vec<(&'static str, KbHit, ph2d_a11y::Role)> {
    use ph2d_a11y::Role as R;
    let doc = &m.tab().doc;
    let open = doc.kb.open.expect("aberta");
    let key = drawn_keys().into_iter().find(|k| k.system_owned().is_none()).expect("tecla");
    vec![
        ("segment", KbHit::Sub(Sub::Gamepad), R::Tab),
        ("row", KbHit::Row(open), R::ListItem),
        ("add", KbHit::Add, R::Button),
        ("remove", KbHit::Remove, R::Button),
        ("key", KbHit::Key(key), R::Button),
        ("listen", KbHit::Listen, R::Button),
    ]
}

#[test]
fn cada_controlo_da_b_paira_preme_cancela_dispara_e_tem_no() {
    let all = kinds(&mesa_b());
    for (what, h, role) in all.clone() {
        let mut m = mesa_b();
        let id = h.id();
        let at = m.at(id);
        m.hover(at);
        let doc = &m.tab().doc;
        assert!(doc.is_hovered(id), "{what}: pairado");
        assert!((doc.hover_live(id) - 1.0).abs() < f32::EPSILON, "{what}: hover instantâneo");
        for (_, o, _) in all.iter().filter(|o| o.1 != h) {
            assert!(doc.hover_live(o.id()) < f32::EPSILON, "{what}: {o:?} também acende");
        }
        assert_eq!(doc.look_at(at.0, at.1), Some(PointerLook::Hand), "{what}: cursor");
        let node = doc.a11y().into_iter().find(|(n, _)| *n == id).expect("nó a11y").1;
        assert_eq!(node.role(), role, "{what}: papel");
        assert!(node.label().is_some_and(|l| !l.is_empty()), "{what}: nome");
        // Premido com o botão em baixo; fora do alvo apaga e o Up lá fora não dispara.
        m.tab_mut().doc.pending.clear();
        m.down(at);
        assert!(m.tab().doc.is_pressed(id), "{what}: premido");
        let far = (at.0, m.tab().doc.body().y + m.tab().doc.body().h - 1.0); // LITERAL-PX-OK: dentro da aresta
        m.hover(far);
        assert!(!m.tab().doc.is_pressed(id), "{what}: arrastar para fora apaga o premido");
        let before = (m.tab().doc.kb.sub, m.tab().doc.kb.adding);
        m.up(far);
        assert!(m.tab().doc.pending.is_empty(), "{what}: o Up fora não dispara");
        assert_eq!((m.tab().doc.kb.sub, m.tab().doc.kb.adding), before, "{what}: nada mudou");
    }
}

#[test]
fn tab_chega_a_cada_controlo_com_o_anel_e_enter_activa() {
    let mut m = mesa_b();
    let want = KbHit::Sub(Sub::Gamepad).id();
    let mut reached = false;
    for _ in 0..400 {
        m.key(DocKey::Tab, None);
        if m.tab().doc.is_focused(want) {
            reached = true;
            break;
        }
    }
    assert!(reached, "o Tab chega ao segmento Gamepad");
    m.key(DocKey::Enter, None);
    assert_eq!(m.tab().doc.kb.sub, Sub::Gamepad, "Enter activa o foco");
}

#[test]
fn o_remove_sem_accao_aberta_fica_desligado_e_diz_porque() {
    let mut m = Mesa::aberta(View::Keyboard);
    m.hero.input_map = ph2d_input::InputMap::default();
    m.frame();
    let id = KbHit::Remove.id();
    let at = m.at(id);
    m.hover(at);
    let doc = &m.tab().doc;
    assert_eq!(doc.look_at(at.0, at.1), Some(PointerLook::Arrow), "desligado: seta");
    assert_eq!(
        doc.disabled_why_here().map(|w| w.1),
        Some("input_map_keyboard.remove_off_none")
    );
    m.tab_mut().doc.pending.clear();
    m.click(at);
    assert!(
        !m.tab().doc.pending.iter().any(|e| matches!(e, InputMapEdit::Remove(_))),
        "o desligado não dispara"
    );
}

#[test]
fn clicar_no_remove_com_accao_aberta_apaga_a_aberta() {
    let mut m = mesa_b();
    let open = m.tab().doc.kb.open.expect("aberta");
    let n = m.hero.input_map.actions().len();
    m.click(m.at(KbHit::Remove.id()));
    // O quadro da família aplica o pedido: a aberta sai do mapa.
    assert!(m.hero.input_map.get(open).is_none(), "a aberta ficou");
    assert_eq!(m.hero.input_map.actions().len(), n - 1);
}
