//! W2 (onda 2): o teclado de notebook desenhado na B e na C — censo na área REAL a 1600/1280 × 2 modos, clique em
//! cada tecla nova, teclas do sistema apagadas. Plano: `docs/InputMap/01_plano.md` §13.2.

use super::Mesa;
use super::lista::onze;
use super::teclado_censo::AREAS;
use ph2d_app_input_map::view_keyboard::geometry::{Arrangement, drawn_keys};
use ph2d_app_input_map::view_keyboard::keys_fit::{MIN_LEGEND_PX, bars_top, fit_for_width};
use ph2d_app_input_map::view_keyboard::state::{KbHit, Sub};
use ph2d_app_input_map::{InputMapEdit, View};
use ph2d_editor_core::zones::Rect;
use ph2d_input::{Binding, Key};

fn overlap(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
}

/// As provas do desenho na última pintura: cada tecla da tabela pintada UMA vez, legenda inteira dentro da tecla e
/// acima das barras, zero teclas sobrepostas, alvo de clique em cada tecla. Devolve (passo, menor margem, altura).
fn provas(m: &Mesa, at: &str) -> (f32, f32, f32) {
    let doc = &m.tab().doc;
    let c = &doc.census;
    assert_eq!(c.elided(), Vec::<&str>::new(), "{at}: texto cortado");
    let keys = drawn_keys();
    let mut rects = Vec::new();
    let mut margin = f32::MAX;
    for k in &keys {
        let n = (doc.kb.bars.iter())
            .filter(|(h, _)| *h == KbHit::Key(*k))
            .count();
        assert_eq!(n, 1, "{at}: {} pintada {n} vezes", k.label());
        let r = doc.rect_of(KbHit::Key(*k).id()).expect("alvo da tecla");
        let who = (doc.kb.bars.iter())
            .find(|(h, _)| *h == KbHit::Key(*k))
            .map_or(0, |(_, w)| w.len());
        let bars_top = bars_top(r, who);
        if let Some(t) = (c.texts.iter()).find(|t| {
            let (cx, cy) = (t.0.x + t.0.w * 0.5, t.0.y + t.0.h * 0.5);
            r.contains(cx, cy)
        }) {
            assert!(t.0.x >= r.x && t.0.x + t.0.w <= r.x + r.w, "{at}: legenda de {} sai da tecla", k.label());
            assert!(t.0.y + t.0.h <= bars_top + 0.5, "{at}: as barras tapam {}", k.label());
            assert!(t.0.h >= MIN_LEGEND_PX, "{at}: a legenda de {} não tem altura", k.label());
            margin = margin.min(t.0.x - r.x).min(r.x + r.w - t.0.x - t.0.w);
        } else {
            let icon = (c.icons.iter()).find(|(ir, _)| r.contains(ir.x + ir.w * 0.5, ir.y + ir.h * 0.5));
            let ir = icon.unwrap_or_else(|| panic!("{at}: {} sem legenda em {r:?}; fit {:?}; textos perto: {:?}", k.label(), doc.kb.fit, c.texts.iter().filter(|t| (t.0.y - r.y).abs() < 40.0).map(|t| (t.1.clone(), t.0)).collect::<Vec<_>>())).0;
            assert!(ir.y + ir.h <= bars_top + 0.5, "{at}: as barras tapam o ícone de {}", k.label());
            margin = margin.min(ir.x - r.x).min(r.x + r.w - ir.x - ir.w);
        }
        rects.push((k.label(), r));
    }
    for (i, (a, ra)) in rects.iter().enumerate() {
        for (b, rb) in &rects[i + 1..] {
            assert!(!overlap(*ra, *rb), "{at}: {a} e {b} sobrepõem-se");
        }
    }
    let fit = doc.kb.fit.expect("o arranjo da pintura");
    let top = rects.iter().map(|r| r.1.y).fold(f32::MAX, f32::min);
    let low = rects.iter().map(|r| r.1.y + r.1.h).fold(0.0, f32::max);
    eprintln!("{at}: {:?} passo {:.1} margem {margin:.1} altura {:.0}", fit.arrangement, fit.pitch, low - top);
    (fit.pitch, margin, low - top)
}

#[test]
fn o_teclado_de_notebook_inteiro_na_b_e_na_c_nas_areas_reais() {
    super::em_cada_modo_de_texto(|| {
        for area in AREAS {
            let mut b = Mesa::aberta(View::Keyboard);
            b.hero.input_map = onze();
            b.frame();
            b.click(b.at(KbHit::Sub(Sub::Keyboard).id()));
            b.paint_in(area);
            provas(&b, &format!("B {}×{}", area.w, area.h));
            let mut c = Mesa::aberta(View::TryIt);
            c.hero.input_map = onze();
            c.frame();
            c.paint_in(area);
            provas(&c, &format!("C {}×{}", area.w, area.h));
        }
    });
}

