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
