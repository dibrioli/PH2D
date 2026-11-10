//! Onda 3, itens 1–2: fila F em meia altura; traços por trás no comando e no «Also:».

use super::Mesa;
use super::lista::onze;
use ph2d_app_input_map::View;
use ph2d_app_input_map::view_keyboard::keys_fit::{
    BAR_VISIBLE_MIN, FN_ROW_FACTOR, PITCH_FLOOR, half_row, measured_floor, measured_fn_factor,
};
use ph2d_app_input_map::view_keyboard::state::{KbHit, Sub};
use ph2d_editor_core::zones::Rect;
use ph2d_input::{Binding, GamepadAxis, GamepadButton, Key};

/// As áreas reais da aba (1600×1000 e 1280×720) e uma estreita, abaixo do piso (rola).
const AREAS: [Rect; 3] = [
    Rect::new(308.0, 28.0, 988.0, 890.0), // LITERAL-PX-OK: a área central a 1600×1000
    Rect::new(297.0, 28.0, 690.0, 604.0), // LITERAL-PX-OK: a área central a 1280×720
    Rect::new(297.0, 28.0, 360.0, 604.0), // LITERAL-PX-OK: estreita, abaixo do piso
];

/// `b` ligada às `n` primeiras acções (e a nenhuma outra).
fn liga(mut map: ph2d_input::InputMap, b: Binding, n: usize) -> ph2d_input::InputMap {
    let ids: Vec<_> = map.actions().iter().map(|a| a.id).collect();
    assert!(ids.len() >= n, "o mapa tem {} acções", ids.len());
    for (i, id) in ids.into_iter().enumerate() {
        let a = map.get_mut(id).expect("acção");
        a.bindings.retain(|x| *x != b);
        if i < n {
            a.bindings.push(b);
        }
    }
    map
}

fn pinta(view: View, sub: Sub, map: ph2d_input::InputMap, a: Rect) -> Mesa {
    let mut m = Mesa::aberta(view);
    m.hero.input_map = map;
    m.frame();
    if view == View::Keyboard {
        m.click(m.at(KbHit::Sub(sub).id()));
    }
    m.paint_in(a);
    m
}

fn dentro(a: Rect, b: Rect) -> bool {
    a.x >= b.x - 0.01
        && a.y >= b.y - 0.01
        && a.x + a.w <= b.x + b.w + 0.01
        && a.y + a.h <= b.y + b.h + 0.01
}

