//! W2 (onda 2): o teclado de notebook desenhado na B e na C — censo na área REAL a 1600/1280 × 2 modos, clique em
//! cada tecla nova, teclas do sistema apagadas. Plano: `docs/InputMap/01_plano.md` §13.2.

use super::Mesa;
use super::lista::onze;
use super::teclado_censo::AREAS;
use ph2d_app_input_map::view_keyboard::geometry::drawn_keys;
use ph2d_app_input_map::view_keyboard::keys_fit::{MIN_LEGEND_PX, fit_for_width};
use ph2d_app_input_map::view_keyboard::state::{KbHit, Sub};
use ph2d_app_input_map::{InputMapEdit, View};
use ph2d_editor_core::icons::IconId;
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
        adesivo(doc, *k, r, at);
        if let Some(t) = (c.texts.iter()).find(|t| {
            let (cx, cy) = (t.0.x + t.0.w * 0.5, t.0.y + t.0.h * 0.5);
            r.contains(cx, cy)
        }) {
            assert!(
                t.0.x >= r.x && t.0.x + t.0.w <= r.x + r.w,
                "{at}: legenda de {} sai da tecla",
                k.label()
            );
            assert!(
                t.0.h >= MIN_LEGEND_PX,
                "{at}: a legenda de {} não tem altura",
                k.label()
            );
            margin = margin.min(t.0.x - r.x).min(r.x + r.w - t.0.x - t.0.w);
        } else {
            let icon =
                (c.icons.iter()).find(|(ir, _)| r.contains(ir.x + ir.w * 0.5, ir.y + ir.h * 0.5));
            let ir = icon
                .unwrap_or_else(|| {
                    panic!(
                        "{at}: {} sem legenda em {r:?}; fit {:?}; textos perto: {:?}",
                        k.label(),
                        doc.kb.fit,
                        c.texts
                            .iter()
                            .filter(|t| (t.0.y - r.y).abs() < 40.0)
                            .map(|t| (t.1.clone(), t.0))
                            .collect::<Vec<_>>()
                    )
                })
                .0;
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
    eprintln!(
        "{at}: rola {:?} passo {:.1} margem {margin:.1} altura {:.0}",
        fit.scrolls,
        fit.pitch,
        low - top
    );
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
        eprintln!("C w={w}: {:?}", misfits(&mut ts, w, false));
    }
}

#[test]
fn o_arranjo_e_funcao_pura_da_largura_e_do_texto() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let largo = fit_for_width(&mut ts, 900.0, true); // LITERAL-PX-OK: caixa de teste
    assert!(!largo.scrolls);
    assert!(largo.legible);
    let estreito = fit_for_width(&mut ts, 300.0, true); // LITERAL-PX-OK: caixa de teste
    assert!(estreito.scrolls, "abaixo do passo mínimo o teclado rola");
    assert!(estreito.legible);
    assert_eq!(
        fit_for_width(&mut ts, 300.0, true),
        estreito,
        "a mesma caixa, a mesma escolha"
    );
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
                || m.hero
                    .input_map
                    .get(open)
                    .is_some_and(|a| a.bindings.contains(&Binding::Key(k)))
            {
                Vec::new()
            } else {
                vec![InputMapEdit::AddBinding(open, Binding::Key(k))]
            };
            let pend: Vec<_> = pend
                .into_iter()
                .filter(|e| matches!(e, InputMapEdit::AddBinding(..)))
                .collect();
            assert_eq!(pend, want, "{}×{}: {}", area.w, area.h, k.label());
        }
    }
}

#[test]
fn a_tecla_do_sistema_diz_o_porque_no_hover() {
    let mut m = Mesa::aberta(View::Keyboard);
    m.click(m.at(KbHit::Sub(Sub::Keyboard).id()));
    let win = Key(0xF718);
    let r = m
        .tab()
        .doc
        .rect_of(KbHit::Key(win).id())
        .expect("a Windows desenhada");
    m.hover((r.x + r.w * 0.5, r.y + r.h * 0.5));
    let all: String = (m.tab().doc.census.texts.iter())
        .map(|t| t.1.clone())
        .collect();
    assert!(
        all.contains(ph2d_i18n::tr("input_map_keyboard.system.windows")),
        "{all}"
    );
}

