//! O passo mínimo da tecla (dono 10/11: «estreitar o app não pode achatar as teclas indefinidamente… a partir do
//! limite o scroll precisa aparecer. O teclado numérico não pode descer.»). Plano `docs/InputMap/01_plano.md` §13.2.

use super::Mesa;
use super::lista::onze;
use ph2d_app_input_map::view_keyboard::geometry::drawn_keys;
use ph2d_app_input_map::view_keyboard::keys_fit::{
    BAR_VISIBLE_MIN, FN_ROW_FACTOR, KEY_MIN_H, PITCH_FLOOR, bar_visible_at, fits_at, half_row,
    measured_floor,
};
use ph2d_app_input_map::view_keyboard::state::{KbHit, Sub};
use ph2d_app_input_map::{InputMapEdit, View};
use ph2d_editor_core::doc_surface::Key as DocKey;
use ph2d_editor_core::zones::Rect;
use ph2d_input::{Binding, Key};

/// A área real a 1280 e larguras cada vez mais estreitas, até 300 px.
const LARGURAS: [f32; 7] = [690.0, 640.0, 560.0, 480.0, 420.0, 360.0, 300.0]; // LITERAL-PX-OK: larguras de teste
const KP1: Key = Key(0xF741);
const PGDN: Key = Key(0xF72D);
const BACKSPACE: Key = Key(0x08);

fn area(w: f32) -> Rect {
    Rect::new(297.0, 28.0, w, 604.0) // LITERAL-PX-OK: a área central na foto da janela 1280×720
}

fn pinta(view: View, a: Rect) -> Mesa {
    let mut m = Mesa::aberta(view);
    m.hero.input_map = onze();
    m.frame();
    if view == View::Keyboard {
        m.click(m.at(KbHit::Sub(Sub::Keyboard).id()));
    }
    m.paint_in(a);
    m
}

fn inside(r: Rect, c: Rect) -> bool {
    r.x >= c.x - 0.01 && r.x + r.w <= c.x + c.w + 0.01
}

#[test]
fn o_passo_minimo_e_o_medido_nos_dois_modos_de_texto() {
    let mut medidos = Vec::new();
    super::em_cada_modo_de_texto(|| {
        let mut ts = ph2d_text::TextSystem::without_system_fonts();
        let p = measured_floor(&mut ts);
        eprintln!(
            "passo mínimo medido {p}; barra à vista {:.2}",
            bar_visible_at(&mut ts, p)
        );
        assert!(fits_at(&mut ts, PITCH_FLOOR), "medido {p}");
        medidos.push(p);
    });
    let maior = medidos.iter().copied().fold(0.0, f32::max);
    assert_eq!(maior, PITCH_FLOOR, "o PITCH_FLOOR é o medido: {medidos:?}");
}

#[test]
fn estreitar_nunca_achata_as_teclas_e_o_numerico_nunca_desce() {
    super::em_cada_modo_de_texto(|| {
        for view in [View::Keyboard, View::TryIt] {
            for w in LARGURAS {
                let at = format!("{view:?} w={w}");
                let m = pinta(view, area(w));
                let doc = &m.tab().doc;
                let f = doc.kb.fit.expect("o teclado pintado");
                assert!(f.pitch >= PITCH_FLOOR, "{at}: passo {}", f.pitch);
                assert!(f.legible, "{at}: legenda que cabe");
                assert_eq!(f.scrolls, doc.kb.hbar.is_some(), "{at}: barra ⇔ rola");
                let keys = &doc.census.keys;
                // Nenhuma legenda de tecla cortada (a lista estreita tem a sua própria lei de reticências).
                for t in doc.census.texts.iter().filter(|t| t.2) {
                    let (cx, cy) = (t.0.x + t.0.w * 0.5, t.0.y + t.0.h * 0.5);
                    assert!(
                        !keys.iter().any(|l| l.rect.contains(cx, cy)),
                        "{at}: legenda cortada «{}»",
                        t.1
                    );
                }
                for l in keys {
                    // A fila de cima é de meia altura (onda 3, a foto do dono): o piso dela é o fator medido.
                    let piso = if half_row(l.key) {
                        KEY_MIN_H * FN_ROW_FACTOR
                    } else {
                        KEY_MIN_H
                    };
                    assert!(l.rect.h >= piso - 0.01, "{at}: {} achatada", l.key.label());
                    let side =
                        (l.plate.x - l.rect.x).min(l.rect.x + l.rect.w - l.plate.x - l.plate.w);
                    assert!(
                        side >= BAR_VISIBLE_MIN,
                        "{at}: {} tapa as barras ({side})",
                        l.key.label()
                    );
                }
                let x_of = |k: Key| keys.iter().find(|l| l.key == k).expect("pintada").rect;
                let bs = x_of(BACKSPACE);
                assert!(
                    x_of(KP1).x > bs.x + bs.w,
                    "{at}: o numérico fica à direita do principal"
                );
                assert!(
                    (x_of(PGDN).y - x_of(Key(0x1B)).y).abs() < 0.5,
                    "{at}: o PgDn na fila de cima"
                );
                if let Some(b) = doc.kb.hbar {
                    for l in keys {
                        // O alvo de clique é só a parte à vista (o recorte do campo).
                        if let Some(h) = doc.rect_of(KbHit::Key(l.key).id()) {
                            assert!(
                                inside(h, b.clip),
                                "{at}: alvo de {} fora da vista",
                                l.key.label()
                            );
                        }
                    }
                    assert!(
                        b.thumb.y >= keys.iter().map(|l| l.rect.y + l.rect.h).fold(0.0, f32::max)
                    );
                }
            }
        }
    });
}