#[test]
fn mede_as_teclas_que_nao_cabem() {
    use ph2d_app_input_map::view_keyboard::keys_fit::misfits;
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    for w in [600.0, 614.0, 640.0] {
        eprintln!("C w={w}: {:?}", misfits(&mut ts, w, Arrangement::Wide, false));
    }
}

#[test]
fn o_arranjo_e_funcao_pura_da_largura_e_do_texto() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let largo = fit_for_width(&mut ts, 900.0, true); // LITERAL-PX-OK: caixa de teste
    assert_eq!(largo.arrangement, Arrangement::Wide);
    assert!(largo.legible);
    let estreito = fit_for_width(&mut ts, 300.0, true); // LITERAL-PX-OK: caixa de teste
    assert_eq!(estreito.arrangement, Arrangement::Narrow);
    assert_eq!(fit_for_width(&mut ts, 300.0, true), estreito, "a mesma caixa, a mesma escolha");
}

#[test]
fn clicar_cada_tecla_liga_exactamente_essa_e_a_do_sistema_nada() {
    for area in AREAS {
        let mut m = Mesa::aberta(View::Keyboard);
        m.hero.input_map = onze();
        m.frame();
        m.click(m.at(KbHit::Sub(Sub::Keyboard).id()));
        let open = m.tab().doc.kb.open.expect("acção aberta");
        for k in drawn_keys() {
            m.paint_in(area);
            m.tab_mut().doc.pending.clear();
            let r = m.tab().doc.rect_of(KbHit::Key(k).id()).expect("alvo");
            // As teclas de duas filas ligam de qualquer ponto: clica-se no FUNDO.
            // O clique INTEIRO na pintura desta área: o quadro do Down repinta a `AREA` da mesa, e o Up tem de cair
            // no rect que a área pintou (senão é «arrastar para fora», que cancela — o Esc era só a primeira tecla).
            let at = (r.x + r.w * 0.5, r.y + r.h - 2.0); // LITERAL-PX-OK: dentro da aresta de baixo
            m.down(at);
            m.paint_in(area);
            m.up(at);
            let pend = m.tab().doc.pending.clone();
            let want: Vec<InputMapEdit> = if k.system_owned().is_some()
                || m.hero.input_map.get(open).is_some_and(|a| a.bindings.contains(&Binding::Key(k)))
            {
                Vec::new()
            } else {
                vec![InputMapEdit::AddBinding(open, Binding::Key(k))]
            };
            let pend: Vec<_> = pend.into_iter().filter(|e| matches!(e, InputMapEdit::AddBinding(..))).collect();
            assert_eq!(pend, want, "{}×{}: {}", area.w, area.h, k.label());
        }
    }
}

#[test]
fn a_tecla_do_sistema_diz_o_porque_no_hover() {
    let mut m = Mesa::aberta(View::Keyboard);
    m.click(m.at(KbHit::Sub(Sub::Keyboard).id()));
    let win = Key(0xF718);
    let r = m.tab().doc.rect_of(KbHit::Key(win).id()).expect("a Windows desenhada");
    m.hover((r.x + r.w * 0.5, r.y + r.h * 0.5));
    let all: String = (m.tab().doc.census.texts.iter()).map(|t| t.1.clone()).collect();
    assert!(all.contains(ph2d_i18n::tr("input_map_keyboard.system.windows")), "{all}");
}

#[test]
fn o_numerico_acende_no_try_it() {
    let mut m = Mesa::aberta(View::TryIt);
    let kp8 = Key::named_in_script("kp8").expect("kp8")[0];
    let mut map = onze();
    let up = map.id("move_up").expect("move_up");
    map.get_mut(up).expect("move_up").bindings.push(Binding::Key(kp8));
    m.hero.input_map = map;
    m.frame();
    assert!(m.physical(kp8), "o bench toma o Numpad 8");
    m.frame();
    let r = m.tab().doc.rect_of(KbHit::Key(kp8).id()).expect("Numpad 8 pintado");
    assert!(
        (m.tab().doc.census.filled.iter())
            .any(|(f, _)| (f.x - r.x).abs() < 0.01 && (f.y - r.y).abs() < 0.01 && (f.w - r.w).abs() < 0.01),
        "o Numpad 8 premido enche-se: {r:?} × {:?} held {:?}",
        m.tab().doc.census.filled,
        m.tab().doc.tryit.held_keys()
    );
}