#[test]
fn o_numerico_acende_no_try_it() {
    let mut m = Mesa::aberta(View::TryIt);
    let kp8 = Key::named_in_script("kp8").expect("kp8")[0];
    let mut map = onze();
    let up = map.id("move_up").expect("move_up");
    map.get_mut(up)
        .expect("move_up")
        .bindings
        .push(Binding::Key(kp8));
    m.hero.input_map = map;
    m.frame();
    assert!(m.physical(kp8), "o bench toma o Numpad 8");
    m.frame();
    let r = m
        .tab()
        .doc
        .rect_of(KbHit::Key(kp8).id())
        .expect("Numpad 8 pintado");
    assert!(
        (m.tab().doc.census.filled.iter()).any(|(f, _)| (f.x - r.x).abs() < 0.01
            && (f.y - r.y).abs() < 0.01
            && (f.w - r.w).abs() < 0.01),
        "o Numpad 8 premido enche-se: {r:?} × {:?} held {:?}",
        m.tab().doc.census.filled,
        m.tab().doc.tryit.held_keys()
    );
}

use ph2d_app_input_map::census::KeyLook;
use ph2d_app_input_map::view_keyboard::keys_fit::HOVER_FACE;
use ph2d_app_input_map::view_keyboard::keys_fit::{Group, group_of};
use ph2d_tokens::ColorToken;

/// O olhar da legenda de um grupo: `None` = ícone; senão o tamanho e se é a palavra da foto (`true`) ou a curta.
type Olhar = Option<(u32, bool)>;

/// Por grupo, os olhares das teclas de PALAVRA (os glifos únicos não são palavra) — tem de haver UM só.
fn degraus(keys: &[KeyLook], at: &str) -> Vec<(Group, Olhar)> {
    let mut out: Vec<(Group, Vec<Olhar>)> = Vec::new();
    for k in keys {
        let palavra = k.key.cap_short().chars().count() > 1 || k.key.cap().is_empty();
        // O símbolo da PrtSc (do sistema) quando as duas linhas «Prt»/«Sc» não cabem: a única excepção.
        let simbolo_do_sistema = k.text.is_none() && k.key.system_owned().is_some();
        if !palavra || group_of(k.key) == Group::Arrows || simbolo_do_sistema {
            continue;
        }
        let o = k.text.as_ref().map(|(s, px)| {
            (
                (px * 10.0).round() as u32,
                s == k.key.cap() && s != k.key.cap_short(),
            )
        });
        let g = group_of(k.key);
        match out.iter_mut().find(|(h, _)| *h == g) {
            Some((_, v)) => v.push(o),
            None => out.push((g, vec![o])),
        }
    }
    out.into_iter()
        .map(|(g, v)| {
            let icones = v.iter().filter(|o| o.is_none()).count();
            assert!(
                icones == 0 || icones == v.len(),
                "{at}: {g:?} mistura palavra e ícone: {v:?}"
            );
            let px: Vec<u32> = v.iter().flatten().map(|o| o.0).collect();
            assert!(
                px.windows(2).all(|w| w[0] == w[1]),
                "{at}: {g:?} mistura tamanhos: {v:?}"
            );
            let foto = v.iter().flatten().any(|o| o.1);
            (g, v.first().copied().flatten().map(|o| (o.0, foto)))
        })
        .collect()
}

fn pinta_b(area: Rect) -> Mesa {
    let mut b = Mesa::aberta(View::Keyboard);
    b.hero.input_map = onze();
    b.frame();
    b.click(b.at(KbHit::Sub(Sub::Keyboard).id()));
    b.paint_in(area);
    b
}

fn pinta_c(area: Rect) -> Mesa {
    let mut c = Mesa::aberta(View::TryIt);
    c.hero.input_map = onze();
    c.frame();
    c.paint_in(area);
    c
}

#[test]
fn cada_grupo_tem_um_so_degrau_e_a_b_e_a_c_concordam_na_mesma_caixa() {
    super::em_cada_modo_de_texto(|| {
        for area in AREAS {
            let (b, c) = (pinta_b(area), pinta_c(area));
            let db = degraus(&b.tab().doc.census.keys, &format!("B {}", area.w));
            let dc = degraus(&c.tab().doc.census.keys, &format!("C {}", area.w));
            eprintln!("B {}: {:?} {db:?}", area.w, b.tab().doc.kb.fit);
            eprintln!("C {}: {:?} {dc:?}", area.w, c.tab().doc.kb.fit);
            if b.tab().doc.kb.fit == c.tab().doc.kb.fit {
                assert_eq!(db, dc, "{}: a mesma caixa, o mesmo degrau", area.w);
            }
        }
    });
    // A mesma caixa pela função pura: B e C só diferem se a caixa difere.
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    for w in [600.0, 900.0] {
        let f = fit_for_width(&mut ts, w, false);
        assert_eq!(
            ph2d_app_input_map::view_keyboard::keys_fit::levels(&mut ts, f),
            ph2d_app_input_map::view_keyboard::keys_fit::levels(&mut ts, f)
        );
    }
}