#[test]
fn acima_do_piso_nao_ha_barra_horizontal() {
    for a in [
        Rect::new(308.0, 28.0, 988.0, 890.0), // LITERAL-PX-OK: a área central a 1600×1000
        area(690.0),
    ] {
        for view in [View::Keyboard, View::TryIt] {
            let m = pinta(view, a);
            assert!(m.tab().doc.kb.hbar.is_none(), "{view:?} {}: sem barra", a.w);
        }
    }
}

#[test]
fn abaixo_do_piso_rola_ate_ao_numerico_e_a_tecla_liga() {
    for view in [View::Keyboard, View::TryIt] {
        let a = area(360.0);
        let mut m = pinta(view, a);
        let b0 = m.tab().doc.kb.hbar.expect("a barra horizontal");
        let pg = m
            .tab()
            .doc
            .census
            .keys
            .iter()
            .find(|l| l.key == PGDN)
            .expect("PgDn")
            .rect;
        assert!(
            !inside(pg, b0.clip),
            "{view:?}: o PgDn começa fora da vista"
        );
        // Shift+roda até ao fim.
        m.tab_mut().doc.wheel_shift(f32::MIN, true);
        m.paint_in(a);
        let doc = &m.tab().doc;
        let b = doc.kb.hbar.expect("a barra");
        assert!(
            (doc.kb.hscroll - b.max()).abs() < 0.01,
            "{view:?}: rolou até ao fim"
        );
        for k in [PGDN, KP1, Key::NUMPAD_ENTER] {
            let r = doc
                .census
                .keys
                .iter()
                .find(|l| l.key == k)
                .expect("pintada")
                .rect;
            assert!(inside(r, b.clip), "{view:?}: {} inteira à vista", k.label());
        }
        if view == View::Keyboard {
            let open = doc.kb.open.expect("acção aberta");
            let r = doc
                .rect_of(KbHit::Key(KP1).id())
                .expect("alvo do 1 do numérico");
            let at = (r.x + r.w * 0.5, r.y + r.h * 0.5);
            m.tab_mut().doc.pending.clear();
            m.down(at);
            m.paint_in(a);
            m.up(at);
            let pend: Vec<_> = (m.tab().doc.pending.iter())
                .filter(|e| matches!(e, InputMapEdit::AddBinding(..)))
                .cloned()
                .collect();
            let ligou = m
                .hero
                .input_map
                .get(open)
                .is_some_and(|a| a.bindings.contains(&Binding::Key(KP1)));
            assert!(
                ligou || pend == vec![InputMapEdit::AddBinding(open, Binding::Key(KP1))],
                "clicar no 1 do numérico liga-o: {pend:?}"
            );
        }
        // A roda sem o Shift não mexe no teclado; o arrasto do polegar volta ao começo.
        let s = m.tab().doc.kb.hscroll;
        m.tab_mut().doc.wheel_shift(-24.0, false);
        m.paint_in(a);
        assert_eq!(
            m.tab().doc.kb.hscroll,
            s,
            "{view:?}: a roda vertical não rola o teclado"
        );
        let t = m.tab().doc.kb.hbar.expect("a barra").thumb;
        let grab = (t.x + t.w * 0.5, t.y + t.h * 0.5);
        m.down(grab);
        m.paint_in(a);
        m.hover((a.x - 2000.0, grab.1)); // LITERAL-PX-OK: arrasto bem para a esquerda
        m.paint_in(a);
        m.release();
        m.paint_in(a);
        assert_eq!(
            m.tab().doc.kb.hscroll,
            0.0,
            "{view:?}: o arrasto do polegar rola"
        );
    }
}

#[test]
fn o_tab_traz_a_tecla_focada_para_a_vista() {
    let a = area(300.0);
    let mut m = pinta(View::Keyboard, a);
    let mut vistas = Vec::new();
    for _ in 0..400 {
        m.key(DocKey::Tab, None);
        m.paint_in(a);
        let doc = &m.tab().doc;
        let Some(k) = drawn_keys()
            .into_iter()
            .find(|k| doc.is_focused(KbHit::Key(*k).id()))
        else {
            continue;
        };
        let b = doc.kb.hbar.expect("a barra");
        let r = doc
            .census
            .keys
            .iter()
            .find(|l| l.key == k)
            .expect("pintada")
            .rect;
        assert!(inside(r, b.clip), "{}: o foco tra-la à vista", k.label());
        vistas.push(k);
    }
    for k in [Key(0x1B), PGDN, KP1, Key::NUMPAD_ENTER] {
        assert!(vistas.contains(&k), "o Tab chega a {}", k.label());
    }
}

#[test]
fn a_dica_fica_inteira_dentro_da_aba_estreita() {
    for w in [360.0, 300.0] {
        let a = area(w);
        let mut m = pinta(View::Keyboard, a);
        for k in [Key(0x1B), BACKSPACE] {
            let r = m
                .tab()
                .doc
                .census
                .keys
                .iter()
                .find(|l| l.key == k)
                .expect("pintada")
                .rect;
            let at = (r.x.max(a.x + 4.0) + 2.0, r.y + r.h * 0.5); // LITERAL-PX-OK: dentro da parte à vista
            m.hover(at);
            m.paint_in(a);
            let tip = m.tab().doc.census.tip;
            if let Some(t) = tip {
                assert!(
                    t.x >= a.x && t.x + t.w <= a.x + a.w + 0.01,
                    "w={w}: a dica sai da aba {t:?}"
                );
                assert!(
                    t.y >= a.y && t.y + t.h <= a.y + a.h + 0.01,
                    "w={w}: a dica sai da aba {t:?}"
                );
            }
        }
    }
}