fn centro(r: Rect) -> (f32, f32) {
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

/// A legenda (texto ou ícone) pintada com o centro em `r`.
fn legenda(doc: &ph2d_app_input_map::InputMapDoc, r: Rect) -> Option<Rect> {
    (doc.census.texts.iter().map(|t| t.0))
        .chain(doc.census.icons.iter().map(|i| i.0))
        .find(|t| {
            let (cx, cy) = centro(*t);
            r.contains(cx, cy)
        })
}

/// As barras pintadas com o centro em `r`.
fn barras(doc: &ph2d_app_input_map::InputMapDoc, r: Rect) -> Vec<Rect> {
    (doc.census.bars.iter())
        .filter(|(b, _)| {
            let (cx, cy) = centro(*b);
            r.contains(cx, cy)
        })
        .map(|b| b.0)
        .collect()
}

/// ⭐ O fator da fila de cima é o MEDIDO (o maior dos dois modos de texto), e é meia altura (≤ 0,7). O piso de passo
/// continua o medido com a fila nova.
#[test]
fn o_fator_da_fila_f_e_o_medido_nos_dois_modos_de_texto() {
    let mut medidos = Vec::new();
    super::em_cada_modo_de_texto(|| {
        let mut ts = ph2d_text::TextSystem::without_system_fonts();
        let k = measured_fn_factor(&mut ts);
        let piso = measured_floor(&mut ts);
        eprintln!("fator medido {k}; piso medido {piso}");
        assert!(piso <= PITCH_FLOOR, "o piso subiu: {piso}");
        medidos.push((k, piso));
    });
    let k = medidos.iter().map(|m| m.0).fold(0.0, f32::max);
    let piso = medidos.iter().map(|m| m.1).fold(0.0, f32::max);
    assert_eq!(k, FN_ROW_FACTOR, "o FN_ROW_FACTOR é o medido: {medidos:?}");
    assert_eq!(piso, PITCH_FLOOR, "o PITCH_FLOOR é o medido: {medidos:?}");
    assert!(
        FN_ROW_FACTOR <= 0.7,
        "meia altura, como na foto: {FN_ROW_FACTOR}"
    ); // LITERAL-PX-OK: fator
}

/// ⭐ Esc F1–F12 Insert PrtSc Delete e Home End PgUp PgDn em meia altura (foto do dono), nas duas áreas reais e na
/// estreita, na B e na C, nos dois modos de texto: a legenda inteira na plaquinha e as barras por trás, à vista de
/// cada lado; a fila de baixo não encosta.
#[test]
fn a_fila_f_e_a_home_end_tem_meia_altura_e_a_legenda_cabe() {
    let ligadas = [
        (Key(0xF705), 2), // F2
        (Key(0xF706), 5), // F3
        (Key(0xF727), 1), // Insert
        (Key(0xF729), 3), // Home
        (Key(0xF72C), 1), // PgUp
    ];
    let mut map = onze();
    for (k, n) in ligadas {
        if k.system_owned().is_none() {
            map = liga(map, Binding::Key(k), n);
        }
    }
    super::em_cada_modo_de_texto(|| {
        for a in AREAS {
            for view in [View::Keyboard, View::TryIt] {
                let at = format!("{view:?} {}", a.w);
                let m = pinta(view, Sub::Keyboard, map.clone(), a);
                let doc = &m.tab().doc;
                assert!(doc.kb.fit.expect("pintado").legible, "{at}: legível");
                let keys = &doc.census.keys;
                // Nenhuma legenda de tecla cortada (a lista estreita tem a sua própria lei de reticências).
                for t in doc.census.texts.iter().filter(|t| t.2) {
                    let (cx, cy) = centro(t.0);
                    let na_tecla = keys.iter().any(|l| l.rect.contains(cx, cy));
                    assert!(!na_tecla, "{at}: legenda cortada «{}»", t.1);
                }
                let q = keys
                    .iter()
                    .find(|l| l.key == Key(u32::from('Q')))
                    .expect("a Q")
                    .rect;
                let um = keys
                    .iter()
                    .find(|l| l.key == Key(u32::from('1')))
                    .expect("o 1")
                    .rect;
                let f2 = keys.iter().find(|l| l.key == Key(0xF705)).expect("o F2");
                let home = keys.iter().find(|l| l.key == Key(0xF729)).expect("o Home");
                eprintln!(
                    "{at}: passo {:.1} · tecla {:.1} · meia {:.1} · F2 {:?} · Home {:?} · espreita {:.1}",
                    doc.kb.fit.expect("pintado").pitch,
                    q.h,
                    f2.rect.h,
                    f2.text,
                    home.text,
                    (f2.plate.x - f2.rect.x).min(f2.rect.x + f2.rect.w - f2.plate.x - f2.plate.w),
                );
                let mut meias = 0;
                for l in keys.iter().filter(|l| half_row(l.key)) {
                    meias += 1;
                    let (r, n) = (l.rect, l.key.label());
                    assert!(
                        (r.h - q.h * FN_ROW_FACTOR).abs() < 0.05 && r.h < q.h * 0.75, // LITERAL-PX-OK: meia altura
                        "{at}: {n} tem {:.1} (a Q {:.1})",
                        r.h,
                        q.h
                    );
                    assert!(r.y + r.h < um.y, "{at}: {n} encosta na fila de baixo");
                    assert!(dentro(l.plate, r), "{at}: a plaquinha de {n} sai da tecla");
                    let lg = legenda(doc, r).expect("a legenda");
                    assert!(
                        dentro(lg, l.plate),
                        "{at}: a legenda de {n} {lg:?} sai da plaquinha {:?}",
                        l.plate
                    );
                    let who = (doc.kb.bars.iter())
                        .find(|(h, _)| *h == KbHit::Key(l.key))
                        .map_or(0, |b| b.1.len());
                    let bs = barras(doc, r);
                    assert_eq!(bs.len(), who, "{at}: {n} tem {who} acções");
                    for b in bs {
                        assert!(dentro(b, r) && b.h > 0.0, "{at}: a barra {b:?} sai de {n}");
                        let lado = (l.plate.x - b.x).min(b.x + b.w - l.plate.x - l.plate.w);
                        assert!(
                            lado >= BAR_VISIBLE_MIN - 0.01,
                            "{at}: {n} espreita {lado:.2}"
                        );
                    }
                }
                assert_eq!(meias, 20, "{at}: 16 da fila F + 4 do numérico"); // LITERAL-PX-OK: contagem
            }
        }
    });
}

/// Um alvo do comando ou um chip do «Also:» ligado a `ns` acções: o nome PARADO no mesmo centro, as barras
/// todas dentro do alvo (por trás), cada uma à vista de cada lado do nome.
fn nome_parado(view_sub: Sub, b: Binding, areas: &[Rect], ns: &[usize]) {
    super::em_cada_modo_de_texto(|| {
        for a in areas {
            let mut centros = Vec::new();
            for &n in ns {
                let at = format!("{b:?} {} n={n}", a.w);
                let m = pinta(View::Keyboard, view_sub, liga(onze(), b, n), *a);
                let doc = &m.tab().doc;
                let hit = match b {
                    Binding::Key(k) => KbHit::Key(k),
                    other => KbHit::Pad(other),
                };
                let r = doc.rect_of(hit.id()).expect("o alvo pintado");
                let lg = legenda(doc, r).unwrap_or_else(|| panic!("{at}: a legenda"));
                centros.push(centro(lg));
                let bs = barras(doc, r);
                let todas = doc.census.bars.len();
                assert_eq!(
                    bs.len(),
                    n,
                    "{at}: {n} barras por trás do alvo (pintadas {todas})"
                );
                for bar in bs {
                    assert!(
                        dentro(bar, r) && bar.h > 0.0,
                        "{at}: a barra {bar:?} sai de {r:?}"
                    );
                    let lado = (lg.x - bar.x).min(bar.x + bar.w - lg.x - lg.w);
                    assert!(lado >= BAR_VISIBLE_MIN - 0.01, "{at}: espreita {lado:.2}");
                }
            }
            for c in &centros {
                assert!(
                    (c.0 - centros[0].0).abs() < 0.01 && (c.1 - centros[0].1).abs() < 0.01,
                    "{b:?} {}: o nome mexeu-se {centros:?}",
                    a.w
                );
            }
        }
    });
}

/// ⭐ Item 2: o comando segue a lei da tecla — redondo (A), pílula (LB), braço da cruz e metade de manche.
#[test]
fn o_comando_poe_o_nome_numa_plaquinha_e_as_barras_por_tras() {
    for b in [
        Binding::PadButton(GamepadButton::South),
        Binding::PadButton(GamepadButton::LeftBumper),
        Binding::PadButton(GamepadButton::DPadUp),
        Binding::PadAxis {
            axis: GamepadAxis::RightStickX,
            positive: true,
        },
    ] {
        nome_parado(Sub::Gamepad, b, &AREAS[..2], &[0, 1, 2, 5]);
    }
}

/// ⭐ Item 2: o chip do «Also:» (teclado e comando) segue a lei da tecla.
#[test]
fn o_chip_do_also_poe_o_nome_numa_plaquinha_e_as_barras_por_tras() {
    // Sem acção o chip não existe: de 1 a 5.
    nome_parado(
        Sub::Keyboard,
        Binding::Key(Key(0xE000)),
        &AREAS[..2],
        &[1, 2, 5],
    );
    nome_parado(
        Sub::Gamepad,
        Binding::PadButton(GamepadButton::Mode),
        &AREAS[..2],
        &[1, 2, 5],
    );
}

// ── Rodada 2 da onda 3 (revisão visual): D1 tecla apertada, D2 plaquinha sem acção, D3 seta da manche ──

use ph2d_app_input_map::census::KeyLook;
use ph2d_tokens::{ColorToken, Theme};

fn olhar(m: &Mesa, k: Key) -> KeyLook {
    (m.tab().doc.census.keys.iter())
        .find(|l| l.key == k)
        .cloned()
        .expect("tecla desenhada")
}

const UP: Key = Key(0xF700);

/// O contraste de cada barra de `k` com o que está por trás dela (a face da tecla), em todo tema, e o da mesma
/// barra numa tecla em repouso. Devolve `(menor agora, menor em repouso)`.
fn contraste_das_barras(m: &Mesa, k: Key, at: &str) -> (f64, f64) {
    let doc = &m.tab().doc;
    let l = olhar(m, k);
    let r = l.rect;
    let mut pior = (f64::MAX, f64::MAX);
    for (b, id) in &doc.census.bars {
        if !r.contains(b.x + b.w * 0.5, b.y + b.h * 0.5) {
            continue;
        }
        let cor = ph2d_tokens::categorical(id.0);
        for theme in Theme::ALL {
            let c = cor.resolve(theme);
            let agora = c.contrast_ratio(&l.body.resolve(theme));
            let repouso = c.contrast_ratio(&ColorToken::KanbanCardRaised.resolve(theme));
            let antes = c.contrast_ratio(&l.face.resolve(theme));
            eprintln!(
                "{at} {theme:?} {cor:?}: atrás {agora:.2} · repouso {repouso:.2} · na face cheia {antes:.2}"
            );
            assert!(
                agora >= repouso.min(3.0) - 1e-6,
                "{at} {theme:?}: a barra {cor:?} some ({agora:.2} < {repouso:.2})"
            );
            pior = (pior.0.min(agora), pior.1.min(repouso));
        }
    }
    pior
}

/// ⭐ D1: a ↑ apertada no Try it (jump + move_up) acende a PLAQUINHA na cor da acção; a tecla fica neutra e toda
/// barra continua distinguível do que está por trás dela (≥ o contraste em repouso, ou ≥ 3:1). O premido do rato
/// na B segue a mesma lei.
#[test]
fn a_tecla_apertada_acende_a_plaquinha_e_nunca_engole_as_barras() {
    for a in &AREAS[..2] {
        let mut m = pinta(View::TryIt, Sub::Keyboard, onze(), *a);
        assert!(m.physical(UP), "a ↑ apertada");
        m.paint_in(*a);
        let at = format!("C {}", a.w);
        let l = olhar(&m, UP);
        let jump = m.hero.input_map.id("jump").expect("jump");
        assert_eq!(
            l.face,
            ph2d_tokens::categorical(jump.0),
            "{at}: aceso na cor do jump"
        );
        assert_eq!(
            l.body,
            ColorToken::KanbanCardRaised,
            "{at}: a tecla fica neutra"
        );
        assert_eq!(l.plate_face, Some(l.face), "{at}: a plaquinha acende");
        assert_eq!(
            l.ink,
            ColorToken::Bg0,
            "{at}: o nome lê-se na plaquinha acesa"
        );
        let (agora, repouso) = contraste_das_barras(&m, UP, &at);
        eprintln!("{at}: pior barra {agora:.2} (repouso {repouso:.2})");

        // O premido do rato na B: a tecla neutra, a plaquinha no AccentPress.
        let mut b = pinta(View::Keyboard, Sub::Keyboard, onze(), *a);
        let q = Key(u32::from('Q'));
        let c = b.at(KbHit::Key(q).id());
        b.down(c);
        b.paint_in(*a);
        let at = format!("B {}", a.w);
        let l = olhar(&b, q);
        assert_eq!(l.face, ColorToken::AccentPress, "{at}: premida");
        assert_ne!(l.body, ColorToken::AccentPress, "{at}: a tecla fica neutra");
        assert_eq!(
            l.plate_face,
            Some(ColorToken::AccentPress),
            "{at}: a plaquinha acende"
        );
        contraste_das_barras(&b, q, &at);
        b.release();
    }
}

/// ⭐ D2: sem acção (e sem estado), a plaquinha não se pinta — nada de contorno à volta da legenda (o numérico, o
/// Home/End). Com acção, pinta-se.
#[test]
fn sem_accao_a_plaquinha_nao_se_ve() {
    for a in &AREAS[..2] {
        for view in [View::Keyboard, View::TryIt] {
            let m = pinta(view, Sub::Keyboard, onze(), *a);
            let doc = &m.tab().doc;
            let at = format!("{view:?} {}", a.w);
            let (mut com, mut sem) = (0, 0);
            for l in &doc.census.keys {
                let who = (doc.kb.bars.iter())
                    .find(|(h, _)| *h == KbHit::Key(l.key))
                    .map_or(0, |b| b.1.len());
                if who == 0 {
                    sem += 1;
                    assert_eq!(
                        l.plate_face,
                        None,
                        "{at}: {} sem acção mostra a plaquinha",
                        l.key.label()
                    );
                } else {
                    com += 1;
                    assert!(
                        l.plate_face.is_some(),
                        "{at}: {} com acção sem plaquinha",
                        l.key.label()
                    );
                }
            }
            assert!(com > 5 && sem > 50, "{at}: {com} com acção, {sem} sem"); // LITERAL-PX-OK: contagens
        }
    }
}

/// ⭐ D3: a seta da manche ligada (o ← do LS com move_left) lê-se como as outras partes ligadas: o glifo do tamanho
/// do braço da cruz, as barras à vista dos dois lados (≥ 2 px), sem tocar o anel da manche nem sair do corpo, e
/// nenhuma parte do comando sobre outra — nas duas larguras.
#[test]
fn a_seta_da_manche_ligada_le_se_como_as_outras_partes() {
    let ls_left = Binding::PadAxis {
        axis: GamepadAxis::LeftStickX,
        positive: false,
    };
    let dpad_up = Binding::PadButton(GamepadButton::DPadUp);
    for a in &AREAS[..2] {
        let at = format!("{}", a.w);
        let map = liga(liga(onze(), dpad_up, 1), ls_left, 2);
        let m = pinta(View::Keyboard, Sub::Gamepad, map, *a);
        let doc = &m.tab().doc;
        let caixa = doc.kb.pad_box.expect("o comando pintado");
        let icone = |b: Binding| {
            let r = doc.rect_of(KbHit::Pad(b).id()).expect("a parte");
            let i = (doc.census.icons.iter())
                .find(|i| r.contains(i.0.x + i.0.w * 0.5, i.0.y + i.0.h * 0.5))
                .expect("a seta")
                .0;
            (r, i)
        };
        let (r, ir) = icone(ls_left);
        let (_, braco) = icone(dpad_up);
        eprintln!("{at}: seta do LS {ir:?} na parte {r:?}; braço da cruz {braco:?}");
        assert!(
            ir.w >= braco.w - 0.01,
            "{at}: o glifo {} < o do braço {}",
            ir.w,
            braco.w
        );
        let bs = barras(doc, r);
        assert_eq!(bs.len(), 2, "{at}: move_left e a outra por trás da seta");
        for b in bs {
            assert!(dentro(b, r), "{at}: a barra {b:?} sai da parte");
            let lado = (ir.x - b.x).min(b.x + b.w - ir.x - ir.w);
            eprintln!("{at}: barra à vista {lado:.2}");
            assert!(lado >= BAR_VISIBLE_MIN - 0.01, "{at}: espreita {lado:.2}");
        }
        // O anel do LS: a legenda «LS» pinta-se na altura do anel (2 r) com o centro nele.
        let ls = (doc.census.texts.iter())
            .find(|t| t.1 == "LS")
            .expect("o LS")
            .0;
        let (cx, cy, raio) = (ls.x + ls.w * 0.5, ls.y + ls.h * 0.5, ls.h * 0.5);
        let partes: Vec<Rect> = (doc.kb.bars.iter())
            .filter_map(|(h, _)| doc.rect_of(h.id()))
            .collect();
        for (i, p) in partes.iter().enumerate() {
            for q in &partes[i + 1..] {
                let separadas =
                    p.x + p.w <= q.x || q.x + q.w <= p.x || p.y + p.h <= q.y || q.y + q.h <= p.y;
                assert!(separadas, "{at}: {p:?} sobre {q:?}");
            }
            for (x, y) in [
                (p.x, p.y),
                (p.x + p.w, p.y),
                (p.x, p.y + p.h),
                (p.x + p.w, p.y + p.h),
            ] {
                let fora_do_topo = p.y + p.h <= caixa.y + caixa.h * 0.2; // LITERAL-PX-OK: os gatilhos na aresta
                assert!(
                    fora_do_topo || ph2d_app_input_map::view_keyboard::body_contains(caixa, x, y),
                    "{at}: {p:?} sai do corpo"
                );
            }
        }
        // As quatro setas do LS não tocam o anel (o traço do anel aceso tem 3 px: metade para fora).
        for (h, _) in &doc.kb.bars {
            let KbHit::Pad(Binding::PadAxis { axis, .. }) = h else {
                continue;
            };
            if !matches!(axis, GamepadAxis::LeftStickX | GamepadAxis::LeftStickY) {
                continue;
            }
            let p = doc.rect_of(h.id()).expect("a seta");
            let nx = cx.clamp(p.x, p.x + p.w);
            let ny = cy.clamp(p.y, p.y + p.h);
            let d = ((nx - cx).powi(2) + (ny - cy).powi(2)).sqrt();
            assert!(
                d > raio + 1.5,
                "{at}: {p:?} toca o anel ({d:.1} ≤ {raio:.1})"
            ); // LITERAL-PX-OK: meio traço
        }
    }
}