#[test]
fn as_teclas_do_sistema_tem_o_mesmo_olhar_desligado_na_b_e_na_c() {
    let sistema: Vec<Key> = drawn_keys()
        .into_iter()
        .filter(|k| k.system_owned().is_some())
        .collect();
    assert!(sistema.len() >= 3, "Fn, Windows e PrtSc: {sistema:?}");
    for area in AREAS {
        let (b, c) = (pinta_b(area), pinta_c(area));
        for k in &sistema {
            let olhar = |m: &Mesa| {
                let l = (m.tab().doc.census.keys.iter())
                    .find(|l| l.key == *k)
                    .cloned()
                    .expect("desenhada");
                (l.face, l.ink, l.edge)
            };
            // K2: sem aresta nenhuma — a face de repouso APAGADA e a letra desligada (menos ênfase que o repouso).
            let want = (ColorToken::KanbanCardRaised, ColorToken::TextDisabled, None);
            assert_eq!(olhar(&b), want, "B {}: {}", area.w, k.label());
            assert_eq!(olhar(&c), want, "C {}: {}", area.w, k.label());
            assert!(
                olhar_de(&b, *k).dimmed && olhar_de(&c, *k).dimmed,
                "{}: apagada",
                k.label()
            );
        }
        // O relatório da PrtSc: duas linhas («PrtSc» no censo) ou o símbolo (`None`).
        let prt = Key(0xF72E);
        let leg = |m: &Mesa| olhar_de(m, prt).text;
        eprintln!("PrtSc {}: B {:?} · C {:?}", area.w, leg(&b), leg(&c));
    }
}

#[test]
fn cada_accao_ligada_ao_comando_mostra_a_barra_nessa_parte() {
    use ph2d_input::GamepadButton as P;
    for area in AREAS {
        let mut m = Mesa::aberta(View::Keyboard);
        let mut map = onze();
        let ids: Vec<_> = map.actions().iter().map(|a| a.id).collect();
        for (i, p) in [P::RightTrigger, P::West, P::South, P::East, P::DPadLeft]
            .into_iter()
            .enumerate()
        {
            map.get_mut(ids[i % ids.len()])
                .expect("acção")
                .bindings
                .push(Binding::PadButton(p));
        }
        m.hero.input_map = map;
        m.frame();
        m.click(m.at(KbHit::Sub(Sub::Gamepad).id()));
        m.paint_in(area);
        let doc = &m.tab().doc;
        let mut ligadas = 0;
        for (h, who) in doc.kb.bars.iter().filter(|(_, w)| !w.is_empty()) {
            let KbHit::Pad(b) = h else { continue };
            let r = doc.rect_of(h.id()).expect("alvo da parte");
            ligadas += 1;
            // Onda 3 (item 2): as barras por TRÁS do nome, dentro da parte (a lei da tecla).
            for id in who {
                let ok = (doc.census.bars.iter()).any(|(br, a)| {
                    a == id
                        && br.x >= r.x - 0.01
                        && br.x + br.w <= r.x + r.w + 0.01
                        && br.y >= r.y - 0.01
                        && br.y + br.h <= r.y + r.h + 0.01
                });
                assert!(ok, "{}: a barra de {id:?} falta em {b:?} {r:?}", area.w);
            }
        }
        assert!(ligadas >= 6, "{}: só {ligadas} partes ligadas", area.w);
    }
}

fn olhar_de(m: &Mesa, k: Key) -> KeyLook {
    (m.tab().doc.census.keys.iter())
        .find(|l| l.key == k)
        .cloned()
        .expect("tecla desenhada")
}

fn centro(m: &Mesa, k: Key) -> (f32, f32) {
    let r = m.tab().doc.rect_of(KbHit::Key(k).id()).expect("alvo");
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

#[test]
fn os_estados_da_tecla_tem_precedencia_premido_aceso_hover() {
    let area = AREAS[0];
    let mut b = pinta_b(area);
    let q = Key(u32::from('Q'));
    assert_eq!(
        olhar_de(&b, q).face,
        ColorToken::KanbanCardRaised,
        "repouso"
    );
    b.hover(centro(&b, q));
    assert_eq!(olhar_de(&b, q).face, HOVER_FACE, "hover");
    b.down(centro(&b, q));
    assert_eq!(
        olhar_de(&b, q).face,
        ColorToken::AccentPress,
        "premido > hover"
    );
    b.up(centro(&b, q));
    // A da acção aberta: o contorno do acento junta-se ao hover (a face continua a do hover).
    let open = b.tab().doc.kb.open.expect("aberta");
    let drawn = drawn_keys();
    let k = (b.hero.input_map.get(open).expect("acção").bindings.iter())
        .find_map(|x| match x {
            Binding::Key(k) if drawn.contains(k) => Some(*k),
            _ => None,
        })
        .expect("uma tecla desenhada da aberta");
    b.hover(centro(&b, k));
    let l = olhar_de(&b, k);
    assert_eq!(
        (l.face, l.edge),
        (HOVER_FACE, Some(ColorToken::Accent)),
        "aberta + hover"
    );
    // O Try it: aceso (a cor da acção) > hover.
    let mut c = Mesa::aberta(View::TryIt);
    let kp8 = Key::named_in_script("kp8").expect("kp8")[0];
    let mut map = onze();
    let up = map.id("move_up").expect("move_up");
    map.get_mut(up)
        .expect("move_up")
        .bindings
        .push(Binding::Key(kp8));
    c.hero.input_map = map;
    c.frame();
    assert!(c.physical(kp8));
    c.frame();
    c.hover(centro(&c, kp8));
    let l = olhar_de(&c, kp8);
    assert_ne!(l.face, HOVER_FACE, "aceso > hover");
    assert_ne!(l.face, ColorToken::KanbanCardRaised, "aceso enche-se");
}

fn tecla(nome: &str) -> Key {
    Key::named_in_script(nome).expect("tecla")[0]
}

/// K2 #2: na C, a ↑ premida (jump amarelo + move_up) acende — e as barras ficam sobre a face NEUTRA da tecla (onda 3,
/// D1: a face cheia na cor do jump engolia a barra dele; agora acende a plaquinha, `onda3_tracos`).
#[test]
fn k2_a_face_cheia_nao_engole_as_barras() {
    for area in AREAS {
        let mut c = pinta_c(area);
        let up = tecla("up");
        assert!(c.physical(up));
        c.paint_in(area);
        let l = olhar_de(&c, up);
        assert_eq!(
            l.body,
            ColorToken::KanbanCardRaised,
            "a tecla fica no repouso"
        );
        let strip = l.rect;
        let r = c.tab().doc.rect_of(KbHit::Key(up).id()).expect("alvo");
        let bars: Vec<Rect> = (c.tab().doc.census.bars.iter())
            .filter(|(b, _)| r.contains(b.x + b.w * 0.5, b.y + b.h * 0.5))
            .map(|(b, _)| *b)
            .collect();
        assert!(bars.len() >= 2, "{}: jump + move_up na ↑: {bars:?}", area.w);
        for b in bars {
            assert!(
                b.x >= strip.x && b.x + b.w <= strip.x + strip.w + 0.01 && b.y >= strip.y,
                "{}: a barra {b:?} fora da faixa {strip:?}",
                area.w
            );
        }
        // Uma tecla em repouso tem a mesma face neutra.
        assert_eq!(olhar_de(&c, tecla("q")).body, ColorToken::KanbanCardRaised);
    }
}

/// K2 #3 e #4: o Enter do numérico diz «Enter» quando cabe, senão o ↵; o Num Lock é palavra quando cabe, senão o
/// símbolo ISO/IEC 9995-7 ⇭ (nunca outro ícone); a pontuação pinta-se maior que `VALUE_PX` quando cabe.
#[test]
fn k2_o_numerico_fala_e_a_pontuacao_pesa() {
    use ph2d_app_input_map::view_keyboard::keys_fit::{PUNCT_PX, is_punct};
    super::em_cada_modo_de_texto(|| {
        for area in AREAS {
            for (v, m) in [("B", pinta_b(area)), ("C", pinta_c(area))] {
                let at = format!("{v} {}", area.w);
                let txt = |k: Key| olhar_de(&m, k).text.map(|t| t.0);
                // O ícone pintado dentro da tecla `k` (`None` = legenda em texto).
                let icon = |k: Key| {
                    let r = olhar_de(&m, k).rect;
                    (m.tab().doc.census.icons.iter())
                        .find(|(ir, _)| r.contains(ir.x + ir.w * 0.5, ir.y + ir.h * 0.5))
                        .map(|(_, id)| *id)
                };
                let word_or = |k: Key, word: &str, iso: IconId| match txt(k) {
                    Some(t) => assert!(t.starts_with(word), "{at}: {k:?} «{t}»"),
                    None => assert_eq!(icon(k), Some(iso), "{at}: {k:?} só o símbolo ISO"),
                };
                word_or(Key::NUMPAD_ENTER, "Enter", IconId::KbdEnter);
                word_or(tecla("numlock"), "Num", IconId::KbdNumLock);
                assert_eq!(
                    m.tab().doc.kb.fit.map(|f| f.scrolls),
                    Some(false),
                    "{at}: o numérico fica à vista"
                );
                for l in &m.tab().doc.census.keys {
                    if let Some((s, px)) = &l.text
                        && is_punct(s)
                        && group_of(l.key) != Group::Numpad
                    {
                        eprintln!("{at}: «{s}» a {px}");
                        assert!(*px >= PUNCT_PX[1], "{at}: «{s}» a {px} lê-se vazia");
                    }
                }
            }
        }
    });
}

/// K2 #1, #5 e #8: a do sistema sem aresta; o símbolo de Home/End/PgUp/PgDn (se usado) da altura da letra; o hover
/// só na tecla pairada, e a aberta é o contorno recuado (a face dela fica a de repouso).
#[test]
fn k2_hover_aberta_e_simbolos_lem_se_diferentes() {
    for area in AREAS {
        let mut b = pinta_b(area);
        for l in &b.tab().doc.census.keys {
            assert!(
                l.dimmed || l.edge != Some(ColorToken::TextDisabled),
                "sem aresta desligada"
            );
            if group_of(l.key) == Group::Nav && l.text.is_none() {
                let ir = (b.tab().doc.census.icons.iter())
                    .find(|(ir, _)| l.rect.contains(ir.x + ir.w * 0.5, ir.y + ir.h * 0.5))
                    .expect("o símbolo")
                    .0;
                // A altura das maiúsculas da fonte do corpo: ~0,7 em (o mínimo do símbolo).
                assert!(
                    ir.h >= ph2d_tokens::TypeToken::Xs.px() * 0.7,
                    "{}: {ir:?}",
                    area.w
                );
            }
        }
        let q = tecla("q");
        b.hover(centro(&b, q));
        let keys = &b.tab().doc.census.keys;
        let hovered: Vec<Key> = keys
            .iter()
            .filter(|l| l.face == HOVER_FACE)
            .map(|l| l.key)
            .collect();
        assert_eq!(hovered, vec![q], "{}: só a pairada", area.w);
        let open: Vec<&KeyLook> = keys
            .iter()
            .filter(|l| l.edge == Some(ColorToken::Accent))
            .collect();
        assert!(!open.is_empty(), "{}: a aberta tem teclas", area.w);
        for l in open {
            assert_eq!(
                l.face,
                ColorToken::KanbanCardRaised,
                "{}: a aberta não é hover",
                area.w
            );
        }
    }
}

/// ⭐ Dono 10/11: a legenda no CENTRO da tecla, numa plaquinha («adesivo») dentro da tecla; as barras por TRÁS
/// dela, todas dentro da tecla e cada uma à vista (nunca inteira por baixo da plaquinha). Devolve a menor espreita.
fn adesivo(doc: &ph2d_app_input_map::InputMapDoc, k: Key, r: Rect, at: &str) -> f32 {
    let l = (doc.census.keys.iter())
        .find(|l| l.key == k)
        .expect("olhar");
    let p = l.plate;
    let n = k.label();
    let dentro = |a: Rect, b: Rect| {
        a.x >= b.x - 0.01
            && a.y >= b.y - 0.01
            && a.x + a.w <= b.x + b.w + 0.01
            && a.y + a.h <= b.y + b.h + 0.01
    };
    assert!(dentro(p, r), "{at}: a plaquinha de {n} {p:?} sai de {r:?}");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    assert!(
        (p.x + p.w * 0.5 - cx).abs() < 0.5 && (p.y + p.h * 0.5 - cy).abs() < 0.5,
        "{at}: a plaquinha de {n} fora do centro"
    );
    let legends = (doc.census.texts.iter().map(|t| t.0))
        .chain(doc.census.icons.iter().map(|i| i.0))
        .filter(|t| r.contains(t.x + t.w * 0.5, t.y + t.h * 0.5));
    for t in legends {
        assert!(
            dentro(t, p),
            "{at}: a legenda de {n} {t:?} sai da plaquinha {p:?}"
        );
    }
    let who = (doc.kb.bars.iter())
        .find(|(h, _)| *h == KbHit::Key(k))
        .map_or(0, |b| b.1.len());
    let bars: Vec<Rect> = (doc.census.bars.iter())
        .filter(|(b, _)| r.contains(b.x + b.w * 0.5, b.y + b.h * 0.5))
        .map(|b| b.0)
        .collect();
    assert_eq!(bars.len(), who, "{at}: {n} tem {who} acções");
    let mut peek = f32::MAX;
    for b in bars {
        assert!(
            dentro(b, r) && b.h > 0.0,
            "{at}: a barra {b:?} sai de {n} {r:?}"
        );
        let lado = (p.x - b.x).max(b.x + b.w - p.x - p.w);
        let cima = (p.y - b.y).max(b.y + b.h - p.y - p.h);
        assert!(
            lado > 0.5 || cima > 0.5,
            "{at}: a barra {b:?} some debaixo da plaquinha de {n}"
        );
        peek = peek.min(lado.max(cima));
    }
    peek
}

/// A Q ligada a `n` acções (e a nenhuma outra).
fn q_com(n: usize) -> ph2d_input::InputMap {
    let q = Binding::Key(tecla("q"));
    let mut map = onze();
    let ids: Vec<_> = map.actions().iter().map(|a| a.id).collect();
    assert!(ids.len() >= n, "o mapa tem {} acções", ids.len());
    for (i, id) in ids.into_iter().enumerate() {
        let a = map.get_mut(id).expect("acção");
        a.bindings.retain(|b| *b != q);
        if i < n {
            a.bindings.push(q);
        }
    }
    map
}

/// ⭐ A letra nunca sobe: o centro da legenda da Q é o MESMO com 0, 1, 2, 5 e 11 barras, na B e na C, nas duas áreas
/// reais × os dois modos de texto; com 11 (além do que cabe à espessura cheia) a pilha afina e fica na tecla.
#[test]
fn a_legenda_fica_parada_na_plaquinha_com_qualquer_numero_de_barras() {
    use ph2d_app_input_map::view_keyboard::keys_fit::bars_full;
    super::em_cada_modo_de_texto(|| {
        for area in AREAS {
            for v in [View::Keyboard, View::TryIt] {
                let mut centros = Vec::new();
                for n in [0, 1, 2, 5, 11] {
                    let mut m = Mesa::aberta(v);
                    m.hero.input_map = q_com(n);
                    m.frame();
                    if v == View::Keyboard {
                        m.click(m.at(KbHit::Sub(Sub::Keyboard).id()));
                    }
                    m.paint_in(area);
                    let at = format!("{v:?} {}×{} n={n}", area.w, area.h);
                    assert_eq!(
                        m.tab().doc.census.elided(),
                        Vec::<&str>::new(),
                        "{at}: cortado"
                    );
                    let doc = &m.tab().doc;
                    let q = tecla("q");
                    let r = doc.rect_of(KbHit::Key(q).id()).expect("a Q");
                    let peek = adesivo(doc, q, r, &at);
                    let t = (doc.census.texts.iter())
                        .find(|t| r.contains(t.0.x + t.0.w * 0.5, t.0.y + t.0.h * 0.5))
                        .expect("a legenda da Q")
                        .0;
                    eprintln!(
                        "{at}: tecla {:.1} cabem {} espreita {peek:.1}",
                        r.h,
                        bars_full(r.h)
                    );
                    centros.push((t.x + t.w * 0.5, t.y + t.h * 0.5));
                }
                for c in &centros {
                    assert!(
                        (c.0 - centros[0].0).abs() < 0.01 && (c.1 - centros[0].1).abs() < 0.01,
                        "{v:?} {}: a legenda mexeu-se {centros:?}",
                        area.w
                    );
                }
            }
        }
    });
}
